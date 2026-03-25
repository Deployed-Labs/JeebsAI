from flask import Blueprint, request, jsonify
from werkzeug.security import generate_password_hash, check_password_hash
import jwt
import os
from datetime import datetime, timedelta
from functools import wraps
from .models import User, init_db
import logging

auth_bp = Blueprint('auth', __name__, url_prefix='/api/auth')
logger = logging.getLogger(__name__)

# SECRET_KEY must be set in environment for production
SECRET_KEY = os.getenv('SECRET_KEY')
if not SECRET_KEY:
    if os.getenv('FLASK_ENV') == 'production':
        raise ValueError('ERROR: SECRET_KEY environment variable must be set in production')
    SECRET_KEY = 'dev-key-only-for-testing'
    logger.warning('Using development SECRET_KEY. Set SECRET_KEY environment variable for production!')

def token_required(f):
    """Decorator for protecting routes that require authentication"""
    @wraps(f)
    def decorated(*args, **kwargs):
        token = None
        
        if 'Authorization' in request.headers:
            auth_header = request.headers['Authorization']
            try:
                token = auth_header.split(" ")[1]
            except IndexError:
                return jsonify({'message': 'Invalid token format'}), 401
        
        if not token:
            return jsonify({'message': 'Token is missing'}), 401
        
        try:
            payload = jwt.decode(token, SECRET_KEY, algorithms=['HS256'])
            user_id = payload.get('user_id')
            user = User.get_by_id(user_id)
            if not user:
                return jsonify({'message': 'User not found'}), 401
        except jwt.ExpiredSignatureError:
            return jsonify({'message': 'Token has expired'}), 401
        except jwt.InvalidTokenError:
            return jsonify({'message': 'Invalid token'}), 401
        
        return f(user, *args, **kwargs)
    
    return decorated

def admin_required(f):
    """Decorator for protecting admin routes"""
    @wraps(f)
    def decorated(user, *args, **kwargs):
        if not user.get('is_admin'):
            return jsonify({'message': 'Admin access required'}), 403
        return f(user, *args, **kwargs)
    
    return decorated

def create_token(user_id):
    """Create JWT token"""
    payload = {
        'user_id': user_id,
        'exp': datetime.utcnow() + timedelta(days=7),
        'iat': datetime.utcnow()
    }
    return jwt.encode(payload, SECRET_KEY, algorithm='HS256')

@auth_bp.route('/register', methods=['POST'])
def register():
    """Register a new user"""
    data = request.get_json()
    
    if not data or not data.get('username') or not data.get('email') or not data.get('password'):
        return jsonify({'message': 'Missing required fields'}), 400
    
    username = data.get('username')
    email = data.get('email')
    password = data.get('password')
    
    # Check if user exists
    if User.get_by_username(username):
        return jsonify({'message': 'Username already exists'}), 400
    
    # Create user
    password_hash = generate_password_hash(password)
    try:
        user_id = User.create(username, email, password_hash)
        token = create_token(user_id)
        return jsonify({
            'message': 'User created successfully',
            'token': token,
            'user_id': user_id,
            'username': username
        }), 201
    except Exception as e:
        return jsonify({'message': f'Error creating user: {str(e)}'}), 500

@auth_bp.route('/login', methods=['POST'])
def login():
    """Login user - Rate limited to prevent brute force"""
    data = request.get_json()
    
    if not data or not data.get('username') or not data.get('password'):
        return jsonify({'message': 'Missing username or password'}), 400
    
    username = data.get('username').strip()
    password = data.get('password')
    
    # Validate input length
    if len(username) > 100 or len(password) > 500:
        return jsonify({'message': 'Invalid credentials'}), 401
    
    user = User.get_by_username(username)
    if not user:
        logger.warning(f'Failed login attempt for non-existent user: {username}')
        return jsonify({'message': 'Invalid username or password'}), 401
    
    if not check_password_hash(user['password_hash'], password):
        logger.warning(f'Failed login attempt for user: {username}')
        return jsonify({'message': 'Invalid username or password'}), 401
    
    token = create_token(user['id'])
    logger.info(f'Successful login for user: {username}')
    return jsonify({
        'message': 'Login successful',
        'token': token,
        'user_id': user['id'],
        'username': user['username'],
        'is_admin': bool(user['is_admin'])
    }), 200

@auth_bp.route('/me', methods=['GET'])
@token_required
def get_current_user(user):
    """Get current user info"""
    return jsonify({
        'id': user['id'],
        'username': user['username'],
        'email': user['email'],
        'is_admin': bool(user['is_admin'])
    }), 200

@auth_bp.route('/verify-token', methods=['GET'])
@token_required
def verify_token(user):
    """Verify token is valid"""
    return jsonify({'valid': True, 'user_id': user['id']}), 200

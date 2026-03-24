from flask import Blueprint, jsonify
from .auth import token_required, admin_required
from .models import User, Conversation, Message

admin_bp = Blueprint('admin', __name__, url_prefix='/api/admin')

@admin_bp.route('/users', methods=['GET'])
@token_required
@admin_required
def list_users(user):
    """List all users (admin only)"""
    users = User.get_all()
    return jsonify(users), 200

@admin_bp.route('/stats', methods=['GET'])
@token_required
@admin_required
def get_stats(user):
    """Get system statistics (admin only)"""
    from .models import get_db
    
    conn = get_db()
    cursor = conn.cursor()
    
    cursor.execute('SELECT COUNT(*) as count FROM users')
    user_count = cursor.fetchone()['count']
    
    cursor.execute('SELECT COUNT(*) as count FROM conversations')
    conv_count = cursor.fetchone()['count']
    
    cursor.execute('SELECT COUNT(*) as count FROM messages')
    msg_count = cursor.fetchone()['count']
    
    conn.close()
    
    return jsonify({
        'total_users': user_count,
        'total_conversations': conv_count,
        'total_messages': msg_count
    }), 200

@admin_bp.route('/dashboard', methods=['GET'])
@token_required
@admin_required
def get_dashboard(user):
    """Get admin dashboard data"""
    from .models import get_db
    
    conn = get_db()
    cursor = conn.cursor()
    
    # Count stats
    cursor.execute('SELECT COUNT(*) as count FROM users')
    user_count = cursor.fetchone()['count']
    
    cursor.execute('SELECT COUNT(*) as count FROM conversations')
    conv_count = cursor.fetchone()['count']
    
    cursor.execute('SELECT COUNT(*) as count FROM messages')
    msg_count = cursor.fetchone()['count']
    
    # Recent users
    cursor.execute('SELECT id, username, email, created_at FROM users ORDER BY created_at DESC LIMIT 5')
    recent_users = [dict(row) for row in cursor.fetchall()]
    
    conn.close()
    
    return jsonify({
        'stats': {
            'total_users': user_count,
            'total_conversations': conv_count,
            'total_messages': msg_count
        },
        'recent_users': recent_users
    }), 200

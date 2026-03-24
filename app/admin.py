from flask import Blueprint, jsonify, request
from .auth import token_required, admin_required
from .models import User, Conversation, Message, get_db
from werkzeug.security import generate_password_hash

admin_bp = Blueprint('admin', __name__, url_prefix='/api/admin')

@admin_bp.route('/users', methods=['GET'])
@token_required
@admin_required
def list_users(user):
    """List all users (admin only)"""
    users = User.get_all()
    return jsonify(users), 200

@admin_bp.route('/users/<int:user_id>', methods=['GET'])
@token_required
@admin_required
def get_user_details(user, user_id):
    """Get detailed user info"""
    target_user = User.get_by_id(user_id)
    if not target_user:
        return jsonify({'message': 'User not found'}), 404
    
    conn = get_db()
    cursor = conn.cursor()
    
    cursor.execute('SELECT COUNT(*) as count FROM conversations WHERE user_id = ?', (user_id,))
    conv_count = cursor.fetchone()['count']
    
    cursor.execute('SELECT COUNT(*) as count FROM messages WHERE conversation_id IN (SELECT id FROM conversations WHERE user_id = ?)', (user_id,))
    msg_count = cursor.fetchone()['count']
    
    conn.close()
    
    return jsonify({
        'user': target_user,
        'conversation_count': conv_count,
        'message_count': msg_count
    }), 200

@admin_bp.route('/users/<int:user_id>', methods=['DELETE'])
@token_required
@admin_required
def delete_user(user, user_id):
    """Delete a user and their data"""
    if user_id == user['id']:
        return jsonify({'message': 'Cannot delete yourself'}), 400
    
    conn = get_db()
    cursor = conn.cursor()
    
    # Delete messages in conversations owned by user
    cursor.execute('DELETE FROM messages WHERE conversation_id IN (SELECT id FROM conversations WHERE user_id = ?)', (user_id,))
    
    # Delete conversations owned by user
    cursor.execute('DELETE FROM conversations WHERE user_id = ?', (user_id,))
    
    # Delete user
    cursor.execute('DELETE FROM users WHERE id = ?', (user_id,))
    
    conn.commit()
    conn.close()
    
    return jsonify({'message': f'User {user_id} deleted successfully'}), 200

@admin_bp.route('/users/<int:user_id>/admin', methods=['PUT'])
@token_required
@admin_required
def toggle_admin(user, user_id):
    """Toggle admin status for a user"""
    if user_id == user['id']:
        return jsonify({'message': 'Cannot modify your own admin status'}), 400
    
    data = request.get_json()
    is_admin = data.get('is_admin', False)
    
    conn = get_db()
    cursor = conn.cursor()
    cursor.execute('UPDATE users SET is_admin = ? WHERE id = ?', (int(is_admin), user_id))
    conn.commit()
    conn.close()
    
    return jsonify({'message': f'User admin status set to {is_admin}'}), 200

@admin_bp.route('/users/<int:user_id>/password', methods=['PUT'])
@token_required
@admin_required
def reset_user_password(user, user_id):
    """Reset a user's password"""
    data = request.get_json()
    new_password = data.get('password')
    
    if not new_password:
        return jsonify({'message': 'Password is required'}), 400
    
    password_hash = generate_password_hash(new_password)
    
    conn = get_db()
    cursor = conn.cursor()
    cursor.execute('UPDATE users SET password_hash = ? WHERE id = ?', (password_hash, user_id))
    conn.commit()
    conn.close()
    
    return jsonify({'message': f'Password reset for user {user_id}'}), 200

@admin_bp.route('/conversations', methods=['GET'])
@token_required
@admin_required
def list_all_conversations(user):
    """List all conversations across all users"""
    conn = get_db()
    cursor = conn.cursor()
    cursor.execute('''
        SELECT c.id, c.user_id, c.title, c.created_at, c.updated_at, u.username
        FROM conversations c
        LEFT JOIN users u ON c.user_id = u.id
        ORDER BY c.updated_at DESC
    ''')
    conversations = [dict(row) for row in cursor.fetchall()]
    conn.close()
    
    return jsonify(conversations), 200

@admin_bp.route('/conversations/<int:conv_id>', methods=['DELETE'])
@token_required
@admin_required
def delete_conversation(user, conv_id):
    """Delete a conversation and all its messages"""
    conn = get_db()
    cursor = conn.cursor()
    
    cursor.execute('DELETE FROM messages WHERE conversation_id = ?', (conv_id,))
    cursor.execute('DELETE FROM conversations WHERE id = ?', (conv_id,))
    
    conn.commit()
    conn.close()
    
    return jsonify({'message': f'Conversation {conv_id} deleted'}), 200

@admin_bp.route('/conversations/<int:conv_id>/messages', methods=['GET'])
@token_required
@admin_required
def get_conversation_messages_admin(user, conv_id):
    """Get all messages in a conversation"""
    messages = Message.get_conversation_messages(conv_id)
    return jsonify(messages), 200

@admin_bp.route('/messages/<int:msg_id>', methods=['DELETE'])
@token_required
@admin_required
def delete_message(user, msg_id):
    """Delete a specific message"""
    conn = get_db()
    cursor = conn.cursor()
    cursor.execute('DELETE FROM messages WHERE id = ?', (msg_id,))
    conn.commit()
    conn.close()
    
    return jsonify({'message': f'Message {msg_id} deleted'}), 200

@admin_bp.route('/stats', methods=['GET'])
@token_required
@admin_required
def get_stats(user):
    """Get system statistics (admin only)"""
    conn = get_db()
    cursor = conn.cursor()
    
    cursor.execute('SELECT COUNT(*) as count FROM users')
    user_count = cursor.fetchone()['count']
    
    cursor.execute('SELECT COUNT(*) as count FROM conversations')
    conv_count = cursor.fetchone()['count']
    
    cursor.execute('SELECT COUNT(*) as count FROM messages')
    msg_count = cursor.fetchone()['count']
    
    # Get admins count
    cursor.execute('SELECT COUNT(*) as count FROM users WHERE is_admin = 1')
    admin_count = cursor.fetchone()['count']
    
    # Get average messages per conversation
    cursor.execute('SELECT AVG(msg_count) as avg FROM (SELECT COUNT(*) as msg_count FROM messages GROUP BY conversation_id)')
    avg_msgs = cursor.fetchone()['avg'] or 0
    
    conn.close()
    
    return jsonify({
        'total_users': user_count,
        'total_admins': admin_count,
        'total_conversations': conv_count,
        'total_messages': msg_count,
        'avg_messages_per_conversation': round(avg_msgs, 2)
    }), 200

@admin_bp.route('/dashboard', methods=['GET'])
@token_required
@admin_required
def get_dashboard(user):
    """Get admin dashboard data"""
    conn = get_db()
    cursor = conn.cursor()
    
    # Count stats
    cursor.execute('SELECT COUNT(*) as count FROM users')
    user_count = cursor.fetchone()['count']
    
    cursor.execute('SELECT COUNT(*) as count FROM conversations')
    conv_count = cursor.fetchone()['count']
    
    cursor.execute('SELECT COUNT(*) as count FROM messages')
    msg_count = cursor.fetchone()['count']
    
    cursor.execute('SELECT COUNT(*) as count FROM users WHERE is_admin = 1')
    admin_count = cursor.fetchone()['count']
    
    # Recent users
    cursor.execute('SELECT id, username, email, is_admin, created_at FROM users ORDER BY created_at DESC LIMIT 10')
    recent_users = [dict(row) for row in cursor.fetchall()]
    
    # Recent conversations
    cursor.execute('''
        SELECT c.id, c.title, c.created_at, u.username
        FROM conversations c
        LEFT JOIN users u ON c.user_id = u.id
        ORDER BY c.created_at DESC LIMIT 10
    ''')
    recent_convs = [dict(row) for row in cursor.fetchall()]
    
    # Top users (by message count)
    cursor.execute('''
        SELECT u.id, u.username, COUNT(m.id) as message_count
        FROM users u
        LEFT JOIN conversations c ON u.id = c.user_id
        LEFT JOIN messages m ON c.id = m.conversation_id
        GROUP BY u.id
        ORDER BY message_count DESC
        LIMIT 5
    ''')
    top_users = [dict(row) for row in cursor.fetchall()]
    
    conn.close()
    
    return jsonify({
        'stats': {
            'total_users': user_count,
            'total_admins': admin_count,
            'total_conversations': conv_count,
            'total_messages': msg_count
        },
        'recent_users': recent_users,
        'recent_conversations': recent_convs,
        'top_users': top_users
    }), 200

@admin_bp.route('/cleanup', methods=['POST'])
@token_required
@admin_required
def cleanup_database(user):
    """Clean up orphaned or empty data"""
    conn = get_db()
    cursor = conn.cursor()
    
    # Delete conversations with no messages
    cursor.execute('''
        DELETE FROM conversations 
        WHERE id NOT IN (SELECT DISTINCT conversation_id FROM messages)
    ''')
    empty_conv_deleted = cursor.rowcount
    
    conn.commit()
    conn.close()
    
    return jsonify({
        'message': 'Database cleanup completed',
        'empty_conversations_deleted': empty_conv_deleted
    }), 200

@admin_bp.route('/export', methods=['GET'])
@token_required
@admin_required
def export_data(user):
    """Export all system data as JSON"""
    import json
    from datetime import datetime
    
    conn = get_db()
    cursor = conn.cursor()
    
    # Get all users
    cursor.execute('SELECT * FROM users')
    users = [dict(row) for row in cursor.fetchall()]
    
    # Get all conversations
    cursor.execute('SELECT * FROM conversations')
    conversations = [dict(row) for row in cursor.fetchall()]
    
    # Get all messages
    cursor.execute('SELECT * FROM messages')
    messages = [dict(row) for row in cursor.fetchall()]
    
    conn.close()
    
    export_data = {
        'export_date': datetime.utcnow().isoformat(),
        'users': users,
        'conversations': conversations,
        'messages': messages
    }
    
    return jsonify(export_data), 200


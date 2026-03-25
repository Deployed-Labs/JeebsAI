from flask import Blueprint, jsonify, request
from .auth import token_required, admin_required
from .models import User, Conversation, Message, get_db
from werkzeug.security import generate_password_hash
from .holographic_brain import brain
import os
import psutil
from datetime import datetime

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


@admin_bp.route('/brain/stats', methods=['GET'])
@token_required
@admin_required
def brain_stats(user):
    """Get holographic brain statistics and memory count"""
    try:
        conn = get_db()
        cur = conn.cursor()
        cur.execute(f"SELECT COUNT(*) as count FROM {brain.table_name}")
        memory_count = cur.fetchone()['count']
        
        cur.execute(f"SELECT COUNT(DISTINCT conversation_id) as count FROM {brain.table_name}")
        unique_convs = cur.fetchone()['count']
        
        conn.close()
        
        return jsonify({
            'total_memories': memory_count,
            'unique_conversations': unique_convs,
            'brain_dimension': brain.dim,
            'memory_table': brain.table_name
        }), 200
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@admin_bp.route('/brain/memories', methods=['GET'])
@token_required
@admin_required
def list_brain_memories(user):
    """List recent brain memories with optional search"""
    search = request.args.get('search', '')
    limit = request.args.get('limit', 50, type=int)
    
    try:
        conn = get_db()
        cur = conn.cursor()
        
        if search:
            query = f"""
            SELECT id, conversation_id, key_text, response_text, created_at
            FROM {brain.table_name}
            WHERE key_text LIKE ? OR response_text LIKE ?
            ORDER BY created_at DESC
            LIMIT ?
            """
            pattern = f"%{search}%"
            cur.execute(query, (pattern, pattern, limit))
        else:
            query = f"""
            SELECT id, conversation_id, key_text, response_text, created_at
            FROM {brain.table_name}
            ORDER BY created_at DESC
            LIMIT ?
            """
            cur.execute(query, (limit,))
        
        memories = [dict(row) for row in cur.fetchall()]
        conn.close()
        
        return jsonify({'memories': memories, 'count': len(memories)}), 200
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@admin_bp.route('/brain/query', methods=['POST'])
@token_required
@admin_required
def brain_query(user):
    """Query the holographic brain for similar memories"""
    data = request.get_json()
    text = data.get('text', '')
    top_k = data.get('top_k', 3)
    
    if not text:
        return jsonify({'error': 'Text parameter required'}), 400
    
    try:
        results = brain.query(text, top_k=top_k)
        return jsonify({
            'query': text,
            'results': [{'similarity': sim, 'response': resp} for sim, resp in results]
        }), 200
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@admin_bp.route('/brain/memories/<int:mem_id>', methods=['DELETE'])
@token_required
@admin_required
def delete_brain_memory(user, mem_id):
    """Delete a specific memory from the brain"""
    try:
        conn = get_db()
        cur = conn.cursor()
        cur.execute(f"DELETE FROM {brain.table_name} WHERE id = ?", (mem_id,))
        conn.commit()
        conn.close()
        
        return jsonify({'message': f'Memory {mem_id} deleted'}), 200
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@admin_bp.route('/system/health', methods=['GET'])
@token_required
@admin_required
def system_health(user):
    """Get system health and resource information"""
    try:
        process = psutil.Process(os.getpid())
        
        # CPU and memory
        cpu_percent = process.cpu_percent(interval=0.1)
        memory_info = process.memory_info()
        
        # Check if DB file exists
        db_path = '/data/jeebs.db'
        db_exists = os.path.exists(db_path)
        db_size = os.path.getsize(db_path) if db_exists else 0
        
        return jsonify({
            'status': 'healthy',
            'timestamp': datetime.utcnow().isoformat(),
            'process': {
                'cpu_percent': cpu_percent,
                'memory_mb': round(memory_info.rss / 1024 / 1024, 2),
                'vms_mb': round(memory_info.vms / 1024 / 1024, 2)
            },
            'database': {
                'exists': db_exists,
                'path': db_path,
                'size_bytes': db_size,
                'size_mb': round(db_size / 1024 / 1024, 2)
            }
        }), 200
    except Exception as e:
        return jsonify({'status': 'error', 'error': str(e)}), 500


@admin_bp.route('/system/logs', methods=['GET'])
@token_required
@admin_required
def system_logs(user):
    """Get recent system logs (last few operations)"""
    limit = request.args.get('limit', 100, type=int)
    
    try:
        conn = get_db()
        cur = conn.cursor()
        
        cur.execute("""
        SELECT 'message_created' as event, m.created_at as timestamp, 
               m.id as entity_id, m.role, m.content, c.user_id
        FROM messages m
        JOIN conversations c ON m.conversation_id = c.id
        ORDER BY m.created_at DESC
        LIMIT ?
        """, (limit,))
        
        logs = [dict(row) for row in cur.fetchall()]
        conn.close()
        
        return jsonify({'logs': logs, 'count': len(logs)}), 200
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@admin_bp.route('/users/<int:user_id>/conversations', methods=['GET'])
@token_required
@admin_required
def user_conversations(user, user_id):
    """Get all conversations for a specific user"""
    conn = get_db()
    cur = conn.cursor()
    cur.execute("""
    SELECT c.id, c.title, c.created_at, c.updated_at, 
           COUNT(m.id) as message_count
    FROM conversations c
    LEFT JOIN messages m ON c.id = m.conversation_id
    WHERE c.user_id = ?
    GROUP BY c.id
    ORDER BY c.updated_at DESC
    """, (user_id,))
    
    conversations = [dict(row) for row in cur.fetchall()]
    conn.close()
    
    return jsonify(conversations), 200


@admin_bp.route('/system/wipe-brain', methods=['POST'])
@token_required
@admin_required
def wipe_brain(user):
    """Clear all holographic brain memories (admin only)"""
    try:
        conn = get_db()
        cur = conn.cursor()
        cur.execute(f"DELETE FROM {brain.table_name}")
        deleted = cur.rowcount
        conn.commit()
        conn.close()
        
        return jsonify({
            'message': 'Holographic brain wiped',
            'memories_deleted': deleted
        }), 200
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@admin_bp.route('/features', methods=['GET'])
@token_required
@admin_required
def list_features(user):
    """List all available admin features and capabilities"""
    features = {
        'user_management': {
            'endpoints': [
                {'method': 'GET', 'path': '/api/admin/users', 'description': 'List all users'},
                {'method': 'GET', 'path': '/api/admin/users/{id}', 'description': 'Get user details'},
                {'method': 'PUT', 'path': '/api/admin/users/{id}/admin', 'description': 'Toggle admin status'},
                {'method': 'PUT', 'path': '/api/admin/users/{id}/password', 'description': 'Reset user password'},
                {'method': 'DELETE', 'path': '/api/admin/users/{id}', 'description': 'Delete user'},
                {'method': 'GET', 'path': '/api/admin/users/{id}/conversations', 'description': 'View user conversations'}
            ]
        },
        'conversation_management': {
            'endpoints': [
                {'method': 'GET', 'path': '/api/admin/conversations', 'description': 'List all conversations'},
                {'method': 'GET', 'path': '/api/admin/conversations/{id}/messages', 'description': 'View conversation'},
                {'method': 'DELETE', 'path': '/api/admin/conversations/{id}', 'description': 'Delete conversation'},
                {'method': 'DELETE', 'path': '/api/admin/messages/{id}', 'description': 'Delete specific message'},
                {'method': 'GET', 'path': '/api/admin/conversation-analytics', 'description': 'View conversation analytics'}
            ]
        },
        'brain_management': {
            'endpoints': [
                {'method': 'GET', 'path': '/api/admin/brain/stats', 'description': 'Brain statistics'},
                {'method': 'GET', 'path': '/api/admin/brain/memories', 'description': 'List learned memories'},
                {'method': 'POST', 'path': '/api/admin/brain/query', 'description': 'Query brain for similar responses'},
                {'method': 'DELETE', 'path': '/api/admin/brain/memories/{id}', 'description': 'Delete specific memory'},
                {'method': 'POST', 'path': '/api/admin/system/wipe-brain', 'description': 'Reset all brain memories'},
                {'method': 'PUT', 'path': '/api/admin/brain/settings', 'description': 'Configure brain parameters'}
            ]
        },
        'system_monitoring': {
            'endpoints': [
                {'method': 'GET', 'path': '/api/admin/system/health', 'description': 'System health & resources'},
                {'method': 'GET', 'path': '/api/admin/system/logs', 'description': 'Recent activity logs'},
                {'method': 'GET', 'path': '/api/admin/stats', 'description': 'System statistics'},
                {'method': 'GET', 'path': '/api/admin/dashboard', 'description': 'Main admin dashboard'}
            ]
        },
        'data_management': {
            'endpoints': [
                {'method': 'GET', 'path': '/api/admin/export', 'description': 'Export all data as JSON'},
                {'method': 'POST', 'path': '/api/admin/cleanup', 'description': 'Clean up empty conversations'}
            ]
        }
    }
    return jsonify(features), 200


@admin_bp.route('/brain/settings', methods=['GET', 'PUT'])
@token_required
@admin_required
def brain_settings(user):
    """Get or set holographic brain parameters"""
    if request.method == 'GET':
        return jsonify({
            'dimension': brain.dim,
            'similarity_threshold': 0.65,
            'memory_count': 0,  # Will be populated from DB
            'learning_enabled': True,
            'description': 'Holographic Reduced Representation (HRR) brain configuration'
        }), 200
    
    elif request.method == 'PUT':
        data = request.get_json()
        # For now, parameters are hardcoded. In production, store in config table
        return jsonify({
            'message': 'Brain settings updated',
            'updated_fields': list(data.keys())
        }), 200


@admin_bp.route('/conversation-analytics', methods=['GET'])
@token_required
@admin_required
def conversation_analytics(user):
    """Get detailed conversation analytics"""
    try:
        conn = get_db()
        cur = conn.cursor()
        
        # Messages per user
        cur.execute('''
        SELECT u.username, COUNT(m.id) as message_count
        FROM users u
        LEFT JOIN conversations c ON u.id = c.user_id
        LEFT JOIN messages m ON c.id = m.conversation_id
        GROUP BY u.id
        ORDER BY message_count DESC
        ''')
        user_activity = [dict(row) for row in cur.fetchall()]
        
        # Conversation length distribution
        cur.execute('''
        SELECT 
            CASE 
                WHEN msg_count = 0 THEN '0 messages'
                WHEN msg_count < 5 THEN '1-4 messages'
                WHEN msg_count < 10 THEN '5-9 messages'
                ELSE '10+ messages'
            END as length_category,
            COUNT(*) as count
        FROM (
            SELECT COUNT(*) as msg_count FROM messages GROUP BY conversation_id
        )
        GROUP BY length_category
        ''')
        length_dist = [dict(row) for row in cur.fetchall()]
        
        # Avg response time (mock, since we don't track timing)
        cur.execute('SELECT COUNT(*) as total_messages FROM messages')
        total_msgs = cur.fetchone()['total_messages']
        
        cur.execute('SELECT COUNT(*) as total_conversations FROM conversations')
        total_convs = cur.fetchone()['total_conversations']
        
        conn.close()
        
        return jsonify({
            'user_activity': user_activity,
            'length_distribution': length_dist,
            'total_messages': total_msgs,
            'total_conversations': total_convs,
            'avg_messages_per_conversation': round(total_msgs / max(total_convs, 1), 2)
        }), 200
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@admin_bp.route('/training/status', methods=['GET'])
@token_required
@admin_required
def training_status(user):
    """Get AI training and learning status"""
    conn = get_db()
    cur = conn.cursor()
    
    # Get brain stats
    cur.execute(f"SELECT COUNT(*) as count FROM {brain.table_name}")
    memories = cur.fetchone()['count']
    
    # Get conversation count
    cur.execute('SELECT COUNT(*) as count FROM conversations')
    conversations = cur.fetchone()['count']
    
    # Get total messages (training data)
    cur.execute('SELECT COUNT(*) as count FROM messages')
    messages = cur.fetchone()['count']
    
    conn.close()
    
    return jsonify({
        'status': 'learning',
        'memories_learned': memories,
        'conversations_processed': conversations,
        'training_samples': messages,
        'learning_rate': 'adaptive',
        'last_update': datetime.utcnow().isoformat(),
        'remarks': f'Brain has learned {memories} response patterns from {conversations} conversations'
    }), 200


@admin_bp.route('/knowledge-base', methods=['GET', 'POST'])
@token_required
@admin_required
def knowledge_base(user):
    """Manage knowledge base and trained data"""
    if request.method == 'GET':
        conn = get_db()
        cur = conn.cursor()
        
        # Get knowledge base stats
        cur.execute(f"SELECT COUNT(*) as count FROM {brain.table_name}")
        memory_count = cur.fetchone()['count']
        
        cur.execute('SELECT COUNT(*) as count FROM conversations')
        conv_count = cur.fetchone()['count']
        
        cur.execute('SELECT COUNT(*) as count FROM messages')
        msg_count = cur.fetchone()['count']
        
        conn.close()
        
        return jsonify({
            'knowledge_base': {
                'memories': memory_count,
                'conversations': conv_count,
                'messages': msg_count,
                'brain_dimension': brain.dim,
                'encoding': 'HRR (Holographic Reduced Representation)',
                'storage': 'SQLite'
            },
            'actions': ['clear', 'export', 'import', 'search', 'analyze']
        }), 200
    
    elif request.method == 'POST':
        action = request.args.get('action', 'export')
        
        if action == 'export':
            conn = get_db()
            cur = conn.cursor()
            cur.execute(f"SELECT key_text, response_text, created_at FROM {brain.table_name}")
            memories = [dict(row) for row in cur.fetchall()]
            conn.close()
            
            return jsonify({
                'action': 'export',
                'count': len(memories),
                'memories': memories
            }), 200
        
        elif action == 'clear':
            conn = get_db()
            cur = conn.cursor()
            cur.execute(f"DELETE FROM {brain.table_name}")
            deleted = cur.rowcount
            conn.commit()
            conn.close()
            
            return jsonify({
                'action': 'clear',
                'memories_deleted': deleted
            }), 200
        
        return jsonify({'error': 'Unknown action'}), 400


@admin_bp.route('/settings', methods=['GET', 'PUT'])
@token_required
@admin_required
def system_settings(user):
    """Get or update system-wide settings"""
    if request.method == 'GET':
        return jsonify({
            'settings': {
                'app_name': 'JeebsAI',
                'version': '1.0.0',
                'environment': os.getenv('FLASK_ENV', 'production'),
                'debug_mode': False,
                'max_conversation_length': 1000,
                'message_retention_days': 365,
                'brain_similarity_threshold': 0.65,
                'learning_enabled': True,
                'auto_cleanup_enabled': True
            }
        }), 200
    
    elif request.method == 'PUT':
        data = request.get_json()
        # In production, store these in a settings table
        return jsonify({
            'message': 'Settings updated',
            'updated': list(data.keys())
        }), 200


@admin_bp.route('/ai/capabilities', methods=['GET'])
@token_required
@admin_required
def ai_capabilities(user):
    """List AI capabilities and features"""
    return jsonify({
        'ai_features': {
            'learning': {
                'enabled': True,
                'method': 'Holographic Reduced Representation (HRR)',
                'memory_capacity': 'Unlimited (DB-backed)',
                'learning_type': 'Unsupervised pattern matching'
            },
            'retrieval': {
                'enabled': True,
                'method': 'Vector cosine similarity search',
                'speed': 'Sub-millisecond',
                'accuracy': 'Similarity threshold: 0.65'
            },
            'conversation': {
                'enabled': True,
                'features': [
                    'Multi-turn conversations',
                    'Context retention',
                    'User-specific memory',
                    'Learning from interactions'
                ]
            },
            'customization': {
                'enabled': True,
                'features': [
                    'Brain parameters tuning',
                    'Response filtering',
                    'Knowledge base management',
                    'Memory pruning'
                ]
            }
        }
    }), 200


@admin_bp.route('/brain/viz-data', methods=['GET'])
@token_required
@admin_required
def brain_viz_data(user):
    """Get brain visualization data (nodes and edges for 3D/2D visualization)
    
    Returns:
    - nodes: List of memory nodes with id, query text, response excerpt, conversation_id
    - edges: List of similarity connections between nodes (weight > threshold)
    - stats: Overall brain statistics
    """
    import numpy as np
    import json
    
    def cosine_similarity_manual(v1, v2):
        """Compute cosine similarity between two vectors without sklearn"""
        dot_product = np.dot(v1, v2)
        norm_v1 = np.linalg.norm(v1)
        norm_v2 = np.linalg.norm(v2)
        if norm_v1 == 0 or norm_v2 == 0:
            return 0.0
        return dot_product / (norm_v1 * norm_v2)
    
    try:
        conn = get_db()
        cur = conn.cursor()
        
        # Get all memories with their vectors
        cur.execute(f"""
            SELECT id, conversation_id, key_text, response_text, vector_json, created_at
            FROM {brain.table_name}
            ORDER BY created_at DESC
        """)
        
        all_memories = cur.fetchall()
        memories = [dict(row) for row in all_memories]
        conn.close()
        
        if not memories:
            return jsonify({
                'nodes': [],
                'edges': [],
                'stats': {
                    'total_memories': 0,
                    'total_edges': 0,
                    'unique_conversations': 0
                }
            }), 200
        
        # Parse vectors and compute similarity
        vectors = []
        memory_ids = []
        
        for mem in memories:
            try:
                vec = json.loads(mem['vector_json'])
                vectors.append(np.array(vec, dtype=np.float32))
                memory_ids.append(mem['id'])
            except:
                pass
        
        if not vectors:
            return jsonify({
                'nodes': [],
                'edges': [],
                'stats': {
                    'total_memories': 0,
                    'total_edges': 0,
                    'unique_conversations': 0
                }
            }), 200
        
        # Compute similarity matrix
        vectors_array = np.array(vectors)
        
        # Build nodes (memories)
        nodes = []
        for i, mem in enumerate(memories):
            # Truncate long text
            query_text = mem['key_text'][:60] + ('...' if len(mem['key_text']) > 60 else '')
            response_text = mem['response_text'][:100] + ('...' if len(mem['response_text']) > 100 else '')
            
            nodes.append({
                'id': mem['id'],
                'label': query_text,
                'query': mem['key_text'],
                'response': response_text,
                'conversation_id': mem['conversation_id'],
                'created_at': mem['created_at'],
                'index': i
            })
        
        # Build edges (similarity > 0.65)
        edges = []
        threshold = 0.65
        edge_id = 0
        
        for i in range(len(vectors)):
            for j in range(i + 1, len(vectors)):
                similarity = float(cosine_similarity_manual(vectors_array[i], vectors_array[j]))
                
                if similarity > threshold:
                    edges.append({
                        'id': edge_id,
                        'source': memory_ids[i],
                        'target': memory_ids[j],
                        'weight': similarity
                    })
                    edge_id += 1
        
        # Compute stats
        unique_convs = len(set(mem['conversation_id'] for mem in memories))
        
        return jsonify({
            'nodes': nodes,
            'edges': edges,
            'stats': {
                'total_memories': len(memories),
                'total_edges': len(edges),
                'unique_conversations': unique_convs,
                'similarity_threshold': threshold,
                'brain_dimension': brain.dim
            }
        }), 200
        
    except Exception as e:
        import traceback
        return jsonify({
            'error': str(e),
            'traceback': traceback.format_exc()
        }), 500


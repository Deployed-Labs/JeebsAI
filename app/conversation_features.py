"""
JeebsAI Conversation Features
Advanced conversation management: branching, history editing, conversation analytics
"""

from .models import get_db, Conversation, Message
from datetime import datetime
import json

class ConversationManager:
    """Advanced conversation management"""
    
    @staticmethod
    def get_conversation_tree(conv_id):
        """Get conversation as a tree structure (for branching/editing)"""
        try:
            db = get_db()
            conversation = Conversation.get_by_id(conv_id)
            if not conversation:
                return None
            
            messages = Message.get_conversation_messages(conv_id)
            
            # Build tree structure
            tree = {
                'id': conversation['id'],
                'title': conversation['title'],
                'created_at': conversation['created_at'],
                'messages': [
                    {
                        'id': m['id'],
                        'role': m['role'],
                        'content': m['content'],
                        'created_at': m['created_at'],
                        'metadata': json.loads(m.get('metadata', '{}'))
                    }
                    for m in messages
                ]
            }
            return tree
        except Exception as e:
            return {'error': str(e)}
    
    @staticmethod
    def branch_conversation(conv_id, from_message_id, new_title=None):
        """Create a new conversation branch from a specific point"""
        try:
            db = get_db()
            original = Conversation.get_by_id(conv_id)
            if not original:
                return {'error': 'Conversation not found'}
            
            # Get messages up to the branch point
            messages = Message.get_conversation_messages(conv_id)
            branch_messages = [m for m in messages if int(m['id']) <= int(from_message_id)]
            
            if not branch_messages:
                return {'error': 'Message not found'}
            
            # Create new conversation
            title = new_title or f"{original['title']} (Branch)"
            new_conv_id = Conversation.create(original['user_id'], title)
            
            # Copy messages up to branch point
            for msg in branch_messages:
                Message.create(new_conv_id, msg['role'], msg['content'])
            
            return {
                'success': True,
                'original_id': conv_id,
                'branch_id': new_conv_id,
                'branch_title': title,
                'messages_copied': len(branch_messages)
            }
        except Exception as e:
            return {'error': str(e)}
    
    @staticmethod
    def edit_message(conv_id, message_id, new_content):
        """Edit a message in a conversation"""
        try:
            db = get_db()
            message = db.execute(
                'SELECT * FROM messages WHERE id = ? AND conversation_id = ?',
                (message_id, conv_id)
            ).fetchone()
            
            if not message:
                return {'error': 'Message not found'}
            
            # Update message
            db.execute(
                'UPDATE messages SET content = ?, updated_at = ? WHERE id = ?',
                (new_content, datetime.now().isoformat(), message_id)
            )
            db.commit()
            
            return {
                'success': True,
                'message_id': message_id,
                'updated_at': datetime.now().isoformat()
            }
        except Exception as e:
            return {'error': str(e)}
    
    @staticmethod
    def delete_message_and_replies(conv_id, message_id):
        """Delete a message and all subsequent messages"""
        try:
            db = get_db()
            messages = Message.get_conversation_messages(conv_id)
            
            # Find message index
            msg_index = None
            for i, m in enumerate(messages):
                if m['id'] == message_id:
                    msg_index = i
                    break
            
            if msg_index is None:
                return {'error': 'Message not found'}
            
            # Delete all messages from this point onwards
            deleted_count = 0
            for msg in messages[msg_index:]:
                db.execute('DELETE FROM messages WHERE id = ?', (msg['id'],))
                deleted_count += 1
            
            db.commit()
            
            return {
                'success': True,
                'deleted_count': deleted_count,
                'remaining_messages': len(messages) - deleted_count
            }
        except Exception as e:
            return {'error': str(e)}
    
    @staticmethod
    def merge_conversations(conv_id_1, conv_id_2, new_title=None):
        """Merge two conversations into one"""
        try:
            db = get_db()
            
            conv1 = Conversation.get_by_id(conv_id_1)
            conv2 = Conversation.get_by_id(conv_id_2)
            
            if not conv1 or not conv2:
                return {'error': 'One or both conversations not found'}
            
            if conv1['user_id'] != conv2['user_id']:
                return {'error': 'Conversations must belong to same user'}
            
            # Create merged conversation
            title = new_title or f"{conv1['title']} + {conv2['title']}"
            merged_id = Conversation.create(conv1['user_id'], title)
            
            # Copy all messages from both conversations
            messages1 = Message.get_conversation_messages(conv_id_1)
            messages2 = Message.get_conversation_messages(conv_id_2)
            
            for msg in messages1:
                Message.create(merged_id, msg['role'], msg['content'])
            
            for msg in messages2:
                Message.create(merged_id, msg['role'], msg['content'])
            
            return {
                'success': True,
                'merged_id': merged_id,
                'merged_title': title,
                'total_messages': len(messages1) + len(messages2)
            }
        except Exception as e:
            return {'error': str(e)}


class ConversationAnalytics:
    """Analytics for conversations and user behavior"""
    
    @staticmethod
    def get_conversation_stats(conv_id):
        """Get detailed stats about a conversation"""
        try:
            db = get_db()
            conversation = Conversation.get_by_id(conv_id)
            if not conversation:
                return {'error': 'Conversation not found'}
            
            messages = Message.get_conversation_messages(conv_id)
            
            user_msgs = [m for m in messages if m['role'] == 'user']
            ai_msgs = [m for m in messages if m['role'] == 'assistant']
            
            # Calculate stats
            total_chars = sum(len(m['content']) for m in messages)
            user_chars = sum(len(m['content']) for m in user_msgs)
            ai_chars = sum(len(m['content']) for m in ai_msgs)
            
            stats = {
                'conversation_id': conv_id,
                'title': conversation['title'],
                'created_at': conversation['created_at'],
                'updated_at': conversation['updated_at'],
                'total_messages': len(messages),
                'user_messages': len(user_msgs),
                'ai_messages': len(ai_msgs),
                'total_characters': total_chars,
                'user_characters': user_chars,
                'ai_characters': ai_chars,
                'avg_user_length': round(user_chars / max(1, len(user_msgs)), 2),
                'avg_ai_length': round(ai_chars / max(1, len(ai_msgs)), 2),
                'conversation_depth': len(messages),
                'balance': round((len(user_msgs) / max(1, len(messages))) * 100, 1)
            }
            
            return stats
        except Exception as e:
            return {'error': str(e)}
    
    @staticmethod
    def get_user_analytics(user_id):
        """Get analytics for a user's conversations"""
        try:
            db = get_db()
            conversations_data = Conversation.get_user_conversations(user_id)
            conversations = conversations_data.get('items', []) if isinstance(conversations_data, dict) else conversations_data
            
            if not conversations:
                return {
                    'user_id': user_id,
                    'total_conversations': 0,
                    'total_messages': 0,
                    'total_characters': 0,
                    'avg_conversation_length': 0,
                    'avg_message_length': 0
                }
            
            total_msgs = 0
            total_chars = 0
            active_conversations = 0
            
            for conv in conversations:
                messages = Message.get_conversation_messages(conv['id'])
                if messages:
                    total_msgs += len(messages)
                    total_chars += sum(len(m['content']) for m in messages)
                    active_conversations += 1
            
            analytics = {
                'user_id': user_id,
                'total_conversations': len(conversations),
                'active_conversations': active_conversations,
                'total_messages': total_msgs,
                'total_characters': total_chars,
                'avg_conversation_length': round(total_msgs / max(1, len(conversations)), 1),
                'avg_message_length': round(total_chars / max(1, total_msgs), 1)
            }
            
            return analytics
        except Exception as e:
            return {'error': str(e)}
    
    @staticmethod
    def search_conversations(user_id, query):
        """Search across user's conversations"""
        try:
            db = get_db()
            conversations_data = Conversation.get_user_conversations(user_id)
            conversations = conversations_data.get('items', []) if isinstance(conversations_data, dict) else conversations_data
            
            results = []
            for conv in conversations:
                messages = Message.get_conversation_messages(conv['id'])
                matching_msgs = [
                    m for m in messages 
                    if query.lower() in m['content'].lower()
                ]
                
                if matching_msgs:
                    results.append({
                        'conversation_id': conv['id'],
                        'conversation_title': conv['title'],
                        'matching_messages': len(matching_msgs),
                        'context': matching_msgs[0]['content'][:200] + '...'
                    })
            
            return {
                'query': query,
                'results': results,
                'total_matches': len(results)
            }
        except Exception as e:
            return {'error': str(e)}
    
    @staticmethod
    def get_trending_topics(user_id, limit=10):
        """Find most talked about topics in user's conversations"""
        try:
            db = get_db()
            conversations_data = Conversation.get_user_conversations(user_id)
            conversations = conversations_data.get('items', []) if isinstance(conversations_data, dict) else conversations_data
            
            # Simple word frequency analysis
            word_freq = {}
            stop_words = {'the', 'a', 'an', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for', 'of', 'is', 'are', 'was', 'were', 'be', 'been', 'being', 'i', 'you', 'he', 'she', 'it', 'we', 'they'}
            
            for conv in conversations:
                messages = Message.get_conversation_messages(conv['id'])
                for msg in messages:
                    words = msg['content'].lower().split()
                    for word in words:
                        word = word.strip('.,!?;:')
                        if len(word) > 3 and word not in stop_words:
                            word_freq[word] = word_freq.get(word, 0) + 1
            
            # Sort by frequency
            trending = sorted(word_freq.items(), key=lambda x: x[1], reverse=True)[:limit]
            
            return {
                'topics': [{'topic': word, 'frequency': count} for word, count in trending],
                'total_unique_topics': len(word_freq)
            }
        except Exception as e:
            return {'error': str(e)}


# Custom Prompts/Instructions
class CustomPrompts:
    """Manage custom system prompts for users"""
    
    @staticmethod
    def set_user_prompt(user_id, system_prompt, name=None):
        """Set a custom system prompt for a user"""
        try:
            db = get_db()
            
            # Check if user_prompts table exists
            db.execute('''
                CREATE TABLE IF NOT EXISTS user_prompts (
                    id INTEGER PRIMARY KEY,
                    user_id INTEGER NOT NULL,
                    name TEXT,
                    system_prompt TEXT NOT NULL,
                    created_at TEXT,
                    updated_at TEXT,
                    FOREIGN KEY (user_id) REFERENCES users(id)
                )
            ''')
            
            if name is None:
                name = f"Prompt {datetime.now().strftime('%Y-%m-%d %H:%M')}"
            
            now = datetime.now().isoformat()
            db.execute(
                'INSERT INTO user_prompts (user_id, name, system_prompt, created_at, updated_at) VALUES (?, ?, ?, ?, ?)',
                (user_id, name, system_prompt, now, now)
            )
            db.commit()
            
            return {
                'success': True,
                'user_id': user_id,
                'prompt_name': name
            }
        except Exception as e:
            return {'error': str(e)}
    
    @staticmethod
    def get_user_prompts(user_id):
        """Get all custom prompts for a user"""
        try:
            db = get_db()
            prompts = db.execute(
                'SELECT id, name, system_prompt, created_at FROM user_prompts WHERE user_id = ? ORDER BY created_at DESC',
                (user_id,)
            ).fetchall()
            
            return [dict(p) for p in prompts]
        except Exception as e:
            return {'error': str(e)}
    
    @staticmethod
    def apply_prompt(user_id, prompt_id):
        """Mark a prompt as active for a user"""
        try:
            db = get_db()
            prompt = db.execute(
                'SELECT * FROM user_prompts WHERE id = ? AND user_id = ?',
                (prompt_id, user_id)
            ).fetchone()
            
            if not prompt:
                return {'error': 'Prompt not found'}
            
            return {
                'success': True,
                'active_prompt': prompt['name'],
                'system_prompt': prompt['system_prompt']
            }
        except Exception as e:
            return {'error': str(e)}

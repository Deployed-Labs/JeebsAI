from flask import Blueprint, request, jsonify
from .models import Conversation, Message
from .auth import token_required

chat_bp = Blueprint('chat', __name__, url_prefix='/api/chat')

def generate_response(user_message):
    """Generate a simple response from JeebsAI"""
    # Simple demo responses - can be replaced with actual ML model
    responses = {
        'hello': 'Hello! I\'m JeebsAI. How can I help you today?',
        'hi': 'Hey there! What can I do for you?',
        'how are you': 'I\'m doing great! Thanks for asking. How about you?',
        'what is your name': 'I\'m JeebsAI, your AI assistant.',
        'help': 'I can help you with questions, provide information, and have conversations. Just ask me anything!',
    }
    
    message_lower = user_message.lower().strip()
    
    # Check for exact or partial matches
    for key, response in responses.items():
        if key in message_lower:
            return response
    
    # Default response
    return f"That's interesting! You said: \"{user_message}\". I'm here to help with any questions you might have."

@chat_bp.route('/conversations', methods=['GET'])
@token_required
def get_conversations(user):
    """Get all conversations for the user"""
    conversations = Conversation.get_user_conversations(user['id'])
    return jsonify(conversations), 200

@chat_bp.route('/conversations', methods=['POST'])
@token_required
def create_conversation(user):
    """Create a new conversation"""
    data = request.get_json() or {}
    title = data.get('title', 'New Chat')
    
    conv_id = Conversation.create(user['id'], title)
    conversation = Conversation.get_by_id(conv_id)
    
    return jsonify(conversation), 201

@chat_bp.route('/conversations/<int:conv_id>', methods=['GET'])
@token_required
def get_conversation(user, conv_id):
    """Get a specific conversation with messages"""
    conversation = Conversation.get_by_id(conv_id)
    
    # Verify user owns this conversation
    if not conversation or conversation['user_id'] != user['id']:
        return jsonify({'message': 'Conversation not found'}), 404
    
    messages = Message.get_conversation_messages(conv_id)
    
    return jsonify({
        'conversation': conversation,
        'messages': messages
    }), 200

@chat_bp.route('/conversations/<int:conv_id>/messages', methods=['POST'])
@token_required
def send_message(user, conv_id):
    """Send a message and get a response"""
    conversation = Conversation.get_by_id(conv_id)
    
    # Verify user owns this conversation
    if not conversation or conversation['user_id'] != user['id']:
        return jsonify({'message': 'Conversation not found'}), 404
    
    data = request.get_json()
    if not data or not data.get('content'):
        return jsonify({'message': 'Message content is required'}), 400
    
    user_message = data.get('content')
    
    # Store user message
    Message.create(conv_id, 'user', user_message)
    
    # Generate and store AI response
    ai_response = generate_response(user_message)
    Message.create(conv_id, 'assistant', ai_response)
    
    # Return the messages
    messages = Message.get_conversation_messages(conv_id)
    
    return jsonify({
        'messages': messages,
        'response': ai_response
    }), 200

@chat_bp.route('/conversations/<int:conv_id>/title', methods=['PUT'])
@token_required
def update_conversation_title(user, conv_id):
    """Update conversation title"""
    conversation = Conversation.get_by_id(conv_id)
    
    # Verify user owns this conversation
    if not conversation or conversation['user_id'] != user['id']:
        return jsonify({'message': 'Conversation not found'}), 404
    
    data = request.get_json()
    if not data or not data.get('title'):
        return jsonify({'message': 'Title is required'}), 400
    
    title = data.get('title')
    Conversation.update_title(conv_id, title)
    
    updated_conv = Conversation.get_by_id(conv_id)
    return jsonify(updated_conv), 200

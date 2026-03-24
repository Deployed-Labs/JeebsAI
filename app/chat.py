from flask import Blueprint, request, jsonify
from .models import Conversation, Message
from .holographic_brain import brain
from .auth import token_required
from .tools import execute_tool

chat_bp = Blueprint('chat', __name__, url_prefix='/api/chat')

def detect_and_use_tools(user_message):
    """Detect if a message requests tool usage and execute if needed"""
    message_lower = user_message.lower()
    
    # Web search detection
    if any(phrase in message_lower for phrase in ['search', 'find', 'look up', 'what is', 'who is', 'latest', 'news']):
        if any(phrase in message_lower for phrase in ['search for', 'look up', 'find', 'search', 'web']):
            # Extract search query
            keywords = message_lower.split()
            query = ' '.join(keywords)
            result = execute_tool('web_search', query=query, max_results=3)
            if result.get('results'):
                response = f"I found some information about '{query}':\n\n"
                for i, r in enumerate(result['results'][:3], 1):
                    response += f"{i}. **{r.get('title', 'Result')}**: {r.get('snippet', 'No info')}\n"
                return response
    
    # Calculator detection
    if any(phrase in message_lower for phrase in ['calculate', 'math', 'what is', 'compute', '+',' -', '*', '/', '=']):
        # Look for mathematical expressions
        import re
        if re.search(r'\d\s*[+\-*/]\s*\d', user_message) or 'calculate' in message_lower:
            result = execute_tool('analyze_code', code='', check_type='syntax')  # Dummy
            return None  # Fall through to other methods
    
    # Code analysis detection
    if 'code' in message_lower and ('error' in message_lower or 'bug' in message_lower or 'fix' in message_lower or 'check' in message_lower):
        return None  # Fall through - would need code parameter
    
    return None

def generate_response(user_message):
    """Generate a response using the holographic brain, tool use, and rule-based fallback."""
    
    # Try to use tools if applicable
    tool_response = detect_and_use_tools(user_message)
    if tool_response:
        return tool_response
    
    # Try retrieval from holographic brain
    try:
        results = brain.query(user_message, top_k=1)
        if results:
            sim, resp = results[0]
            if sim >= 0.65:
                return resp
    except Exception:
        # if brain fails, continue to fallback
        pass

    # Simple demo rule-based fallback with enhanced responses
    responses = {
        'hello': "Hello! I'm JeebsAI 🧠. I have access to tools like web search, calculator, code analysis, and more. How can I help you today?",
        'hi': 'Hey there! What can I do for you?',
        'how are you': "I'm doing great! Thanks for asking. How about you?",
        'what is your name': "I'm JeebsAI, your AI assistant with the Holographic Brain 🧠. Ask me about my special features!",
        'help': 'I can help you with questions, web searches, calculations, code analysis, conversation management, and more! Try asking me to search for something, calculate math, or analyze code.',
        'what can you do': 'I have many capabilities! I can search the web, do calculations, analyze code, track conversation analytics, create custom prompts, branch conversations, and use my special Holographic Brain for smart learning.',
    }

    message_lower = user_message.lower().strip()

    for key, response in responses.items():
        if key in message_lower:
            return response

    return f"That's interesting! You said: \"{user_message}\". I'm here to help with any questions you might have. You can also ask me to search the web, calculate math, analyze code, or manage your conversations!"

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

    # Save the user->assistant pair to the holographic brain for learning
    try:
        brain.save_memory(conv_id, user_message, ai_response)
    except Exception:
        pass
    
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

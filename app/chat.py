from flask import Blueprint, request, jsonify
from .models import Conversation, Message
from .holographic_brain import brain
from .auth import token_required
from .tools import execute_tool, TOOLS_REGISTRY

chat_bp = Blueprint('chat', __name__, url_prefix='/api/chat')

def suggest_tools(user_message, max_suggestions=3):
    """Analyze user message and suggest relevant tools"""
    message_lower = user_message.lower()
    
    # Define keyword-to-tool mappings
    tool_keywords = {
        'web_search': ['search', 'find', 'look up', 'what is', 'who is', 'news', 'latest', 'tell me about'],
        'calculator': ['calculate', 'math', 'compute', 'what is', 'how much', 'equals', '+', '-', '*', '/', 'multiply', 'divide', 'add', 'subtract'],
        'wikipedia_summary': ['wikipedia', 'about', 'explain', 'tell me', 'summary', 'overview'],
        'define_word': ['define', 'meaning', 'what does', 'definition', 'means'],
        'sentiment_analysis': ['sentiment', 'emotion', 'feeling', 'mood', 'positive', 'negative', 'analyze text'],
        'text_summarizer': ['summarize', 'summary', 'tldr', 'shorten', 'brief', 'condense'],
        'keyword_extractor': ['keywords', 'key points', 'main ideas', 'extract', 'important'],
        'convert_units': ['convert', 'km to miles', 'pounds to kg', 'fahrenheit', 'celsius', 'units'],
        'convert_color': ['color', 'hex', 'rgb', 'hsl', 'convert color'],
        'base64_encode_decode': ['encode', 'decode', 'base64', 'encoding'],
        'hash_generator': ['hash', 'md5', 'sha', 'encrypt', 'cryptographic'],
        'generate_password': ['password', 'generate', 'passphrase', 'secure password'],
        'format_json': ['json', 'format', 'pretty', 'minify', 'validate json'],
        'regex_match': ['regex', 'pattern', 'match', 'regular expression', 'pattern matching'],
        'markdown_to_html': ['markdown', 'html', 'convert', 'render'],
        'date_calculator': ['date', 'days', 'between', 'difference', 'how many days'],
        'time_range_calculator': ['time', 'duration', 'how long', 'start time', 'end time'],
        'csv_parser': ['csv', 'parse', 'comma', 'spreadsheet', 'data'],
        'data_validator': ['validate', 'email', 'url', 'phone', 'format', 'check'],
        'create_todo': ['todo', 'task', 'create', 'task list', 'remember'],
        'pomodoro_calculator': ['pomodoro', 'timer', 'break', 'work session', 'productivity'],
        'task_priority_score': ['priority', 'important', 'urgent', 'eisenhower'],
        'generate_qr_ascii': ['qr code', 'qr', 'encode', 'barcode'],
        'fun_fact': ['fact', 'interesting', 'did you know', 'fun'],
        'get_joke': ['joke', 'funny', 'laugh', 'humor'],
        'get_quote': ['quote', 'inspirational', 'motivation', 'saying'],
        'code_analysis': ['code', 'error', 'bug', 'fix', 'syntax'],
        'ip_info': ['ip', 'address', 'domain', 'network'],
        'analyze_code': ['code', 'analyze', 'review', 'check']
    }
    
    suggestions = []
    
    # Score each tool based on keyword matches
    for tool_name, keywords in tool_keywords.items():
        if tool_name not in TOOLS_REGISTRY:
            continue
            
        score = 0
        matched_keywords = []
        
        for keyword in keywords:
            if keyword in message_lower:
                score += 1
                matched_keywords.append(keyword)
        
        if score > 0:
            tool = TOOLS_REGISTRY[tool_name]
            suggestions.append({
                'name': tool['name'],
                'description': tool['description'],
                'score': score,
                'matched_keywords': matched_keywords,
                'parameters': tool['parameters']
            })
    
    # Sort by score and return top N
    suggestions.sort(key=lambda x: x['score'], reverse=True)
    return suggestions[:max_suggestions]

def detect_and_use_tools(user_message, conv_id=None):
    """Detect if a message requests tool usage and execute if needed, then learn results"""
    message_lower = user_message.lower()
    
    # Web search detection
    if any(phrase in message_lower for phrase in ['search', 'find', 'look up', 'what is', 'who is', 'latest', 'news', 'tell me about', 'information about']):
        # Use better query extraction
        query = user_message
        # Remove common prefixes
        for prefix in ['search for ', 'look up ', 'find ', 'search ', 'what is ', 'who is ', 'latest ', 'news about ', 'tell me about ', 'information about ', 'i want to know about ']:
            if query.lower().startswith(prefix):
                query = query[len(prefix):]
                break
        
        query = query.strip()
        if len(query) > 1:
            result = execute_tool('web_search', query=query, max_results=3)
            if result.get('success') and result.get('results'):
                # Build response with search results
                response = f"🔍 **Search Results for '{query}':**\n\n"
                for i, r in enumerate(result['results'], 1):
                    snippet = r.get('snippet', 'No info available')
                    # Truncate if too long
                    if len(snippet) > 200:
                        snippet = snippet[:197] + '...'
                    response += f"{i}. **{r.get('title', 'Result')}**\n   {snippet}\n\n"
                
                # Learn this search result to the brain
                if conv_id:
                    try:
                        combined_knowledge = f"{query}: " + " ".join([r.get('snippet', '') for r in result['results'][:3]])
                        brain.save_memory(conv_id, query, combined_knowledge)
                    except Exception as e:
                        pass  # Silently fail if brain learning fails
                
                return response
    
    # Calculator detection
    if any(phrase in message_lower for phrase in ['calculate', 'math', 'what is', 'compute', '+',' -', '*', '/', '=']):
        # Look for mathematical expressions
        import re
        if re.search(r'\d\s*[+\-*/]\s*\d', user_message) or 'calculate' in message_lower:
            return None  # Fall through to other methods
    
    # Code analysis detection
    if 'code' in message_lower and ('error' in message_lower or 'bug' in message_lower or 'fix' in message_lower or 'check' in message_lower):
        return None  # Fall through - would need code parameter
    
    return None

def generate_response(user_message, conv_id=None):
    """Generate a response using the holographic brain, tool use, and rule-based fallback."""
    
    # Try to use tools if applicable (pass conv_id so tools can learn)
    tool_response = detect_and_use_tools(user_message, conv_id)
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
    ai_response = generate_response(user_message, conv_id)
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

@chat_bp.route('/suggest-tools', methods=['POST'])
@token_required
def suggest_tools_endpoint(user):
    """Suggest tools based on user message"""
    data = request.get_json() or {}
    message = data.get('message', '').strip()
    max_suggestions = data.get('max_suggestions', 3)
    
    if not message:
        return jsonify({'message': 'Message is required'}), 400
    
    suggestions = suggest_tools(message, max_suggestions=max_suggestions)
    
    return jsonify({
        'message': message,
        'suggestions': suggestions,
        'count': len(suggestions)
    }), 200

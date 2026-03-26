from flask import Blueprint, request, jsonify
from .models import Conversation, Message, get_db
from .holographic_brain import brain
from .auth import token_required
from .tools import execute_tool, TOOLS_REGISTRY
from datetime import datetime

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

def generate_response(user_message, conv_id=None, conversation_messages=None):
    """Generate a response using the holographic brain, tool use, and rule-based fallback.
    
    Args:
        user_message: The user's input message
        conv_id: Conversation ID for context-aware learning
        conversation_messages: List of recent messages for context (improves understanding)
    """
    
    # Try to use tools if applicable (pass conv_id so tools can learn)
    tool_response = detect_and_use_tools(user_message, conv_id)
    if tool_response:
        return tool_response
    
    # Try retrieval from holographic brain with conversation context
    try:
        # Prepare context from recent messages for better understanding
        use_context = conversation_messages is not None and len(conversation_messages) > 0
        conv_context = None
        
        if use_context:
            # Build context from recent messages (last 3-5 exchanges for understanding)
            conv_context = []
            for msg in conversation_messages[-6:]:  # Last 6 messages = 3 exchanges
                conv_context.append({
                    'role': msg.get('role', 'unknown'),
                    'content': msg.get('content', ''),
                    'conversation_id': conv_id
                })
        
        # Query with context awareness for better semantic matching
        results = brain.query(user_message, top_k=2, use_priority=True, 
                            use_context=use_context, conv_context=conv_context)
        
        if results:
            sim, resp = results[0]
            # Higher confidence threshold when using context
            threshold = 0.60 if use_context else 0.65
            if sim >= threshold:
                return resp
    except Exception as e:
        # if brain fails, continue to fallback
        pass

    # Check for correction patterns (when user corrects previous response)
    correction_markers = ['no, ', 'not ', 'i meant ', 'actually ', 'sorry ', 'what i meant', 'correct']
    msg_lower = user_message.lower()
    is_correction = any(msg_lower.startswith(marker) for marker in correction_markers)
    
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

    # If this is a correction, acknowledge it and learn the correction
    if is_correction:
        corrected_msg = user_message
        for marker in correction_markers:
            if corrected_msg.lower().startswith(marker):
                corrected_msg = corrected_msg[len(marker):].strip()
                break
        
        response = f"Got it! Thank you for the correction. I understand now: {corrected_msg}. I'll remember this for future reference!"
        
        # Learn the correction with high priority
        if conv_id and corrected_msg:
            try:
                brain.save_memory(conv_id, user_message, response, priority=2, category='correction')
            except:
                pass
        
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
    
    # Get recent conversation history for context-aware response generation
    recent_messages = Message.get_conversation_messages(conv_id)
    
    # Generate and store AI response with conversation context
    ai_response = generate_response(user_message, conv_id, conversation_messages=recent_messages)
    Message.create(conv_id, 'assistant', ai_response)

    # Save the user->assistant pair to the holographic brain for learning
    try:
        # Determine priority based on message characteristics
        priority = 1  # Default priority
        if len(user_message) > 100:  # Longer messages might be important
            priority = 2
        if any(marker in user_message.lower() for marker in ['important', 'remember', 'key point']):
            priority = 2
        
        brain.save_memory(conv_id, user_message, ai_response, priority=priority)
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
    data = request.get_json()
    if not data or not data.get('message'):
        return jsonify({'message': 'Message is required'}), 400
    
    user_message = data.get('message')
    suggestions = suggest_tools(user_message, max_suggestions=5)
    
    return jsonify({'suggestions': suggestions}), 200


@chat_bp.route('/conversations/<int:conv_id>/search', methods=['GET'])
@token_required
def search_conversation(user, conv_id):
    """Search messages in a conversation"""
    conversation = Conversation.get_by_id(conv_id)
    
    # Verify user owns this conversation
    if not conversation or conversation['user_id'] != user['id']:
        return jsonify({'message': 'Conversation not found'}), 404
    
    search_term = request.args.get('q', '').strip()
    if not search_term or len(search_term) < 2:
        return jsonify({'message': 'Search term too short'}), 400
    
    conn = get_db()
    cursor = conn.cursor()
    search_pattern = f"%{search_term}%"
    
    cursor.execute('''
        SELECT id, role, content, created_at
        FROM messages
        WHERE conversation_id = ? AND (content LIKE ?)
        ORDER BY created_at DESC
        LIMIT 50
    ''', (conv_id, search_pattern))
    
    results = [dict(row) for row in cursor.fetchall()]
    conn.close()
    
    return jsonify({
        'query': search_term,
        'count': len(results),
        'results': results
    }), 200


@chat_bp.route('/conversations/<int:conv_id>/export', methods=['GET'])
@token_required
def export_conversation(user, conv_id):
    """Export conversation as JSON"""
    conversation = Conversation.get_by_id(conv_id)
    
    # Verify user owns this conversation
    if not conversation or conversation['user_id'] != user['id']:
        return jsonify({'message': 'Conversation not found'}), 404
    
    conn = get_db()
    cursor = conn.cursor()
    
    cursor.execute('''
        SELECT id, role, content, created_at
        FROM messages
        WHERE conversation_id = ?
        ORDER BY created_at ASC
    ''', (conv_id,))
    
    messages = [dict(row) for row in cursor.fetchall()]
    conn.close()
    
    export_data = {
        'conversation': conversation,
        'messages': messages,
        'exported_at': datetime.utcnow().isoformat()
    }
    
    return jsonify(export_data), 200


@chat_bp.route('/teach', methods=['POST'])
@token_required
def teach_jeebs(user):
    """Explicitly teach JeebsAI new knowledge/facts with high priority"""
    data = request.get_json()
    if not data:
        return jsonify({'message': 'Request body required'}), 400
    
    key_text = data.get('key', '').strip()
    response_text = data.get('response', '').strip()
    category = data.get('category', 'general').strip()
    conv_id = data.get('conversation_id', 0)
    
    if not key_text or not response_text:
        return jsonify({'message': 'Both "key" and "response" are required'}), 400
    
    try:
        brain.teach(key_text, response_text, conversation_id=conv_id, category=category)
        return jsonify({
            'message': 'Knowledge successfully taught to JeebsAI!',
            'key': key_text,
            'category': category,
            'success': True
        }), 201
    except Exception as e:
        return jsonify({'message': f'Error saving knowledge: {str(e)}'} ), 500


@chat_bp.route('/brain/memories', methods=['GET'])
@token_required
def list_memories(user):
    """List memories learned by JeebsAI (optionally filtered by conversation)"""
    conv_id = request.args.get('conversation_id', type=int)
    limit = request.args.get('limit', 50, type=int)
    
    try:
        memories = brain.list_memories(conversation_id=conv_id, limit=limit)
        return jsonify({
            'memories': memories,
            'count': len(memories),
            'conversation_id': conv_id
        }), 200
    except Exception as e:
        return jsonify({'message': f'Error retrieving memories: {str(e)}'}), 500


@chat_bp.route('/brain/forget/<int:memory_id>', methods=['DELETE'])
@token_required
def forget_memory(user, memory_id):
    """Tell JeebsAI to forget a specific memory"""
    try:
        success = brain.delete_memory(memory_id)
        if success:
            return jsonify({
                'message': 'Memory successfully forgotten',
                'memory_id': memory_id
            }), 200
        else:
            return jsonify({'message': 'Memory not found'}), 404
    except Exception as e:
        return jsonify({'message': f'Error deleting memory: {str(e)}'}), 500


@chat_bp.route('/brain/recall', methods=['POST'])
@token_required
def recall_memories(user):
    """Query/recall all related memories for a given text"""
    data = request.get_json()
    if not data or not data.get('query'):
        return jsonify({'message': 'query parameter required'}), 400
    
    query = data.get('query')
    top_k = data.get('top_k', 3)
    
    try:
        results = brain.query(query, top_k=top_k, use_priority=True)
        return jsonify({
            'query': query,
            'memories': [{'similarity': round(sim, 3), 'response': resp} for sim, resp in results],
            'count': len(results)
        }), 200
    except Exception as e:
        return jsonify({'message': f'Error recalling memories: {str(e)}'}), 500


@chat_bp.route('/brain/conversation-context/<int:conv_id>', methods=['GET'])
@token_required
def get_conversation_context(user, conv_id):
    """Get what JeebsAI has learned about a specific conversation (topics, themes, style)"""
    conversation = Conversation.get_by_id(conv_id)
    
    # Verify user owns this conversation
    if not conversation or conversation['user_id'] != user['id']:
        return jsonify({'message': 'Conversation not found'}), 404
    
    try:
        context = brain.get_conversation_context(conv_id)
        return jsonify({
            'conversation_id': conv_id,
            'title': conversation.get('title', 'Untitled'),
            'learning_context': context,
            'success': True
        }), 200
    except Exception as e:
        return jsonify({'message': f'Error analyzing conversation: {str(e)}'}), 500


@chat_bp.route('/brain/extract-concepts', methods=['POST'])
@token_required
def extract_text_concepts(user):
    """Extract important concepts and entities from text for semantic understanding"""
    data = request.get_json()
    if not data or not data.get('text'):
        return jsonify({'message': 'text parameter required'}), 400
    
    text = data.get('text')
    
    try:
        concepts = brain.extract_concepts(text)
        return jsonify({
            'text': text[:100] + ('...' if len(text) > 100 else ''),
            'concepts': concepts,
            'count': len(concepts),
            'success': True
        }), 200
    except Exception as e:
        return jsonify({'message': f'Error extracting concepts: {str(e)}'}), 500


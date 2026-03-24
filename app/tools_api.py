"""
JeebsAI Tools API - Endpoints for tools, conversation features, and analytics
"""

from flask import Blueprint, request, jsonify
from .auth import token_required
from .tools import execute_tool, get_available_tools
from .conversation_features import ConversationManager, ConversationAnalytics, CustomPrompts

tools_bp = Blueprint('tools', __name__, url_prefix='/api/tools')

# ============================================================================
# TOOLS ENDPOINTS
# ============================================================================

@tools_bp.route('/available', methods=['GET'])
@token_required
def list_tools(user):
    """Get list of available tools"""
    tools = get_available_tools()
    return jsonify({
        'tools': tools,
        'count': len(tools)
    }), 200


@tools_bp.route('/execute', methods=['POST'])
@token_required
def execute_user_tool(user):
    """Execute a tool"""
    data = request.get_json() or {}
    tool_name = data.get('tool_name')
    params = data.get('params', {})
    
    if not tool_name:
        return jsonify({'message': 'tool_name is required'}), 400
    
    result = execute_tool(tool_name, **params)
    return jsonify({
        'tool': tool_name,
        'result': result
    }), 200


# Calculator endpoint
@tools_bp.route('/calculator', methods=['POST'])
@token_required
def calculator(user):
    """Quick calculator endpoint"""
    data = request.get_json() or {}
    expression = data.get('expression')
    operation = data.get('operation')
    
    result = execute_tool('calculator', expression=expression, operation=operation, **data)
    return jsonify(result), 200


# Web search endpoint
@tools_bp.route('/search', methods=['POST'])
@token_required
def search(user):
    """Web search endpoint"""
    data = request.get_json() or {}
    query = data.get('query')
    
    if not query:
        return jsonify({'message': 'query is required'}), 400
    
    max_results = data.get('max_results', 5)
    result = execute_tool('web_search', query=query, max_results=max_results)
    
    return jsonify(result), 200


# Code analysis endpoint
@tools_bp.route('/analyze-code', methods=['POST'])
@token_required
def analyze(user):
    """Code analysis endpoint"""
    data = request.get_json() or {}
    code = data.get('code')
    check_type = data.get('check_type', 'syntax')
    
    if not code:
        return jsonify({'message': 'code is required'}), 400
    
    result = execute_tool('analyze_code', code=code, check_type=check_type)
    return jsonify(result), 200


# Text statistics endpoint
@tools_bp.route('/text-stats', methods=['POST'])
@token_required
def text_stats(user):
    """Text statistics endpoint"""
    data = request.get_json() or {}
    text = data.get('text')
    include = data.get('include', 'words,sentences')
    
    if not text:
        return jsonify({'message': 'text is required'}), 400
    
    result = execute_tool('text_stats', text=text, include=include)
    return jsonify(result), 200


# ============================================================================
# CONVERSATION MANAGEMENT ENDPOINTS
# ============================================================================

@tools_bp.route('/conversations/<int:conv_id>/tree', methods=['GET'])
@token_required
def get_conversation_tree(user, conv_id):
    """Get conversation as tree structure"""
    tree = ConversationManager.get_conversation_tree(conv_id)
    return jsonify(tree), 200


@tools_bp.route('/conversations/<int:conv_id>/branch', methods=['POST'])
@token_required
def branch_conversation(user, conv_id):
    """Create a conversation branch"""
    data = request.get_json() or {}
    from_message_id = data.get('from_message_id')
    new_title = data.get('new_title')
    
    if not from_message_id:
        return jsonify({'message': 'from_message_id is required'}), 400
    
    result = ConversationManager.branch_conversation(conv_id, from_message_id, new_title)
    return jsonify(result), 200 if result.get('success') else 400


@tools_bp.route('/conversations/<int:conv_id>/messages/<int:msg_id>', methods=['PUT'])
@token_required
def edit_message(user, conv_id, msg_id):
    """Edit a message"""
    data = request.get_json() or {}
    new_content = data.get('content')
    
    if not new_content:
        return jsonify({'message': 'content is required'}), 400
    
    result = ConversationManager.edit_message(conv_id, msg_id, new_content)
    return jsonify(result), 200 if result.get('success') else 400


@tools_bp.route('/conversations/<int:conv_id>/messages/<int:msg_id>', methods=['DELETE'])
@token_required
def delete_message(user, conv_id, msg_id):
    """Delete message and all subsequent messages"""
    result = ConversationManager.delete_message_and_replies(conv_id, msg_id)
    return jsonify(result), 200 if result.get('success') else 400


@tools_bp.route('/conversations/merge', methods=['POST'])
@token_required
def merge_conversations(user):
    """Merge two conversations"""
    data = request.get_json() or {}
    conv_id_1 = data.get('conversation_id_1')
    conv_id_2 = data.get('conversation_id_2')
    new_title = data.get('new_title')
    
    if not conv_id_1 or not conv_id_2:
        return jsonify({'message': 'conversation_id_1 and conversation_id_2 are required'}), 400
    
    result = ConversationManager.merge_conversations(conv_id_1, conv_id_2, new_title)
    return jsonify(result), 200 if result.get('success') else 400


# ============================================================================
# ANALYTICS ENDPOINTS
# ============================================================================

@tools_bp.route('/analytics/conversation/<int:conv_id>', methods=['GET'])
@token_required
def conversation_stats(user, conv_id):
    """Get stats for a conversation"""
    stats = ConversationAnalytics.get_conversation_stats(conv_id)
    return jsonify(stats), 200


@tools_bp.route('/analytics/user', methods=['GET'])
@token_required
def user_analytics(user):
    """Get analytics for current user"""
    analytics = ConversationAnalytics.get_user_analytics(user['id'])
    return jsonify(analytics), 200


@tools_bp.route('/analytics/search', methods=['POST'])
@token_required
def search_conversations(user):
    """Search conversations"""
    data = request.get_json() or {}
    query = data.get('query')
    
    if not query:
        return jsonify({'message': 'query is required'}), 400
    
    results = ConversationAnalytics.search_conversations(user['id'], query)
    return jsonify(results), 200


@tools_bp.route('/analytics/trending', methods=['GET'])
@token_required
def trending_topics(user):
    """Get trending topics in user's conversations"""
    limit = request.args.get('limit', 10, type=int)
    topics = ConversationAnalytics.get_trending_topics(user['id'], limit)
    return jsonify(topics), 200


# ============================================================================
# CUSTOM PROMPTS ENDPOINTS
# ============================================================================

@tools_bp.route('/prompts', methods=['GET'])
@token_required
def get_prompts(user):
    """Get user's custom prompts"""
    prompts = CustomPrompts.get_user_prompts(user['id'])
    return jsonify({'prompts': prompts}), 200


@tools_bp.route('/prompts', methods=['POST'])
@token_required
def create_prompt(user):
    """Create a custom prompt"""
    data = request.get_json() or {}
    system_prompt = data.get('system_prompt')
    name = data.get('name')
    
    if not system_prompt:
        return jsonify({'message': 'system_prompt is required'}), 400
    
    result = CustomPrompts.set_user_prompt(user['id'], system_prompt, name)
    return jsonify(result), 201 if result.get('success') else 400


@tools_bp.route('/prompts/<int:prompt_id>/apply', methods=['POST'])
@token_required
def apply_prompt(user, prompt_id):
    """Apply a custom prompt"""
    result = CustomPrompts.apply_prompt(user['id'], prompt_id)
    return jsonify(result), 200 if result.get('success') else 400


# ============================================================================
# FEATURE INFO ENDPOINT
# ============================================================================

@tools_bp.route('/features', methods=['GET'])
def list_features(user=None):
    """List all available tools and features (public endpoint)"""
    features = {
        'tools': {
            'calculator': 'Perform mathematical calculations',
            'web_search': 'Search the internet for information',
            'analyze_code': 'Analyze Python code for errors and improvements',
            'text_stats': 'Analyze text for statistics',
            'get_url_info': 'Get metadata about URLs'
        },
        'conversation_features': {
            'branching': 'Create alternate conversation paths',
            'history_editing': 'Edit messages and rewind conversations',
            'merging': 'Combine conversations',
            'search': 'Full-text search across conversations'
        },
        'analytics': {
            'conversation_stats': 'Detailed conversation statistics',
            'user_analytics': 'User behavior analytics',
            'trending_topics': 'Find most discussed topics',
            'conversation_search': 'Search conversations'
        },
        'customization': {
            'custom_prompts': 'Create and apply custom system prompts',
            'user_instructions': 'Configure AI behavior per user'
        }
    }
    
    return jsonify(features), 200

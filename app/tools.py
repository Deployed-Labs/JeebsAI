"""
JeebsAI Tools/Functions - Extended capabilities for the AI
Implements custom versions of tools offered by OpenAI, Anthropic, Google, etc.
"""

import json
import math
import requests
from datetime import datetime, timedelta
from functools import wraps
from .models import get_db
import time

# Tool Registry - Maps tool names to implementations
TOOLS_REGISTRY = {}

def register_tool(name, description, parameters):
    """Decorator to register a new tool"""
    def decorator(func):
        TOOLS_REGISTRY[name] = {
            'name': name,
            'description': description,
            'parameters': parameters,
            'handler': func
        }
        return func
    return decorator

# ============================================================================
# CALCULATOR TOOL
# ============================================================================

@register_tool(
    'calculator',
    'Perform mathematical calculations',
    {
        'expression': 'Math expression (e.g., "2 + 3 * 4")',
        'operation': 'Type: add, subtract, multiply, divide, power, sqrt, sin, cos, tan, log'
    }
)
def calculator(expression=None, operation=None, **kwargs):
    """Safe calculator tool"""
    try:
        if operation:
            a = kwargs.get('a', 0)
            b = kwargs.get('b', 0)
            
            if operation == 'add':
                result = a + b
            elif operation == 'subtract':
                result = a - b
            elif operation == 'multiply':
                result = a * b
            elif operation == 'divide':
                if b == 0:
                    return {'error': 'Division by zero'}
                result = a / b
            elif operation == 'power':
                result = a ** b
            elif operation == 'sqrt':
                if a < 0:
                    return {'error': 'Cannot take sqrt of negative number'}
                result = math.sqrt(a)
            elif operation == 'sin':
                result = math.sin(math.radians(a))
            elif operation == 'cos':
                result = math.cos(math.radians(a))
            elif operation == 'tan':
                result = math.tan(math.radians(a))
            elif operation == 'log':
                if a <= 0:
                    return {'error': 'Cannot take log of non-positive number'}
                result = math.log10(a)
            else:
                return {'error': f'Unknown operation: {operation}'}
            
            return {'result': round(result, 4), 'operation': operation}
        
        elif expression:
            # Safe eval for simple expressions
            safe_dict = {
                'sin': math.sin, 'cos': math.cos, 'tan': math.tan,
                'sqrt': math.sqrt, 'log': math.log10, 'log2': math.log2,
                'pi': math.pi, 'e': math.e
            }
            result = eval(expression, {"__builtins__": {}}, safe_dict)
            return {'result': round(result, 4), 'expression': expression}
    except Exception as e:
        return {'error': str(e)}
    
    return {'error': 'No calculation provided'}


# ============================================================================
# WEB SEARCH TOOL
# ============================================================================

@register_tool(
    'web_search',
    'Search the web for information',
    {
        'query': 'Search query string',
        'max_results': 'Maximum results to return (default 5)'
    }
)
def web_search(query, max_results=5, **kwargs):
    """Search the web using DuckDuckGo API (free, no key required)"""
    try:
        # Using DuckDuckGo instant answer API (no auth required)
        url = 'https://duckduckgo.com/'
        params = {
            'q': query,
            'format': 'json'
        }
        
        headers = {'User-Agent': 'Mozilla/5.0'}
        response = requests.get(url, params=params, headers=headers, timeout=5)
        
        if response.status_code != 200:
            return {'error': 'Search failed', 'query': query}
        
        data = response.json()
        
        # Extract results from DuckDuckGo response
        results = []
        
        # Abstract/instant answer
        if data.get('AbstractText'):
            results.append({
                'title': query,
                'snippet': data.get('AbstractText'),
                'url': data.get('AbstractURL')
            })
        
        # Related topics
        if data.get('RelatedTopics'):
            for item in data.get('RelatedTopics', [])[:max_results-1]:
                if isinstance(item, dict) and item.get('Text'):
                    results.append({
                        'title': item.get('FirstURL', '').split('/')[-1],
                        'snippet': item.get('Text'),
                        'url': item.get('FirstURL')
                    })
        
        return {
            'query': query,
            'results': results[:max_results],
            'count': len(results)
        }
    except Exception as e:
        return {'error': str(e), 'query': query}


# ============================================================================
# CODE ANALYZER TOOL
# ============================================================================

@register_tool(
    'analyze_code',
    'Analyze Python code for errors and improvements',
    {
        'code': 'Python code to analyze',
        'check_type': 'Type: syntax, performance, security, style'
    }
)
def analyze_code(code, check_type='syntax', **kwargs):
    """Analyze Python code"""
    try:
        import ast
        import re
        
        results = {
            'syntax_valid': True,
            'issues': [],
            'suggestions': []
        }
        
        # Check syntax
        try:
            ast.parse(code)
        except SyntaxError as e:
            results['syntax_valid'] = False
            results['issues'].append(f"Syntax Error at line {e.lineno}: {e.msg}")
            return results
        
        # Performance checks
        if check_type in ['performance', 'all']:
            if re.search(r'while True', code):
                results['suggestions'].append('Infinite loop detected - ensure exit condition')
            if re.search(r'import \*', code):
                results['suggestions'].append('Avoid "import *" - explicitly import needed modules')
        
        # Security checks
        if check_type in ['security', 'all']:
            dangerous = ['eval', 'exec', 'pickle.loads', 'os.system', 'subprocess.call']
            for danger in dangerous:
                if danger in code:
                    results['issues'].append(f'Security risk: {danger} detected')
        
        # Style checks
        if check_type in ['style', 'all']:
            lines = code.split('\n')
            for i, line in enumerate(lines, 1):
                if len(line) > 100:
                    results['suggestions'].append(f'Line {i} too long ({len(line)} chars)')
                if line.strip() and not line[0].isspace() and i > 1:
                    if re.match(r'^(def|class|if|for|while)', line):
                        pass  # OK
        
        results['lines'] = len(code.split('\n'))
        return results
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# TEXT STATISTICS TOOL
# ============================================================================

@register_tool(
    'text_stats',
    'Analyze text for statistics',
    {
        'text': 'Text to analyze',
        'include': 'What to include: words, sentences, paragraphs, readability'
    }
)
def text_stats(text, include='words,sentences', **kwargs):
    """Analyze text statistics"""
    try:
        stats = {}
        
        if 'words' in include:
            words = text.split()
            stats['word_count'] = len(words)
            stats['unique_words'] = len(set(w.lower() for w in words))
        
        if 'sentences' in include:
            sentences = text.split('.')
            stats['sentence_count'] = len([s for s in sentences if s.strip()])
            stats['avg_words_per_sentence'] = round(len(words) / max(1, len(sentences)), 2) if 'words' in include else 0
        
        if 'paragraphs' in include:
            paragraphs = text.split('\n\n')
            stats['paragraph_count'] = len([p for p in paragraphs if p.strip()])
        
        if 'readability' in include:
            # Simple readability score (0-100, higher = easier)
            words = text.split()
            sentences = len([s for s in text.split('.') if s.strip()])
            chars = len(text)
            
            if words and sentences:
                # Flesch Kincaid Grade Level
                grade = (0.39 * (len(words) / max(1, sentences)) + 
                        11.8 * (sum(1 for w in words if len(w) > 2) / max(1, len(words))) - 15.59)
                grade = max(0, min(18, grade))
                stats['readability_grade'] = round(grade, 1)
                stats['readability_notes'] = (
                    'Very Easy' if grade < 6 else
                    'Easy' if grade < 9 else
                    'Medium' if grade < 13 else
                    'Hard' if grade < 15 else
                    'Very Hard'
                )
        
        stats['character_count'] = len(text)
        return stats
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# URL METADATA TOOL
# ============================================================================

@register_tool(
    'get_url_info',
    'Get metadata about a URL without fetching full content',
    {
        'url': 'URL to check',
        'include': 'What to include: title, description, image, status'
    }
)
def get_url_info(url, include='title,status', **kwargs):
    """Get URL metadata"""
    try:
        if not url.startswith('http'):
            url = 'https://' + url
        
        headers = {'User-Agent': 'Mozilla/5.0'}
        response = requests.head(url, headers=headers, timeout=5, allow_redirects=True)
        
        info = {
            'url': response.url,
            'status_code': response.status_code,
            'status_text': {
                200: 'OK', 301: 'Moved', 401: 'Unauthorized', 403: 'Forbidden',
                404: 'Not Found', 500: 'Server Error'
            }.get(response.status_code, 'Unknown')
        }
        
        if 'title' in include or 'description' in include or 'image' in include:
            # Need to fetch content for these
            response = requests.get(url, headers=headers, timeout=5)
            response.encoding = 'utf-8'
            content = response.text
            
            import re
            
            if 'title' in include:
                match = re.search(r'<title[^>]*>([^<]+)</title>', content, re.IGNORECASE)
                info['title'] = match.group(1) if match else 'N/A'
            
            if 'description' in include:
                match = re.search(r'<meta\s+name=["\']description["\']\s+content=["\']([^"\']+)["\']', content, re.IGNORECASE)
                info['description'] = match.group(1) if match else 'N/A'
            
            if 'image' in include:
                match = re.search(r'<meta\s+property=["\']og:image["\']\s+content=["\']([^"\']+)["\']', content, re.IGNORECASE)
                info['image'] = match.group(1) if match else None
        
        return info
    
    except Exception as e:
        return {'error': str(e), 'url': url}


# ============================================================================
# RATE LIMITER
# ============================================================================

class RateLimiter:
    """Rate limiter for user actions"""
    
    @staticmethod
    def check_limit(user_id, action, limit_per_hour=100):
        """Check if user has exceeded rate limit"""
        try:
            db = get_db()
            now = datetime.now()
            hour_ago = now - timedelta(hours=1)
            
            # Track actions in memory or database
            cursor = db.execute(
                'SELECT COUNT(*) as count FROM actions WHERE user_id = ? AND action = ? AND created_at > ?',
                (user_id, action, hour_ago.isoformat())
            )
            result = cursor.fetchone()
            count = result['count'] if result else 0
            
            return count < limit_per_hour, limit_per_hour - count
        except:
            # If table doesn't exist, allow
            return True, limit_per_hour
    
    @staticmethod
    def record_action(user_id, action):
        """Record a user action"""
        try:
            db = get_db()
            db.execute(
                'INSERT INTO actions (user_id, action, created_at) VALUES (?, ?, ?)',
                (user_id, action, datetime.now().isoformat())
            )
            db.commit()
        except:
            pass  # Table might not exist


# ============================================================================
# TOOL EXECUTOR
# ============================================================================

def execute_tool(tool_name, **kwargs):
    """Execute a registered tool"""
    if tool_name not in TOOLS_REGISTRY:
        return {'error': f'Tool not found: {tool_name}'}
    
    tool = TOOLS_REGISTRY[tool_name]
    try:
        result = tool['handler'](**kwargs)
        return result
    except Exception as e:
        return {'error': str(e), 'tool': tool_name}


def get_available_tools():
    """Return list of available tools"""
    return [
        {
            'name': tool['name'],
            'description': tool['description'],
            'parameters': tool['parameters']
        }
        for tool in TOOLS_REGISTRY.values()
    ]

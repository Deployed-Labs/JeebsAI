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
    """Search the web using multiple sources with fallback"""
    results = []
    
    try:
        # Try DuckDuckGo instant answer first
        url = 'https://api.duckduckgo.com/'
        params = {
            'q': query,
            'format': 'json',
            'no_redirect': '1'
        }
        
        headers = {'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'}
        response = requests.get(url, params=params, headers=headers, timeout=5)
        
        if response.status_code == 200:
            data = response.json()
            
            # Get instant answer
            if data.get('AbstractText'):
                results.append({
                    'title': 'Direct Answer',
                    'snippet': data.get('AbstractText'),
                    'url': data.get('AbstractURL') or f'https://duckduckgo.com/?q={query}',
                    'source': 'instant_answer'
                })
            
            # Get related topics
            if data.get('RelatedTopics'):
                for item in data.get('RelatedTopics', [])[:max_results-len(results)]:
                    if isinstance(item, dict) and item.get('Text'):
                        results.append({
                            'title': item.get('FirstURL', query).split('/')[-1] or query,
                            'snippet': item.get('Text'),
                            'url': item.get('FirstURL') or f'https://duckduckgo.com/?q={query}',
                            'source': 'related'
                        })
            
            # Get definition
            if data.get('Definition'):
                results.insert(0, {
                    'title': 'Definition',
                    'snippet': data.get('Definition'),
                    'url': f'https://duckduckgo.com/?q={query}',
                    'source': 'definition'
                })
        
        # If no results, provide a fallback result
        if not results:
            results = [{
                'title': query.title(),
                'snippet': f'No specific information found about "{query}". Try searching with different keywords.',
                'url': f'https://duckduckgo.com/?q={query}',
                'source': 'fallback'
            }]
        
        return {
            'query': query,
            'results': results[:max_results],
            'count': len(results),
            'success': True
        }
    except Exception as e:
        return {
            'error': str(e),
            'query': query,
            'results': [{
                'title': 'Error',
                'snippet': f'Could not search: {str(e)}',
                'url': '',
                'source': 'error'
            }],
            'success': False
        }


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
# WIKIPEDIA SUMMARY TOOL
# ============================================================================

@register_tool(
    'wikipedia_summary',
    'Get a summary of a Wikipedia article',
    {
        'topic': 'Topic to look up on Wikipedia',
        'sentences': 'Number of sentences in summary (1-5, default 3)'
    }
)
def wikipedia_summary(topic, sentences=3, **kwargs):
    """Fetch Wikipedia summary"""
    try:
        # Use MediaWiki API
        url = 'https://en.wikipedia.org/w/api.php'
        params = {
            'action': 'query',
            'format': 'json',
            'titles': topic,
            'prop': 'extracts',
            'exintro': True,
            'explaintext': True,
            'redirects': 1
        }
        
        response = requests.get(url, params=params, timeout=10)
        data = response.json()
        
        # Get first page result
        if 'query' in data and 'pages' in data['query']:
            pages = data['query']['pages']
            page = next(iter(pages.values()))
            
            if 'extract' in page:
                extract = page['extract']
                title = page.get('title', topic)
                
                # Split into sentences and limit
                sent_list = [s.strip() + '.' for s in extract.replace('\n', ' ').split('.') if s.strip()]
                summary = ' '.join(sent_list[:int(sentences)])
                
                return {
                    'topic': title,
                    'summary': summary,
                    'url': f'https://en.wikipedia.org/wiki/{title.replace(" ", "_")}',
                    'success': True
                }
            else:
                return {'error': 'No extract found', 'topic': topic, 'success': False}
        else:
            return {'error': 'Topic not found on Wikipedia', 'topic': topic, 'success': False}
    
    except Exception as e:
        return {'error': str(e), 'topic': topic, 'success': False}


# ============================================================================
# NEWS AGGREGATOR TOOL
# ============================================================================

@register_tool(
    'latest_news',
    'Get latest news about a topic',
    {
        'topic': 'Topic to find news about',
        'count': 'Number of articles to return (default 3)'
    }
)
def latest_news(topic, count=3, **kwargs):
    """Get latest news using NewsAPI or DuckDuckGo"""
    try:
        # Try to get news via web search (since NewsAPI requires key)
        url = 'https://api.duckduckgo.com/'
        params = {
            'q': f'{topic} news',
            'format': 'json',
            'no_redirect': '1'
        }
        
        headers = {'User-Agent': 'Mozilla/5.0'}
        response = requests.get(url, params=params, headers=headers, timeout=10)
        
        if response.status_code == 200:
            data = response.json()
            articles = []
            
            # Extract from RelatedTopics
            if data.get('RelatedTopics'):
                for item in data.get('RelatedTopics', [])[:int(count)]:
                    if isinstance(item, dict) and item.get('Text'):
                        articles.append({
                            'title': item.get('FirstURL', '').split('/')[-1] or topic,
                            'description': item.get('Text'),
                            'url': item.get('FirstURL', '')
                        })
            
            return {
                'topic': topic,
                'articles': articles,
                'count': len(articles),
                'success': len(articles) > 0
            }
        
        return {'error': 'Could not fetch news', 'topic': topic, 'success': False}
    
    except Exception as e:
        return {'error': str(e), 'topic': topic, 'success': False}


# ============================================================================
# SENTIMENT ANALYSIS TOOL
# ============================================================================

@register_tool(
    'sentiment_analysis',
    'Analyze sentiment of text (positive, negative, neutral)',
    {
        'text': 'Text to analyze',
        'language': 'Language (en, es, fr, etc. - default: en)'
    }
)
def sentiment_analysis(text, language='en', **kwargs):
    """Analyze sentiment of text using simple rules"""
    try:
        # Simple sentiment analysis using word scoring
        positive_words = {
            'good', 'great', 'excellent', 'amazing', 'wonderful', 'beautiful', 'love', 'fantastic',
            'awesome', 'perfect', 'brilliant', 'brilliant', 'outstanding', 'superb', 'terrific',
            'incredible', 'delightful', 'magnificent', 'wonderful', 'happy', 'joy', 'glad', 'best'
        }
        
        negative_words = {
            'bad', 'terrible', 'horrible', 'awful', 'poor', 'hate', 'ugly', 'useless', 'never',
            'worst', 'disgusting', 'pathetic', 'dreadful', 'atrocious', 'vile', 'despicable',
            'sad', 'angry', 'upset', 'frustrated', 'annoyed', 'dislike', 'sucks'
        }
        
        words = text.lower().split()
        positive_count = sum(1 for w in words if w in positive_words)
        negative_count = sum(1 for w in words if w in negative_words)
        
        total = positive_count + negative_count
        
        if total == 0:
            sentiment = 'neutral'
            score = 0.5
        elif positive_count > negative_count:
            sentiment = 'positive'
            score = min(1.0, positive_count / (positive_count + negative_count + len(words) * 0.1))
        elif negative_count > positive_count:
            sentiment = 'negative'
            score = max(0.0, 1 - (negative_count / (positive_count + negative_count + len(words) * 0.1)))
        else:
            sentiment = 'neutral'
            score = 0.5
        
        return {
            'text': text[:100] + ('...' if len(text) > 100 else ''),
            'sentiment': sentiment,
            'confidence': round(abs(positive_count - negative_count) / max(1, total) * 100, 1),
            'positive_words': positive_count,
            'negative_words': negative_count,
            'score': round(score, 2)
        }
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# BRAIN STATISTICS TOOL
# ============================================================================

@register_tool(
    'brain_stats',
    'Get statistics about the brain\'s learning',
    {
        'user_id': 'User ID (optional)',
        'include': 'What to show: total_memories, top_topics, learning_rate'
    }
)
def brain_stats(user_id=None, include='total_memories,top_topics', **kwargs):
    """Get brain learning statistics"""
    try:
        from .models import get_db
        db = get_db()
        
        stats = {}
        
        # Total memories
        if 'total_memories' in include:
            cursor = db.execute('SELECT COUNT(*) as count FROM holographic_memories')
            result = cursor.fetchone()
            stats['total_memories'] = result['count'] if result else 0
        
        # Top topics
        if 'top_topics' in include:
            cursor = db.execute('''
                SELECT query, COUNT(*) as frequency FROM holographic_memories
                GROUP BY query ORDER BY frequency DESC LIMIT 10
            ''')
            results = cursor.fetchall()
            stats['top_topics'] = [
                {'topic': r['query'], 'frequency': r['frequency']} for r in results
            ] if results else []
        
        # Recent learning
        if 'recent_learning' in include:
            cursor = db.execute('''
                SELECT query, created_at FROM holographic_memories
                ORDER BY created_at DESC LIMIT 5
            ''')
            results = cursor.fetchall()
            stats['recent_learning'] = [
                {'query': r['query'], 'time': r['created_at']} for r in results
            ] if results else []
        
        stats['success'] = True
        return stats
    
    except Exception as e:
        return {'error': str(e), 'success': False}


# ============================================================================
# DEFINITION LOOKUP TOOL
# ============================================================================

@register_tool(
    'define_word',
    'Get definition and usage of a word',
    {
        'word': 'Word to define',
        'detailed': 'Get detailed info (true/false)'
    }
)
def define_word(word, detailed=False, **kwargs):
    """Get word definition"""
    try:
        # Use DuckDuckGo to get definition
        url = 'https://api.duckduckgo.com/'
        params = {
            'q': f'define {word}',
            'format': 'json',
            'no_redirect': '1'
        }
        
        headers = {'User-Agent': 'Mozilla/5.0'}
        response = requests.get(url, params=params, headers=headers, timeout=10)
        
        if response.status_code == 200:
            data = response.json()
            
            result = {
                'word': word,
                'definition': data.get('AbstractText') or data.get('Definition') or 'Definition not found'
            }
            
            if detailed:
                result['url'] = data.get('AbstractURL') or f'https://duckduckgo.com/?q=define+{word}'
                result['related'] = []
                if data.get('RelatedTopics'):
                    for item in data.get('RelatedTopics', [])[:3]:
                        if isinstance(item, dict) and item.get('Text'):
                            result['related'].append(item.get('Text'))
            
            return result
        else:
            return {'error': 'Could not find definition', 'word': word}
    
    except Exception as e:
        return {'error': str(e), 'word': word}


# ============================================================================
# UNIT CONVERTER TOOL
# ============================================================================

@register_tool(
    'convert_units',
    'Convert between different units',
    {
        'value': 'Value to convert',
        'from_unit': 'From unit (km, mi, kg, lb, C, F, etc.)',
        'to_unit': 'To unit'
    }
)
def convert_units(value, from_unit, to_unit, **kwargs):
    """Convert between units"""
    try:
        value = float(value)
        
        # Distance conversions
        distance_conversions = {
            'km': {'m': 1000, 'mi': 0.621371, 'km': 1, 'ft': 3280.84, 'yd': 1093.61},
            'm': {'km': 0.001, 'mi': 0.000621371, 'm': 1, 'ft': 3.28084, 'yd': 1.09361},
            'mi': {'km': 1.60934, 'm': 1609.34, 'mi': 1, 'ft': 5280, 'yd': 1760},
            'ft': {'km': 0.0003048, 'm': 0.3048, 'mi': 0.000189394, 'ft': 1, 'yd': 0.333333},
            'yd': {'km': 0.0009144, 'm': 0.9144, 'mi': 0.000568182, 'ft': 3, 'yd': 1}
        }
        
        # Weight conversions
        weight_conversions = {
            'kg': {'g': 1000, 'lb': 2.20462, 'oz': 35.274, 'kg': 1},
            'g': {'kg': 0.001, 'lb': 0.00220462, 'oz': 0.035274, 'g': 1},
            'lb': {'kg': 0.453592, 'g': 453.592, 'lb': 1, 'oz': 16},
            'oz': {'kg': 0.0283495, 'g': 28.3495, 'lb': 0.0625, 'oz': 1}
        }
        
        # Temperature conversions
        if from_unit in ['C', 'F', 'K'] and to_unit in ['C', 'F', 'K']:
            if from_unit == 'C':
                c_value = value
            elif from_unit == 'F':
                c_value = (value - 32) * 5/9
            else:  # K
                c_value = value - 273.15
            
            if to_unit == 'C':
                result = c_value
            elif to_unit == 'F':
                result = c_value * 9/5 + 32
            else:  # K
                result = c_value + 273.15
            
            return {
                'value': value,
                'from': from_unit,
                'to': to_unit,
                'result': round(result, 2)
            }
        
        # Check distance
        if from_unit in distance_conversions and to_unit in distance_conversions[from_unit]:
            result = value * distance_conversions[from_unit][to_unit]
            return {
                'value': value,
                'from': from_unit,
                'to': to_unit,
                'result': round(result, 4)
            }
        
        # Check weight
        if from_unit in weight_conversions and to_unit in weight_conversions[from_unit]:
            result = value * weight_conversions[from_unit][to_unit]
            return {
                'value': value,
                'from': from_unit,
                'to': to_unit,
                'result': round(result, 4)
            }
        
        return {'error': f'Conversion from {from_unit} to {to_unit} not supported'}
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# PASSWORD GENERATOR TOOL
# ============================================================================

@register_tool(
    'generate_password',
    'Generate a secure random password',
    {
        'length': 'Password length (default 16)',
        'include_symbols': 'Include symbols (true/false)'
    }
)
def generate_password(length=16, include_symbols=True, **kwargs):
    """Generate secure password"""
    try:
        import random
        import string
        
        length = int(length)
        if length < 4 or length > 100:
            length = 16
        
        chars = string.ascii_letters + string.digits
        if include_symbols:
            chars += '!@#$%^&*()_+-=[]{}|;:,.<>?'
        
        password = ''.join(random.choice(chars) for _ in range(length))
        
        # Calculate strength
        has_upper = any(c.isupper() for c in password)
        has_lower = any(c.islower() for c in password)
        has_digit = any(c.isdigit() for c in password)
        has_symbol = any(c in string.punctuation for c in password)
        
        strength_score = sum([has_upper, has_lower, has_digit, has_symbol])
        strength = {1: 'Weak', 2: 'Fair', 3: 'Good', 4: 'Strong'}.get(strength_score, 'Weak')
        
        return {
            'password': password,
            'length': length,
            'strength': strength,
            'entropy': round(math.log2(len(chars) ** length), 1)
        }
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# JSON FORMATTER TOOL
# ============================================================================

@register_tool(
    'format_json',
    'Format and validate JSON',
    {
        'json_text': 'JSON string to format',
        'minify': 'Minify instead of pretty-print (true/false)'
    }
)
def format_json(json_text, minify=False, **kwargs):
    """Format JSON"""
    try:
        data = json.loads(json_text)
        
        if minify:
            result = json.dumps(data, separators=(',', ':'))
        else:
            result = json.dumps(data, indent=2)
        
        return {
            'formatted': result,
            'valid': True,
            'size_before': len(json_text),
            'size_after': len(result)
        }
    
    except json.JSONDecodeError as e:
        return {'error': f'Invalid JSON: {e.msg}', 'valid': False}
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# COLOR CONVERTER TOOL  
# ============================================================================

@register_tool(
    'convert_color',
    'Convert between color formats (hex, rgb, hsl)',
    {
        'color': 'Color value (#FFF, rgb(255,255,255), hsl(0,100%,100%))',
        'to_format': 'Target format (hex, rgb, hsl)'
    }
)
def convert_color(color, to_format='hex', **kwargs):
    """Convert between color formats"""
    try:
        import re
        
        color = color.strip()
        result = {}
        
        # Parse hex
        hex_match = re.match(r'^#?([0-9A-Fa-f]{3}|[0-9A-Fa-f]{6})$', color)
        if hex_match:
            hex_val = hex_match.group(1)
            if len(hex_val) == 3:
                hex_val = ''.join([c*2 for c in hex_val])
            r, g, b = int(hex_val[0:2], 16), int(hex_val[2:4], 16), int(hex_val[4:6], 16)
            result['hex'] = f'#{hex_val.upper()}'
            result['rgb'] = f'rgb({r}, {g}, {b})'
        
        # Parse RGB
        rgb_match = re.match(r'rgb\((\d+),\s*(\d+),\s*(\d+)\)', color, re.IGNORECASE)
        if rgb_match:
            r, g, b = int(rgb_match.group(1)), int(rgb_match.group(2)), int(rgb_match.group(3))
            result['rgb'] = f'rgb({r}, {g}, {b})'
            result['hex'] = f'#{r:02x}{g:02x}{b:02x}'.upper()
        
        if result:
            return {
                'input': color,
                'conversions': result,
                'success': True
            }
        else:
            return {'error': f'Could not parse color: {color}'}
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# RANDOM QUOTE TOOL
# ============================================================================

@register_tool(
    'get_quote',
    'Get a random inspirational quote',
    {
        'category': 'Quote category (motivational, funny, wise, etc.)'
    }
)
def get_quote(category='motivational', **kwargs):
    """Get random quote"""
    try:
        quotes = {
            'motivational': [
                ("The only way to do great work is to love what you do.", "Steve Jobs"),
                ("It is during our darkest moments that we must focus to see the light.", "Aristotle"),
                ("The future belongs to those who believe in the beauty of their dreams.", "Eleanor Roosevelt"),
                ("It is impossible to live without failing at something.", "J.K. Rowling"),
                ("Success is not final, failure is not fatal.", "Winston Churchill"),
            ],
            'funny': [
                ("Why fit in when you were born to stand out?", "Dr. Seuss"),
                ("Two things are infinite: the universe and human stupidity.", "Albert Einstein"),
                ("Be yourself; everyone else is already taken.", "Oscar Wilde"),
                ("The early bird gets the worm, but the second mouse gets the cheese.", "Unknown"),
                ("I'm not insane, my mother had me tested.", "Sherlock Holmes"),
            ],
            'wise': [
                ("The only true wisdom is in knowing you know nothing.", "Socrates"),
                ("Knowledge speaks, but wisdom listens.", "Jimi Hendrix"),
                ("Turn your wounds into wisdom.", "Oprah Winfrey"),
                ("The journey is more important than the destination.", "Ursula K. Le Guin"),
                ("Life is what happens when you're busy making other plans.", "John Lennon"),
            ]
        }
        
        import random
        cat = category.lower() if category.lower() in quotes else 'motivational'
        quote, author = random.choice(quotes[cat])
        
        return {
            'quote': quote,
            'author': author,
            'category': cat
        }
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# JOKE GENERATOR TOOL
# ============================================================================

@register_tool(
    'get_joke',
    'Get a random joke',
    {
        'category': 'Joke category (programming, knock-knock, etc.)'
    }
)
def get_joke(category='programming', **kwargs):
    """Get random joke"""
    try:
        jokes = {
            'programming': [
                "How many programmers does it take to change a light bulb? None, that's a hardware problem!",
                "Why do Java developers wear glasses? Because they don't C#!",
                "Why did the developer go broke? Because he used up all his cache!",
                "How many programmers does it take to change a light bulb? Zero, that's what 'dark mode' is for!",
                "Why do programmers prefer dark mode? Because light attracts bugs!",
            ],
            'knock-knock': [
                "Knock knock. Who's there? Interrupting function. Interru-- return 42;",
                "Knock knock. Who's there? Cache. Cache who? Cache me outside!",
                "Knock knock. Who's there? Recursion. Recursion who? Recursion who?",
                "Knock knock. Who's there? Boolean. Boolean who? Boolean value!",
            ],
            'general': [
                "What do you call a sleeping bull? A dozer!",
                "Why don't scientists trust atoms? Because they make up everything!",
                "Did you hear about the mathematician who's afraid of negative numbers? He'll stop at nothing!",
                "What do you call a bear with no teeth? A gummy bear!",
            ]
        }
        
        import random
        cat = category.lower() if category.lower() in jokes else 'programming'
        joke = random.choice(jokes[cat])
        
        return {
            'joke': joke,
            'category': cat
        }
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# FUN FACTS TOOL
# ============================================================================

@register_tool(
    'fun_fact',
    'Get a random fun fact',
    {
        'category': 'Category (science, nature, history, etc.)'
    }
)
def fun_fact(category='science', **kwargs):
    """Get fun fact"""
    try:
        facts = {
            'science': [
                "Honey never spoils and archaeologists have found pots of honey in ancient Egyptian tombs that are over 3000 years old!",
                "A single bolt of lightning is about 5 times hotter than the surface of the sun!",
                "Octopuses have three hearts - two pump blood to the gills, one pumps it to the rest of the body!",
                "Bananas are berries, but strawberries aren't!",
                "A group of flamingos is called a 'flamboyance'!",
            ],
            'nature': [
                "Butterflies taste with their feet!",
                "Sharks have been around longer than dinosaurs!",
                "A cockroach can live for a week without its head!",
                "Dolphins have names for each other!",
                "Penguins propose to their mates with a pebble!",
            ],
            'history': [
                "The Great Wall of China isn't visible from space!",
                "Cleopatra lived closer to the moon landing than to the building of the Great Pyramid!",
                "Oxford University is older than the Aztec Empire!",
                "Woolly mammoths roamed the Earth when the pyramids were being built!",
            ]
        }
        
        import random
        cat = category.lower() if category.lower() in facts else 'science'
        fact = random.choice(facts[cat])
        
        return {
            'fact': fact,
            'category': cat
        }
    
    except Exception as e:
        return {'error': str(e)}


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

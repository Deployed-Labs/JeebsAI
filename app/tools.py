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
def analyze_code(code, check_type='full', **kwargs):
    """Analyze Python code for errors and improvements"""
    try:
        import ast
        import re
        
        results = {
            'syntax_valid': True,
            'issues': [],
            'suggestions': [],
            'lines': 0,
            'coverage': ''
        }
        
        # Check syntax
        try:
            ast.parse(code)
            results['syntax_valid'] = True
        except SyntaxError as e:
            results['syntax_valid'] = False
            results['issues'].append({
                'type': 'Syntax Error',
                'line': e.lineno,
                'message': e.msg if e.msg else 'Invalid Python syntax'
            })
            return results  # Stop if syntax error
        
        lines = code.split('\n')
        results['lines'] = len(lines)
        
        # Performance checks
        if check_type in ['performance', 'full']:
            # Check for common performance issues
            for i, line in enumerate(lines, 1):
                if re.search(r'while\s+True', line):
                    results['suggestions'].append({
                        'type': 'Performance',
                        'line': i,
                        'message': 'Infinite loop detected - ensure proper exit condition'
                    })
                if re.search(r'import\s+\*', line):
                    results['suggestions'].append({
                        'type': 'Performance',
                        'line': i,
                        'message': 'Avoid "import *" - explicitly import needed modules'
                    })
                if re.search(r'for.*in.*range.*len', line):
                    results['suggestions'].append({
                        'type': 'Performance',
                        'line': i,
                        'message': 'Consider using "for item in list:" instead of "for i in range(len(list)):"'
                    })
        
        # Security checks
        if check_type in ['security', 'full']:
            dangerous = {
                'eval': 'Remote code execution vulnerability',
                'exec': 'Code injection vulnerability',
                'pickle.loads': 'Arbitrary code execution vulnerability',
                'os.system': 'Command injection risk',
                'subprocess.call': 'Use subprocess.run() with shell=False instead'
            }
            
            for i, line in enumerate(lines, 1):
                for danger, warning in dangerous.items():
                    if danger in line and not line.strip().startswith('#'):
                        results['issues'].append({
                            'type': 'Security Risk',
                            'line': i,
                            'message': warning
                        })
        
        # Style checks
        if check_type in ['style', 'full']:
            for i, line in enumerate(lines, 1):
                if len(line) > 100:
                    results['suggestions'].append({
                        'type': 'Style',
                        'line': i,
                        'message': f'Line too long ({len(line)} chars) - consider breaking it up'
                    })
                
                if line.strip() and line[0].isspace() and not line[0].isspace() * 4:
                    results['suggestions'].append({
                        'type': 'Style',
                        'line': i,
                        'message': 'Indentation should be multiples of 4 spaces'
                    })
        
        # Summary
        results['coverage'] = f"Checked {len(lines)} lines: {len(results['issues'])} issues, {len(results['suggestions'])} suggestions"
        results['success'] = True
        
        return results
    
    except Exception as e:
        return {'error': f'Code analysis failed: {str(e)}', 'success': False}


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
    """Get URL metadata with better error handling"""
    try:
        if not url.startswith('http'):
            url = 'https://' + url
        
        # Validate URL format
        if not url or ' ' in url:
            return {'error': 'Invalid URL format', 'url': url, 'success': False}
        
        headers = {
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
            'Accept': 'text/html,application/xhtml+xml'
        }
        
        info = {
            'url': url,
            'accessible': False,
            'success': False
        }
        
        try:
            # Try HEAD request first (faster)
            response = requests.head(url, headers=headers, timeout=5, allow_redirects=True)
            info['url'] = response.url  # Get final URL after redirects
            info['status_code'] = response.status_code
            info['accessible'] = response.status_code < 400
            
            status_map = {
                200: 'OK', 301: 'Moved Permanently', 302: 'Moved Temporarily',
                401: 'Unauthorized', 403: 'Forbidden', 404: 'Not Found',
                500: 'Server Error', 503: 'Service Unavailable'
            }
            info['status_text'] = status_map.get(response.status_code, f'HTTP {response.status_code}')
            
            # Get additional metadata if requested and status is OK
            if response.status_code == 200 and ('title' in include or 'description' in include or 'image' in include):
                try:
                    response_get = requests.get(url, headers=headers, timeout=5)
                    response_get.encoding = 'utf-8'
                    content = response_get.text[:50000]  # Limit to 50KB
                    
                    import re
                    
                    if 'title' in include:
                        match = re.search(r'<title[^>]*>([^<]+)</title>', content, re.IGNORECASE)
                        info['title'] = match.group(1).strip() if match else 'No title found'
                    
                    if 'description' in include:
                        match = re.search(r'<meta\s+name=["\']description["\']\s+content=["\']([^"\']+)["\']', content, re.IGNORECASE)
                        info['description'] = match.group(1).strip() if match else 'No description'
                    
                    if 'image' in include:
                        match = re.search(r'<meta\s+property=["\']og:image["\']\s+content=["\']([^"\']+)["\']', content, re.IGNORECASE)
                        info['image'] = match.group(1) if match else None
                except requests.Timeout:
                    info['warning'] = 'Full content fetch timed out, showing header info only'
                except Exception:
                    info['warning'] = 'Could not fetch full page content'
            
            info['success'] = True
            return info
            
        except requests.Timeout:
            return {'error': 'Request timed out - website may be slow or unreachable', 'url': url, 'success': False}
        except requests.ConnectionError:
            return {'error': 'Connection failed - check the URL or network', 'url': url, 'success': False}
        except Exception as e:
            return {'error': f'Request failed: {type(e).__name__}', 'url': url, 'success': False}
    
    except Exception as e:
        return {'error': f'URL info failed: {str(e)}', 'success': False}


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
    """Get latest news using web search with news-focused queries"""
    try:
        # Add news keywords to the query for better results
        search_query = f'{topic} latest news recent'
        
        url = 'https://api.duckduckgo.com/'
        params = {
            'q': search_query,
            'format': 'json',
            'no_redirect': '1',
            'kl': 'en-us'
        }
        
        headers = {'User-Agent': 'Mozilla/5.0'}
        response = requests.get(url, params=params, headers=headers, timeout=10)
        
        if response.status_code == 200:
            data = response.json()
            articles = []
            
            # Try AbstractText first (highest quality)
            if data.get('AbstractText') and data.get('AbstractURL'):
                articles.append({
                    'title': data.get('Heading', topic.title() + ' News'),
                    'description': data.get('AbstractText'),
                    'url': data.get('AbstractURL'),
                    'source': 'direct'
                })
            
            # Extract from RelatedTopics for more articles
            if data.get('RelatedTopics'):
                for item in data.get('RelatedTopics', [])[:max(0, int(count)-len(articles))]:
                    if isinstance(item, dict):
                        if item.get('Text'):
                            title = item.get('FirstURL', '').split('/')[-1]
                            if not title:
                                title = topic.title() + ' Info'
                            
                            articles.append({
                                'title': title,
                                'description': item.get('Text')[:200] if item.get('Text') else 'No description',
                                'url': item.get('FirstURL', f'https://duckduckgo.com/?q={search_query}'),
                                'source': 'related'
                            })
            
            # If still no results, provide helpful message
            if not articles:
                articles = [{
                    'title': f'{topic} Information',
                    'description': f'No recent news found. Try a different search term or check major news sites for {topic}.',
                    'url': f'https://duckduckgo.com/?q={search_query}',
                    'source': 'fallback'
                }]
            
            return {
                'topic': topic,
                'query': search_query,
                'articles': articles[:int(count)],
                'count': len(articles),
                'success': True
            }
        
        return {'error': 'Could not fetch news', 'topic': topic, 'success': False}
    
    except Exception as e:
        return {'error': f'News search failed: {str(e)}', 'topic': topic, 'success': False}


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
# BASE64 ENCODER/DECODER TOOL
# ============================================================================

@register_tool(
    'base64_encode_decode',
    'Encode text to Base64 or decode Base64 to text',
    {
        'text': 'Text to encode or decode',
        'action': 'Action: encode or decode'
    }
)
def base64_encode_decode(text, action='encode', **kwargs):
    """Encode/decode Base64"""
    try:
        import base64
        
        if action.lower() == 'encode':
            encoded = base64.b64encode(text.encode()).decode()
            return {
                'action': 'encode',
                'input': text[:50] + ('...' if len(text) > 50 else ''),
                'output': encoded,
                'length': len(encoded)
            }
        elif action.lower() == 'decode':
            decoded = base64.b64decode(text).decode()
            return {
                'action': 'decode',
                'input': text[:50] + ('...' if len(text) > 50 else ''),
                'output': decoded,
                'length': len(decoded)
            }
        else:
            return {'error': 'Action must be encode or decode'}
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# HASH GENERATOR TOOL
# ============================================================================

@register_tool(
    'hash_generator',
    'Generate cryptographic hashes of text',
    {
        'text': 'Text to hash',
        'algorithm': 'Hash algorithm (md5, sha1, sha256, sha512)'
    }
)
def hash_generator(text, algorithm='sha256', **kwargs):
    """Generate cryptographic hashs"""
    try:
        import hashlib
        
        algo = algorithm.lower()
        
        if algo == 'md5':
            hash_result = hashlib.md5(text.encode()).hexdigest()
        elif algo == 'sha1':
            hash_result = hashlib.sha1(text.encode()).hexdigest()
        elif algo == 'sha256':
            hash_result = hashlib.sha256(text.encode()).hexdigest()
        elif algo == 'sha512':
            hash_result = hashlib.sha512(text.encode()).hexdigest()
        else:
            return {'error': f'Unknown algorithm: {algo}'}
        
        return {
            'text': text[:50] + ('...' if len(text) > 50 else ''),
            'algorithm': algo,
            'hash': hash_result,
            'length': len(hash_result)
        }
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# UUID GENERATOR TOOL
# ============================================================================

@register_tool(
    'generate_uuid',
    'Generate unique identifiers (UUIDs)',
    {
        'count': 'Number of UUIDs to generate (default 1)',
        'version': 'UUID version (4 for random, 1 for timestamp)'
    }
)
def generate_uuid(count=1, version=4, **kwargs):
    """Generate UUIDs"""
    try:
        import uuid
        
        count = min(int(count), 10)  # Max 10 at a time
        uuids = []
        
        for _ in range(count):
            if version == 1:
                uuids.append(str(uuid.uuid1()))
            else:
                uuids.append(str(uuid.uuid4()))
        
        return {
            'count': len(uuids),
            'version': version,
            'uuids': uuids
        }
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# REGEX PATTERN MATCHER TOOL
# ============================================================================

@register_tool(
    'regex_match',
    'Test text against regex patterns',
    {
        'text': 'Text to search',
        'pattern': 'Regex pattern to match',
        'action': 'Action: match, findall, substitute'
    }
)
def regex_match(text, pattern, action='match', replacement='', **kwargs):
    """Match regex patterns"""
    try:
        import re
        
        if action == 'match':
            match = re.search(pattern, text)
            if match:
                return {
                    'action': 'match',
                    'found': True,
                    'match': match.group(0),
                    'groups': match.groups(),
                    'start': match.start(),
                    'end': match.end()
                }
            else:
                return {'action': 'match', 'found': False}
        
        elif action == 'findall':
            matches = re.findall(pattern, text)
            return {
                'action': 'findall',
                'count': len(matches),
                'matches': matches[:10]  # Max 10
            }
        
        elif action == 'substitute':
            result = re.sub(pattern, replacement, text)
            return {
                'action': 'substitute',
                'original': text[:50],
                'result': result[:50],
                'changes': len(re.findall(pattern, text))
            }
        
        else:
            return {'error': 'Unknown action'}
    
    except re.error as e:
        return {'error': f'Regex error: {e}'}
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# MARKDOWN TO HTML CONVERTER TOOL
# ============================================================================

@register_tool(
    'markdown_to_html',
    'Convert Markdown to HTML',
    {
        'markdown': 'Markdown text to convert'
    }
)
def markdown_to_html(markdown, **kwargs):
    """Convert Markdown to HTML"""
    try:
        import re
        
        html = markdown
        
        # Headers
        html = re.sub(r'^### (.*?)$', r'<h3>\1</h3>', html, flags=re.MULTILINE)
        html = re.sub(r'^## (.*?)$', r'<h2>\1</h2>', html, flags=re.MULTILINE)
        html = re.sub(r'^# (.*?)$', r'<h1>\1</h1>', html, flags=re.MULTILINE)
        
        # Bold and italic
        html = re.sub(r'\*\*(.*?)\*\*', r'<strong>\1</strong>', html)
        html = re.sub(r'\*(.*?)\*', r'<em>\1</em>', html)
        html = re.sub(r'__(.*?)__', r'<strong>\1</strong>', html)
        html = re.sub(r'_(.*?)_', r'<em>\1</em>', html)
        
        # Links
        html = re.sub(r'\[(.*?)\]\((.*?)\)', r'<a href="\2">\1</a>', html)
        
        # Code blocks
        html = re.sub(r'```(.*?)```', r'<pre><code>\1</code></pre>', html, flags=re.DOTALL)
        html = re.sub(r'`(.*?)`', r'<code>\1</code>', html)
        
        # Line breaks
        html = html.replace('\n\n', '</p><p>')
        html = '<p>' + html + '</p>'
        
        return {
            'html': html,
            'length': len(html)
        }
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# IP ADDRESS INFO TOOL
# ============================================================================

@register_tool(
    'ip_info',
    'Get information about an IP address',
    {
        'ip': 'IP address to look up'
    }
)
def ip_info(ip, **kwargs):
    """Get IP address information"""
    try:
        import socket
        
        # Basic validation
        parts = ip.split('.')
        if len(parts) != 4 or not all(p.isdigit() and 0 <= int(p) <= 255 for p in parts):
            return {'error': 'Invalid IP address'}
        
        # Try hostname lookup
        try:
            hostname = socket.gethostbyaddr(ip)[0]
        except:
            hostname = 'N/A'
        
        # Check IP type
        ip_int = sum(int(p) << (8 * (3 - i)) for i, p in enumerate(parts))
        
        if parts[0] == '127':
            ip_type = 'Loopback'
        elif parts[0] == '192' and parts[1] == '168':
            ip_type = 'Private (RFC1918)'
        elif parts[0] == '10':
            ip_type = 'Private (RFC1918)'
        elif parts[0] == '172' and 16 <= int(parts[1]) <= 31:
            ip_type = 'Private (RFC1918)'
        elif parts[0] in ['224', '225', '226', '227', '228', '229', '230', '231', '232', '233', '234', '235', '236', '237', '238', '239']:
            ip_type = 'Multicast'
        else:
            ip_type = 'Public'
        
        return {
            'ip': ip,
            'hostname': hostname,
            'type': ip_type,
            'binary': '.'.join(format(int(p), '08b') for p in parts)
        }
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# TIMEZONE CONVERTER TOOL
# ============================================================================

@register_tool(
    'timezone_convert',
    'Convert times between timezones',
    {
        'hour': 'Hour (0-23)',
        'from_tz': 'From timezone offset (e.g., -5 for EST)',
        'to_tz': 'To timezone offset (e.g., 1 for CET)'
    }
)
def timezone_convert(hour, from_tz, to_tz, **kwargs):
    """Convert between timezones"""
    try:
        hour = int(hour)
        from_tz = int(from_tz)
        to_tz = int(to_tz)
        
        if not 0 <= hour <= 23:
            return {'error': 'Hour must be 0-23'}
        
        offset_diff = to_tz - from_tz
        new_hour = (hour + offset_diff) % 24
        
        tz_names = {
            -12: 'Baker Island', -11: 'SST', -10: 'HST', -9: 'AKST', -8: 'PST',
            -7: 'MST', -6: 'CST', -5: 'EST', -4: 'EDT', -3: 'ART', -2: 'GST',
            -1: 'AZT', 0: 'GMT', 1: 'CET', 2: 'EET', 3: 'MSK', 4: 'AZT',
            5: 'PKT', 6: 'BST', 7: 'ICT', 8: 'CST', 9: 'JST', 10: 'AEST',
            11: 'AEDT', 12: 'NZDT'
        }
        
        return {
            'from_hour': hour,
            'from_tz': f'UTC{from_tz:+d}',
            'from_name': tz_names.get(from_tz, 'Unknown'),
            'to_hour': new_hour,
            'to_tz': f'UTC{to_tz:+d}',
            'to_name': tz_names.get(to_tz, 'Unknown'),
            'offset': offset_diff
        }
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# EQUATION SOLVER TOOL
# ============================================================================

@register_tool(
    'solve_equation',
    'Solve simple linear equations',
    {
        'equation': 'Equation to solve (e.g., "2x + 5 = 13", solve for x)'
    }
)
def solve_equation(equation, **kwargs):
    """Solve simple equations"""
    try:
        import re
        
        # Simple linear equation solver
        # Format: ax + b = c
        equation = equation.replace(' ', '').replace('=', '=').split('=')
        
        if len(equation) != 2:
            return {'error': 'Invalid equation format'}
        
        left, right = equation[0], equation[1]
        
        # Try to extract a, b from ax + b
        left_match = re.match(r'([+-]?\d*)x([+-]?\d+)?', left)
        right_val = float(right)
        
        if left_match:
            a = left_match.group(1)
            a = int(a) if a and a != '+' else 1 if a != '-' else -1
            
            b = left_match.group(2)
            b = int(b) if b else 0
            
            if a == 0:
                return {'error': 'No solution or infinite solutions'}
            
            x = (right_val - b) / a
            
            return {
                'equation': equation[0] + '=' + equation[1],
                'x': round(x, 4),
                'verification': f'{a}*{x} + {b} = {a*x + b}'
            }
        
        return {'error': 'Could not parse equation'}
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# CRON EXPRESSION PARSER TOOL
# ============================================================================

@register_tool(
    'parse_cron',
    'Parse and explain cron expressions',
    {
        'cron': 'Cron expression (e.g., "0 9 * * 1")'
    }
)
def parse_cron(cron, **kwargs):
    """Parse cron expressions"""
    try:
        parts = cron.split()
        if len(parts) != 5:
            return {'error': 'Cron must have 5 fields: minute hour day month weekday'}
        
        minute, hour, day, month, weekday = parts
        
        day_names = {0: 'Mon', 1: 'Tue', 2: 'Wed', 3: 'Thu', 4: 'Fri', 5: 'Sat', 6: 'Sun'}
        month_names = {
            1: 'Jan', 2: 'Feb', 3: 'Mar', 4: 'Apr', 5: 'May', 6: 'Jun',
            7: 'Jul', 8: 'Aug', 9: 'Sep', 10: 'Oct', 11: 'Nov', 12: 'Dec'
        }
        
        # Simple parsing
        minute_desc = 'every minute' if minute == '*' else f'minute {minute}'
        hour_desc = 'every hour' if hour == '*' else f'{hour}:00'
        day_desc = 'every day' if day == '*' else f'day {day}'
        month_desc = 'every month' if month == '*' else f'{month_names.get(int(month), month)}'
        weekday_desc = 'any weekday' if weekday == '*' else day_names.get(int(weekday), weekday)
        
        description = f'Runs at {hour_desc} on {weekday_desc}'
        
        return {
            'cron': cron,
            'minute': minute,
            'hour': hour,
            'day': day,
            'month': month,
            'weekday': weekday,
            'description': description,
            'readable': f'{minute_desc}, {hour_desc}, {day_desc}, {month_desc}, {weekday_desc}'
        }
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# QR CODE TEXT GENERATOR TOOL
# ============================================================================

@register_tool(
    'generate_qr_ascii',
    'Generate ASCII representation of QR code data',
    {
        'text': 'Text to encode in QR code',
        'size': 'Size: small, medium, large'
    }
)
def generate_qr_ascii(text, size='medium', **kwargs):
    """Generate ASCII QR code representation"""
    try:
        import hashlib
        
        # Create a simple ASCII representation based on hash
        hash_val = hashlib.md5(text.encode()).hexdigest()
        
        # Convert to grid
        grid = []
        for i in range(0, len(hash_val), 2):
            byte = int(hash_val[i:i+2], 16)
            row = ''
            for j in range(8):
                row += '██' if byte & (1 << j) else '  '
            grid.append(row)
        
        # Create frame
        ascii_qr = '  ┌' + '─' * 16 + '┐\n'
        for row in grid[:2]:
            ascii_qr += '  │' + row[:16] + '│\n'
        ascii_qr += '  │' + text[:16].ljust(16) + '│\n'
        for row in grid[2:4]:
            ascii_qr += '  │' + row[:16] + '│\n'
        ascii_qr += '  └' + '─' * 16 + '┘\n'
        
        return {
            'text': text[:50],
            'qr_ascii': ascii_qr,
            'hash': hash_val,
            'size': size
        }
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# FILE SIZE CALCULATOR TOOL
# ============================================================================

@register_tool(
    'file_size_calc',
    'Calculate and convert file sizes',
    {
        'size': 'Size value',
        'from_unit': 'From unit (bytes, kb, mb, gb, tb)',
        'to_unit': 'To unit'
    }
)
def file_size_calc(size, from_unit='mb', to_unit='gb', **kwargs):
    """Calculate file sizes"""
    try:
        size = float(size)
        
        units = {
            'bytes': 1,
            'byte': 1,
            'b': 1,
            'kb': 1024,
            'mb': 1024**2,
            'gb': 1024**3,
            'tb': 1024**4
        }
        
        from_u = from_unit.lower()
        to_u = to_unit.lower()
        
        if from_u not in units or to_u not in units:
            return {'error': f'Unknown unit. Available: {list(units.keys())}'}
        
        # Convert to bytes then to target
        bytes_val = size * units[from_u]
        result = bytes_val / units[to_u]
        
        return {
            'original': f'{size} {from_u}',
            'converted': round(result, 4),
            'unit': to_u,
            'bytes': bytes_val
        }
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# DATA VISUALIZATION TOOL
# ============================================================================

@register_tool(
    'ascii_chart',
    'Create ASCII charts from data',
    {
        'data': 'Space-separated numbers (e.g., "10 20 15 30")',
        'chart_type': 'Type: bar, line'
    }
)
def ascii_chart(data, chart_type='bar', **kwargs):
    """Create ASCII charts"""
    try:
        numbers = [float(x) for x in data.split()]
        
        if not numbers:
            return {'error': 'No data provided'}
        
        max_val = max(numbers)
        min_val = min(numbers)
        
        if chart_type.lower() == 'bar':
            chart = ''
            max_bar = 20
            
            for i, num in enumerate(numbers):
                bar_length = int((num / max_val) * max_bar) if max_val > 0 else 0
                bar = '█' * bar_length + '░' * (max_bar - bar_length)
                chart += f'{num:>6.1f} │ {bar}\n'
            
            return {
                'chart_type': 'bar',
                'data': numbers,
                'chart': chart,
                'max': max_val,
                'min': min_val
            }
        
        elif chart_type.lower() == 'line':
            chart = ''
            # Simple line chart
            for num in numbers:
                normalized = int((num / max_val) * 10) if max_val > 0 else 0
                chart += '▁' * normalized + '▔' * (10 - normalized) + '  ' + str(round(num, 1)) + '\n'
            
            return {
                'chart_type': 'line',
                'data': numbers,
                'chart': chart,
                'max': max_val,
                'min': min_val
            }
        
        else:
            return {'error': 'Unknown chart type'}
    
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# PRODUCTIVITY TOOLS
# ============================================================================

@register_tool(
    'create_todo',
    'Create and manage todo items',
    {
        'task': 'Task description',
        'priority': 'Priority: high, medium, low',
        'due_date': 'Due date (YYYY-MM-DD) optional'
    }
)
def create_todo(task, priority='medium', due_date=None, **kwargs):
    """Create a todo item"""
    try:
        # Simple in-memory todo storage
        todo = {
            'id': hash(task + str(datetime.now())) % 1000000,
            'task': task,
            'priority': priority.lower(),
            'due_date': due_date,
            'created_at': datetime.now().isoformat(),
            'completed': False
        }
        
        # Validate priority
        if priority.lower() not in ['high', 'medium', 'low']:
            return {'error': 'Priority must be high, medium, or low'}
        
        # Validate date format
        if due_date:
            try:
                datetime.strptime(due_date, '%Y-%m-%d')
            except:
                return {'error': 'Date must be YYYY-MM-DD format'}
        
        return {
            'success': True,
            'todo': todo,
            'message': f'Todo created: {task}'
        }
    except Exception as e:
        return {'error': str(e)}


@register_tool(
    'pomodoro_calculator',
    'Calculate pomodoro breaks and sessions',
    {
        'work_sessions': 'Number of 25-min sessions (default 4)',
        'short_break': 'Minutes for short break (default 5)',
        'long_break': 'Minutes for long break (default 15)'
    }
)
def pomodoro_calculator(work_sessions=4, short_break=5, long_break=15, **kwargs):
    """Calculate pomodoro timer schedule"""
    try:
        sessions = int(work_sessions)
        short_b = int(short_break)
        long_b = int(long_break)
        
        schedule = []
        total_time = 0
        
        for i in range(1, sessions + 1):
            schedule.append({'session': i, 'type': 'work', 'duration': 25})
            total_time += 25
            
            if i < sessions:
                if i % 4 == 0:
                    schedule.append({'break': i, 'type': 'long_break', 'duration': long_b})
                    total_time += long_b
                else:
                    schedule.append({'break': i, 'type': 'short_break', 'duration': short_b})
                    total_time += short_b
        
        return {
            'total_sessions': sessions,
            'total_time_minutes': total_time,
            'total_time_hours': round(total_time / 60, 1),
            'schedule': schedule,
            'efficiency': f'{(sessions * 25) / total_time * 100:.1f}% work time'
        }
    except Exception as e:
        return {'error': str(e)}


@register_tool(
    'task_priority_score',
    'Calculate task priority using Eisenhower Matrix',
    {
        'task': 'Task description',
        'urgency': 'Urgency level 1-10 (10 is most urgent)',
        'importance': 'Importance level 1-10 (10 is most important)'
    }
)
def task_priority_score(task, urgency=5, importance=5, **kwargs):
    """Calculate task priority score using Eisenhower Matrix"""
    try:
        urgency = min(10, max(1, int(urgency)))
        importance = min(10, max(1, int(importance)))
        
        score = (urgency * 0.4) + (importance * 0.6)
        
        # Eisenhower quadrant
        if importance >= 6 and urgency >= 6:
            quadrant = 'Do First (Crisis/Important & Urgent)'
        elif importance >= 6 and urgency < 6:
            quadrant = 'Schedule (Important but Not Urgent)'
        elif importance < 6 and urgency >= 6:
            quadrant = 'Delegate (Urgent but Not Important)'
        else:
            quadrant = 'Eliminate (Neither Important nor Urgent)'
        
        return {
            'task': task[:50],
            'urgency': urgency,
            'importance': importance,
            'priority_score': round(score, 1),
            'eisenhower_quadrant': quadrant,
            'recommendation': f'Score: {round(score, 1)}/10 - {quadrant}'
        }
    except Exception as e:
        return {'error': str(e)}


@register_tool(
    'effort_estimator',
    'Estimate task effort and complexity',
    {
        'task': 'Task description',
        'complexity': 'Complexity 1-5 (1=simple, 5=very complex)',
        'dependencies': 'Number of dependencies (0-10)'
    }
)
def effort_estimator(task, complexity=3, dependencies=0, **kwargs):
    """Estimate task effort"""
    try:
        complexity = min(5, max(1, int(complexity)))
        deps = min(10, max(0, int(dependencies)))
        
        base_effort = complexity * 2  # Hours
        dep_multiplier = 1 + (deps * 0.15)  # 15% per dependency
        
        estimated_hours = base_effort * dep_multiplier
        estimated_days = estimated_hours / 8
        
        confidence = max(0.5, 1.0 - (deps * 0.05))
        
        return {
            'task': task[:50],
            'complexity_level': complexity,
            'dependencies': deps,
            'estimated_hours': round(estimated_hours, 1),
            'estimated_days': round(estimated_days, 1),
            'confidence_percent': round(confidence * 100, 1),
            'recommendation': f'{round(estimated_days, 1)} days ({round(estimated_hours, 1)} hours)'
        }
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# DATA PROCESSING TOOLS
# ============================================================================

@register_tool(
    'csv_parser',
    'Parse and analyze CSV data',
    {
        'csv_data': 'CSV data (comma-separated)',
        'has_header': 'First row is header (true/false)'
    }
)
def csv_parser(csv_data, has_header=True, **kwargs):
    """Parse CSV data"""
    try:
        import csv
        import io
        
        rows = []
        reader = csv.reader(io.StringIO(csv_data))
        
        for row in reader:
            rows.append(row)
        
        if not rows:
            return {'error': 'No CSV data provided'}
        
        header = rows[0] if has_header and rows else None
        data_rows = rows[1:] if has_header and rows else rows
        
        return {
            'row_count': len(data_rows),
            'column_count': len(rows[0]) if rows else 0,
            'header': header,
            'data': data_rows[:10],  # Show first 10 rows
            'total_data_rows': len(data_rows),
            'preview': f'{len(data_rows)} rows, {len(rows[0]) if rows else 0} columns'
        }
    except Exception as e:
        return {'error': str(e)}


@register_tool(
    'data_validator',
    'Validate data types and formats',
    {
        'value': 'Value to validate',
        'data_type': 'Expected type: email, url, phone, number, date, ipv4'
    }
)
def data_validator(value, data_type='email', **kwargs):
    """Validate data format"""
    try:
        import re
        
        validators = {
            'email': r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$',
            'url': r'^https?://',
            'phone': r'^\+?1?\d{9,15}$',
            'ipv4': r'^(\d{1,3}\.){3}\d{1,3}$',
            'date': r'^\d{4}-\d{2}-\d{2}$',
            'number': r'^-?\d+\.?\d*$'
        }
        
        if data_type not in validators:
            return {'error': f'Unknown type: {data_type}'}
        
        pattern = validators[data_type]
        is_valid = bool(re.match(pattern, str(value)))
        
        return {
            'value': value,
            'data_type': data_type,
            'valid': is_valid,
            'message': f'{value} is {"valid" if is_valid else "invalid"} {data_type}'
        }
    except Exception as e:
        return {'error': str(e)}


@register_tool(
    'data_formatter',
    'Format data in various styles',
    {
        'data': 'Data to format',
        'format_type': 'Format: list, table, html, xml, yaml'
    }
)
def data_formatter(data, format_type='list', **kwargs):
    """Format data in different styles"""
    try:
        items = [item.strip() for item in data.split(',')]
        
        if format_type == 'list':
            result = '\n'.join(f'• {item}' for item in items)
        elif format_type == 'table':
            result = '| Item |\n|------|\n' + '\n'.join(f'| {item} |' for item in items)
        elif format_type == 'html':
            result = '<ul>\n' + '\n'.join(f'  <li>{item}</li>' for item in items) + '\n</ul>'
        elif format_type == 'xml':
            result = '<items>\n' + '\n'.join(f'  <item>{item}</item>' for item in items) + '\n</items>'
        elif format_type == 'yaml':
            result = 'items:\n' + '\n'.join(f'  - {item}' for item in items)
        else:
            return {'error': f'Unknown format: {format_type}'}
        
        return {
            'count': len(items),
            'format': format_type,
            'formatted': result
        }
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# TEXT PROCESSING TOOLS
# ============================================================================

@register_tool(
    'text_summarizer',
    'Summarize text into key points',
    {
        'text': 'Text to summarize',
        'sentences': 'Number of sentences in summary (1-5, default 3)'
    }
)
def text_summarizer(text, sentences=3, **kwargs):
    """Summarize text"""
    try:
        import re
        
        sentences_num = min(5, max(1, int(sentences)))
        
        # Split into sentences
        sent_list = re.split(r'[.!?]+', text)
        sent_list = [s.strip() for s in sent_list if s.strip()]
        
        if len(sent_list) <= sentences_num:
            summary = '. '.join(sent_list) + '.'
        else:
            # Take first and last sentences
            selected = [sent_list[0]]
            step = len(sent_list) // (sentences_num - 1)
            for i in range(step, len(sent_list) - 1, step):
                if len(selected) < sentences_num:
                    selected.append(sent_list[i])
            selected.append(sent_list[-1])
            summary = '. '.join(selected[:sentences_num]) + '.'
        
        return {
            'original_length': len(text),
            'summary_length': len(summary),
            'compression_ratio': round((1 - len(summary)/len(text)) * 100, 1),
            'summary': summary,
            'original_sentences': len(sent_list),
            'summary_sentences': sentences_num
        }
    except Exception as e:
        return {'error': str(e)}


@register_tool(
    'keyword_extractor',
    'Extract keywords from text',
    {
        'text': 'Text to analyze',
        'keyword_count': 'Number of keywords to extract (default 5)'
    }
)
def keyword_extractor(text, keyword_count=5, **kwargs):
    """Extract keywords from text"""
    try:
        import re
        
        # Simple keyword extraction
        stop_words = {'the', 'a', 'an', 'and', 'or', 'but', 'is', 'are', 'was', 'were',
                     'be', 'been', 'being', 'have', 'has', 'had', 'do', 'does', 'did',
                     'will', 'would', 'could', 'should', 'may', 'might', 'can', 'must',
                     'to', 'for', 'in', 'on', 'at', 'by', 'of', 'with', 'from'}
        
        # Extract words
        words = re.findall(r'\b\w+\b', text.lower())
        
        # Filter stop words and count
        word_freq = {}
        for word in words:
            if len(word) > 3 and word not in stop_words:
                word_freq[word] = word_freq.get(word, 0) + 1
        
        # Get top keywords
        top_keywords = sorted(word_freq.items(), key=lambda x: x[1], reverse=True)[:int(keyword_count)]
        
        return {
            'text_length': len(text),
            'unique_words': len(set(words)),
            'keywords': [{'word': w[0], 'frequency': w[1]} for w in top_keywords],
            'keyword_count': len(top_keywords)
        }
    except Exception as e:
        return {'error': str(e)}


@register_tool(
    'text_to_outline',
    'Convert text to outline format',
    {
        'text': 'Text to outline',
        'depth': 'Outline depth 1-3 (default 2)'
    }
)
def text_to_outline(text, depth=2, **kwargs):
    """Convert text to outline"""
    try:
        import re
        
        # Split by sentences and paragraphs
        paragraphs = text.split('\n\n')
        
        outline = []
        for i, para in enumerate(paragraphs, 1):
            if para.strip():
                sentences = re.split(r'[.!?]+', para)
                if int(depth) >= 1:
                    outline.append(f'I. {para[:50]}...' if len(para) > 50 else f'I. {para}')
                if int(depth) >= 2:
                    for j, sent in enumerate(sentences[:3], 1):
                        if sent.strip():
                            outline.append(f'   A. {sent.strip()[:40]}...' if len(sent.strip()) > 40 else f'   A. {sent.strip()}')
        
        return {
            'original_length': len(text),
            'outline_depth': int(depth),
            'outline_items': len(outline),
            'outline': outline[:20]  # Max 20 items
        }
    except Exception as e:
        return {'error': str(e)}


# ============================================================================
# DATE & TIME TOOLS
# ============================================================================

@register_tool(
    'date_calculator',
    'Calculate days between dates',
    {
        'start_date': 'Start date (YYYY-MM-DD)',
        'end_date': 'End date (YYYY-MM-DD)',
        'include_weekdays': 'Count only weekdays (true/false)'
    }
)
def date_calculator(start_date, end_date, include_weekdays=False, **kwargs):
    """Calculate days between dates"""
    try:
        from datetime import datetime, timedelta
        
        start = datetime.strptime(start_date, '%Y-%m-%d')
        end = datetime.strptime(end_date, '%Y-%m-%d')
        
        delta = end - start
        total_days = delta.days
        
        # Calculate weekdays
        weekdays = 0
        if include_weekdays:
            current = start
            while current <= end:
                if current.weekday() < 5:  # Mon-Fri
                    weekdays += 1
                current += timedelta(days=1)
        
        weeks = total_days // 7
        months = total_days // 30
        
        return {
            'start_date': start_date,
            'end_date': end_date,
            'total_days': total_days,
            'weeks': weeks,
            'months': months,
            'weekdays': weekdays if include_weekdays else 'Not calculated',
            'time_period': f'{months} months, {total_days % 30} days'
        }
    except Exception as e:
        return {'error': str(e)}


@register_tool(
    'time_range_calculator',
    'Calculate time ranges and durations',
    {
        'start_time': 'Start time (HH:MM)',
        'end_time': 'End time (HH:MM)',
        'include_breaks': 'Include break time (minutes, optional)'
    }
)
def time_range_calculator(start_time, end_time, include_breaks=0, **kwargs):
    """Calculate time durations"""
    try:
        start = datetime.strptime(start_time, '%H:%M')
        end = datetime.strptime(end_time, '%H:%M')
        
        if end < start:
            end = end + timedelta(days=1)
        
        delta = end - start
        total_minutes = delta.total_seconds() / 60
        breaks = int(include_breaks)
        work_minutes = total_minutes - breaks
        
        hours = int(work_minutes // 60)
        minutes = int(work_minutes % 60)
        
        return {
            'start_time': start_time,
            'end_time': end_time,
            'total_duration_minutes': int(total_minutes),
            'total_duration_hours': round(total_minutes / 60, 2),
            'break_time_minutes': breaks,
            'work_time': f'{hours}h {minutes}m'
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

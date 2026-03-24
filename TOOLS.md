# 🛠️ JeebsAI Tools & Capabilities Guide

## Overview
JeebsAI now includes **16+ powerful tools** for information gathering, analysis, and productivity. All tools are:
- ✅ **Integrated into chat** - Just ask!
- ✅ **Testable from Admin** - Try before using
- ✅ **Auto-learning** - Results saved to brain
- ✅ **Chainable** - Tools can call other tools

---

## 📚 Information & Research Tools

### 1. **Web Search** 🔍
Search the web for real-time information via DuckDuckGo.

**How to use:**
- Chat: "Search for artificial intelligence"
- Or: "What's the latest on quantum computing?"
- Tool: `web_search(query, max_results=5)`

**Returns:** List of search results with title, snippet, URL, source

**Learning:** Results automatically saved to brain for future reference

---

### 2. **Wikipedia Summary** 📖
Get concise summaries of any Wikipedia topic.

**How to use:**
- Chat: "Tell me about Python programming from Wikipedia"
- Tool: `wikipedia_summary(topic, sentences=3)`

**Returns:** Topic name, summary text, Wikipedia URL

**Best for:** Quick educational overviews

---

### 3. **Latest News** 📰
Get trending news and articles about any topic.

**How to use:**
- Chat: "What's the latest news on machine learning?"
- Tool: `latest_news(topic, count=3)`

**Returns:** Article list with titles, descriptions, URLs

---

### 4. **Definition Lookup** 📝
Get word definitions and related information.

**How to use:**
- Chat: "Define quantum physics"
- Tool: `define_word(word, detailed=False)`

**Returns:** Word definition, related topics, source URL

---

## 📊 Analysis & Statistics Tools

### 5. **Text Statistics** 📈
Analyze any text for detailed statistics and readability metrics.

**How to use:**
- Tool: `text_stats(text, include='words,sentences,readability')`

**Returns:**
- Word count, unique words
- Sentence count, avg words per sentence
- Paragraph count
- Readability grade level
- Character count

**Grades:**
- Grade 0-6: "Very Easy"
- Grade 6-9: "Easy"
- Grade 9-13: "Medium"
- Grade 13-15: "Hard"
- Grade 15+: "Very Hard"

---

### 6. **Sentiment Analysis** 😊
Analyze the emotional tone and sentiment of text.

**How to use:**
- Tool: `sentiment_analysis(text, language='en')`

**Returns:**
- Sentiment: positive, negative, or neutral
- Confidence score (0-100%)
- Positive word count
- Negative word count
- Overall score (0-1)

**Examples:**
- "This is amazing!" → Positive (95%)
- "I hate this" → Negative (92%)
- "The weather is cold" → Neutral (50%)

---

### 7. **Brain Statistics** 🧠
View detailed statistics about the holographic brain's learning.

**How to use:**
- Tool: `brain_stats(include='total_memories,top_topics,recent_learning')`

**Returns:**
- Total memories stored
- Top topics (frequency)
- Recent learning from conversations
- Memory growth metrics

---

### 8. **Code Analysis** 🐛
Analyze Python code for syntax, performance, security, and style issues.

**How to use:**
- Tool: `analyze_code(code, check_type='syntax')`

**Check types:**
- `syntax` - Check for syntax errors
- `performance` - Find infinite loops, bad imports
- `security` - Detect dangerous functions (eval, exec, os.system)
- `style` - Line length, formatting issues

**Returns:** Issues list, suggestions, line count

---

## 🔧 Conversion & Transformation Tools

### 9. **Unit Converter** ⚙️
Convert between various units of measurement.

**Supported conversions:**

**Distance:**
- km ↔ m ↔ mi ↔ ft ↔ yd

**Weight:**
- kg ↔ g ↔ lb ↔ oz

**Temperature:**
- °C ↔ °F ↔ K

**How to use:**
- Chat: "Convert 5 miles to kilometers"
- Tool: `convert_units(value, from_unit='mi', to_unit='km')`

**Returns:** Original value, converted value, units

---

### 10. **Color Converter** 🎨
Convert between color formats.

**Supported formats:**
- Hex: `#FF5733`
- RGB: `rgb(255, 87, 51)`
- HSL: `hsl(0, 100%, 60%)`

**How to use:**
- Chat: "Convert #FF5733 to RGB"
- Tool: `convert_color(color, to_format='rgb')`

**Returns:** All color format conversions

---

### 11. **JSON Formatter** 📄
Format, validate, and minify JSON.

**How to use:**
- Tool: `format_json(json_text, minify=False)`

**Returns:**
- Formatted JSON (pretty-printed or minified)
- Validity status
- File size comparison

---

## 🔐 Security & Generation Tools

### 12. **Password Generator** 🔑
Generate secure, cryptographically random passwords.

**How to use:**
- Tool: `generate_password(length=16, include_symbols=True)`

**Returns:**
- Generated password
- Strength rating: Weak, Fair, Good, Strong
- Entropy score (bits)

**Strength factors:**
- Contains uppercase letters
- Contains lowercase letters
- Contains numbers
- Contains special symbols

---

## 🧮 Calculation Tools

### 13. **Calculator** 🔢
Perform mathematical calculations with support for complex functions.

**Operations:**
- Basic: `add`, `subtract`, `multiply`, `divide`
- Advanced: `power`, `sqrt`, `sin`, `cos`, `tan`, `log`
- Constants: `pi`, `e`

**How to use:**
- Chat: "Calculate 2 plus 2"
- Tool: `calculator(expression='2 + 2')`
- Or: `calculator(operation='add', a=5, b=3)`

**Returns:** Calculation result, operation performed

---

## 🎁 Entertainment & Inspiration Tools

### 14. **Random Quote** 💡
Get inspirational and motivational quotes.

**Categories:**
- `motivational` - Success and achievement quotes
- `funny` - Humorous one-liners
- `wise` - Philosophical wisdom

**How to use:**
- Chat: "Give me a funny quote"
- Tool: `get_quote(category='motivational')`

**Returns:** Quote text, author, category

---

### 15. **Joke Generator** 😄
Get random jokes across multiple categories.

**Categories:**
- `programming` - Developer humor
- `knock-knock` - Classic knock-knock jokes
- `general` - All-purpose jokes

**How to use:**
- Chat: "Tell me a programming joke"
- Tool: `get_joke(category='programming')`

**Returns:** Joke text, category

---

### 16. **Fun Facts** 🌟
Get interesting facts across multiple domains.

**Categories:**
- `science` - Scientific facts
- `nature` - Wildlife and natural world
- `history` - Historical facts

**How to use:**
- Chat: "Give me a fun fact about science"
- Tool: `fun_fact(category='science')`

**Returns:** Fact text, category

---

## 🌐 Web Tools

### 17. **URL Info** 🔗
Get metadata about any URL without full page load.

**How to use:**
- Tool: `get_url_info(url, include='title,description,image')`

**Returns:**
- URL status code
- Page title
- Meta description
- Open Graph image
- Redirect info

---

## 💪 Advanced Tool Features

### Tool Chaining
Tools can work together automatically:
1. **Search for topic** → Web search returns results
2. **Brain learns results** → Automatically saved
3. **Future queries** → Brain recognizes similar questions

### Auto-Detection
JeebsAI automatically detects tool needs in chat:
- "Search for..." → Triggers `web_search`
- "Calculate..." → Triggers `calculator`
- "Define..." → Triggers `define_word`
- "Convert..." → Triggers `convert_units`

### Rate Limiting
Public API tools are rate-limited to prevent abuse:
- Web search: 10 per hour per user
- Wikipedia: 5 per hour per user
- News: 10 per hour per user

---

## 🚀 Using Tools in Chat

### Basic Usage
```
"Search for artificial intelligence"
→ Triggers web_search automatically
→ Results displayed in chat
→ Brain learns the information
```

### Advanced Usage
```
"What's the readability level of this text? [paste text]"
→ Detects text_stats request
→ Analyzes readability
→ Returns grade level + notes
```

### Multi-Tool Workflows
```
1. "Search for Bitcoin history"
   → Web search returns results
   → Brain learns about Bitcoin
   
2. "Summarize Bitcoin from Wikipedia"
   → Wikipedia_summary called
   → More detailed info obtained
   
3. "What did you learn about Bitcoin?" 
   → Brain retrieves both sources
   → Synthesizes comprehensive answer
```

---

## 🎯 Tips & Tricks

### Use Multiple Tools
Ask JeebsAI to use multiple tools in one request:
- "Search for climate change AND give me the readability level of Wikipedia's article"

### Refine Results
Tools can be refined with parameters:
- "Search for quantum physics with 10 results"
- "Generate a 32-character password with symbols"

### Learn Over Time
The brain learns from every tool result:
- First query: "Search for machine learning"
- Later: "Tell me about ML" → Uses learned knowledge

### Export Learning
Use the brain_stats tool to see what's been learned:
- "Show me my brain statistics"
- View top topics, total memories, learning rate

---

## 📈 Tool Statistics

| Tool | Category | Learning | Real-time |
|------|----------|----------|-----------|
| Web Search | Research | ✅ | ✅ |
| Wikipedia | Research | ✅ | ✅ |
| News | Research | ✅ | ✅ |
| Definition | Research | ✅ | ✅ |
| Text Stats | Analysis | ✅ | ✅ |
| Sentiment | Analysis | ✅ | ✅ |
| Brain Stats | Analysis | N/A | ✅ |
| Code Analysis | Analysis | ✅ | ✅ |
| Unit Converter | Transform | ✅ | ✅ |
| Color Converter | Transform | ✅ | ✅ |
| JSON Formatter | Transform | ✅ | ✅ |
| Password Gen | Security | N/A | ✅ |
| Calculator | Math | ✅ | ✅ |
| Quote | Entertainment | ✅ | ✅ |
| Joke | Entertainment | ✅ | ✅ |
| Fun Fact | Entertainment | ✅ | ✅ |
| URL Info | Web | ✅ | ✅ |

---

## 🔮 Coming Soon

- **Equation Solver** - Solve complex mathematical equations
- **Image Analysis** - Analyze images from URLs
- **Stock/Crypto** - Real-time market data
- **Language Translation** - Translate text between languages
- **Email** - Send emails
- **Calendar** - Schedule and manage events
- **PDF Analysis** - Extract text from documents
- **Conversation Export** - Export to PDF/DOCX
- **Data Visualization** - Create charts

---

## ⚡ Performance Notes

- Most tools respond within **< 1 second**
- Web search may take 2-5 seconds (network dependent)
- All results are cached in the brain for instant recall
- Zero API keys required for most tools
- Completely private - no external data tracking

---

## 🆘 Troubleshooting

**Tool returns error?**
- Check internet connection
- Verify input format
- Try similar query
- Check tool rate limits

**Brain not learning?**
- Ensure conversation is created first
- Check `brain_stats` for memory count
- Tool may be rate-limited

**Results seem incomplete?**
- Increase `max_results` parameter
- Try broader search terms
- Use Wikipedia for detailed info
- Check web_search returned success

---

## 🎓 Learn More

For detailed tool API documentation, see:
- `app/tools.py` - Tool implementations
- `app/tools_api.py` - API endpoints
- Admin Dashboard → Tools tab - Interactive testing

Enjoy! 🚀

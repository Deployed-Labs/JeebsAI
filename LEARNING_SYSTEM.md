# Jeebs Learning & Knowledge System

## Overview

JeebsAI now features advanced language learning and knowledge retrieval systems that enable Jeebs to:
- **Learn from every conversation** - vocabulary, patterns, and context
- **Synthesize intelligent answers** - pulling from multiple knowledge sources
- **Track learning progress** - monitor vocabulary growth and knowledge accumulation
- **Store contextual knowledge** - organize information by topics and relationships

---

## 🧠 Knowledge Retrieval System

### What It Does

The advanced knowledge retrieval system searches across multiple sources simultaneously:

1. **Brain Nodes** - Core knowledge entries stored from web crawling and training
2. **Knowledge Triples** - Structured facts (subject-predicate-object relationships)
3. **Contextual Knowledge** - Topic-organized information with related concepts
4. **FAQ Entries** - User-taught question-answer pairs

### How It Works

When you ask Jeebs a question, it:
1. Extracts meaningful query terms (filtering stop words)
2. Searches all knowledge sources in parallel
3. Ranks results by relevance score
4. **Synthesizes a coherent answer** from multiple sources
5. Returns both the synthesized answer and source attribution

### Relevance Scoring

Results are ranked using multiple factors:
- **Exact matches** in labels (highest weight)
- **Term frequency** across labels, summaries, and content
- **Tag matches** for semantic relationships
- **Category bonuses** (FAQ entries get priority for direct questions)
- **Recency bonus** (newer knowledge weighted slightly higher)

### Example Usage

**User:** "What is Rust?"

**Jeebs Response:**
```
Based on what I know: Rust is a systems programming language focused on safety, 
speed, and concurrency. It prevents memory errors through its ownership system. 
Additionally: Rust is used for building reliable and efficient software. 
The language was designed at Mozilla Research.

(Retrieved from 4 sources)
```

---

## 📚 Language Learning System

### Automatic Learning

Jeebs learns from every message you send:

#### Vocabulary Tracking
- Extracts meaningful words (3+ characters, non-common)
- Tracks frequency of usage
- Guesses part of speech (noun, verb, adjective, etc.)
- Stores examples and context

#### Pattern Recognition
- Categorizes inputs: greetings, questions, commands, statements, expressions
- Normalizes patterns (replaces numbers, names with placeholders)
- Tracks usage frequency
- Associates response templates

### Vocabulary Statistics

Check what Jeebs has learned:

**Command:** `vocabulary stats` or `language stats`

**Response:**
```
📖 Vocabulary Statistics:

Total Unique Words Learned: 1,247
Total Word Encounters: 8,953

Parts of Speech:
  • noun: 423
  • verb/gerund: 287
  • adjective: 195
  • adverb: 89
  • verb/past: 156
  • unknown: 97
```

---

## 📊 Knowledge Base Statistics

### Check Your Knowledge

**Command:** `knowledge stats` or `what do you know`

**Response:**
```
📊 Knowledge Base Statistics:

🧠 Brain Nodes: 342
🔗 Knowledge Triples: 1,089
❓ FAQ Entries: 47
📚 Contextual Topics: 23

Total Knowledge Items: 1,501
```

---

## 💡 Teaching Jeebs

### Store FAQ Responses

**Format:** `remember: [question] => [answer]`

**Example:**
```
User: remember: what is your purpose? => I exist to help you learn and retrieve information efficiently.
Jeebs: Saved. Ask me "what is your purpose?" and I will use that answer.
```

### Store Contextual Knowledge

**Format:** `store this: [information]`

**Example:**
```
User: store this: Machine learning is a subset of artificial intelligence that enables systems to learn from data
Jeebs: Stored contextual knowledge about 'Machine learning is a'. I can now reference this when needed.
```

### Teach About Topics

**Format:** `teach me about [topic]` or `learn about [topic]`

**Example:**
```
User: teach me about quantum computing
Jeebs: I'll start learning about quantum computing. Please share some facts or information, 
and I'll store them. You can say things like:
- "quantum computing" is important because...
- Key concepts in quantum computing include...
- quantum computing relates to...
```

---

## 🔍 API Endpoints

### Advanced Knowledge Search

**Endpoint:** `POST /api/knowledge/search`

**Request:**
```json
{
  "query": "rust programming",
  "max_results": 10
}
```

**Response:**
```json
{
  "items": [
    {
      "id": "brain:rust_intro",
      "label": "Rust Programming Language",
      "summary": "A systems programming language...",
      "content": "Rust is designed for performance...",
      "category": "brain_node",
      "tags": ["programming", "systems"],
      "relevance_score": 8.5,
      "created_at": "2026-02-21T10:30:00Z"
    }
  ],
  "total_searched": 15,
  "query_terms": ["rust", "programming"],
  "synthesized_answer": "Rust is a systems programming language..."
}
```

### Knowledge Statistics

**Endpoint:** `GET /api/knowledge/stats`

**Response:**
```json
{
  "brain_nodes": 342,
  "knowledge_triples": 1089,
  "faq_entries": 47,
  "contexts": 23
}
```

### Language Statistics

**Endpoint:** `GET /api/language/stats`

**Response:**
```json
{
  "total_words": 1247,
  "total_frequency": 8953,
  "pos_noun": 423,
  "pos_verb/gerund": 287,
  "pos_adjective": 195
}
```

---

## 🎯 Benefits

### For Users

1. **Smarter Answers**: Jeebs synthesizes information from multiple sources
2. **Better Context**: Understands relationships between concepts
3. **Transparent Sources**: Shows where information came from
4. **Personalized Learning**: Remembers what you teach it

### For Jeebs

1. **Continuous Improvement**: Learns from every interaction
2. **Growing Vocabulary**: Expands language understanding automatically
3. **Pattern Recognition**: Identifies communication styles and adapts
4. **Knowledge Organization**: Structures information for efficient retrieval

---

## 📈 Learning Progress

### What Jeebs Tracks

1. **Vocabulary Growth**
   - Unique words encountered
   - Word frequencies
   - Parts of speech distribution

2. **Knowledge Accumulation**
   - Brain nodes from web crawling
   - User-taught facts (FAQ)
   - Contextual topics
   - Structured relationships (triples)

3. **Communication Patterns**
   - Greeting styles
   - Question patterns
   - Command structures
   - Expression recognition

---

## 🔧 Commands Reference

### Learning Commands
- `remember: [question] => [answer]` - Teach FAQ
- `forget: [question]` - Remove FAQ
- `store this: [info]` - Store context
- `teach me about [topic]` - Start topic learning

### Statistics Commands
- `knowledge stats` - View knowledge base stats
- `vocabulary stats` - View language learning stats
- `what do you know` - Check knowledge coverage
- `how much knowledge` - Same as knowledge stats

### Query Commands
- Just ask a question! - Jeebs will search its knowledge base
- Questions ending with `?` trigger enhanced retrieval
- Specific terms get better matches

---

## 🚀 Advanced Features

### Multi-Source Synthesis

When you ask about a topic, Jeebs:
1. Finds related brain nodes
2. Retrieves relevant knowledge triples
3. Checks contextual knowledge
4. Reviews FAQ entries
5. **Combines** all sources into a coherent answer

### Intelligent Ranking

Results prioritized by:
- Direct relevance to your query
- Source reliability (knowledge triples have confidence scores)
- Freshness of information
- Category appropriateness

### Contextual Understanding

Jeebs maintains context about:
- Topics and their key concepts
- Related topics and connections
- Facts organized by subject
- Last update timestamps

---

## 💻 Technical Implementation

### Files Created

1. **`src/language_learning.rs`** - Vocabulary and pattern learning
   - VocabularyEntry tracking
   - LanguagePattern recognition
   - ContextualKnowledge storage

2. **`src/knowledge_retrieval.rs`** - Advanced search and synthesis
   - Multi-source searching
   - Relevance scoring
   - Answer synthesis

### Integration Points

- **`src/cortex.rs`** - Enhanced with:
  - Automatic learning on every input
  - Advanced retrieval replacing simple search
  - New commands for stats and teaching

- **`src/main.rs`** - New API endpoints:
  - `/api/knowledge/search`
  - `/api/knowledge/stats`
  - `/api/language/stats`

### Database Storage

All data stored in `jeebs_store` with key prefixes:
- `vocab:*` - Vocabulary entries
- `language:patterns:*` - Language patterns by category
- `context:*` - Contextual knowledge by topic
- `chat:faq:*` - FAQ question-answer pairs

---

## 🎓 Learning Examples

### Example 1: Vocabulary Learning

**User:** "Jeebs, I'm fascinated by quantum entanglement and superposition."

**Behind the scenes:**
- Learns: "fascinated", "quantum", "entanglement", "superposition"
- Categories: adjective/verb, noun, noun, noun
- Stores frequency: 1 for each
- Pattern: "expression" category

### Example 2: Knowledge Synthesis

**User:** "What is machine learning?"

**Behind the scenes:**
1. Searches brain_nodes: 3 results
2. Searches knowledge_triples: 5 results  
3. Searches contexts: 1 result
4. Ranks by relevance
5. Synthesizes: "Machine learning is a subset of AI that enables systems to learn from data without explicit programming. It uses algorithms to identify patterns. Key applications include image recognition, natural language processing, and predictive analytics."

### Example 3: Context Building

**User:** "store this: Neural networks are inspired by biological brains and consist of interconnected nodes organized in layers"

**Result:**
- Topic extracted: "Neural networks are inspired"
- Fact stored with context
- Available for future retrieval
- Can be referenced in related queries

---

## 🌟 Future Enhancements

Planned improvements:
- **Semantic embeddings** for better similarity matching
- **Confidence scoring** for synthesized answers
- **Source citation** in detailed format
- **Learning suggestions** based on knowledge gaps
- **Auto-categorization** of stored knowledge
- **Cross-referencing** between related topics
- **Etymology tracking** for vocabulary
- **Synonym/antonym** learning
- **Multi-language** support

---

## 📝 Tips for Best Results

1. **Be specific** in questions - more context = better retrieval
2. **Teach frequently** - use `remember:` for important facts
3. **Check progress** - use stats commands to see learning
4. **Store context** - organize related information by topic
5. **Ask follow-ups** - Jeebs maintains conversation context
6. **Review knowledge** - periodically check `knowledge stats`

---

## 🎉 Summary

Jeebs now has **real learning power**:
- ✅ Learns vocabulary automatically
- ✅ Recognizes communication patterns
- ✅ Retrieves from multiple knowledge sources
- ✅ Synthesizes coherent answers
- ✅ Tracks learning progress
- ✅ Accepts user teaching
- ✅ Provides transparent statistics

**The more you interact with Jeebs, the smarter it becomes!**

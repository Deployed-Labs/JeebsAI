# Quick Start: New Jeebs Learning Features

## 🚀 Try These Right Away!

### 1. Check What Jeebs Knows
```
User: knowledge stats
```
Jeebs will show you:
- Total brain nodes
- Knowledge triples
- FAQ entries
- Contextual topics

### 2. See Language Learning Progress
```
User: vocabulary stats
```
Jeebs will display:
- Total words learned
- Word frequencies
- Parts of speech breakdown

### 3. Ask a Question
```
User: what is rust programming?
```
Jeebs will:
- Search all knowledge sources
- Synthesize an answer
- Show source attribution

### 4. Teach Jeebs Something
```
User: remember: what is the answer? => 42
```
Then ask:
```
User: what is the answer?
Jeebs: 42
```

### 5. Store Context
```
User: store this: JavaScript is a dynamic programming language used for web development
```
Jeebs will store this for future retrieval.

### 6. Get a Proactive Suggestion
```
User: what do you want to do?
```
Jeebs will propose:
- Learning a new topic
- Adding a feature
- Running an experiment
- Reviewing evolution proposals

### 7. View Experiments List
```
User: what experiments?
```
See all the experiments Jeebs wants to run.

### 8. Accept a Proposal
When Jeebs suggests something:
```
User: yes, do it
```
Or decline:
```
User: no thanks
```

### 9. Check Active Sessions (Admin)
Navigate to: `/webui/admin_dashboard.html`
- See all active user sessions
- View IP addresses and last activity
- Terminate sessions if needed

### 10. Get Help
```
User: help
```
See all available commands and features.

---

## 🎯 Pro Tips

1. **Be specific** - More detail = better answers
2. **Ask follow-ups** - Jeebs maintains context
3. **Check progress** - Use stats commands regularly
4. **Teach actively** - Use `remember:` for important facts
5. **Store context** - Organize knowledge by topic
6. **Try proposals** - Let Jeebs suggest what to learn next

---

## 📊 Example Conversation

```
User: Hi Jeebs!
Jeebs: Hey! I am Jeebs. I am online and ready to chat.

User: knowledge stats
Jeebs: 📊 Knowledge Base Statistics:
       🧠 Brain Nodes: 342
       🔗 Knowledge Triples: 1,089
       Total Knowledge Items: 1,431

User: what is machine learning?
Jeebs: Based on what I know: Machine learning is a subset of AI that 
       enables systems to learn from data. It uses algorithms to identify 
       patterns. (Retrieved from 3 sources)

User: remember: favorite color => deep blue
Jeebs: Saved. Ask me "favorite color" and I will use that answer.

User: vocabulary stats
Jeebs: 📖 Vocabulary Statistics:
       Total Unique Words Learned: 89
       Total Word Encounters: 234
       Parts of Speech:
         • noun: 32
         • verb/gerund: 18

User: what do you want to do?
Jeebs: 💡 Proactive Suggestion: I want to learn about quantum computing fundamentals
       Why: Learning quantum computing fundamentals would expand my knowledge 
       and help me answer more questions in this domain.
       Would you like me to research this topic and add it to my knowledge base?

User: yes, do it
Jeebs: Great! I've added this to my action queue. I'll work on it during 
       my next autonomous cycle and report back with results.
```

---

## 🔧 For Developers

### New API Endpoints:

**Search Knowledge:**
```bash
curl -X POST http://localhost:8080/api/knowledge/search \
  -H "Content-Type: application/json" \
  -d '{"query":"rust programming","max_results":5}'
```

**Get Knowledge Stats:**
```bash
curl http://localhost:8080/api/knowledge/stats
```

**Get Language Stats:**
```bash
curl http://localhost:8080/api/language/stats
```

---

## 📚 Learn More

- **Full Learning Guide**: See `LEARNING_SYSTEM.md`
- **Proactive Actions**: See `PROACTIVE_ACTIONS.md`
- **Enhancement Summary**: See `ENHANCEMENT_SUMMARY.md`

---

## 🎉 Enjoy Your Enhanced Jeebs!

Every conversation makes Jeebs smarter. The more you interact, the better it becomes at:
- Understanding your communication style
- Retrieving relevant knowledge
- Synthesizing coherent answers
- Proposing valuable actions

**Start chatting and watch Jeebs learn!**

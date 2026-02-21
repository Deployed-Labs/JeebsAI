# Jeebs Proactive Action Proposals

## Overview

Jeebs now has the capability to proactively suggest actions it wants to take, including:
- Learning about new topics
- Proposing new features
- Running experiments
- Highlighting pending evolution proposals

## How It Works

### Automatic Proposals
- Jeebs will periodically (every 30 minutes) suggest an action during conversations
- Proposals appear after certain types of responses (when context is appropriate)
- Each proposal includes:
  - **Description**: What Jeebs wants to do
  - **Reason**: Why this action would be valuable
  - **Icon**: 💡 for learning, 🔧 for features, 🧪 for experiments, 🧬 for evolution

### Types of Actions

#### 1. Learning Proposals 💡
Jeebs will suggest learning about topics like:
- Quantum computing fundamentals
- Distributed systems architecture
- Natural language processing techniques
- Machine learning inference optimization
- And many more...

#### 2. Feature Proposals 🔧
Jeebs will propose new features such as:
- Voice input support
- Real-time collaborative editing
- Multi-language support
- Conversation export to PDF
- And more...

#### 3. Experiment Proposals 🧪
Jeebs will suggest experiments to improve performance:
- Different knowledge graph traversal algorithms
- Response time benchmarks
- Context compression experiments
- Training data optimization
- And more...

#### 4. Evolution Proposals 🧬
When Jeebs has self-improvement proposals pending in the evolution system, it will alert you and direct you to review them at `/webui/evolution.html`

## User Interaction

### Accepting a Proposal
Users can accept a proposal by responding with:
- "yes, do it"
- "go ahead"
- "yes please"
- "yes" (when mentioning research/learn/add)

### Declining a Proposal
Users can decline by saying:
- "no thanks"
- "not now"
- "skip it"
- "maybe later"

### Requesting Proposals
Users can manually request a proposal at any time:
- "what do you want to do?"
- "what do you want to learn?"
- "suggest something"
- "propose an action"

### Viewing Experiments List
Users can see all experiments Jeebs wants to run:
- "what experiments?"
- "show experiments"
- "experiments list"

## Implementation Details

### Files Modified
1. **`src/proposals.rs`** (new): Core proactive proposal logic
   - Manages proposal timing and generation
   - Contains lists of learning topics, features, and experiments
   - Formats proposals for display

2. **`src/cortex.rs`**: Integrated proposals into chat
   - Appends proposals to appropriate responses
   - Handles user acceptance/decline commands
   - Added "what do you want" queries

3. **`src/lib.rs`**: Registered proposals module

4. **`src/auth/mod.rs`**: Enhanced session tracking
   - Sessions now created on login
   - Sessions removed on logout
   - IP and user agent tracked

5. **`src/chat.rs`**: Improved session updates
   - Uses INSERT OR REPLACE for reliability
   - Tracks IP and user agent on each chat

### Configuration
- **Proposal Interval**: 30 minutes (configurable via `PROPOSAL_INTERVAL_SECS`)
- **Proposals are context-aware**: Only shown after appropriate responses
- **Non-intrusive**: Won't spam users with constant suggestions

## Benefits

1. **Proactive Intelligence**: Jeebs demonstrates initiative and self-direction
2. **Transparency**: Users see what Jeebs wants to improve
3. **Engagement**: Creates interactive dialogue about system improvements
4. **Alignment**: Ensures Jeebs' goals align with user priorities
5. **Evolution Integration**: Connects manual proposals with autonomous evolution system

## Future Enhancements

Potential improvements:
- Learn from which proposals users accept/decline
- Prioritize proposals based on usage patterns
- Allow users to add custom experiments
- Track proposal success rates
- Generate proposals based on detected knowledge gaps
- Integrate with external knowledge sources for learning

## Example Interactions

**Example 1: Learning Proposal**
```
User: tell me about rust
Jeebs: I am still learning that topic, and I want to learn it deeply.

---

💡 **Proactive Suggestion**: I want to learn about distributed systems architecture

**Why**: Learning distributed systems architecture would expand my knowledge 
and help me answer more questions in this domain.

Would you like me to research this topic and add it to my knowledge base?
```

**Example 2: Feature Proposal**
```
User: what can you do?
Jeebs: I can handle conversation and basic assistant tasks...

---

🔧 **Feature Idea**: Add voice input support for chat

**Why**: This feature would improve user experience and system capabilities.

Should I create a proposal for this feature?
```

**Example 3: User Request**
```
User: what do you want to do?
Jeebs: 🧪 **Experiment Proposal**: Test different knowledge graph traversal algorithms

**Why**: Running this experiment would provide data to optimize performance and quality.

Shall I add this to my experiment backlog?
```

## Notes

- Proposals are stored in the database to track timing
- The system intelligently rotates through different proposal types
- Evolution proposals are surfaced when autonomous system creates them
- All proposals respect the 30-minute cooldown to avoid spam

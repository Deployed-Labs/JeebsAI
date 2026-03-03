/// Conversation Context Manager
///
/// Understands multi-turn conversations by:
/// - Tracking conversation history and context
/// - Identifying topic continuity and shifts
/// - Extracting user intent from context
/// - Maintaining focus on current topic
/// - Preventing context loss across turns

use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ConversationContext {
    pub session_id: String,
    pub user_id: Option<String>,
    pub messages: Vec<ConversationMessage>,
    pub current_topic: String,
    pub previous_topics: Vec<String>,
    pub user_intent: String,
    pub focus_level: f32,
}

#[derive(Debug, Clone)]
pub struct ConversationMessage {
    pub role: String, // "user" or "jeebs"
    pub content: String,
    pub timestamp: String,
    pub topics_mentioned: Vec<String>,
    pub intent: String,
}

#[derive(Debug, Clone)]
pub struct UserIntent {
    pub primary: String, // "ask", "clarify", "explore", "learn"
    pub sentiment: f32,  // -1.0 to 1.0
    pub confidence: f32, // 0.0 to 1.0
    pub requires_explanation: bool,
}

/// Load or create conversation context
pub async fn load_conversation_context(
    pool: &SqlitePool,
    session_id: &str,
    user_id: Option<&str>,
) -> Result<ConversationContext, String> {
    // Try to fetch existing conversation from chat_history
    let history = sqlx::query_as::<_, (String, String)>(
        "SELECT role, message FROM chat_history
         WHERE session_id = ?
         ORDER BY created_at DESC
         LIMIT 20",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut messages = Vec::new();
    let mut topics = Vec::new();

    for (role, content) in history.iter().rev() {
        let msg_topics = extract_topics(&content);
        topics.extend(msg_topics.clone());

        messages.push(ConversationMessage {
            role: role.clone(),
            content: content.clone(),
            timestamp: chrono::Local::now().to_rfc3339(),
            topics_mentioned: msg_topics,
            intent: String::new(),
        });
    }

    topics.sort();
    topics.dedup_by(|a, b| a.to_lowercase().eq(&b.to_lowercase()));

    let current_topic = messages
        .first()
        .and_then(|m| m.topics_mentioned.first())
        .cloned()
        .unwrap_or_else(|| "general".to_string());

    println!(
        "[Context] Loaded {} messages, {} topics, focus: {}",
        messages.len(),
        topics.len(),
        current_topic
    );

    Ok(ConversationContext {
        session_id: session_id.to_string(),
        user_id: user_id.map(|s| s.to_string()),
        messages,
        current_topic,
        previous_topics: topics,
        user_intent: "ask".to_string(),
        focus_level: 0.8,
    })
}

/// Analyze user message to extract intent and topics
pub fn analyze_user_message(message: &str) -> UserIntent {
    let lower = message.to_lowercase();

    // Detect intent
    let primary = if lower.contains("why") || lower.contains("explain") {
        "explain"
    } else if lower.contains("how") || lower.contains("can you") {
        "instruct"
    } else if lower.contains("what") || lower.contains("tell me") {
        "ask"
    } else if lower.contains("more") || lower.contains("another") {
        "explore"
    } else if lower.contains("?") && message.len() < 20 {
        "clarify"
    } else {
        "ask"
    };

    // Detect sentiment
    let sentiment = if lower.contains("awesome") || lower.contains("great") || lower.contains("thanks") {
        0.8
    } else if lower.contains("wrong") || lower.contains("bad") || lower.contains("hate") {
        -0.7
    } else {
        0.2
    };

    // Detect if needs explanation
    let requires_explanation = message.len() > 30
        && (lower.contains("why") || lower.contains("explain") || lower.contains("understand"));

    UserIntent {
        primary: primary.to_string(),
        sentiment,
        confidence: 0.75,
        requires_explanation,
    }
}

/// Extract topics from message
fn extract_topics(message: &str) -> Vec<String> {
    let words: Vec<&str> = message
        .split_whitespace()
        .filter(|w| w.len() > 4 && !is_common_word(w))
        .collect();

    words
        .iter()
        .take(5)
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
}

/// Check if word is common filler
fn is_common_word(word: &str) -> bool {
    matches!(
        word.to_lowercase().as_str(),
        "the" | "and" | "that" | "this" | "what" | "with" | "from" | "about" | "which"
            | "their" | "would" | "there" | "these" | "could" | "should" | "think"
            | "like" | "know" | "make" | "just" | "very" | "more" | "also" | "even"
            | "only" | "some" | "such" | "when" | "where" | "come" | "over" | "have"
            | "been" | "does" | "most" | "many" | "actually" | "really" | "still"
    )
}

/// Build concise context summary for response generation
pub fn build_context_summary(context: &ConversationContext) -> String {
    let topic_str = context
        .previous_topics
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");

    let recent_turns = context.messages.len().min(3);

    format!(
        "Topic: {} | Context: {} recent messages | Topics: {}",
        context.current_topic, recent_turns, topic_str
    )
}

/// Get most recent user question
pub fn get_last_user_question(context: &ConversationContext) -> Option<String> {
    context
        .messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.clone())
}

/// Detect if user is continuing existing topic or shifting
pub fn detect_topic_shift(context: &ConversationContext, new_message: &str) -> bool {
    let new_topics = extract_topics(new_message);

    // If no overlap with previous topics, it's a shift
    let overlap = new_topics
        .iter()
        .filter(|t| context.previous_topics.iter().any(|pt| pt.eq_ignore_ascii_case(t)))
        .count();

    overlap == 0 && !new_topics.is_empty()
}

/// Store conversation state for persistence
pub async fn save_conversation_state(
    pool: &SqlitePool,
    context: &ConversationContext,
) -> Result<(), String> {
    let state = json!({
        "session_id": context.session_id,
        "current_topic": context.current_topic,
        "previous_topics": context.previous_topics,
        "message_count": context.messages.len(),
        "timestamp": chrono::Local::now().to_rfc3339()
    });

    let key = format!("conversation_state:{}", context.session_id);

    sqlx::query(
        "INSERT INTO jeebs_store (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(&key)
    .bind(
        serde_json::to_vec(&state)
            .map_err(|e| format!("Serialization error: {}", e))?,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    Ok(())
}

/// Summarize conversation for context efficiency
pub fn summarize_conversation(context: &ConversationContext) -> String {
    let mut summary = String::new();

    if context.messages.len() > 5 {
        let recent = &context.messages[0..3];
        summary.push_str("Recent: ");
        for msg in recent {
            let preview = if msg.content.len() > 50 {
                format!("{}...", &msg.content[..50])
            } else {
                msg.content.clone()
            };
            summary.push_str(&format!("[{}] ", preview));
        }
    }

    summary
}

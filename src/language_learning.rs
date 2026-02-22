use chrono::Local;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

const LANGUAGE_PATTERNS_KEY: &str = "language:patterns";
const VOCABULARY_KEY_PREFIX: &str = "vocab:";
/// Prefix for grammar-related brain store keys (reserved for grammar rule storage)
pub const GRAMMAR_KEY_PREFIX: &str = "grammar:";
const CONTEXT_KEY_PREFIX: &str = "context:";

#[derive(Serialize, Deserialize, Clone)]
pub struct LanguagePattern {
    pub pattern: String,
    pub category: String, // "greeting", "question", "statement", "command", "expression"
    pub examples: Vec<String>,
    pub response_templates: Vec<String>,
    pub usage_count: u64,
    pub last_used: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VocabularyEntry {
    pub word: String,
    pub part_of_speech: String, // "noun", "verb", "adjective", etc.
    pub definitions: Vec<String>,
    pub examples: Vec<String>,
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
    pub frequency: u64,
    pub learned_at: String,
    #[serde(default)]
    pub sentiment: f32, // -1.0 (negative) to 1.0 (positive)
    #[serde(default)]
    pub associations: Vec<String>, // Related words/concepts
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GrammarRule {
    pub rule_name: String,
    pub description: String,
    pub examples: Vec<String>,
    pub exceptions: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ContextualKnowledge {
    pub topic: String,
    pub key_concepts: Vec<String>,
    pub related_topics: Vec<String>,
    pub facts: Vec<String>,
    pub last_updated: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Thought {
    pub internal_monologue: String,
    pub detected_sentiment: f32,
    pub curiosity_target: Option<String>,
    pub suggested_angle: String, // "empathetic", "analytical", "curious", "direct"
    pub new_concepts: Vec<String>,
}

/// Learn from user input by analyzing patterns
pub async fn learn_from_input(db: &SqlitePool, input: &str) -> Result<(), String> {
    let words = extract_words(input);

    // Learn vocabulary
    for word in &words {
        learn_vocabulary(db, word).await?;
    }

    // Detect and learn patterns
    let pattern_category = categorize_input(input);
    learn_pattern(db, input, &pattern_category).await?;

    Ok(())
}

/// Extract meaningful words from input
fn extract_words(input: &str) -> Vec<String> {
    input
        .to_lowercase()
        .split_whitespace()
        .filter(|w| w.len() > 2) // Filter out very short words
        .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()))
        .filter(|w| !w.is_empty())
        .map(String::from)
        .collect()
}

/// Categorize input type
fn categorize_input(input: &str) -> String {
    let lower = input.to_lowercase();

    if lower.starts_with("hello")
        || lower.starts_with("hi ")
        || lower.starts_with("hey")
        || lower.starts_with("good morning")
    {
        "greeting".to_string()
    } else if input.contains('?') {
        "question".to_string()
    } else if lower.starts_with("please")
        || lower.starts_with("can you")
        || lower.starts_with("could you")
    {
        "command".to_string()
    } else if lower.contains("love") || lower.contains("hate") || lower.contains("like") {
        "expression".to_string()
    } else {
        "statement".to_string()
    }
}

/// Learn a vocabulary word
async fn learn_vocabulary(db: &SqlitePool, word: &str) -> Result<(), String> {
    if word.len() < 3 || is_common_word(word) {
        return Ok(());
    }

    let key = format!("{}{}", VOCABULARY_KEY_PREFIX, word);

    // Check if word exists
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&key)
        .fetch_optional(db)
        .await
    {
        // Update frequency
        let value: Vec<u8> = row.get(0);
        if let Ok(mut entry) = serde_json::from_slice::<VocabularyEntry>(&value) {
            entry.frequency += 1;
            entry.learned_at = Local::now().to_rfc3339();

            if let Ok(updated) = serde_json::to_vec(&entry) {
                let _ = sqlx::query("UPDATE jeebs_store SET value = ? WHERE key = ?")
                    .bind(&updated)
                    .bind(&key)
                    .execute(db)
                    .await;
            }
        }
    } else {
        // Create new entry
        let entry = VocabularyEntry {
            word: word.to_string(),
            part_of_speech: guess_part_of_speech(word),
            definitions: vec![],
            examples: vec![],
            synonyms: vec![],
            antonyms: vec![],
            frequency: 1,
            learned_at: Local::now().to_rfc3339(),
            sentiment: estimate_sentiment(word),
            associations: Vec::new(),
        };

        if let Ok(payload) = serde_json::to_vec(&entry) {
            let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
                .bind(&key)
                .bind(&payload)
                .execute(db)
                .await;
        }
    }

    Ok(())
}

/// Learn a language pattern
async fn learn_pattern(db: &SqlitePool, input: &str, category: &str) -> Result<(), String> {
    let key = format!("{}:{}", LANGUAGE_PATTERNS_KEY, category);

    let mut patterns: Vec<LanguagePattern> = if let Ok(Some(row)) =
        sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
            .bind(&key)
            .fetch_optional(db)
            .await
    {
        let value: Vec<u8> = row.get(0);
        serde_json::from_slice(&value).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Check if similar pattern exists
    let normalized = normalize_pattern(input);
    let mut found = false;

    for pattern in &mut patterns {
        if pattern.pattern == normalized {
            pattern.usage_count += 1;
            pattern.last_used = Local::now().to_rfc3339();
            if !pattern.examples.contains(&input.to_string()) {
                pattern.examples.push(input.to_string());
            }
            found = true;
            break;
        }
    }

    if !found {
        patterns.push(LanguagePattern {
            pattern: normalized,
            category: category.to_string(),
            examples: vec![input.to_string()],
            response_templates: generate_response_templates(category),
            usage_count: 1,
            last_used: Local::now().to_rfc3339(),
        });
    }

    // Keep only top 50 patterns per category
    patterns.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
    patterns.truncate(50);

    if let Ok(payload) = serde_json::to_vec(&patterns) {
        let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
            .bind(&key)
            .bind(&payload)
            .execute(db)
            .await;
    }

    Ok(())
}

/// Normalize a pattern by replacing specific words with placeholders
fn normalize_pattern(input: &str) -> String {
    let mut result = input.to_lowercase();

    // Replace numbers
    for digit in 0..=9 {
        result = result.replace(&digit.to_string(), "[NUM]");
    }

    // Replace common names (simplified)
    let names = ["john", "jane", "bob", "alice", "david", "sarah"];
    for name in &names {
        result = result.replace(name, "[NAME]");
    }

    result
}

/// Generate response templates based on category
fn generate_response_templates(category: &str) -> Vec<String> {
    match category {
        "greeting" => vec![
            "Hello! How can I help you?".to_string(),
            "Hi there! What can I do for you?".to_string(),
            "Hey! Ready to assist.".to_string(),
        ],
        "question" => vec![
            "Let me search my knowledge for that...".to_string(),
            "That's a good question. Let me think...".to_string(),
            "I'll look that up in my brain.".to_string(),
        ],
        "command" => vec![
            "I'll do my best to help with that.".to_string(),
            "Working on it...".to_string(),
            "Sure, let me handle that.".to_string(),
        ],
        "expression" => vec![
            "I understand how you feel.".to_string(),
            "Thanks for sharing that.".to_string(),
            "I appreciate you telling me.".to_string(),
        ],
        _ => vec!["Got it.".to_string()],
    }
}

/// Check if word is too common to track
fn is_common_word(word: &str) -> bool {
    matches!(
        word,
        "the"
            | "a"
            | "an"
            | "and"
            | "or"
            | "but"
            | "in"
            | "on"
            | "at"
            | "to"
            | "for"
            | "of"
            | "with"
            | "by"
            | "from"
            | "up"
            | "about"
            | "into"
            | "through"
            | "during"
            | "before"
            | "after"
            | "above"
            | "below"
            | "between"
            | "under"
            | "is"
            | "are"
            | "was"
            | "were"
            | "be"
            | "been"
            | "being"
            | "have"
            | "has"
            | "had"
            | "do"
            | "does"
            | "did"
            | "will"
            | "would"
            | "could"
            | "should"
            | "may"
            | "might"
            | "must"
            | "can"
            | "this"
            | "that"
            | "these"
            | "those"
            | "i"
            | "you"
            | "he"
            | "she"
            | "it"
            | "we"
            | "they"
            | "me"
            | "him"
            | "her"
            | "us"
            | "them"
            | "my"
            | "your"
            | "his"
            | "its"
            | "our"
            | "their"
    )
}

/// Guess part of speech (simplified)
fn guess_part_of_speech(word: &str) -> String {
    if word.ends_with("ing") {
        "verb/gerund".to_string()
    } else if word.ends_with("ed") {
        "verb/past".to_string()
    } else if word.ends_with("ly") {
        "adverb".to_string()
    } else if word.ends_with("tion") || word.ends_with("ment") || word.ends_with("ness") {
        "noun".to_string()
    } else if word.ends_with("ful") || word.ends_with("less") || word.ends_with("ous") {
        "adjective".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Estimate sentiment of a word (simple heuristic for initial learning)
fn estimate_sentiment(word: &str) -> f32 {
    let w = word.to_lowercase();
    if ["good", "great", "love", "excellent", "happy", "awesome", "best", "like", "yes"].contains(&w.as_str()) {
        0.8
    } else if ["bad", "hate", "terrible", "sad", "awful", "worst", "no", "wrong", "fail"].contains(&w.as_str()) {
        -0.8
    } else if ["interesting", "cool", "okay", "fine", "sure"].contains(&w.as_str()) {
        0.3
    } else if ["hard", "difficult", "pain", "problem", "issue"].contains(&w.as_str()) {
        -0.5
    } else {
        0.0
    }
}

/// Get vocabulary statistics
pub async fn get_vocabulary_stats(db: &SqlitePool) -> Result<HashMap<String, u64>, String> {
    let rows = sqlx::query("SELECT key, value FROM jeebs_store WHERE key LIKE ?")
        .bind(format!("{}%", VOCABULARY_KEY_PREFIX))
        .fetch_all(db)
        .await
        .map_err(|e| e.to_string())?;

    let mut stats = HashMap::new();
    stats.insert("total_words".to_string(), rows.len() as u64);

    let mut total_frequency = 0u64;
    let mut parts_of_speech: HashMap<String, u64> = HashMap::new();

    for row in rows {
        let value: Vec<u8> = row.get(1);
        if let Ok(entry) = serde_json::from_slice::<VocabularyEntry>(&value) {
            total_frequency += entry.frequency;
            *parts_of_speech.entry(entry.part_of_speech).or_insert(0) += 1;
        }
    }

    stats.insert("total_frequency".to_string(), total_frequency);
    for (pos, count) in parts_of_speech {
        stats.insert(format!("pos_{}", pos), count);
    }

    Ok(stats)
}

/// Store contextual knowledge about a topic
pub async fn store_context(
    db: &SqlitePool,
    topic: &str,
    concepts: Vec<String>,
    related: Vec<String>,
    facts: Vec<String>,
) -> Result<(), String> {
    let key = format!("{}{}", CONTEXT_KEY_PREFIX, topic.to_lowercase());

    let context = ContextualKnowledge {
        topic: topic.to_string(),
        key_concepts: concepts,
        related_topics: related,
        facts,
        last_updated: Local::now().to_rfc3339(),
    };

    let payload = serde_json::to_vec(&context).map_err(|e| e.to_string())?;

    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(&key)
        .bind(&payload)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Retrieve contextual knowledge
pub async fn get_context(db: &SqlitePool, topic: &str) -> Option<ContextualKnowledge> {
    let key = format!("{}{}", CONTEXT_KEY_PREFIX, topic.to_lowercase());

    let row = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&key)
        .fetch_optional(db)
        .await
        .ok()??;

    let value: Vec<u8> = row.get(0);
    serde_json::from_slice(&value).ok()
}

/// Ponder the input to generate a thought process and cognitive stance
pub async fn ponder(db: &SqlitePool, input: &str) -> Result<Thought, String> {
    let words = extract_words(input);
    let mut total_sentiment = 0.0;
    let mut word_count = 0;
    let mut new_concepts = Vec::new();
    let mut curiosity_target = None;

    // 1. Analyze Sentiment & Novelty
    for word in &words {
        let key = format!("{}{}", VOCABULARY_KEY_PREFIX, word);
        if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
            .bind(&key)
            .fetch_optional(db)
            .await
        {
            let value: Vec<u8> = row.get(0);
            if let Ok(entry) = serde_json::from_slice::<VocabularyEntry>(&value) {
                total_sentiment += entry.sentiment;
                word_count += 1;
            }
        } else {
            // Unknown word - potential curiosity target if it's significant
            if !is_common_word(word) && word.len() > 4 {
                new_concepts.push(word.clone());
                if curiosity_target.is_none() {
                    curiosity_target = Some(word.clone());
                }
            }
        }
    }

    let avg_sentiment = if word_count > 0 {
        total_sentiment / word_count as f32
    } else {
        0.0
    };

    // 2. Determine Angle
    let angle = if avg_sentiment > 0.5 {
        "celebratory"
    } else if avg_sentiment < -0.5 {
        "empathetic"
    } else if curiosity_target.is_some() {
        "curious"
    } else if input.contains('?') {
        "helpful"
    } else {
        "conversational"
    };

    // 3. Formulate Internal Monologue
    let monologue = if !new_concepts.is_empty() {
        format!(
            "User mentioned new concepts: {:?}. I should learn more about {}. Sentiment seems {}.",
            new_concepts,
            curiosity_target.as_deref().unwrap_or("them"),
            if avg_sentiment > 0.0 { "positive" } else { "neutral/negative" }
        )
    } else if avg_sentiment.abs() > 0.4 {
        format!(
            "User is expressing strong emotion ({:.2}). I should respond with a {} tone.",
            avg_sentiment, angle
        )
    } else {
        "Input is standard. I will process this logically and check my knowledge base.".to_string()
    };

    // 4. Update Cognitive State (Simple implementation: just log it for now)
    // In a full implementation, we would store this state to persist "mood" across messages.
    
    Ok(Thought {
        internal_monologue: monologue,
        detected_sentiment: avg_sentiment,
        curiosity_target,
        suggested_angle: angle.to_string(),
        new_concepts,
    })
}

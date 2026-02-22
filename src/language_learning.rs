use chrono::Local;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

const LANGUAGE_PATTERNS_KEY: &str = "language:patterns";
const VOCABULARY_KEY_PREFIX: &str = "vocab:";
const GRAMMAR_KEY_PREFIX: &str = "grammar:";
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

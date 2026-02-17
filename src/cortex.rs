use std::collections::HashSet;
use sqlx::{SqlitePool, Row};
use crate::state::AppState;
use actix_web::web;
use serde_json::json;
use chrono::Local;
use crate::utils::{encode_all, decode_all};
use once_cell::sync::Lazy;
use log;

pub struct Cortex;

impl Cortex {
    pub async fn think(prompt: &str, data: &web::Data<AppState>) -> String {
        let db = &data.db;
        let prompt_lower = prompt.to_lowercase();

        crate::logging::log(db, "INFO", "CORTEX", &format!("Processing thought: {}", prompt)).await;

        // --- Layer 0: Deja Vu (Cache Check) ---
        if let Some(cached) = check_dejavu(prompt, db).await {
            return cached;
        }

        // --- Layer 1: Reflexes (Fast, hardcoded responses) ---
        if let Some(reflex) = check_reflexes(&prompt_lower) {
            return reflex;
        }

        // --- Layer 2: Short-term Memory (Context) ---
        if prompt_lower == "what did i just say" {
            return retrieve_last_prompt(db).await;
        }

        // --- Layer 3: Intent Router (Scored Execution) ---
        // Score plugins based on the prompt to prioritize the best match
        let mut scored_plugins: Vec<_> = data.plugins.iter().map(|p| {
            (p, score_intent(p.name(), &prompt_lower))
        }).collect();

        scored_plugins.sort_by(|a, b| b.1.cmp(&a.1));

        for (plugin, _score) in scored_plugins {
            if let Some(resp) = plugin.handle(prompt, db.clone()).await {
                if resp.starts_with("Error:") {
                    report_error_to_evolution(db, plugin.name(), &resp).await;
                    crate::logging::log(db, "ERROR", "PLUGIN", &format!("Plugin {} failed: {}", plugin.name(), resp)).await;
                }

                // Subconscious: We could spawn a background task here to analyze the interaction
                let db_clone = db.clone();
                let prompt_clone = prompt.to_string();
                let resp_clone = resp.clone();
                tokio::spawn(async move {
                    subconscious_process(prompt_clone, resp_clone, db_clone).await;
                });
                save_memory(prompt, &resp, db).await;
                return resp;
            }
        }

        // --- Layer 4: Cognition (Deep Thinking / Fallback) ---
        store_context(prompt, db).await;
        
        let response = custom_ai_logic(prompt, db).await;

        let db_clone = db.clone();
        let prompt_clone = prompt.to_string();
        let response_clone = response.clone();

        tokio::spawn(async move {
            subconscious_process(prompt_clone, response_clone, db_clone).await;
        });
        save_memory(prompt, &response, db).await;
        
        response
    }

    pub async fn dream(db: SqlitePool) {
        loop {
            // Dream logic here
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }
}

fn check_reflexes(prompt: &str) -> Option<String> {
    if prompt.contains("hello") || prompt.contains("hi ") || prompt == "hi" {
        return Some("Hello! I'm Jeebs. How can I help you today?".to_string());
    }
    None
}

async fn retrieve_last_prompt(db: &SqlitePool) -> String {
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = 'last_prompt'").fetch_optional(db).await {
        let val: Vec<u8> = row.get(0);
        if let Ok(decompressed) = decode_all(&val) {
            if let Ok(text) = String::from_utf8(decompressed) {
                return format!("You just said: '{}'.", text);
            }
        }
    }
    "I don't have any previous input from you yet.".to_string()
}

async fn store_context(prompt: &str, db: &SqlitePool) {
    if let Ok(encoded) = encode_all(prompt.as_bytes(), 1) {
        let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)").bind("last_prompt").bind(encoded).execute(db).await;
    }
}

// Use once_cell for efficient, one-time initialization of the stop words set.
static STOP_WORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        "what", "when", "where", "who", "why", "how", "the", "is", "are", "a", "an", 
        "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "i", 
        "you", "me", "my", "it", "that", "about",
    ])
});

/// The cognitive fallback logic for when no specific plugin matches the prompt.
/// It searches the knowledge graph and unstructured memories to formulate a response.
async fn custom_ai_logic(prompt: &str, db: &sqlx::SqlitePool) -> String {
    // 1. Extract Keywords (Naive NLP)
    let keywords: Vec<String> = prompt.to_lowercase().split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| w.len() > 2 && !STOP_WORDS.contains(w.as_str()))
        .collect();

    if keywords.is_empty() {
        return "I'm not sure how to respond to that. Could you be more specific?".to_string();
    }

    // 2. Concurrently search structured and unstructured memory
    let (triples_results, nodes_result) = tokio::join!(
        futures_util::future::join_all(
            keywords.iter().map(|word| crate::brain::get_triples_for_subject(db, word))
        ),
        crate::brain::search_knowledge(db, prompt)
    );

    let mut response_parts = Vec::new();

    // Process structured facts from the knowledge graph
    let facts: Vec<String> = triples_results.into_iter().enumerate().flat_map(|(i, result)| {
        match result {
            Ok(triples) => triples.into_iter().map(|t| format!("- {} {} {}.", t.subject, t.predicate, t.object)).collect(),
            Err(e) => {
                log::error!("Failed to get triples for keyword '{}': {}", keywords[i], e);
                vec![]
            }
        }
    }).collect();

    if !facts.is_empty() {
        response_parts.push("Facts from my knowledge graph:".to_string());
        response_parts.extend(facts);
    }

    // Process related context from brain nodes
    match nodes_result {
        Ok(nodes) if !nodes.is_empty() => {
            if !response_parts.is_empty() { response_parts.push("".to_string()); } // Spacer
            response_parts.push("Related context from my memory:".to_string());
            response_parts.extend(nodes.iter().map(|n| format!("- {} ({})", n.summary, n.label)));
        }
        Err(e) => log::error!("Failed to search knowledge nodes: {}", e),
        _ => {}
    }

    if !response_parts.is_empty() {
        format!("Here is what I found in my memory:\n{}", response_parts.join("\n"))
    } else {
        "I don't have any specific memories about that yet. You can teach me by providing a URL for me to train on.".to_string()
    }
}

async fn subconscious_process(prompt: String, response: String, db: SqlitePool) {
    // This runs in the background after a response is sent.
    // It can be used for sentiment analysis, self-correction, or memory consolidation.
    let log_message = format!("Reflected on: '{}' -> '{}'", prompt, response);
    crate::logging::log(&db, "DEBUG", "SUBCONSCIOUS", &log_message).await;

    let extract_keywords = |text: &str| -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| w.len() > 2 && !STOP_WORDS.contains(w.as_str()))
            .collect()
    };

    let prompt_keywords = extract_keywords(&prompt);
    let response_keywords = extract_keywords(&response);

    let common_keywords: Vec<_> = prompt_keywords
        .iter()
        .filter(|&k| response_keywords.contains(k))
        .collect();

    if common_keywords.is_empty() {
        return;
    }

    for subject in common_keywords {
        for object in &response_keywords {
            if subject != object {
                let triple = crate::brain::KnowledgeTriple {
                    subject: subject.to_string(),
                    predicate: "is".to_string(),
                    object: object.to_string(),
                    confidence: 0.5,
                };
                crate::brain::store_triple(&db, &triple).await;
            }
        }
    }
}

fn score_intent(plugin_name: &str, prompt: &str) -> i32 {
    match plugin_name {
        "Time" => if prompt.contains("time") || prompt.contains("clock") { 100 } else { 0 },
        "Calc" => if prompt.contains("math") || prompt.contains("calc") || prompt.contains("+") { 100 } else { 0 },
        "Weather" => if prompt.contains("weather") || prompt.contains("rain") { 100 } else { 0 },
        "News" => if prompt.contains("news") || prompt.contains("headline") { 100 } else { 0 },
        "System" => if prompt.contains("system") || prompt.contains("cpu") || prompt.contains("ram") { 100 } else { 0 },
        _ => 1, // Default low priority
    }
}

async fn check_dejavu(prompt: &str, db: &SqlitePool) -> Option<String> {
    let key = blake3::hash(prompt.as_bytes()).to_hex().to_string();
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(key).fetch_optional(db).await {
        let val: Vec<u8> = row.get(0);
        if let Ok(decompressed) = decode_all(&val) {
            if let Ok(text) = String::from_utf8(decompressed) {
                return Some(format!("[Deja Vu] {}", text));
            }
        }
    }
    None
}

async fn save_memory(prompt: &str, response: &str, db: &SqlitePool) {
    let key = blake3::hash(prompt.as_bytes()).to_hex().to_string();
    if let Ok(compressed) = encode_all(response.as_bytes(), 1) {
        let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
            .bind(key).bind(compressed).execute(db).await;
    }
}

async fn report_error_to_evolution(db: &SqlitePool, plugin_name: &str, error: &str) {
    let id = uuid::Uuid::new_v4().to_string();
    let title = format!("Auto-Fix: {} Error", plugin_name);
    let description = format!("The {} plugin reported an error: '{}'. I should investigate and fix this.", plugin_name, error);
    
    crate::logging::log(db, "WARN", "EVOLUTION", &format!("Reporting error for evolution: {}", title)).await;

    // Create a proposal entry directly in the store
    let update_json = json!({
        "id": id,
        "title": title,
        "author": "Jeebs (Auto-Fix)",
        "severity": "High",
        "comments": [],
        "description": description,
        "changes": [], // No automated changes; manual intervention required.
        "status": "pending",
        "created_at": Local::now().to_rfc3339(),
        "backup": null
    });

    let key = format!("evolution:update:{}", id);
    if let Ok(json_bytes) = serde_json::to_vec(&update_json) {
        if let Ok(val) = encode_all(&json_bytes, 1) {
            let _ = sqlx::query("INSERT INTO jeebs_store (key, value) VALUES (?, ?)")
                .bind(key).bind(val)
                .execute(db).await;
        }
    }

    // Create Notification for High Severity
    if update_json["severity"] == "High" {
        let notif_id = uuid::Uuid::new_v4().to_string();
        let notif_json = json!({
            "id": notif_id,
            "message": format!("High Severity Update Proposed: {}", title),
            "severity": "High",
            "created_at": Local::now().to_rfc3339(),
            "link": id
        });
        let notif_key = format!("notification:{}", notif_id);
        if let Ok(notif_bytes) = serde_json::to_vec(&notif_json) {
            if let Ok(val) = encode_all(&notif_bytes, 1) {
                let _ = sqlx::query("INSERT INTO jeebs_store (key, value) VALUES (?, ?)")
                    .bind(notif_key).bind(val)
                    .execute(db).await;
            }
        }
    }
}

use chrono::Local;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeItem {
    pub id: String,
    pub label: String,
    pub summary: String,
    pub content: String,
    pub category: String,
    pub tags: Vec<String>,
    pub relevance_score: f64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RetrievalResult {
    pub items: Vec<KnowledgeItem>,
    pub total_searched: usize,
    pub query_terms: Vec<String>,
    pub synthesized_answer: Option<String>,
}

/// Advanced search that retrieves and ranks knowledge from multiple sources
pub async fn retrieve_knowledge(
    db: &SqlitePool,
    query: &str,
    max_results: usize,
) -> Result<RetrievalResult, String> {
    let query_terms = extract_query_terms(query);
    let mut all_items = Vec::new();
    let mut seen_ids = HashSet::new();

    // 1. Search brain nodes
    let brain_items = search_brain_nodes(db, &query_terms).await?;
    for item in brain_items {
        if seen_ids.insert(item.id.clone()) {
            all_items.push(item);
        }
    }

    // 2. Search knowledge triples
    let triple_items = search_knowledge_triples(db, &query_terms).await?;
    for item in triple_items {
        if seen_ids.insert(item.id.clone()) {
            all_items.push(item);
        }
    }

    // 3. Search stored contexts
    let context_items = search_contexts(db, &query_terms).await?;
    for item in context_items {
        if seen_ids.insert(item.id.clone()) {
            all_items.push(item);
        }
    }

    // 4. Search FAQ/learned responses
    let faq_items = search_faq(db, &query_terms).await?;
    for item in faq_items {
        if seen_ids.insert(item.id.clone()) {
            all_items.push(item);
        }
    }

    // Calculate relevance scores
    for item in &mut all_items {
        item.relevance_score = calculate_relevance(query, &query_terms, item);
    }

    // Sort by relevance
    all_items.sort_by(|a, b| {
        b.relevance_score
            .partial_cmp(&a.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_searched = all_items.len();
    let top_items: Vec<KnowledgeItem> = all_items.into_iter().take(max_results).collect();

    // Synthesize answer from top results
    let synthesized = if !top_items.is_empty() {
        Some(synthesize_answer(&top_items, query))
    } else {
        None
    };

    Ok(RetrievalResult {
        items: top_items,
        total_searched,
        query_terms: query_terms.iter().map(|s| s.to_string()).collect(),
        synthesized_answer: synthesized,
    })
}

/// Extract meaningful terms from query
fn extract_query_terms(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split_whitespace()
        .filter(|w| w.len() > 2 && !is_stop_word(w))
        .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()))
        .filter(|w| !w.is_empty())
        .map(String::from)
        .collect()
}

/// Check if word is a stop word
fn is_stop_word(word: &str) -> bool {
    matches!(
        word,
        "the" | "a" | "an" | "and" | "or" | "but" | "in" | "on" | "at" | "to" | "for"
        | "of" | "with" | "by" | "from" | "is" | "are" | "was" | "were" | "be" | "been"
        | "have" | "has" | "had" | "do" | "does" | "did" | "will" | "would" | "could"
        | "should" | "this" | "that" | "what" | "which" | "who" | "when" | "where" | "why" | "how"
    )
}

/// Search brain nodes
async fn search_brain_nodes(
    db: &SqlitePool,
    terms: &[String],
) -> Result<Vec<KnowledgeItem>, String> {
    let mut items = Vec::new();

    for term in terms {
        let pattern = format!("%{}%", term);
        let rows = sqlx::query(
            "SELECT id, COALESCE(label, '') AS label, COALESCE(summary, '') AS summary,
                    COALESCE(data, '{}') AS data, created_at
             FROM brain_nodes
             WHERE label LIKE ? OR summary LIKE ? OR id LIKE ?
             ORDER BY created_at DESC
             LIMIT 20"
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(db)
        .await
        .map_err(|e| e.to_string())?;

        for row in rows {
            let id: String = row.get(0);
            let label: String = row.get(1);
            let summary: String = row.get(2);
            let data: Vec<u8> = row.get(3);
            let created_at: String = row.get(4);

            let content = if let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(&data) {
                json_val.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            } else {
                String::new()
            };

            items.push(KnowledgeItem {
                id: format!("brain:{}", id),
                label,
                summary,
                content,
                category: "brain_node".to_string(),
                tags: vec![],
                relevance_score: 0.0,
                created_at,
            });
        }
    }

    Ok(items)
}

/// Search knowledge triples
async fn search_knowledge_triples(
    db: &SqlitePool,
    terms: &[String],
) -> Result<Vec<KnowledgeItem>, String> {
    let mut items = Vec::new();

    for term in terms {
        let pattern = format!("%{}%", term);
        let rows = sqlx::query(
            "SELECT subject, predicate, object, confidence, created_at
             FROM knowledge_triples
             WHERE subject LIKE ? OR predicate LIKE ? OR object LIKE ?
             ORDER BY confidence DESC, created_at DESC
             LIMIT 15"
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(db)
        .await
        .map_err(|e| e.to_string())?;

        for row in rows {
            let subject: String = row.get(0);
            let predicate: String = row.get(1);
            let object: String = row.get(2);
            let confidence: f64 = row.get(3);
            let created_at: String = row.get(4);

            let summary = format!("{} {} {}", subject, predicate, object);

            items.push(KnowledgeItem {
                id: format!("triple:{}:{}:{}", subject, predicate, object),
                label: subject.clone(),
                summary,
                content: format!(
                    "Fact: {} {} {} (confidence: {:.2})",
                    subject, predicate, object, confidence
                ),
                category: "knowledge_triple".to_string(),
                tags: vec![subject, predicate, object],
                relevance_score: confidence,
                created_at,
            });
        }
    }

    Ok(items)
}

/// Search stored contexts
async fn search_contexts(
    db: &SqlitePool,
    terms: &[String],
) -> Result<Vec<KnowledgeItem>, String> {
    let mut items = Vec::new();

    let rows = sqlx::query("SELECT key, value FROM jeebs_store WHERE key LIKE 'context:%'")
        .fetch_all(db)
        .await
        .map_err(|e| e.to_string())?;

    for row in rows {
        let _key: String = row.get(0);
        let value: Vec<u8> = row.get(1);

        if let Ok(context) = serde_json::from_slice::<crate::language_learning::ContextualKnowledge>(&value) {
            // Check if any term matches
            let mut matches = false;
            for term in terms {
                if context.topic.to_lowercase().contains(term)
                    || context.key_concepts.iter().any(|c| c.to_lowercase().contains(term))
                    || context.facts.iter().any(|f| f.to_lowercase().contains(term))
                {
                    matches = true;
                    break;
                }
            }

            if matches {
                let content = context.facts.join(". ");
                items.push(KnowledgeItem {
                    id: format!("context:{}", context.topic),
                    label: context.topic.clone(),
                    summary: format!("Context about {}", context.topic),
                    content,
                    category: "context".to_string(),
                    tags: context.key_concepts.clone(),
                    relevance_score: 0.0,
                    created_at: context.last_updated.clone(),
                });
            }
        }
    }

    Ok(items)
}

/// Search FAQ/learned responses
async fn search_faq(db: &SqlitePool, terms: &[String]) -> Result<Vec<KnowledgeItem>, String> {
    let mut items = Vec::new();

    let rows = sqlx::query("SELECT key, value FROM jeebs_store WHERE key LIKE 'chat:faq:%'")
        .fetch_all(db)
        .await
        .map_err(|e| e.to_string())?;

    for row in rows {
        let key: String = row.get(0);
        let value: Vec<u8> = row.get(1);

        // Extract question from key
        let question = key.strip_prefix("chat:faq:").unwrap_or("");

        // Check if any term matches the question
        let mut matches = false;
        for term in terms {
            if question.contains(term) {
                matches = true;
                break;
            }
        }

        if matches {
            if let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(&value) {
                if let Some(answer) = json_val.get("answer").and_then(|v| v.as_str()) {
                    items.push(KnowledgeItem {
                        id: format!("faq:{}", question),
                        label: question.to_string(),
                        summary: format!("Q: {}", question),
                        content: answer.to_string(),
                        category: "faq".to_string(),
                        tags: vec![],
                        relevance_score: 0.0,
                        created_at: json_val
                            .get("updated_at")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    });
                }
            }
        }
    }

    Ok(items)
}

/// Calculate relevance score for a knowledge item
fn calculate_relevance(query: &str, terms: &[String], item: &KnowledgeItem) -> f64 {
    let mut score = 0.0;
    let query_lower = query.to_lowercase();
    let label_lower = item.label.to_lowercase();
    let summary_lower = item.summary.to_lowercase();
    let content_lower = item.content.to_lowercase();

    // Exact match in label (highest weight)
    if label_lower == query_lower {
        score += 10.0;
    } else if label_lower.contains(&query_lower) {
        score += 5.0;
    }

    // Term matches in label
    for term in terms {
        if label_lower.contains(term) {
            score += 2.0;
        }
        if summary_lower.contains(term) {
            score += 1.0;
        }
        if content_lower.contains(term) {
            score += 0.5;
        }
    }

    // Tag matches
    for tag in &item.tags {
        for term in terms {
            if tag.to_lowercase().contains(term) {
                score += 1.5;
            }
        }
    }

    // Category bonuses
    match item.category.as_str() {
        "brain_node" => score += 1.0,
        "knowledge_triple" => score += 0.8,
        "faq" => score += 1.2,
        "context" => score += 0.9,
        _ => {}
    }

    // Recency bonus (more recent = slightly higher score)
    if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&item.created_at) {
        let age_days = (Local::now().signed_duration_since(created)).num_days();
        if age_days < 7 {
            score += 0.5;
        } else if age_days < 30 {
            score += 0.2;
        }
    }

    score
}

/// Synthesize an answer from multiple knowledge items
fn synthesize_answer(items: &[KnowledgeItem], query: &str) -> String {
    if items.is_empty() {
        return String::new();
    }

    let mut answer_parts = Vec::new();

    // Start with most relevant item
    if let Some(top_item) = items.first() {
        if !top_item.summary.is_empty() {
            answer_parts.push(top_item.summary.clone());
        } else if !top_item.content.is_empty() {
            answer_parts.push(truncate(&top_item.content, 200));
        }
    }

    // Add supporting information from other items
    for item in items.iter().skip(1).take(3) {
        if item.category == "knowledge_triple" && !item.content.is_empty() {
            answer_parts.push(item.content.clone());
        } else if !item.summary.is_empty() && item.summary != answer_parts[0] {
            answer_parts.push(truncate(&item.summary, 150));
        }
    }

    // Combine parts
    if answer_parts.is_empty() {
        format!(
            "I found {} piece(s) of information related to '{}', but couldn't synthesize a clear answer.",
            items.len(),
            query
        )
    } else if answer_parts.len() == 1 {
        answer_parts[0].clone()
    } else {
        format!("{}. Additionally: {}", answer_parts[0], answer_parts[1..].join(". "))
    }
}

/// Truncate text to max length
fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len.min(text.len())])
    }
}

/// Get statistics about stored knowledge
pub async fn get_knowledge_stats(db: &SqlitePool) -> Result<HashMap<String, u64>, String> {
    let mut stats = HashMap::new();

    // Count brain nodes
    if let Ok(Some(row)) = sqlx::query("SELECT COUNT(*) FROM brain_nodes")
        .fetch_optional(db)
        .await
    {
        let count: i64 = row.get(0);
        stats.insert("brain_nodes".to_string(), count as u64);
    }

    // Count knowledge triples
    if let Ok(Some(row)) = sqlx::query("SELECT COUNT(*) FROM knowledge_triples")
        .fetch_optional(db)
        .await
    {
        let count: i64 = row.get(0);
        stats.insert("knowledge_triples".to_string(), count as u64);
    }

    // Count FAQ entries
    if let Ok(rows) = sqlx::query("SELECT COUNT(*) FROM jeebs_store WHERE key LIKE 'chat:faq:%'")
        .fetch_optional(db)
        .await
    {
        if let Some(row) = rows {
            let count: i64 = row.get(0);
            stats.insert("faq_entries".to_string(), count as u64);
        }
    }

    // Count contexts
    if let Ok(rows) = sqlx::query("SELECT COUNT(*) FROM jeebs_store WHERE key LIKE 'context:%'")
        .fetch_optional(db)
        .await
    {
        if let Some(row) = rows {
            let count: i64 = row.get(0);
            stats.insert("contexts".to_string(), count as u64);
        }
    }

    Ok(stats)
}

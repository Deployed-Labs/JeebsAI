use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use std::collections::HashSet;

/// Represents a question-answer pair that Jeebs has learned
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LearnedQA {
    pub id: String,
    pub question: String,
    pub answer: String,
    pub source_url: String,
    pub confidence: f64,
    pub learned_at: String,
    pub category: String,
}

/// Store a question-answer pair in the knowledge base
pub async fn store_qa_pair(
    db: &SqlitePool,
    question: &str,
    answer: &str,
    source_url: &str,
    confidence: f64,
    category: &str,
) -> Result<String, String> {
    let qa_id = format!("qa:{}", blake3::hash(question.as_bytes()).to_hex());

    // Store as brain node
    let payload = serde_json::to_vec(&json!({
        "type": "question_answer",
        "question": question,
        "answer": answer,
        "source_url": source_url,
        "confidence": confidence,
        "category": category,
        "learned_at": Local::now().to_rfc3339(),
    }))
    .unwrap_or_default();

    sqlx::query(
        "INSERT OR REPLACE INTO brain_nodes (id, label, summary, data, created_at)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&qa_id)
    .bind(format!("Q: {}", question))
    .bind(answer.chars().take(200).collect::<String>())
    .bind(payload)
    .bind(Local::now().to_rfc3339())
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    // Store as knowledge triple for retrieval
    sqlx::query(
        "INSERT OR REPLACE INTO knowledge_triples (subject, predicate, object, confidence, created_at)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(question)
    .bind("answer_is")
    .bind(answer)
    .bind(confidence)
    .bind(Local::now().to_rfc3339())
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    // Store in jeebs_store for fast retrieval
    let qa_key = format!("qa:{}", blake3::hash(question.as_bytes()).to_hex());
    let qa_data = serde_json::to_vec(&LearnedQA {
        id: qa_id.clone(),
        question: question.to_string(),
        answer: answer.to_string(),
        source_url: source_url.to_string(),
        confidence,
        learned_at: Local::now().to_rfc3339(),
        category: category.to_string(),
    })
    .unwrap_or_default();

    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(&qa_key)
        .bind(&qa_data)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(qa_id)
}

/// Search for an answer in stored Q&A pairs
pub async fn find_answer_in_memory(db: &SqlitePool, question: &str) -> Result<Option<LearnedQA>, String> {
    // Try exact match first
    let qa_key = format!("qa:{}", blake3::hash(question.as_bytes()).to_hex());

    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&qa_key)
        .fetch_optional(db)
        .await
    {
        let raw: Vec<u8> = row.get(0);
        if let Ok(qa) = serde_json::from_slice::<LearnedQA>(&raw) {
            return Ok(Some(qa));
        }
    }

    // Try similar question search
    let pattern = format!("%{}%", question);
    if let Ok(Some(row)) = sqlx::query(
        "SELECT id, json_extract(data, '$.question') as question,
                json_extract(data, '$.answer') as answer,
                json_extract(data, '$.source_url') as source_url,
                json_extract(data, '$.confidence') as confidence,
                json_extract(data, '$.category') as category,
                created_at
         FROM brain_nodes
         WHERE json_extract(data, '$.type') = 'question_answer'
         AND json_extract(data, '$.question') LIKE ?
         ORDER BY created_at DESC
         LIMIT 1"
    )
    .bind(&pattern)
    .fetch_optional(db)
    .await
    .map_err(|e| e.to_string())?
    {
        let id: String = row.get(0);
        let q: Option<String> = row.get(1);
        let a: Option<String> = row.get(2);
        let url: Option<String> = row.get(3);
        let conf: Option<f64> = row.get(4);
        let cat: Option<String> = row.get(5);
        let created: String = row.get(6);

        if let (Some(q), Some(a)) = (q, a) {
            return Ok(Some(LearnedQA {
                id,
                question: q,
                answer: a,
                source_url: url.unwrap_or_default(),
                confidence: conf.unwrap_or(0.8),
                learned_at: created,
                category: cat.unwrap_or_else(|| "general".to_string()),
            }));
        }
    }

    Ok(None)
}

/// Get all learned Q&A pairs in a category
pub async fn get_learned_qa_by_category(
    db: &SqlitePool,
    category: &str,
    limit: usize,
) -> Result<Vec<LearnedQA>, String> {
    let mut qa_pairs = Vec::new();

    let rows = sqlx::query(
        "SELECT id, json_extract(data, '$.question') as question,
                json_extract(data, '$.answer') as answer,
                json_extract(data, '$.source_url') as source_url,
                json_extract(data, '$.confidence') as confidence,
                json_extract(data, '$.category') as category,
                created_at
         FROM brain_nodes
         WHERE json_extract(data, '$.type') = 'question_answer'
         AND json_extract(data, '$.category') = ?
         ORDER BY created_at DESC
         LIMIT ?"
    )
    .bind(category)
    .bind(limit as i32)
    .fetch_all(db)
    .await
    .map_err(|e| e.to_string())?;

    for row in rows {
        let id: String = row.get(0);
        let q: Option<String> = row.get(1);
        let a: Option<String> = row.get(2);
        let url: Option<String> = row.get(3);
        let conf: Option<f64> = row.get(4);
        let cat: Option<String> = row.get(5);
        let created: String = row.get(6);

        if let (Some(q), Some(a)) = (q, a) {
            qa_pairs.push(LearnedQA {
                id,
                question: q,
                answer: a,
                source_url: url.unwrap_or_default(),
                confidence: conf.unwrap_or(0.8),
                learned_at: created,
                category: cat.unwrap_or_else(|| "general".to_string()),
            });
        }
    }

    Ok(qa_pairs)
}

/// Search the web for an answer to a question
pub async fn ask_web_question(
    client: &reqwest::Client,
    question: &str,
) -> Result<(String, String), String> {
    // Build a Google search URL
    let search_query = urlencoding::encode(question);
    let search_url = format!(
        "https://www.google.com/search?q={}",
        search_query
    );

    // Try to fetch the search results page
    let response = client
        .get(&search_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Failed to search: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Search failed with status: {}", response.status()));
    }

    let html = response.text().await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Extract answer from Google's featured snippet or search results
    let answer = extract_answer_from_html(&html, question);

    Ok((question.to_string(), answer))
}

/// Extract answer from Google search results HTML
fn extract_answer_from_html(html: &str, _question: &str) -> String {
    // Look for Google's featured snippet (answer box)
    if let Some(start) = html.find("class=\"dw4D2c\"") {
        let snippet = &html[start..];
        if let Some(end) = snippet.find("</div>") {
            let content = &snippet[..end.min(500)];
            // Remove HTML tags
            let clean = strip_html_tags(content);
            if !clean.is_empty() {
                return clean;
            }
        }
    }

    // Fallback: Look for description in search results
    if let Some(start) = html.find("class=\"VwiC3b iBnbqf\"") {
        let snippet = &html[start..];
        if let Some(end) = snippet.find("</span>") {
            let content = &snippet[..end.min(500)];
            let clean = strip_html_tags(content);
            if !clean.is_empty() {
                return clean;
            }
        }
    }

    // If no answer found, return a generic response
    "Based on web search, multiple sources discuss this topic. Search the web directly for more specific information.".to_string()
}

/// Strip HTML tags from content
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
            result.push(' ');
        } else if !in_tag {
            result.push(ch);
        }
    }

    result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Get Q&A statistics
pub async fn get_qa_statistics(db: &SqlitePool) -> Result<serde_json::Value, String> {
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM brain_nodes WHERE json_extract(data, '$.type') = 'question_answer'"
    )
    .fetch_one(db)
    .await
    .map_err(|e| e.to_string())?;

    let categories: Vec<(String, i64)> = sqlx::query_as(
        "SELECT json_extract(data, '$.category') as category, COUNT(*) as count
         FROM brain_nodes
         WHERE json_extract(data, '$.type') = 'question_answer'
         GROUP BY category
         ORDER BY count DESC"
    )
    .fetch_all(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(json!({
        "total_qa_pairs": total,
        "categories": categories,
    }))
}

/// Ask Jeebs a question (from user in chat)
pub async fn ask_jeebs_question(
    db: &SqlitePool,
    client: &reqwest::Client,
    question: &str,
    category: &str,
) -> Result<(String, bool), String> {
    // First check if we already know the answer
    if let Some(qa) = find_answer_in_memory(db, question).await? {
        return Ok((qa.answer, true)); // True = found in memory
    }

    // If not in memory, ask the web
    let (_q, answer) = ask_web_question(client, question).await?;

    // Store the new Q&A pair
    let _ = store_qa_pair(db, question, &answer, "", 0.7, category).await;

    Ok((answer, false)) // False = new answer from web
}

/// Get most recent learned questions
pub async fn get_recent_questions(
    db: &SqlitePool,
    limit: usize,
) -> Result<Vec<LearnedQA>, String> {
    let rows = sqlx::query(
        "SELECT id, json_extract(data, '$.question') as question,
                json_extract(data, '$.answer') as answer,
                json_extract(data, '$.source_url') as source_url,
                json_extract(data, '$.confidence') as confidence,
                json_extract(data, '$.category') as category,
                created_at
         FROM brain_nodes
         WHERE json_extract(data, '$.type') = 'question_answer'
         ORDER BY created_at DESC
         LIMIT ?"
    )
    .bind(limit as i32)
    .fetch_all(db)
    .await
    .map_err(|e| e.to_string())?;

    let mut qa_pairs = Vec::new();
    for row in rows {
        let id: String = row.get(0);
        let q: Option<String> = row.get(1);
        let a: Option<String> = row.get(2);
        let url: Option<String> = row.get(3);
        let conf: Option<f64> = row.get(4);
        let cat: Option<String> = row.get(5);
        let created: String = row.get(6);

        if let (Some(q), Some(a)) = (q, a) {
            qa_pairs.push(LearnedQA {
                id,
                question: q,
                answer: a,
                source_url: url.unwrap_or_default(),
                confidence: conf.unwrap_or(0.8),
                learned_at: created,
                category: cat.unwrap_or_else(|| "general".to_string()),
            });
        }
    }

    Ok(qa_pairs)
}

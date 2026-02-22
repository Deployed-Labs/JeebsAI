pub mod coded_holographic_data_storage_container;

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct BrainNode {
    pub id: Option<i64>,
    pub key: String,
    pub value: String,
    pub label: String,
    pub summary: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct KnowledgeTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f64,
}

pub async fn search_knowledge(db: &SqlitePool, query: &str) -> Vec<BrainNode> {
    let pattern = format!("%{}%", query); // Name it so it lives longer
    sqlx::query_as::<_, BrainNode>(
        "SELECT id, key, value, label, summary, created_at FROM brain WHERE key LIKE ?",
    )
    .bind(pattern)
    .fetch_all(db)
    .await
    .unwrap_or_default()
}

pub async fn get_triples_for_subject(db: &SqlitePool, subject: &str) -> Vec<KnowledgeTriple> {
    sqlx::query_as::<_, KnowledgeTriple>(
        "SELECT subject, predicate, object, confidence FROM triples WHERE subject = ?",
    )
    .bind(subject)
    .fetch_all(db)
    .await
    .unwrap_or_default()
}

pub async fn store_triple(db: &SqlitePool, triple: &KnowledgeTriple) {
    let _ = sqlx::query(
        "INSERT INTO triples (subject, predicate, object, confidence) VALUES (?, ?, ?, ?)",
    )
    .bind(&triple.subject)
    .bind(&triple.predicate)
    .bind(&triple.object)
    .bind(triple.confidence)
    .execute(db)
    .await;
}

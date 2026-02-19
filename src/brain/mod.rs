use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrainNode {
    pub id: Option<i64>,
    pub key: String,
    pub value: String,
    pub label: String,
    pub summary: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f64,
}

pub async fn search_knowledge(db: &SqlitePool, query: &str) -> Vec<BrainNode> {
    let pattern = format!("%{}%", query); // Name it so it lives longer
    sqlx::query_as!(BrainNode, 
        "SELECT id, key as 'key!', value as 'value!', label as 'label!', summary as 'summary!', created_at as 'created_at!' FROM brain WHERE key LIKE ?", 
        pattern)
        .fetch_all(db).await.unwrap_or_default()
}

pub async fn get_triples_for_subject(db: &SqlitePool, subject: &str) -> Vec<KnowledgeTriple> {
    sqlx::query_as!(KnowledgeTriple, 
        "SELECT subject as 'subject!', predicate as 'predicate!', object as 'object!', confidence as 'confidence!' FROM triples WHERE subject = ?", 
        subject)
        .fetch_all(db).await.unwrap_or_default()
}

pub async fn store_triple(db: &SqlitePool, triple: &KnowledgeTriple) {
    let _ = sqlx::query!(
        "INSERT INTO triples (subject, predicate, object, confidence) VALUES (?, ?, ?, ?)",
        triple.subject, triple.predicate, triple.object, triple.confidence)
        .execute(db).await;
}

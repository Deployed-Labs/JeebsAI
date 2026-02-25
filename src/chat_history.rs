use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ChatMessage {
    pub id: i64,
    pub session_id: Option<String>,
    pub username: Option<String>,
    pub role: String,
    pub message: String,
    pub timestamp: String,
}

pub async fn insert_chat_message(
    db: &SqlitePool,
    session_id: Option<&str>,
    username: Option<&str>,
    role: &str,
    message: &str,
) -> Result<(), sqlx::Error> {
    let filtered_message = crate::filter_knowledge::filter_knowledge_content(message);
    sqlx::query(
        "INSERT INTO chat_history (session_id, username, role, message) VALUES (?, ?, ?, ?)"
    )
    .bind(session_id)
    .bind(username)
    .bind(role)
    .bind(&filtered_message)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn fetch_chat_history(
    db: &SqlitePool,
    session_id: Option<&str>,
    username: Option<&str>,
    limit: usize,
) -> Result<Vec<ChatMessage>, sqlx::Error> {
    let mut query = String::from("SELECT id, session_id, username, role, message, timestamp FROM chat_history WHERE 1=1");
    if session_id.is_some() {
        query.push_str(" AND session_id = ?");
    }
    if username.is_some() {
        query.push_str(" AND username = ?");
    }
    query.push_str(" ORDER BY timestamp DESC LIMIT ?");

    let mut q = sqlx::query_as::<_, ChatMessage>(&query);
    if let Some(sid) = session_id {
        q = q.bind(sid);
    }
    if let Some(uname) = username {
        q = q.bind(uname);
    }
    q = q.bind(limit as i64);

    let messages = q.fetch_all(db).await?;
    Ok(messages)
}

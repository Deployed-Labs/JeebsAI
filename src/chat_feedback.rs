use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use crate::chat_history;

#[derive(Debug, Deserialize)]
pub struct FeedbackRequest {
    pub chat_message_id: i64,
    pub feedback_type: String, // 'up', 'down', 'correction'
    pub feedback_text: Option<String>,
}

#[post("/api/chat/feedback")]
pub async fn submit_feedback(
    data: web::Data<crate::state::AppState>,
    req: web::Json<FeedbackRequest>,
    http_req: HttpRequest,
) -> impl Responder {
    let username = http_req
        .headers()
        .get("x-username")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let result = sqlx::query(
        "INSERT INTO feedback (chat_message_id, username, feedback_type, feedback_text) VALUES (?, ?, ?, ?)"
    )
    .bind(req.chat_message_id)
    .bind(username)
    .bind(&req.feedback_type)
    .bind(&req.feedback_text)
    .execute(&data.db)
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json("Feedback submitted"),
        Err(e) => HttpResponse::InternalServerError().json(format!("DB error: {}", e)),
    }
}

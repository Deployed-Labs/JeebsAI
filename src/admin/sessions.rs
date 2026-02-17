use actix_web::{get, delete, web, Responder, HttpResponse};
use actix_session::Session;
use serde::Serialize;
use serde_json::json;
use sqlx::Row;
use crate::state::AppState;

#[derive(Serialize)]
pub struct UserSession {
    username: String,
    ip: String,
    user_agent: String,
    last_seen: String,
}

#[get("/api/admin/sessions")]
pub async fn get_active_sessions(data: web::Data<AppState>, session: Session) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)).unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }

    let rows = sqlx::query("SELECT username, ip, user_agent, last_seen FROM user_sessions ORDER BY last_seen DESC")
        .fetch_all(&data.db).await.unwrap_or_default();

    let sessions: Vec<UserSession> = rows.iter().map(|row| UserSession {
        username: row.get(0), ip: row.get(1), user_agent: row.get(2), last_seen: row.get(3)
    }).collect();

    HttpResponse::Ok().json(sessions)
}

#[delete("/api/admin/session/{username}")]
pub async fn terminate_session(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: Session,
) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)).unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }
    let username = path.into_inner();

    sqlx::query("DELETE FROM user_sessions WHERE username = ?").bind(username).execute(&data.db).await.unwrap();

    HttpResponse::Ok().json(json!({"ok": true}))
}
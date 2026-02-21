use crate::state::AppState;
use actix_session::Session;
use actix_web::{delete, get, web, HttpResponse, Responder};
use serde::Serialize;
use serde_json::json;
use sqlx::Row;

#[derive(Serialize)]
pub struct UserSession {
    username: String,
    ip: String,
    user_agent: String,
    last_seen: String,
}

#[get("/api/admin/sessions")]
pub async fn get_active_sessions(data: web::Data<AppState>, session: Session) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let rows = sqlx::query(
        "SELECT username, ip, user_agent, last_seen FROM user_sessions ORDER BY last_seen DESC",
    )
    .fetch_all(&data.db)
    .await
    .unwrap_or_default();

    let sessions: Vec<UserSession> = rows
        .iter()
        .map(|row| UserSession {
            username: row.get(0),
            ip: row.get(1),
            user_agent: row.get(2),
            last_seen: row.get(3),
        })
        .collect();

    HttpResponse::Ok().json(sessions)
}

#[delete("/api/admin/session/{username}")]
pub async fn terminate_session(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: Session,
) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }
    let username = path.into_inner();

    sqlx::query("DELETE FROM user_sessions WHERE username = ?")
        .bind(&username)
        .execute(&data.db)
        .await
        .unwrap();

    crate::logging::log(
        &data.db,
        "WARN",
        "SECURITY",
        &format!("Admin terminated active session username={username}"),
    )
    .await;

    HttpResponse::Ok().json(json!({"ok": true}))
}

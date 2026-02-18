use actix_session::Session;
use actix_web::{get, HttpResponse, Responder};
use serde_json::json;

#[get("/api/admin/logs")]
pub async fn get_logs(session: Session) -> impl Responder {
    let is_admin = session
        .get::<bool>("is_admin")
        .unwrap_or(Some(false))
        .unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }

    // let buffer = get_log_buffer();
    // let logs: Vec<String> = buffer.lock().unwrap().iter().cloned().collect();
    let logs: Vec<String> = Vec::new();
    HttpResponse::Ok().json(logs)
}

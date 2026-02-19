use crate::state::AppState;
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;
use sysinfo::System;

#[get("/api/admin/status")]
pub async fn get_system_status(data: web::Data<AppState>, session: Session) -> impl Responder {
    let is_admin = session
        .get::<bool>("is_admin")
        .unwrap_or(Some(false))
        .unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }

    let mut sys = data.sys.lock().unwrap();
    sys.refresh_memory();

    let used_memory = sys.used_memory();
    let total_memory = sys.total_memory();
    let uptime = System::uptime();

    HttpResponse::Ok().json(json!({
        "used_memory": used_memory,
        "total_memory": total_memory,
        "uptime": uptime,
        "uptime_formatted": format_uptime(uptime)
    }))
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    format!("{days}d {hours}h {minutes}m")
}

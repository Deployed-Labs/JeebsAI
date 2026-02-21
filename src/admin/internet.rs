use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Serialize)]
struct InternetStatusResponse {
    enabled: bool,
}

#[derive(Deserialize)]
struct SetInternetRequest {
    enabled: bool,
}

/// Get current internet connectivity status
#[get("/api/admin/internet/status")]
pub async fn get_internet_status(data: web::Data<AppState>, session: Session) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Restricted to 1090mb admin account"}));
    }

    let enabled = *data.internet_enabled.read().unwrap();
    HttpResponse::Ok().json(InternetStatusResponse { enabled })
}

/// Set internet connectivity status (admin only)
#[post("/api/admin/internet/set")]
pub async fn set_internet_status(
    data: web::Data<AppState>,
    session: Session,
    req: web::Json<SetInternetRequest>,
) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Restricted to 1090mb admin account"}));
    }

    // Update internet connectivity status
    let mut enabled = data.internet_enabled.write().unwrap();
    *enabled = req.enabled;

    // Log the change
    crate::logging::log(
        &data.db,
        "INFO",
        "internet_toggle",
        &format!(
            "Internet connectivity {} by 1090mb root admin",
            if req.enabled { "enabled" } else { "disabled" },
        ),
    )
    .await;

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "enabled": req.enabled,
        "message": format!("Internet connectivity {}", if req.enabled { "enabled" } else { "disabled" })
    }))
}

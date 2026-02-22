use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;

#[derive(Deserialize)]
struct TrainingToggleRequest {
    enabled: bool,
}

#[get("/api/admin/training/status")]
pub async fn get_training_status(data: web::Data<AppState>, session: Session) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let internet_enabled = *data.internet_enabled.read().unwrap();
    let status = crate::cortex::get_training_status(&data.db, internet_enabled).await;

    HttpResponse::Ok().json(status)
}

#[post("/api/admin/training/mode")]
pub async fn set_training_mode(
    data: web::Data<AppState>,
    session: Session,
    req: web::Json<TrainingToggleRequest>,
) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    if let Err(err) =
        crate::cortex::set_training_enabled_for_trainer(&data.db, req.enabled, "root_admin").await
    {
        return HttpResponse::InternalServerError().json(json!({ "error": err }));
    }

    let internet_enabled = *data.internet_enabled.read().unwrap();
    let status = crate::cortex::get_training_status(&data.db, internet_enabled).await;

    crate::logging::log(
        &data.db,
        "INFO",
        "training_toggle",
        &format!(
            "Training {} by 1090mb root admin",
            if req.enabled { "enabled" } else { "disabled" },
        ),
    )
    .await;

    HttpResponse::Ok().json(json!({
        "success": true,
        "message": format!("Training {}", if req.enabled { "enabled" } else { "disabled" }),
        "training": status.training,
        "internet_enabled": status.internet_enabled,
        "interval_seconds": status.interval_seconds,
    }))
}

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

/// Manually trigger a single training cycle
#[post("/api/admin/training/run")]
pub async fn run_training_now(
    data: web::Data<AppState>,
    session: Session,
) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let start = std::time::Instant::now();

    // Ensure training is enabled for this cycle
    let _ = crate::cortex::set_training_enabled_for_trainer(&data.db, true, "manual_run").await;

    // Run a deep learning session on a general topic to simulate a training cycle
    let topic = "general knowledge expansion";
    let result = crate::deep_learning::start_deep_learning_session(&data.db, topic).await;

    let duration_ms = start.elapsed().as_millis() as u64;
    let nodes_written: u64;

    match result {
        Ok(session_result) => {
            nodes_written = session_result.subtopics.len() as u64;
            crate::logging::log(
                &data.db,
                "INFO",
                "training_manual",
                &format!(
                    "Manual training cycle completed: {} nodes in {}ms",
                    nodes_written, duration_ms
                ),
            )
            .await;

            HttpResponse::Ok().json(json!({
                "success": true,
                "nodes_written": nodes_written,
                "duration_ms": duration_ms,
                "message": format!("Training cycle completed: {} nodes in {}ms", nodes_written, duration_ms)
            }))
        }
        Err(e) => {
            crate::logging::log(
                &data.db,
                "ERROR",
                "training_manual",
                &format!("Manual training cycle failed: {}", e),
            )
            .await;

            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": e,
                "duration_ms": duration_ms
            }))
        }
    }
}

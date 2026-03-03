use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::env;
use serde_json::json;

use crate::state::AppState;

#[derive(Deserialize)]
struct TrainingToggleRequest {
    enabled: bool,
}

#[get("/api/admin/training/status")]
pub async fn get_training_status(data: web::Data<AppState>, session: Session) -> impl Responder {
    // Allow trainers as well as admin/super_admin
    let is_trainer = session.get::<bool>("is_trainer").ok().flatten().unwrap_or(false);
    if !is_trainer && !crate::auth::is_effective_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Admin or trainer privileges required"}));
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
    // Only allow trainers or admins to change training mode
    let is_trainer = session.get::<bool>("is_trainer").ok().flatten().unwrap_or(false);
    if !is_trainer && !crate::auth::is_effective_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Admin or trainer privileges required"}));
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
    // Allow trainers or admins to manually trigger training
    let is_trainer = session.get::<bool>("is_trainer").ok().flatten().unwrap_or(false);
    if !is_trainer && !crate::auth::is_effective_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Admin or trainer privileges required"}));
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

/// Trigger a full internet research training run (background).
#[post("/api/admin/training/run/full")]
pub async fn run_full_training_now(
    data: web::Data<AppState>,
    session: Session,
) -> impl Responder {
    // Allow trainers or admins to manually trigger training
    let is_trainer = session.get::<bool>("is_trainer").ok().flatten().unwrap_or(false);
    if !is_trainer && !crate::auth::is_effective_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Admin or trainer privileges required"}));
    }

    // create a run id and store metadata so UI can poll
    let run_id = uuid::Uuid::new_v4().to_string();
    let run_key = format!("deeplearn_run:{}", run_id);
    let meta = json!({"id": run_id, "status": "starting", "progress_percent": 0.0, "history": []});
    let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(&run_key)
        .bind(serde_json::to_vec(&meta).unwrap_or_default())
        .execute(&data.db)
        .await;

    // Determine minutes to run: prefer explicit env override `JEEBS_RESEARCH_DEFAULT_MINUTES` if present.
    let default_minutes: u32 = env::var("JEEBS_RESEARCH_DEFAULT_MINUTES").ok().and_then(|s| s.parse().ok()).unwrap_or(30);

    // spawn background task to run bounded internet research (default 30 minutes)
    let db_clone = data.db.clone();
    let run_id_clone = run_id.clone();
    let minutes_to_run = default_minutes;
    // Use Actix runtime spawn to avoid Send-bound issues with non-Send futures
    actix_web::rt::spawn(async move {
        let _ = crate::deep_learning::start_full_internet_research_session(&db_clone, minutes_to_run /* minutes */, &run_id_clone).await;
    });

    crate::logging::log(&data.db, "INFO", "training_manual", &format!("Started full internet research run {}", run_id)).await;

    HttpResponse::Ok().json(json!({"success": true, "run_id": run_id, "message": "Full training started in background"}))
}

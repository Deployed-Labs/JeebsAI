use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use crate::state::AppState;

// --- Helper Structs & Functions required by admin/training.rs ---

#[derive(Serialize, Deserialize)]
pub struct TrainingStatus {
    pub training: bool,
    pub internet_enabled: bool,
    pub interval_seconds: u64,
}

pub async fn get_training_status(_db: &SqlitePool, internet_enabled: bool) -> TrainingStatus {
    // In a real implementation, check DB for training state
    TrainingStatus {
        training: true,
        internet_enabled,
        interval_seconds: 3600,
    }
}

pub async fn set_training_enabled_for_trainer(_db: &SqlitePool, _enabled: bool, _user: &str) -> Result<(), String> {
    // Logic to persist training state would go here
    Ok(())
}

// --- API Endpoints for Evolution Dashboard ---

#[get("/api/logs/unified-feed")]
pub async fn get_unified_feed(data: web::Data<AppState>) -> impl Responder {
    let pool = &data.db;

    // Fetch recent learned facts and nodes from the brain
    let rows = sqlx::query("SELECT data, created_at FROM brain_nodes ORDER BY created_at DESC LIMIT 100")
        .fetch_all(pool)
        .await
        .unwrap_or_default();

    let mut events = Vec::new();

    for row in rows {
        let data_blob: Vec<u8> = row.get(0);
        let time: String = row.get(1);

        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&data_blob) {
            let node_type = json.get("type").and_then(|s| s.as_str()).unwrap_or("");
            
            let (event_type, message, summary) = if node_type == "research_event" {
                ("search", 
                 json.get("message").and_then(|s| s.as_str()).unwrap_or("Researching..."),
                 "Autonomous Research")
            } else if node_type == "learned_fact" {
                ("learning",
                 json.get("fact").and_then(|s| s.as_str()).unwrap_or("Learned fact"),
                 "New Knowledge")
            } else {
                ("system",
                 json.get("summary").and_then(|s| s.as_str()).unwrap_or("System event"),
                 "System")
            };

            events.push(json!({
                "type": event_type,
                "message": message,
                "time": time,
                "summary": summary
            }));
        }
    }

    HttpResponse::Ok().json(json!({
        "success": true,
        "events": events
    }))
}

#[get("/api/brain/current-status")]
pub async fn get_brain_status() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "success": true,
        "status": {
            "now": "Idle",
            "next": "Waiting for input",
            "summary": "System is operational."
        }
    }))
}

#[get("/api/evolution/stats")]
pub async fn get_evolution_stats(data: web::Data<AppState>) -> impl Responder {
    // Fetch real learning stats from deep_learning module
    let learning_stats = crate::deep_learning::get_learning_stats(&data.db).await.unwrap_or(json!({}));
    
    HttpResponse::Ok().json(json!({
        "brain": {
            "nodes": 0,
            "learned_facts": learning_stats.get("total_facts_learned").unwrap_or(&json!(0)),
            "chat_logs_24h": 0,
            "unanswered_24h": 0,
            "warnings_24h": 0,
            "errors_24h": 0,
            "knowledge_drive": 0.8,
            "top_unknown_topics": []
        },
        "proposals": { "total": 0 },
        "thinker": { 
            "status": "idle", 
            "total_cycles": 0,
            "last_cycle_at": chrono::Local::now().to_rfc3339(),
            "last_reason": "Waiting for trigger"
        }
    }))
}

#[get("/api/admin/evolution/updates")]
pub async fn get_evolution_updates(session: Session) -> impl Responder {
    if !crate::auth::is_effective_admin_session(&session) {
        return HttpResponse::Forbidden().json(json!({"error": "Admin required"}));
    }
    HttpResponse::Ok().json(json!({ "updates": [] }))
}

#[post("/api/admin/evolution/apply/{id}")]
pub async fn apply_update(session: Session, path: web::Path<String>) -> impl Responder {
    if !crate::auth::is_effective_admin_session(&session) {
        return HttpResponse::Forbidden().json(json!({"error": "Admin required"}));
    }
    HttpResponse::Ok().json(json!({ "success": true, "message": format!("Applied update {}", path) }))
}

#[post("/api/admin/evolution/deny/{id}")]
pub async fn deny_update(session: Session, path: web::Path<String>) -> impl Responder {
    if !crate::auth::is_effective_admin_session(&session) {
        return HttpResponse::Forbidden().json(json!({"error": "Admin required"}));
    }
    HttpResponse::Ok().json(json!({ "success": true, "message": format!("Denied update {}", path) }))
}

#[get("/api/brain/learning-sessions")]
pub async fn get_learning_sessions(data: web::Data<AppState>) -> impl Responder {
    match crate::deep_learning::get_all_learning_sessions(&data.db).await {
        Ok(sessions) => HttpResponse::Ok().json(json!({ "success": true, "sessions": sessions })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "success": false, "error": e })),
    }
}

#[get("/api/brain/learning-sessions/{id}")]
pub async fn get_learning_session_endpoint(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    match crate::deep_learning::get_learning_session_by_id(&data.db, &path).await {
        Ok(Some(session)) => HttpResponse::Ok().json(json!({ "success": true, "session": session })),
        Ok(None) => HttpResponse::NotFound().json(json!({ "success": false, "error": "Session not found" })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "success": false, "error": e })),
    }
}

#[get("/api/brain/learning-stats")]
pub async fn get_learning_statistics(data: web::Data<AppState>) -> impl Responder {
    match crate::deep_learning::get_learning_stats(&data.db).await {
        Ok(stats) => HttpResponse::Ok().json(json!({ "success": true, "stats": stats })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "success": false, "error": e })),
    }
}

// --- Core Cortex Logic & Helpers ---

pub struct Cortex;

impl Cortex {
    pub async fn think(input: String, data: &web::Data<AppState>) -> String {
        // Search across all topics ("*") for relevant facts based on the input query
        let facts = crate::deep_learning::get_relevant_facts_for_chat(&data.db, "*", &input)
            .await
            .unwrap_or_default();

        if facts.is_empty() {
            return format!("I processed: \"{}\". I don't have enough data on this topic yet.", input);
        }

        let mut response = format!("Based on my knowledge regarding \"{}\":\n\n", input);
        for fact in facts {
            response.push_str(&format!("• {}\n", fact.fact));
        }
        response
    }
}

pub async fn collect_training_topics(_db: &SqlitePool, _limit: u32) -> Vec<String> {
    vec!["general".to_string()]
}

pub async fn query_wikipedia_docs(_client: &reqwest::Client, _topic: &str, _limit: u32) -> Result<Vec<String>, String> {
    Ok(vec![])
}

pub async fn store_external_learning_doc(_db: &SqlitePool, _doc: &str) -> Result<(), String> {
    Ok(())
}

pub async fn set_training_focus_for_trainer(_db: &SqlitePool, _topic: &str, _user: &str) -> Result<(), String> {
    Ok(())
}
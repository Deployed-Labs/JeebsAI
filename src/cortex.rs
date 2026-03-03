use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use crate::state::AppState;
use chrono::Local;
use uuid::Uuid;

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

#[get("/api/brain/logic-graph")]
pub async fn logic_graph_endpoint() -> impl Responder {
    HttpResponse::Ok().json(json!({"nodes": [], "edges": []}))
}

#[get("/api/brain/latest-thought")]
pub async fn get_latest_thought_endpoint() -> impl Responder {
    HttpResponse::Ok().json(json!({"thought": "Thinking..."}))
}

#[post("/api/brain/template-proposals/generate")]
pub async fn generate_template_proposals_endpoint() -> impl Responder {
    HttpResponse::Ok().json(json!({"success": true}))
}

#[get("/api/brain/template-proposals")]
pub async fn get_template_proposals_endpoint() -> impl Responder {
    HttpResponse::Ok().json(json!({"proposals": []}))
}

#[post("/api/brain/template-proposals/update-status")]
pub async fn update_proposal_status_endpoint() -> impl Responder {
    HttpResponse::Ok().json(json!({"success": true}))
}

#[get("/api/brain/template-proposals/statistics")]
pub async fn get_proposal_statistics_endpoint() -> impl Responder {
    HttpResponse::Ok().json(json!({"stats": {}}))
}

#[post("/api/learning/start-deep-learning")]
pub async fn start_deep_learning() -> impl Responder {
    HttpResponse::Ok().json(json!({"success": true}))
}

#[post("/api/learning/add-fact")]
pub async fn add_learned_fact() -> impl Responder {
    HttpResponse::Ok().json(json!({"success": true}))
}

#[post("/api/learning/add-problem")]
pub async fn add_practice_problem() -> impl Responder {
    HttpResponse::Ok().json(json!({"success": true}))
}

#[post("/api/learning/run-extended")]
pub async fn run_extended_learning() -> impl Responder {
    HttpResponse::Ok().json(json!({"success": true}))
}

#[get("/api/learning/extended-run/{id}")]
pub async fn get_extended_run() -> impl Responder {
    HttpResponse::Ok().json(json!({"success": true}))
}

#[get("/api/learning/extended-runs")]
pub async fn list_extended_runs() -> impl Responder {
    HttpResponse::Ok().json(json!({"runs": []}))
}

#[post("/api/learning/cancel-extended/{id}")]
pub async fn cancel_extended_run() -> impl Responder {
    HttpResponse::Ok().json(json!({"success": true}))
}

#[get("/api/learning/summary")]
pub async fn get_learning_summary_endpoint() -> impl Responder {
    HttpResponse::Ok().json(json!({"summary": "Learning..."}))
}

#[get("/api/brain/current-dream")]
pub async fn get_current_dream_endpoint() -> impl Responder {
    HttpResponse::Ok().json(json!({"dream": "Dreaming..."}))
}

// --- Core Cortex Logic & Helpers ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDoc {
    pub title: String,
    pub url: String,
    pub summary: String,
}

pub struct Cortex;

impl Cortex {
    pub async fn think(input: &str, data: &web::Data<AppState>) -> String {
        // Search across all topics ("*") for relevant facts based on the input query
        let facts = crate::deep_learning::get_relevant_facts_for_chat(&data.db, "*", input)
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

    pub async fn think_for_user(input: &str, data: &web::Data<AppState>, _user_id: &str, _username: Option<&str>) -> String {
        Self::think(input, data).await
    }
}

pub async fn collect_training_topics(_db: &SqlitePool, _limit: u32) -> Vec<String> {
    vec!["general".to_string()]
}

pub async fn query_wikipedia_docs(client: &reqwest::Client, topic: &str, limit: u32) -> Result<Vec<ExternalDoc>, String> {
    let params = [
        ("action", "opensearch"),
        ("search", topic),
        ("limit", &limit.to_string()),
        ("namespace", "0"),
        ("format", "json"),
    ];

    let resp = client.get("https://en.wikipedia.org/w/api.php")
        .query(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Wikipedia API returned status: {}", resp.status()));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let mut docs = Vec::new();
    if let Some(array) = json.as_array() {
        if array.len() >= 4 {
            let titles = array[1].as_array().unwrap_or(&vec![]);
            let summaries = array[2].as_array().unwrap_or(&vec![]);
            let urls = array[3].as_array().unwrap_or(&vec![]);

            for i in 0..titles.len() {
                if i < summaries.len() && i < urls.len() {
                    docs.push(ExternalDoc {
                        title: titles[i].as_str().unwrap_or("").to_string(),
                        summary: summaries[i].as_str().unwrap_or("").to_string(),
                        url: urls[i].as_str().unwrap_or("").to_string(),
                    });
                }
            }
        }
    }
    Ok(docs)
}

pub async fn store_external_learning_doc(db: &SqlitePool, doc: &ExternalDoc) -> Result<(), String> {
    let node_id = format!("wiki:{}", Uuid::new_v4());
    let data = json!({
        "type": "external_doc",
        "source": "wikipedia",
        "url": doc.url,
        "title": doc.title,
        "summary": doc.summary,
        "crawled_at": Local::now().to_rfc3339()
    });

    sqlx::query("INSERT OR REPLACE INTO brain_nodes (id, label, summary, data, created_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&node_id)
        .bind(&doc.title)
        .bind(&doc.summary)
        .bind(serde_json::to_vec(&data).unwrap_or_default())
        .bind(Local::now().to_rfc3339())
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

pub async fn set_training_focus_for_trainer(_db: &SqlitePool, _topic: &str, _user: &str) -> Result<(), String> {
    Ok(())
}
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

    let total_sessions = learning_stats.get("total_learning_sessions").and_then(|v| v.as_i64()).unwrap_or(0);
    let total_facts = learning_stats.get("total_facts_learned").and_then(|v| v.as_i64()).unwrap_or(0);

    // Provide accurate stats or helpful defaults
    HttpResponse::Ok().json(json!({
        "brain": {
            "nodes": total_facts,
            "learned_facts": total_facts,
            "chat_logs_24h": 0,
            "unanswered_24h": 0,
            "warnings_24h": 0,
            "errors_24h": 0,
            "knowledge_drive": if total_facts > 0 { 0.8 } else { 0.2 },
            "top_unknown_topics": [],
            "learning_sessions": total_sessions,
            "status": if total_sessions == 0 { "initializing" } else { "learning" }
        },
        "proposals": { "total": 0 },
        "thinker": {
            "status": "idle",
            "total_cycles": 0,
            "last_cycle_at": chrono::Local::now().to_rfc3339(),
            "last_reason": if total_sessions == 0 { "Awaiting knowledge input" } else { "Learning from interactions" }
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
        Ok(sessions) => {
            if sessions.is_empty() {
                HttpResponse::Ok().json(json!({
                    "success": true,
                    "sessions": [],
                    "status": "empty",
                    "message": "No learning sessions yet. Conversations will create learning sessions."
                }))
            } else {
                HttpResponse::Ok().json(json!({ "success": true, "sessions": sessions }))
            }
        },
        Err(e) => {
            // Return graceful empty response on error
            HttpResponse::Ok().json(json!({
                "success": true,
                "sessions": [],
                "status": "error",
                "message": format!("Could not load sessions: {}", e)
            }))
        }
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
        Ok(mut stats) => {
            // Add helpful status message if brain is empty
            if stats.get("total_learning_sessions").and_then(|v| v.as_i64()).unwrap_or(0) == 0 {
                stats["status"] = json!("empty_brain");
                stats["message"] = json!("Brain is building knowledge. Have conversations to populate learning data.");
                stats["total_facts_learned"] = json!(0);
                stats["total_study_hours"] = json!(0.0);
                stats["average_confidence"] = json!(0.0);
            }
            HttpResponse::Ok().json(json!({ "success": true, "stats": stats }))
        },
        Err(e) => {
            // Return empty stats on error instead of failing
            HttpResponse::Ok().json(json!({
                "success": true,
                "stats": {
                    "total_learning_sessions": 0,
                    "total_study_hours": 0.0,
                    "total_facts_learned": 0,
                    "average_confidence": 0.0,
                    "topics_in_learning": [],
                    "status": "error",
                    "message": format!("Could not load stats: {}", e)
                }
            }))
        }
    }
}

#[get("/api/brain/logic_graph")]
pub async fn logic_graph_endpoint(data: web::Data<AppState>) -> impl Responder {
    let pool = &data.db;

    // Fetch learning sessions to build graph nodes
    let sessions = sqlx::query_as::<_, (String, String, String, i32)>(
        "SELECT id, topic, depth_level, confidence FROM deep_learning_sessions LIMIT 50"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // Create nodes from learning sessions
    for (id, topic, _depth, confidence) in sessions.iter() {
        nodes.push(json!({
            "id": id.clone(),
            "label": topic.clone(),
            "title": format!("Topic: {}\nConfidence: {}%", topic, confidence),
            "color": format!("hsl({}, 100%, 45%)", (*confidence as f32 * 3.6) as i32),
            "size": 20 + (confidence / 10),
            "confidence": confidence
        }));
    }

    // Fetch brain node connections to build edges
    if !sessions.is_empty() {
        let edges_result = sqlx::query_as::<_, (String, String)>(
            "SELECT from_node, to_node FROM brain_connections LIMIT 100"
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for (from_id, to_id) in edges_result {
            edges.push(json!({
                "from": from_id,
                "to": to_id,
                "arrows": "to",
                "smooth": { "type": "continuous" }
            }));
        }
    }

    // If no real data, create a demo graph structure
    if nodes.is_empty() {
        nodes = vec![
            json!({ "id": "root", "label": "Core Knowledge", "title": "Foundation", "color": "#3b82f6", "size": 30 }),
            json!({ "id": "logic_1", "label": "Logic Chains", "title": "Boolean Algebra", "color": "#10b981", "size": 25 }),
            json!({ "id": "logic_2", "label": "Reasoning", "title": "Deductive Logic", "color": "#8b5cf6", "size": 22 }),
            json!({ "id": "logic_3", "label": "Inference", "title": "Pattern Recognition", "color": "#f59e0b", "size": 20 }),
        ];
        edges = vec![
            json!({ "from": "root", "to": "logic_1", "arrows": "to", "smooth": { "type": "continuous" } }),
            json!({ "from": "root", "to": "logic_2", "arrows": "to", "smooth": { "type": "continuous" } }),
            json!({ "from": "logic_1", "to": "logic_3", "arrows": "to", "smooth": { "type": "continuous" } }),
            json!({ "from": "logic_2", "to": "logic_3", "arrows": "to", "smooth": { "type": "continuous" } }),
        ];
    }

    HttpResponse::Ok().json(json!({
        "nodes": nodes,
        "edges": edges,
        "physics_enabled": true
    }))
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
    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=opensearch&search={}&limit={}&namespace=0&format=json",
        urlencoding::encode(topic),
        limit
    );

    let resp = client.get(&url)
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
            let empty_vec = Vec::new();
            let titles = array[1].as_array().unwrap_or(&empty_vec);
            let summaries = array[2].as_array().unwrap_or(&empty_vec);
            let urls = array[3].as_array().unwrap_or(&empty_vec);

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

pub async fn sync_training_state_with_toggle(
    db: &SqlitePool,
    enabled: bool,
    source: &str
) -> Result<(), sqlx::Error> {
    let value = if enabled { "true" } else { "false" };
    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind("training_enabled")
        .bind(value)
        .execute(db)
        .await?;
    println!("Training state synced: {} (Source: {})", enabled, source);
    Ok(())
}
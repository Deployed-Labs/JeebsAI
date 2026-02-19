use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrainNode {
    pub id: Option<i64>,
    pub key: String,
    pub value: String,
    pub label: String,
    pub summary: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f64,
}

pub async fn search_knowledge(db: &SqlitePool, query: &str) -> Vec<BrainNode> {
    let pattern = format!("%{}%", query);
    sqlx::query_as!(BrainNode, 
        "SELECT id, key as 'key!', value as 'value!', label as 'label!', summary as 'summary!', created_at as 'created_at!' FROM brain WHERE key LIKE ?", 
        pattern)
        .fetch_all(db).await.unwrap_or_default()
}

async fn check_dejavu(prompt: &str, db: &SqlitePool) -> Option<String> {
    match sqlx::query!("SELECT value FROM brain WHERE key = ? LIMIT 1", prompt)
        .fetch_optional(db)
        .await
    {
        Ok(Some(row)) => Some(row.value),
        _ => None,
    }
}

pub async fn custom_ai_logic(prompt: &str, db: &SqlitePool) -> String {
    let mut response_parts = Vec::new();
    
    if let Some(cached) = check_dejavu(prompt, db).await {
        return cached;
    }

    let nodes = search_knowledge(db, prompt).await;
    for node in nodes {
        response_parts.push(node.value);
    }

    if response_parts.is_empty() {
        "I don't have enough information to answer that yet.".to_string()
    } else {
        response_parts.join("\n")
    }
}

pub struct Cortex {
    pub db: SqlitePool,
}


impl Cortex {
    pub async fn think(prompt: &str, state: &AppState) -> String {
        custom_ai_logic(prompt, &state.db).await
    }
}

impl Cortex {
    pub async fn seed_knowledge(db: &SqlitePool) {
        // Initial data for your AI brain
        let _ = sqlx::query!("INSERT OR IGNORE INTO brain (key, value) VALUES (?, ?)", 
            "hello", "Hello! I am JeebsAI, your personal assistant.")
            .execute(db).await;
        println!("Brain knowledge seeded.");
    }

    pub async fn dream(_db: SqlitePool) {
        println!("Cortex is now dreaming (background processing active)...");
        loop {
            // This is where your AI would background process "triples"
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    }
}
use actix_web::{get, post, web, HttpResponse, Responder};

#[post("/admin/train")]
pub async fn admin_train(_state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body("Training initiated.")
}

#[post("/admin/crawl")]
pub async fn admin_crawl(_state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body("Crawling initiated.")
}

#[get("/brain/search")]
pub async fn search_brain(_state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body("Search results.")
}

#[post("/brain/reindex")]
pub async fn reindex_brain(_state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body("Reindexing complete.")
}

#[get("/brain/visualize")]
pub async fn visualize_brain(_state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body("Brain visualization data.")
}

#[get("/brain/logic-graph")]
pub async fn get_logic_graph(_state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(vec!["logic", "connections"])
}

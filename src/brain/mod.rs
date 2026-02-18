use sqlx::{FromRow, Row, SqlitePool};
use meval;
use chrono;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use actix_web::{get, post, web, Responder, HttpResponse};
use actix_session::Session;
use log;
use reqwest;
use scraper::{Html, Selector};
use crate::state::AppState;
use crate::utils::{encode_all, decode_all};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BrainNode {
    pub id: String,
    pub label: String,
    pub summary: String,
    pub sources: Vec<String>,
    pub edges: HashSet<String>,
    pub last_trained: String,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, FromRow)]
pub struct KnowledgeTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f64,
}

// --- Restored Functions ---

pub async fn store_brain_node(db: &SqlitePool, node: &BrainNode) {
    if let Ok(val) = serde_json::to_vec(node) {
        if let Ok(compressed) = encode_all(&val[..], 1) {
            if let Err(e) = sqlx::query("INSERT OR REPLACE INTO brain_nodes (id, label, summary, data, created_at) VALUES (?, ?, ?, ?, ?)")
                .bind(&node.id)
                .bind(&node.label)
                .bind(&node.summary)
                .bind(&compressed)
                .bind(&node.created_at)
                .execute(db).await 
            {
                log::error!("Failed to store brain node {}: {}", node.id, e);
            }
        }
    }
}

pub async fn get_brain_node(db: &SqlitePool, id: &str) -> Option<BrainNode> {
    if let Ok(Some(row)) = sqlx::query("SELECT data FROM brain_nodes WHERE id = ?").bind(id).fetch_optional(db).await {
        let val: Vec<u8> = row.get(0);
        return decode_all(&val).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok());
    }
    None
}

pub async fn store_triple(db: &SqlitePool, triple: &KnowledgeTriple) {
    if let Err(e) = sqlx::query("INSERT OR REPLACE INTO knowledge_triples (subject, predicate, object, confidence) VALUES (?, ?, ?, ?)")
        .bind(&triple.subject)
        .bind(&triple.predicate)
        .bind(&triple.object)
        .bind(triple.confidence)
        .execute(db).await {
        log::error!("Failed to store triple {} {} {}: {}", triple.subject, triple.predicate, triple.object, e);
    }
}

pub async fn get_triples_for_subject(db: &SqlitePool, subject: &str) -> sqlx::Result<Vec<KnowledgeTriple>> {
    sqlx::query_as(
        "SELECT subject, predicate, object, confidence FROM knowledge_triples WHERE subject = ?",
    )
    .bind(subject)
    .fetch_all(db)
    .await
}

pub async fn search_knowledge(db: &SqlitePool, query: &str) -> sqlx::Result<Vec<BrainNode>> {
    let term = format!("%{}%", query);
    let rows = sqlx::query("SELECT data FROM brain_nodes WHERE label LIKE ? OR summary LIKE ? LIMIT 3")
        .bind(&term)
        .bind(&term)
        .fetch_all(db).await?;

    let nodes = rows.iter().filter_map(|row| {
        let val: Vec<u8> = row.get(0);
        match decode_all(&val) {
            Ok(bytes) => match serde_json::from_slice(&bytes) {
                Ok(node) => Some(node),
                Err(e) => {
                    log::error!("Failed to deserialize BrainNode: {}", e);
                    None
                }
            },
            Err(e) => {
                log::error!("Failed to decompress BrainNode data: {}", e);
                None
            }
        }
    }).collect();
    Ok(nodes)
}

#[derive(Deserialize)]
pub struct TrainRequest {
    pub url: String,
}

#[post("/api/admin/train")]
pub async fn admin_train(
    data: web::Data<AppState>,
    req: web::Json<TrainRequest>,
    session: Session,
) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").ok().flatten().unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Admin only"}));
    }

    let url = &req.url;
    let client = reqwest::Client::new();
    let res = match client.get(url).header("User-Agent", "JeebsAI/1.0").send().await {
        Ok(r) => r,
        Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()})),
    };
    
    let body = match res.text().await {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let doc = Html::parse_document(&body);
    let title_selector = match Selector::parse("title") {
        Ok(s) => s,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("Selector error: {}", e)})),
    };
    let title = doc.select(&title_selector).next().map(|e| e.text().collect::<String>()).unwrap_or_else(|| url.to_string());
    let mut text = String::new();
    if let Ok(selector) = Selector::parse("p") {
        for el in doc.select(&selector) {
            text.push_str(&el.text().collect::<Vec<_>>().join(" "));
            text.push(' ');
        }
    }
    let summary: String = text.chars().take(400).collect();
    let id = blake3::hash(url.as_bytes()).to_hex().to_string();
    let node = BrainNode {
        id: id.clone(),
        label: title,
        summary,
        sources: vec![url.clone()],
        edges: HashSet::new(),
        last_trained: chrono::Local::now().to_rfc3339(),
        created_at: Some(chrono::Local::now().to_rfc3339()),
    };
    store_brain_node(&data.db, &node).await;
    crate::logging::log(&data.db, "INFO", "BRAIN", &format!("Trained on URL: {}", url)).await;
    HttpResponse::Ok().json(serde_json::json!({"ok": true, "id": id, "label": node.label}))
}

#[derive(Deserialize)]
pub struct CrawlRequest {
    pub url: String,
    pub depth: u32,
}

#[post("/api/admin/crawl")]
pub async fn admin_crawl(
    data: web::Data<AppState>,
    req: web::Json<CrawlRequest>,
    session: Session,
) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").ok().flatten().unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Admin only"}));
    }

    let start_url = req.url.clone();
    let max_depth = req.depth.min(3); // Hard limit depth to 3 for safety
    let db = data.db.clone();

    tokio::spawn(async move {
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start_url, 0));
        let mut visited = HashSet::new();

        // Limit total pages to avoid database explosion
        let mut pages_crawled = 0;
        let max_pages = 50; 

        while let Some((url, depth)) = queue.pop_front() {
            if depth > max_depth || pages_crawled >= max_pages { continue; }
            if !visited.insert(url.clone()) { continue; }

            println!("Jeebs Crawling: {}", url);
            crate::logging::log(&db, "INFO", "CRAWLER", &format!("Crawling: {}", url)).await;

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .user_agent("JeebsAI/1.0")
                .build()
                .unwrap_or_default();

            if let Ok(res) = client.get(&url).send().await {
                if let Ok(body) = res.text().await {
                    // Extract all data from doc in a non-async block so Html (non-Send) is dropped before .await
                    let (title, summary, links) = {
                        let doc = Html::parse_document(&body);

                        let title = if let Ok(sel) = Selector::parse("title") {
                            doc.select(&sel).next().map(|e| e.text().collect::<String>()).unwrap_or_else(|| url.clone())
                        } else {
                            url.clone()
                        };
                        let mut text = String::new();
                        if let Ok(selector) = Selector::parse("p") {
                            for el in doc.select(&selector) {
                                text.push_str(&el.text().collect::<Vec<_>>().join(" "));
                                text.push(' ');
                            }
                        }
                        let summary: String = text.chars().take(600).collect();

                        let mut links = Vec::new();
                        if depth < max_depth {
                            if let Ok(selector) = Selector::parse("a[href]") {
                                for element in doc.select(&selector) {
                                    if let Some(href) = element.value().attr("href") {
                                        if href.starts_with("http") {
                                            links.push(href.to_string());
                                        } else if href.starts_with("/") {
                                            if let Ok(base) = reqwest::Url::parse(&url) {
                                                if let Ok(joined) = base.join(href) {
                                                    links.push(joined.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        (title, summary, links)
                    };
                    
                    let id = blake3::hash(url.as_bytes()).to_hex().to_string();
                    let node = BrainNode {
                        id: id.clone(),
                        label: title,
                        summary,
                        sources: vec![url.clone()],
                        edges: HashSet::new(),
                        last_trained: chrono::Local::now().to_rfc3339(),
                        created_at: Some(chrono::Local::now().to_rfc3339()),
                    };
                    store_brain_node(&db, &node).await;
                    pages_crawled += 1;

                    for link in links {
                        queue.push_back((link, depth + 1));
                    }
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await; // 1s delay to be polite
        }
        println!("Jeebs finished crawling.");
        crate::logging::log(&db, "INFO", "CRAWLER", "Crawl job finished.").await;
    });

    HttpResponse::Ok().json(serde_json::json!({"ok": true, "message": "Jeebs unleashed! Crawling in background."}))
}

#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
}

#[post("/api/brain/search")]
pub async fn search_brain(
    data: web::Data<AppState>,
    req: web::Json<SearchRequest>,
) -> impl Responder {
    let db = &data.db;
    let term = format!("%{}%", req.query);
    let rows = match sqlx::query("SELECT data FROM brain_nodes WHERE label LIKE ? OR summary LIKE ? LIMIT 20")
        .bind(&term)
        .bind(&term)
        .fetch_all(db).await {
            Ok(r) => r,
            Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
        };

    let nodes: Vec<BrainNode> = rows.iter().filter_map(|row| {
        let val: Vec<u8> = row.get(0);
        decode_all(&val).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok())
    }).collect();
    
    HttpResponse::Ok().json(nodes)
}

#[post("/api/admin/reindex")]
pub async fn reindex_brain(
    data: web::Data<AppState>,
    session: Session,
) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").ok().flatten().unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Admin only"}));
    }

    let db = &data.db;
    let rows = match sqlx::query("SELECT value FROM jeebs_store WHERE key LIKE 'brain:node:%'").fetch_all(db).await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };
    
    let mut count = 0;
    for row in rows {
        let val: Vec<u8> = row.get(0);
        if let Ok(bytes) = decode_all(&val) {
            if let Ok(node) = serde_json::from_slice::<BrainNode>(&bytes) {
                store_brain_node(db, &node).await;
                count += 1;
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({"ok": true, "migrated": count}))
}

#[derive(Serialize)]
struct GraphNode {
    id: String,
    label: String,
    title: String,
}

#[derive(Serialize)]
struct GraphEdge {
    from: String,
    to: String,
}

#[get("/api/brain/visualize")]
pub async fn visualize_brain(data: web::Data<AppState>) -> impl Responder {
    let db = &data.db;
    let rows = match sqlx::query("SELECT data FROM brain_nodes").fetch_all(db).await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let nodes: Vec<BrainNode> = rows.iter().filter_map(|row| {
        let val: Vec<u8> = row.get(0);
        decode_all(&val).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok())
    }).collect();

    let mut graph_nodes = Vec::new();
    let mut graph_edges = Vec::new();

    for node in nodes {
        graph_nodes.push(GraphNode {
            id: node.id.clone(),
            label: node.label.clone(),
            title: node.summary.chars().take(150).collect::<String>(),
        });
        for target in node.edges {
            graph_edges.push(GraphEdge {
                from: node.id.clone(),
                to: target,
            });
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "nodes": graph_nodes,
        "edges": graph_edges
    }))
}

#[get("/api/brain/logic_graph")]
pub async fn get_logic_graph(data: web::Data<AppState>) -> impl Responder {
    let rows = sqlx::query("SELECT subject, predicate, object FROM knowledge_triples LIMIT 1000")
        .fetch_all(&data.db).await.unwrap_or_default();

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut seen = HashSet::new();

    for row in rows {
        let s: String = row.get(0);
        let p: String = row.get(1);
        let o: String = row.get(2);

        if seen.insert(s.clone()) {
            nodes.push(serde_json::json!({ "id": s, "label": s, "shape": "box", "color": "#97C2FC" }));
        }
        if seen.insert(o.clone()) {
            nodes.push(serde_json::json!({ "id": o, "label": o, "shape": "box", "color": "#FFD700" }));
        }
        edges.push(serde_json::json!({ "from": s, "to": o, "label": p, "arrows": "to" }));
    }

    HttpResponse::Ok().json(serde_json::json!({ "nodes": nodes, "edges": edges }))
}

pub async fn seed_knowledge(db: &SqlitePool) {
    if let Ok(row) = sqlx::query("SELECT COUNT(*) FROM knowledge_triples").fetch_one(db).await {
        let count: i64 = row.get(0);
        if count == 0 {
            let facts = vec![
                ("sky", "is", "blue"),
                ("sun", "is", "hot"),
                ("water", "is", "wet"),
                ("fire", "is", "hot"),
                ("bird", "has", "wings"),
                ("dog", "has", "tail"),
                ("human", "has", "brain"),
                ("jeebs", "is", "ai"),
                ("rust", "is", "fast"),
            ];

            for (subj, pred, obj) in facts {
                let triple = KnowledgeTriple {
                    subject: subj.to_string(),
                    predicate: pred.to_string(),
                    object: obj.to_string(),
                    confidence: 1.0,
                };
                store_triple(db, &triple).await;
            }
            println!("Jeebs has learned basic facts.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_db() -> SqlitePool {
        let db = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query("CREATE TABLE brain_nodes (id TEXT PRIMARY KEY, label TEXT, summary TEXT, data BLOB, created_at TEXT)")
            .execute(&db)
            .await
            .unwrap();

        sqlx::query("CREATE TABLE knowledge_triples (subject TEXT, predicate TEXT, object TEXT, confidence REAL, PRIMARY KEY (subject, predicate, object))")
            .execute(&db)
            .await
            .unwrap();

        db
    }

    #[tokio::test]
    async fn test_store_and_retrieve_brain_node() {
        let db = setup_db().await;
        let node = BrainNode {
            id: "test-node".to_string(),
            label: "Test Node".to_string(),
            summary: "A summary of the test node.".to_string(),
            sources: vec!["http://example.com".to_string()],
            edges: HashSet::new(),
            last_trained: "2023-01-01T00:00:00Z".to_string(),
            created_at: Some("2023-01-01T00:00:00Z".to_string()),
        };

        store_brain_node(&db, &node).await;

        let retrieved = get_brain_node(&db, "test-node").await;
        assert!(retrieved.is_some());
        let r = retrieved.unwrap();
        assert_eq!(r.label, "Test Node");
        assert_eq!(r.summary, "A summary of the test node.");
    }

    #[tokio::test]
    async fn test_store_and_search_knowledge() {
        let db = setup_db().await;
        let node = BrainNode {
            id: "rust-lang".to_string(),
            label: "Rust Language".to_string(),
            summary: "Rust is a systems programming language.".to_string(),
            sources: vec![],
            edges: HashSet::new(),
            last_trained: "2023-01-01T00:00:00Z".to_string(),
            created_at: None,
        };
        store_brain_node(&db, &node).await;

        let results = search_knowledge(&db, "Rust").await.expect("Search failed");
        assert!(!results.is_empty());
        assert_eq!(results[0].label, "Rust Language");
    }

    #[tokio::test]
    async fn test_store_triple_error_handling() {
        let db = setup_db().await;
        // Drop table to force an error
        sqlx::query("DROP TABLE knowledge_triples").execute(&db).await.unwrap();

        let triple = KnowledgeTriple {
            subject: "S".to_string(), predicate: "P".to_string(), object: "O".to_string(), confidence: 1.0
        };
        
        // Should not panic, just log error
        store_triple(&db, &triple).await;
    }

    #[tokio::test]
    async fn test_get_triples_error_handling() {
        let db = setup_db().await;
        
        // Happy path (empty)
        let res = get_triples_for_subject(&db, "NonExistent").await;
        assert!(res.is_ok());
        assert!(res.unwrap().is_empty());

        // Error path
        sqlx::query("DROP TABLE knowledge_triples").execute(&db).await.unwrap();
        let res = get_triples_for_subject(&db, "Something").await;
        assert!(res.is_err());
    }
}
use crate::state::AppState;
use crate::utils::{decode_all, encode_all};
use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, SqlitePool};
use std::collections::HashSet;

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

pub async fn get_triples_for_subject(db: &SqlitePool, subject: &str) -> Vec<KnowledgeTriple> {
    let rows = sqlx::query(
        "SELECT subject, predicate, object, confidence FROM knowledge_triples WHERE subject = ?",
    )
    .bind(subject)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    rows.iter()
        .map(|row| KnowledgeTriple {
            subject: row.get(0),
            predicate: row.get(1),
            object: row.get(2),
            confidence: row.get(3),
        })
        .collect()
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
    let is_admin = session
        .get::<bool>("is_admin")
        .unwrap_or(Some(false))
        .unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Admin only"}));
    }

    let url = &req.url;
    let client = reqwest::Client::new();
    let res = match client
        .get(url)
        .header("User-Agent", "JeebsAI/1.0")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()}));
        }
    };

    let body = match res.text().await {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}));
        }
    };

    let doc = Html::parse_document(&body);
    let title = doc
        .select(&Selector::parse("title").unwrap())
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or_else(|| url.to_string());
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
    crate::logging::log(
        &data.db,
        "INFO",
        "BRAIN",
        &format!("Trained on URL: {url}"),
    )
    .await;
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
    let is_admin = session
        .get::<bool>("is_admin")
        .unwrap_or(Some(false))
        .unwrap_or(false);
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
            if depth > max_depth || pages_crawled >= max_pages {
                continue;
            }
            if !visited.insert(url.clone()) {
                continue;
            }

            println!("Jeebs Crawling: {url}");
            crate::logging::log(&db, "INFO", "CRAWLER", &format!("Crawling: {url}")).await;

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .user_agent("JeebsAI/1.0")
                .build()
                .unwrap_or_default();

            if let Ok(res) = client.get(&url).send().await {
                if let Ok(body) = res.text().await {
                    // Offload HTML parsing to a blocking thread so `scraper::Html` (which
                    // is not `Send`) never becomes part of the async future captured by
                    // `tokio::spawn`.
                    // Clone `url` for the blocking closure so the outer `url` remains available
                    let url_clone = url.clone();
                    let parse_result = tokio::task::spawn_blocking(move || {
                        let doc = Html::parse_document(&body);

                        let title = doc
                            .select(&Selector::parse("title").unwrap())
                            .next()
                            .map(|e| e.text().collect::<String>())
                            .unwrap_or_else(|| url_clone.clone());

                        let mut text = String::new();
                        if let Ok(selector) = Selector::parse("p") {
                            for el in doc.select(&selector) {
                                text.push_str(&el.text().collect::<Vec<_>>().join(" "));
                                text.push(' ');
                            }
                        }
                        let summary: String = text.chars().take(600).collect();

                        let mut links = Vec::new();
                        if let Ok(selector) = Selector::parse("a[href]") {
                            for element in doc.select(&selector) {
                                if let Some(href) = element.value().attr("href") {
                                    links.push(href.to_string());
                                }
                            }
                        }

                        (title, summary, links)
                    })
                    .await
                    .unwrap_or_else(|_| (url.clone(), String::new(), Vec::new()));

                    let (title, summary, links) = parse_result;

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

                    if depth < max_depth {
                        for href in links {
                            if href.starts_with("http") {
                                queue.push_back((href.to_string(), depth + 1));
                            } else if href.starts_with("/") {
                                if let Ok(base) = reqwest::Url::parse(&url) {
                                    if let Ok(joined) = base.join(&href) {
                                        queue.push_back((joined.to_string(), depth + 1));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await; // 1s delay to be polite
        }
        println!("Jeebs finished crawling.");
        crate::logging::log(&db, "INFO", "CRAWLER", "Crawl job finished.").await;
    });

    HttpResponse::Ok().json(
        serde_json::json!({"ok": true, "message": "Jeebs unleashed! Crawling in background."}),
    )
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
    let rows =
        sqlx::query("SELECT data FROM brain_nodes WHERE label LIKE ? OR summary LIKE ? LIMIT 20")
            .bind(&term)
            .bind(&term)
            .fetch_all(db)
            .await
            .unwrap();

    let nodes: Vec<BrainNode> = rows
        .iter()
        .filter_map(|row| {
            let val: Vec<u8> = row.get(0);
            decode_all(&val)
                .ok()
                .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        })
        .collect();

    HttpResponse::Ok().json(nodes)
}

/// Search knowledge nodes by label/summary. Returns Result so callers can handle DB errors.
pub async fn search_knowledge(db: &SqlitePool, query: &str) -> Result<Vec<BrainNode>, sqlx::Error> {
    let term = format!("%{query}%");
    let rows =
        sqlx::query("SELECT data FROM brain_nodes WHERE label LIKE ? OR summary LIKE ? LIMIT 20")
            .bind(&term)
            .bind(&term)
            .fetch_all(db)
            .await?;

    let mut nodes = Vec::new();
    for row in rows {
        let val: Vec<u8> = row.get(0);
        if let Ok(decompressed) = decode_all(&val) {
            if let Ok(node) = serde_json::from_slice::<BrainNode>(&decompressed) {
                nodes.push(node);
            }
        }
    }
    Ok(nodes)
}

#[post("/api/admin/reindex")]
pub async fn reindex_brain(data: web::Data<AppState>, session: Session) -> impl Responder {
    let is_admin = session
        .get::<bool>("is_admin")
        .unwrap_or(Some(false))
        .unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Admin only"}));
    }

    let db = &data.db;
    let rows = sqlx::query("SELECT value FROM jeebs_store WHERE key LIKE 'brain:node:%'")
        .fetch_all(db)
        .await
        .unwrap();

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
    let rows = sqlx::query("SELECT data FROM brain_nodes")
        .fetch_all(db)
        .await
        .unwrap();

    let nodes: Vec<BrainNode> = rows
        .iter()
        .filter_map(|row| {
            let val: Vec<u8> = row.get(0);
            decode_all(&val)
                .ok()
                .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        })
        .collect();

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
        .fetch_all(&data.db)
        .await
        .unwrap_or_default();

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut seen = HashSet::new();

    for row in rows {
        let s: String = row.get(0);
        let p: String = row.get(1);
        let o: String = row.get(2);

        if seen.insert(s.clone()) {
            nodes.push(
                serde_json::json!({ "id": s, "label": s, "shape": "box", "color": "#97C2FC" }),
            );
        }
        if seen.insert(o.clone()) {
            nodes.push(
                serde_json::json!({ "id": o, "label": o, "shape": "box", "color": "#FFD700" }),
            );
        }
        edges.push(serde_json::json!({ "from": s, "to": o, "label": p, "arrows": "to" }));
    }

    HttpResponse::Ok().json(serde_json::json!({ "nodes": nodes, "edges": edges }))
}

pub async fn seed_knowledge(db: &SqlitePool) {
    if let Ok(row) = sqlx::query("SELECT COUNT(*) FROM knowledge_triples")
        .fetch_one(db)
        .await
    {
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

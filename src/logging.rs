use actix::{self, ActorContext, AsyncContext};
use sqlx::{SqlitePool, Row};
use chrono::Local;
use actix_web::{get, delete, web, HttpResponse, Responder};
use actix_web_actors::ws;
use csv::Writer;
use serde::{Serialize, Deserialize};
use crate::state::AppState;
use actix_session::Session;
use tokio::sync::broadcast;
use std::sync::OnceLock;
use tokio_stream::wrappers::BroadcastStream;
use futures_util::StreamExt;

#[derive(Serialize, Clone, sqlx::FromRow)]
pub struct LogEntry {
    pub id: i64,
    pub timestamp: String,
    pub level: String,
    pub category: String,
    pub message: String,
}

// Global broadcaster for real-time logs
static LOG_BROADCASTER: OnceLock<broadcast::Sender<LogEntry>> = OnceLock::new();

fn get_broadcaster() -> &'static broadcast::Sender<LogEntry> {
    LOG_BROADCASTER.get_or_init(|| {
        let (tx, _) = broadcast::channel(100);
        tx
    })
}

pub async fn init(db: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS system_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            level TEXT NOT NULL,
            category TEXT NOT NULL,
            message TEXT NOT NULL
        )"
    )
    .execute(db)
    .await
    .expect("Failed to initialize logging table");
}

pub async fn log(db: &SqlitePool, level: &str, category: &str, message: &str) {
    let timestamp = Local::now().to_rfc3339();
    let res = sqlx::query("INSERT INTO system_logs (timestamp, level, category, message) VALUES (?, ?, ?, ?)")
        .bind(timestamp)
        .bind(level)
        .bind(category)
        .bind(message)
        .execute(db).await;
    
    if let Ok(r) = res {
        let entry = LogEntry {
            id: r.last_insert_rowid(),
            timestamp: Local::now().to_rfc3339(),
            level: level.to_string(),
            category: category.to_string(),
            message: message.to_string(),
        };
        let _ = get_broadcaster().send(entry);
    }
}

/// Deletes log entries from the database that are older than 30 days.
pub async fn cleanup_old_logs(db: &SqlitePool) {
    // RFC3339 timestamps are lexicographically sortable, making this comparison reliable.
    let thirty_days_ago = (Local::now() - chrono::Duration::days(30)).to_rfc3339();
    
    match sqlx::query("DELETE FROM system_logs WHERE timestamp < ?")
        .bind(thirty_days_ago)
        .execute(db)
        .await
    {
        Ok(result) => {
            let rows_affected = result.rows_affected();
            if rows_affected > 0 {
                // Log the cleanup event itself.
                log(db, "INFO", "SYSTEM", &format!("Log cleanup task finished. Removed {} entries older than 30 days.", rows_affected)).await;
            }
        },
        Err(e) => eprintln!("[ERROR] Failed to execute log cleanup task: {}", e),
    }
}

#[derive(Deserialize)]
pub struct LogQuery {
    category: Option<String>,
    page: Option<u32>,
    limit: Option<u32>,
    search: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    level: Option<String>,
}

#[get("/api/admin/logs")]
pub async fn get_logs(data: web::Data<AppState>, query: web::Query<LogQuery>, session: Session) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)).unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Admin only"}));
    }

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).max(1).min(1000);
    let offset = (page - 1) * limit;

    let search_term = query.search.as_deref().unwrap_or("");
    let has_search = !search_term.is_empty();
    let search_pattern = format!("%{}%", search_term);

    // Build dynamic query
    let mut conditions = Vec::new();
    if let Some(cat) = &query.category {
        if cat != "All" && !cat.is_empty() {
            conditions.push("category = ?");
        }
    }
    if let Some(lvl) = &query.level {
        if lvl != "All" && !lvl.is_empty() {
            conditions.push("level = ?");
        }
    }
    if has_search {
        conditions.push("message LIKE ?");
    }
    if let Some(start) = &query.start_date {
        if !start.is_empty() { conditions.push("timestamp >= ?"); }
    }
    if let Some(end) = &query.end_date {
        if !end.is_empty() { conditions.push("timestamp <= ?"); }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!("SELECT id, timestamp, level, category, message FROM system_logs {} ORDER BY id DESC LIMIT ? OFFSET ?", where_clause);
    let mut q = sqlx::query_as::<_, LogEntry>(&sql);

    if let Some(cat) = &query.category {
        if cat != "All" && !cat.is_empty() { q = q.bind(cat); }
    }
    if let Some(lvl) = &query.level {
        if lvl != "All" && !lvl.is_empty() { q = q.bind(lvl); }
    }
    if has_search { q = q.bind(&search_pattern); }
    if let Some(start) = &query.start_date { if !start.is_empty() { q = q.bind(start); } }
    if let Some(end) = &query.end_date { if !end.is_empty() { q = q.bind(end); } }

    q = q.bind(limit).bind(offset);

    let rows = q.fetch_all(&data.db).await;

    match rows {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch logs"}))
    }
}

#[delete("/api/admin/logs")]
pub async fn clear_logs(data: web::Data<AppState>, session: Session) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)).unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Admin only"}));
    }

    match sqlx::query("DELETE FROM system_logs").execute(&data.db).await {
        Ok(_) => {
            log(&data.db, "WARN", "SYSTEM", "All system logs cleared by admin.").await;
            HttpResponse::Ok().json(serde_json::json!({"ok": true}))
        },
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to clear logs"}))
    }
}

#[get("/api/admin/logs/export")]
pub async fn export_logs(data: web::Data<AppState>, query: web::Query<LogQuery>, session: Session) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)).unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Admin only"}));
    }

    let search_term = query.search.as_deref().unwrap_or("");
    let has_search = !search_term.is_empty();
    let search_pattern = format!("%{}%", search_term);

    let mut conditions = Vec::new();
    if let Some(cat) = &query.category {
        if cat != "All" && !cat.is_empty() { conditions.push("category = ?"); }
    }
    if let Some(lvl) = &query.level {
        if lvl != "All" && !lvl.is_empty() { conditions.push("level = ?"); }
    }
    if has_search { conditions.push("message LIKE ?"); }
    if let Some(start) = &query.start_date { if !start.is_empty() { conditions.push("timestamp >= ?"); } }
    if let Some(end) = &query.end_date { if !end.is_empty() { conditions.push("timestamp <= ?"); } }

    let where_clause = if conditions.is_empty() { String::new() } else { format!("WHERE {}", conditions.join(" AND ")) };
    let sql = format!("SELECT id, timestamp, level, category, message FROM system_logs {} ORDER BY id ASC", where_clause);
    
    let mut q = sqlx::query_as::<_, LogEntry>(&sql);

    if let Some(cat) = &query.category {
        if cat != "All" && !cat.is_empty() { q = q.bind(cat); }
    }
    if let Some(lvl) = &query.level {
        if lvl != "All" && !lvl.is_empty() { q = q.bind(lvl); }
    }
    if has_search { q = q.bind(&search_pattern); }
    if let Some(start) = &query.start_date { if !start.is_empty() { q = q.bind(start); } }
    if let Some(end) = &query.end_date { if !end.is_empty() { q = q.bind(end); } }

    let rows = q.fetch_all(&data.db).await;

    match rows {
        Ok(logs) => {
            let mut wtr = Writer::from_writer(vec![]);
            // Serialize all the log entries into the CSV writer.
            for log_entry in logs {
                if wtr.serialize(log_entry).is_err() {
                    return HttpResponse::InternalServerError().finish();
                }
            }
            
            let csv_data = wtr.into_inner().unwrap_or_default();
            let filename = format!("jeebs_logs_{}.csv", Local::now().format("%Y-%m-%d"));
            HttpResponse::Ok()
                .content_type("text/csv")
                .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
                .body(csv_data)
        },
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch logs for export"}))
    }
}

#[get("/api/admin/log_categories")]
pub async fn get_categories(data: web::Data<AppState>, session: Session) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)).unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Admin only"}));
    }
    
    let rows = sqlx::query("SELECT DISTINCT category FROM system_logs ORDER BY category ASC")
        .fetch_all(&data.db).await;
        
    match rows {
        Ok(rs) => {
            let mut cats: Vec<String> = rs.iter().map(|r| r.get(0)).collect();
            if !cats.contains(&"All".to_string()) {
                cats.insert(0, "All".to_string());
            }
            HttpResponse::Ok().json(cats)
        },
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch categories"}))
    }
}

#[get("/api/my_logs")]
pub async fn get_my_logs(data: web::Data<AppState>, session: Session) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Not logged in"})),
    };

    // Filter logs where the message contains the username
    let pattern = format!("%{}%", username);
    let rows = sqlx::query_as::<_, LogEntry>("SELECT id, timestamp, level, category, message FROM system_logs WHERE message LIKE ? ORDER BY id DESC LIMIT 100")
        .bind(pattern)
        .fetch_all(&data.db).await;

    match rows {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch logs"}))
    }
}

// --- WebSocket Actor ---

struct LogWs;

impl actix::Actor for LogWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let rx = get_broadcaster().subscribe();
        let stream = BroadcastStream::new(rx)
            .filter_map(|res| async { res.ok() }) // Ignore lag errors
            .map(|entry| {
                let json = serde_json::to_string(&entry).unwrap_or_default();
                Ok(ws::Message::Text(json.into()))
            });
        ctx.add_stream(stream);
    }
}

impl actix::StreamHandler<Result<ws::Message, ws::ProtocolError>> for LogWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

#[get("/api/admin/logs/stream")]
pub async fn ws_index(req: actix_web::HttpRequest, stream: web::Payload, session: Session) -> Result<HttpResponse, actix_web::Error> {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)).unwrap_or(false);
    if !is_admin { return Ok(HttpResponse::Unauthorized().finish()); }
    ws::start(LogWs, &req, stream)
}
use crate::state::AppState;
use actix::ActorContext;
use actix::AsyncContext;
use actix_session::Session;
use actix_web::{delete, get, web, HttpResponse, Responder, post};
use actix_web_actors::ws;
use chrono::Local;
use csv::Writer;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

const MAX_LOG_MESSAGE_LEN: usize = 4096;

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

// In-memory recent-log buffer for admin UI
static LOG_BUFFER: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

// Scan job tracker
static SCAN_JOBS: OnceLock<Mutex<HashMap<u64, String>>> = OnceLock::new();

fn get_scan_jobs() -> &'static Mutex<HashMap<u64, String>> {
    SCAN_JOBS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn get_log_buffer() -> &'static Mutex<Vec<String>> {
    LOG_BUFFER.get_or_init(|| Mutex::new(Vec::new()))
}

pub async fn init(db: &SqlitePool) {
    if let Err(err) = sqlx::query(
        "CREATE TABLE IF NOT EXISTS system_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            level TEXT NOT NULL,
            category TEXT NOT NULL,
            message TEXT NOT NULL
        )",
    )
    .execute(db)
    .await
    {
        eprintln!("[WARN] Failed to initialize logging table: {err}");
    }

    // Ensure anomalies table exists
    if let Err(err) = sqlx::query(
        "CREATE TABLE IF NOT EXISTS anomalies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            log_id INTEGER,
            timestamp TEXT NOT NULL,
            level TEXT NOT NULL,
            category TEXT NOT NULL,
            message TEXT NOT NULL,
            reason TEXT,
            metadata TEXT
        )",
    )
    .execute(db)
    .await
    {
        eprintln!("[WARN] Failed to initialize anomalies table: {err}");
    }
}

fn truncate_message(message: &str) -> String {
    if message.chars().count() <= MAX_LOG_MESSAGE_LEN {
        return message.to_string();
    }

    let mut out = String::with_capacity(MAX_LOG_MESSAGE_LEN + 32);
    for ch in message.chars().take(MAX_LOG_MESSAGE_LEN) {
        out.push(ch);
    }
    out.push_str("... [truncated]");
    out
}

async fn insert_log_row(
    db: &SqlitePool,
    timestamp: &str,
    level: &str,
    category: &str,
    message: &str,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO system_logs (timestamp, level, category, message) VALUES (?, ?, ?, ?)",
    )
    .bind(timestamp)
    .bind(level)
    .bind(category)
    .bind(message)
    .execute(db)
    .await?;

    Ok(result.last_insert_rowid())
}

#[derive(Serialize, Clone, sqlx::FromRow)]
pub struct AnomalyEntry {
    pub id: i64,
    pub log_id: Option<i64>,
    pub timestamp: String,
    pub level: String,
    pub category: String,
    pub message: String,
    pub reason: Option<String>,
    pub metadata: Option<String>,
}

async fn insert_anomaly_row(
    db: &SqlitePool,
    log_id: Option<i64>,
    timestamp: &str,
    level: &str,
    category: &str,
    message: &str,
    reason: Option<&str>,
    metadata: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO anomalies (log_id, timestamp, level, category, message, reason, metadata) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(log_id)
    .bind(timestamp)
    .bind(level)
    .bind(category)
    .bind(message)
    .bind(reason)
    .bind(metadata)
    .execute(db)
    .await?;

    Ok(result.last_insert_rowid())
}

#[derive(Serialize, Clone, sqlx::FromRow)]
pub struct ReasoningTrace {
    pub id: i64,
    pub timestamp: String,
    pub username: Option<String>,
    pub prompt: String,
    pub response: String,
    pub metadata: Option<String>,
}

async fn insert_reasoning_trace_row(
    db: &SqlitePool,
    username: Option<&str>,
    prompt: &str,
    response: &str,
    metadata: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO reasoning_traces (timestamp, username, prompt, response, metadata) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(Local::now().to_rfc3339())
    .bind(username)
    .bind(prompt)
    .bind(response)
    .bind(metadata)
    .execute(db)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Public helper to record a reasoning trace (best-effort)
pub async fn record_reasoning_trace(
    db: &SqlitePool,
    username: Option<&str>,
    prompt: &str,
    response: &str,
    metadata: Option<&str>,
) {
    let _ = insert_reasoning_trace_row(db, username, prompt, response, metadata).await;
}

#[get("/api/admin/reasoning_traces")]
pub async fn get_reasoning_traces(data: web::Data<AppState>, session: Session) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Restricted to 1090mb admin account"}));
    }

    let rows = sqlx::query_as::<_, ReasoningTrace>(
        "SELECT id, timestamp, username, prompt, response, metadata FROM reasoning_traces ORDER BY id DESC LIMIT 200",
    )
    .fetch_all(&data.db)
    .await;

    match rows {
        Ok(traces) => HttpResponse::Ok().json(traces),
        Err(_) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to fetch traces"})),
    }
}

pub async fn log(db: &SqlitePool, level: &str, category: &str, message: &str) {
    let timestamp = Local::now().to_rfc3339();
    let message = truncate_message(message);
    let mut insert_result = insert_log_row(db, &timestamp, level, category, &message).await;

    // Recover automatically if older deployments call log() before initialization/migration.
    if let Err(err) = &insert_result {
        if err.to_string().contains("no such table: system_logs") {
            init(db).await;
            insert_result = insert_log_row(db, &timestamp, level, category, &message).await;
        }
    }

    match insert_result {
        Ok(id) => {
            let entry = LogEntry {
                id,
                timestamp: timestamp.clone(),
                level: level.to_string(),
                category: category.to_string(),
                message,
            };
            // push to in-memory buffer (bounded)
            if let Ok(mut buf) = get_log_buffer().lock() {
                buf.push(format!(
                    "{} [{}] {}: {}",
                    entry.timestamp, entry.category, entry.level, entry.message
                ));
                let len = buf.len();
                if len > 1000 {
                    buf.drain(0..(len - 1000));
                }
            }
            let _ = get_broadcaster().send(entry.clone());
            // Improved anomaly detection heuristics
            let msg_lc = entry.message.to_lowercase();
            let keywords = [
                "panic",
                "failed",
                "exception",
                "traceback",
                "segfault",
                "oom",
                "permission denied",
                "forbidden",
                "denied",
                "rate limit",
                "429",
                "503",
                "500",
                "timeout",
                "connection refused",
                "unhandled",
                "error",
            ];

            let mut hit_keyword = false;
            for k in keywords.iter() {
                if msg_lc.contains(k) {
                    hit_keyword = true;
                    break;
                }
            }

            // Detect repeated identical messages in recent buffer
            let mut repeated = false;
            if let Ok(buf) = get_log_buffer().lock() {
                let mut count = 0usize;
                for item in buf.iter().rev().take(200) {
                    if item.contains(&entry.message) {
                        count += 1;
                    }
                    if count >= 5 {
                        repeated = true;
                        break;
                    }
                }
            }

            // Treat WARN/ERROR as anomaly; also treat INFO containing keywords/repeats
            let is_anomaly = matches!(level, "ERROR" | "WARN") || hit_keyword || repeated;

            if is_anomaly {
                let reason = if matches!(level, "ERROR" | "WARN") {
                    "level"
                } else if repeated {
                    "repeated"
                } else if hit_keyword {
                    "keyword"
                } else {
                    "auto-detected"
                };

                let _ = insert_anomaly_row(
                    db,
                    Some(id),
                    &timestamp,
                    level,
                    category,
                    &entry.message,
                    Some(reason),
                    None,
                )
                .await;
            }
        }
        Err(err) => {
            eprintln!("[WARN] Failed to persist system log entry: {err}");
        }
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
                log(
                    db,
                    "INFO",
                    "SYSTEM",
                    &format!(
                        "Log cleanup task finished. Removed {rows_affected} entries older than 30 days."
                    ),
                )
                .await;
            }
        }
        Err(e) => eprintln!("[ERROR] Failed to execute log cleanup task: {e}"),
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
pub async fn get_logs(
    data: web::Data<AppState>,
    query: web::Query<LogQuery>,
    session: Session,
) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Restricted to 1090mb admin account"}));
    }

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).max(1).min(1000);
    let offset = (page - 1) * limit;

    let search_term = query.search.as_deref().unwrap_or("");
    let has_search = !search_term.is_empty();
    let search_pattern = format!("%{search_term}%");

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
        if !start.is_empty() {
            conditions.push("timestamp >= ?");
        }
    }
    if let Some(end) = &query.end_date {
        if !end.is_empty() {
            conditions.push("timestamp <= ?");
        }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT id, timestamp, level, category, message FROM system_logs {where_clause} ORDER BY id DESC LIMIT ? OFFSET ?"
    );
    let mut q = sqlx::query_as::<_, LogEntry>(&sql);

    if let Some(cat) = &query.category {
        if cat != "All" && !cat.is_empty() {
            q = q.bind(cat);
        }
    }
    if let Some(lvl) = &query.level {
        if lvl != "All" && !lvl.is_empty() {
            q = q.bind(lvl);
        }
    }
    if has_search {
        q = q.bind(&search_pattern);
    }
    if let Some(start) = &query.start_date {
        if !start.is_empty() {
            q = q.bind(start);
        }
    }
    if let Some(end) = &query.end_date {
        if !end.is_empty() {
            q = q.bind(end);
        }
    }

    q = q.bind(limit).bind(offset);

    let rows = q.fetch_all(&data.db).await;

    match rows {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(_) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to fetch logs"})),
    }
}

#[delete("/api/admin/logs")]
pub async fn clear_logs(data: web::Data<AppState>, session: Session) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Restricted to 1090mb admin account"}));
    }

    match sqlx::query("DELETE FROM system_logs")
        .execute(&data.db)
        .await
    {
        Ok(_) => {
            log(
                &data.db,
                "WARN",
                "SYSTEM",
                "All system logs cleared by admin.",
            )
            .await;
            HttpResponse::Ok().json(serde_json::json!({"ok": true}))
        }
        Err(_) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to clear logs"})),
    }
}

#[get("/api/admin/logs/export")]
pub async fn export_logs(
    data: web::Data<AppState>,
    query: web::Query<LogQuery>,
    session: Session,
) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Restricted to 1090mb admin account"}));
    }

    let search_term = query.search.as_deref().unwrap_or("");
    let has_search = !search_term.is_empty();
    let search_pattern = format!("%{search_term}%");

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
        if !start.is_empty() {
            conditions.push("timestamp >= ?");
        }
    }
    if let Some(end) = &query.end_date {
        if !end.is_empty() {
            conditions.push("timestamp <= ?");
        }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    let sql = format!(
        "SELECT id, timestamp, level, category, message FROM system_logs {where_clause} ORDER BY id ASC"
    );

    let mut q = sqlx::query_as::<_, LogEntry>(&sql);

    if let Some(cat) = &query.category {
        if cat != "All" && !cat.is_empty() {
            q = q.bind(cat);
        }
    }
    if let Some(lvl) = &query.level {
        if lvl != "All" && !lvl.is_empty() {
            q = q.bind(lvl);
        }
    }
    if has_search {
        q = q.bind(&search_pattern);
    }
    if let Some(start) = &query.start_date {
        if !start.is_empty() {
            q = q.bind(start);
        }
    }
    if let Some(end) = &query.end_date {
        if !end.is_empty() {
            q = q.bind(end);
        }
    }

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
                .append_header((
                    "Content-Disposition",
                    format!("attachment; filename=\"{filename}\""),
                ))
                .body(csv_data)
        }
        Err(_) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to fetch logs for export"})),
    }
}

#[get("/api/admin/log_categories")]
pub async fn get_categories(data: web::Data<AppState>, session: Session) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Restricted to 1090mb admin account"}));
    }

    let rows = sqlx::query("SELECT DISTINCT category FROM system_logs ORDER BY category ASC")
        .fetch_all(&data.db)
        .await;

    match rows {
        Ok(rs) => {
            let mut cats: Vec<String> = rs.iter().map(|r| r.get(0)).collect();
            if !cats.contains(&"All".to_string()) {
                cats.insert(0, "All".to_string());
            }
            HttpResponse::Ok().json(cats)
        }
        Err(_) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to fetch categories"})),
    }
}

#[get("/api/admin/anomalies")]
pub async fn get_anomalies(data: web::Data<AppState>, session: Session) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Restricted to 1090mb admin account"}));
    }

    let rows = sqlx::query_as::<_, AnomalyEntry>(
        "SELECT id, log_id, timestamp, level, category, message, reason, metadata FROM anomalies ORDER BY id DESC LIMIT 200",
    )
    .fetch_all(&data.db)
    .await;

    match rows {
        Ok(anoms) => HttpResponse::Ok().json(anoms),
        Err(_) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to fetch anomalies"})),
    }
}

#[get("/api/admin/anomalies/scan/jobs")]
pub async fn list_scan_jobs(data: web::Data<AppState>, session: Session) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Restricted to 1090mb admin account"}));
    }
    let jobs = get_scan_jobs().lock().unwrap();
    let mut out: Vec<serde_json::Value> = Vec::new();
    for (k, v) in jobs.iter() {
        out.push(serde_json::json!({"job_id": k, "status": v}));
    }
    HttpResponse::Ok().json(out)
}

#[get("/api/admin/anomalies/scan/status/{job_id}")]
pub async fn scan_job_status(data: web::Data<AppState>, session: Session, path: web::Path<u64>) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Restricted to 1090mb admin account"}));
    }
    let job_id = path.into_inner();
    let jobs = get_scan_jobs().lock().unwrap();
    if let Some(status) = jobs.get(&job_id) {
        HttpResponse::Ok().json(serde_json::json!({"job_id": job_id, "status": status}))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({"error": "Job not found"}))
    }
}

#[post("/api/admin/anomalies/scan")]
pub async fn scan_legacy_logs(
    data: web::Data<AppState>,
    session: Session,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Restricted to 1090mb admin account"}));
    }
    // Optional parameters: async=true, limit=N, days=M
    let is_async = query.get("async").map(|s| s == "1" || s == "true").unwrap_or(false);
    let limit: i64 = query
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(2000);
    let days: Option<i64> = query.get("days").and_then(|s| s.parse().ok());

    if is_async {
        // spawn background job and return job id
        use std::time::{SystemTime, UNIX_EPOCH};
        let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let job_id = start as u64;
        {
            let mut jobs = get_scan_jobs().lock().unwrap();
            jobs.insert(job_id, "running".to_string());
        }

        let db = data.db.clone();
        let q = query.into_inner();
        tokio::spawn(async move {
            let mut flagged = 0u32;
            // build query depending on days
            let rows = if let Some(d) = days {
                let threshold = (Local::now() - chrono::Duration::days(d)).to_rfc3339();
                sqlx::query_as::<_, LogEntry>(
                    "SELECT id, timestamp, level, category, message FROM system_logs WHERE timestamp >= ? ORDER BY id DESC LIMIT ?",
                )
                .bind(threshold)
                .bind(limit)
                .fetch_all(&db)
                .await
            } else {
                sqlx::query_as::<_, LogEntry>(
                    "SELECT id, timestamp, level, category, message FROM system_logs ORDER BY id DESC LIMIT ?",
                )
                .bind(limit)
                .fetch_all(&db)
                .await
            };

            if let Ok(logs) = rows {
                for log in logs.into_iter() {
                    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM anomalies WHERE log_id = ?")
                        .bind(log.id)
                        .fetch_one(&db)
                        .await
                        .unwrap_or(0);
                    if exists > 0 { continue; }
                    let msg_lc = log.message.to_lowercase();
                    let keywords = ["panic","failed","exception","traceback","segfault","oom","permission denied","forbidden","denied","rate limit","429","503","500","timeout","connection refused","unhandled","error"];
                    let mut hit = false;
                    for k in keywords.iter() { if msg_lc.contains(k) { hit = true; break; } }
                    if hit {
                        let _ = insert_anomaly_row(&db, Some(log.id), &log.timestamp, &log.level, &log.category, &log.message, Some("retro-keyword"), None).await;
                        flagged += 1;
                    }
                }
            }

            let mut jobs = get_scan_jobs().lock().unwrap();
            jobs.insert(job_id, format!("done:{}", flagged));
        });

        return HttpResponse::Ok().json(serde_json::json!({"ok": true, "job_id": start}));
    }

    // synchronous path (short run)
    let rows = if let Some(d) = days {
        let threshold = (Local::now() - chrono::Duration::days(d)).to_rfc3339();
        sqlx::query_as::<_, LogEntry>(
            "SELECT id, timestamp, level, category, message FROM system_logs WHERE timestamp >= ? ORDER BY id DESC LIMIT ?",
        )
        .bind(threshold)
        .bind(limit)
        .fetch_all(&data.db)
        .await
    } else {
        sqlx::query_as::<_, LogEntry>(
            "SELECT id, timestamp, level, category, message FROM system_logs ORDER BY id DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&data.db)
        .await
    };

    match rows {
        Ok(logs) => {
            let mut flagged = 0u32;
            for log in logs.into_iter() {
                let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM anomalies WHERE log_id = ?")
                    .bind(log.id)
                    .fetch_one(&data.db)
                    .await
                    .unwrap_or(0);
                if exists > 0 { continue; }
                let msg_lc = log.message.to_lowercase();
                let keywords = ["panic","failed","exception","traceback","segfault","oom","permission denied","forbidden","denied","rate limit","429","503","500","timeout","connection refused","unhandled","error"];
                let mut hit = false;
                for k in keywords.iter() { if msg_lc.contains(k) { hit = true; break; } }
                if hit {
                    let _ = insert_anomaly_row(&data.db, Some(log.id), &log.timestamp, &log.level, &log.category, &log.message, Some("retro-keyword"), None).await;
                    flagged += 1;
                }
            }
            HttpResponse::Ok().json(serde_json::json!({"ok": true, "flagged": flagged}))
        }
        Err(_) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to fetch logs for scan"})),
    }
}

#[get("/api/my_logs")]
pub async fn get_my_logs(data: web::Data<AppState>, session: Session) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Not logged in"}));
        }
    };

    // Do not reveal admin or root-admin activity on profile pages.
    // If the logged-in user is an admin (or the root admin `1090mb`),
    // return an empty list to avoid exposing recent admin actions.
    let is_admin = session.get::<bool>("is_admin").ok().flatten().unwrap_or(false);
    if is_admin || username == crate::auth::ROOT_ADMIN_USERNAME {
        let empty: Vec<LogEntry> = Vec::new();
        return HttpResponse::Ok().json(empty);
    }

    // Filter logs where the message contains the username
    let pattern = format!("%{username}%");
    let rows = sqlx::query_as::<_, LogEntry>("SELECT id, timestamp, level, category, message FROM system_logs WHERE message LIKE ? ORDER BY id DESC LIMIT 100")
        .bind(pattern)
        .fetch_all(&data.db).await;

    match rows {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(_) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to fetch logs"})),
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
pub async fn ws_index(
    req: actix_web::HttpRequest,
    stream: web::Payload,
    session: Session,
) -> Result<HttpResponse, actix_web::Error> {
    if !crate::auth::is_root_admin_session(&session) {
        return Ok(HttpResponse::Forbidden().finish());
    }
    ws::start(LogWs, &req, stream)
}

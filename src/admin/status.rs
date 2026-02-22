use crate::state::AppState;
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Local};
use serde_json::json;
use sqlx::Row;
use sysinfo::System;

/// Original admin-only endpoint (kept for backwards compat)
#[get("/api/admin/status")]
pub async fn get_system_status(data: web::Data<AppState>, session: Session) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let mut sys = data.sys.lock().unwrap();
    sys.refresh_memory();

    let used_memory = sys.used_memory();
    let total_memory = sys.total_memory();
    let available_memory = sys.available_memory();
    let uptime = System::uptime();
    let used_percent = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };

    HttpResponse::Ok().json(json!({
        "used_memory": used_memory,
        "total_memory": total_memory,
        "available_memory": available_memory,
        "used_memory_mb": bytes_to_mb(used_memory),
        "total_memory_mb": bytes_to_mb(total_memory),
        "available_memory_mb": bytes_to_mb(available_memory),
        "used_percent": (used_percent * 10.0).round() / 10.0,
        "uptime": uptime,
        "uptime_formatted": format_uptime(uptime)
    }))
}

/// Public health check — no auth required
pub async fn health_check(data: web::Data<AppState>) -> HttpResponse {
    // Quick DB ping
    let db_ok = sqlx::query("SELECT 1").fetch_one(&data.db).await.is_ok();
    let status = if db_ok { "ok" } else { "degraded" };
    let code = if db_ok { 200 } else { 503 };

    let body = json!({
        "status": status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "dependencies": {
            "database": if db_ok { "ok" } else { "error" }
        }
    });

    if code == 200 {
        HttpResponse::Ok().json(body)
    } else {
        HttpResponse::ServiceUnavailable().json(body)
    }
}

/// Comprehensive server stats — requires admin session
#[get("/api/server/stats")]
pub async fn get_server_stats(data: web::Data<AppState>, session: Session) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    // ── System metrics ──
    let (used_memory, total_memory, available_memory, uptime, cpu_count) = {
        let mut sys = data.sys.lock().unwrap();
        sys.refresh_memory();
        (
            sys.used_memory(),
            sys.total_memory(),
            sys.available_memory(),
            System::uptime(),
            sys.cpus().len(),
        )
    };

    let mem_percent = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };

    // ── Database counts ──
    let brain_nodes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM brain_nodes")
        .fetch_one(&data.db).await.unwrap_or(0);

    let knowledge_triples: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_triples")
        .fetch_one(&data.db).await.unwrap_or(0);

    let store_entries: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM jeebs_store")
        .fetch_one(&data.db).await.unwrap_or(0);

    let system_logs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM system_logs")
        .fetch_one(&data.db).await.unwrap_or(0);

    let anomalies: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM anomalies")
        .fetch_one(&data.db).await.unwrap_or(0);

    // ── Active sessions ──
    let cutoff = Local::now() - chrono::Duration::minutes(30);
    let rows = sqlx::query(
        "SELECT username, ip, user_agent, last_seen FROM user_sessions ORDER BY last_seen DESC",
    )
    .fetch_all(&data.db)
    .await
    .unwrap_or_default();

    let mut sessions = Vec::new();
    let mut stale = Vec::new();
    for row in &rows {
        let username: String = row.get(0);
        let last_seen: String = row.get(3);
        let active = DateTime::parse_from_rfc3339(&last_seen)
            .ok()
            .map(|dt| dt.with_timezone(&Local) >= cutoff)
            .unwrap_or(false);
        if active {
            sessions.push(json!({
                "username": username,
                "ip": row.get::<String, _>(1),
                "user_agent": row.get::<String, _>(2),
                "last_seen": last_seen
            }));
        } else {
            stale.push(username);
        }
    }
    // clean stale
    for u in stale {
        let _ = sqlx::query("DELETE FROM user_sessions WHERE username = ?")
            .bind(u).execute(&data.db).await;
    }

    // ── Feature toggles ──
    let internet_enabled = *data.internet_enabled.read().unwrap();

    // ── Training status ──
    let training_enabled: bool = sqlx::query_scalar::<_, String>(
        "SELECT value FROM jeebs_store WHERE key = 'training:mode'"
    )
    .fetch_optional(&data.db)
    .await
    .unwrap_or(None)
    .map(|v| v != "disabled")
    .unwrap_or(true);

    HttpResponse::Ok().json(json!({
        "system": {
            "uptime": uptime,
            "uptime_formatted": format_uptime(uptime),
            "cpu_count": cpu_count,
            "memory": {
                "used_mb": bytes_to_mb(used_memory),
                "total_mb": bytes_to_mb(total_memory),
                "available_mb": bytes_to_mb(available_memory),
                "used_percent": (mem_percent * 10.0).round() / 10.0
            }
        },
        "database": {
            "brain_nodes": brain_nodes,
            "knowledge_triples": knowledge_triples,
            "store_entries": store_entries,
            "system_logs": system_logs,
            "anomalies": anomalies
        },
        "features": {
            "internet_enabled": internet_enabled,
            "training_enabled": training_enabled
        },
        "sessions": {
            "active_count": sessions.len(),
            "list": sessions
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

fn bytes_to_mb(bytes: u64) -> f64 {
    let mb = bytes as f64 / 1024.0 / 1024.0;
    (mb * 100.0).round() / 100.0
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    format!("{days}d {hours}h {minutes}m")
}

/// Admin Dashboard: Comprehensive metrics, monitoring, and control panel for JeebsAI
/// Provides visibility into brain state, proposals, chat metrics, and system health

use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Serialize};
use sqlx::SqlitePool;
use serde_json::json;
use crate::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct BrainMetrics {
    pub total_nodes: u64,
    pub total_links: u64,
    pub quantum_entropy: f32,
    pub comprehension: f32,
    pub personality_attitude: String,
    pub nodes_added_today: u64,
    pub links_created_today: u64,
    pub avg_node_importance: f32,
    pub avg_node_usage_count: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProposalMetrics {
    pub pending_proposals: u64,
    pub approved_proposals: u64,
    pub denied_proposals: u64,
    pub implemented_proposals: u64,
    pub pending_list: Vec<PendingProposal>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PendingProposal {
    pub id: String,
    pub proposal_type: String,
    pub title: String,
    pub proposer_id: String,
    pub votes_for: i64,
    pub votes_against: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMetrics {
    pub total_messages_today: u64,
    pub unique_users_today: u64,
    pub avg_response_confidence: f32,
    pub greeting_count: u64,
    pub inference_errors_today: u64,
    pub avg_query_length: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub total_chat_messages: u64,
    pub total_brain_facts: u64,
    pub internet_enabled: bool,
    pub training_enabled: bool,
    pub memory_usage_mb: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminDashboard {
    pub brain_metrics: BrainMetrics,
    pub proposal_metrics: ProposalMetrics,
    pub chat_metrics: ChatMetrics,
    pub system_metrics: SystemMetrics,
    pub timestamp: String,
}

/// Get comprehensive admin dashboard with all metrics
#[get("/api/admin/dashboard")]
pub async fn get_admin_dashboard(
    data: web::Data<AppState>,
    session: actix_session::Session,
) -> impl Responder {
    // Verify admin
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)) == Some(true);
    if !is_admin {
        return HttpResponse::Forbidden().json(json!({
            "error": "Admin access required"
        }));
    }

    // Collect brain metrics
    let chdsc = data.chdsc.read().unwrap();
    let brain_metrics = BrainMetrics {
        total_nodes: chdsc.nodes.len() as u64,
        total_links: chdsc.links.len() as u64,
        quantum_entropy: chdsc.quantum_entropy as f32,
        comprehension: chdsc.comprehension as f32,
        personality_attitude: chdsc.attitude.clone(),
        nodes_added_today: 0,  // Would need to query DB with timestamp filter
        links_created_today: 0,
        avg_node_importance: calculate_avg_importance(&chdsc),
        avg_node_usage_count: calculate_avg_usage(&chdsc),
    };
    drop(chdsc);

    // Collect proposal metrics
    let proposal_metrics = fetch_proposal_metrics(&data.db).await.unwrap_or_default();

    // Collect chat metrics
    let chat_metrics = fetch_chat_metrics(&data.db).await.unwrap_or_default();

    // Collect system metrics
    let system_metrics = SystemMetrics {
        uptime_seconds: 0,  // Would need to track server start time
        total_chat_messages: count_total_messages(&data.db).await.unwrap_or(0),
        total_brain_facts: 0,
        internet_enabled: *data.internet_enabled.read().unwrap(),
        training_enabled: get_training_enabled(&data.db).await.unwrap_or(false),
        memory_usage_mb: 0,  // Would need system_memory_usage crate
    };

    let dashboard = AdminDashboard {
        brain_metrics,
        proposal_metrics,
        chat_metrics,
        system_metrics,
        timestamp: chrono::Local::now().to_rfc3339(),
    };

    HttpResponse::Ok().json(dashboard)
}

/// Get detailed brain state (nodes, links, entropy timeline)
#[get("/api/admin/brain/detailed")]
pub async fn get_brain_details(
    data: web::Data<AppState>,
    session: actix_session::Session,
) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)) == Some(true);
    if !is_admin {
        return HttpResponse::Forbidden().json(json!({"error": "Admin access required"}));
    }

    let chdsc = data.chdsc.read().unwrap();

    // Get sample of top nodes by usage
    let mut node_list: Vec<_> = chdsc.nodes.values().collect();
    node_list.sort_by(|a, b| {
        let a_usage = a.meta.get("usage_count").and_then(|s| s.parse::<i32>().ok()).unwrap_or(0) as i32;
        let b_usage = b.meta.get("usage_count").and_then(|s| s.parse::<i32>().ok()).unwrap_or(0) as i32;
        b_usage.cmp(&a_usage)
    });

    let top_nodes: Vec<_> = node_list
        .iter()
        .take(20)
        .map(|node| {
            json!({
                "id": node.id,
                "fact": node.meta.get("fact").cloned().unwrap_or_default(),
                "importance": node.meta.get("importance").cloned().unwrap_or_else(|| "0.0".to_string()),
                "usage_count": node.meta.get("usage_count").cloned().unwrap_or_else(|| "0".to_string()),
                "source": node.meta.get("source").cloned().unwrap_or_default(),
                "tags": node.tags.clone(),
            })
        })
        .collect();

    HttpResponse::Ok().json(json!({
        "total_nodes": chdsc.nodes.len(),
        "total_links": chdsc.links.len(),
        "entropy": chdsc.quantum_entropy,
        "comprehension": chdsc.comprehension,
        "attitude": chdsc.attitude,
        "top_nodes_by_usage": top_nodes,
    }))
}

/// Get all proposals with filtering and sorting
#[get("/api/admin/proposals/all")]
pub async fn get_all_proposals(
    data: web::Data<AppState>,
    web::Query(params): web::Query<std::collections::HashMap<String, String>>,
    session: actix_session::Session,
) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)) == Some(true);
    if !is_admin {
        return HttpResponse::Forbidden().json(json!({"error": "Admin access required"}));
    }

    let status_filter = params.get("status").map(|s| s.as_str()).unwrap_or("pending");

    let rows = sqlx::query_as::<_, (String, String, String, String, i64, i64, String)>(
        "SELECT id, proposal_type, title, proposer_id, votes_for, votes_against, created_at
         FROM cdhsc_proposals WHERE status = ? ORDER BY votes_for DESC, created_at DESC"
    )
    .bind(status_filter)
    .fetch_all(&data.db)
    .await
    .unwrap_or_default();

    let proposals: Vec<_> = rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.0,
                "type": r.1,
                "title": r.2,
                "proposer": r.3,
                "votes_for": r.4,
                "votes_against": r.5,
                "net_votes": r.4 - r.5,
                "created_at": r.6,
            })
        })
        .collect();

    HttpResponse::Ok().json(json!({
        "status": status_filter,
        "count": proposals.len(),
        "proposals": proposals,
    }))
}

/// Get chat activity statistics
#[get("/api/admin/chat/statistics")]
pub async fn get_chat_statistics(
    data: web::Data<AppState>,
    web::Query(params): web::Query<std::collections::HashMap<String, String>>,
    session: actix_session::Session,
) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)) == Some(true);
    if !is_admin {
        return HttpResponse::Forbidden().json(json!({"error": "Admin access required"}));
    }

    let days = params.get("days").and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);

    // Get messages by role
    let messages_by_role = sqlx::query_as::<_, (String, i64)>(
        "SELECT role, COUNT(*) as count FROM chat_history
         WHERE timestamp > datetime('now', '-' || ? || ' days')
         GROUP BY role"
    )
    .bind(days)
    .fetch_all(&data.db)
    .await
    .unwrap_or_default();

    // Get unique users
    let unique_users = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT username) FROM chat_history
         WHERE timestamp > datetime('now', '-' || ? || ' days') AND username IS NOT NULL"
    )
    .bind(days)
    .fetch_one(&data.db)
    .await
    .unwrap_or(0);

    HttpResponse::Ok().json(json!({
        "days": days,
        "unique_users": unique_users,
        "messages_by_role": messages_by_role,
        "timestamp": chrono::Local::now().to_rfc3339(),
    }))
}

/// Get recent inference outcomes (for learning monitoring)
#[get("/api/admin/inference/recent")]
pub async fn get_recent_inferences(
    data: web::Data<AppState>,
    web::Query(params): web::Query<std::collections::HashMap<String, String>>,
    session: actix_session::Session,
) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)) == Some(true);
    if !is_admin {
        return HttpResponse::Forbidden().json(json!({"error": "Admin access required"}));
    }

    let limit = params.get("limit").and_then(|s| s.parse::<i32>().ok()).unwrap_or(50);

    let rows = sqlx::query_as::<_, (String, Vec<u8>)>(
        "SELECT key, value FROM jeebs_store WHERE key LIKE 'inference_outcome:%'
         ORDER BY key DESC LIMIT ?"
    )
    .bind(limit)
    .fetch_all(&data.db)
    .await
    .unwrap_or_default();

    let outcomes: Vec<_> = rows
        .iter()
        .filter_map(|(_, value)| {
            serde_json::from_slice::<serde_json::Value>(value).ok()
        })
        .collect();

    HttpResponse::Ok().json(json!({
        "count": outcomes.len(),
        "recent_inferences": outcomes,
    }))
}

/// System control: Enable/disable internet
#[post("/api/admin/system/internet/{enabled}")]
pub async fn set_internet(
    enabled: web::Path<bool>,
    data: web::Data<AppState>,
    session: actix_session::Session,
) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)) == Some(true);
    if !is_admin {
        return HttpResponse::Forbidden().json(json!({"error": "Admin access required"}));
    }

    let enabled = enabled.into_inner();
    *data.internet_enabled.write().unwrap() = enabled;

    HttpResponse::Ok().json(json!({
        "internet_enabled": enabled,
        "message": if enabled { "Internet enabled" } else { "Internet disabled" }
    }))
}

/// System control: Enable/disable training
#[post("/api/admin/system/training/{enabled}")]
pub async fn set_training(
    enabled: web::Path<bool>,
    data: web::Data<AppState>,
    session: actix_session::Session,
) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)) == Some(true);
    if !is_admin {
        return HttpResponse::Forbidden().json(json!({"error": "Admin access required"}));
    }

    let enabled = enabled.into_inner();

    // Update in database
    let _ = sqlx::query("INSERT INTO jeebs_store (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
        .bind("system:training_enabled")
        .bind(if enabled { b"true".to_vec() } else { b"false".to_vec() })
        .execute(&data.db)
        .await;

    HttpResponse::Ok().json(json!({
        "training_enabled": enabled,
        "message": if enabled { "Training enabled" } else { "Training disabled" }
    }))
}

// Helper functions

fn calculate_avg_importance(cdhsc: &crate::brain::coded_holographic_data_storage_container::CodedHolographicDataStorageContainer) -> f32 {
    if cdhsc.nodes.is_empty() {
        return 0.0;
    }
    let sum: f32 = cdhsc.nodes
        .values()
        .map(|n| n.meta.get("importance").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.7))
        .sum();
    sum / cdhsc.nodes.len() as f32
}

fn calculate_avg_usage(cdhsc: &crate::brain::coded_holographic_data_storage_container::CodedHolographicDataStorageContainer) -> f32 {
    if cdhsc.nodes.is_empty() {
        return 0.0;
    }
    let sum: f32 = cdhsc.nodes
        .values()
        .map(|n| n.meta.get("usage_count").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0))
        .sum();
    sum / cdhsc.nodes.len() as f32
}

async fn fetch_proposal_metrics(db: &SqlitePool) -> Result<ProposalMetrics, String> {
    let pending = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM cdhsc_proposals WHERE status = 'pending'"
    ).fetch_one(db).await.unwrap_or(0);

    let approved = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM cdhsc_proposals WHERE status = 'approved'"
    ).fetch_one(db).await.unwrap_or(0);

    let denied = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM cdhsc_proposals WHERE status = 'denied'"
    ).fetch_one(db).await.unwrap_or(0);

    let implemented = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM cdhsc_proposals WHERE status = 'implemented'"
    ).fetch_one(db).await.unwrap_or(0);

    let pending_list = sqlx::query_as::<_, (String, String, String, String, i64, i64, String)>(
        "SELECT id, proposal_type, title, proposer_id, votes_for, votes_against, created_at
         FROM cdhsc_proposals WHERE status = 'pending'
         ORDER BY votes_for DESC LIMIT 10"
    ).fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(|(id, ptype, title, proposer, for_votes, against_votes, created)| {
            PendingProposal {
                id,
                proposal_type: ptype,
                title,
                proposer_id: proposer,
                votes_for: for_votes,
                votes_against: against_votes,
                created_at: created,
            }
        })
        .collect();

    Ok(ProposalMetrics {
        pending_proposals: pending as u64,
        approved_proposals: approved as u64,
        denied_proposals: denied as u64,
        implemented_proposals: implemented as u64,
        pending_list,
    })
}

async fn fetch_chat_metrics(db: &SqlitePool) -> Result<ChatMetrics, String> {
    let total_today = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM chat_history WHERE timestamp > datetime('now', '-1 day')"
    ).fetch_one(db).await.unwrap_or(0);

    let unique_users = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT username) FROM chat_history
         WHERE timestamp > datetime('now', '-1 day') AND username IS NOT NULL"
    ).fetch_one(db).await.unwrap_or(0);

    let greeting_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM chat_history WHERE message IN ('hello', 'hi', 'hey', 'greetings')
         AND timestamp > datetime('now', '-1 day')"
    ).fetch_one(db).await.unwrap_or(0);

    Ok(ChatMetrics {
        total_messages_today: total_today as u64,
        unique_users_today: unique_users as u64,
        avg_response_confidence: 0.75,  // Would need to aggregate from inference outcomes
        greeting_count: greeting_count as u64,
        inference_errors_today: 0,  // Would need to track from logs
        avg_query_length: 0.0,
    })
}

async fn count_total_messages(db: &SqlitePool) -> Result<u64, String> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM chat_history"
    ).fetch_one(db).await.unwrap_or(0);
    Ok(count as u64)
}

async fn get_training_enabled(_db: &SqlitePool) -> Result<bool, String> {
    // For now, default to true. In production, would fetch from DB
    Ok(true)
}

impl Default for ProposalMetrics {
    fn default() -> Self {
        ProposalMetrics {
            pending_proposals: 0,
            approved_proposals: 0,
            denied_proposals: 0,
            implemented_proposals: 0,
            pending_list: vec![],
        }
    }
}

impl Default for ChatMetrics {
    fn default() -> Self {
        ChatMetrics {
            total_messages_today: 0,
            unique_users_today: 0,
            avg_response_confidence: 0.0,
            greeting_count: 0,
            inference_errors_today: 0,
            avg_query_length: 0.0,
        }
    }
}

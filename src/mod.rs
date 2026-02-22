use actix_web::{get, post, web, HttpResponse, Responder};
use actix_session::Session;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use std::fs;
use std::path::Path;
use crate::state::AppState;
use crate::utils::{encode_all, decode_all};

#[derive(Serialize, Deserialize, Clone)]
pub struct FileChange {
    pub path: String,
    pub new_content: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProposedUpdate {
    pub id: String,
    pub title: String,
    pub description: String,
    pub changes: Vec<FileChange>,
    pub status: String, // "pending", "applied", "denied"
    pub created_at: String,
    pub backup: Option<Vec<FileChange>>,
    #[serde(default)]
    pub feeling: Option<String>,
    #[serde(default)]
    pub feeling_confidence: f32,
}

#[get("/api/admin/evolution/updates")]
pub async fn list_updates(data: web::Data<AppState>, session: Session) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").ok().flatten().unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }

    let rows = sqlx::query("SELECT value FROM jeebs_store WHERE key LIKE 'evolution:update:%'")
        .fetch_all(&data.db).await.unwrap_or_default();

    let mut updates = Vec::new();
    for row in rows {
        let val: Vec<u8> = row.get(0);
        if let Ok(bytes) = decode_all(&val) {
            if let Ok(update) = serde_json::from_slice::<ProposedUpdate>(&bytes) {
                updates.push(update);
            }
        }
    }
    // Sort by date desc
    updates.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    HttpResponse::Ok().json(updates)
}

#[post("/api/admin/evolution/apply/{id}")]
pub async fn apply_update(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: Session,
) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").ok().flatten().unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }

    let id = path.into_inner();
    let key = format!("evolution:update:{}", id);

    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(&key).fetch_optional(&data.db).await {
        let val: Vec<u8> = row.get(0);
        if let Ok(bytes) = decode_all(&val) {
            if let Ok(mut update) = serde_json::from_slice::<ProposedUpdate>(&bytes) {
                if update.status == "denied" {
                    return HttpResponse::BadRequest().json(json!({"error": "Cannot apply a denied update"}));
                }
                if update.status != "pending" {
                    return HttpResponse::BadRequest().json(json!({"error": "Update already processed"}));
                }

                // Create Backup
                let mut backups = Vec::new();
                for change in &update.changes {
                    let path = Path::new(&change.path);
                    if path.exists() {
                        if let Ok(content) = fs::read_to_string(path) {
                            backups.push(FileChange { path: change.path.clone(), new_content: content });
                        }
                    }
                }
                update.backup = Some(backups);

                // Apply changes to file system
                for change in &update.changes {
                    // Security check: prevent directory traversal
                    if change.path.contains("..") || change.path.starts_with("/") {
                        return HttpResponse::BadRequest().json(json!({"error": "Invalid file path detected in update"}));
                    }
                    
                    let path = Path::new(&change.path);
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent).ok();
                    }
                    if let Err(e) = fs::write(path, &change.new_content) {
                        return HttpResponse::InternalServerError().json(json!({"error": format!("Failed to write file: {}", e)}));
                    }
                }

                update.status = "applied".to_string();
                if let Ok(json_bytes) = serde_json::to_vec(&update) {
                    if let Ok(new_val) = encode_all(&json_bytes, 1) {
                        if let Err(e) = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)").bind(&key).bind(new_val).execute(&data.db).await {
                             return HttpResponse::InternalServerError().json(json!({"error": format!("Database error: {}", e)}));
                        }
                    }
                }

                return HttpResponse::Ok().json(json!({"message": "Update applied successfully. Please rebuild/restart Jeebs."}));
            }
        }
    }
    HttpResponse::NotFound().json(json!({"error": "Update not found"}))
}

#[post("/api/admin/evolution/deny/{id}")]
pub async fn deny_update(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: Session,
) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").ok().flatten().unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }

    let id = path.into_inner();
    let key = format!("evolution:update:{}", id);

    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(&key).fetch_optional(&data.db).await {
        let val: Vec<u8> = row.get(0);
        if let Ok(bytes) = decode_all(&val) {
            if let Ok(mut update) = serde_json::from_slice::<ProposedUpdate>(&bytes) {
                if update.status == "applied" {
                    return HttpResponse::BadRequest().json(json!({"error": "Cannot deny an applied update"}));
                }
                update.status = "denied".to_string();
                if let Ok(json_bytes) = serde_json::to_vec(&update) {
                    if let Ok(new_val) = encode_all(&json_bytes, 1) {
                        if let Err(e) = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)").bind(&key).bind(new_val).execute(&data.db).await {
                            return HttpResponse::InternalServerError().json(json!({"error": format!("Database error: {}", e)}));
                        }
                    }
                }
                return HttpResponse::Ok().json(json!({"message": "Update denied"}));
            }
        }
    }
    HttpResponse::NotFound().json(json!({"error": "Update not found"}))
}

#[post("/api/admin/evolution/rollback/{id}")]
pub async fn rollback_update(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: Session,
) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").ok().flatten().unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }

    let id = path.into_inner();
    let key = format!("evolution:update:{}", id);

    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(&key).fetch_optional(&data.db).await {
        let val: Vec<u8> = row.get(0);
        if let Ok(bytes) = decode_all(&val) {
            if let Ok(mut update) = serde_json::from_slice::<ProposedUpdate>(&bytes) {
                if update.status == "applied" {
                    if let Some(backups) = &update.backup {
                        for file in backups {
                            let _ = fs::write(&file.path, &file.new_content);
                        }
                        update.status = "rolled_back".to_string();
                        if let Ok(json_bytes) = serde_json::to_vec(&update) {
                            if let Ok(new_val) = encode_all(&json_bytes, 1) {
                                if let Err(e) = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)").bind(&key).bind(new_val).execute(&data.db).await {
                                    return HttpResponse::InternalServerError().json(json!({"error": format!("Database error: {}", e)}));
                                }
                            }
                        }
                        return HttpResponse::Ok().json(json!({"message": "Update rolled back successfully"}));
                    }
                    return HttpResponse::BadRequest().json(json!({"error": "No backup available for this update"}));
                }
                return HttpResponse::BadRequest().json(json!({"error": "Update is not in applied state"}));
            }
        }
    }
    HttpResponse::NotFound().json(json!({"error": "Update not found"}))
}

// Simulation endpoint for Jeebs to "think" of an update
#[post("/api/evolution/brainstorm")]
pub async fn brainstorm_update(data: web::Data<AppState>, session: Session) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").ok().flatten().unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only (for simulation trigger)"}));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let update = ProposedUpdate {
        id: id.clone(),
        title: "Self-Evolution: New Reflex".to_string(),
        description: "I have researched my interaction logs and decided to add a new reflex for 'ping'.".to_string(),
        changes: vec![FileChange { path: "src/plugins/ping.rs".to_string(), new_content: "// Auto-generated Ping Plugin\n".to_string() }],
        status: "pending".to_string(),
        created_at: chrono::Local::now().to_rfc3339(),
        backup: None,
    };
    let key = format!("evolution:update:{}", id);
    if let Ok(json_bytes) = serde_json::to_vec(&update) {
        if let Ok(val) = encode_all(&json_bytes, 1) {
            if let Err(e) = sqlx::query("INSERT INTO jeebs_store (key, value) VALUES (?, ?)").bind(key).bind(val).execute(&data.db).await {
                return HttpResponse::InternalServerError().json(json!({"error": format!("Database error: {}", e)}));
            }
        }
    }
    HttpResponse::Ok().json(json!({"message": "Jeebs has proposed a new update!", "id": id}))
}
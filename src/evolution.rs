use actix_web::{get, post, delete, web, HttpResponse, Responder};
use actix_session::Session;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use std::fs;
use std::path::Path;
use crate::state::AppState;
use crate::utils::{encode_all, decode_all};
use chrono::Local;

#[derive(Serialize, Deserialize, Clone)]
pub struct FileChange {
    pub path: String,
    pub new_content: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Comment {
    pub author: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Notification {
    pub id: String,
    pub message: String,
    pub severity: String,
    pub created_at: String,
    pub link: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProposedUpdate {
    pub id: String,
    pub title: String,
    pub author: String,
    pub severity: String,
    #[serde(default)]
    pub comments: Vec<Comment>,
    pub description: String,
    pub changes: Vec<FileChange>,
    pub status: String, // "pending", "applied", "denied"
    pub created_at: String,
    pub backup: Option<Vec<FileChange>>,
}

#[derive(Serialize)]
struct UpdatesResponse {
    updates: Vec<ProposedUpdate>,
    role: String,
}

#[get("/api/admin/evolution/updates")]
pub async fn list_updates(data: web::Data<AppState>, session: Session) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"})),
    };

    let role: String = match sqlx::query("SELECT role FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&data.db)
        .await {
            Ok(Some(row)) => row.get(0),
            _ => return HttpResponse::Unauthorized().json(json!({"error": "User not found"})),
        };

    if role == "user" {
        return HttpResponse::Forbidden().json(json!({"error": "Access denied"}));
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

    HttpResponse::Ok().json(UpdatesResponse {
        updates,
        role
    })
}

#[post("/api/admin/evolution/apply/{id}")]
pub async fn apply_update(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: Session,
) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"})),
    };

    let role: String = match sqlx::query("SELECT role FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&data.db)
        .await {
            Ok(Some(row)) => row.get(0),
            _ => return HttpResponse::Unauthorized().json(json!({"error": "User not found"})),
        };

    if role != "admin" {
        return HttpResponse::Forbidden().json(json!({"error": "Admin only"}));
    }

    let id = path.into_inner();
    let key = format!("evolution:update:{}", id);

    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(&key).fetch_optional(&data.db).await {
        let val: Vec<u8> = row.get(0);
        if let Ok(bytes) = decode_all(&val) {
            if let Ok(mut update) = serde_json::from_slice::<ProposedUpdate>(&bytes) {
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
                let new_val = encode_all(&serde_json::to_vec(&update).unwrap(), 1).unwrap();
                sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)").bind(&key).bind(new_val).execute(&data.db).await.unwrap();
                crate::logging::log(&data.db, "INFO", "EVOLUTION", &format!("Applied update: {}", update.title)).await;

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
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"})),
    };

    let role: String = match sqlx::query("SELECT role FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&data.db)
        .await {
            Ok(Some(row)) => row.get(0),
            _ => return HttpResponse::Unauthorized().json(json!({"error": "User not found"})),
        };

    if role != "admin" {
        return HttpResponse::Forbidden().json(json!({"error": "Admin only"}));
    }

    let id = path.into_inner();
    let key = format!("evolution:update:{}", id);

    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(&key).fetch_optional(&data.db).await {
        let val: Vec<u8> = row.get(0);
        if let Ok(bytes) = decode_all(&val) {
            if let Ok(mut update) = serde_json::from_slice::<ProposedUpdate>(&bytes) {
                update.status = "denied".to_string();
                let new_val = encode_all(&serde_json::to_vec(&update).unwrap(), 1).unwrap();
                sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)").bind(&key).bind(new_val).execute(&data.db).await.unwrap();
                crate::logging::log(&data.db, "WARN", "EVOLUTION", &format!("Denied update: {}", update.title)).await;
                return HttpResponse::Ok().json(json!({"message": "Update denied"}));
            }
        }
    }
    HttpResponse::NotFound().json(json!({"error": "Update not found"}))
}

#[post("/api/admin/evolution/resolve/{id}")]
pub async fn resolve_update(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: Session,
) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"})),
    };

    let role: String = match sqlx::query("SELECT role FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&data.db)
        .await {
            Ok(Some(row)) => row.get(0),
            _ => return HttpResponse::Unauthorized().json(json!({"error": "User not found"})),
        };

    if role != "admin" {
        return HttpResponse::Forbidden().json(json!({"error": "Admin only"}));
    }

    let id = path.into_inner();
    let key = format!("evolution:update:{}", id);

    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(&key).fetch_optional(&data.db).await {
        let val: Vec<u8> = row.get(0);
        if let Ok(bytes) = decode_all(&val) {
            if let Ok(mut update) = serde_json::from_slice::<ProposedUpdate>(&bytes) {
                update.status = "resolved".to_string();
                let new_val = encode_all(&serde_json::to_vec(&update).unwrap(), 1).unwrap();
                sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)").bind(&key).bind(new_val).execute(&data.db).await.unwrap();
                return HttpResponse::Ok().json(json!({"message": "Update resolved"}));
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
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"})),
    };

    let role: String = match sqlx::query("SELECT role FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&data.db)
        .await {
            Ok(Some(row)) => row.get(0),
            _ => return HttpResponse::Unauthorized().json(json!({"error": "User not found"})),
        };

    if role != "admin" {
        return HttpResponse::Forbidden().json(json!({"error": "Admin only"}));
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
                        let new_val = encode_all(&serde_json::to_vec(&update).unwrap(), 1).unwrap();
                        sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)").bind(&key).bind(new_val).execute(&data.db).await.unwrap();
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

#[derive(Deserialize)]
pub struct CreateComment {
    pub content: String,
}

#[post("/api/admin/evolution/comment/{id}")]
pub async fn add_comment(
    data: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<CreateComment>,
    session: Session,
) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"})),
    };

    let role: String = match sqlx::query("SELECT role FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&data.db)
        .await {
            Ok(Some(row)) => row.get(0),
            _ => return HttpResponse::Unauthorized().json(json!({"error": "User not found"})),
        };

    if role == "user" {
        return HttpResponse::Forbidden().json(json!({"error": "Access denied"}));
    }

    let id = path.into_inner();
    let key = format!("evolution:update:{}", id);

    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(&key).fetch_optional(&data.db).await {
        let val: Vec<u8> = row.get(0);
        if let Ok(bytes) = decode_all(&val) {
            if let Ok(mut update) = serde_json::from_slice::<ProposedUpdate>(&bytes) {
                let comment = Comment {
                    author: username,
                    content: body.content.clone(),
                    timestamp: Local::now().to_rfc3339(),
                };
                update.comments.push(comment);
                
                let new_val = encode_all(&serde_json::to_vec(&update).unwrap(), 1).unwrap();
                sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)").bind(&key).bind(new_val).execute(&data.db).await.unwrap();
                return HttpResponse::Ok().json(json!({"message": "Comment added"}));
            }
        }
    }
    HttpResponse::NotFound().json(json!({"error": "Update not found"}))
}

#[get("/api/admin/notifications")]
pub async fn get_notifications(data: web::Data<AppState>, session: Session) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"})),
    };

    let role: String = match sqlx::query("SELECT role FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&data.db)
        .await {
            Ok(Some(row)) => row.get(0),
            _ => return HttpResponse::Unauthorized().json(json!({"error": "User not found"})),
        };

    if role != "admin" {
        return HttpResponse::Forbidden().json(json!({"error": "Admin only"}));
    }

    let rows = sqlx::query("SELECT value FROM jeebs_store WHERE key LIKE 'notification:%'")
        .fetch_all(&data.db).await.unwrap_or_default();

    let mut notifications = Vec::new();
    for row in rows {
        let val: Vec<u8> = row.get(0);
        if let Ok(bytes) = decode_all(&val) {
            if let Ok(notif) = serde_json::from_slice::<Notification>(&bytes) {
                notifications.push(notif);
            }
        }
    }
    // Sort by date desc
    notifications.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    HttpResponse::Ok().json(notifications)
}

#[delete("/api/admin/notification/{id}")]
pub async fn dismiss_notification(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: Session,
) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"})),
    };

    let role: String = match sqlx::query("SELECT role FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&data.db)
        .await {
            Ok(Some(row)) => row.get(0),
            _ => return HttpResponse::Unauthorized().json(json!({"error": "User not found"})),
        };

    if role != "admin" {
        return HttpResponse::Forbidden().json(json!({"error": "Admin only"}));
    }

    let id = path.into_inner();
    let key = format!("notification:{}", id);
    sqlx::query("DELETE FROM jeebs_store WHERE key = ?").bind(key).execute(&data.db).await.unwrap();
    
    HttpResponse::Ok().json(json!({"ok": true}))
}

// Simulation endpoint for Jeebs to "think" of an update
#[post("/api/evolution/brainstorm")]
pub async fn brainstorm_update(data: web::Data<AppState>, session: Session) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"})),
    };

    let role: String = match sqlx::query("SELECT role FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&data.db)
        .await {
            Ok(Some(row)) => row.get(0),
            _ => return HttpResponse::Unauthorized().json(json!({"error": "User not found"})),
        };

    if role != "admin" {
        return HttpResponse::Forbidden().json(json!({"error": "Admin only"}));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let update = ProposedUpdate {
        id: id.clone(),
        title: "Self-Evolution: New Reflex".to_string(),
        author: "Jeebs (Simulation)".to_string(),
        severity: "Low".to_string(),
        comments: Vec::new(),
        description: "I have researched my interaction logs and decided to add a new reflex for 'ping'.".to_string(),
        changes: vec![FileChange { path: "src/plugins/ping.rs".to_string(), new_content: "// Auto-generated Ping Plugin\n".to_string() }],
        status: "pending".to_string(),
        created_at: chrono::Local::now().to_rfc3339(),
        backup: None,
    };
    let key = format!("evolution:update:{}", id);
    let val = encode_all(&serde_json::to_vec(&update).unwrap(), 1).unwrap();
    sqlx::query("INSERT INTO jeebs_store (key, value) VALUES (?, ?)").bind(key).bind(val).execute(&data.db).await.unwrap();
    HttpResponse::Ok().json(json!({"message": "Jeebs has proposed a new update!", "id": id}))
}
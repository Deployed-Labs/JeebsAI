use actix_web::{post, web, Responder, HttpResponse};
use actix_session::Session;
use serde::Deserialize;
use serde_json::Value;
use crate::state::AppState;
use crate::utils::encode_all;
use crate::brain::BrainNode;

#[derive(Deserialize)]
pub struct ImportRequest {
    store: Option<serde_json::Map<String, Value>>,
    brain: Option<Vec<BrainNode>>,
}

#[post("/api/admin/import")]
pub async fn import_database(
    data: web::Data<AppState>,
    req: web::Json<ImportRequest>,
    session: Session,
) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").ok().flatten().unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Admin only"}));
    }

    let db = &data.db;

    // Import Store
    if let Some(store) = &req.store {
        for (key, value) in store {
            // If value is a string, it was likely a raw string (token/chat).
            // If it's an object, it was a struct (User).
            let bytes = if let Some(s) = value.as_str() {
                s.as_bytes().to_vec()
            } else {
                serde_json::to_vec(value).unwrap_or_default()
            };
            
            if !bytes.is_empty() {
                if let Ok(compressed) = encode_all(&bytes, 1) {
                     let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
                        .bind(key)
                        .bind(compressed)
                        .execute(db).await;
                }
            }
        }
    }

    // Import Brain
    if let Some(brain) = &req.brain {
        for node in brain {
             if let Ok(val) = serde_json::to_vec(node) {
                 if let Ok(compressed) = encode_all(&val, 1) {
                    let _ = sqlx::query("INSERT OR REPLACE INTO brain_nodes (id, label, summary, data, created_at) VALUES (?, ?, ?, ?, ?)")
                        .bind(&node.id)
                        .bind(&node.label)
                        .bind(&node.summary)
                        .bind(&compressed)
                        .bind(&node.created_at)
                        .execute(db).await;
                 }
             }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({"ok": true}))
}
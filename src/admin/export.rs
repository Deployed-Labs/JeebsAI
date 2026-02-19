use crate::state::AppState;
use crate::utils::decode_all;
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;
use sqlx::Row;

#[get("/api/admin/export")]
pub async fn export_database(data: web::Data<AppState>, session: Session) -> impl Responder {
    let is_admin = session
        .get::<bool>("is_admin")
        .unwrap_or(Some(false))
        .unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }

    let db = &data.db;

    // Export jeebs_store
    let mut store_data = serde_json::Map::new();
    if let Ok(rows) = sqlx::query("SELECT key, value FROM jeebs_store")
        .fetch_all(db)
        .await
    {
        for row in rows {
            let key: String = row.get(0);
            let val: Vec<u8> = row.get(1);

            if let Ok(decompressed) = decode_all(&val) {
                if let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(&decompressed) {
                    store_data.insert(key, json_val);
                } else if let Ok(text) = String::from_utf8(decompressed) {
                    store_data.insert(key, json!(text));
                }
            }
        }
    }

    // Export brain_nodes
    let mut brain_data = Vec::new();
    if let Ok(rows) = sqlx::query("SELECT data FROM brain_nodes")
        .fetch_all(db)
        .await
    {
        for row in rows {
            let data_blob: Vec<u8> = row.get(0);
            if let Ok(decompressed) = decode_all(&data_blob) {
                if let Ok(node) = serde_json::from_slice::<serde_json::Value>(&decompressed) {
                    brain_data.push(node);
                }
            }
        }
    }

    let export = json!({
        "store": store_data,
        "brain": brain_data,
        "exported_at": chrono::Local::now().to_rfc3339()
    });

    HttpResponse::Ok()
        .insert_header((
            "Content-Disposition",
            "attachment; filename=\"jeebs_export.json\"",
        ))
        .json(export)
}

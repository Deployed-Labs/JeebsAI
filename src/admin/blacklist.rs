use actix_session::Session;
use actix_web::{HttpResponse, Responder, delete, get, post, web};
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct IpRequest {
    pub ip: String,
}

#[get("/api/admin/blacklist")]
pub async fn get_blacklist(data: web::Data<AppState>, session: Session) -> impl Responder {
    let is_admin = session
        .get::<bool>("is_admin")
        .unwrap_or(Some(false))
        .unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }

    let blacklist = data.ip_blacklist.read().unwrap();
    let ips: Vec<String> = blacklist.iter().cloned().collect();
    HttpResponse::Ok().json(ips)
}

#[post("/api/admin/blacklist")]
pub async fn add_blacklist_ip(
    data: web::Data<AppState>,
    req: web::Json<IpRequest>,
    session: Session,
) -> impl Responder {
    let is_admin = session
        .get::<bool>("is_admin")
        .unwrap_or(Some(false))
        .unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }

    let ip = req.ip.trim().to_string();
    sqlx::query("INSERT OR REPLACE INTO ip_blacklist (ip) VALUES (?)")
        .bind(&ip)
        .execute(&data.db)
        .await
        .unwrap();

    data.ip_blacklist.write().unwrap().insert(ip);
    HttpResponse::Ok().json(json!({"ok": true}))
}

#[delete("/api/admin/blacklist")]
pub async fn remove_blacklist_ip(
    data: web::Data<AppState>,
    req: web::Json<IpRequest>,
    session: Session,
) -> impl Responder {
    let is_admin = session
        .get::<bool>("is_admin")
        .unwrap_or(Some(false))
        .unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }

    let ip = req.ip.trim().to_string();
    sqlx::query("DELETE FROM ip_blacklist WHERE ip = ?")
        .bind(&ip)
        .execute(&data.db)
        .await
        .unwrap();

    data.ip_blacklist.write().unwrap().remove(&ip);
    HttpResponse::Ok().json(json!({"ok": true}))
}

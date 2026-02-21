use actix_session::Session;
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct IpRequest {
    pub ip: String,
}

#[get("/api/admin/whitelist")]
pub async fn get_whitelist(data: web::Data<AppState>, session: Session) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let whitelist = data.ip_whitelist.read().unwrap();
    let ips: Vec<String> = whitelist.iter().cloned().collect();
    HttpResponse::Ok().json(ips)
}

#[post("/api/admin/whitelist")]
pub async fn add_whitelist_ip(
    data: web::Data<AppState>,
    req: web::Json<IpRequest>,
    session: Session,
) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let ip = req.ip.trim().to_string();
    sqlx::query("INSERT OR REPLACE INTO ip_whitelist (ip) VALUES (?)")
        .bind(&ip)
        .execute(&data.db)
        .await
        .unwrap();

    data.ip_whitelist.write().unwrap().insert(ip);
    crate::logging::log(
        &data.db,
        "INFO",
        "SECURITY",
        &format!("Added IP to whitelist ip={}", req.ip.trim()),
    )
    .await;
    HttpResponse::Ok().json(json!({"ok": true}))
}

#[delete("/api/admin/whitelist")]
pub async fn remove_whitelist_ip(
    data: web::Data<AppState>,
    req: web::Json<IpRequest>,
    session: Session,
) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let ip = req.ip.trim().to_string();
    sqlx::query("DELETE FROM ip_whitelist WHERE ip = ?")
        .bind(&ip)
        .execute(&data.db)
        .await
        .unwrap();

    data.ip_whitelist.write().unwrap().remove(&ip);
    crate::logging::log(
        &data.db,
        "INFO",
        "SECURITY",
        &format!("Removed IP from whitelist ip={}", req.ip.trim()),
    )
    .await;
    HttpResponse::Ok().json(json!({"ok": true}))
}

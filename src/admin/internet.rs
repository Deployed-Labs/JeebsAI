use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::state::AppState;

#[derive(Serialize)]
struct InternetStatusResponse {
    enabled: bool,
}

#[derive(Deserialize)]
struct SetInternetRequest {
    enabled: bool,
}

/// Get current internet connectivity status
#[get("/api/admin/internet/status")]
pub async fn get_internet_status(
    data: web::Data<AppState>,
    session: Session,
) -> impl Responder {
    // Check if user is logged in
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Not logged in"}));
        }
    };

    // Check if user is admin
    let user_key = format!("user:{username}");
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            let role = user_json["role"].as_str().unwrap_or("user");
            if role != "admin" {
                return HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "Admin access required"}));
            }
        }
    } else {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Admin access required"}));
    }

    let enabled = *data.internet_enabled.read().unwrap();
    HttpResponse::Ok().json(InternetStatusResponse { enabled })
}

/// Set internet connectivity status (admin only)
#[post("/api/admin/internet/set")]
pub async fn set_internet_status(
    data: web::Data<AppState>,
    session: Session,
    req: web::Json<SetInternetRequest>,
) -> impl Responder {
    // Check if user is logged in
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Not logged in"}));
        }
    };

    // Check if user is admin
    let user_key = format!("user:{username}");
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            let role = user_json["role"].as_str().unwrap_or("user");
            if role != "admin" {
                return HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "Admin access required"}));
            }
        }
    } else {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Admin access required"}));
    }

    // Update internet connectivity status
    let mut enabled = data.internet_enabled.write().unwrap();
    *enabled = req.enabled;

    // Log the change
    crate::logging::log(
        &data.db,
        "internet_toggle",
        &format!(
            "Internet connectivity {} by admin {}",
            if req.enabled { "enabled" } else { "disabled" },
            username
        ),
        Some(&username),
    )
    .await;

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "enabled": req.enabled,
        "message": format!("Internet connectivity {}", if req.enabled { "enabled" } else { "disabled" })
    }))
}

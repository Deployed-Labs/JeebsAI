use crate::state::AppState;
use actix_session::Session;
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;

#[derive(Serialize)]
pub struct UserInfo {
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    pub role: String,
}

#[get("/api/admin/users")]
pub async fn admin_list_users(data: web::Data<AppState>, session: Session) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let rows = sqlx::query("SELECT key, value FROM jeebs_store WHERE key LIKE 'user:%'")
        .fetch_all(&data.db)
        .await
        .unwrap_or_default();

    let mut users = Vec::new();
    for row in rows {
        let key: String = row.get(0);
        let val: Vec<u8> = row.get(1);
        if let Ok(user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            let username = key.strip_prefix("user:").unwrap_or(&key).to_string();
            let email = user_json["email"].as_str().unwrap_or("").to_string();
            let role = user_json["role"].as_str().unwrap_or("user").to_string();
            let is_admin = crate::auth::is_admin_role(&role);
            users.push(UserInfo {
                username,
                email,
                is_admin,
                role,
            });
        }
    }
    HttpResponse::Ok().json(users)
}

#[delete("/api/admin/user/{username}")]
pub async fn admin_delete_user(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: Session,
) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }
    let username = path.into_inner();
    if username == "admin" || username == crate::auth::ROOT_ADMIN_USERNAME {
        return HttpResponse::BadRequest().json(json!({"error": "Cannot delete root admin"}));
    }

    let user_key = format!("user:{username}");
    sqlx::query("DELETE FROM jeebs_store WHERE key = ?")
        .bind(user_key)
        .execute(&data.db)
        .await
        .unwrap();
    HttpResponse::Ok().json(json!({"ok": true}))
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub username: String,
    pub new_password: String,
}

#[post("/api/admin/user/reset_password")]
pub async fn admin_reset_user_password(
    data: web::Data<AppState>,
    req: web::Json<ResetPasswordRequest>,
    session: Session,
) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let user_key = format!("user:{}", req.username);
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(mut user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            // Switch to PGP-only accounts: clear any stored password and mark as PGP
            user_json["password"] = serde_json::Value::String("".to_string());
            user_json["auth_type"] = serde_json::Value::String("pgp".to_string());

            sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
                .bind(&user_key)
                .bind(serde_json::to_vec(&user_json).unwrap())
                .execute(&data.db)
                .await
                .unwrap();

            return HttpResponse::Ok().json(json!({"ok": true}));
        }
    }
    HttpResponse::NotFound().json(json!({"error": "User not found"}))
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub username: String,
    pub role: String,
}

#[post("/api/admin/user/role")]
pub async fn admin_update_user_role(
    data: web::Data<AppState>,
    req: web::Json<UpdateRoleRequest>,
    session: Session,
) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    if req.username == "admin" || req.username == crate::auth::ROOT_ADMIN_USERNAME {
        return HttpResponse::BadRequest().json(json!({"error": "Cannot change root admin role"}));
    }

    if req.role == "admin" || req.role == "super_admin" {
        return HttpResponse::BadRequest().json(json!({"error": "Only root admin can hold admin role"}));
    }

    if req.role != "user" && req.role != "trainer" {
        return HttpResponse::BadRequest()
            .json(json!({"error": "Role must be 'user' or 'trainer'"}));
    }

    let user_key = format!("user:{}", req.username);
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(mut user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            user_json["role"] = serde_json::Value::String(req.role.clone());

            sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
                .bind(&user_key)
                .bind(serde_json::to_vec(&user_json).unwrap())
                .execute(&data.db)
                .await
                .unwrap();

            return HttpResponse::Ok().json(json!({"ok": true}));
        }
    }
    HttpResponse::NotFound().json(json!({"error": "User not found"}))
}

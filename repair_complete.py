path = "/root/JeebsAI/src/auth/mod.rs"

content = """use actix_web::{web, HttpResponse, Responder, HttpRequest};
use actix_session::Session;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::state::AppState;
use crate::admin::user::User;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

pub async fn login(
    data: web::Data<AppState>,
    req: web::Json<LoginRequest>,
    session: Session,
) -> impl Responder {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&req.username)
        .fetch_optional(&data.db)
        .await;

    match user {
        Ok(Some(user)) => {
            if req.password == user.password {
                let _ = session.insert("user_id", user.id);
                HttpResponse::Ok().json(json!({"status": "success", "user": user.username}))
            } else {
                HttpResponse::Unauthorized().json(json!({"error": "Invalid password"}))
            }
        }
        _ => HttpResponse::Unauthorized().json(json!({"error": "User not found"})),
    }
}
"""

with open(path, "w") as f:
    f.write(content)

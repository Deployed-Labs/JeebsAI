use actix_web::{web, HttpResponse, Responder, HttpRequest, post};
use actix_session::Session;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::state::AppState;
use sqlx::Row;
use argon2::{Argon2};
use argon2::password_hash::{PasswordHash, PasswordVerifier};

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

#[post("/api/login")]
pub async fn login(
    data: web::Data<AppState>,
    req: web::Json<LoginRequest>,
    session: Session,
) -> impl Responder {
    // Users are stored in `jeebs_store` under key `user:{username}` as JSON blob.
    let user_key = format!("user:{}", req.username);
    let row = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await;

    match row {
        Ok(Some(r)) => {
            let val: Vec<u8> = r.get(0);
            if let Ok(user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
                // If account is PGP-only, instruct client to use PGP flow
                if user_json.get("auth_type").and_then(|v| v.as_str()) == Some("pgp") {
                    return HttpResponse::BadRequest().json(json!({"error": "This account requires PGP authentication", "use_pgp": true}));
                }

                let stored = user_json.get("password").and_then(|v| v.as_str()).unwrap_or("");
                if stored.is_empty() {
                    return HttpResponse::Unauthorized().json(json!({"error": "User has no password set"}));
                }

                // Verify Argon2 password hash
                match PasswordHash::new(stored) {
                    Ok(parsed_hash) => {
                        let verifier = Argon2::default();
                        if verifier.verify_password(req.password.as_bytes(), &parsed_hash).is_ok() {
                            // Successful login: store username in session
                            let _ = session.insert("username", req.username.clone());
                            let role = user_json.get("role").and_then(|v| v.as_str()).unwrap_or("user");
                            let is_admin = role == "admin";
                            let _ = session.insert("is_admin", is_admin);
                            return HttpResponse::Ok().json(json!({"status": "success", "user": req.username, "token": ""}));
                        } else {
                            return HttpResponse::Unauthorized().json(json!({"error": "Invalid password"}));
                        }
                    }
                    Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Malformed password hash"})),
                }
            } else {
                return HttpResponse::InternalServerError().json(json!({"error": "Cannot parse user data"}));
            }
        }
        _ => HttpResponse::Unauthorized().json(json!({"error": "User not found"})),
    }
}

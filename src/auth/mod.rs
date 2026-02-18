use crate::state::AppState;
use crate::utils::{encode_all, decode_all};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use actix_session::Session;
use serde_json::json;
use serde::Deserialize;
use chrono::Local;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;
use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::Local;
use futures_util::TryStreamExt;
use rand_core::OsRng;
use serde::Deserialize;
use serde_json::json;
use sqlx::{Row, SqlitePool};

pub mod pgp;

pub async fn ensure_admin_exists(db: &SqlitePool) {
    let key = "user:admin";
    match sqlx::query("SELECT 1 FROM jeebs_store WHERE key = ?")
        .bind(key)
        .fetch_optional(db)
        .await
    {
        Ok(None) => {
            // Generate a random password instead of hardcoding "admin"
            let new_password = uuid::Uuid::new_v4().to_string();
            let salt = argon2::password_hash::SaltString::generate(&mut OsRng);
            let hash = match Argon2::default().hash_password(new_password.as_bytes(), &salt) {
                Ok(h) => h.to_string(),
                Err(e) => {
                    eprintln!("Failed to hash admin password: {e}");
                    return;
                }
            };
            let user_json = json!({
                "username": "admin",
                "password": hash,
                "email": "admin@jeebs.club",
                "role": "admin"
            });

            if let Ok(user_bytes) = serde_json::to_vec(&user_json) {
                if let Err(e) = sqlx::query("INSERT INTO jeebs_store (key, value) VALUES (?, ?)")
                    .bind(key)
                    .bind(user_bytes)
                    .execute(db)
                    .await
                {
                    eprintln!("Failed to create admin user: {e}");
                    return;
                }
                println!(
                    "\n!!! IMPORTANT !!!\nAdmin account created.\nUsername: admin\nPassword: {new_password}\n!!! SAVE THIS PASSWORD !!!\n"
                );
            }
        }
        Ok(Some(_)) => {}
        Err(e) => eprintln!("Failed to check for admin user: {e}"),
    }
}

pub async fn ensure_user(db: &SqlitePool, username: &str, password: &str, role: &str) {
    let key = format!("user:{username}");
    match sqlx::query("SELECT 1 FROM jeebs_store WHERE key = ?")
        .bind(&key)
        .fetch_optional(db)
        .await
    {
        Ok(None) => {
            let salt = argon2::password_hash::SaltString::generate(&mut OsRng);
            let hash = match Argon2::default().hash_password(password.as_bytes(), &salt) {
                Ok(h) => h.to_string(),
                Err(e) => {
                    eprintln!("Failed to hash password: {e}");
                    return;
                }
            };
            let user_json = json!({
                "username": username,
                "password": hash,
                "email": format!("{}@jeebs.club", username),
                "role": role
            });

            if let Ok(user_bytes) = serde_json::to_vec(&user_json) {
                if let Err(e) = sqlx::query("INSERT INTO jeebs_store (key, value) VALUES (?, ?)")
                    .bind(&key)
                    .bind(user_bytes)
                    .execute(db)
                    .await
                {
                    eprintln!("Failed to create user {username}: {e}");
                    return;
                }
                println!("User account created: {username}");
            }
        }
        Ok(Some(_)) => {}
        Err(e) => eprintln!("Failed to check for user {username}: {e}"),
    }
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[post("/api/register")]
pub async fn register(
    data: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    let user_key = format!("user:{}", req.username);
    match sqlx::query("SELECT 1 FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await
    {
        Ok(Some(_)) => return HttpResponse::BadRequest().json(json!({"error": "Username taken"})),
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({"error": "Database error"}));
        }
        Ok(None) => {}
    }

    let salt = argon2::password_hash::SaltString::generate(&mut OsRng);
    let hash = match Argon2::default().hash_password(req.password.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Password hashing failed"}));
        }
    };

    let user_json = json!({
        "username": req.username,
        "password": hash,
        "email": req.email,
        "role": "user"
    });

    if let Ok(user_bytes) = serde_json::to_vec(&user_json) {
        if sqlx::query("INSERT INTO jeebs_store (key, value) VALUES (?, ?)")
            .bind(&user_key)
            .bind(user_bytes)
            .execute(&data.db)
            .await
            .is_err()
        {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Database insert failed"}));
        }
    } else {
        return HttpResponse::InternalServerError().json(json!({"error": "Serialization failed"}));
    }

    // Verification token logic
    let token = uuid::Uuid::new_v4().to_string();
    let token_key = format!("verify_token:{}", req.username);
    if let Ok(encoded_token) = encode_all(token.as_bytes(), 1) {
        let _ = sqlx::query("INSERT INTO jeebs_store (key, value) VALUES (?, ?)")
            .bind(&token_key)
            .bind(encoded_token)
            .execute(&data.db)
            .await;
    }

    crate::logging::log(
        &data.db,
        "INFO",
        "AUTH",
        &format!("User registered: {}. Verify token: {}", req.username, token),
    )
    .await;

    HttpResponse::Ok().json(json!({"ok": true}))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

#[post("/api/login")]
pub async fn login(
    data: web::Data<AppState>,
    req: web::Json<LoginRequest>,
    session: Session,
    http_req: HttpRequest,
) -> impl Responder {
    // Extract IP, preferring X-Forwarded-For if available (for Nginx)
    let ip = http_req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| http_req.peer_addr().map(|a| a.ip().to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    let rate_limit_key = format!("ratelimit:login:{ip}");
    let now = Local::now().timestamp();

    // Check Rate Limit
    let mut attempts = 0;
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&rate_limit_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(limit_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            attempts = limit_json["attempts"].as_u64().unwrap_or(0);
            let last_attempt = limit_json["last_attempt"].as_i64().unwrap_or(0);

            if attempts >= 5 {
                if now - last_attempt < 15 * 60 {
                    crate::logging::log(
                        &data.db,
                        "WARN",
                        "AUTH",
                        &format!("Rate limit exceeded for IP: {ip}"),
                    )
                    .await;
                    return HttpResponse::TooManyRequests().json(
                        json!({"error": "Too many login attempts. Try again in 15 minutes."}),
                    );
                } else {
                    attempts = 0; // Reset after timeout
                }
            }
        }
    }

    let user_key = format!("user:{}", req.username);
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            // Check if this is a PGP-only user
            if user_json.get("auth_type").and_then(|v| v.as_str()) == Some("pgp") {
                return HttpResponse::BadRequest().json(json!({
                    "error": "This account requires PGP authentication",
                    "use_pgp": true
                }));
            }

            let stored_hash = user_json["password"].as_str().unwrap_or("");
            let parsed_hash = match PasswordHash::new(stored_hash) {
                Ok(h) => h,
                Err(_) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "Invalid password hash"}));
                }
            };

            if Argon2::default()
                .verify_password(req.password.as_bytes(), &parsed_hash)
                .is_ok()
            {
                if session.insert("username", &req.username).is_err() {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "Session error"}));
                }
                let role = user_json["role"].as_str().unwrap_or("user");
                if session.insert("is_admin", role == "admin").is_err() {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "Session error"}));
                }

                if req.remember_me.unwrap_or(false) {
                    // Extend session if supported by middleware configuration
                    session.renew();
                }

                // Clear rate limit on success
                let _ = sqlx::query("DELETE FROM jeebs_store WHERE key = ?")
                    .bind(&rate_limit_key)
                    .execute(&data.db)
                    .await;

                return HttpResponse::Ok().json(json!({
                    "username": req.username,
                    "is_admin": role == "admin"
                }));
            }
        }
    }

    // Increment rate limit on failure
    attempts += 1;
    let limit_json = json!({
        "attempts": attempts,
        "last_attempt": now
    });
    if let Ok(val) = serde_json::to_vec(&limit_json) {
        let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
            .bind(&rate_limit_key)
            .bind(val)
            .execute(&data.db)
            .await;
    }

    HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"}))
}

#[derive(Deserialize)]
pub struct PgpLoginRequest {
    pub username: String,
    pub signed_message: String,
    pub remember_me: Option<bool>,
}

#[post("/api/login_pgp")]
pub async fn login_pgp(
    data: web::Data<AppState>,
    req: web::Json<PgpLoginRequest>,
    session: Session,
    http_req: HttpRequest,
) -> impl Responder {
    // Only allow PGP login for the 1090mb user
    if req.username != "1090mb" {
        return HttpResponse::BadRequest()
            .json(json!({"error": "PGP authentication not enabled for this user"}));
    }

    // Extract IP for rate limiting
    let ip = http_req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| http_req.peer_addr().map(|a| a.ip().to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    let rate_limit_key = format!("ratelimit:login:{ip}");
    let now = Local::now().timestamp();

    // Check Rate Limit
    let mut attempts = 0;
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&rate_limit_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(limit_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            attempts = limit_json["attempts"].as_u64().unwrap_or(0);
            let last_attempt = limit_json["last_attempt"].as_i64().unwrap_or(0);

            if attempts >= 5 {
                if now - last_attempt < 15 * 60 {
                    crate::logging::log(
                        &data.db,
                        "WARN",
                        "AUTH",
                        &format!("Rate limit exceeded for IP: {ip}"),
                    )
                    .await;
                    return HttpResponse::TooManyRequests().json(
                        json!({"error": "Too many login attempts. Try again in 15 minutes."}),
                    );
                } else {
                    attempts = 0; // Reset after timeout
                }
            }
        }
    }

    // Verify the PGP signature
    let verified_message = match pgp::verify_signature(&req.signed_message) {
        Ok(msg) => msg,
        Err(e) => {
            // Increment rate limit on failure
            attempts += 1;
            let limit_json = json!({
                "attempts": attempts,
                "last_attempt": now
            });
            if let Ok(val) = serde_json::to_vec(&limit_json) {
                let _ =
                    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
                        .bind(&rate_limit_key)
                        .bind(val)
                        .execute(&data.db)
                        .await;
            }

            crate::logging::log(
                &data.db,
                "WARN",
                "AUTH",
                &format!("Failed PGP login attempt for {}: {}", req.username, e),
            )
            .await;
            return HttpResponse::Unauthorized()
                .json(json!({"error": format!("PGP verification failed: {}", e)}));
        }
    };

    // The signed message should contain a timestamp to prevent replay attacks
    // Format: "LOGIN:username:timestamp"
    let parts: Vec<&str> = verified_message.trim().split(':').collect();
    if parts.len() != 3 || parts[0] != "LOGIN" || parts[1] != req.username {
        attempts += 1;
        let limit_json = json!({
            "attempts": attempts,
            "last_attempt": now
        });
        if let Ok(val) = serde_json::to_vec(&limit_json) {
            let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
                .bind(&rate_limit_key)
                .bind(val)
                .execute(&data.db)
                .await;
        }

        return HttpResponse::BadRequest().json(json!({"error": "Invalid message format"}));
    }

    // Check timestamp to prevent replay attacks (must be within 5 minutes in the past, allow small clock skew)
    if let Ok(timestamp) = parts[2].parse::<i64>() {
        let time_diff = now - timestamp;
        // Reject if timestamp is more than 5 minutes old or more than 1 minute in the future
        if !(-60..=300).contains(&time_diff) {
            return HttpResponse::BadRequest()
                .json(json!({"error": "Timestamp expired or invalid"}));
        }
    } else {
        return HttpResponse::BadRequest().json(json!({"error": "Invalid timestamp"}));
    }

    // Verify user exists and has PGP authentication enabled
    let user_key = format!("user:{}", req.username);
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            if user_json.get("auth_type").and_then(|v| v.as_str()) != Some("pgp") {
                return HttpResponse::BadRequest()
                    .json(json!({"error": "PGP authentication not enabled for this user"}));
            }

            // Login successful
            if session.insert("username", &req.username).is_err() {
                return HttpResponse::InternalServerError().json(json!({"error": "Session error"}));
            }
            let role = user_json["role"].as_str().unwrap_or("user");
            if session.insert("is_admin", role == "admin").is_err() {
                return HttpResponse::InternalServerError().json(json!({"error": "Session error"}));
            }

            if req.remember_me.unwrap_or(false) {
                session.renew();
            }

            // Clear rate limit on success
            let _ = sqlx::query("DELETE FROM jeebs_store WHERE key = ?")
                .bind(&rate_limit_key)
                .execute(&data.db)
                .await;

            crate::logging::log(
                &data.db,
                "INFO",
                "AUTH",
                &format!("User {} logged in via PGP", req.username),
            )
            .await;

            return HttpResponse::Ok().json(json!({
                "username": req.username,
                "is_admin": role == "admin"
            }));
        }
    }

    HttpResponse::Unauthorized().json(json!({"error": "User not found"}))
}

#[post("/api/logout")]
pub async fn logout(session: Session) -> impl Responder {
    session.purge();
    HttpResponse::Ok().json(json!({"ok": true}))
}

#[derive(Deserialize)]
pub struct RequestResetRequest {
    pub username: String,
    pub email: String,
}

#[post("/api/request_reset")]
pub async fn request_reset(
    data: web::Data<AppState>,
    req: web::Json<RequestResetRequest>,
) -> impl Responder {
    let user_key = format!("user:{}", req.username);
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            if user_json["email"] == req.email {
                let token = uuid::Uuid::new_v4().to_string();
                // Store token
                let token_key = format!("reset_token:{}", req.username);
                if let Ok(encoded) = encode_all(token.as_bytes(), 1) {
                    let _ = sqlx::query(
                        "INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)",
                    )
                    .bind(&token_key)
                    .bind(encoded)
                    .execute(&data.db)
                    .await;
                }

                crate::logging::log(
                    &data.db,
                    "INFO",
                    "AUTH",
                    &format!("Password reset token for {}: {}", req.username, token),
                )
                .await;
                return HttpResponse::Ok().json(json!({"ok": true}));
            }
        }
    }
    // Return OK to avoid user enumeration
    HttpResponse::Ok().json(json!({"ok": true}))
}

#[derive(Deserialize)]
pub struct VerifyEmailRequest {
    pub username: String,
    pub token: String,
}

#[post("/api/verify_email")]
pub async fn verify_email(
    data: web::Data<AppState>,
    req: web::Json<VerifyEmailRequest>,
) -> impl Responder {
    let token_key = format!("verify_token:{}", req.username);
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&token_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(stored_token_bytes) = decode_all(&val) {
            let stored_token = String::from_utf8(stored_token_bytes).unwrap_or_default();
            if stored_token == req.token {
                // Token matches, delete it to verify
                if sqlx::query("DELETE FROM jeebs_store WHERE key = ?")
                    .bind(&token_key)
                    .execute(&data.db)
                    .await
                    .is_ok()
                {
                    crate::logging::log(
                        &data.db,
                        "INFO",
                        "AUTH",
                        &format!("Email verified for {}", req.username),
                    )
                    .await;
                    return HttpResponse::Ok().json(json!({"ok": true}));
                } else {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "Database error"}));
                }
            }
        }
    }
    HttpResponse::BadRequest().json(json!({"error": "Invalid token"}))
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub username: String,
    pub token: String,
    pub new_password: String,
}

#[post("/api/reset_password")]
pub async fn reset_password(
    data: web::Data<AppState>,
    req: web::Json<ResetPasswordRequest>,
) -> impl Responder {
    let token_key = format!("reset_token:{}", req.username);
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&token_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(stored_token_bytes) = decode_all(&val) {
            let stored_token = String::from_utf8(stored_token_bytes).unwrap_or_default();
            if stored_token == req.token {
                // Token matches, update password
                let user_key = format!("user:{}", req.username);
                if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
                    .bind(&user_key)
                    .fetch_optional(&data.db)
                    .await
                {
                    let val: Vec<u8> = row.get(0);
                    if let Ok(mut user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
                        let salt = argon2::password_hash::SaltString::generate(&mut OsRng);
                        let new_hash = match Argon2::default()
                            .hash_password(req.new_password.as_bytes(), &salt)
                        {
                            Ok(h) => h.to_string(),
                            Err(_) => {
                                return HttpResponse::InternalServerError()
                                    .json(json!({"error": "Hashing failed"}));
                            }
                        };
                        user_json["password"] = serde_json::Value::String(new_hash);

                        if let Ok(user_bytes) = serde_json::to_vec(&user_json) {
                            if sqlx::query(
                                "INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)",
                            )
                            .bind(&user_key)
                            .bind(user_bytes)
                            .execute(&data.db)
                            .await
                            .is_ok()
                            {
                                // Delete token
                                let _ = sqlx::query("DELETE FROM jeebs_store WHERE key = ?")
                                    .bind(&token_key)
                                    .execute(&data.db)
                                    .await;
                                return HttpResponse::Ok().json(json!({"ok": true}));
                            }
                        }
                    }
                }
            }
        }
    }
    HttpResponse::BadRequest().json(json!({"error": "Invalid token"}))
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[post("/api/change_password")]
pub async fn change_password(
    data: web::Data<AppState>,
    req: web::Json<ChangePasswordRequest>,
    session: Session,
) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"})),
    };

    let user_key = format!("user:{username}");
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(mut user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            // Verify old password
            let stored_hash = user_json["password"].as_str().unwrap_or("");
            let parsed_hash = match PasswordHash::new(stored_hash) {
                Ok(h) => h,
                Err(_) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "Invalid password hash"}));
                }
            };

            if Argon2::default()
                .verify_password(req.old_password.as_bytes(), &parsed_hash)
                .is_err()
            {
                return HttpResponse::BadRequest().json(json!({"error": "Incorrect old password"}));
            }

            // Set new password
            let salt = argon2::password_hash::SaltString::generate(&mut OsRng);
            let new_hash = match Argon2::default().hash_password(req.new_password.as_bytes(), &salt)
            {
                Ok(h) => h.to_string(),
                Err(_) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "Hashing failed"}));
                }
            };
            user_json["password"] = serde_json::Value::String(new_hash);

            if let Ok(user_bytes) = serde_json::to_vec(&user_json) {
                if sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
                    .bind(&user_key)
                    .bind(user_bytes)
                    .execute(&data.db)
                    .await
                    .is_ok()
                {
                    crate::logging::log(
                        &data.db,
                        "INFO",
                        "AUTH",
                        &format!("User {username} changed password"),
                    )
                    .await;
                    return HttpResponse::Ok().json(json!({"ok": true}));
                }
            }
        }
    }
    HttpResponse::InternalServerError().json(json!({"error": "User not found"}))
}

// --- Update email endpoint (used by main.rs router) ---
#[derive(Deserialize)]
pub struct UpdateEmailRequest {
    pub new_email: String,
    pub password: String,
}

#[post("/api/update_email")]
pub async fn update_email(
    data: web::Data<AppState>,
    req: web::Json<UpdateEmailRequest>,
    session: Session,
) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"})),
    };

    let user_key = format!("user:{username}");
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(mut user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            // Verify password
            let stored_hash = user_json["password"].as_str().unwrap_or("");
            let parsed_hash = match PasswordHash::new(stored_hash) {
                Ok(h) => h,
                Err(_) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "Invalid password hash"}));
                }
            };

            if Argon2::default()
                .verify_password(req.password.as_bytes(), &parsed_hash)
                .is_err()
            {
                return HttpResponse::BadRequest().json(json!({"error": "Incorrect password"}));
            }

            // Update email
            user_json["email"] = serde_json::Value::String(req.new_email.clone());

            if let Ok(user_bytes) = serde_json::to_vec(&user_json) {
                if sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
                    .bind(&user_key)
                    .bind(user_bytes)
                    .execute(&data.db)
                    .await
                    .is_ok()
                {
                    crate::logging::log(
                        &data.db,
                        "INFO",
                        "AUTH",
                        &format!("User {username} updated email"),
                    )
                    .await;
                    return HttpResponse::Ok().json(json!({"ok": true}));
                }
            }
        }
    }

    HttpResponse::InternalServerError().json(json!({"error": "User not found or update failed"}))
}

#[post("/api/upload_avatar")]
pub async fn upload_avatar(
    data: web::Data<AppState>,
    mut payload: Multipart,
    session: Session,
) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"})),
    };

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        if content_disposition.get_name() == Some("avatar") {
            let mut image_data = Vec::new();
            while let Ok(Some(chunk)) = field.try_next().await {
                image_data.extend_from_slice(&chunk);
                if image_data.len() > 5 * 1024 * 1024 {
                    // 5MB limit
                    return HttpResponse::BadRequest().json(json!({"error": "File too large"}));
                }
            }
            if !image_data.is_empty() {
                let key = format!("avatar:{username}");
                if sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
                    .bind(&key)
                    .bind(&image_data)
                    .execute(&data.db)
                    .await
                    .is_ok()
                {
                    return HttpResponse::Ok().json(json!({"ok": true}));
                } else {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "Database error"}));
                }
            }
        }
    }
    HttpResponse::BadRequest().json(json!({"error": "Upload failed"}))
}

#[get("/api/avatar/{username}")]
pub async fn get_avatar(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let username = path.into_inner();
    let key = format!("avatar:{username}");
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        let ct = if val.starts_with(&[0xFF, 0xD8, 0xFF]) {
            "image/jpeg"
        } else if val.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            "image/png"
        } else if val.starts_with(&[0x47, 0x49, 0x46]) {
            "image/gif"
        } else {
            "application/octet-stream"
        };
        return HttpResponse::Ok().content_type(ct).body(val);
    }
    let initial = username.chars().next().unwrap_or('?').to_uppercase();
    let svg = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 100\" fill=\"#23283a\"><rect width=\"100\" height=\"100\"/><text x=\"50\" y=\"65\" font-size=\"50\" text-anchor=\"middle\" fill=\"#7fffd4\" font-family=\"sans-serif\">{}</text></svg>", initial);
    HttpResponse::Ok().content_type("image/svg+xml").body(svg)
}

#[get("/api/profile")]
pub async fn get_profile(data: web::Data<AppState>, session: Session) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"})),
    };

    let user_key = format!("user:{username}");
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            return HttpResponse::Ok().json(json!({
                "username": username,
                "email": user_json["email"],
                "role": user_json["role"]
            }));
        }
    }
    HttpResponse::NotFound().json(json!({"error": "User not found"}))
}

#[derive(Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
}

#[post("/api/delete_account")]
pub async fn delete_account(
    data: web::Data<AppState>,
    req: web::Json<DeleteAccountRequest>,
    session: Session,
) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"})),
    };

    if username == "admin" {
        return HttpResponse::BadRequest()
            .json(json!({"error": "Cannot delete root admin account"}));
    }

    let user_key = format!("user:{username}");
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            // Verify password
            let stored_hash = user_json["password"].as_str().unwrap_or("");
            let parsed_hash = match PasswordHash::new(stored_hash) {
                Ok(h) => h,
                Err(_) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "Invalid password hash"}));
                }
            };

            if Argon2::default()
                .verify_password(req.password.as_bytes(), &parsed_hash)
                .is_err()
            {
                return HttpResponse::BadRequest().json(json!({"error": "Incorrect password"}));
            }

            // Delete user data
            let keys_to_delete = vec![
                format!("user:{}", username),
                format!("avatar:{}", username),
                format!("verify_token:{}", username),
                format!("reset_token:{}", username),
            ];

            for key in keys_to_delete {
                let _ = sqlx::query("DELETE FROM jeebs_store WHERE key = ?")
                    .bind(key)
                    .execute(&data.db)
                    .await;
            }

            crate::logging::log(
                &data.db,
                "WARN",
                "AUTH",
                &format!("User {username} deleted their own account"),
            )
            .await;
            session.purge();
            return HttpResponse::Ok().json(json!({"ok": true}));
        }
    }
    HttpResponse::InternalServerError().json(json!({"error": "User not found"}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cortex::Cortex;
    use crate::plugins::ErrorPlugin;
    use crate::plugins::Plugin;
    use actix_web::web;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex, RwLock};
    use sysinfo::System;

    #[tokio::test]
    async fn test_evolution_error_reporting() {
        // 1. Setup in-memory DB
        let db = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query("CREATE TABLE jeebs_store (key TEXT PRIMARY KEY, value BLOB)")
            .execute(&db)
            .await
            .unwrap();

        // 2. Setup AppState with ErrorPlugin
        let plugins: Vec<Box<dyn Plugin>> = vec![Box::new(ErrorPlugin)];
        let state = web::Data::new(AppState {
            db: db.clone(),
            plugins,
            ip_blacklist: Arc::new(RwLock::new(HashSet::new())),
            ip_whitelist: Arc::new(RwLock::new(HashSet::new())),
            sys: Arc::new(Mutex::new(System::new_all())),
        });

        // 3. Trigger the error via Cortex
        let response = Cortex::think("trigger error", &state).await;
        assert!(response.starts_with("Error:"));

        // 4. Verify Evolution Update was created in DB
        let rows = sqlx::query("SELECT key FROM jeebs_store WHERE key LIKE 'evolution:update:%'")
            .fetch_all(&db)
            .await
            .unwrap();

        assert_eq!(
            rows.len(),
            1,
            "Should have created exactly one evolution update proposal"
        );
    }
}

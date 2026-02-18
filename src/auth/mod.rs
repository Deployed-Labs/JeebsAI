use crate::plugins::Plugin;
use crate::state::AppState;
use crate::utils::{decode_all, encode_all};
use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::Local;
use futures_util::TryStreamExt;
use rand_core::OsRng;
use serde::Deserialize;
use serde_json::json;
use sqlx::{Row, SqlitePool};

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
                    eprintln!("Failed to hash admin password: {}", e);
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
                    eprintln!("Failed to create admin user: {}", e);
                    return;
                }
                println!(
                    "\n!!! IMPORTANT !!!\nAdmin account created.\nUsername: admin\nPassword: {}\n!!! SAVE THIS PASSWORD !!!\n",
                    new_password
                );
            }
        }
        Ok(Some(_)) => {}
        Err(e) => eprintln!("Failed to check for admin user: {}", e),
    }
}

pub async fn ensure_user(db: &SqlitePool, username: &str, password: &str, role: &str) {
    let key = format!("user:{}", username);
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
                    eprintln!("Failed to hash password: {}", e);
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
                    eprintln!("Failed to create user {}: {}", username, e);
                    return;
                }
                println!("User account created: {}", username);
            }
        }
        Ok(Some(_)) => {}
        Err(e) => eprintln!("Failed to check for user {}: {}", username, e),
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

    let rate_limit_key = format!("ratelimit:login:{}", ip);
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
                        &format!("Rate limit exceeded for IP: {}", ip),
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
                    let _ = session.renew();
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

#[post("/api/logout")]
pub async fn logout(session: Session) -> impl Responder {
    session.purge();
    HttpResponse::Ok().json(json!({"ok": true}))
}

// The Central Nervous System of Jeebs
pub struct Cortex;

impl Cortex {
    pub async fn think(prompt: &str, data: &web::Data<AppState>) -> String {
        let db = &data.db;
        let prompt_lower = prompt.to_lowercase();

        crate::logging::log(
            db,
            "INFO",
            "CORTEX",
            &format!("Processing thought: {}", prompt),
        )
        .await;

        // --- Layer 0: Deja Vu (Cache Check) ---
        if let Some(cached) = check_dejavu(prompt, db).await {
            return cached;
        }

        // --- Layer 1: Reflexes (Fast, hardcoded responses) ---
        if let Some(reflex) = check_reflexes(&prompt_lower) {
            return reflex;
        }

        // --- Layer 2: Short-term Memory (Context) ---
        if prompt_lower == "what did i just say" {
            return retrieve_last_prompt(db).await;
        }

        // --- Layer 3: Intent Router (Scored Execution) ---
        // Score plugins based on the prompt to prioritize the best match
        let mut scored_plugins: Vec<_> = data
            .plugins
            .iter()
            .map(|p| (p, score_intent(p.name(), &prompt_lower)))
            .collect();

        scored_plugins.sort_by(|a, b| b.1.cmp(&a.1));

        for (plugin, _score) in scored_plugins {
            if let Some(resp) = plugin.handle(prompt.to_string(), db.clone()).await {
                if resp.starts_with("Error:") {
                    report_error_to_evolution(db, plugin.name(), &resp).await;
                    crate::logging::log(
                        db,
                        "ERROR",
                        "PLUGIN",
                        &format!("Plugin {} failed: {}", plugin.name(), resp),
                    )
                    .await;
                }

                // Subconscious: We could spawn a background task here to analyze the interaction
                let db_clone = db.clone();
                let prompt_clone = prompt.to_string();
                let resp_clone = resp.clone();
                tokio::spawn(async move {
                    subconscious_process(prompt_clone, resp_clone, db_clone).await;
                });
                save_memory(prompt, &resp, db).await;
                return resp;
            }
        }

        // --- Layer 4: Cognition (Deep Thinking / Fallback) ---
        // Store the current thought for context
        store_context(prompt, db).await;

        let response = custom_ai_logic(prompt);

        let db_clone = db.clone();
        let prompt_clone = prompt.to_string();
        let resp_clone = response.clone();
        tokio::spawn(async move {
            subconscious_process(prompt_clone, resp_clone, db_clone).await;
        });
        save_memory(prompt, &response, db).await;

        response
    }
}

fn check_reflexes(prompt: &str) -> Option<String> {
    if prompt.contains("hello") || prompt.contains("hi ") || prompt == "hi" {
        return Some("Hello! I'm Jeebs. How can I help you today?".to_string());
    }
    None
}

async fn retrieve_last_prompt(db: &SqlitePool) -> String {
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = 'last_prompt'")
        .fetch_optional(db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(decompressed) = decode_all(&val) {
            if let Ok(text) = String::from_utf8(decompressed) {
                return format!("You just said: '{}'.", text);
            }
        }
    }
    "I don't have any previous input from you yet.".to_string()
}

async fn store_context(prompt: &str, db: &SqlitePool) {
    if let Ok(encoded) = encode_all(prompt.as_bytes(), 1) {
        let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
            .bind("last_prompt")
            .bind(encoded)
            .execute(db)
            .await;
    }
}

fn custom_ai_logic(prompt: &str) -> String {
    // This is where we would connect to an LLM or more complex internal logic
    format!("I'm not sure how to respond to: '{}'.", prompt)
}

async fn subconscious_process(prompt: String, response: String, _db: SqlitePool) {
    // This runs in the background after a response is sent.
    // It can be used for sentiment analysis, self-correction, or memory consolidation.
    println!(
        "[Subconscious] Reflecting on: '{}' -> '{}'",
        prompt, response
    );
}

fn score_intent(plugin_name: &str, prompt: &str) -> i32 {
    match plugin_name {
        "Time" => {
            if prompt.contains("time") || prompt.contains("clock") {
                100
            } else {
                0
            }
        }
        "Calc" => {
            if prompt.contains("math") || prompt.contains("calc") || prompt.contains("+") {
                100
            } else {
                0
            }
        }
        "Weather" => {
            if prompt.contains("weather") || prompt.contains("rain") {
                100
            } else {
                0
            }
        }
        "News" => {
            if prompt.contains("news") || prompt.contains("headline") {
                100
            } else {
                0
            }
        }
        "System" => {
            if prompt.contains("system") || prompt.contains("cpu") || prompt.contains("ram") {
                100
            } else {
                0
            }
        }
        _ => 1, // Default low priority
    }
}

async fn check_dejavu(prompt: &str, db: &SqlitePool) -> Option<String> {
    let key = blake3::hash(prompt.as_bytes()).to_hex().to_string();
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(key)
        .fetch_optional(db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(decompressed) = decode_all(&val) {
            if let Ok(text) = String::from_utf8(decompressed) {
                return Some(format!("[Deja Vu] {}", text));
            }
        }
    }
    None
}

async fn save_memory(prompt: &str, response: &str, db: &SqlitePool) {
    let key = blake3::hash(prompt.as_bytes()).to_hex().to_string();
    if let Ok(compressed) = encode_all(response.as_bytes(), 1) {
        let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(compressed)
            .execute(db)
            .await;
    }
}

async fn report_error_to_evolution(db: &SqlitePool, plugin_name: &str, error: &str) {
    let id = uuid::Uuid::new_v4().to_string();
    let title = format!("Auto-Fix: {} Error", plugin_name);
    let description = format!(
        "The {} plugin reported an error: '{}'. I should investigate and fix this.",
        plugin_name, error
    );

    crate::logging::log(
        db,
        "WARN",
        "EVOLUTION",
        &format!("Reporting error for evolution: {}", title),
    )
    .await;

    // Create a proposal entry directly in the store
    let update_json = json!({
        "id": id,
        "title": title,
        "author": "Jeebs (Auto-Fix)",
        "severity": "High",
        "comments": [],
        "description": description,
        "changes": [], // No automated changes; manual intervention required.
        "status": "pending",
        "created_at": Local::now().to_rfc3339(),
        "backup": null
    });

    let key = format!("evolution:update:{}", id);
    if let Ok(json_bytes) = serde_json::to_vec(&update_json) {
        if let Ok(val) = encode_all(&json_bytes, 1) {
            let _ = sqlx::query("INSERT INTO jeebs_store (key, value) VALUES (?, ?)")
                .bind(key)
                .bind(val)
                .execute(db)
                .await;
        }
    }

    // Create Notification for High Severity
    if update_json["severity"] == "High" {
        let notif_id = uuid::Uuid::new_v4().to_string();
        let notif_json = json!({
            "id": notif_id,
            "message": format!("High Severity Update Proposed: {}", title),
            "severity": "High",
            "created_at": Local::now().to_rfc3339(),
            "link": id
        });
        let notif_key = format!("notification:{}", notif_id);
        if let Ok(val) = encode_all(&serde_json::to_vec(&notif_json).unwrap(), 1) {
            let _ = sqlx::query("INSERT INTO jeebs_store (key, value) VALUES (?, ?)")
                .bind(notif_key)
                .bind(val)
                .execute(db)
                .await;
        }
    }
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

    let user_key = format!("user:{}", username);
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
                        &format!("User {} changed password", username),
                    )
                    .await;
                    return HttpResponse::Ok().json(json!({"ok": true}));
                }
            }
        }
    }
    HttpResponse::InternalServerError().json(json!({"error": "User not found"}))
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
                let key = format!("avatar:{}", username);
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
    let key = format!("avatar:{}", username);
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
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" fill="#23283a"><rect width="100" height="100"/><text x="50" y="65" font-size="50" text-anchor="middle" fill="#7fffd4" font-family="sans-serif">{}</text></svg>"##,
        username.chars().next().unwrap_or('?').to_uppercase()
    );
    HttpResponse::Ok().content_type("image/svg+xml").body(svg)
}

#[get("/api/profile")]
pub async fn get_profile(data: web::Data<AppState>, session: Session) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"})),
    };

    let user_key = format!("user:{}", username);
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

    let user_key = format!("user:{}", username);
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
                &format!("User {} deleted their own account", username),
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
    use crate::plugins::ErrorPlugin;
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

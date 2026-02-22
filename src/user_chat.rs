use actix_session::Session;
use actix_web::{get, options, post, web, HttpRequest, HttpResponse, Responder};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use std::env;

use crate::logging;
use crate::state::AppState;

const DEFAULT_JWT_SECRET: &str = "jeebs-secret-key-change-in-production";

#[derive(Debug, Deserialize, Serialize, Clone)]
struct TokenClaims {
    username: String,
    is_admin: bool,
    iat: i64,
    exp: i64,
}

#[derive(Deserialize)]
pub struct UserChatRequest {
    pub message: String,
}

#[derive(Serialize)]
pub struct UserChatResponse {
    pub response: String,
    pub username: String,
    pub is_admin: bool,
    pub is_trainer: bool,
}

/// Check if user is authenticated (logged in)
fn is_user_authenticated(session: &Session) -> bool {
    session
        .get::<bool>("logged_in")
        .ok()
        .flatten()
        .unwrap_or(false)
}

/// Check if user is root admin
fn is_root_admin(session: &Session) -> bool {
    let logged_in = is_user_authenticated(session);
    if !logged_in {
        return false;
    }

    let is_admin = session
        .get::<bool>("is_admin")
        .ok()
        .flatten()
        .unwrap_or(false);
    if !is_admin {
        return false;
    }

    session
        .get::<String>("username")
        .ok()
        .flatten()
        .map(|u| u == crate::auth::ROOT_ADMIN_USERNAME)
        .unwrap_or(false)
}

fn is_trainer(session: &Session) -> bool {
    let logged_in = is_user_authenticated(session);
    if !logged_in {
        return false;
    }

    session
        .get::<bool>("is_trainer")
        .ok()
        .flatten()
        .unwrap_or(false)
}

/// Get username from session
fn get_username(session: &Session) -> Option<String> {
    session.get::<String>("username").ok().flatten()
}

fn extract_bearer_claims(http_req: &HttpRequest) -> Option<TokenClaims> {
    let auth_header = http_req.headers().get("authorization")?.to_str().ok()?;
    let token = auth_header.strip_prefix("Bearer ")?;
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| DEFAULT_JWT_SECRET.to_string());
    let mut validation = Validation::default();
    validation.validate_exp = false;
    let decoded = decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .ok()?;
    Some(decoded.claims)
}

/// Get peer IP address
fn peer_addr(http_req: &HttpRequest) -> String {
    http_req
        .peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// User-friendly chat endpoint (requires authentication)
#[post("/api/chat")]
pub async fn user_chat(
    data: web::Data<AppState>,
    req: web::Json<UserChatRequest>,
    session: Session,
    http_req: HttpRequest,
) -> impl Responder {
    // Verify user is authenticated (session or bearer token)
    if !is_user_authenticated(&session) {
        if let Some(claims) = extract_bearer_claims(&http_req) {
            let role = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
                .bind(format!("user:{}", claims.username))
                .fetch_optional(&data.db)
                .await
                .ok()
                .and_then(|row| row.map(|r| r.get::<Vec<u8>, _>(0)))
                .and_then(|raw| serde_json::from_slice::<serde_json::Value>(&raw).ok())
                .and_then(|json| {
                    json.get("role")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "user".to_string());
            let is_trainer = role == "trainer";

            let _ = session.insert("logged_in", true);
            let _ = session.insert("username", &claims.username);
            let _ = session.insert("role", &role);
            let _ = session.insert("is_admin", claims.is_admin);
            let _ = session.insert("is_trainer", is_trainer);
        }
    }

    if !is_user_authenticated(&session) {
        logging::log(
            &data.db,
            "WARN",
            "CHAT",
            &format!(
                "Rejected chat request from unauthenticated user ip={}",
                peer_addr(&http_req)
            ),
        )
        .await;
        return HttpResponse::Unauthorized().json(json!({
            "error": "Not logged in. Please register and log in using PGP authentication."
        }));
    }

    let username = match get_username(&session) {
        Some(u) => u,
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Unable to retrieve username"
            }));
        }
    };

    let is_admin = is_root_admin(&session);
    let is_trainer = is_trainer(&session);
    let message = req.message.trim();

    if message.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "error": "Message cannot be empty"
        }));
    }

    // Log chat message
    logging::log(
        &data.db,
        "INFO",
        "CHAT",
        &format!("User {} sent message: {}", username, message),
    )
    .await;

    // Google learning command for all authenticated users (requires internet enabled)
    if let Some(query) = message.strip_prefix(".google").map(|s| s.trim()) {
        if query.is_empty() {
            return HttpResponse::BadRequest().json(json!({
                "error": "Usage: .google <your query>"
            }));
        }

        if !*data.internet_enabled.read().unwrap() {
            return HttpResponse::Forbidden().json(json!({
                "error": "Internet is disabled. Enable it in admin settings first."
            }));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(12))
            .user_agent("JeebsAI-Google/1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        match crate::question_learning::google_learn_and_store(&data.db, &client, query).await {
            Ok(summary) => {
                crate::logging::log(
                    &data.db,
                    "INFO",
                    "GOOGLE",
                    &format!("Google learned query='{}' by username={}", query, username),
                )
                .await;

                crate::logging::log(
                    &data.db,
                    "INFO",
                    "CHAT",
                    &format!("Jeebs reply to {} (.google): {}", username, summary),
                )
                .await;

                return HttpResponse::Ok().json(UserChatResponse {
                    response: format!(
                        "🔎 **Google Summary for** `{}`:\n\n{}\n\n✅ Stored in Jeebs brain for future recall.",
                        query, summary
                    ),
                    username,
                    is_admin,
                    is_trainer,
                });
            }
            Err(err) => {
                return HttpResponse::InternalServerError().json(json!({
                    "error": err
                }));
            }
        }
    }

    // Trainer commands: allow trainer group to trigger training focus
    if is_admin || is_trainer {
        if message.eq_ignore_ascii_case("train help") {
            return HttpResponse::Ok().json(UserChatResponse {
                response: "🎯 **Trainer Commands**:\n\n\
                    • `train: <topic>` - Set training focus topic\n\
                    • `train on` - Enable training mode\n\
                    • `train off` - Disable training mode\n\n\
                    Example: `train: improve rust error handling`"
                    .to_string(),
                username,
                is_admin,
                is_trainer,
            });
        }

        if let Some(topic) = message.strip_prefix("train:").map(|s| s.trim()) {
            if topic.is_empty() {
                return HttpResponse::BadRequest().json(json!({
                    "error": "Train command requires a topic. Example: train: improve rust error handling"
                }));
            }

            if let Err(err) =
                crate::cortex::set_training_focus_for_trainer(&data.db, topic, &username).await
            {
                return HttpResponse::InternalServerError().json(json!({
                    "error": err
                }));
            }

            crate::logging::log(
                &data.db,
                "INFO",
                "TRAINER",
                &format!("Trainer set focus username={} topic={}", username, topic),
            )
            .await;

            crate::logging::log(
                &data.db,
                "INFO",
                "CHAT",
                &format!(
                    "Jeebs reply to {} (trainer focus): Training focus set to {}",
                    username, topic
                ),
            )
            .await;

            return HttpResponse::Ok().json(UserChatResponse {
                response: format!(
                    "✅ Training focus set to: {}. Jeebs will prioritize this task in the next training cycle.",
                    topic
                ),
                username,
                is_admin,
                is_trainer,
            });
        }

        if message.eq_ignore_ascii_case("train stop") || message.eq_ignore_ascii_case("train off") {
            if let Err(err) =
                crate::cortex::set_training_enabled_for_trainer(&data.db, false, &username).await
            {
                return HttpResponse::InternalServerError().json(json!({
                    "error": err
                }));
            }

            crate::logging::log(
                &data.db,
                "INFO",
                "TRAINER",
                &format!("Trainer disabled training username={}", username),
            )
            .await;

            crate::logging::log(
                &data.db,
                "INFO",
                "CHAT",
                &format!(
                    "Jeebs reply to {} (trainer toggle): Training disabled",
                    username
                ),
            )
            .await;

            return HttpResponse::Ok().json(UserChatResponse {
                response: "🛑 Training disabled by trainer command.".to_string(),
                username,
                is_admin,
                is_trainer,
            });
        }

        if message.eq_ignore_ascii_case("train on") || message.eq_ignore_ascii_case("train start") {
            if let Err(err) =
                crate::cortex::set_training_enabled_for_trainer(&data.db, true, &username).await
            {
                return HttpResponse::InternalServerError().json(json!({
                    "error": err
                }));
            }

            crate::logging::log(
                &data.db,
                "INFO",
                "TRAINER",
                &format!("Trainer enabled training username={}", username),
            )
            .await;

            crate::logging::log(
                &data.db,
                "INFO",
                "CHAT",
                &format!(
                    "Jeebs reply to {} (trainer toggle): Training enabled",
                    username
                ),
            )
            .await;

            return HttpResponse::Ok().json(UserChatResponse {
                response: "▶️ Training enabled by trainer command.".to_string(),
                username,
                is_admin,
                is_trainer,
            });
        }
    }

    // Get response from Jeebs
    let response = crate::cortex::Cortex::think(message, &data).await;

    logging::log(
        &data.db,
        "INFO",
        "CHAT",
        &format!("Jeebs reply to {}: {}", username, response),
    )
    .await;

    // Persist a lightweight reasoning trace for analysis (best-effort)
    let _ = crate::logging::record_reasoning_trace(
        &data.db,
        Some(&username),
        message,
        &response,
        Some("{\"source\":\"cortex::think\"}"),
    )
    .await;

    HttpResponse::Ok().json(UserChatResponse {
        response,
        username,
        is_admin,
        is_trainer,
    })
}

#[options("/api/chat")]
pub async fn chat_preflight() -> impl Responder {
    HttpResponse::Ok().finish()
}

/// Get chat status (check if user is authenticated)
#[get("/api/chat/status")]
pub async fn chat_status(session: Session) -> impl Responder {
    if !is_user_authenticated(&session) {
        return HttpResponse::Ok().json(json!({
            "authenticated": false,
            "username": null,
            "message": "Not logged in"
        }));
    }

    let username = get_username(&session);
    let is_admin = is_root_admin(&session);
    let is_trainer = is_trainer(&session);

    HttpResponse::Ok().json(json!({
        "authenticated": true,
        "username": username,
        "is_admin": is_admin,
        "is_trainer": is_trainer,
        "message": "Ready to chat!"
    }))
}

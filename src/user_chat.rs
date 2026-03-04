use actix_session::Session;
use actix_web::{get, options, post, web, HttpRequest, HttpResponse, Responder};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use std::env;

use crate::logging;
use crate::chat_history;
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
            let is_trainer = role == "trainer" || claims.username == "peaci";

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

    // Store user message in chat_history
    let session_id = session.get::<String>("session_id").ok().flatten();
    let _ = chat_history::insert_chat_message(
        &data.db,
        session_id.as_deref(),
        Some(&username),
        "user",
        message
    ).await;

    // EARLY EXIT: Handle simple greetings without expensive processing
    if is_simple_greeting(message) {
        let greeting_response = handle_greeting(message);

        let _ = chat_history::insert_chat_message(
            &data.db,
            session_id.as_deref(),
            None,
            "jeebs",
            &greeting_response
        ).await;

        return HttpResponse::Ok().json(UserChatResponse {
            response: greeting_response,
            username,
            is_admin,
            is_trainer,
        });
    }

    // Google learning command for all authenticated users (requires internet enabled)
    if let Some(query) = message.strip_prefix(".google").map(|s| s.trim()) {
        if query.is_empty() {
            return HttpResponse::BadRequest().json(json!({
                "error": "Usage: .google <your query>"
            }));
        }

        // ENFORCE: If internet is OFF, no access
        if !*data.internet_enabled.read().unwrap() {
            return HttpResponse::Forbidden().json(json!({
                "error": "Internet is disabled. Enable it in admin settings first."
            }));
        }
        // ENFORCE: If training is OFF, no autonomous learning allowed
        if let Ok((_, enabled)) = crate::toggle_manager::get_toggle_states(&data.db).await {
            if !enabled {
                return HttpResponse::Forbidden().json(json!({
                    "error": "Training is disabled. Enable it in admin settings first."
                }));
            }
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
            // ENFORCE: Only allow if training is currently enabled
            if let Ok((_, enabled)) = crate::toggle_manager::get_toggle_states(&data.db).await {
                if !enabled {
                    return HttpResponse::Forbidden().json(json!({
                        "error": "Training is already disabled."
                    }));
                }
            }
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
            // ENFORCE: Only allow if training is currently disabled
            if let Ok((_, enabled)) = crate::toggle_manager::get_toggle_states(&data.db).await {
                if enabled {
                    return HttpResponse::Forbidden().json(json!({
                        "error": "Training is already enabled."
                    }));
                }
            }
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
                response: "✅ Training enabled by trainer command.".to_string(),
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

    // Store Jeebs reply in chat_history
    let _ = chat_history::insert_chat_message(
        &data.db,
        session_id.as_deref(),
        Some(&username),
        "jeebs",
        &response
    ).await;

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


/// Fetch chat history for a user/session
#[get("/api/chat/history")]
// Return chat history; accepts optional limit query parameter
pub async fn chat_history_endpoint(
    data: web::Data<AppState>,
    session: Session,
    http_req: HttpRequest,
) -> impl Responder {
    let session_id = session.get::<String>("session_id").ok().flatten();
    let username = get_username(&session);
    // parse optional ?limit= from query string manually
    let limit = http_req
        .query_string()
        .split('&')
        .find_map(|kv| {
            let mut parts = kv.splitn(2, '=');
            if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                if k == "limit" {
                    return v.parse::<usize>().ok();
                }
            }
            None
        })
        .unwrap_or(20);
    let history = chat_history::fetch_chat_history(
        &data.db,
        session_id.as_deref(),
        username.as_deref(),
        limit,
    )
    .await;
    match history {
        Ok(messages) => HttpResponse::Ok().json(messages),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": format!("DB error: {}", e)})),
    }
}

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

/// Enhanced intelligent chat endpoint using inference engine
#[post("/api/chat/intelligent")]
pub async fn intelligent_chat(
    data: web::Data<AppState>,
    req: web::Json<UserChatRequest>,
    session: Session,
    _http_req: HttpRequest,
) -> impl Responder {
    // Verify authentication
    if !is_user_authenticated(&session) {
        return HttpResponse::Unauthorized().json(json!({
            "error": "Not authenticated"
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

    let message = req.message.trim();
    if message.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "error": "Message cannot be empty"
        }));
    }

    // EARLY EXIT: Handle simple greetings without inference
    if is_simple_greeting(message) {
        let greeting_response = handle_greeting(message);

        // Still record message in history
        let session_id = session.get::<String>("session_id").ok().flatten();
        let _ = chat_history::insert_chat_message(
            &data.db,
            session_id.as_deref(),
            Some(&username),
            "user",
            message,
        )
        .await;

        let _ = chat_history::insert_chat_message(
            &data.db,
            session_id.as_deref(),
            None,
            "jeebs",
            &greeting_response,
        )
        .await;

        return HttpResponse::Ok().json(json!({
            "response": greeting_response,
            "confidence": 1.0,
            "reasoning": "Simple greeting detected",
            "sources": vec!["greeting_handler".to_string()],
            "learned_concepts": vec![] as Vec<String>,
            "username": username,
        }));
    }

    // Build inference context from brain databases
    let chdsc = data.chdsc.read().unwrap().clone();
    match crate::intelligent_inference::build_context(&chdsc, &data.db, message, Some(&username)).await {
        Ok(context) => {
            // Perform intelligent inference with reasoning
            match crate::intelligent_inference::infer_response(&context).await {
                Ok(inference) => {
                    // Store chat message
                    let session_id = session.get::<String>("session_id").ok().flatten();
                    let _ = chat_history::insert_chat_message(
                        &data.db,
                        session_id.as_deref(),
                        Some(&username),
                        "user",
                        message,
                    )
                    .await;

                    // Store response in chat history
                    let _ = chat_history::insert_chat_message(
                        &data.db,
                        session_id.as_deref(),
                        None,
                        "jeebs",
                        &inference.response,
                    )
                    .await;

                    // Log learning outcome for continuous learning
                    let _ = crate::intelligent_inference::log_inference_outcome(
                        &data.db,
                        &inference,
                        None,
                    )
                    .await;

                    // Log the interaction
                    logging::log(
                        &data.db,
                        "INFO",
                        "INTELLIGENT_CHAT",
                        &format!(
                            "User {} confidence={:.0}% sources={}",
                            username,
                            inference.confidence * 100.0,
                            inference.sources.join(",")
                        ),
                    )
                    .await;

                    HttpResponse::Ok().json(json!({
                        "response": inference.response,
                        "confidence": inference.confidence,
                        "reasoning": inference.reasoning,
                        "sources": inference.sources,
                        "learned_concepts": inference.learned_concepts,
                        "username": username,
                    }))
                }
                Err(e) => {
                    logging::log(
                        &data.db,
                        "ERROR",
                        "INTELLIGENT_CHAT",
                        &format!("Inference error: {}", e),
                    )
                    .await;

                    HttpResponse::InternalServerError().json(json!({
                        "error": format!("Inference failed: {}", e)
                    }))
                }
            }
        }
        Err(e) => {
            logging::log(
                &data.db,
                "ERROR",
                "INTELLIGENT_CHAT",
                &format!("Context building error: {}", e),
            )
            .await;

            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to build context: {}", e)
            }))
        }
    }
}

/// Detect simple greetings (same as enhanced_chat)
fn is_simple_greeting(message: &str) -> bool {
    let lower_msg = message.to_lowercase();
    let lower = lower_msg.trim();
    let common_greetings = [
        "hello", "hi", "hey", "greetings", "howdy",
        "what's up", "whats up", "yo", "sup",
        "good morning", "good afternoon", "good evening",
        "morning", "afternoon", "evening",
        "how are you", "how're you", "how are you doing",
        "how do you do", "pleased to meet you"
    ];

    for greeting in &common_greetings {
        if lower == *greeting || lower.starts_with(&format!("{} ", greeting)) || lower.ends_with(&format!(" {}", greeting)) {
            return true;
        }
    }

    // Also detect very short messages that look like greetings
    lower.len() <= 10 && (
        lower.ends_with("?") && !lower.contains(" ")
        || lower == "hello?" || lower == "hi?" || lower == "hey?"
    )
}

/// Generate appropriate greeting response (same as enhanced_chat)
fn handle_greeting(message: &str) -> String {
    let lower_msg = message.to_lowercase();
    let lower = lower_msg.trim();

    // Generate contextual greetings
    let response = if lower.contains("morning") {
        "Good morning! What would you like to talk about today?"
    } else if lower.contains("afternoon") {
        "Good afternoon! How can I help you?"
    } else if lower.contains("evening") {
        "Good evening! What's on your mind?"
    } else if lower.contains("how are") || lower.contains("how're") {
        "I'm here and ready to help! What would you like to know?"
    } else if lower.contains("up") {
        "Not much! What would you like to talk about?"
    } else if lower.contains("hey") {
        "Hey there! What can I help you with?"
    } else {
        "Hello! I'm ready to learn and help. What's your question?"
    };

    response.to_string()
}

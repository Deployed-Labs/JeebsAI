use actix_session::Session;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use chrono::Local;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::io::Write;

use crate::cortex::Cortex;
use crate::language_learning::Thought;
use crate::logging;
use crate::state::AppState;

const DEFAULT_JWT_SECRET: &str = "jeebs-secret-key-change-in-production";

#[derive(Deserialize)]
struct JeebsRequest {
    prompt: String,
}

#[derive(Serialize)]
struct JeebsResponse {
    response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    thought: Option<Thought>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct TokenClaims {
    username: String,
    is_admin: bool,
    iat: i64,
    exp: i64,
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

fn peer_addr(http_req: &HttpRequest) -> String {
    http_req
        .peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[post("/api/jeebs")]
pub async fn jeebs_api(
    data: web::Data<AppState>,
    req: web::Json<JeebsRequest>,
    session: Session,
    http_req: HttpRequest,
) -> impl Responder {
    let mut logged_in = session
        .get::<bool>("logged_in")
        .unwrap_or(Some(false))
        .unwrap_or(false);
    let mut username = session.get::<String>("username").unwrap_or(None);

    if !logged_in || username.is_none() {
        if let Some(claims) = extract_bearer_claims(&http_req) {
            logged_in = true;
            username = Some(claims.username.clone());
            let _ = session.insert("logged_in", true);
            let _ = session.insert("username", &claims.username);
            let _ = session.insert("is_admin", claims.is_admin);
        }
    }

    if !logged_in {
        logging::log(
            &data.db,
            "WARN",
            "CHAT",
            &format!(
                "Rejected chat request from unauthenticated client ip={}",
                peer_addr(&http_req)
            ),
        )
        .await;
        return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"}));
    }
    let db = &data.db;
    let prompt = req.prompt.trim();
    let username_for_log = username
        .as_deref()
        .map(str::to_string)
        .unwrap_or_else(|| "unknown".to_string());
    let user_id = if let Some(uid) = session.get::<String>("user_id").unwrap_or(None) {
        uid
    } else {
        let new_id = uuid::Uuid::new_v4().to_string();
        let _ = session.insert("user_id", &new_id);
        new_id
    };

    // Update last_seen
    let ip = peer_addr(&http_req);
    let user_agent = http_req
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    let now = Local::now().to_rfc3339();

    if let Err(err) = sqlx::query(
        "INSERT OR REPLACE INTO user_sessions (username, ip, user_agent, last_seen) VALUES (?, ?, ?, ?)"
    )
        .bind(username.as_deref().unwrap_or(""))
        .bind(&ip)
        .bind(user_agent)
        .bind(&now)
        .execute(db)
        .await
    {
        logging::log(
            db,
            "WARN",
            "CHAT",
            &format!(
                "Failed to update user session username={} user_id={} reason={}",
                username_for_log, user_id, err
            ),
        )
        .await;
    }

    logging::log(
        db,
        "INFO",
        "CHAT",
        &format!(
            "Prompt received username={} user_id={} ip={} prompt_chars={}",
            username_for_log,
            user_id,
            peer_addr(&http_req),
            prompt.chars().count()
        ),
    )
    .await;

    println!(
        "[API] user_id={} username={:?} ip={} prompt_chars={}",
        user_id,
        username,
        peer_addr(&http_req),
        prompt.chars().count()
    );
    let result = Cortex::think_for_user(prompt, &data, &user_id, username.as_deref()).await;
    HttpResponse::Ok().json(JeebsResponse { 
        response: result.response,
        thought: result.thought,
    })
}

pub fn start_cli(data: web::Data<AppState>) {
    tokio::spawn(async move {
        let stdin = std::io::stdin();
        let mut input = String::new();
        loop {
            print!("Enter a prompt (or 'exit'): ");
            std::io::stdout().flush().unwrap();
            input.clear();
            stdin.read_line(&mut input).unwrap();
            let prompt = input.trim();
            if prompt == "exit" {
                break;
            }
            let response = Cortex::think(prompt, &data).await;
            println!("Jeebs: {response}");
        }
        println!("Goodbye from Jeebs!");
    });
}

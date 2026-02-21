use actix_session::Session;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use std::env;

use crate::state::AppState;

mod pgp;

const DEFAULT_JWT_SECRET: &str = "jeebs-secret-key-change-in-production";
pub const ROOT_ADMIN_USERNAME: &str = "1090mb";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub username: String,
    pub is_admin: bool,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: Option<String>,
    pub pgp_public_key: String,
}

#[derive(Debug, Deserialize)]
pub struct PgpLoginRequest {
    pub username: String,
    pub signed_message: String,
    pub remember_me: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct LoginAliasRequest {
    pub username: String,
    pub signed_message: Option<String>,
    pub remember_me: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthStatusResponse {
    pub logged_in: bool,
    pub username: Option<String>,
    pub is_admin: bool,
    pub token: Option<String>,
}

fn valid_username(username: &str) -> bool {
    let len = username.chars().count();
    if !(3..=32).contains(&len) {
        return false;
    }

    username
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

pub fn is_root_admin_session(session: &Session) -> bool {
    let logged_in = session
        .get::<bool>("logged_in")
        .ok()
        .flatten()
        .unwrap_or(false);
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
        .map(|u| u == ROOT_ADMIN_USERNAME)
        .unwrap_or(false)
}

fn issue_token(username: &str, is_admin: bool) -> Result<String, String> {
    let now = Utc::now().timestamp();
    let claims = TokenClaims {
        username: username.to_string(),
        is_admin,
        iat: now,
        exp: now + 60 * 60 * 24 * 30,
    };

    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| DEFAULT_JWT_SECRET.to_string());
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| format!("Failed to issue token: {e}"))
}

fn extract_bearer_claims(http_req: &HttpRequest) -> Option<TokenClaims> {
    let auth_header = http_req.headers().get("authorization")?.to_str().ok()?;
    let token = auth_header.strip_prefix("Bearer ")?;
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| DEFAULT_JWT_SECRET.to_string());
    let decoded = decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
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

async fn handle_pgp_login(
    data: &web::Data<AppState>,
    req: &PgpLoginRequest,
    http_req: &HttpRequest,
    session: &Session,
) -> HttpResponse {
    let username = req.username.trim();
    if !valid_username(username) {
        crate::logging::log(
            &data.db,
            "WARN",
            "AUTH",
            &format!(
                "Rejected login due to invalid username format from ip={} username={}",
                peer_addr(http_req),
                username
            ),
        )
        .await;
        return HttpResponse::BadRequest().json(json!({"error": "Invalid username format"}));
    }

    let user_key = format!("user:{username}");
    let row = match sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await
    {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({"error": "Database error"}));
        }
    };

    let Some(row) = row else {
        crate::logging::log(
            &data.db,
            "WARN",
            "AUTH",
            &format!(
                "Failed login for unknown username from ip={} username={}",
                peer_addr(http_req),
                username
            ),
        )
        .await;
        return HttpResponse::Unauthorized().json(json!({"error": "Unknown username"}));
    };

    let val: Vec<u8> = row.get(0);
    let user_json = match serde_json::from_slice::<serde_json::Value>(&val) {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Corrupted user record"}));
        }
    };

    if user_json.get("auth_type").and_then(|v| v.as_str()) != Some("pgp") {
        return HttpResponse::BadRequest().json(json!({
            "error": "This account does not support PGP login"
        }));
    }

    let public_key = match user_json.get("pgp_public_key").and_then(|v| v.as_str()) {
        Some(v) if !v.trim().is_empty() => v.trim(),
        _ => {
            return HttpResponse::BadRequest().json(json!({
                "error": "No PGP key registered for this account"
            }));
        }
    };

    let verified =
        match crate::auth::pgp::verify_signature_with_public_key(&req.signed_message, public_key) {
            Ok(v) => v,
            Err(e) => {
                crate::logging::log(
                    &data.db,
                    "WARN",
                    "AUTH",
                    &format!(
                        "PGP signature verification failed from ip={} username={} reason={}",
                        peer_addr(http_req),
                        username,
                        e
                    ),
                )
                .await;
                return HttpResponse::Unauthorized().json(json!({
                    "error": format!("Signature verification failed: {e}")
                }));
            }
        };

    let parts: Vec<&str> = verified.trim().split(':').collect();
    if parts.len() != 3 || parts[0] != "LOGIN" {
        return HttpResponse::BadRequest()
            .json(json!({"error": "Signed message format is invalid"}));
    }
    if parts[1] != username {
        return HttpResponse::BadRequest().json(json!({"error": "Signed username mismatch"}));
    }

    let ts = match parts[2].parse::<i64>() {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({"error": "Invalid signed timestamp"}));
        }
    };

    let now = Utc::now().timestamp();
    if ts < now - 300 || ts > now + 60 {
        crate::logging::log(
            &data.db,
            "WARN",
            "AUTH",
            &format!(
                "Rejected login due to stale timestamp from ip={} username={}",
                peer_addr(http_req),
                username
            ),
        )
        .await;
        return HttpResponse::Unauthorized().json(json!({
            "error": "Signed timestamp is outside the allowed window"
        }));
    }

    let role = user_json
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("user");
    let is_admin = role == "admin";

    let token = match issue_token(username, is_admin) {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e})),
    };

    if session.insert("logged_in", true).is_err()
        || session.insert("username", username).is_err()
        || session.insert("is_admin", is_admin).is_err()
        || session.insert("auth_token", &token).is_err()
    {
        return HttpResponse::InternalServerError().json(json!({"error": "Session error"}));
    }

    if req.remember_me.unwrap_or(false) {
        session.renew();
    }

    crate::logging::log(
        &data.db,
        "INFO",
        "AUTH",
        &format!(
            "Successful login username={} is_admin={} ip={}",
            username,
            is_admin,
            peer_addr(http_req)
        ),
    )
    .await;

    HttpResponse::Ok().json(json!({
        "status": "success",
        "username": username,
        "is_admin": is_admin,
        "token": token
    }))
}

#[post("/api/register")]
pub async fn register(
    data: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
    http_req: HttpRequest,
) -> impl Responder {
    let username = req.username.trim();
    if !valid_username(username) {
        return HttpResponse::BadRequest().json(json!({
            "error": "Username must be 3-32 chars and use only letters, numbers, '-' or '_'"
        }));
    }

    let public_key = req.pgp_public_key.trim();
    if public_key.is_empty() {
        return HttpResponse::BadRequest().json(json!({"error": "pgp_public_key is required"}));
    }
    if let Err(e) = crate::auth::pgp::validate_public_key(public_key) {
        return HttpResponse::BadRequest().json(json!({"error": e}));
    }

    let user_key = format!("user:{username}");
    match sqlx::query("SELECT 1 FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await
    {
        Ok(Some(_)) => {
            crate::logging::log(
                &data.db,
                "WARN",
                "AUTH",
                &format!(
                    "Registration conflict for existing username={} from ip={}",
                    username,
                    peer_addr(&http_req)
                ),
            )
            .await;
            return HttpResponse::Conflict().json(json!({"error": "Username already exists"}));
        }
        Ok(None) => {}
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({"error": "Database error"}));
        }
    }

    let email = req.email.as_deref().unwrap_or("").trim();
    let user_json = json!({
        "username": username,
        "email": email,
        "role": "user",
        "auth_type": "pgp",
        "pgp_public_key": public_key,
        "created_at": Utc::now().to_rfc3339(),
    });

    let user_bytes = match serde_json::to_vec(&user_json) {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to serialize user"}));
        }
    };

    if sqlx::query("INSERT INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(&user_key)
        .bind(user_bytes)
        .execute(&data.db)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().json(json!({"error": "Failed to create user"}));
    }

    crate::logging::log(
        &data.db,
        "INFO",
        "AUTH",
        &format!(
            "Registered new user username={} from ip={}",
            username,
            peer_addr(&http_req)
        ),
    )
    .await;

    HttpResponse::Created().json(json!({
        "status": "registered",
        "username": username
    }))
}

#[post("/api/login_pgp")]
pub async fn login_pgp(
    data: web::Data<AppState>,
    req: web::Json<PgpLoginRequest>,
    http_req: HttpRequest,
    session: Session,
) -> impl Responder {
    handle_pgp_login(&data, &req, &http_req, &session).await
}

#[post("/api/login")]
pub async fn login(
    data: web::Data<AppState>,
    req: web::Json<LoginAliasRequest>,
    http_req: HttpRequest,
    session: Session,
) -> impl Responder {
    let signed_message = match req.signed_message.as_deref() {
        Some(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => {
            return HttpResponse::BadRequest().json(json!({
                "error": "PGP login requires signed_message"
            }));
        }
    };

    let pgp_req = PgpLoginRequest {
        username: req.username.clone(),
        signed_message,
        remember_me: req.remember_me,
    };

    handle_pgp_login(&data, &pgp_req, &http_req, &session).await
}

#[get("/api/auth/status")]
pub async fn auth_status(session: Session, http_req: HttpRequest) -> impl Responder {
    let logged_in = session
        .get::<bool>("logged_in")
        .ok()
        .flatten()
        .unwrap_or(false);

    if !logged_in {
        if let Some(claims) = extract_bearer_claims(&http_req) {
            let bearer_token = http_req
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
                .map(|s| s.to_string());

            let _ = session.insert("logged_in", true);
            let _ = session.insert("username", &claims.username);
            let _ = session.insert("is_admin", claims.is_admin);
            if let Some(token) = &bearer_token {
                let _ = session.insert("auth_token", token);
            }

            return HttpResponse::Ok().json(AuthStatusResponse {
                logged_in: true,
                username: Some(claims.username),
                is_admin: claims.is_admin,
                token: bearer_token,
            });
        }

        return HttpResponse::Ok().json(AuthStatusResponse {
            logged_in: false,
            username: None,
            is_admin: false,
            token: None,
        });
    }

    let username = session.get::<String>("username").ok().flatten();
    let is_admin = session
        .get::<bool>("is_admin")
        .ok()
        .flatten()
        .unwrap_or(false);
    let token = session.get::<String>("auth_token").ok().flatten();

    HttpResponse::Ok().json(AuthStatusResponse {
        logged_in: true,
        username,
        is_admin,
        token,
    })
}

#[post("/api/logout")]
pub async fn logout(
    data: web::Data<AppState>,
    session: Session,
    http_req: HttpRequest,
) -> impl Responder {
    let username = session.get::<String>("username").ok().flatten();
    let is_admin = session
        .get::<bool>("is_admin")
        .ok()
        .flatten()
        .unwrap_or(false);

    session.purge();

    crate::logging::log(
        &data.db,
        "INFO",
        "AUTH",
        &format!(
            "Logout username={} is_admin={} ip={}",
            username.unwrap_or_else(|| "unknown".to_string()),
            is_admin,
            peer_addr(&http_req)
        ),
    )
    .await;

    HttpResponse::Ok().json(json!({"status": "logged_out"}))
}

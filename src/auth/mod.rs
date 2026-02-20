use actix_web::{post, web, HttpResponse, Responder};
use actix_session::Session;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::Utc;
use crate::state::AppState;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;

mod pgp;

#[derive(Serialize, Deserialize)]
pub struct TokenClaims {
    pub username: String,
    pub is_admin: bool,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Deserialize)]
pub struct PgpLoginRequest {
    pub username: String,
    pub signed_message: String,
    pub remember_me: Option<bool>,
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
) -> impl Responder {
    let user_key = format!("user:{}", req.username);
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
        return HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"}));
    };

    let val: Vec<u8> = row.get(0);
    let user_json = match serde_json::from_slice::<serde_json::Value>(&val) {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"}));
        }
    };

    if user_json.get("auth_type").and_then(|v| v.as_str()) == Some("pgp") {
        return HttpResponse::BadRequest().json(json!({
            "error": "This account requires PGP authentication",
            "use_pgp": true
        }));
    }

    let stored_password = user_json
        .get("password")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if stored_password.is_empty() {
        return HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"}));
    }

    let password_valid = if stored_password.starts_with("$argon2") {
        match PasswordHash::new(stored_password) {
            Ok(parsed) => Argon2::default()
                .verify_password(req.password.as_bytes(), &parsed)
                .is_ok(),
            Err(_) => false,
        }
    } else {
        stored_password == req.password
    };

    if !password_valid {
        return HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"}));
    }

    if session.insert("username", &req.username).is_err() {
        return HttpResponse::InternalServerError().json(json!({"error": "Session error"}));
    }

    let role = user_json
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("user");
    let is_admin = role == "admin";
    if session.insert("is_admin", is_admin).is_err() {
        return HttpResponse::InternalServerError().json(json!({"error": "Session error"}));
    }

    if req.remember_me.unwrap_or(false) {
        session.renew();
    }

    HttpResponse::Ok().json(json!({
        "status": "success",
        "username": req.username,
        "is_admin": is_admin
    }))
}

#[post("/api/login_pgp")]
pub async fn login_pgp(
    data: web::Data<AppState>,
    req: web::Json<PgpLoginRequest>,
    session: Session,
) -> impl Responder {
    // Verify PGP signature using the helper in src/auth/pgp.rs
    let verified = match crate::auth::pgp::verify_signature(&req.signed_message) {
        Ok(s) => s,
        Err(e) => return HttpResponse::Unauthorized().json(json!({"error": format!("Signature verification failed: {}", e)})),
    };

    // Expect format: LOGIN:username:timestamp
    let parts: Vec<&str> = verified.trim().split(':').collect();
    if parts.len() != 3 || parts[0] != "LOGIN" {
        return HttpResponse::BadRequest().json(json!({"error": "Signed message has invalid format"}));
    }
    let username = parts[1];
    if username != req.username {
        return HttpResponse::BadRequest().json(json!({"error": "Signed username mismatch"}));
    }

    let ts = match parts[2].parse::<i64>() {
        Ok(v) => v,
        Err(_) => return HttpResponse::BadRequest().json(json!({"error": "Invalid timestamp in signed message"})),
    };

    let now = Utc::now().timestamp();
    if (now - ts).abs() > 300 {
        return HttpResponse::Unauthorized().json(json!({"error": "Signed message timestamp is outside allowed window"}));
    }

    // At this point the signature is valid and fresh. Grant session and admin if appropriate.
    let uname = req.username.clone();
    let _ = session.insert("username", uname.clone());

    // Determine admin status: if username == "1090mb" or user role in jeebs_store == "admin"
    let mut is_admin = false;
    if uname == "1090mb" {
        is_admin = true;
    } else {
        let user_key = format!("user:{}", uname);
        if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
            .bind(&user_key)
            .fetch_optional(&data.db)
            .await
        {
            let val: Vec<u8> = row.get(0);
            if let Ok(user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
                if user_json.get("role").and_then(|v| v.as_str()) == Some("admin") {
                    is_admin = true;
                }
            }
        }
    }

    let _ = session.insert("is_admin", is_admin);

    // Generate JWT token with user role
    let now = Utc::now().timestamp();
    let claims = TokenClaims {
        username: uname.clone(),
        is_admin,
        iat: now,
        exp: now + 86400 * 30, // 30 days
    };

    let secret = "jeebs-secret-key-change-in-production"; // TODO: use env var
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ).unwrap_or_default();

    HttpResponse::Ok().json(json!({
        "status": "success",
        "user": uname,
        "is_admin": is_admin,
        "token": token
    }))
}

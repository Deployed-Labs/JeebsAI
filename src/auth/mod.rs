use actix_web::{web, HttpResponse, Responder, post};
use actix_session::Session;
use serde::Deserialize;
use serde_json::json;
use crate::state::AppState;
use sqlx::Row;
use chrono::Utc;

mod pgp;

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

    HttpResponse::Ok().json(json!({"status": "success", "user": uname, "is_admin": is_admin}))
}

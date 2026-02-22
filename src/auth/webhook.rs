use actix_web::{web, HttpRequest, HttpResponse, post};
use serde::Deserialize;
use serde_json::json;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;
use std::process::Command;

type HmacSha256 = Hmac<Sha256>;

#[derive(Deserialize)]
pub struct GithubPushEvent {
    pub repository: GithubRepo,
    pub ref_: Option<String>,
    pub after: Option<String>,
}

#[derive(Deserialize)]
pub struct GithubRepo {
    pub full_name: String,
    pub clone_url: Option<String>,
}

/// GitHub webhook endpoint: pulls latest code on push, rebuilds, restarts service.
/// GitHub sends: POST /api/webhook/github with X-Hub-Signature-256 header.
/// Secret: read from GITHUB_WEBHOOK_SECRET env var (required).
#[post("/api/webhook/github")]
pub async fn github_webhook(
    req: HttpRequest,
    body: web::Bytes,
) -> impl actix_web::Responder {
    let secret = std::env::var("GITHUB_WEBHOOK_SECRET")
        .unwrap_or_else(|_| String::new());

    if secret.is_empty() {
        return HttpResponse::BadRequest().json(json!({"error": "GITHUB_WEBHOOK_SECRET not set"}));
    }

    // Verify signature: X-Hub-Signature-256 = sha256=<hex>
    let signature = match req.headers().get("X-Hub-Signature-256") {
        Some(h) => match h.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return HttpResponse::BadRequest().json(json!({"error": "invalid signature header"})),
        },
        None => return HttpResponse::BadRequest().json(json!({"error": "missing X-Hub-Signature-256"})),
    };

    let expected_sig = compute_signature(&body, secret.as_bytes());
    if !constant_time_compare(&signature, &expected_sig) {
        return HttpResponse::Unauthorized().json(json!({"error": "signature mismatch"}));
    }

    // Parse payload
    let payload = match serde_json::from_slice::<GithubPushEvent>(&body) {
        Ok(p) => p,
        Err(e) => return HttpResponse::BadRequest().json(json!({"error": format!("parse error: {}", e)})),
    };

    // Only pull if pushed to main branch
    let ref_str = payload.ref_.unwrap_or_default();
    if ref_str != "refs/heads/main" {
        return HttpResponse::Ok().json(json!({"status": "ignored (not main branch)"}));
    }

    // Pull latest code, rebuild, restart
    match pull_and_restart() {
        Ok(msg) => HttpResponse::Ok().json(json!({"status": "success", "message": msg})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e})),
    }
}

fn compute_signature(body: &[u8], secret: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).unwrap();
    mac.update(body);
    let result = mac.finalize();
    format!("sha256={}", hex::encode(result.into_bytes()))
}

fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (ca, cb) in a.bytes().zip(b.bytes()) {
        result |= ca ^ cb;
    }
    result == 0
}

fn pull_and_restart() -> Result<String, String> {
    let repo_dir = std::env::var("APP_DIR").unwrap_or_else(|_| "/root/JeebsAI".to_string());

    // Pull latest
    let pull_output = Command::new("git")
        .arg("pull")
        .current_dir(repo_dir)
        .output()
        .map_err(|e| format!("git pull failed: {}", e))?;

    if !pull_output.status.success() {
        let err = String::from_utf8_lossy(&pull_output.stderr);
        return Err(format!("git pull error: {}", err));
    }

    // Rebuild
    let build_output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(repo_dir)
        .output()
        .map_err(|e| format!("cargo build failed: {}", e))?;

    if !build_output.status.success() {
        let err = String::from_utf8_lossy(&build_output.stderr);
        return Err(format!("cargo build error: {}", err));
    }

    // Restart production service
    let restart_output = Command::new("systemctl")
        .args(&["restart", "jeebs"])
        .output()
        .map_err(|e| format!("systemctl restart failed: {}", e))?;

    if !restart_output.status.success() {
        let err = String::from_utf8_lossy(&restart_output.stderr);
        return Err(format!("systemctl restart error: {}", err));
    }

    Ok("pulled code, rebuilt, restarted jeebs".to_string())
}

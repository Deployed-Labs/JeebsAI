use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, post, web};
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;
use std::io::{self, Write};

use crate::cortex::Cortex;
use crate::state::AppState;

#[derive(Deserialize)]
struct JeebsRequest {
    prompt: String,
}

#[derive(Serialize)]
struct JeebsResponse {
    response: String,
}

#[post("/api/jeebs")]
pub async fn jeebs_api(
    data: web::Data<AppState>,
    req: web::Json<JeebsRequest>,
    session: Session,
    http_req: HttpRequest,
) -> impl Responder {
    let logged_in = session
        .get::<bool>("logged_in")
        .unwrap_or(Some(false))
        .unwrap_or(false);
    let username = session.get::<String>("username").unwrap_or(None);
    if !logged_in {
        return HttpResponse::Unauthorized().json(json!({"error": "Not logged in"}));
    }
    let db = &data.db;
    let prompt = req.prompt.trim();
    let user_id = if let Some(uid) = session.get::<String>("user_id").unwrap_or(None) {
        uid
    } else {
        let new_id = uuid::Uuid::new_v4().to_string();
        session.insert("user_id", &new_id).unwrap();
        new_id
    };

    // Update last_seen
    let _ = sqlx::query("UPDATE user_sessions SET last_seen = ? WHERE username = ?")
        .bind(Local::now().to_rfc3339())
        .bind(username.as_deref().unwrap_or(""))
        .execute(db)
        .await;

    println!(
        "[API] user_id={} username={:?} ip={:?} prompt=\"{}\"",
        user_id,
        username,
        http_req.peer_addr(),
        prompt
    );

    let response = Cortex::think(prompt, &data).await;
    HttpResponse::Ok().json(JeebsResponse { response })
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
            println!("Jeebs: {}", response);
        }
        println!("Goodbye from Jeebs!");
    });
}

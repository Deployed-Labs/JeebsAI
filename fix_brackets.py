import re

path = "/root/JeebsAI/src/auth/mod.rs"
with open(path, "r") as f:
    content = f.read()

# This replaces the entire login function with a clean, plaintext, bracket-perfect version
new_login_fn = """
pub async fn login(
    data: web::Data<AppState>,
    req: web::Json<LoginRequest>,
    session: Session,
    _http_req: HttpRequest,
) -> impl Responder {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&req.username)
        .fetch_optional(&data.db)
        .await;

    match user {
        Ok(Some(user)) => {
            if req.password == user.password {
                if let Err(e) = session.insert("user_id", user.id) {
                    return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Session error"}));
                }
                HttpResponse::Ok().json(serde_json::json!({"token": "session_active", "user": user.username}))
            } else {
                HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid credentials"}))
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(serde_json::json!({"error": "User not found"})),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "Database error"})),
    }
}
"""

# Replace the messy old login function with the clean one
# This uses a regex to find the start of the function and matches until a safe point
content = re.sub(r'pub async fn login\(.*?\)\s*->\s*impl Responder\s*\{.*?\}\n\n' , new_login_fn + "\n\n", content, flags=re.DOTALL)

with open(path, "w") as f:
    f.write(content)

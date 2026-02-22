import re

import os
APP_DIR = os.environ.get("APP_DIR", "/root/JeebsAI")
path = os.path.join(APP_DIR, "src/auth/mod.rs")
with open(path, "r") as f:
    lines = f.readlines()

# We will reconstruct the file by keeping the imports but replacing the broken logic
new_content = []
for line in lines:
    if line.startswith("use "):
        # Keep imports but ignore Argon2/Hashing ones
        if "argon2" not in line and "password_hash" not in line:
            new_content.append(line)
    if "struct" in line or "enum" in line:
        new_content.append(line)

# Add back the clean, plaintext login function
new_content.append("""
pub async fn login(
    data: web::Data<AppState>,
    req: web::Json<LoginRequest>,
    session: Session,
) -> impl Responder {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&req.username)
        .fetch_optional(&data.db)
        .await;

    match user {
        Ok(Some(user)) => {
            if req.password == user.password {
                let _ = session.insert("user_id", user.id);
                HttpResponse::Ok().json(serde_json::json!({"status": "success", "user": user.username}))
            } else {
                HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid password"}))
            }
        }
        _ => HttpResponse::Unauthorized().json(serde_json::json!({"error": "User not found"})),
    }
}
""")

with open(path, "w") as f:
    f.writelines(new_content)

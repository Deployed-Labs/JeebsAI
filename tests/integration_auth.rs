use actix_web::{App, test, web};
use jeebs::auth::TokenClaims;
use jeebs::state::AppState;
use jeebs::brain::coded_holographic_data_storage_container::CodedHolographicDataStorageContainer;
use sqlx::SqlitePool;
use std::env;
use std::sync::{Arc, RwLock};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;

#[actix_web::test]
async fn test_auth_session_and_admin_with_bearer() {
    // set JWT secret for deterministic token
    env::set_var("JWT_SECRET", "test-jwt-secret");

    // create temp sqlite file
    let db = SqlitePool::connect("sqlite::memory:").await.expect("create pool");

    // create minimal jeebs_store table
    sqlx::query("CREATE TABLE IF NOT EXISTS jeebs_store (key TEXT PRIMARY KEY, value BLOB)")
        .execute(&db).await.expect("create table");

    // insert a test admin user record
    let user = json!({
        "username": "alice",
        "email": "alice@example.com",
        "role": "admin",
        "auth_type": "pgp",
    });
    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind("user:alice")
        .bind(serde_json::to_vec(&user).unwrap())
        .execute(&db).await.expect("insert user");

    // build minimal AppState
    let state = AppState {
        db: db.clone(),
        plugins: Vec::new(),
        ip_blacklist: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashSet::new())),
        ip_whitelist: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashSet::new())),
        sys: std::sync::Arc::new(std::sync::Mutex::new(sysinfo::System::new_all())),
        internet_enabled: std::sync::Arc::new(std::sync::RwLock::new(true)),
        chdsc: Arc::new(RwLock::new(CodedHolographicDataStorageContainer::new())),
    };

    // create JWT for alice
    let now = chrono::Utc::now().timestamp();
    let claims = TokenClaims { username: "alice".to_string(), is_admin: true, iat: now, exp: now + 3600 };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(b"test-jwt-secret")).unwrap();

    // init service with just the auth endpoints we need
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(jeebs::auth::auth_session)
            .service(jeebs::admin::user::admin_list_users)
    ).await;

    // call /api/auth/session with bearer token
    let req = test::TestRequest::get()
        .uri("/api/auth/session")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("identity").is_some());
    assert_eq!(body["identity"]["username"], "alice");

    // call admin list users (should succeed for admin bearer)
    let req2 = test::TestRequest::get()
        .uri("/api/admin/users")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp2 = test::call_service(&app, req2).await;
    assert!(resp2.status().is_success());
}

use jeebs::{auth, AppState};
use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use sqlx::SqlitePool;
use std::env;
use std::path::Path;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./jeebs.db".to_string());

    // Ensure the SQLite directory exists if using a file path
    if let Some(path_str) = database_url.strip_prefix("sqlite:") {
        let db_path = path_str.trim_start_matches('/');
        let path = Path::new(db_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
    }

    let pool: SqlitePool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(database_url)
        .await
        .expect("DB Fail");

    // Apply any pending migrations at startup so offline-built binaries catch up when DB is available.
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("Failed to run migrations: {e}");
    }

    let state = web::Data::new(AppState {
        db: pool,
        plugins: vec![],
        ip_blacklist: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashSet::new())),
        ip_whitelist: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashSet::new())),
        sys: std::sync::Arc::new(std::sync::Mutex::new(sysinfo::System::new_all())),
        internet_enabled: std::sync::Arc::new(std::sync::RwLock::new(false)),
    });

    println!("Jeebs is awake on port 8080");

    // Session cookie secret
    let secret_key = Key::generate();

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive()) // This allows your phone to connect
            .wrap(SessionMiddleware::new(CookieSessionStore::default(), secret_key.clone()))
            .app_data(state.clone())
                .service(auth::login_pgp)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

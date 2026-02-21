use actix_cors::Cors;
use actix_files::Files;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::{web, App, HttpServer};
use jeebs::{admin, auth, chat, cortex, evolution, logging, AppState};
use sqlx::SqlitePool;
use std::env;
use std::path::Path;
use std::time::Duration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./jeebs.db".to_string());

    // Ensure the SQLite directory exists if using a file path
    if let Some(path_str) = database_url.strip_prefix("sqlite:") {
        if !path_str.is_empty() && path_str != ":memory:" {
            let path = Path::new(path_str);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
        }
    }

    let pool: SqlitePool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(&database_url)
        .await
        .expect("DB Fail");

    // Apply any pending migrations at startup so offline-built binaries catch up when DB is available.
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("Failed to run migrations: {e}");
    }

    // Ensure logging storage exists even on databases created before log migrations.
    logging::init(&pool).await;

    // Run log retention cleanup on startup and then every 24 hours.
    let log_pool = pool.clone();
    tokio::spawn(async move {
        loop {
            logging::cleanup_old_logs(&log_pool).await;
            tokio::time::sleep(Duration::from_secs(24 * 60 * 60)).await;
        }
    });

    let state = web::Data::new(AppState {
        db: pool,
        plugins: vec![],
        ip_blacklist: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashSet::new())),
        ip_whitelist: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashSet::new())),
        sys: std::sync::Arc::new(std::sync::Mutex::new(sysinfo::System::new_all())),
        internet_enabled: std::sync::Arc::new(std::sync::RwLock::new(false)),
    });

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8080);

    logging::log(
        &state.db,
        "INFO",
        "SYSTEM",
        &format!("Jeebs server starting on 127.0.0.1:{port}"),
    )
    .await;

    println!("Jeebs is awake on port {}", port);

    // Session cookie secret
    let secret_key = Key::generate();

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive()) // This allows your phone to connect
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .app_data(state.clone())
            .service(auth::register)
            .service(auth::login)
            .service(auth::login_pgp)
            .service(auth::auth_status)
            .service(auth::logout)
            .service(chat::jeebs_api)
            .service(cortex::admin_crawl)
            .service(cortex::search_brain)
            .service(cortex::reindex_brain)
            .service(cortex::admin_train)
            .service(cortex::visualize_brain)
            .service(cortex::get_logic_graph)
            .service(admin::status::get_system_status)
            .service(admin::sessions::get_active_sessions)
            .service(admin::sessions::terminate_session)
            .service(admin::export::export_database)
            .service(admin::import::import_database)
            .service(admin::blacklist::get_blacklist)
            .service(admin::blacklist::add_blacklist_ip)
            .service(admin::blacklist::remove_blacklist_ip)
            .service(admin::whitelist::get_whitelist)
            .service(admin::whitelist::add_whitelist_ip)
            .service(admin::whitelist::remove_whitelist_ip)
            .service(admin::user::admin_list_users)
            .service(admin::user::admin_delete_user)
            .service(admin::user::admin_reset_user_password)
            .service(admin::user::admin_update_user_role)
            .service(admin::internet::get_internet_status)
            .service(admin::internet::set_internet_status)
            .service(logging::get_logs)
            .service(logging::clear_logs)
            .service(logging::export_logs)
            .service(logging::get_categories)
            .service(logging::ws_index)
            .service(logging::get_my_logs)
            .service(evolution::list_updates)
            .service(evolution::apply_update)
            .service(evolution::deny_update)
            .service(evolution::resolve_update)
            .service(evolution::rollback_update)
            .service(evolution::add_comment)
            .service(evolution::get_notifications)
            .service(evolution::dismiss_notification)
            .service(evolution::brainstorm_update)
            .service(Files::new("/webui", "./webui").index_file("index.html"))
            .service(Files::new("/", "./webui").index_file("index.html"))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

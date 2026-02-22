use actix_cors::Cors;
use actix_files::Files;
// Rate limiting removed: actix-governor disabled to avoid 429 responses
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::dev::ServiceRequest;
use actix_web::{web, App, HttpServer};
use base64;
use jeebs::plugins::{
    Base64Plugin, CalcPlugin, ContactPlugin, ErrorPlugin, HashPlugin, LogicPlugin, MemoryPlugin,
    NewsPlugin, PasswordPlugin, SummaryPlugin, SystemPlugin, TimePlugin, TodoPlugin,
    TranslatePlugin, WeatherPlugin, WebsiteStatusPlugin,
};
use jeebs::{
    admin, auth, brain_parsing_api, chat, cortex, evolution, logging, user_chat, AppState,
};
use sqlx::{Row, SqlitePool};
use std::collections::HashSet;
use std::env;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use sysinfo::System;

// Rate limiter key extractor removed — no per-IP throttling.

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

    // Apply any pending migrations at startup
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("Failed to run migrations: {e}");
    }

    // Ensure logging storage exists
    logging::init(&pool).await;

    // Run log retention cleanup on startup and then every 24 hours
    let log_pool = pool.clone();
    tokio::spawn(async move {
        loop {
            logging::cleanup_old_logs(&log_pool).await;
            tokio::time::sleep(Duration::from_secs(24 * 60 * 60)).await;
        }
    });

    // Load IP Blacklist
    let rows = sqlx::query("SELECT ip FROM ip_blacklist")
        .fetch_all(&pool)
        .await
        .expect("Failed to load blacklist");
    let mut ips = HashSet::new();
    for row in rows {
        let ip: String = row.get(0);
        ips.insert(ip);
    }
    let ip_blacklist = Arc::new(RwLock::new(ips));

    // Load IP Whitelist
    let rows = sqlx::query("SELECT ip FROM ip_whitelist")
        .fetch_all(&pool)
        .await
        .expect("Failed to load whitelist");
    let mut w_ips = HashSet::new();
    for row in rows {
        let ip: String = row.get(0);
        w_ips.insert(ip);
    }
    let ip_whitelist = Arc::new(RwLock::new(w_ips));

    // Initialize Plugins
    let mut plugins: Vec<Box<dyn jeebs::plugins::Plugin>> = vec![
        Box::new(TimePlugin),
        Box::new(CalcPlugin),
        Box::new(WeatherPlugin),
        Box::new(NewsPlugin),
        Box::new(MemoryPlugin),
        Box::new(SystemPlugin),
        Box::new(SummaryPlugin),
        Box::new(TranslatePlugin),
        Box::new(PasswordPlugin),
        Box::new(HashPlugin),
        Box::new(Base64Plugin),
        Box::new(jeebs::security::SecurityPlugin),
        Box::new(LogicPlugin),
        Box::new(ContactPlugin),
        Box::new(WebsiteStatusPlugin),
        Box::new(TodoPlugin),
        Box::new(ErrorPlugin),
    ];
    plugins.extend(jeebs::plugins::load_dynamic_plugins("plugins"));

    // Load toggle states from database (remembers user's last settings)
    let (internet_enabled, training_enabled) = jeebs::toggle_manager::load_toggle_states(&pool)
        .await
        .unwrap_or((false, false));

    let state = web::Data::new(AppState {
        db: pool.clone(),
        plugins,
        ip_blacklist,
        ip_whitelist,
        sys: Arc::new(Mutex::new(System::new_all())),
        internet_enabled: Arc::new(RwLock::new(internet_enabled)),
    });

    // Start Jeebs autonomous evolution loop
    evolution::spawn_autonomous_thinker(state.db.clone());

    // Sync training state with persisted toggle and always run worker
    let _ = cortex::sync_training_state_with_toggle(&state.db, training_enabled, "startup").await;
    // Training worker was removed; if reintroduced, wire it here.

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8080);

    logging::log(
        &state.db,
        "INFO",
        "SYSTEM",
        &format!("Jeebs server starting on 0.0.0.0:{port}"),
    )
    .await;

    println!("Jeebs is awake on port {}", port);

    // Session cookie secret: prefer SESSION_KEY_B64 env var (base64-encoded).
    // If not set or invalid, fall back to a generated ephemeral key.
    let secret_key = match std::env::var("SESSION_KEY_B64") {
        Ok(s) => match base64::decode(&s) {
            Ok(bytes) => {
                if bytes.is_empty() {
                    eprintln!("SESSION_KEY_B64 is empty; generating ephemeral key");
                    Key::generate()
                } else {
                    Key::from(&bytes)
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to decode SESSION_KEY_B64: {}. Generating ephemeral key",
                    e
                );
                Key::generate()
            }
        },
        Err(_) => {
            eprintln!("SESSION_KEY_B64 not set; generating ephemeral session key (sessions won't persist across restarts). Set SESSION_KEY_B64 env to persist.");
            Key::generate()
        }
    };

    // Rate limiting disabled: no governor config created.

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            // Rate limiter removed to prevent 429 "Too Many Requests"
            .app_data(state.clone())
            .service(auth::register)
            .service(auth::login)
            .service(auth::login_pgp)
            .service(auth::logout)
            .service(auth::session_ping)
            .service(auth::change_username)
            .service(auth::auth_status)
            .service(chat::jeebs_api)
            .service(user_chat::user_chat)
            .service(user_chat::chat_preflight)
            .service(user_chat::chat_status)
            // Removed admin/knowledge routes that are no longer implemented in cortex
            .service(admin::status::get_system_status)
            .service(admin::status::health_check)
            .service(admin::status::get_server_stats)
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
            .service(admin::training::get_training_status)
            .service(admin::training::set_training_mode)
            .service(admin::training::run_training_now)
            .service(logging::get_logs)
            .service(logging::get_anomalies)
            .service(logging::scan_legacy_logs)
            .service(logging::list_scan_jobs)
            .service(logging::scan_job_status)
            .service(logging::get_reasoning_traces)
            .service(logging::clear_logs)
            .service(logging::export_logs)
            .service(logging::get_categories)
            .service(logging::ws_index)
            .service(logging::get_my_logs)
            .service(evolution::list_updates)
            .service(evolution::public_list_updates)
            .service(evolution::public_evolution_stats)
            .service(evolution::get_evolution_status)
            .service(evolution::run_think_cycle)
            .service(evolution::apply_update)
            .service(evolution::deny_update)
            .service(evolution::resolve_update)
            .service(evolution::rollback_update)
            .service(evolution::add_comment)
            .service(evolution::get_notifications)
            .service(evolution::dismiss_notification)
            .service(evolution::brainstorm_update)
            .service(evolution::vote_update)
            .service(brain_parsing_api::parse_brain_node)
            .service(brain_parsing_api::visualize)
            .service(brain_parsing_api::build_brain_graph)
            .service(brain_parsing_api::query_graph_entity)
            .service(brain_parsing_api::query_graph_category)
            .service(brain_parsing_api::get_graph_statistics)
            .service(brain_parsing_api::analyze_relationships)
            .service(brain_parsing_api::get_entities_report)
            .service(cortex::generate_template_proposals_endpoint)
            .service(cortex::get_template_proposals_endpoint)
            .service(cortex::update_proposal_status_endpoint)
            .service(cortex::get_proposal_statistics_endpoint)
            .service(cortex::start_deep_learning)
            .service(cortex::add_learned_fact)
            .service(cortex::add_practice_problem)
            .service(cortex::get_learning_sessions)
            .service(cortex::get_learning_session_endpoint)
            .service(cortex::run_extended_learning)
            .service(cortex::get_extended_run)
            .service(cortex::list_extended_runs)
            .service(cortex::cancel_extended_run)
            .service(cortex::get_learning_statistics)
            .service(cortex::get_learning_summary_endpoint)
            .service(Files::new("/webui", "./webui").index_file("index.html"))
            .service(Files::new("/", "./webui").index_file("index.html"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

use actix_cors::Cors;
use actix_files::Files;
use actix_governor::{Governor, GovernorConfigBuilder, KeyExtractor, SimpleKeyExtractionError};
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::dev::ServiceRequest;
use actix_web::{web, App, HttpServer};
use jeebs::{
    admin, auth, brain_parsing_api, brain_shard, chat, cortex, evolution, logging, user_chat,
    AppState,
};
use jeebs::plugins::{
    Base64Plugin, CalcPlugin, ContactPlugin, ErrorPlugin, HashPlugin, LogicPlugin, MemoryPlugin,
    NewsPlugin, PasswordPlugin, SummaryPlugin, SystemPlugin, TimePlugin, TodoPlugin,
    TranslatePlugin, WeatherPlugin, WebsiteStatusPlugin,
};
use sqlx::{mysql::MySqlPoolOptions, Row, SqlitePool};
use std::collections::HashSet;
use std::env;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use sysinfo::System;

#[derive(Clone)]
struct WhitelistedKeyExtractor;

impl KeyExtractor for WhitelistedKeyExtractor {
    type Key = String;
    type KeyExtractionError = SimpleKeyExtractionError<String>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        if let Some(state) = req.app_data::<web::Data<AppState>>() {
            let ip = req.peer_addr().map(|a| a.ip().to_string()).unwrap_or_else(|| "unknown".to_string());
            if let Ok(whitelist) = state.ip_whitelist.read() {
                if whitelist.contains(&ip) {
                    return Ok(format!("whitelist:{}", uuid::Uuid::new_v4()));
                }
            }
            Ok(ip)
        } else {
            Ok("unknown".to_string())
        }
    }
}

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

    // Optional MySQL brain shard
    let mysql_url = env::var("MYSQL_BRAIN_URL")
        .unwrap_or_else(|_| "mysql://admin:L1QbNDvv@mysql-208625-0.cloudclusters.net:10060/jeebs_brain".to_string());
    let mysql_brain = match MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&mysql_url)
        .await
    {
        Ok(pool) => {
            if let Err(err) = jeebs::brain_shard::ensure_schema(&pool).await {
                eprintln!("Failed to ensure MySQL brain shard schema: {err}");
            }
            Some(pool)
        }
        Err(err) => {
            eprintln!("MySQL brain shard unavailable: {err}");
            None
        }
    };

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
        mysql_brain,
        plugins,
        ip_blacklist,
        ip_whitelist,
        sys: Arc::new(Mutex::new(System::new_all())),
        internet_enabled: Arc::new(RwLock::new(internet_enabled)),
    });

    // Start Jeebs autonomous evolution loop
    evolution::spawn_autonomous_thinker(state.db.clone());

    // Initialize MySQL brain shard pool eagerly so cross-brain storage is ready
    tokio::spawn(async move {
        if let Some(_) = brain_shard::global_pool().await {
            logging::log(&pool, "INFO", "BRAIN_SHARD", "Connected to MySQL brain shard").await;
        } else {
            logging::log(&pool, "WARN", "BRAIN_SHARD", "Brain shard connection not available").await;
        }
    });

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

    // Session cookie secret
    let secret_key = Key::generate();

    // Governor configuration
    let governor_conf = GovernorConfigBuilder::default()
        .key_extractor(WhitelistedKeyExtractor)
        .per_second(2)
        .burst_size(5)
        .finish()
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .wrap(Governor::new(&governor_conf))
            .app_data(state.clone())
            .service(auth::register)
            .service(auth::login)
            .service(auth::login_pgp)
            .service(auth::logout)
            .service(auth::session_ping)
            .service(auth::auth_status)
            .service(chat::jeebs_api)
            .service(user_chat::user_chat)
            .service(user_chat::chat_preflight)
            .service(user_chat::chat_status)
            // Removed admin/knowledge routes that are no longer implemented in cortex
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
            .service(admin::training::get_training_status)
            .service(admin::training::set_training_mode)
            .service(logging::get_logs)
            .service(logging::clear_logs)
            .service(logging::export_logs)
            .service(logging::get_categories)
            .service(logging::ws_index)
            .service(logging::get_my_logs)
            .service(evolution::list_updates)
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
            .service(brain_parsing_api::parse_brain_node)
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
            .service(cortex::get_learning_statistics)
            .service(cortex::get_learning_summary_endpoint)
            .service(Files::new("/webui", "./webui").index_file("index.html"))
            .service(Files::new("/", "./webui").index_file("index.html"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

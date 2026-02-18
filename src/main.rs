use actix_session::Session;
mod admin;
mod brain;
mod auth;
mod cortex;
mod chat;
mod updater;
mod state;
mod utils;
mod plugins;
mod logging;
mod security;
mod evolution;

use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use actix_web::dev::Service;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use actix_web::middleware::Logger;
use actix_files::Files;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Row;
use std::sync::{Arc, RwLock};
use std::collections::HashSet;
use std::sync::Mutex;
use sysinfo::System;
use actix_governor::{Governor, GovernorConfigBuilder, KeyExtractor, SimpleKeyExtractionError};
use actix_web::dev::ServiceRequest;
use crate::plugins::{TimePlugin, CalcPlugin, WeatherPlugin, NewsPlugin, MemoryPlugin, SystemPlugin, SummaryPlugin, TranslatePlugin, PasswordPlugin, HashPlugin, Base64Plugin, LogicPlugin, ContactPlugin, WebsiteStatusPlugin, TodoPlugin, ErrorPlugin};

use crate::state::AppState;

#[get("/health")]
async fn health_check(data: web::Data<AppState>, session: Session) -> impl Responder {
	let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Not logged in"})),
    };

    let user_key = format!("user:{}", username);
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(&user_key).fetch_optional(&data.db).await {
        let val: Vec<u8> = row.get(0);
        if let Ok(user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            let role = user_json["role"].as_str().unwrap_or("user");
            if role == "admin" || role == "moderator" {
                return HttpResponse::Ok().json(serde_json::json!({
                    "status": "ok",
                    "timestamp": chrono::Local::now().to_rfc3339()
                }));
            }
        }
    }
    
	HttpResponse::Forbidden().json(serde_json::json!({"error": "Access denied"}))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
	let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:jeebs.db".to_string());
	println!("Connecting to database: {}", database_url);
	let db = SqlitePoolOptions::new()
		.connect(&database_url).await.expect("Failed to connect to database");

	sqlx::migrate!().run(&db).await.expect("Failed to run migrations");

	// Ensure admin account exists
	auth::ensure_admin_exists(&db).await;
	auth::ensure_pgp_user(&db, "1090mb", "1090mb@protonmail.com", "admin").await;

	// Initialize Logging (Database)
	logging::init(&db).await;

	// Start log cleanup background task
	let log_db_clone = db.clone();
	tokio::spawn(async move {
		// This task will run on startup and then once every 24 hours.
		loop {
			logging::cleanup_old_logs(&log_db_clone).await;
			tokio::time::sleep(tokio::time::Duration::from_secs(24 * 60 * 60)).await;
		}
	});

	// Seed basic knowledge
	brain::seed_knowledge(&db).await;

		// Start GitHub update watcher in background
	tokio::spawn(updater::github_update_watcher());

	// Start Cortex Dreaming (Background Optimization)
	tokio::spawn(cortex::Cortex::dream(db.clone()));

	// Initialize Plugins
	let mut plugins: Vec<Box<dyn crate::plugins::Plugin>> = vec![
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
		Box::new(crate::security::SecurityPlugin),
		Box::new(LogicPlugin),
		Box::new(ContactPlugin),
		Box::new(WebsiteStatusPlugin),
		Box::new(TodoPlugin),
		Box::new(ErrorPlugin),
	];
	plugins.extend(crate::plugins::load_dynamic_plugins("plugins"));

	// Load IP Blacklist
	let rows = sqlx::query("SELECT ip FROM ip_blacklist").fetch_all(&db).await.expect("Failed to load blacklist");
	let mut ips = HashSet::new();
	for row in rows {
		let ip: String = row.get(0);
		ips.insert(ip);
	}
	let ip_blacklist = Arc::new(RwLock::new(ips));

	// Load IP Whitelist
	let rows = sqlx::query("SELECT ip FROM ip_whitelist").fetch_all(&db).await.expect("Failed to load whitelist");
	let mut w_ips = HashSet::new();
	for row in rows {
		let ip: String = row.get(0);
		w_ips.insert(ip);
	}
	let ip_whitelist = Arc::new(RwLock::new(w_ips));

	// System Info
	let sys = System::new_all();

		// Start web server in background
		let state = web::Data::new(AppState { db: db.clone(), plugins, ip_blacklist, ip_whitelist, sys: Arc::new(Mutex::new(sys)) });
		let secret_key = Key::generate();

		#[derive(Clone)]
		struct WhitelistedKeyExtractor;
		impl KeyExtractor for WhitelistedKeyExtractor {
			type Key = String;
			type KeyExtractionError = SimpleKeyExtractionError<String>;
			fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
				let state = req.app_data::<web::Data<AppState>>().unwrap();
				let ip = req.peer_addr().map(|a| a.ip().to_string()).unwrap_or_else(|| "unknown".to_string());
				if state.ip_whitelist.read().unwrap().contains(&ip) {
					return Ok(format!("whitelist:{}", uuid::Uuid::new_v4())); // Unique key per request bypasses rate limit
				}
				Ok(ip)
			}
		}

		let governor_conf = GovernorConfigBuilder::default()
			.key_extractor(WhitelistedKeyExtractor)
			.per_second(2)
			.burst_size(5)
			.finish()
			.unwrap();
		let web_server = HttpServer::new(move || {
			App::new()
				.app_data(web::JsonConfig::default().limit(50 * 1024 * 1024))
				.wrap_fn(|req, srv| {
					let state = req.app_data::<web::Data<AppState>>().unwrap();
					let ip = req.peer_addr().map(|a| a.ip().to_string()).unwrap_or_default();
					if state.ip_blacklist.read().unwrap().contains(&ip) {
						return Box::pin(async { Ok(req.error_response(actix_web::error::ErrorForbidden("IP Blacklisted"))) })
							as std::pin::Pin<Box<dyn std::future::Future<Output = Result<actix_web::dev::ServiceResponse, actix_web::Error>>>>;
					}
					Box::pin(srv.call(req))
				})
				.wrap(Governor::new(&governor_conf))
				.wrap(Logger::default())
				.wrap(SessionMiddleware::new(
					CookieSessionStore::default(),
					secret_key.clone(),
				))
				.app_data(state.clone())
				.service(auth::register)
				.service(auth::login)
				.service(auth::login_pgp)
				.service(auth::logout)
				.service(auth::request_reset)
				.service(auth::reset_password)
				.service(auth::verify_email)
				.service(auth::change_password)
				// .service(auth::update_email)
				.service(auth::upload_avatar)
				.service(auth::get_avatar)
				.service(auth::get_profile)
				.service(auth::delete_account)
				.service(health_check)
				.service(chat::jeebs_api)
				.service(brain::admin_train)
				.service(brain::admin_crawl)
				.service(brain::search_brain)
				.service(brain::reindex_brain)
				.service(brain::visualize_brain)
				.service(brain::get_logic_graph)
				.service(admin::admin_list_users)
				.service(admin::admin_delete_user)
				.service(admin::admin_reset_user_password)
				.service(admin::admin_update_user_role)
				.service(admin::get_blacklist)
				.service(admin::add_blacklist_ip)
				.service(admin::remove_blacklist_ip)
				.service(admin::get_whitelist)
				.service(admin::add_whitelist_ip)
				.service(admin::remove_whitelist_ip)
				.service(admin::get_system_status)
				.service(logging::get_logs)
				.service(logging::clear_logs)
				.service(logging::get_categories)
				.service(logging::get_my_logs)
				.service(logging::export_logs)
				.service(logging::ws_index)
				.service(admin::export_database)
				.service(admin::import_database)
				.service(admin::get_active_sessions)
				.service(admin::terminate_session)
				.service(evolution::list_updates)
				.service(evolution::apply_update)
				.service(evolution::deny_update)
				.service(evolution::resolve_update)
				.service(evolution::rollback_update)
				.service(evolution::add_comment)
				.service(evolution::get_notifications)
				.service(evolution::dismiss_notification)
				.service(evolution::brainstorm_update)
				.service(Files::new("/", "webui").index_file("index.html"))
	})
	.bind(("0.0.0.0", std::env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse::<u16>().unwrap_or(8080)))?;

	// CLI loop (optional, can be removed if only web is needed)
	let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
	println!("Jeebs is running! Web API at http://127.0.0.1:{}", port);

	let mut cli_plugins: Vec<Box<dyn crate::plugins::Plugin>> = vec![
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
		Box::new(crate::security::SecurityPlugin),
		Box::new(LogicPlugin),
		Box::new(ContactPlugin),
		Box::new(WebsiteStatusPlugin),
		Box::new(TodoPlugin),
		Box::new(ErrorPlugin),
	];
	cli_plugins.extend(crate::plugins::load_dynamic_plugins("plugins"));

	let cli_state = web::Data::new(AppState { db: db.clone(), plugins: cli_plugins, ip_blacklist: Arc::new(RwLock::new(HashSet::new())), ip_whitelist: Arc::new(RwLock::new(HashSet::new())), sys: Arc::new(Mutex::new(System::new_all())) });
	// Run CLI in a separate thread so it doesn't block the web server startup
	std::thread::spawn(move || {
		chat::start_cli(cli_state);
	});

	web_server.run().await
}

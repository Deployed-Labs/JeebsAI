use sqlx::SqlitePool;
use crate::plugins::Plugin;
use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashSet;
use sysinfo::System;

pub struct AppState {
    pub db: SqlitePool,
    pub plugins: Vec<Box<dyn Plugin>>,
    pub ip_blacklist: Arc<RwLock<HashSet<String>>>,
    pub ip_whitelist: Arc<RwLock<HashSet<String>>>,
    pub sys: Arc<Mutex<System>>,
}
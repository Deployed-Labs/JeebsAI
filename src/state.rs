use crate::plugins::Plugin;
use sqlx::{MySqlPool, SqlitePool};
use std::collections::HashSet;
use std::sync::{Arc, Mutex, RwLock};
use sysinfo::System;

pub struct AppState {
    pub db: SqlitePool,
    pub mysql_brain: Option<MySqlPool>,
    pub plugins: Vec<Box<dyn Plugin>>,
    pub ip_blacklist: Arc<RwLock<HashSet<String>>>,
    pub ip_whitelist: Arc<RwLock<HashSet<String>>>,
    pub sys: Arc<Mutex<System>>,
    pub internet_enabled: Arc<RwLock<bool>>,
}

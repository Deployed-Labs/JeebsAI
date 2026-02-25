use crate::plugins::Plugin;
use crate::brain::coded_holographic_data_storage_container::CodedHolographicDataStorageContainer;
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::sync::{Arc, Mutex, RwLock};
use sysinfo::System;

pub struct AppState {
    pub db: SqlitePool,
    pub plugins: Vec<Box<dyn Plugin>>,
    pub ip_blacklist: Arc<RwLock<HashSet<String>>>,
    pub ip_whitelist: Arc<RwLock<HashSet<String>>>,
    pub sys: Arc<Mutex<System>>,
    pub internet_enabled: Arc<RwLock<bool>>,
    pub chdsc: Arc<RwLock<CodedHolographicDataStorageContainer>>,
}

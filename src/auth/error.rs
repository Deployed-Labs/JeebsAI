use std::pin::Pin;
use std::future::Future;
use sqlx::SqlitePool;
use crate::plugins::Plugin;

pub struct ErrorPlugin;

impl Plugin for ErrorPlugin {
    fn name(&self) -> &str {
        "ErrorTest"
    }

    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            if prompt.to_lowercase().contains("trigger error") {
                return Some("Error: Simulated plugin failure for testing evolution.".to_string());
            }
            None
        })
    }
}
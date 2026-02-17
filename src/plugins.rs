
use crate::state::AppState;
use async_trait::async_trait;

#[async_trait]
pub trait Plugin: Send + Sync {
	fn name(&self) -> &'static str;
	async fn handle(&self, input: &str, state: &AppState) -> Option<String>;
}

pub struct TimePlugin;
pub struct CalcPlugin;
pub struct WeatherPlugin;
pub struct NewsPlugin;
pub struct MemoryPlugin;
pub struct SystemPlugin;
pub struct SummaryPlugin;
pub struct TranslatePlugin;
pub struct PasswordPlugin;
pub struct HashPlugin;
pub struct Base64Plugin;
pub struct LogicPlugin;
pub struct ContactPlugin;
pub struct WebsiteStatusPlugin;
pub struct TodoPlugin;
pub struct ErrorPlugin;

// Dynamic plugin loader stub
pub fn load_dynamic_plugins(_dir: &str) -> Vec<Box<dyn Plugin>> {
	Vec::new()
}

use crate::state::AppState;
use async_trait::async_trait;

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
    async fn handle(&self, input: &str, state: &AppState) -> Option<String>;
}

// ── TimePlugin ──
pub struct TimePlugin;
#[async_trait]
impl Plugin for TimePlugin {
    fn name(&self) -> &'static str { "time" }
    async fn handle(&self, input: &str, _state: &AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("time") || lower.contains("date") || lower.contains("clock") {
            let now = chrono::Local::now();
            Some(format!("{}", now.format("%A, %B %e, %Y at %I:%M:%S %p")))
        } else {
            None
        }
    }
}

// ── CalcPlugin ──
pub struct CalcPlugin;
#[async_trait]
impl Plugin for CalcPlugin {
    fn name(&self) -> &'static str { "calc" }
    async fn handle(&self, input: &str, _state: &AppState) -> Option<String> {
        let lower = input.to_lowercase();
        let expr = lower
            .trim_start_matches("calc ")
            .trim_start_matches("calculate ")
            .trim_start_matches("compute ")
            .trim();
        if expr.is_empty() { return None; }
        match meval::eval_str(expr) {
            Ok(result) => Some(format!("{} = {}", expr, result)),
            Err(_) => None,
        }
    }
}

// ── HashPlugin ──
pub struct HashPlugin;
#[async_trait]
impl Plugin for HashPlugin {
    fn name(&self) -> &'static str { "hash" }
    async fn handle(&self, input: &str, _state: &AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if !lower.starts_with("hash ") { return None; }
        let text = lower.trim_start_matches("hash ").trim();
        if text.is_empty() { return None; }
        let hash = blake3::hash(text.as_bytes()).to_hex().to_string();
        Some(format!("BLAKE3({}): {}", text, hash))
    }
}

// ── Base64Plugin ──
pub struct Base64Plugin;
#[async_trait]
impl Plugin for Base64Plugin {
    fn name(&self) -> &'static str { "base64" }
    async fn handle(&self, input: &str, _state: &AppState) -> Option<String> {
        let lower = input.to_lowercase();
        use base64::Engine;
        if lower.starts_with("base64 encode ") {
            let data = input[14..].trim();
            Some(base64::engine::general_purpose::STANDARD.encode(data.as_bytes()))
        } else if lower.starts_with("base64 decode ") {
            let data = input[14..].trim();
            match base64::engine::general_purpose::STANDARD.decode(data) {
                Ok(bytes) => Some(String::from_utf8_lossy(&bytes).to_string()),
                Err(e) => Some(format!("Decode error: {}", e)),
            }
        } else {
            None
        }
    }
}

// ── PasswordPlugin ──
pub struct PasswordPlugin;
#[async_trait]
impl Plugin for PasswordPlugin {
    fn name(&self) -> &'static str { "password" }
    async fn handle(&self, input: &str, _state: &AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("password") && (lower.contains("generate") || lower.contains("random")) {
            let mut rng = rand::thread_rng();
            let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
            let password: String = (0..20)
                .map(|_| { let idx = rand::Rng::gen_range(&mut rng, 0..charset.len()); charset[idx] as char })
                .collect();
            Some(password)
        } else {
            None
        }
    }
}

// ── SystemPlugin ──
pub struct SystemPlugin;
#[async_trait]
impl Plugin for SystemPlugin {
    fn name(&self) -> &'static str { "system" }
    async fn handle(&self, input: &str, state: &AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("system") || lower.contains("status") || lower.contains("uptime") {
            let mut sys = state.sys.lock().unwrap();
            sys.refresh_all();
            let total_mem = sys.total_memory();
            let used_mem = sys.used_memory();
            let cpu_count = sys.cpus().len();
            let avg_cpu = if cpu_count > 0 {
                sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / cpu_count as f32
            } else { 0.0 };
            Some(format!(
                "CPU: {:.1}% ({} cores), Memory: {:.0}/{:.0} MB",
                avg_cpu, cpu_count,
                used_mem as f64 / 1_048_576.0,
                total_mem as f64 / 1_048_576.0,
            ))
        } else {
            None
        }
    }
}

// ── LogicPlugin ──
pub struct LogicPlugin;
#[async_trait]
impl Plugin for LogicPlugin {
    fn name(&self) -> &'static str { "logic" }
    async fn handle(&self, input: &str, _state: &AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if !lower.starts_with("logic ") { return None; }
        let expr = lower.trim_start_matches("logic ").trim();
        match evalexpr::eval(expr) {
            Ok(val) => Some(format!("{}", val)),
            Err(_) => None,
        }
    }
}

// ── MemoryPlugin ──
pub struct MemoryPlugin;
#[async_trait]
impl Plugin for MemoryPlugin {
    fn name(&self) -> &'static str { "memory" }
    async fn handle(&self, input: &str, _state: &AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.starts_with("remember ") || lower.contains("what do you know") {
            Some("Memory operations are handled through the Cortex brain engine.".to_string())
        } else {
            None
        }
    }
}

// ── Stub plugins (no-op, kept for forward compatibility) ──
macro_rules! stub_plugin {
    ($name:ident, $str_name:expr) => {
        pub struct $name;
        #[async_trait]
        impl Plugin for $name {
            fn name(&self) -> &'static str { $str_name }
            async fn handle(&self, _input: &str, _state: &AppState) -> Option<String> { None }
        }
    };
}

stub_plugin!(ContactPlugin, "contact");
stub_plugin!(ErrorPlugin, "error");
stub_plugin!(NewsPlugin, "news");
stub_plugin!(SummaryPlugin, "summary");
stub_plugin!(TodoPlugin, "todo");
stub_plugin!(TranslatePlugin, "translate");
stub_plugin!(WeatherPlugin, "weather");
stub_plugin!(WebsiteStatusPlugin, "website_status");

pub fn load_dynamic_plugins(_dir: &str) -> Vec<Box<dyn Plugin>> {
    Vec::new()
}

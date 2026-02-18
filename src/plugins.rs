#[async_trait]
impl Plugin for ErrorPlugin {
    fn name(&self) -> &'static str {
        "error"
    }
    async fn handle(&self, _input: &str, _state: &crate::state::AppState) -> Option<String> {
        Some("Error: This is a generic error from ErrorPlugin.".to_string())
    }
}
#[async_trait]
impl Plugin for TodoPlugin {
    fn name(&self) -> &'static str {
        "todo"
    }
    async fn handle(&self, input: &str, _state: &crate::state::AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("todo") || lower.contains("task") {
            Some("Todo management is not available in this build.".to_string())
        } else {
            None
        }
    }
}
#[async_trait]
impl Plugin for WebsiteStatusPlugin {
    fn name(&self) -> &'static str {
        "website_status"
    }
    async fn handle(&self, input: &str, _state: &crate::state::AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("website status") || lower.contains("is website up") {
            Some("Website status checking is not available in this build.".to_string())
        } else {
            None
        }
    }
}
#[async_trait]
impl Plugin for ContactPlugin {
    fn name(&self) -> &'static str {
        "contact"
    }
    async fn handle(&self, input: &str, _state: &crate::state::AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("contact") || lower.contains("email") {
            Some("Contact: jeebs@jeebs.club".to_string())
        } else {
            None
        }
    }
}
#[async_trait]
impl Plugin for LogicPlugin {
    fn name(&self) -> &'static str {
        "logic"
    }
    async fn handle(&self, input: &str, _state: &crate::state::AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("logic")
            || lower.contains("and")
            || lower.contains("or")
            || lower.contains("not")
        {
            let expr = lower
                .replace("logic", "")
                .replace("?", "")
                .trim()
                .to_string();
            if expr.is_empty() {
                return Some("Please provide a logic expression to evaluate.".to_string());
            }
            // Try to evaluate as a boolean expression
            match evalexpr::eval_boolean(&expr) {
                Ok(result) => Some(format!("The result is {}.", result)),
                Err(_) => Some("Sorry, I couldn't evaluate that logic expression.".to_string()),
            }
        } else {
            None
        }
    }
}
use base64::engine::general_purpose;
#[async_trait]
impl Plugin for Base64Plugin {
    fn name(&self) -> &'static str {
        "base64"
    }
    async fn handle(&self, input: &str, _state: &crate::state::AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("base64 encode") {
            let text = input.splitn(2, ':').nth(1).unwrap_or(input).trim();
            if text.is_empty() {
                return Some("Please provide text to encode.".to_string());
            }
            Some(format!(
                "Base64: {}",
                general_purpose::STANDARD.encode(text)
            ))
        } else if lower.contains("base64 decode") {
            let text = input.splitn(2, ':').nth(1).unwrap_or(input).trim();
            match general_purpose::STANDARD.decode(text) {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(s) => Some(format!("Decoded: {}", s)),
                    Err(_) => Some("Decoded bytes are not valid UTF-8.".to_string()),
                },
                Err(_) => Some("Invalid base64 input.".to_string()),
            }
        } else {
            None
        }
    }
}
use sha2::{Digest, Sha256};
#[async_trait]
impl Plugin for HashPlugin {
    fn name(&self) -> &'static str {
        "hash"
    }
    async fn handle(&self, input: &str, _state: &crate::state::AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("hash") {
            let text = input.splitn(2, ':').nth(1).unwrap_or(input).trim();
            if text.is_empty() {
                return Some("Please provide text to hash.".to_string());
            }
            let mut hasher = Sha256::new();
            hasher.update(text.as_bytes());
            let result = hasher.finalize();
            Some(format!("SHA-256: {:x}", result))
        } else {
            None
        }
    }
}
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use rand_core::OsRng;
#[async_trait]
impl Plugin for PasswordPlugin {
    fn name(&self) -> &'static str {
        "password"
    }
    async fn handle(&self, input: &str, _state: &crate::state::AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("password") || lower.contains("generate password") {
            let mut buf = [0u8; 16];
            OsRng.fill_bytes(&mut buf);
            let pw = URL_SAFE_NO_PAD.encode(&buf);
            Some(format!("Generated password: {}", pw))
        } else {
            None
        }
    }
}
#[async_trait]
impl Plugin for TranslatePlugin {
    fn name(&self) -> &'static str {
        "translate"
    }
    async fn handle(&self, input: &str, _state: &crate::state::AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("translate") || lower.contains("translation") {
            Some("Sorry, translation is not available in this build.".to_string())
        } else {
            None
        }
    }
}
#[async_trait]
impl Plugin for SummaryPlugin {
    fn name(&self) -> &'static str {
        "summary"
    }
    async fn handle(&self, input: &str, _state: &crate::state::AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("summarize") || lower.contains("summary") {
            let text = input.splitn(2, ':').nth(1).unwrap_or(input).trim();
            if text.is_empty() {
                return Some("Please provide text to summarize.".to_string());
            }
            let summary = text.split('.').next().unwrap_or(text).trim();
            Some(format!("Summary: {}...", summary))
        } else {
            None
        }
    }
}
#[async_trait]
impl Plugin for SystemPlugin {
    fn name(&self) -> &'static str {
        "system"
    }
    async fn handle(&self, _input: &str, state: &crate::state::AppState) -> Option<String> {
        let sys = state.sys.lock().unwrap();
        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let cpu_usage =
            sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;
        Some(format!(
            "System Info: {:.1}% CPU usage, {} MB used / {} MB total RAM",
            cpu_usage,
            used_mem / 1024,
            total_mem / 1024
        ))
    }
}
use std::sync::Mutex;
lazy_static::lazy_static! {
    static ref MEMORY: Mutex<Option<String>> = Mutex::new(None);
}

#[async_trait]
impl Plugin for MemoryPlugin {
    fn name(&self) -> &'static str {
        "memory"
    }
    async fn handle(&self, input: &str, _state: &crate::state::AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("remember") {
            let to_remember = input.splitn(2, "remember").nth(1).unwrap_or("").trim();
            if to_remember.is_empty() {
                return Some("What should I remember?".to_string());
            }
            let mut mem = MEMORY.lock().unwrap();
            *mem = Some(to_remember.to_string());
            Some(format!("Okay, I'll remember: {}", to_remember))
        } else if lower.contains("recall")
            || lower.contains("what did you remember")
            || lower.contains("what do you remember")
        {
            let mem = MEMORY.lock().unwrap();
            match &*mem {
                Some(val) => Some(format!("I remember: {}", val)),
                None => Some("I don't remember anything yet.".to_string()),
            }
        } else {
            None
        }
    }
}
#[async_trait]
impl Plugin for NewsPlugin {
    fn name(&self) -> &'static str {
        "news"
    }
    async fn handle(&self, input: &str, _state: &crate::state::AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("news") || lower.contains("headline") || lower.contains("headlines") {
            Some("Sorry, live news is not available in this build. Please check your favorite news site for updates.".to_string())
        } else {
            None
        }
    }
}
#[async_trait]
impl Plugin for WeatherPlugin {
    fn name(&self) -> &'static str {
        "weather"
    }
    async fn handle(&self, input: &str, _state: &crate::state::AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("weather") || lower.contains("temperature") || lower.contains("forecast")
        {
            Some("Sorry, live weather data is not available in this build. Please check your local forecast online.".to_string())
        } else {
            None
        }
    }
}
#[async_trait]
impl Plugin for CalcPlugin {
    fn name(&self) -> &'static str {
        "calc"
    }
    async fn handle(&self, input: &str, _state: &crate::state::AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("calculate")
            || lower.contains("what is")
            || lower.contains("solve")
            || lower.contains("math")
        {
            let expr = lower
                .replace("calculate", "")
                .replace("what is", "")
                .replace("solve", "")
                .replace("math", "")
                .replace("?", "")
                .trim()
                .to_string();
            if expr.is_empty() {
                return Some("Please provide a math expression to calculate.".to_string());
            }
            match evalexpr::eval(&expr) {
                Ok(result) => Some(format!("The answer is {}.", result)),
                Err(_) => Some("Sorry, I couldn't evaluate that expression.".to_string()),
            }
        } else {
            None
        }
    }
}
use chrono::Local;
#[async_trait]
impl Plugin for TimePlugin {
    fn name(&self) -> &'static str {
        "time"
    }
    async fn handle(&self, input: &str, _state: &crate::state::AppState) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("time") || lower.contains("date") || lower.contains("clock") {
            let now = Local::now();
            Some(format!(
                "The current time is {}.",
                now.format("%Y-%m-%d %H:%M:%S")
            ))
        } else {
            None
        }
    }
}

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

// External CLI plugin wrapper — supports simple JSON-over-stdin contract
pub struct ExternalCliPlugin {
    pub name: &'static str,
    pub cmd: Vec<String>,
}

#[async_trait]
impl Plugin for ExternalCliPlugin {
    fn name(&self) -> &'static str {
        self.name
    }

    async fn handle(&self, input: &str, _state: &AppState) -> Option<String> {
                use tokio::io::AsyncWriteExt;
        use tokio::process::Command;
        use tokio::time::{Duration, timeout};

        let payload = match serde_json::json!({ "input": input })
            .to_string()
            .into_bytes()
        {
            b => b,
        };

        let mut cmd = Command::new(&self.cmd[0]);
        if self.cmd.len() > 1 {
            cmd.args(&self.cmd[1..]);
        }
        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        match cmd.spawn() {
            Ok(mut child) => {
                if let Some(mut stdin) = child.stdin.take() {
                    if let Err(e) = stdin.write_all(&payload).await {
                        let _ = child.kill().await;
                        return Some(format!("plugin '{}' write error: {}", self.name, e));
                    }
                }

                // enforce a short timeout for plugin execution
                let pid = child.id();
                match timeout(Duration::from_secs(3), child.wait_with_output()).await {
                    Ok(Ok(output)) => {
                        if !output.status.success() {
                            return Some(format!(
                                "plugin '{}' failed: {}",
                                self.name,
                                String::from_utf8_lossy(&output.stderr)
                            ));
                        }
                        let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        // Try structured response first
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&out) {
                            if let Some(resp) = json.get("response").and_then(|v| v.as_str()) {
                                return Some(resp.to_string());
                            }
                        }
                        if !out.is_empty() {
                            return Some(out);
                        }
                        None
                    }
                    Ok(Err(e)) => {
                        // waiting returned an I/O error; try to kill by PID if available
                        if let Some(p) = pid {
                            let _ = Command::new("kill").arg("-9").arg(p.to_string()).status().await;
                        }
                        Some(format!("plugin '{}' execution error: {}", self.name, e))
                    }
                    Err(_) => {
                        // timeout: kill process by PID if possible
                        if let Some(p) = pid {
                            let _ = Command::new("kill").arg("-9").arg(p.to_string()).status().await;
                        }
                        Some(format!("plugin '{}' timed out", self.name))
                    }
                }
            }
            Err(e) => Some(format!("plugin '{}' spawn error: {}", self.name, e)),
        }
    }
}

// Discover simple CLI-based plugins under `plugins/`.
// Plugin contract: plugin reads JSON from stdin { "input": "..." } and writes JSON { "response": "..." }
// Supported runners (by file present):
//  - run          (executable in plugin directory)
//  - run.py       (invoked with `python3 run.py`)
//  - run.js/index.js (invoked with `node <file>`)
pub fn load_dynamic_plugins(dir: &str) -> Vec<Box<dyn Plugin>> {
    let mut v: Vec<Box<dyn Plugin>> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = path.file_name().unwrap().to_string_lossy().to_string();

            let runner: Option<Vec<String>> = if path.join("run").exists() {
                Some(vec![path.join("run").to_string_lossy().to_string()])
            } else if path.join("run.py").exists() {
                Some(vec![
                    "python3".to_string(),
                    path.join("run.py").to_string_lossy().to_string(),
                ])
            } else if path.join("run.js").exists() || path.join("index.js").exists() {
                let js = if path.join("run.js").exists() {
                    "run.js"
                } else {
                    "index.js"
                };
                Some(vec![
                    "node".to_string(),
                    path.join(js).to_string_lossy().to_string(),
                ])
            } else {
                None
            };

            if let Some(cmd) = runner {
                // leak name into &'static str (one-time, acceptable for plugin names)
                let leaked: &'static str = Box::leak(name.into_boxed_str());
                v.push(Box::new(ExternalCliPlugin { name: leaked, cmd }));
            }
        }
    }

    v
}

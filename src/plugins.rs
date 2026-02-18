<<<<<<< HEAD
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
                Ok(result) => Some(format!("The result is {result}.")),
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
            let text = input.split_once(':').map(|x| x.1).unwrap_or(input).trim();
            if text.is_empty() {
                return Some("Please provide text to encode.".to_string());
            }
            Some(format!(
                "Base64: {}",
                general_purpose::STANDARD.encode(text)
            ))
        } else if lower.contains("base64 decode") {
            let text = input.split_once(':').map(|x| x.1).unwrap_or(input).trim();
            match general_purpose::STANDARD.decode(text) {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(s) => Some(format!("Decoded: {s}")),
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
            let text = input.split_once(':').map(|x| x.1).unwrap_or(input).trim();
            if text.is_empty() {
                return Some("Please provide text to hash.".to_string());
            }
            let mut hasher = Sha256::new();
            hasher.update(text.as_bytes());
            let result = hasher.finalize();
            Some(format!("SHA-256: {result:x}"))
        } else {
            None
        }
    }
}
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
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
            let pw = URL_SAFE_NO_PAD.encode(buf);
            Some(format!("Generated password: {pw}"))
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
            let text = input.split_once(':').map(|x| x.1).unwrap_or(input).trim();
            if text.is_empty() {
                return Some("Please provide text to summarize.".to_string());
            }
            let summary = text.split('.').next().unwrap_or(text).trim();
            Some(format!("Summary: {summary}..."))
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
            let to_remember = input.split_once("remember").map(|x| x.1).unwrap_or("").trim();
            if to_remember.is_empty() {
                return Some("What should I remember?".to_string());
            }
            let mut mem = MEMORY.lock().unwrap();
            *mem = Some(to_remember.to_string());
            Some(format!("Okay, I'll remember: {to_remember}"))
        } else if lower.contains("recall")
            || lower.contains("what did you remember")
            || lower.contains("what do you remember")
        {
            let mem = MEMORY.lock().unwrap();
            match &*mem {
                Some(val) => Some(format!("I remember: {val}")),
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
                Ok(result) => Some(format!("The answer is {result}.")),
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
        use tokio::time::{timeout, Duration};

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
                            let _ = Command::new("kill")
                                .arg("-9")
                                .arg(p.to_string())
                                .status()
                                .await;
                        }
                        Some(format!("plugin '{}' execution error: {}", self.name, e))
                    }
                    Err(_) => {
                        // timeout: kill process by PID if possible
                        if let Some(p) = pid {
                            let _ = Command::new("kill")
                                .arg("-9")
                                .arg(p.to_string())
                                .status()
                                .await;
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
=======
use std::pin::Pin;
use std::future::Future;
use sqlx::{SqlitePool, Row};
use crate::utils::{encode_all, decode_all};
use chrono::Local;
use reqwest::Client;
use meval;
use sysinfo::System;
use base64::Engine;
use serde_json::json;
use base64 as b64;
use blake3;
use sha2::{Sha256, Digest};
use md5;
use hex;
use crate::security::generate_password;

// Plugin trait used across the app (object-safe)
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn handle(&self, prompt: String, db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>>;
}

// Time plugin — returns current time
pub struct TimePlugin;
impl Plugin for TimePlugin {
    fn name(&self) -> &str { "Time" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let p = prompt.to_lowercase();
            if p.contains("time") || p.contains("what time") {
                Some(Local::now().to_rfc3339())
            } else { None }
        })
    }
}

// Calc plugin — evaluates simple math expressions using `meval`.
pub struct CalcPlugin;
impl Plugin for CalcPlugin {
    fn name(&self) -> &str { "Calc" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let p = prompt.trim();
            let lower = p.to_lowercase();
            let expr = if lower.starts_with("calc ") { Some(p[5..].trim()) }
                else if lower.starts_with("calculate ") { Some(p[10..].trim()) }
                else if p.chars().all(|c| c.is_ascii() && (c.is_whitespace() || "0123456789.+-*/()%".contains(c))) { Some(p) }
                else { None };

            if let Some(e) = expr {
                match meval::eval_str(e) {
                    Ok(v) => Some(format!("{}", v)),
                    Err(err) => Some(format!("Calculation error: {}", err)),
                }
            } else { None }
        })
    }
}

// Weather plugin — uses wttr.in for a lightweight no-key weather lookup.
pub struct WeatherPlugin;
impl Plugin for WeatherPlugin {
    fn name(&self) -> &str { "Weather" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            if lower.starts_with("weather") {
                let parts: Vec<_> = prompt.splitn(2, ' ').collect();
                let location = parts.get(1).map(|s| s.trim()).filter(|s| !s.is_empty()).unwrap_or("_");
                let client = Client::new();
                let url = format!("https://wttr.in/{}?format=3", urlencoding::encode(location));
                if let Ok(resp) = client.get(&url).send().await {
                    if let Ok(text) = resp.text().await { return Some(text); }
                }
                return Some("Failed to fetch weather.".to_string());
            }
            None
        })
    }
}

// NewsPlugin — lightweight top-stories fetch (keeps interface used by main)
pub struct NewsPlugin;
impl NewsPlugin { pub fn new() -> Self { Self } }
impl Plugin for NewsPlugin {
    fn name(&self) -> &str { "News" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let p = prompt.to_lowercase();
            if p.contains("news") || p.contains("headlines") {
                // Use Hacker News public API (no key)
                let client = Client::new();
                if let Ok(resp) = client.get("https://hacker-news.firebaseio.com/v0/topstories.json").send().await {
                    if let Ok(ids) = resp.json::<Vec<u64>>().await {
                        let mut headlines = Vec::new();
                        for id in ids.into_iter().take(5) {
                            let item_url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
                            if let Ok(item_resp) = client.get(&item_url).send().await {
                                if let Ok(item_json) = item_resp.json::<serde_json::Value>().await {
                                    if let Some(title) = item_json["title"].as_str() {
                                        let url = item_json["url"].as_str().unwrap_or("");
                                        headlines.push(format!("- {} ({})", title, url));
                                    }
                                }
                            }
                        }
                        if !headlines.is_empty() { return Some(format!("Latest headlines:\n{}", headlines.join("\n"))); }
                    }
                }
                return Some("Failed to fetch news.".to_string());
            }
            None
        })
    }
}

// Memory plugin — simple key/value memory stored in `jeebs_store`.
pub struct MemoryPlugin;
impl Plugin for MemoryPlugin {
    fn name(&self) -> &str { "Memory" }
    fn handle(&self, prompt: String, db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            if lower.starts_with("remember ") {
                let rest = prompt[8..].trim();
                if let Some((k, v)) = rest.split_once('=') {
                    let key = format!("mem:{}", k.trim());
                    let payload = json!({ "value": v.trim(), "created_at": Local::now().to_rfc3339() });
                    if let Ok(bytes) = serde_json::to_vec(&payload) {
                        if let Ok(enc) = encode_all(&bytes, 1) {
                            let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)").bind(&key).bind(enc).execute(&db).await;
                            return Some(format!("Remembered {}={}", k.trim(), v.trim()));
                        }
                    }
                    return Some("Failed to store memory.".to_string());
                }
                return Some("Usage: remember KEY=VALUE".to_string());
            } else if lower.starts_with("recall ") {
                let key = prompt[7..].trim();
                let full = format!("mem:{}", key);
                if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(&full).fetch_optional(&db).await {
                    let val: Vec<u8> = row.get(0);
                    if let Ok(bytes) = decode_all(&val) {
                        if let Ok(vj) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                            return Some(vj["value"].as_str().unwrap_or("").to_string());
                        }
                    }
                }
                return Some(format!("No memory found for '{}'.", key));
            } else if lower.starts_with("memories") || lower.starts_with("list memories") {
                if let Ok(rows) = sqlx::query("SELECT key, value FROM jeebs_store WHERE key LIKE 'mem:%'").fetch_all(&db).await {
                    let items: Vec<String> = rows.into_iter().filter_map(|r| {
                        let k: String = r.get(0);
                        let val: Vec<u8> = r.get(1);
                        decode_all(&val).ok().and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok()).and_then(|vj| vj["value"].as_str().map(|s| format!("{} -> {}", k.strip_prefix("mem:").unwrap_or(&k), s.to_string())))
                    }).collect();
                    return Some(format!("Memories:\n{}", items.join("\n")));
                }
                return Some("No memories found.".to_string());
            }
            None
        })
    }
}

// System plugin — returns basic system stats
pub struct SystemPlugin;
impl Plugin for SystemPlugin {
    fn name(&self) -> &str { "System" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let p = prompt.to_lowercase();
            if p.contains("system") || p.contains("cpu") || p.contains("ram") || p.contains("memory") {
                let mut sys = System::new_all();
                sys.refresh_all();
                let total = sys.total_memory();
                let free = sys.available_memory();
                let used = total.saturating_sub(free);
                return Some(format!("CPU cores: {} | Memory used: {} / {} KB", sys.cpus().len(), used, total));
            }
            None
        })
    }
}

// Summary plugin — trivial summarizer (first 2 sentences / truncated)
pub struct SummaryPlugin;
impl Plugin for SummaryPlugin {
    fn name(&self) -> &str { "Summary" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            if lower.starts_with("summarize ") {
                let text = prompt[10..].trim();
                let sentences: Vec<_> = text.split('.').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                if sentences.is_empty() { return Some("Nothing to summarize.".to_string()); }
                let summary = sentences.into_iter().take(2).collect::<Vec<_>>().join(". ");
                return Some(summary.chars().take(400).collect::<String>());
            }
            None
        })
    }
}

// Translate plugin — supports simple 'uppercase'/'lowercase' and pig-latin demo
fn pig_latin_word(w: &str) -> String {
    let w = w.trim();
    if w.is_empty() { return "".to_string(); }
    let first = w.chars().next().unwrap();
    if "aeiouAEIOU".contains(first) { format!("{}-ay", w) } else { format!("{}-{}ay", &w[1..], first) }
}
pub struct TranslatePlugin;
impl Plugin for TranslatePlugin {
    fn name(&self) -> &str { "Translate" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            if lower.starts_with("translate to ") {
                if let Some((lang, rest)) = prompt[13..].split_once(':') {
                    let lang = lang.trim().to_lowercase();
                    let text = rest.trim();
                    match lang.as_str() {
                        "uppercase" => return Some(text.to_uppercase()),
                        "lowercase" => return Some(text.to_lowercase()),
                        "pig" => return Some(text.split_whitespace().map(pig_latin_word).collect::<Vec<_>>().join(" ")),
                        _ => return Some("Translation for that language is not supported in this build.".to_string()),
                    }
                }
                return Some("Usage: translate to <lang>: <text> (supported: uppercase, lowercase, pig)".to_string());
            }
            None
        })
    }
}

// Password generator plugin
pub struct PasswordPlugin;
impl Plugin for PasswordPlugin {
    fn name(&self) -> &str { "Password" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            if lower.starts_with("password") || lower.starts_with("gen password") || lower.starts_with("generate password") {
                let parts: Vec<_> = prompt.split_whitespace().collect();
                let len = parts.iter().find_map(|p| p.parse::<usize>().ok()).unwrap_or(16);
                return Some(generate_password(len));
            }
            None
        })
    }
}

// Hash plugin — supports blake3 (default), sha256 and md5
pub struct HashPlugin;
impl Plugin for HashPlugin {
    fn name(&self) -> &str { "Hash" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            if lower.starts_with("hash ") {
                let rest = prompt[5..].trim();
                if rest.starts_with("sha256 ") {
                    let txt = &rest[7..];
                    let mut hasher = Sha256::new();
                    hasher.update(txt.as_bytes());
                    return Some(hex::encode(hasher.finalize()));
                } else if rest.starts_with("md5 ") {
                    let txt = &rest[4..];
                    // compute MD5 via md5::Md5
                    let mut hasher = md5::Md5::new();
                    hasher.update(txt.as_bytes());
                    let digest = hasher.finalize();
                    return Some(hex::encode(digest));
                } else {
                    return Some(blake3::hash(rest.as_bytes()).to_hex().to_string());
                }
            }
            None
        })
    }
}

// Base64 plugin
pub struct Base64Plugin;
impl Plugin for Base64Plugin {
    fn name(&self) -> &str { "Base64" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            if lower.starts_with("b64 encode ") {
                let txt = prompt[11..].trim();
                return Some(b64::engine::general_purpose::STANDARD.encode(txt.as_bytes()));
            } else if lower.starts_with("b64 decode ") {
                let txt = prompt[11..].trim();
                if let Ok(bytes) = b64::engine::general_purpose::STANDARD.decode(txt) { return Some(String::from_utf8_lossy(&bytes).to_string()); }
                return Some("Invalid base64 input".to_string());
            }
            None
        })
    }
}

// Simple Logic plugin — evaluates trivial "true and false" style prompts
pub struct LogicPlugin;
impl Plugin for LogicPlugin {
    fn name(&self) -> &str { "Logic" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let p = prompt.to_lowercase();
            if p.contains(" and ") || p.contains(" or ") || p.contains("not ") {
                let tokens: Vec<_> = p.split_whitespace().collect();
                // very naive eval: treat "true"/"false" and boolean ops left-to-right
                let mut acc: Option<bool> = None;
                let mut op: Option<&str> = None;
                for t in tokens {
                    match t {
                        "true" => { if let Some(a) = acc { acc = Some(match op { Some("and") => a && true, Some("or") => a || true, _ => true }) } else { acc = Some(true); } }
                        "false" => { if let Some(a) = acc { acc = Some(match op { Some("and") => a && false, Some("or") => a || false, _ => false }) } else { acc = Some(false); } }
                        "and" | "or" => op = Some(t),
                        "not" => { op = Some("not"); if let Some(a) = acc { acc = Some(!a); } }
                        _ => {}
                    }
                }
                if let Some(v) = acc { return Some(format!("{}", v)); }
            }
            None
        })
    }
}

// Contact plugin — extracts simple email/phone patterns
pub struct ContactPlugin;
impl Plugin for ContactPlugin {
    fn name(&self) -> &str { "Contact" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let re = regex::Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}").unwrap();
            if let Some(m) = re.find(&prompt) { return Some(format!("Found email: {}", m.as_str())); }
            None
        })
    }
}

// Website status plugin — quick HTTP check
pub struct WebsiteStatusPlugin;
impl Plugin for WebsiteStatusPlugin {
    fn name(&self) -> &str { "WebsiteStatus" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            if lower.starts_with("status ") || lower.starts_with("check ") {
                let parts: Vec<_> = prompt.split_whitespace().collect();
                if let Some(url) = parts.get(1) {
                    let client = Client::new();
                    if let Ok(r) = client.get(*url).send().await {
                        return Some(format!("{} -> {}", url, r.status()));
                    }
                    return Some(format!("Failed to reach {}", url));
                }
            }
            None
        })
    }
}

// Todo plugin — simple list stored in jeebs_store under key `todo:list`
pub struct TodoPlugin;
impl Plugin for TodoPlugin {
    fn name(&self) -> &str { "Todo" }
    fn handle(&self, prompt: String, db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            let key = "todo:list".to_string();
            if lower.starts_with("todo add ") {
                let item = prompt[9..].trim();
                let mut items: Vec<String> = Vec::new();
                if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(&key).fetch_optional(&db).await {
                    let val: Vec<u8> = row.get(0);
                    if let Ok(bytes) = decode_all(&val) {
                        if let Ok(existing) = serde_json::from_slice::<Vec<String>>(&bytes) { items = existing; }
                    }
                }
                items.push(item.to_string());
                if let Ok(bytes) = serde_json::to_vec(&items) {
                    if let Ok(enc) = encode_all(&bytes, 1) {
                        let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)").bind(&key).bind(enc).execute(&db).await;
                        return Some("Todo added.".to_string());
                    }
                }
                return Some("Failed to add todo.".to_string());
            } else if lower.starts_with("todo list") {
                if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(&key).fetch_optional(&db).await {
                    let val: Vec<u8> = row.get(0);
                    if let Ok(bytes) = decode_all(&val) {
                        if let Ok(items) = serde_json::from_slice::<Vec<String>>(&bytes) {
                            let out = items.into_iter().enumerate().map(|(i, s)| format!("{}: {}", i + 1, s)).collect::<Vec<_>>().join("\n");
                            return Some(out);
                        }
                    }
                }
                return Some("No todos.".to_string());
            } else if lower.starts_with("todo remove ") {
                if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(&key).fetch_optional(&db).await {
                    let val: Vec<u8> = row.get(0);
                    if let Ok(bytes) = decode_all(&val) {
                        if let Ok(mut items) = serde_json::from_slice::<Vec<String>>(&bytes) {
                            if let Ok(idx) = prompt[12..].trim().parse::<usize>() {
                                if idx > 0 && idx <= items.len() {
                                    items.remove(idx-1);
                                    if let Ok(bytes2) = serde_json::to_vec(&items) {
                                        if let Ok(enc) = encode_all(&bytes2, 1) {
                                            let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)").bind(&key).bind(enc).execute(&db).await;
                                            return Some("Removed.".to_string());
                                        }
                                    }
                                }
                            }
                            return Some("Invalid index".to_string());
                        }
                    }
                }
                return Some("No todos to remove.".to_string());
            }
            None
        })
    }
}

// Error plugin (small test helper)
pub struct ErrorPlugin;
impl Plugin for ErrorPlugin {
    fn name(&self) -> &str { "ErrorTest" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            if prompt.to_lowercase().contains("trigger error") { Some("Error: simulated failure".to_string()) } else { None }
        })
    }
}

// Dynamic plugin loader (no-op; runtime plugins may be added to `/plugins` directory)
pub fn load_dynamic_plugins(_dir: &str) -> Vec<Box<dyn Plugin>> { Vec::new() }
>>>>>>> feat/dev-container-ci

use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::pin::Pin;
use std::future::Future;
use sqlx::SqlitePool;
use serde_json::Value;
use crate::plugins::Plugin;

#[derive(Clone)]
pub struct NewsPlugin {
    cache: Arc<Mutex<Option<(String, Instant)>>>,
}

impl NewsPlugin {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(None)),
        }
    }
}

impl Plugin for NewsPlugin {
    fn name(&self) -> &str {
        "News"
    }

    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        let cache = self.cache.clone();
        Box::pin(async move {
            let prompt_lower = prompt.to_lowercase();
            if prompt_lower.contains("news") || prompt_lower.contains("headlines") {
                {
                    let lock = cache.lock().unwrap();
                    if let Some((content, timestamp)) = &*lock {
                        if timestamp.elapsed() < Duration::from_secs(900) {
                            return Some(format!("(Cached) {}", content));
                        }
                    }
                }

                let client = reqwest::Client::new();
                if let Ok(resp) = client.get("https://hacker-news.firebaseio.com/v0/topstories.json").send().await {
                    if let Ok(ids) = resp.json::<Vec<u64>>().await {
                        let mut headlines = Vec::new();
                        for id in ids.iter().take(5) {
                            let item_url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
                            if let Ok(item_resp) = client.get(&item_url).send().await {
                                if let Ok(item) = item_resp.json::<Value>().await {
                                    if let Some(title) = item["title"].as_str() {
                                        let url = item["url"].as_str().unwrap_or("");
                                        headlines.push(format!("- {} ({})", title, url));
                                    }
                                }
                            }
                        }
                        if !headlines.is_empty() {
                            let content = format!("Latest Hacker News:\n{}", headlines.join("\n"));
                            let mut lock = cache.lock().unwrap();
                            *lock = Some((content.clone(), Instant::now()));
                            return Some(content);
                        }
                    }
                }
                return Some("Failed to fetch news.".to_string());
            }
            None
        })
    }
}
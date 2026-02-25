use reqwest::Response;
use reqwest::header;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub link: String,
    pub snippet: String,
}

pub async fn google_search(query: &str, api_key: &str, cx: &str) -> Result<Vec<SearchResult>, String> {
    let url = format!(
        "https://www.googleapis.com/customsearch/v1?q={}&key={}&cx={}",
        urlencoding::encode(query), api_key, cx
    );
    let client = Client::new();
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Google search failed: {}", resp.status()));
    }
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    if let Some(items) = json.get("items").and_then(|v| v.as_array()) {
        for item in items {
            let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let link = item.get("link").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let snippet = item.get("snippet").and_then(|v| v.as_str()).unwrap_or("").to_string();
            results.push(SearchResult { title, link, snippet });
        }
    }
    Ok(results)
}

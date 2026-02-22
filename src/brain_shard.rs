use once_cell::sync::OnceCell;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode};
use sqlx::MySqlPool;
use std::path::PathBuf;
use std::str::FromStr;
use std::collections::HashSet;

// Default points to the provided shard DB `brain_shard_1` on the Databases instance.
const DEFAULT_MYSQL_URL: &str = "mysql://admin:L1QbNDvv@mysql-208625-0.cloudclusters.net:10060/brain_shard_1?ssl-mode=REQUIRED";
static MYSQL_POOL: OnceCell<MySqlPool> = OnceCell::new();

pub async fn init_pool_from_env() -> Option<MySqlPool> {
    let url = std::env::var("MYSQL_BRAIN_URL").unwrap_or_else(|_| DEFAULT_MYSQL_URL.to_string());

    let mut opts = match MySqlConnectOptions::from_str(&url) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("[brain_shard] invalid MYSQL_BRAIN_URL: {err}");
            return None;
        }
    };

    let ca_path = std::env::var("MYSQL_SSL_CA").ok();
    if let Some(path) = ca_path {
        opts = opts.ssl_mode(MySqlSslMode::Required).ssl_ca(PathBuf::from(path));
    } else {
        opts = opts.ssl_mode(MySqlSslMode::Preferred);
    }

    match MySqlPoolOptions::new()
        .max_connections(12)
        .min_connections(1)
        .connect_with(opts)
        .await
    {
        Ok(pool) => {
            if let Err(err) = ensure_schema(&pool).await {
                eprintln!("[brain_shard] failed to ensure schema: {err}");
            }
            Some(pool)
        }
        Err(err) => {
            eprintln!("[brain_shard] failed to connect to MySQL shard: {err}");
            None
        }
    }
}

pub async fn global_pool() -> Option<&'static MySqlPool> {
    if let Some(existing) = MYSQL_POOL.get() {
        return Some(existing);
    }
    if let Some(pool) = init_pool_from_env().await {
        let _ = MYSQL_POOL.set(pool);
        return MYSQL_POOL.get();
    }
    None
}

pub async fn ensure_schema(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    // Create table if missing; store topic, content, tags, and source.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS brain_shard_entries (
            id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            topic VARCHAR(255) NOT NULL,
            content TEXT NOT NULL,
            source VARCHAR(255) DEFAULT 'jeebs',
            tags TEXT DEFAULT '',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;",
    )
    .execute(pool)
    .await?;

    // Lightweight indexes for fast lookup
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_shard_topic ON brain_shard_entries (topic(191));")
        .execute(pool)
        .await
        .ok();
    sqlx::query("CREATE FULLTEXT INDEX IF NOT EXISTS idx_shard_content ON brain_shard_entries (content);")
        .execute(pool)
        .await
        .ok();

    Ok(())
}

pub async fn store_entry(
    pool: &MySqlPool,
    topic: &str,
    content: &str,
    source: &str,
    tags: &[String],
) {
    let tags_joined = if tags.is_empty() {
        "".to_string()
    } else {
        tags.join(",")
    };

    let _ = sqlx::query(
        "INSERT INTO brain_shard_entries (topic, content, source, tags) VALUES (?, ?, ?, ?)",
    )
    .bind(topic)
    .bind(content)
    .bind(source)
    .bind(tags_joined)
    .execute(pool)
    .await;
}

pub async fn store_entry_global(
    topic: &str,
    content: &str,
    source: &str,
    tags: &[String],
) {
    if let Some(pool) = global_pool().await {
        store_entry(pool, topic, content, source, tags).await;
    }
}

pub async fn search_entries(
    pool: &MySqlPool,
    terms: &[String],
    limit: usize,
) -> Result<Vec<(String, String, String, String)>, sqlx::Error> {
    let mut results = Vec::new();
    for term in terms {
        let pattern = format!("%{}%", term);
        let rows = sqlx::query(
            "SELECT topic, content, source, tags
             FROM brain_shard_entries
             WHERE topic LIKE ? OR content LIKE ?
             ORDER BY created_at DESC
             LIMIT ?",
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?;

        for row in rows {
            let topic: String = row.get(0);
            let content: String = row.get(1);
            let source: String = row.get(2);
            let tags: String = row.get(3);
            results.push((topic, content, source, tags));
        }
    }
    Ok(results)
}

/// Retrieve a batch of entries for offline linking/logic building
pub async fn fetch_recent_entries(
    pool: &MySqlPool,
    limit: usize,
) -> Result<Vec<(String, String, String, String)>, sqlx::Error> {
    // Use covering index on created_at for recency and avoid large scans.
    let rows = sqlx::query(
        "SELECT topic, content, source, tags
         FROM brain_shard_entries
         ORDER BY created_at DESC
         LIMIT ?",
    )
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;

    let mut out = Vec::new();
    for row in rows {
        let topic: String = row.get(0);
        let content: String = row.get(1);
        let source: String = row.get(2);
        let tags: String = row.get(3);
        out.push((topic, content, source, tags));
    }
    Ok(out)
}

pub async fn fetch_recent_entries_global(limit: usize) -> Vec<(String, String, String, String)> {
    if let Some(pool) = global_pool().await {
        return fetch_recent_entries(pool, limit).await.unwrap_or_default();
    }
    Vec::new()
}

fn simple_overlap_score(a_tags: &[String], b_tags: &[String], a_topic: &str, b_topic: &str) -> f32 {
    let a: HashSet<_> = a_tags.iter().map(|t| t.to_ascii_lowercase()).collect();
    let b: HashSet<_> = b_tags.iter().map(|t| t.to_ascii_lowercase()).collect();
    let inter = a.intersection(&b).count() as f32;
    let topic_bonus = if a_topic.to_ascii_lowercase() == b_topic.to_ascii_lowercase() {
        1.0
    } else {
        0.0
    };
    inter * 1.5 + topic_bonus
}

/// Suggest logical links between stored entries for later proposal/logic-building
pub async fn suggest_links(limit: usize, max_links: usize) -> Vec<(String, String, f32)> {
    let entries = fetch_recent_entries_global(limit).await;
    let mut links = Vec::new();

    for i in 0..entries.len() {
        for j in (i + 1)..entries.len() {
            let (a_topic, _, _, a_tags_str) = &entries[i];
            let (b_topic, _, _, b_tags_str) = &entries[j];
            let a_tags: Vec<String> = a_tags_str
                .split(',')
                .filter(|t| !t.is_empty())
                .map(|t| t.trim().to_string())
                .collect();
            let b_tags: Vec<String> = b_tags_str
                .split(',')
                .filter(|t| !t.is_empty())
                .map(|t| t.trim().to_string())
                .collect();

            let score = simple_overlap_score(&a_tags, &b_tags, a_topic, b_topic);
            if score >= 1.0 {
                links.push((a_topic.clone(), b_topic.clone(), score));
            }
        }
    }

    links.sort_by(|x, y| y.2.partial_cmp(&x.2).unwrap_or(std::cmp::Ordering::Equal));
    links.truncate(max_links);
    links
}

pub async fn search_entries_global(
    terms: &[String],
    limit: usize,
) -> Vec<(String, String, String, String)> {
    if let Some(pool) = global_pool().await {
        return search_entries(pool, terms, limit).await.unwrap_or_default();
    }
    Vec::new()
}

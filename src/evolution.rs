use crate::state::AppState;
use crate::utils::{decode_all, encode_all};
use actix_session::Session;
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use chrono::{DateTime, Duration as ChronoDuration, Local, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Component, Path};
use std::time::Duration;

const UPDATE_KEY_PREFIX: &str = "evolution:update:";
const NOTIFICATION_KEY_PREFIX: &str = "notification:";
const STATE_KEY: &str = "evolution:runtime:state";
const DEFAULT_THINK_INTERVAL_SECS: u64 = 300;
const DEFAULT_MIN_PROPOSAL_INTERVAL_SECS: i64 = 900;
const MAX_PENDING_UPDATES: usize = 12;
const MAX_CHANGE_BYTES: usize = 200_000;
const MAX_TOTAL_CHANGE_BYTES: usize = 1_000_000;
const MAX_NOTIFICATIONS: usize = 200;
const MAX_TOPIC_CANDIDATES: usize = 8;

fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileChange {
    pub path: String,
    pub new_content: String,
    #[serde(default = "default_true")]
    pub existed_before: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Comment {
    pub author: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Notification {
    pub id: String,
    pub message: String,
    pub severity: String,
    pub created_at: String,
    pub link: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProposedUpdate {
    pub id: String,
    pub title: String,
    pub author: String,
    pub severity: String,
    #[serde(default)]
    pub comments: Vec<Comment>,
    pub description: String,
    #[serde(default)]
    pub changes: Vec<FileChange>,
    pub status: String, // "pending", "applied", "denied", "resolved", "rolled_back"
    pub created_at: String,
    #[serde(default)]
    pub backup: Option<Vec<FileChange>>,
    #[serde(default)]
    pub auto_generated: bool,
    #[serde(default)]
    pub rationale: Vec<String>,
    #[serde(default)]
    pub source_signals: Vec<String>,
    #[serde(default)]
    pub confidence: f32,
    #[serde(default)]
    pub fingerprint: String,
    #[serde(default)]
    pub requires_restart: bool,
    #[serde(default)]
    pub applied_at: Option<String>,
    #[serde(default)]
    pub denied_at: Option<String>,
    #[serde(default)]
    pub resolved_at: Option<String>,
    #[serde(default)]
    pub rolled_back_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EvolutionRuntimeState {
    pub started_at: String,
    pub last_cycle_at: Option<String>,
    pub last_proposal_at: Option<String>,
    pub total_cycles: u64,
    pub total_proposals: u64,
    pub empty_cycles: u64,
    pub duplicate_skips: u64,
    pub last_reason: String,
    pub status: String,
}

impl EvolutionRuntimeState {
    fn new() -> Self {
        Self {
            started_at: Local::now().to_rfc3339(),
            last_cycle_at: None,
            last_proposal_at: None,
            total_cycles: 0,
            total_proposals: 0,
            empty_cycles: 0,
            duplicate_skips: 0,
            last_reason: "Initialized".to_string(),
            status: "running".to_string(),
        }
    }
}

#[derive(Serialize)]
struct UpdatesResponse {
    updates: Vec<ProposedUpdate>,
    role: String,
}

#[derive(Serialize)]
struct EvolutionStatusResponse {
    state: EvolutionRuntimeState,
    pending_updates: usize,
    total_updates: usize,
    warn_last_24h: i64,
    error_last_24h: i64,
    brain_nodes_count: i64,
    unanswered_questions_last_24h: i64,
    top_unknown_topics: Vec<String>,
}

#[derive(Clone)]
struct SignalSnapshot {
    warn_last_24h: i64,
    error_last_24h: i64,
    chat_logs_last_24h: i64,
    learned_fact_count: i64,
    recent_fact_samples: Vec<String>,
    brain_nodes_count: i64,
    unanswered_questions_last_24h: i64,
    top_unknown_topics: Vec<String>,
    pending_updates: usize,
}

#[derive(Serialize, Deserialize, Clone)]
struct ConversationTurnLite {
    role: String,
    content: String,
    timestamp: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct LearnedFactRecord {
    fact: String,
    canonical: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct ThinkResponse {
    message: String,
    id: Option<String>,
    created_update: bool,
    reason: String,
    state: EvolutionRuntimeState,
}

struct ThinkOutcome {
    created_update: bool,
    update_id: Option<String>,
    reason: String,
    state: EvolutionRuntimeState,
}

fn require_root_admin(session: &Session) -> Result<String, HttpResponse> {
    if !crate::auth::is_root_admin_session(session) {
        return Err(
            HttpResponse::Forbidden().json(json!({"error": "Restricted to 1090mb admin account"}))
        );
    }

    Ok(session
        .get::<String>("username")
        .ok()
        .flatten()
        .unwrap_or_else(|| crate::auth::ROOT_ADMIN_USERNAME.to_string()))
}

fn require_manual_proposal_actor(session: &Session) -> Result<String, HttpResponse> {
    let actor = require_root_admin(session)?;
    if actor != crate::auth::ROOT_ADMIN_USERNAME {
        return Err(HttpResponse::Forbidden().json(
            json!({"error": "Only the 1090mb admin account can apply or approve proposals"}),
        ));
    }

    let actor_lower = actor.to_ascii_lowercase();
    if actor_lower.contains("jeebs")
        || actor_lower.contains("autonomy")
        || actor_lower.contains("scheduler")
    {
        return Err(HttpResponse::Forbidden().json(json!({
            "error": "Autonomous actors are blocked from proposal approval/apply actions"
        })));
    }

    Ok(actor)
}

fn update_key(id: &str) -> String {
    format!("{UPDATE_KEY_PREFIX}{id}")
}

fn notification_key(id: &str) -> String {
    format!("{NOTIFICATION_KEY_PREFIX}{id}")
}

fn now_rfc3339() -> String {
    Local::now().to_rfc3339()
}

fn parse_env_bool(name: &str, default: bool) -> bool {
    match env::var(name) {
        Ok(raw) => match raw.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => default,
        },
        Err(_) => default,
    }
}

fn parse_env_u64(name: &str, default: u64, min: u64, max: u64) -> u64 {
    let raw = env::var(name).ok();
    let parsed = raw
        .as_deref()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(default);
    parsed.clamp(min, max)
}

fn seconds_since(timestamp: &str) -> Option<i64> {
    let parsed = DateTime::parse_from_rfc3339(timestamp).ok()?;
    Some((Utc::now() - parsed.with_timezone(&Utc)).num_seconds())
}

fn decode_json<T: DeserializeOwned>(value: &[u8]) -> Option<T> {
    if let Ok(decoded) = decode_all(value) {
        if let Ok(parsed) = serde_json::from_slice::<T>(&decoded) {
            return Some(parsed);
        }
    }

    serde_json::from_slice::<T>(value).ok()
}

fn encode_json<T: Serialize>(value: &T) -> Option<Vec<u8>> {
    let bytes = serde_json::to_vec(value).ok()?;
    encode_all(&bytes, 1).ok()
}

fn normalize_whitespace(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn strip_generated_lines(input: &str) -> String {
    input
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.starts_with("Generated at:")
                && !line.starts_with("Generated On:")
                && !line.starts_with("Run Timestamp:")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn strip_timestamp_prefix(filename: &str) -> String {
    let bytes = filename.as_bytes();
    if bytes.len() > 16
        && bytes.get(8) == Some(&b'-')
        && bytes[..8].iter().all(|b| b.is_ascii_digit())
        && bytes[9..15].iter().all(|b| b.is_ascii_digit())
        && bytes.get(15) == Some(&b'-')
    {
        return filename[16..].to_string();
    }
    filename.to_string()
}

fn canonicalize_change_path(path: &str) -> String {
    let p = Path::new(path);
    let parent = p.parent().map(|v| v.to_string_lossy().to_string());
    let file = p
        .file_name()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());
    let stripped = strip_timestamp_prefix(&file);
    if let Some(parent) = parent {
        if parent.is_empty() || parent == "." {
            stripped
        } else {
            format!("{parent}/{stripped}")
        }
    } else {
        stripped
    }
}

fn update_fingerprint(update: &ProposedUpdate) -> String {
    let mut source = String::new();
    source.push_str(&normalize_whitespace(&update.title).to_ascii_lowercase());
    source.push('\n');
    source.push_str(&normalize_whitespace(&update.description).to_ascii_lowercase());
    source.push('\n');
    for signal in &update.source_signals {
        source.push_str(&normalize_whitespace(signal).to_ascii_lowercase());
        source.push('\n');
    }
    for rationale in &update.rationale {
        source.push_str(&normalize_whitespace(rationale).to_ascii_lowercase());
        source.push('\n');
    }
    for change in &update.changes {
        source.push_str(&canonicalize_change_path(&change.path).to_ascii_lowercase());
        source.push('\n');
        let normalized_content = strip_generated_lines(&change.new_content);
        source.push_str(
            &blake3::hash(normalized_content.as_bytes())
                .to_hex()
                .to_string(),
        );
        source.push('\n');
    }
    blake3::hash(source.as_bytes()).to_hex().to_string()
}

fn normalize_update(mut update: ProposedUpdate) -> ProposedUpdate {
    update.title = normalize_whitespace(&update.title);
    update.description = normalize_whitespace(&update.description);
    if update.comments.is_empty() {
        update.comments = Vec::new();
    }
    if update.fingerprint.trim().is_empty() {
        update.fingerprint = update_fingerprint(&update);
    }
    if update.confidence < 0.0 {
        update.confidence = 0.0;
    }
    if update.confidence > 1.0 {
        update.confidence = 1.0;
    }
    update
}

fn allowed_change_path(path: &str) -> bool {
    // Keep self-modification constrained to source + docs + UI assets.
    let allowed_prefixes = [
        "src/",
        "webui/",
        "migrations/",
        "scripts/",
        "evolution/",
        "README",
        "CHANGELOG",
        "Cargo.toml",
    ];
    allowed_prefixes
        .iter()
        .any(|prefix| path == *prefix || path.starts_with(prefix))
}

fn validate_change_path(path: &str) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("File path cannot be empty".to_string());
    }
    if trimmed.starts_with('/') || trimmed.starts_with('\\') || trimmed.contains("..") {
        return Err(format!("Invalid file path: {trimmed}"));
    }
    if !allowed_change_path(trimmed) {
        return Err(format!(
            "Path outside allowed self-evolution scope: {trimmed}"
        ));
    }

    let parsed = Path::new(trimmed);
    for component in parsed.components() {
        match component {
            Component::Normal(_) => {}
            _ => return Err(format!("Invalid path component in {trimmed}")),
        }
    }
    Ok(())
}

fn validate_changes(changes: &[FileChange]) -> Result<(), String> {
    if changes.is_empty() {
        return Err("Proposed update has no file changes".to_string());
    }

    let mut total_bytes = 0usize;
    let mut seen_paths = HashSet::new();
    for change in changes {
        validate_change_path(&change.path)?;
        if !seen_paths.insert(change.path.clone()) {
            return Err(format!("Duplicate change path in update: {}", change.path));
        }
        let bytes = change.new_content.as_bytes().len();
        if bytes > MAX_CHANGE_BYTES {
            return Err(format!(
                "Change for '{}' exceeds {} bytes",
                change.path, MAX_CHANGE_BYTES
            ));
        }
        total_bytes += bytes;
    }

    if total_bytes > MAX_TOTAL_CHANGE_BYTES {
        return Err(format!(
            "Total proposed content exceeds {} bytes",
            MAX_TOTAL_CHANGE_BYTES
        ));
    }

    Ok(())
}

fn create_backup_snapshot(changes: &[FileChange]) -> Result<Vec<FileChange>, String> {
    let mut backups = Vec::with_capacity(changes.len());
    for change in changes {
        let path = Path::new(&change.path);
        if path.exists() {
            let content = fs::read_to_string(path).map_err(|err| {
                format!(
                    "Failed to read existing file '{}' for backup: {err}",
                    change.path
                )
            })?;
            backups.push(FileChange {
                path: change.path.clone(),
                new_content: content,
                existed_before: true,
            });
        } else {
            backups.push(FileChange {
                path: change.path.clone(),
                new_content: String::new(),
                existed_before: false,
            });
        }
    }
    Ok(backups)
}

fn restore_prior_versions(prior: &[(String, Option<String>)]) {
    for (path, old_content) in prior.iter().rev() {
        if let Some(content) = old_content {
            let _ = fs::write(path, content);
        } else {
            let _ = fs::remove_file(path);
        }
    }
}

fn apply_changes_atomically(changes: &[FileChange]) -> Result<(), String> {
    let mut prior_versions: Vec<(String, Option<String>)> = Vec::with_capacity(changes.len());

    for change in changes {
        let path = Path::new(&change.path);
        let prior =
            if path.exists() {
                Some(fs::read_to_string(path).map_err(|err| {
                    format!("Failed to read existing file '{}': {err}", change.path)
                })?)
            } else {
                None
            };
        prior_versions.push((change.path.clone(), prior));

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "Failed to create parent directory for '{}': {err}",
                    change.path
                )
            })?;
        }
        if let Err(err) = fs::write(path, &change.new_content) {
            restore_prior_versions(&prior_versions);
            return Err(format!("Failed to write '{}': {err}", change.path));
        }
    }

    Ok(())
}

fn restore_backup_snapshot(backups: &[FileChange]) -> Result<(), String> {
    for backup in backups.iter().rev() {
        let path = Path::new(&backup.path);
        if backup.existed_before {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|err| {
                    format!("Failed to recreate directory for '{}': {err}", backup.path)
                })?;
            }
            fs::write(path, &backup.new_content)
                .map_err(|err| format!("Failed to restore '{}': {err}", backup.path))?;
        } else if path.exists() {
            fs::remove_file(path)
                .map_err(|err| format!("Failed to remove new file '{}': {err}", backup.path))?;
        }
    }
    Ok(())
}

async fn load_runtime_state(db: &SqlitePool) -> EvolutionRuntimeState {
    match sqlx::query("SELECT value FROM jeebs_store WHERE key = ? LIMIT 1")
        .bind(STATE_KEY)
        .fetch_optional(db)
        .await
    {
        Ok(Some(row)) => {
            let value: Vec<u8> = row.get(0);
            decode_json::<EvolutionRuntimeState>(&value).unwrap_or_else(EvolutionRuntimeState::new)
        }
        _ => EvolutionRuntimeState::new(),
    }
}

async fn save_runtime_state(db: &SqlitePool, state: &EvolutionRuntimeState) {
    if let Some(value) = encode_json(state) {
        let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
            .bind(STATE_KEY)
            .bind(value)
            .execute(db)
            .await;
    }
}

async fn load_all_updates(db: &SqlitePool) -> Vec<ProposedUpdate> {
    let rows = sqlx::query("SELECT key, value FROM jeebs_store WHERE key LIKE ?")
        .bind(format!("{UPDATE_KEY_PREFIX}%"))
        .fetch_all(db)
        .await
        .unwrap_or_default();

    let mut updates = Vec::new();
    for row in rows {
        let value: Vec<u8> = row.get(1);
        if let Some(update) = decode_json::<ProposedUpdate>(&value) {
            updates.push(normalize_update(update));
        }
    }

    updates.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    updates
}

async fn load_update_by_id(db: &SqlitePool, id: &str) -> Option<ProposedUpdate> {
    let key = update_key(id);
    let row = sqlx::query("SELECT value FROM jeebs_store WHERE key = ? LIMIT 1")
        .bind(&key)
        .fetch_optional(db)
        .await
        .ok()??;

    let value: Vec<u8> = row.get(0);
    decode_json::<ProposedUpdate>(&value).map(normalize_update)
}

async fn save_update(db: &SqlitePool, update: &ProposedUpdate) -> Result<(), String> {
    let key = update_key(&update.id);
    let encoded = encode_json(update).ok_or_else(|| "Failed to encode update".to_string())?;
    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(key)
        .bind(encoded)
        .execute(db)
        .await
        .map_err(|err| format!("Failed to save update: {err}"))?;
    Ok(())
}

async fn save_notification(db: &SqlitePool, notif: &Notification) {
    let key = notification_key(&notif.id);
    if let Some(encoded) = encode_json(notif) {
        let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(encoded)
            .execute(db)
            .await;
    }
}

async fn trim_notifications(db: &SqlitePool) {
    let rows = sqlx::query("SELECT key, value FROM jeebs_store WHERE key LIKE ?")
        .bind(format!("{NOTIFICATION_KEY_PREFIX}%"))
        .fetch_all(db)
        .await
        .unwrap_or_default();

    let mut notifications = Vec::new();
    for row in rows {
        let key: String = row.get(0);
        let value: Vec<u8> = row.get(1);
        if let Some(notif) = decode_json::<Notification>(&value) {
            notifications.push((key, notif.created_at));
        }
    }

    if notifications.len() <= MAX_NOTIFICATIONS {
        return;
    }

    notifications.sort_by(|a, b| b.1.cmp(&a.1));
    for (key, _) in notifications.into_iter().skip(MAX_NOTIFICATIONS) {
        let _ = sqlx::query("DELETE FROM jeebs_store WHERE key = ?")
            .bind(key)
            .execute(db)
            .await;
    }
}

async fn create_notification(
    db: &SqlitePool,
    message: String,
    severity: &str,
    link: Option<String>,
) {
    let notif = Notification {
        id: uuid::Uuid::new_v4().to_string(),
        message,
        severity: severity.to_string(),
        created_at: now_rfc3339(),
        link,
    };
    save_notification(db, &notif).await;
    trim_notifications(db).await;
}

async fn count_logs_for_level(db: &SqlitePool, level: &str, since: &str) -> i64 {
    sqlx::query("SELECT COUNT(*) FROM system_logs WHERE level = ? AND timestamp >= ?")
        .bind(level)
        .bind(since)
        .fetch_one(db)
        .await
        .ok()
        .map(|row| row.get::<i64, _>(0))
        .unwrap_or(0)
}

async fn count_chat_logs(db: &SqlitePool, since: &str) -> i64 {
    sqlx::query("SELECT COUNT(*) FROM system_logs WHERE category = 'CHAT' AND timestamp >= ?")
        .bind(since)
        .fetch_one(db)
        .await
        .ok()
        .map(|row| row.get::<i64, _>(0))
        .unwrap_or(0)
}

async fn count_brain_nodes(db: &SqlitePool) -> i64 {
    sqlx::query("SELECT COUNT(*) FROM brain_nodes")
        .fetch_one(db)
        .await
        .ok()
        .map(|row| row.get::<i64, _>(0))
        .unwrap_or(0)
}

async fn count_learned_facts(db: &SqlitePool) -> i64 {
    sqlx::query("SELECT COUNT(*) FROM jeebs_store WHERE key LIKE 'chat:fact:%'")
        .fetch_one(db)
        .await
        .ok()
        .map(|row| row.get::<i64, _>(0))
        .unwrap_or(0)
}

fn is_topic_stopword(token: &str) -> bool {
    matches!(
        token,
        "the"
            | "a"
            | "an"
            | "and"
            | "or"
            | "of"
            | "to"
            | "for"
            | "in"
            | "on"
            | "is"
            | "are"
            | "my"
            | "me"
            | "you"
            | "your"
            | "what"
            | "which"
            | "where"
            | "when"
            | "why"
            | "how"
            | "who"
            | "do"
            | "does"
            | "did"
            | "can"
            | "could"
            | "would"
            | "should"
            | "tell"
            | "about"
            | "please"
    )
}

fn extract_topic_key(question: &str) -> String {
    let mut normalized = String::with_capacity(question.len());
    for ch in question.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
        } else {
            normalized.push(' ');
        }
    }

    let tokens = normalized
        .split_whitespace()
        .filter(|token| token.len() >= 3 && !is_topic_stopword(token))
        .take(6)
        .map(|token| token.to_string())
        .collect::<Vec<_>>();

    if !tokens.is_empty() {
        return tokens.join(" ");
    }

    normalize_whitespace(question)
        .trim_end_matches('?')
        .to_ascii_lowercase()
}

fn parse_rfc3339_to_utc(ts: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn is_unknown_assistant_reply(content: &str) -> bool {
    let lower = content.to_ascii_lowercase();
    lower.contains("i am still learning that topic")
        || lower.contains("keep chatting with me and i will help")
        || lower.contains("could not match that to a saved detail yet")
}

async fn recent_learned_fact_samples(db: &SqlitePool, limit: usize) -> Vec<String> {
    let rows = sqlx::query("SELECT value FROM jeebs_store WHERE key LIKE 'chat:fact:%'")
        .fetch_all(db)
        .await
        .unwrap_or_default();

    let mut facts: Vec<(String, String)> = Vec::new();
    for row in rows {
        let value: Vec<u8> = row.get(0);
        if let Some(record) = decode_json::<LearnedFactRecord>(&value) {
            let fact = normalize_whitespace(&record.fact);
            if !fact.is_empty() {
                facts.push((record.updated_at, fact));
            }
        }
    }

    facts.sort_by(|a, b| b.0.cmp(&a.0));
    facts
        .into_iter()
        .map(|(_, fact)| fact)
        .take(limit)
        .collect()
}

async fn unknown_question_topics(db: &SqlitePool, since: &str) -> (i64, Vec<String>) {
    let rows = sqlx::query("SELECT value FROM jeebs_store WHERE key LIKE 'chat:history:%'")
        .fetch_all(db)
        .await
        .unwrap_or_default();

    let since_dt = parse_rfc3339_to_utc(since);
    let mut unknown_question_count = 0_i64;
    let mut topic_counts: HashMap<String, usize> = HashMap::new();

    for row in rows {
        let value: Vec<u8> = row.get(0);
        let Some(history) = decode_json::<Vec<ConversationTurnLite>>(&value) else {
            continue;
        };

        for idx in 0..history.len() {
            let turn = &history[idx];
            if turn.role != "user" {
                continue;
            }
            if !turn.content.trim_end().ends_with('?') {
                continue;
            }

            if let Some(ref since_dt) = since_dt {
                let Some(turn_ts) = parse_rfc3339_to_utc(&turn.timestamp) else {
                    continue;
                };
                if turn_ts < since_dt.to_owned() {
                    continue;
                }
            }

            let mut next_assistant: Option<&ConversationTurnLite> = None;
            for follow in history.iter().skip(idx + 1) {
                if follow.role == "assistant" {
                    next_assistant = Some(follow);
                    break;
                }
            }

            let Some(assistant_turn) = next_assistant else {
                continue;
            };

            if !is_unknown_assistant_reply(&assistant_turn.content) {
                continue;
            }

            unknown_question_count += 1;
            let topic = extract_topic_key(&turn.content);
            if !topic.is_empty() {
                *topic_counts.entry(topic).or_insert(0) += 1;
            }
        }
    }

    let mut ordered = topic_counts.into_iter().collect::<Vec<_>>();
    ordered.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let top_topics = ordered
        .into_iter()
        .take(MAX_TOPIC_CANDIDATES)
        .map(|(topic, count)| format!("{topic} ({count})"))
        .collect();

    (unknown_question_count, top_topics)
}

async fn collect_signals(db: &SqlitePool) -> SignalSnapshot {
    let since = (Local::now() - ChronoDuration::hours(24)).to_rfc3339();
    let updates = load_all_updates(db).await;
    let pending_updates = updates.iter().filter(|u| u.status == "pending").count();
    let (unknown_q_count, top_unknown_topics) = unknown_question_topics(db, &since).await;

    SignalSnapshot {
        warn_last_24h: count_logs_for_level(db, "WARN", &since).await,
        error_last_24h: count_logs_for_level(db, "ERROR", &since).await,
        chat_logs_last_24h: count_chat_logs(db, &since).await,
        learned_fact_count: count_learned_facts(db).await,
        recent_fact_samples: recent_learned_fact_samples(db, 5).await,
        brain_nodes_count: count_brain_nodes(db).await,
        unanswered_questions_last_24h: unknown_q_count,
        top_unknown_topics,
        pending_updates,
    }
}

fn jeebs_core_likes() -> &'static [&'static str] {
    &[
        "learning new knowledge",
        "rigorous reasoning",
        "broad experimentation",
        "measurable progress",
    ]
}

fn jeebs_core_dislikes() -> &'static [&'static str] {
    &[
        "stagnation",
        "low-confidence guessing",
        "narrow repetitive proposals",
        "unverified assumptions",
    ]
}

fn jeebs_core_wants() -> &'static [&'static str] {
    &[
        "to expand knowledge coverage continuously",
        "to reduce unknown-answer rate",
        "to demonstrate smarter behavior with measurable upgrades",
    ]
}

fn knowledge_drive_score(signals: &SignalSnapshot) -> f32 {
    let unanswered_component =
        (signals.unanswered_questions_last_24h as f32 / 10.0).clamp(0.0, 1.0);
    let volume_component = (signals.chat_logs_last_24h as f32 / 80.0).clamp(0.0, 1.0);
    let coverage_ratio = if signals.chat_logs_last_24h <= 0 {
        0.0
    } else {
        (signals.brain_nodes_count as f32 / signals.chat_logs_last_24h as f32).clamp(0.0, 1.0)
    };
    let coverage_gap_component = (1.0 - coverage_ratio).clamp(0.0, 1.0);
    let topic_pressure = if signals.top_unknown_topics.is_empty() {
        0.0
    } else {
        1.0
    };

    (0.35
        + unanswered_component * 0.35
        + volume_component * 0.15
        + coverage_gap_component * 0.1
        + topic_pressure * 0.05)
        .clamp(0.0, 1.0)
}

fn ensure_broad_search_queries(search_queries: &mut Vec<String>, top_unknown_topics: &[String]) {
    for topic in top_unknown_topics.iter().take(6) {
        let label = topic_label(topic);
        if !label.is_empty() {
            let query = format!("deep research and practical explanation for {}", label);
            if !search_queries.iter().any(|existing| existing == &query) {
                search_queries.push(query);
            }
        }
    }

    let broad_queries = [
        "retrieval augmented generation evaluation metrics",
        "knowledge graph enrichment strategies",
        "conversation memory ranking and recall quality",
        "autonomous agent experiment design",
        "reliable web crawling for knowledge ingestion",
        "llm reasoning benchmark methodology",
    ];
    for query in broad_queries {
        if search_queries.len() >= 10 {
            break;
        }
        if !search_queries.iter().any(|existing| existing == query) {
            search_queries.push(query.to_string());
        }
    }
}

fn build_reflection_markdown(
    title: &str,
    reason: &str,
    rationale: &[String],
    signals: &[String],
    confidence: f32,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# {title}"));
    lines.push(String::new());
    lines.push(format!("Generated at: {}", now_rfc3339()));
    lines.push(format!("Reason: {reason}"));
    lines.push(format!("Confidence: {:.2}", confidence));
    lines.push(String::new());
    lines.push("## Observations".to_string());
    if signals.is_empty() {
        lines.push(
            "- No strong external signal was present; this is a periodic reflection.".to_string(),
        );
    } else {
        for signal in signals {
            lines.push(format!("- {signal}"));
        }
    }
    lines.push(String::new());
    lines.push("## Suggested Actions".to_string());
    if rationale.is_empty() {
        lines.push("- Review recent logs and keep watch for recurring patterns.".to_string());
    } else {
        for item in rationale {
            lines.push(format!("- {item}"));
        }
    }
    lines.push(String::new());
    lines.push("## Notes".to_string());
    lines.push("- This file was generated by Jeebs autonomous evolution thinking.".to_string());
    lines.join("\n")
}

fn topic_label(entry: &str) -> String {
    if let Some(open_idx) = entry.rfind(" (") {
        if entry.ends_with(')')
            && entry[open_idx + 2..entry.len() - 1]
                .chars()
                .all(|ch| ch.is_ascii_digit())
        {
            return entry[..open_idx].to_string();
        }
    }
    entry.to_string()
}

fn build_learning_plan_markdown(
    title: &str,
    reason: &str,
    top_unknown_topics: &[String],
    recent_fact_samples: &[String],
    search_queries: &[String],
) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# {title}"));
    lines.push(String::new());
    lines.push(format!("Generated at: {}", now_rfc3339()));
    lines.push(format!("Reason: {reason}"));
    lines.push(String::new());
    lines.push("## Conversation Gaps To Learn".to_string());
    if top_unknown_topics.is_empty() {
        lines.push("- No unresolved high-frequency unknown questions were detected.".to_string());
    } else {
        for topic in top_unknown_topics {
            lines.push(format!("- {topic}"));
        }
    }
    lines.push(String::new());
    lines.push("## Known User Facts (Most Recent Samples)".to_string());
    if recent_fact_samples.is_empty() {
        lines.push("- No recent personal fact samples found.".to_string());
    } else {
        for fact in recent_fact_samples {
            lines.push(format!("- {fact}"));
        }
    }
    lines.push(String::new());
    lines.push("## Search Queries For Knowledge Expansion".to_string());
    if search_queries.is_empty() {
        lines.push("- No explicit search queries generated this cycle.".to_string());
    } else {
        for query in search_queries {
            lines.push(format!("- {query}"));
        }
    }
    lines.push(String::new());
    lines.push("## Action".to_string());
    lines.push(
        "- Convert top gaps into new knowledge entries and validate with chat tests.".to_string(),
    );
    lines.join("\n")
}

fn build_scope_roadmap_markdown(
    title: &str,
    reason: &str,
    knowledge_drive: f32,
    top_unknown_topics: &[String],
) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# {} - Scope Expansion Roadmap", title));
    lines.push(String::new());
    lines.push(format!("Generated at: {}", now_rfc3339()));
    lines.push(format!("Reason: {reason}"));
    lines.push(format!("Knowledge Drive Score: {:.2}", knowledge_drive));
    lines.push(String::new());
    lines.push("## Jeebs Personality".to_string());
    lines.push(format!("- Likes: {}", jeebs_core_likes().join(", ")));
    lines.push(format!("- Dislikes: {}", jeebs_core_dislikes().join(", ")));
    lines.push(format!("- Wants: {}", jeebs_core_wants().join(", ")));
    lines.push(String::new());
    lines.push("## Broader Evolution Tracks".to_string());
    lines.push(
        "- Knowledge Ingestion: increase source diversity (docs, references, technical explainers)."
            .to_string(),
    );
    lines.push(
        "- Reasoning Quality: improve answer structure, confidence signaling, and follow-up logic."
            .to_string(),
    );
    lines.push(
        "- Retrieval Precision: improve node labeling, summaries, and query-to-node relevance."
            .to_string(),
    );
    lines.push(
        "- Reliability: expand tests and runbooks so improvements are stable in production."
            .to_string(),
    );
    lines.push(String::new());
    lines.push("## Priority Topics".to_string());
    if top_unknown_topics.is_empty() {
        lines.push(
            "- No dominant unknown topics this cycle; explore broad capability upgrades."
                .to_string(),
        );
    } else {
        for topic in top_unknown_topics.iter().take(8) {
            lines.push(format!("- {topic}"));
        }
    }
    lines.join("\n")
}

fn build_experiment_backlog_markdown(
    title: &str,
    knowledge_drive: f32,
    search_queries: &[String],
) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# {} - Learning Experiments", title));
    lines.push(String::new());
    lines.push(format!("Generated at: {}", now_rfc3339()));
    lines.push(format!("Knowledge Drive Score: {:.2}", knowledge_drive));
    lines.push(String::new());
    lines.push("## Experiments To Run".to_string());
    lines.push(
        "- Build a weekly benchmark of unknown questions and target a measurable reduction."
            .to_string(),
    );
    lines.push(
        "- Compare retrieval quality before/after adding new node summarization templates."
            .to_string(),
    );
    lines.push(
        "- Validate that training outputs are reusable in conversations within 1-2 cycles."
            .to_string(),
    );
    lines.push(
        "- Stress-test response quality with adversarial vague prompts and require clarifying questions."
            .to_string(),
    );
    lines.push(String::new());
    lines.push("## Research Queue".to_string());
    if search_queries.is_empty() {
        lines.push("- No queued queries yet. Add domain-focused research tasks.".to_string());
    } else {
        for query in search_queries.iter().take(12) {
            lines.push(format!("- {query}"));
        }
    }
    lines.push(String::new());
    lines.push("## Success Criteria".to_string());
    lines.push("- Fewer unknown-answer responses in chat logs over 7 days.".to_string());
    lines.push("- Higher reuse rate of learned nodes in real conversations.".to_string());
    lines.push("- Fewer repetitive proposals and more multi-track improvements.".to_string());
    lines.join("\n")
}

fn build_candidate_update(
    signals: &SignalSnapshot,
) -> (
    String,
    String,
    String,
    f32,
    f32,
    Vec<String>,
    Vec<String>,
    Vec<String>,
) {
    let mut rationale = Vec::new();
    let mut source_signals = Vec::new();
    let mut search_queries = Vec::new();
    let knowledge_drive = knowledge_drive_score(signals);

    let (title, mut severity, reason, mut confidence) = if signals.error_last_24h >= 5 {
        source_signals.push(format!(
            "{} ERROR logs were recorded in the last 24h",
            signals.error_last_24h
        ));
        rationale.push(
            "Instrument error hotspots with tighter categories and root-cause tags.".to_string(),
        );
        rationale
            .push("Add focused regression checks for failing paths before deploy.".to_string());
        search_queries.push("actix-web production error triage patterns".to_string());
        search_queries.push("rust sqlite reliability checklist".to_string());
        (
            "Autonomous Reflection: Stability hardening".to_string(),
            "High".to_string(),
            "High runtime error volume detected".to_string(),
            0.92,
        )
    } else if signals.unanswered_questions_last_24h >= 3 {
        source_signals.push(format!(
            "{} unanswered user questions were detected in the last 24h",
            signals.unanswered_questions_last_24h
        ));
        if !signals.top_unknown_topics.is_empty() {
            source_signals.push(format!(
                "Top unknown topics: {}",
                signals.top_unknown_topics.join(", ")
            ));
        }
        rationale.push(
            "Mine repeated unknown user questions and prioritize them as learning targets."
                .to_string(),
        );
        rationale.push(
            "Create new brain notes and retrieval prompts to close coverage gaps.".to_string(),
        );
        for topic in signals.top_unknown_topics.iter().take(4) {
            let label = topic_label(topic);
            if !label.is_empty() {
                search_queries.push(format!("research answer for {}", label));
            }
        }
        (
            "Autonomous Learning Sprint: close conversation gaps".to_string(),
            "High".to_string(),
            "Repeated unknown question patterns detected".to_string(),
            0.89,
        )
    } else if signals.brain_nodes_count < 25 && signals.chat_logs_last_24h >= 25 {
        source_signals.push(format!(
            "brain_nodes contains only {} entries",
            signals.brain_nodes_count
        ));
        source_signals.push(format!(
            "{} chat interactions in the last 24h indicate higher coverage demand",
            signals.chat_logs_last_24h
        ));
        rationale.push("Expand searchable knowledge nodes to improve answer coverage.".to_string());
        rationale.push(
            "Prioritize ingestion for top requested topics from recent chat traffic.".to_string(),
        );
        for topic in signals.top_unknown_topics.iter().take(5) {
            let label = topic_label(topic);
            if !label.is_empty() {
                search_queries.push(format!("collect references for {}", label));
            }
        }
        if search_queries.is_empty() {
            search_queries.push("jeebsai knowledge base expansion roadmap".to_string());
        }
        (
            "Autonomous Search Plan: expand knowledge coverage".to_string(),
            "Medium".to_string(),
            "Knowledge base size lags behind conversation demand".to_string(),
            0.81,
        )
    } else if signals.warn_last_24h >= 15 {
        source_signals.push(format!(
            "{} WARN logs were recorded in the last 24h",
            signals.warn_last_24h
        ));
        rationale.push(
            "Convert recurring warnings into structured diagnostics with clear remediation."
                .to_string(),
        );
        rationale.push("Reduce noisy warnings that hide critical failures.".to_string());
        search_queries.push("rust warning classification playbook".to_string());
        (
            "Autonomous Reflection: Warning-noise reduction".to_string(),
            "Medium".to_string(),
            "Warning volume suggests preventable instability".to_string(),
            0.78,
        )
    } else if signals.chat_logs_last_24h >= 40 && signals.learned_fact_count < 15 {
        source_signals.push(format!(
            "{} chat interactions were recorded in the last 24h",
            signals.chat_logs_last_24h
        ));
        source_signals.push(format!(
            "Only {} learned personal facts are stored",
            signals.learned_fact_count
        ));
        rationale.push(
            "Expand memory extraction patterns to capture more useful user preferences."
                .to_string(),
        );
        rationale.push(
            "Prioritize retrieval quality for 'what do you remember about me' queries.".to_string(),
        );
        if !signals.recent_fact_samples.is_empty() {
            source_signals.push(format!(
                "Recent facts: {}",
                signals.recent_fact_samples.join(" | ")
            ));
        }
        search_queries.push("improve conversational memory retrieval ranking".to_string());
        (
            "Autonomous Reflection: Conversation memory expansion".to_string(),
            "Medium".to_string(),
            "Conversation load is high but retained memory is low".to_string(),
            0.74,
        )
    } else {
        source_signals.push("Periodic heartbeat cycle".to_string());
        source_signals.push(format!(
            "WARN={} ERROR={} CHAT={} BRAIN={} UNANSWERED_Q={}",
            signals.warn_last_24h,
            signals.error_last_24h,
            signals.chat_logs_last_24h,
            signals.brain_nodes_count,
            signals.unanswered_questions_last_24h
        ));
        rationale.push(
            "No urgent risk detected; keep monitoring and preserve learning cadence.".to_string(),
        );
        search_queries.push("periodic knowledge maintenance checklist".to_string());
        (
            "Autonomous Reflection: Heartbeat and self-check".to_string(),
            "Low".to_string(),
            "Routine autonomous thinking cycle".to_string(),
            0.61,
        )
    };

    source_signals.push(format!("Core likes: {}", jeebs_core_likes().join(", ")));
    source_signals.push(format!(
        "Core dislikes: {}",
        jeebs_core_dislikes().join(", ")
    ));
    source_signals.push(format!("Core wants: {}", jeebs_core_wants().join(", ")));
    source_signals.push(format!("Knowledge drive score: {:.2}", knowledge_drive));

    rationale.push(
        "Broaden proposal scope across ingestion, retrieval, reasoning quality, and reliability so Jeebs evolves across multiple dimensions.".to_string(),
    );
    if knowledge_drive >= 0.72 {
        rationale.push(
            "Prioritize aggressive knowledge expansion this cycle to reduce unknown-answer rate."
                .to_string(),
        );
        if severity.eq_ignore_ascii_case("low") {
            severity = "Medium".to_string();
        }
        confidence = (confidence + 0.08_f32).clamp(0.0_f32, 0.98_f32);
    }

    ensure_broad_search_queries(&mut search_queries, &signals.top_unknown_topics);

    (
        title,
        severity,
        reason,
        confidence,
        knowledge_drive,
        rationale,
        source_signals,
        search_queries,
    )
}

fn build_update_from_signals(signals: &SignalSnapshot) -> ProposedUpdate {
    let (
        title,
        severity,
        reason,
        confidence,
        knowledge_drive,
        rationale,
        source_signals,
        search_queries,
    ) = build_candidate_update(signals);
    let slug = slugify(&title);
    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let reflection_path = format!("evolution/reflections/{timestamp}-{slug}.md");
    let reflection_markdown =
        build_reflection_markdown(&title, &reason, &rationale, &source_signals, confidence);
    let plan_path = format!("evolution/learning/{timestamp}-{slug}-plan.md");
    let plan_markdown = build_learning_plan_markdown(
        &title,
        &reason,
        &signals.top_unknown_topics,
        &signals.recent_fact_samples,
        &search_queries,
    );
    let scope_path = format!("evolution/scope/{timestamp}-{slug}-scope.md");
    let scope_markdown = build_scope_roadmap_markdown(
        &title,
        &reason,
        knowledge_drive,
        &signals.top_unknown_topics,
    );
    let experiments_path = format!("evolution/experiments/{timestamp}-{slug}-experiments.md");
    let experiments_markdown =
        build_experiment_backlog_markdown(&title, knowledge_drive, &search_queries);

    let mut update = ProposedUpdate {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        author: "Jeebs Autonomy Engine".to_string(),
        severity,
        comments: Vec::new(),
        description: format!(
            "{} | knowledge_drive={:.2} | scope=ingestion+retrieval+reasoning+reliability",
            reason, knowledge_drive
        ),
        changes: vec![
            FileChange {
                path: reflection_path,
                new_content: reflection_markdown,
                existed_before: false,
            },
            FileChange {
                path: plan_path,
                new_content: plan_markdown,
                existed_before: false,
            },
            FileChange {
                path: scope_path,
                new_content: scope_markdown,
                existed_before: false,
            },
            FileChange {
                path: experiments_path,
                new_content: experiments_markdown,
                existed_before: false,
            },
        ],
        status: "pending".to_string(),
        created_at: now_rfc3339(),
        backup: None,
        auto_generated: true,
        rationale,
        source_signals,
        confidence,
        fingerprint: String::new(),
        requires_restart: false,
        applied_at: None,
        denied_at: None,
        resolved_at: None,
        rolled_back_at: None,
    };

    update.fingerprint = update_fingerprint(&update);
    update
}

fn has_duplicate_update(existing: &[ProposedUpdate], fingerprint: &str) -> bool {
    existing.iter().any(|update| {
        update.fingerprint == fingerprint
            && matches!(update.status.as_str(), "pending" | "applied" | "resolved")
    })
}

async fn run_think_cycle_internal(
    db: &SqlitePool,
    actor: &str,
    force: bool,
) -> Result<ThinkOutcome, String> {
    let min_interval_secs = parse_env_u64(
        "EVOLUTION_MIN_PROPOSAL_INTERVAL_SECS",
        DEFAULT_MIN_PROPOSAL_INTERVAL_SECS as u64,
        30,
        86_400,
    ) as i64;

    let mut state = load_runtime_state(db).await;
    let now = now_rfc3339();
    state.last_cycle_at = Some(now.clone());
    state.total_cycles = state.total_cycles.saturating_add(1);
    state.status = "running".to_string();

    let signals = collect_signals(db).await;

    if signals.pending_updates >= MAX_PENDING_UPDATES {
        state.empty_cycles = state.empty_cycles.saturating_add(1);
        state.last_reason = format!(
            "Skipped: {} pending updates already in queue",
            signals.pending_updates
        );
        crate::logging::log(db, "INFO", "EVOLUTION", &state.last_reason).await;
        save_runtime_state(db, &state).await;
        return Ok(ThinkOutcome {
            created_update: false,
            update_id: None,
            reason: state.last_reason.clone(),
            state,
        });
    }

    if !force {
        if let Some(last) = &state.last_proposal_at {
            if let Some(age) = seconds_since(last) {
                if age < min_interval_secs {
                    state.empty_cycles = state.empty_cycles.saturating_add(1);
                    state.last_reason = format!(
                        "Skipped: proposal cooldown active ({}s remaining)",
                        min_interval_secs - age
                    );
                    crate::logging::log(db, "INFO", "EVOLUTION", &state.last_reason).await;
                    save_runtime_state(db, &state).await;
                    return Ok(ThinkOutcome {
                        created_update: false,
                        update_id: None,
                        reason: state.last_reason.clone(),
                        state,
                    });
                }
            }
        }
    }

    let mut candidate = build_update_from_signals(&signals);
    // Autonomous proposals are always staged for explicit human review.
    candidate.status = "pending".to_string();
    candidate.backup = None;
    candidate.applied_at = None;
    candidate.denied_at = None;
    candidate.resolved_at = None;
    candidate.rolled_back_at = None;
    candidate = normalize_update(candidate);
    validate_changes(&candidate.changes)?;

    let existing = load_all_updates(db).await;
    if has_duplicate_update(&existing, &candidate.fingerprint) {
        state.duplicate_skips = state.duplicate_skips.saturating_add(1);
        state.last_reason = "Skipped: duplicate evolution proposal fingerprint".to_string();
        crate::logging::log(db, "INFO", "EVOLUTION", &state.last_reason).await;
        save_runtime_state(db, &state).await;
        return Ok(ThinkOutcome {
            created_update: false,
            update_id: None,
            reason: state.last_reason.clone(),
            state,
        });
    }

    save_update(db, &candidate).await?;

    if candidate.severity.eq_ignore_ascii_case("high")
        || candidate.severity.eq_ignore_ascii_case("medium")
    {
        create_notification(
            db,
            format!(
                "Jeebs proposed '{}' ({})",
                candidate.title, candidate.severity
            ),
            &candidate.severity,
            Some("/webui/evolution.html".to_string()),
        )
        .await;
    }

    state.total_proposals = state.total_proposals.saturating_add(1);
    state.last_proposal_at = Some(now);
    state.last_reason = format!(
        "Created proposal '{}' from actor {}",
        candidate.title, actor
    );
    save_runtime_state(db, &state).await;

    crate::logging::log(
        db,
        "INFO",
        "EVOLUTION",
        &format!(
            "Autonomous think cycle created proposal '{}' ({}) by {actor}",
            candidate.title, candidate.id
        ),
    )
    .await;

    Ok(ThinkOutcome {
        created_update: true,
        update_id: Some(candidate.id),
        reason: state.last_reason.clone(),
        state,
    })
}

pub fn spawn_autonomous_thinker(db: SqlitePool) {
    let enabled = parse_env_bool("EVOLUTION_AUTONOMOUS", true);
    if !enabled {
        tokio::spawn(async move {
            crate::logging::log(
                &db,
                "INFO",
                "EVOLUTION",
                "Autonomous thinker disabled via EVOLUTION_AUTONOMOUS=false",
            )
            .await;
        });
        return;
    }

    let interval_secs = parse_env_u64(
        "EVOLUTION_THINK_INTERVAL_SECS",
        DEFAULT_THINK_INTERVAL_SECS,
        30,
        86_400,
    );

    tokio::spawn(async move {
        crate::logging::log(
            &db,
            "INFO",
            "EVOLUTION",
            &format!("Autonomous thinker started with {interval_secs}s interval"),
        )
        .await;

        loop {
            if let Err(err) = run_think_cycle_internal(&db, "scheduler", false).await {
                crate::logging::log(
                    &db,
                    "WARN",
                    "EVOLUTION",
                    &format!("Autonomous thinker cycle failed: {err}"),
                )
                .await;
            }
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        }
    });
}

#[get("/api/admin/evolution/updates")]
pub async fn list_updates(data: web::Data<AppState>, session: Session) -> impl Responder {
    if let Err(response) = require_root_admin(&session) {
        return response;
    }

    let updates = load_all_updates(&data.db).await;
    HttpResponse::Ok().json(UpdatesResponse {
        updates,
        role: "admin".to_string(),
    })
}

#[get("/api/admin/evolution/status")]
pub async fn get_evolution_status(data: web::Data<AppState>, session: Session) -> impl Responder {
    if let Err(response) = require_root_admin(&session) {
        return response;
    }

    let state = load_runtime_state(&data.db).await;
    let updates = load_all_updates(&data.db).await;
    let signals = collect_signals(&data.db).await;

    HttpResponse::Ok().json(EvolutionStatusResponse {
        state,
        pending_updates: updates.iter().filter(|u| u.status == "pending").count(),
        total_updates: updates.len(),
        warn_last_24h: signals.warn_last_24h,
        error_last_24h: signals.error_last_24h,
        brain_nodes_count: signals.brain_nodes_count,
        unanswered_questions_last_24h: signals.unanswered_questions_last_24h,
        top_unknown_topics: signals.top_unknown_topics,
    })
}

#[post("/api/admin/evolution/think")]
pub async fn run_think_cycle(data: web::Data<AppState>, session: Session) -> impl Responder {
    let actor = match require_root_admin(&session) {
        Ok(username) => username,
        Err(response) => return response,
    };

    match run_think_cycle_internal(&data.db, &actor, true).await {
        Ok(outcome) => HttpResponse::Ok().json(ThinkResponse {
            message: if outcome.created_update {
                "Jeebs thought and created a new evolution proposal.".to_string()
            } else {
                "Jeebs thought, but did not create a new proposal this cycle.".to_string()
            },
            id: outcome.update_id,
            created_update: outcome.created_update,
            reason: outcome.reason,
            state: outcome.state,
        }),
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err })),
    }
}

#[post("/api/admin/evolution/apply/{id}")]
pub async fn apply_update(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: Session,
) -> impl Responder {
    let actor = match require_manual_proposal_actor(&session) {
        Ok(username) => username,
        Err(response) => return response,
    };

    let id = path.into_inner();
    let mut update = match load_update_by_id(&data.db, &id).await {
        Some(update) => update,
        None => return HttpResponse::NotFound().json(json!({"error": "Update not found"})),
    };

    if update.status != "pending" {
        return HttpResponse::BadRequest().json(json!({"error": "Update already processed"}));
    }

    if let Err(err) = validate_changes(&update.changes) {
        return HttpResponse::BadRequest().json(json!({ "error": err }));
    }

    let backups = match create_backup_snapshot(&update.changes) {
        Ok(backups) => backups,
        Err(err) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": err
            }))
        }
    };

    if let Err(err) = apply_changes_atomically(&update.changes) {
        create_notification(
            &data.db,
            format!("Failed to apply update '{}': {err}", update.title),
            "High",
            Some("/webui/evolution.html".to_string()),
        )
        .await;
        return HttpResponse::InternalServerError().json(json!({
            "error": err
        }));
    }

    update.backup = Some(backups);
    update.status = "applied".to_string();
    update.applied_at = Some(now_rfc3339());
    if let Err(err) = save_update(&data.db, &update).await {
        return HttpResponse::InternalServerError().json(json!({
            "error": err
        }));
    }

    crate::logging::log(
        &data.db,
        "INFO",
        "EVOLUTION",
        &format!("Applied update '{}' by {}", update.title, actor),
    )
    .await;

    HttpResponse::Ok()
        .json(json!({"message": "Update applied successfully. Please rebuild/restart Jeebs."}))
}

#[post("/api/admin/evolution/deny/{id}")]
pub async fn deny_update(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: Session,
) -> impl Responder {
    let actor = match require_manual_proposal_actor(&session) {
        Ok(username) => username,
        Err(response) => return response,
    };

    let id = path.into_inner();
    let mut update = match load_update_by_id(&data.db, &id).await {
        Some(update) => update,
        None => return HttpResponse::NotFound().json(json!({"error": "Update not found"})),
    };

    update.status = "denied".to_string();
    update.denied_at = Some(now_rfc3339());

    if let Err(err) = save_update(&data.db, &update).await {
        return HttpResponse::InternalServerError().json(json!({"error": err}));
    }

    crate::logging::log(
        &data.db,
        "WARN",
        "EVOLUTION",
        &format!("Denied update '{}' by {}", update.title, actor),
    )
    .await;

    HttpResponse::Ok().json(json!({"message": "Update denied"}))
}

#[post("/api/admin/evolution/resolve/{id}")]
pub async fn resolve_update(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: Session,
) -> impl Responder {
    let actor = match require_manual_proposal_actor(&session) {
        Ok(username) => username,
        Err(response) => return response,
    };

    let id = path.into_inner();
    let mut update = match load_update_by_id(&data.db, &id).await {
        Some(update) => update,
        None => return HttpResponse::NotFound().json(json!({"error": "Update not found"})),
    };

    update.status = "resolved".to_string();
    update.resolved_at = Some(now_rfc3339());
    if let Err(err) = save_update(&data.db, &update).await {
        return HttpResponse::InternalServerError().json(json!({"error": err}));
    }

    crate::logging::log(
        &data.db,
        "INFO",
        "EVOLUTION",
        &format!("Resolved update '{}' by {}", update.title, actor),
    )
    .await;

    HttpResponse::Ok().json(json!({"message": "Update resolved"}))
}

#[post("/api/admin/evolution/rollback/{id}")]
pub async fn rollback_update(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: Session,
) -> impl Responder {
    let actor = match require_manual_proposal_actor(&session) {
        Ok(username) => username,
        Err(response) => return response,
    };

    let id = path.into_inner();
    let mut update = match load_update_by_id(&data.db, &id).await {
        Some(update) => update,
        None => return HttpResponse::NotFound().json(json!({"error": "Update not found"})),
    };

    if update.status != "applied" {
        return HttpResponse::BadRequest().json(json!({"error": "Update is not in applied state"}));
    }

    let backups = match &update.backup {
        Some(backups) => backups.clone(),
        None => {
            return HttpResponse::BadRequest()
                .json(json!({"error": "No backup available for this update"}))
        }
    };

    if let Err(err) = restore_backup_snapshot(&backups) {
        create_notification(
            &data.db,
            format!("Rollback failed for '{}': {err}", update.title),
            "High",
            Some("/webui/evolution.html".to_string()),
        )
        .await;
        return HttpResponse::InternalServerError().json(json!({"error": err}));
    }

    update.status = "rolled_back".to_string();
    update.rolled_back_at = Some(now_rfc3339());
    if let Err(err) = save_update(&data.db, &update).await {
        return HttpResponse::InternalServerError().json(json!({"error": err}));
    }

    crate::logging::log(
        &data.db,
        "WARN",
        "EVOLUTION",
        &format!("Rolled back update '{}' by {}", update.title, actor),
    )
    .await;

    HttpResponse::Ok().json(json!({"message": "Update rolled back successfully"}))
}

#[derive(Deserialize)]
pub struct CreateComment {
    pub content: String,
}

#[post("/api/admin/evolution/comment/{id}")]
pub async fn add_comment(
    data: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<CreateComment>,
    session: Session,
) -> impl Responder {
    let actor = match require_root_admin(&session) {
        Ok(username) => username,
        Err(response) => return response,
    };

    let comment_content = body.content.trim();
    if comment_content.is_empty() {
        return HttpResponse::BadRequest().json(json!({"error": "Comment cannot be empty"}));
    }

    let id = path.into_inner();
    let mut update = match load_update_by_id(&data.db, &id).await {
        Some(update) => update,
        None => return HttpResponse::NotFound().json(json!({"error": "Update not found"})),
    };

    let comment = Comment {
        author: actor.clone(),
        content: comment_content.to_string(),
        timestamp: now_rfc3339(),
    };
    update.comments.push(comment);
    if let Err(err) = save_update(&data.db, &update).await {
        return HttpResponse::InternalServerError().json(json!({"error": err}));
    }

    crate::logging::log(
        &data.db,
        "INFO",
        "EVOLUTION",
        &format!("Added comment to update '{}' by {}", update.title, actor),
    )
    .await;

    HttpResponse::Ok().json(json!({"message": "Comment added"}))
}

#[get("/api/admin/notifications")]
pub async fn get_notifications(data: web::Data<AppState>, session: Session) -> impl Responder {
    if let Err(response) = require_root_admin(&session) {
        return response;
    }

    let rows = sqlx::query("SELECT value FROM jeebs_store WHERE key LIKE ?")
        .bind(format!("{NOTIFICATION_KEY_PREFIX}%"))
        .fetch_all(&data.db)
        .await
        .unwrap_or_default();

    let mut notifications = Vec::new();
    for row in rows {
        let val: Vec<u8> = row.get(0);
        if let Some(notif) = decode_json::<Notification>(&val) {
            notifications.push(notif);
        }
    }
    notifications.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    HttpResponse::Ok().json(notifications)
}

#[delete("/api/admin/notification/{id}")]
pub async fn dismiss_notification(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: Session,
) -> impl Responder {
    let actor = match require_root_admin(&session) {
        Ok(username) => username,
        Err(response) => return response,
    };

    let id = path.into_inner();
    let key = notification_key(&id);
    if let Err(err) = sqlx::query("DELETE FROM jeebs_store WHERE key = ?")
        .bind(&key)
        .execute(&data.db)
        .await
    {
        return HttpResponse::InternalServerError()
            .json(json!({ "error": format!("Failed to dismiss notification: {err}") }));
    }

    crate::logging::log(
        &data.db,
        "INFO",
        "EVOLUTION",
        &format!("Dismissed notification '{id}' by {actor}"),
    )
    .await;

    HttpResponse::Ok().json(json!({"ok": true}))
}

// Backwards-compatible endpoint used by existing UI button.
#[post("/api/evolution/brainstorm")]
pub async fn brainstorm_update(data: web::Data<AppState>, session: Session) -> impl Responder {
    let actor = match require_root_admin(&session) {
        Ok(username) => username,
        Err(response) => return response,
    };

    match run_think_cycle_internal(&data.db, &actor, true).await {
        Ok(outcome) => HttpResponse::Ok().json(json!({
            "message": if outcome.created_update {
                "Jeebs has proposed a new update."
            } else {
                "Jeebs thought, but no new proposal was needed."
            },
            "id": outcome.update_id,
            "reason": outcome.reason,
            "created_update": outcome.created_update
        })),
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_validation_rejects_traversal() {
        assert!(validate_change_path("../src/main.rs").is_err());
        assert!(validate_change_path("/etc/passwd").is_err());
        assert!(validate_change_path("src/ok.rs").is_ok());
    }

    #[test]
    fn change_validation_rejects_duplicate_paths() {
        let changes = vec![
            FileChange {
                path: "src/a.rs".to_string(),
                new_content: "a".to_string(),
                existed_before: true,
            },
            FileChange {
                path: "src/a.rs".to_string(),
                new_content: "b".to_string(),
                existed_before: true,
            },
        ];
        assert!(validate_changes(&changes).is_err());
    }

    #[test]
    fn generated_update_has_expected_shape() {
        let signals = SignalSnapshot {
            warn_last_24h: 1,
            error_last_24h: 0,
            chat_logs_last_24h: 5,
            learned_fact_count: 2,
            recent_fact_samples: vec!["my favorite color is blue".to_string()],
            brain_nodes_count: 3,
            unanswered_questions_last_24h: 2,
            top_unknown_topics: vec!["favorite foods (2)".to_string()],
            pending_updates: 0,
        };
        let update = build_update_from_signals(&signals);
        assert_eq!(update.status, "pending");
        assert!(update.auto_generated);
        assert!(!update.changes.is_empty());
        assert!(!update.fingerprint.is_empty());
    }

    #[test]
    fn fingerprint_is_stable_across_timestamp_noise() {
        let base = ProposedUpdate {
            id: "a".to_string(),
            title: "Autonomous Reflection: Heartbeat and self-check".to_string(),
            author: "Jeebs".to_string(),
            severity: "Low".to_string(),
            comments: Vec::new(),
            description: "Routine autonomous thinking cycle".to_string(),
            changes: vec![FileChange {
                path: "evolution/reflections/20260101-010101-heartbeat.md".to_string(),
                new_content: "# x\nGenerated at: 2026-01-01T01:01:01Z\nReason: test".to_string(),
                existed_before: false,
            }],
            status: "pending".to_string(),
            created_at: "2026-01-01T01:01:01Z".to_string(),
            backup: None,
            auto_generated: true,
            rationale: vec!["No urgent risk detected".to_string()],
            source_signals: vec!["Periodic heartbeat cycle".to_string()],
            confidence: 0.6,
            fingerprint: String::new(),
            requires_restart: false,
            applied_at: None,
            denied_at: None,
            resolved_at: None,
            rolled_back_at: None,
        };

        let mut variant = base.clone();
        variant.changes[0].path = "evolution/reflections/20260102-020202-heartbeat.md".to_string();
        variant.changes[0].new_content =
            "# x\nGenerated at: 2026-01-02T02:02:02Z\nReason: test".to_string();

        assert_eq!(update_fingerprint(&base), update_fingerprint(&variant));
    }
}

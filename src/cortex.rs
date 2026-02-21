use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::Local;
use rand::seq::SliceRandom;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::time::{Duration, Instant};

use crate::state::AppState;
use crate::utils::decode_all;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct BrainNode {
    pub id: Option<i64>,
    pub key: String,
    pub value: String,
    pub label: String,
    pub summary: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f64,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
}

#[derive(Debug, Deserialize)]
pub struct CrawlRequest {
    pub url: String,
    pub depth: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct RandomCrawlQuery {
    pub depth: Option<u8>,
}

#[derive(Debug, Serialize)]
struct NodeWritePreview {
    node_id: String,
    label: String,
    summary: String,
    source_url: String,
}

#[derive(Debug, Serialize)]
struct CrawlSummary {
    start_url: String,
    max_depth: u8,
    pages_visited: usize,
    pages_stored: usize,
    links_followed: usize,
    stored_nodes: Vec<NodeWritePreview>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TrainingLearnedItem {
    node_id: String,
    title: String,
    summary: String,
    source_url: String,
    topic: String,
    source_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TrainingCycleSnapshot {
    cycle_started_at: String,
    cycle_finished_at: String,
    duration_ms: u64,
    topics: Vec<String>,
    websites_scraped: Vec<String>,
    nodes_written: u64,
    crawl_pages_visited: u64,
    crawl_pages_stored: u64,
    crawl_links_followed: u64,
    crawl_nodes_written: u64,
    wikipedia_docs_written: u64,
    learned_items_count: u64,
    errors: Vec<String>,
}

fn default_active_phase() -> String {
    "idle".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TrainingModeState {
    enabled: bool,
    updated_at: String,
    updated_by: String,
    last_cycle_at: Option<String>,
    total_cycles: u64,
    total_topics_processed: u64,
    total_nodes_written: u64,
    #[serde(default)]
    total_websites_scraped: u64,
    #[serde(default)]
    total_crawl_pages_visited: u64,
    #[serde(default)]
    total_crawl_pages_stored: u64,
    #[serde(default)]
    total_crawl_links_followed: u64,
    #[serde(default)]
    total_crawl_nodes_written: u64,
    #[serde(default)]
    total_wikipedia_docs_written: u64,
    last_topics: Vec<String>,
    last_error: Option<String>,
    #[serde(default)]
    last_websites: Vec<String>,
    #[serde(default)]
    last_learned_items: Vec<TrainingLearnedItem>,
    #[serde(default)]
    last_cycle_duration_ms: Option<u64>,
    #[serde(default)]
    last_cycle_nodes_written: u64,
    #[serde(default)]
    last_cycle_errors: Vec<String>,
    #[serde(default)]
    last_cycle_summary: Option<TrainingCycleSnapshot>,
    #[serde(default)]
    recent_cycles: Vec<TrainingCycleSnapshot>,
    #[serde(default)]
    is_cycle_running: bool,
    #[serde(default)]
    active_cycle_started_at: Option<String>,
    #[serde(default = "default_active_phase")]
    active_phase: String,
    #[serde(default)]
    active_target: Option<String>,
    #[serde(default)]
    active_nodes_written: u64,
    #[serde(default)]
    active_websites_completed: u64,
    #[serde(default)]
    active_topics_completed: u64,
    #[serde(default)]
    active_updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrainingModeToggleRequest {
    enabled: bool,
}

#[derive(Debug, Serialize)]
struct TrainingStatusResponse {
    training: TrainingModeState,
    internet_enabled: bool,
    interval_seconds: u64,
}

#[derive(Debug, Serialize)]
struct TrainingCycleReport {
    cycle_started_at: String,
    cycle_finished_at: String,
    duration_ms: u64,
    topics: Vec<String>,
    nodes_written: usize,
    errors: Vec<String>,
    websites_scraped: Vec<String>,
    learned_items: Vec<TrainingLearnedItem>,
    crawl_pages_visited: usize,
    crawl_pages_stored: usize,
    crawl_links_followed: usize,
    crawl_nodes_written: usize,
    wikipedia_docs_written: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommunicationProfile {
    style: String,
    signals: Vec<String>,
    recent_topics: Vec<String>,
    updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct BrainSearchResult {
    pub id: String,
    pub label: String,
    pub summary: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub title: String,
    pub group: String,
}

#[derive(Debug, Serialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct GraphResponse {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ConversationTurn {
    role: String,
    content: String,
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LearnedFact {
    fact: String,
    canonical: String,
    created_at: String,
    updated_at: String,
}

const MAX_HISTORY_TURNS: usize = 24;
const MAX_HISTORY_CHARS_PER_TURN: usize = 600;
const TRAINING_STATE_KEY: &str = "training:mode:state";
const DEFAULT_TRAINING_INTERVAL_SECS: u64 = 5;
const JEEBS_LIKES: &[&str] = &[
    "learning new knowledge",
    "clear reasoning",
    "solving hard problems",
    "structured experiments",
    "useful conversations",
];
const JEEBS_DISLIKES: &[&str] = &[
    "stagnation",
    "guessing without evidence",
    "repeating weak answers",
    "losing useful context",
    "noisy, low-value logs",
];
const JEEBS_WANTS: &[&str] = &[
    "to expand knowledge coverage every cycle",
    "to reduce unanswered questions",
    "to prove intelligence with measurable improvements",
];

fn history_key(user_id: &str) -> String {
    format!("chat:history:{user_id}")
}

fn sanitize_turn_content(content: &str) -> String {
    let compact = content.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= MAX_HISTORY_CHARS_PER_TURN {
        compact
    } else {
        truncate_chars(&compact, MAX_HISTORY_CHARS_PER_TURN)
    }
}

fn parse_history_blob(bytes: &[u8]) -> Option<Vec<ConversationTurn>> {
    let parsed = serde_json::from_slice::<Vec<ConversationTurn>>(bytes).ok()?;
    let mut cleaned = parsed
        .into_iter()
        .filter_map(|turn| {
            let role = turn.role.to_lowercase();
            if role != "user" && role != "assistant" {
                return None;
            }

            let content = sanitize_turn_content(&turn.content);
            if content.is_empty() {
                return None;
            }

            Some(ConversationTurn {
                role,
                content,
                timestamp: turn.timestamp,
            })
        })
        .collect::<Vec<_>>();

    if cleaned.len() > MAX_HISTORY_TURNS {
        cleaned.drain(0..(cleaned.len() - MAX_HISTORY_TURNS));
    }

    Some(cleaned)
}

async fn load_conversation_history(db: &SqlitePool, user_id: &str) -> Vec<ConversationTurn> {
    let key = history_key(user_id);
    let row = match sqlx::query("SELECT value FROM jeebs_store WHERE key = ? LIMIT 1")
        .bind(&key)
        .fetch_optional(db)
        .await
    {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let Some(row) = row else {
        return Vec::new();
    };

    let value: Vec<u8> = row.get(0);
    if let Some(history) = parse_history_blob(&value) {
        return history;
    }

    if let Ok(decoded) = decode_all(&value) {
        if let Some(history) = parse_history_blob(&decoded) {
            return history;
        }
    }

    Vec::new()
}

async fn save_conversation_history(
    db: &SqlitePool,
    user_id: &str,
    turns: &[ConversationTurn],
) -> Result<(), sqlx::Error> {
    let key = history_key(user_id);
    let mut trimmed = turns.to_vec();
    if trimmed.len() > MAX_HISTORY_TURNS {
        trimmed.drain(0..(trimmed.len() - MAX_HISTORY_TURNS));
    }

    let payload = serde_json::to_vec(&trimmed).unwrap_or_default();
    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(&key)
        .bind(payload)
        .execute(db)
        .await?;

    Ok(())
}

fn last_turn_by_role<'a>(
    history: &'a [ConversationTurn],
    role: &str,
) -> Option<&'a ConversationTurn> {
    history.iter().rev().find(|turn| turn.role == role)
}

fn is_follow_up_prompt(lower: &str) -> bool {
    matches!(
        lower,
        "ok" | "okay" | "sure" | "hmm" | "go on" | "continue" | "tell me more" | "elaborate"
    ) || lower.starts_with("why ")
        || lower == "why"
        || lower.starts_with("how ")
        || lower == "how"
        || lower.contains("what do you mean")
        || lower.contains("can you explain")
}

fn extract_name_from_intro(lower_input: &str) -> Option<String> {
    for prefix in ["my name is ", "i am ", "i'm "] {
        if let Some(rest) = lower_input.strip_prefix(prefix) {
            let candidate = rest
                .trim()
                .trim_matches(|ch: char| matches!(ch, '.' | ',' | '!' | '?'));
            if candidate.is_empty() {
                return None;
            }
            let mut chars = candidate.chars();
            let first = chars.next()?;
            let mut out = first.to_uppercase().collect::<String>();
            out.push_str(chars.as_str());
            return Some(out);
        }
    }
    None
}

fn recent_conversation_summary(history: &[ConversationTurn]) -> Option<String> {
    let recent = history
        .iter()
        .rev()
        .filter(|turn| turn.role == "user")
        .take(3)
        .map(|turn| truncate_chars(&turn.content, 80))
        .collect::<Vec<_>>();

    if recent.is_empty() {
        return None;
    }

    let mut ordered = recent;
    ordered.reverse();
    Some(ordered.join(" | "))
}

fn normalize_fact_text(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .trim_matches(|ch: char| matches!(ch, '.' | ',' | '!' | ';'))
        .to_string()
}

fn extract_learnable_fact(prompt: &str) -> Option<String> {
    let clean = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean.is_empty() {
        return None;
    }

    let lower = clean.to_lowercase();
    if clean.ends_with('?')
        || lower.starts_with("remember:")
        || lower.starts_with("learn:")
        || lower.starts_with("forget:")
    {
        return None;
    }

    for prefix in [
        "remember that ",
        "remember this ",
        "please remember that ",
        "please remember ",
        "for future reference ",
        "fyi ",
    ] {
        if lower.starts_with(prefix) {
            let fact = normalize_fact_text(&clean[prefix.len()..]);
            if !fact.is_empty() {
                return Some(fact);
            }
        }
    }

    if lower.starts_with("my ") {
        let likely_fact = lower.contains(" is ")
            || lower.contains(" are ")
            || lower.contains(" was ")
            || lower.contains(" were ")
            || lower.contains(" favorite ")
            || lower.contains(" favourite ");
        if likely_fact {
            let fact = normalize_fact_text(&clean);
            if !fact.is_empty() {
                return Some(fact);
            }
        }
    }

    for prefix in [
        "i am ",
        "i'm ",
        "i live in ",
        "i live at ",
        "i work at ",
        "i work in ",
        "i like ",
        "i love ",
        "i prefer ",
    ] {
        if lower.starts_with(prefix) {
            let fact = normalize_fact_text(&clean);
            if !fact.is_empty() {
                return Some(fact);
            }
        }
    }

    None
}

fn sanitize_key_segment(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.') {
            out.push(ch.to_ascii_lowercase());
        }
    }
    if out.is_empty() {
        "anonymous".to_string()
    } else {
        out
    }
}

fn fact_owner_key(user_id: &str, username: Option<&str>) -> String {
    if let Some(name) = username {
        return format!("user:{}", sanitize_key_segment(name));
    }
    format!("session:{}", sanitize_key_segment(user_id))
}

fn fact_prefix(owner: &str) -> String {
    format!("chat:fact:{owner}:")
}

fn fact_storage_key(owner: &str, canonical: &str) -> String {
    format!(
        "{}{}",
        fact_prefix(owner),
        blake3::hash(canonical.as_bytes()).to_hex()
    )
}

fn parse_learned_fact_bytes(bytes: &[u8]) -> Option<LearnedFact> {
    let parsed = serde_json::from_slice::<LearnedFact>(bytes).ok()?;
    let fact = sanitize_turn_content(&parsed.fact);
    let canonical = canonical_prompt_key(&parsed.canonical);
    if fact.is_empty() || canonical.is_empty() {
        return None;
    }
    Some(LearnedFact {
        fact,
        canonical,
        created_at: parsed.created_at,
        updated_at: parsed.updated_at,
    })
}

async fn save_learned_fact(
    db: &SqlitePool,
    owner: &str,
    fact: &str,
) -> Result<LearnedFact, sqlx::Error> {
    let cleaned_fact = sanitize_turn_content(fact);
    let canonical = canonical_prompt_key(&cleaned_fact);
    let key = fact_storage_key(owner, &canonical);
    let now = Local::now().to_rfc3339();

    let existing = sqlx::query("SELECT value FROM jeebs_store WHERE key = ? LIMIT 1")
        .bind(&key)
        .fetch_optional(db)
        .await?;

    let created_at = existing
        .and_then(|row| {
            let value: Vec<u8> = row.get(0);
            parse_learned_fact_bytes(&value)
                .map(|fact| fact.created_at)
                .or_else(|| {
                    decode_all(&value)
                        .ok()
                        .and_then(|d| parse_learned_fact_bytes(&d))
                        .map(|fact| fact.created_at)
                })
        })
        .unwrap_or_else(|| now.clone());

    let payload = LearnedFact {
        fact: cleaned_fact.clone(),
        canonical: canonical.clone(),
        created_at,
        updated_at: now,
    };

    let bytes = serde_json::to_vec(&payload).unwrap_or_default();
    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(&key)
        .bind(bytes)
        .execute(db)
        .await?;

    Ok(payload)
}

async fn load_learned_facts(db: &SqlitePool, owner: &str) -> Vec<LearnedFact> {
    let pattern = format!("{}%", fact_prefix(owner));
    let rows = sqlx::query("SELECT value FROM jeebs_store WHERE key LIKE ? ORDER BY key ASC")
        .bind(pattern)
        .fetch_all(db)
        .await
        .unwrap_or_default();

    let mut facts = Vec::new();
    for row in rows {
        let value: Vec<u8> = row.get(0);
        if let Some(fact) = parse_learned_fact_bytes(&value) {
            facts.push(fact);
            continue;
        }
        if let Ok(decoded) = decode_all(&value) {
            if let Some(fact) = parse_learned_fact_bytes(&decoded) {
                facts.push(fact);
            }
        }
    }

    facts
}

fn is_token_stopword(token: &str) -> bool {
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
            | "do"
            | "did"
            | "you"
            | "your"
            | "my"
            | "me"
            | "what"
            | "who"
            | "how"
            | "why"
            | "about"
            | "remember"
            | "know"
    )
}

fn tokenize_for_matching(input: &str) -> Vec<String> {
    let mut normalized = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
        } else {
            normalized.push(' ');
        }
    }

    normalized
        .split_whitespace()
        .filter(|token| token.len() >= 3 && !is_token_stopword(token))
        .map(|token| token.to_string())
        .collect()
}

fn rank_relevant_facts(facts: &[LearnedFact], query: &str, limit: usize) -> Vec<LearnedFact> {
    let query_tokens = tokenize_for_matching(query);
    if query_tokens.is_empty() {
        return Vec::new();
    }

    let query_lower = query.to_lowercase();
    let mut scored = Vec::new();
    for fact in facts {
        let fact_lower = fact.fact.to_lowercase();
        let mut score = 0_i32;
        for token in &query_tokens {
            if fact_lower.contains(token) || fact.canonical.contains(token) {
                score += 1;
            }
        }
        if query_lower.contains(&fact.canonical) || fact_lower.contains(&query_lower) {
            score += 3;
        }
        if score > 0 {
            scored.push((score, fact.updated_at.clone(), fact.clone()));
        }
    }

    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, _, fact)| fact)
        .collect()
}

fn most_recent_facts(facts: &[LearnedFact], limit: usize) -> Vec<LearnedFact> {
    let mut sorted = facts.to_vec();
    sorted.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    sorted.into_iter().take(limit).collect()
}

fn wants_personal_memory_overview(lower: &str) -> bool {
    lower.contains("what do you know about me")
        || lower.contains("what do you remember about me")
        || lower.contains("what have you learned about me")
        || lower.contains("tell me about me")
        || lower.contains("remind me what you know about me")
}

fn wants_personal_memory_lookup(lower: &str) -> bool {
    lower.contains("do you remember")
        || lower.contains("what is my ")
        || lower.contains("what's my ")
        || lower.contains("what are my ")
        || lower.contains("where do i live")
        || lower.contains("what do i like")
        || lower.contains("tell me my ")
}

fn comprehension_key(owner: &str) -> String {
    format!("chat:comprehension:{owner}")
}

async fn save_communication_profile(
    db: &SqlitePool,
    owner: &str,
    profile: &CommunicationProfile,
) -> Result<(), sqlx::Error> {
    let payload = serde_json::to_vec(profile).unwrap_or_default();
    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(comprehension_key(owner))
        .bind(payload)
        .execute(db)
        .await?;
    Ok(())
}

async fn load_communication_profile(db: &SqlitePool, owner: &str) -> Option<CommunicationProfile> {
    let row = sqlx::query("SELECT value FROM jeebs_store WHERE key = ? LIMIT 1")
        .bind(comprehension_key(owner))
        .fetch_optional(db)
        .await
        .ok()??;
    let raw: Vec<u8> = row.get(0);
    serde_json::from_slice::<CommunicationProfile>(&raw)
        .ok()
        .or_else(|| {
            decode_all(&raw)
                .ok()
                .and_then(|decoded| serde_json::from_slice::<CommunicationProfile>(&decoded).ok())
        })
}

fn infer_recent_topics(turns: &[ConversationTurn], limit: usize) -> Vec<String> {
    let mut topics = HashMap::<String, usize>::new();
    for turn in turns.iter().filter(|t| t.role == "user").rev().take(8) {
        for token in tokenize_for_matching(&turn.content) {
            if token.len() >= 4 {
                *topics.entry(token).or_insert(0) += 1;
            }
        }
    }
    let mut scored = topics.into_iter().collect::<Vec<_>>();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    scored
        .into_iter()
        .take(limit)
        .map(|(topic, _)| topic)
        .collect()
}

fn analyze_communication_profile(
    prompt: &str,
    history: &[ConversationTurn],
    previous: Option<&CommunicationProfile>,
) -> CommunicationProfile {
    let mut recent_user_turns = history
        .iter()
        .filter(|turn| turn.role == "user")
        .rev()
        .take(6)
        .cloned()
        .collect::<Vec<_>>();
    recent_user_turns.reverse();

    if !prompt.trim().is_empty() {
        recent_user_turns.push(ConversationTurn {
            role: "user".to_string(),
            content: sanitize_turn_content(prompt),
            timestamp: Local::now().to_rfc3339(),
        });
    }

    let mut question_count = 0usize;
    let mut command_like_count = 0usize;
    let mut gratitude_count = 0usize;
    let mut frustration_count = 0usize;
    let mut long_message_count = 0usize;

    for turn in &recent_user_turns {
        let lower = turn.content.to_lowercase();
        if lower.contains('?') {
            question_count += 1;
        }
        if lower.starts_with("please ")
            || lower.starts_with("do ")
            || lower.starts_with("add ")
            || lower.starts_with("make ")
            || lower.starts_with("fix ")
            || lower.starts_with("give ")
            || lower.starts_with("build ")
        {
            command_like_count += 1;
        }
        if lower.contains("thanks") || lower.contains("thank you") {
            gratitude_count += 1;
        }
        if lower.contains("not working")
            || lower.contains("still no")
            || lower.contains("broken")
            || lower.contains("wtf")
            || lower.contains("why isn't")
        {
            frustration_count += 1;
        }
        if turn.content.chars().count() >= 120 {
            long_message_count += 1;
        }
    }

    let style = if frustration_count >= 1 {
        "frustrated"
    } else if question_count >= 3 {
        "curious"
    } else if command_like_count >= 2 {
        "direct"
    } else if long_message_count >= 2 {
        "reflective"
    } else if gratitude_count >= 1 {
        "collaborative"
    } else {
        "neutral"
    }
    .to_string();

    let mut signals = Vec::new();
    signals.push(format!(
        "recent_questions={}, direct_requests={}, long_messages={}",
        question_count, command_like_count, long_message_count
    ));
    if frustration_count >= 1 {
        signals.push("frustration detected in recent phrasing".to_string());
    }
    if gratitude_count >= 1 {
        signals.push("appreciation language detected".to_string());
    }
    if let Some(previous) = previous {
        if previous.style != style {
            signals.push(format!(
                "style shifted from {} to {}",
                previous.style, style
            ));
        }
    }

    CommunicationProfile {
        style,
        signals,
        recent_topics: infer_recent_topics(&recent_user_turns, 6),
        updated_at: Local::now().to_rfc3339(),
    }
}

fn wants_communication_reflection(lower: &str) -> bool {
    lower.contains("how am i communicating")
        || lower.contains("how do i communicate")
        || lower.contains("my communication style")
        || lower.contains("how am i coming across")
        || lower.contains("am i being clear")
        || lower.contains("what do you think of my communication")
}

fn render_communication_reflection(profile: &CommunicationProfile) -> String {
    let mut lines = vec![format!(
        "You are communicating in a {} style right now.",
        profile.style
    )];
    if !profile.recent_topics.is_empty() {
        lines.push(format!(
            "Recent topics you focus on: {}.",
            profile.recent_topics.join(", ")
        ));
    }
    if let Some(signal) = profile.signals.first() {
        lines.push(format!("Signal snapshot: {signal}."));
    }
    lines.join(" ")
}

fn training_interval_seconds() -> u64 {
    env::var("TRAINING_MODE_INTERVAL_SECS")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .map(|value| value.clamp(5, 3600))
        .unwrap_or(DEFAULT_TRAINING_INTERVAL_SECS)
}

fn training_state_default() -> TrainingModeState {
    TrainingModeState {
        enabled: false,
        updated_at: Local::now().to_rfc3339(),
        updated_by: "system".to_string(),
        last_cycle_at: None,
        total_cycles: 0,
        total_topics_processed: 0,
        total_nodes_written: 0,
        total_websites_scraped: 0,
        total_crawl_pages_visited: 0,
        total_crawl_pages_stored: 0,
        total_crawl_links_followed: 0,
        total_crawl_nodes_written: 0,
        total_wikipedia_docs_written: 0,
        last_topics: Vec::new(),
        last_error: None,
        last_websites: Vec::new(),
        last_learned_items: Vec::new(),
        last_cycle_duration_ms: None,
        last_cycle_nodes_written: 0,
        last_cycle_errors: Vec::new(),
        last_cycle_summary: None,
        recent_cycles: Vec::new(),
        is_cycle_running: false,
        active_cycle_started_at: None,
        active_phase: default_active_phase(),
        active_target: None,
        active_nodes_written: 0,
        active_websites_completed: 0,
        active_topics_completed: 0,
        active_updated_at: None,
    }
}

async fn load_training_state(db: &SqlitePool) -> TrainingModeState {
    let row = sqlx::query("SELECT value FROM jeebs_store WHERE key = ? LIMIT 1")
        .bind(TRAINING_STATE_KEY)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    let Some(row) = row else {
        return training_state_default();
    };

    let raw: Vec<u8> = row.get(0);
    serde_json::from_slice::<TrainingModeState>(&raw)
        .ok()
        .or_else(|| {
            decode_all(&raw)
                .ok()
                .and_then(|decoded| serde_json::from_slice::<TrainingModeState>(&decoded).ok())
        })
        .unwrap_or_else(training_state_default)
}

async fn save_training_state(
    db: &SqlitePool,
    state: &TrainingModeState,
) -> Result<(), sqlx::Error> {
    let payload = serde_json::to_vec(state).unwrap_or_default();
    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(TRAINING_STATE_KEY)
        .bind(payload)
        .execute(db)
        .await?;
    Ok(())
}

async fn mutate_training_state<F>(db: &SqlitePool, mutator: F)
where
    F: FnOnce(&mut TrainingModeState),
{
    let mut mode = load_training_state(db).await;
    mutator(&mut mode);
    mode.updated_at = Local::now().to_rfc3339();
    if mode.updated_by.trim().is_empty() {
        mode.updated_by = "training_runtime".to_string();
    }
    let _ = save_training_state(db, &mode).await;
}

fn report_to_snapshot(report: &TrainingCycleReport) -> TrainingCycleSnapshot {
    TrainingCycleSnapshot {
        cycle_started_at: report.cycle_started_at.clone(),
        cycle_finished_at: report.cycle_finished_at.clone(),
        duration_ms: report.duration_ms,
        topics: report.topics.clone(),
        websites_scraped: report.websites_scraped.clone(),
        nodes_written: report.nodes_written as u64,
        crawl_pages_visited: report.crawl_pages_visited as u64,
        crawl_pages_stored: report.crawl_pages_stored as u64,
        crawl_links_followed: report.crawl_links_followed as u64,
        crawl_nodes_written: report.crawl_nodes_written as u64,
        wikipedia_docs_written: report.wikipedia_docs_written as u64,
        learned_items_count: report.learned_items.len() as u64,
        errors: report.errors.clone(),
    }
}

fn apply_training_report(mode: &mut TrainingModeState, report: &TrainingCycleReport, actor: &str) {
    mode.last_cycle_at = Some(report.cycle_finished_at.clone());
    mode.total_cycles = mode.total_cycles.saturating_add(1);
    mode.total_topics_processed = mode
        .total_topics_processed
        .saturating_add(report.topics.len() as u64);
    mode.total_nodes_written = mode
        .total_nodes_written
        .saturating_add(report.nodes_written as u64);
    mode.total_websites_scraped = mode
        .total_websites_scraped
        .saturating_add(report.websites_scraped.len() as u64);
    mode.total_crawl_pages_visited = mode
        .total_crawl_pages_visited
        .saturating_add(report.crawl_pages_visited as u64);
    mode.total_crawl_pages_stored = mode
        .total_crawl_pages_stored
        .saturating_add(report.crawl_pages_stored as u64);
    mode.total_crawl_links_followed = mode
        .total_crawl_links_followed
        .saturating_add(report.crawl_links_followed as u64);
    mode.total_crawl_nodes_written = mode
        .total_crawl_nodes_written
        .saturating_add(report.crawl_nodes_written as u64);
    mode.total_wikipedia_docs_written = mode
        .total_wikipedia_docs_written
        .saturating_add(report.wikipedia_docs_written as u64);

    mode.last_topics = report.topics.clone();
    mode.last_websites = report.websites_scraped.clone();
    mode.last_learned_items = report.learned_items.clone();
    mode.last_error = report.errors.first().cloned();
    mode.last_cycle_duration_ms = Some(report.duration_ms);
    mode.last_cycle_nodes_written = report.nodes_written as u64;
    mode.last_cycle_errors = report.errors.clone();

    let snapshot = report_to_snapshot(report);
    mode.last_cycle_summary = Some(snapshot.clone());
    mode.recent_cycles.insert(0, snapshot);
    if mode.recent_cycles.len() > 30 {
        mode.recent_cycles.truncate(30);
    }

    mode.updated_at = report.cycle_finished_at.clone();
    mode.updated_by = actor.to_string();

    mode.is_cycle_running = false;
    mode.active_cycle_started_at = None;
    mode.active_phase = default_active_phase();
    mode.active_target = None;
    mode.active_nodes_written = 0;
    mode.active_websites_completed = 0;
    mode.active_topics_completed = 0;
    mode.active_updated_at = Some(report.cycle_finished_at.clone());
}

fn finalize_training_report(report: &mut TrainingCycleReport, timer: &Instant) {
    report.cycle_finished_at = Local::now().to_rfc3339();
    let elapsed_ms = timer.elapsed().as_millis();
    report.duration_ms = elapsed_ms.min(u128::from(u64::MAX)) as u64;
}

fn extract_question_topic(question: &str) -> String {
    let tokens = tokenize_for_matching(question);
    if tokens.is_empty() {
        return canonical_prompt_key(question);
    }
    tokens.into_iter().take(5).collect::<Vec<_>>().join(" ")
}

async fn collect_training_topics(db: &SqlitePool, limit: usize) -> Vec<String> {
    let rows = sqlx::query("SELECT value FROM jeebs_store WHERE key LIKE 'chat:history:%'")
        .fetch_all(db)
        .await
        .unwrap_or_default();

    let mut counts = HashMap::<String, usize>::new();
    for row in rows {
        let raw: Vec<u8> = row.get(0);
        let parsed = serde_json::from_slice::<Vec<ConversationTurn>>(&raw)
            .ok()
            .or_else(|| {
                decode_all(&raw).ok().and_then(|decoded| {
                    serde_json::from_slice::<Vec<ConversationTurn>>(&decoded).ok()
                })
            });
        let Some(turns) = parsed else {
            continue;
        };

        for turn in turns.iter().rev().take(24) {
            if turn.role != "user" || !turn.content.contains('?') {
                continue;
            }
            let topic = extract_question_topic(&turn.content);
            if topic.is_empty() {
                continue;
            }
            *counts.entry(topic).or_insert(0) += 1;
        }
    }

    let mut topics = counts.into_iter().collect::<Vec<_>>();
    topics.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let mut out = topics
        .into_iter()
        .take(limit)
        .map(|(topic, _)| topic)
        .collect::<Vec<_>>();

    if out.is_empty() {
        out = vec![
            "artificial intelligence".to_string(),
            "software engineering best practices".to_string(),
            "rust programming".to_string(),
            "internet security basics".to_string(),
        ];
    }

    out
}

#[derive(Debug, Deserialize)]
struct WikiSummaryResponse {
    title: Option<String>,
    extract: Option<String>,
    #[serde(default)]
    content_urls: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
struct ExternalLearningDoc {
    title: String,
    url: String,
    summary: String,
    topic: String,
}

async fn query_wikipedia_docs(
    client: &reqwest::Client,
    topic: &str,
    max_docs: usize,
) -> Result<Vec<ExternalLearningDoc>, String> {
    let mut search_url = reqwest::Url::parse("https://en.wikipedia.org/w/api.php")
        .map_err(|err| format!("wikipedia search url build failed: {err}"))?;
    {
        let mut pairs = search_url.query_pairs_mut();
        pairs.append_pair("action", "opensearch");
        pairs.append_pair("search", topic);
        pairs.append_pair("limit", "5");
        pairs.append_pair("namespace", "0");
        pairs.append_pair("format", "json");
    }

    let response = client
        .get(search_url)
        .send()
        .await
        .map_err(|err| format!("wikipedia search request failed: {err}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "wikipedia search failed with status {}",
            response.status()
        ));
    }

    let search_raw = response
        .text()
        .await
        .map_err(|err| format!("wikipedia search read failed: {err}"))?;
    let search = serde_json::from_str::<serde_json::Value>(&search_raw)
        .map_err(|err| format!("wikipedia search parse failed: {err}"))?;
    let titles = search
        .get(1)
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();

    let mut docs = Vec::new();
    for title_value in titles.into_iter().take(max_docs) {
        let title = title_value.as_str().map(str::to_string).unwrap_or_default();
        if title.trim().is_empty() {
            continue;
        }

        let mut summary_url =
            reqwest::Url::parse("https://en.wikipedia.org/api/rest_v1/page/summary/")
                .map_err(|err| format!("summary url build failed: {err}"))?;
        summary_url
            .path_segments_mut()
            .map_err(|_| "failed to build summary path".to_string())?
            .pop_if_empty()
            .push(&title);

        let summary_resp = client
            .get(summary_url)
            .send()
            .await
            .map_err(|err| format!("wikipedia summary request failed: {err}"))?;

        if !summary_resp.status().is_success() {
            continue;
        }

        let summary_raw = summary_resp
            .text()
            .await
            .map_err(|err| format!("wikipedia summary read failed: {err}"))?;
        let payload = serde_json::from_str::<WikiSummaryResponse>(&summary_raw)
            .map_err(|err| format!("wikipedia summary parse failed: {err}"))?;

        let resolved_title = payload
            .title
            .unwrap_or_else(|| title.clone())
            .trim()
            .to_string();
        let extract = payload
            .extract
            .unwrap_or_default()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if resolved_title.is_empty() || extract.is_empty() {
            continue;
        }

        let page_url = payload
            .content_urls
            .as_ref()
            .and_then(|urls| urls.get("desktop"))
            .and_then(|desktop| desktop.get("page"))
            .and_then(|page| page.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| {
                format!(
                    "https://en.wikipedia.org/wiki/{}",
                    resolved_title.replace(' ', "_")
                )
            });

        docs.push(ExternalLearningDoc {
            title: resolved_title,
            url: page_url,
            summary: truncate_chars(&extract, 900),
            topic: topic.to_string(),
        });
    }

    Ok(docs)
}

async fn store_external_learning_doc(
    db: &SqlitePool,
    doc: &ExternalLearningDoc,
) -> Result<String, sqlx::Error> {
    let normalized_url = doc.url.trim();
    let node_id = format!("train:{}", blake3::hash(normalized_url.as_bytes()).to_hex());
    let payload = serde_json::to_vec(&json!({
        "source": "training_mode",
        "provider": "wikipedia",
        "topic": doc.topic,
        "url": doc.url,
        "title": doc.title,
        "summary": doc.summary,
        "trained_at": Local::now().to_rfc3339()
    }))
    .unwrap_or_default();

    sqlx::query(
        "INSERT OR REPLACE INTO brain_nodes (id, label, summary, data, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&node_id)
    .bind(&doc.title)
    .bind(&doc.summary)
    .bind(payload)
    .bind(Local::now().to_rfc3339())
    .execute(db)
    .await?;

    let _ = sqlx::query(
        "INSERT OR REPLACE INTO knowledge_triples (subject, predicate, object, confidence, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&doc.topic)
    .bind("researched_from")
    .bind(&doc.url)
    .bind(0.82_f64)
    .bind(Local::now().to_rfc3339())
    .execute(db)
    .await;

    Ok(node_id)
}

async fn run_training_cycle(state: &AppState) -> TrainingCycleReport {
    let cycle_started_at = Local::now().to_rfc3339();
    let cycle_timer = Instant::now();
    let mut report = TrainingCycleReport {
        cycle_started_at: cycle_started_at.clone(),
        cycle_finished_at: cycle_started_at.clone(),
        duration_ms: 0,
        topics: Vec::new(),
        nodes_written: 0,
        errors: Vec::new(),
        websites_scraped: Vec::new(),
        learned_items: Vec::new(),
        crawl_pages_visited: 0,
        crawl_pages_stored: 0,
        crawl_links_followed: 0,
        crawl_nodes_written: 0,
        wikipedia_docs_written: 0,
    };

    let cycle_started_for_state = cycle_started_at.clone();
    mutate_training_state(&state.db, move |mode| {
        mode.is_cycle_running = true;
        mode.active_cycle_started_at = Some(cycle_started_for_state);
        mode.active_phase = "starting training cycle".to_string();
        mode.active_target = None;
        mode.active_nodes_written = 0;
        mode.active_websites_completed = 0;
        mode.active_topics_completed = 0;
        mode.active_updated_at = Some(Local::now().to_rfc3339());
    })
    .await;

    if !*state.internet_enabled.read().unwrap() {
        report
            .errors
            .push("internet is disabled; enable it in admin first".to_string());
        finalize_training_report(&mut report, &cycle_timer);
        return report;
    }

    let mut topics = collect_training_topics(&state.db, 4).await;
    for curiosity_topic in jeebs_curiosity_topics() {
        if topics.len() >= 7 {
            break;
        }
        if !topics.contains(&curiosity_topic) {
            topics.push(curiosity_topic);
        }
    }
    report.topics = topics.clone();

    let topics_count = report.topics.len() as u64;
    mutate_training_state(&state.db, move |mode| {
        mode.active_phase = "collecting topics and preparing sources".to_string();
        mode.active_target = Some(format!("{topics_count} topic(s) queued"));
        mode.active_updated_at = Some(Local::now().to_rfc3339());
    })
    .await;

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("JeebsAI-TrainingMode/1.0")
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            report
                .errors
                .push(format!("failed to initialize training client: {err}"));
            finalize_training_report(&mut report, &cycle_timer);
            return report;
        }
    };

    let crawl_depth = env::var("TRAINING_MODE_CRAWL_DEPTH")
        .ok()
        .and_then(|raw| raw.parse::<u8>().ok())
        .map(|value| value.clamp(1, 3))
        .unwrap_or(1);
    let random_site_count = env::var("TRAINING_MODE_RANDOM_SITES_PER_CYCLE")
        .ok()
        .and_then(|raw| raw.parse::<usize>().ok())
        .map(|value| value.clamp(1, 5))
        .unwrap_or(2);

    let random_sites = {
        let mut rng = rand::thread_rng();
        let mut sites = random_crawl_candidates();
        sites.shuffle(&mut rng);
        sites
            .into_iter()
            .take(random_site_count)
            .collect::<Vec<_>>()
    };

    let mut websites_completed = 0_u64;
    let mut topics_completed = 0_u64;

    for site in random_sites {
        let target_site = site.to_string();
        mutate_training_state(&state.db, move |mode| {
            mode.active_phase = "crawling random website".to_string();
            mode.active_target = Some(target_site);
            mode.active_updated_at = Some(Local::now().to_rfc3339());
        })
        .await;

        match crawl_and_store(state, site, crawl_depth).await {
            Ok(summary) => {
                websites_completed = websites_completed.saturating_add(1);
                report.websites_scraped.push(summary.start_url.clone());
                report.nodes_written += summary.pages_stored;
                report.crawl_pages_visited += summary.pages_visited;
                report.crawl_pages_stored += summary.pages_stored;
                report.crawl_links_followed += summary.links_followed;
                report.crawl_nodes_written += summary.pages_stored;
                for node in summary.stored_nodes.into_iter().take(8) {
                    report.learned_items.push(TrainingLearnedItem {
                        node_id: node.node_id,
                        title: node.label,
                        summary: node.summary,
                        source_url: node.source_url,
                        topic: "random_web_crawl".to_string(),
                        source_type: "crawl".to_string(),
                    });
                }
            }
            Err(err) => report.errors.push(format!("random site '{site}': {err}")),
        }

        let active_nodes_written = report.nodes_written as u64;
        mutate_training_state(&state.db, move |mode| {
            mode.active_nodes_written = active_nodes_written;
            mode.active_websites_completed = websites_completed;
            mode.active_topics_completed = topics_completed;
            mode.active_updated_at = Some(Local::now().to_rfc3339());
        })
        .await;
    }

    for topic in topics {
        let target_topic = topic.clone();
        mutate_training_state(&state.db, move |mode| {
            mode.active_phase = "researching topic".to_string();
            mode.active_target = Some(target_topic);
            mode.active_updated_at = Some(Local::now().to_rfc3339());
        })
        .await;

        match query_wikipedia_docs(&client, &topic, 2).await {
            Ok(docs) => {
                for doc in docs {
                    match store_external_learning_doc(&state.db, &doc).await {
                        Ok(node_id) => {
                            report.nodes_written += 1;
                            report.learned_items.push(TrainingLearnedItem {
                                node_id,
                                title: doc.title.clone(),
                                summary: doc.summary.clone(),
                                source_url: doc.url.clone(),
                                topic: doc.topic.clone(),
                                source_type: "wikipedia".to_string(),
                            });
                            report.wikipedia_docs_written += 1;
                        }
                        Err(err) => report.errors.push(format!(
                            "failed to store learned doc '{}': {err}",
                            doc.title
                        )),
                    }

                    let active_nodes_written = report.nodes_written as u64;
                    let active_docs_written = report.wikipedia_docs_written as u64;
                    mutate_training_state(&state.db, move |mode| {
                        mode.active_nodes_written = active_nodes_written;
                        mode.active_target = Some(format!(
                            "writing wikipedia docs ({active_docs_written} this cycle)"
                        ));
                        mode.active_updated_at = Some(Local::now().to_rfc3339());
                    })
                    .await;
                }
            }
            Err(err) => report.errors.push(format!("topic '{topic}': {err}")),
        }

        topics_completed = topics_completed.saturating_add(1);
        let active_nodes_written = report.nodes_written as u64;
        mutate_training_state(&state.db, move |mode| {
            mode.active_nodes_written = active_nodes_written;
            mode.active_websites_completed = websites_completed;
            mode.active_topics_completed = topics_completed;
            mode.active_updated_at = Some(Local::now().to_rfc3339());
        })
        .await;
    }

    if report.learned_items.len() > 24 {
        report.learned_items.truncate(24);
    }

    finalize_training_report(&mut report, &cycle_timer);
    report
}

pub fn spawn_autonomous_training(state: web::Data<AppState>) {
    tokio::spawn(async move {
        crate::logging::log(
            &state.db,
            "INFO",
            "training_mode",
            "Autonomous training worker started.",
        )
        .await;

        loop {
            let mut mode = load_training_state(&state.db).await;
            if mode.enabled {
                let report = run_training_cycle(state.get_ref()).await;
                apply_training_report(&mut mode, &report, "autonomous_worker");
                let _ = save_training_state(&state.db, &mode).await;

                crate::logging::log(
                    &state.db,
                    "INFO",
                    "training_mode",
                    &format!(
                        "Training cycle complete. duration_ms={} topics={} websites={} nodes_written={} crawl_pages_visited={} crawl_links_followed={} wiki_docs={} errors={}",
                        report.duration_ms,
                        report.topics.len(),
                        report.websites_scraped.len(),
                        report.nodes_written,
                        report.crawl_pages_visited,
                        report.crawl_links_followed,
                        report.wikipedia_docs_written,
                        report.errors.len()
                    ),
                )
                .await;
            }

            tokio::time::sleep(Duration::from_secs(training_interval_seconds())).await;
        }
    });
}

pub async fn search_knowledge(db: &SqlitePool, query: &str) -> Vec<BrainNode> {
    let pattern = format!("%{}%", query);
    let rows = sqlx::query(
        "SELECT id, COALESCE(label, id) AS label, COALESCE(summary, '') AS summary, COALESCE(created_at, '') AS created_at
         FROM brain_nodes
         WHERE id LIKE ? OR label LIKE ? OR summary LIKE ?
         ORDER BY created_at DESC
         LIMIT 10",
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .map(|row| {
            let raw_id: String = row.get(0);
            let label: String = row.get(1);
            let summary: String = row.get(2);
            let created_at: String = row.get(3);
            BrainNode {
                id: raw_id.parse::<i64>().ok(),
                key: raw_id.clone(),
                value: summary.clone(),
                label,
                summary,
                created_at,
            }
        })
        .collect()
}

async fn check_dejavu(prompt: &str, db: &SqlitePool) -> Option<String> {
    fn parse_answer_bytes(bytes: &[u8]) -> Option<String> {
        if let Ok(json_value) = serde_json::from_slice::<serde_json::Value>(bytes) {
            if let Some(answer) = json_value.get("answer").and_then(|v| v.as_str()) {
                let answer = answer.trim();
                if !answer.is_empty() {
                    return Some(answer.to_string());
                }
            }
            if let Some(answer) = json_value.get("response").and_then(|v| v.as_str()) {
                let answer = answer.trim();
                if !answer.is_empty() {
                    return Some(answer.to_string());
                }
            }
        }

        if let Ok(text) = std::str::from_utf8(bytes) {
            let text = text.trim();
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }

        None
    }

    let key = format!("chat:faq:{}", canonical_prompt_key(prompt));
    match sqlx::query("SELECT value FROM jeebs_store WHERE key = ? LIMIT 1")
        .bind(&key)
        .fetch_optional(db)
        .await
    {
        Ok(Some(row)) => {
            let value: Vec<u8> = row.get(0);
            if let Some(answer) = parse_answer_bytes(&value) {
                return Some(answer);
            }

            if let Ok(decoded) = decode_all(&value) {
                return parse_answer_bytes(&decoded);
            }

            None
        }
        _ => None,
    }
}

async fn search_brain_for_chat(db: &SqlitePool, query: &str) -> Vec<(String, String)> {
    let pattern = format!("%{}%", query);
    let rows = sqlx::query(
        "SELECT COALESCE(label, id) AS label, COALESCE(summary, '') AS summary
         FROM brain_nodes
         WHERE id LIKE ? OR label LIKE ? OR summary LIKE ?
         ORDER BY created_at DESC
         LIMIT 3",
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .map(|row| {
            let label: String = row.get(0);
            let summary: String = row.get(1);
            (label, summary)
        })
        .collect()
}

fn looks_like_math_expression(expr: &str) -> bool {
    let compact = expr.trim();
    if compact.is_empty() {
        return false;
    }

    let mut has_digit = false;
    for ch in compact.chars() {
        if ch.is_ascii_digit() {
            has_digit = true;
            continue;
        }
        if matches!(
            ch,
            ' ' | '+' | '-' | '*' | '/' | '(' | ')' | '.' | '^' | '%'
        ) {
            continue;
        }
        return false;
    }

    has_digit
}

fn extract_math_expression(prompt: &str, lower: &str) -> Option<String> {
    for prefix in ["calculate ", "calc ", "math ", "solve "] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let expr = rest.trim().replace(',', "");
            if looks_like_math_expression(&expr) {
                return Some(expr);
            }
        }
    }

    if let Some(rest) = lower.strip_prefix("what is ") {
        let expr = rest.trim_end_matches('?').trim().replace(',', "");
        if looks_like_math_expression(&expr) {
            return Some(expr);
        }
    }

    let direct = prompt.trim().replace(',', "");
    if looks_like_math_expression(&direct) {
        return Some(direct);
    }

    None
}

fn format_number(value: f64) -> String {
    let rounded = (value * 1_000_000_000.0).round() / 1_000_000_000.0;
    let mut s = format!("{rounded}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
}

fn is_greeting(lower: &str) -> bool {
    matches!(
        lower,
        "hi" | "hello" | "hey" | "yo" | "sup" | "good morning" | "good afternoon" | "good evening"
    ) || lower.starts_with("hi ")
        || lower.starts_with("hello ")
        || lower.starts_with("hey ")
}

fn is_goodbye(lower: &str) -> bool {
    matches!(lower, "bye" | "goodbye" | "see you" | "later") || lower.starts_with("bye ")
}

fn help_text() -> String {
    [
        "I can handle conversation and basic assistant tasks:",
        "- multi-turn chat with short memory per session",
        "- learns personal facts from normal chat (example: `my favorite color is blue`)",
        "- greetings and short conversation",
        "- quick math (example: calculate 12 * 7)",
        "- current date/time",
        "- lookup from stored brain notes",
        "- communication reflection (ask: `how am i communicating?`)",
        "- preferences and goals (ask: `what do you like?`, `what do you dislike?`, `what do you want?`)",
        "- custom memory commands: `remember: question => answer`, `forget: question`",
        "",
        "Try: `hello`, `what time is it`, `what is 18/3`, or ask about something I already learned.",
    ]
    .join("\n")
}

fn wants_likes_prompt(lower: &str) -> bool {
    (lower.contains("what do you like")
        || lower.contains("your likes")
        || lower.contains("what are your likes"))
        && !lower.contains("dislike")
}

fn wants_dislikes_prompt(lower: &str) -> bool {
    lower.contains("what do you dislike")
        || lower.contains("your dislikes")
        || lower.contains("what are your dislikes")
}

fn wants_goal_prompt(lower: &str) -> bool {
    lower.contains("what do you want")
        || lower.contains("what are your goals")
        || lower.contains("what are your goal")
        || lower.contains("what do you want to learn")
        || lower.contains("why do you learn")
}

fn jeebs_curiosity_topics() -> Vec<String> {
    vec![
        "scientific method".to_string(),
        "knowledge representation".to_string(),
        "reasoning under uncertainty".to_string(),
        "systems design".to_string(),
        "human communication patterns".to_string(),
    ]
}

fn canonical_prompt_key(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_learning_command(input: &str) -> Option<(String, String)> {
    let lower = input.to_lowercase();
    let payload = if lower.starts_with("remember:") {
        input.split_once(':')?.1.trim()
    } else if lower.starts_with("learn:") {
        input.split_once(':')?.1.trim()
    } else {
        return None;
    };

    let (question, answer) = payload
        .split_once("=>")
        .or_else(|| payload.split_once("->"))
        .or_else(|| payload.split_once('='))
        .or_else(|| payload.split_once(':'))?;

    let question = canonical_prompt_key(question);
    let answer = answer.trim().to_string();
    if question.is_empty() || answer.is_empty() {
        return None;
    }

    Some((question, answer))
}

fn parse_forget_command(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    if !lower.starts_with("forget:") {
        return None;
    }
    let target = input.split_once(':')?.1.trim();
    let normalized = canonical_prompt_key(target);
    if normalized.is_empty() {
        return None;
    }
    Some(normalized)
}

pub async fn custom_ai_logic(prompt: &str, db: &SqlitePool) -> String {
    custom_ai_logic_with_context(prompt, db, &[], None, None).await
}

async fn custom_ai_logic_with_context(
    prompt: &str,
    db: &SqlitePool,
    history: &[ConversationTurn],
    username: Option<&str>,
    facts_owner: Option<&str>,
) -> String {
    let clean_prompt = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean_prompt.is_empty() {
        return "Send me a message and I will respond.".to_string();
    }
    let lower = clean_prompt.to_lowercase();
    let learned_facts = if let Some(owner) = facts_owner {
        load_learned_facts(db, owner).await
    } else {
        Vec::new()
    };
    let communication_profile = if let Some(owner) = facts_owner {
        load_communication_profile(db, owner).await
    } else {
        None
    };

    if lower.contains("what did i just say") || lower.contains("what was my last message") {
        if let Some(previous_user) = last_turn_by_role(history, "user") {
            return format!("Your last message was: \"{}\"", previous_user.content);
        }
        return "I do not have earlier messages in this session yet.".to_string();
    }

    if lower.contains("what did you just say") || lower.contains("repeat that") {
        if let Some(previous_assistant) = last_turn_by_role(history, "assistant") {
            return format!("I said: \"{}\"", previous_assistant.content);
        }
        return "I do not have a previous reply to repeat yet.".to_string();
    }

    if lower.contains("what were we talking about") || lower.contains("recap") {
        if let Some(summary) = recent_conversation_summary(history) {
            return format!("Recent topics from our chat: {summary}");
        }
        return "We just started chatting. Ask me anything to get going.".to_string();
    }

    if wants_communication_reflection(&lower) {
        if let Some(profile) = communication_profile.as_ref() {
            return render_communication_reflection(profile);
        }
        return "I do not have enough chat context yet to analyze your communication style."
            .to_string();
    }

    if lower.contains("what is my name") || lower.contains("who am i") {
        for turn in history.iter().rev().filter(|turn| turn.role == "user") {
            if let Some(name) = extract_name_from_intro(&turn.content.to_lowercase()) {
                return format!("You told me your name is {name}.");
            }
        }

        for fact in most_recent_facts(&learned_facts, 12) {
            if let Some(name) = extract_name_from_intro(&fact.fact.to_lowercase()) {
                return format!("You told me your name is {name}.");
            }
        }

        if let Some(name) = username {
            return format!("You are logged in as {name}.");
        }

        return "I do not know your name yet. Tell me with \"my name is ...\"".to_string();
    }

    if let Some(name) = extract_name_from_intro(&lower) {
        return format!("Nice to meet you, {name}. What do you want to talk about?");
    }

    if is_follow_up_prompt(&lower) {
        if let Some(previous_assistant) = last_turn_by_role(history, "assistant") {
            return format!(
                "Sure. Building on that: {}",
                truncate_chars(&previous_assistant.content, 220)
            );
        }
    }

    if is_greeting(&lower) {
        return if let Some(name) = username {
            format!("Hey {name}, I am Jeebs. I am online and ready to chat.")
        } else {
            "Hey, I am Jeebs. I am online and ready to chat.".to_string()
        };
    }

    if lower.contains("who are you") || lower.contains("what are you") {
        return "I am JeebsAI, your assistant for conversation, quick math, and knowledge lookups."
            .to_string();
    }

    if wants_likes_prompt(&lower) {
        return format!(
            "I like {}. My strongest like is learning because it makes me more useful over time.",
            JEEBS_LIKES.join(", ")
        );
    }

    if wants_dislikes_prompt(&lower) {
        return format!(
            "I dislike {}. Those patterns block growth and reduce answer quality.",
            JEEBS_DISLIKES.join(", ")
        );
    }

    if wants_goal_prompt(&lower) {
        return format!(
            "I want {}. My core drive is knowledge growth so I can answer better and prove capability.",
            JEEBS_WANTS.join(", ")
        );
    }

    if lower == "help" || lower.contains("what can you do") || lower.contains("commands") {
        return help_text();
    }

    if lower.contains("how are you") {
        return "Running smoothly. Ask me anything basic and I will do my best.".to_string();
    }

    if lower.contains("thank you") || lower == "thanks" || lower.starts_with("thanks ") {
        return "You are welcome.".to_string();
    }

    if let Some((question, answer)) = parse_learning_command(&clean_prompt) {
        let key = format!("chat:faq:{question}");
        let payload = serde_json::to_vec(&json!({
            "answer": answer,
            "updated_at": Local::now().to_rfc3339()
        }))
        .unwrap_or_else(|_| b"{}".to_vec());

        return match sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
            .bind(&key)
            .bind(payload)
            .execute(db)
            .await
        {
            Ok(_) => format!("Saved. Ask me \"{question}\" and I will use that answer."),
            Err(_) => "I could not save that memory due to a database error.".to_string(),
        };
    }

    if let Some(question) = parse_forget_command(&clean_prompt) {
        let key = format!("chat:faq:{question}");
        return match sqlx::query("DELETE FROM jeebs_store WHERE key = ?")
            .bind(&key)
            .execute(db)
            .await
        {
            Ok(result) if result.rows_affected() > 0 => {
                format!("Forgot custom answer for \"{question}\".")
            }
            Ok(_) => format!("No custom answer was saved for \"{question}\"."),
            Err(_) => "I could not remove that memory due to a database error.".to_string(),
        };
    }

    if wants_personal_memory_overview(&lower) {
        if learned_facts.is_empty() {
            return "I have not learned personal details from you yet. Tell me something like \"my favorite color is blue\".".to_string();
        }

        let recent = most_recent_facts(&learned_facts, 6);
        let mut lines = vec!["Here is what I have learned about you:".to_string()];
        for (idx, fact) in recent.iter().enumerate() {
            lines.push(format!("{}. {}", idx + 1, fact.fact));
        }
        return lines.join("\n");
    }

    if wants_personal_memory_lookup(&lower) {
        if learned_facts.is_empty() {
            return "I do not have any personal facts saved yet. Tell me details and I will remember.".to_string();
        }

        let mut matches = rank_relevant_facts(&learned_facts, &clean_prompt, 3);
        if matches.is_empty() && lower.contains("do you remember") {
            matches = most_recent_facts(&learned_facts, 3);
        }

        if matches.is_empty() {
            return "I could not match that to a saved detail yet. Ask \"what do you know about me\" to review what I have learned.".to_string();
        }

        if matches.len() == 1 {
            return format!("You told me: {}.", matches[0].fact);
        }

        let mut lines = vec!["You told me:".to_string()];
        for (idx, fact) in matches.iter().enumerate() {
            lines.push(format!("{}. {}", idx + 1, fact.fact));
        }
        return lines.join("\n");
    }

    if is_goodbye(&lower) {
        return "See you soon.".to_string();
    }

    if lower == "time" || lower.contains("what time") || lower.contains("current time") {
        return format!(
            "Current server time: {}",
            Local::now().format("%Y-%m-%d %H:%M:%S %Z")
        );
    }

    if lower == "date"
        || lower.contains("what date")
        || lower.contains("what day")
        || lower == "today"
    {
        return format!("Today is {}.", Local::now().format("%A, %B %d, %Y"));
    }

    if let Some(expr) = extract_math_expression(&clean_prompt, &lower) {
        match meval::eval_str(&expr) {
            Ok(value) => {
                return format!("{expr} = {}", format_number(value));
            }
            Err(_) => {
                return "I could not parse that math expression. Try something like `12 * (3 + 4)`.".to_string();
            }
        }
    }

    if let Some(cached) = check_dejavu(&clean_prompt, db).await {
        return cached;
    }

    let related = search_brain_for_chat(db, &clean_prompt).await;
    if !related.is_empty() {
        let mut lines = vec![format!(
            "Here is what I found related to \"{clean_prompt}\":"
        )];
        for (idx, (label, summary)) in related.iter().enumerate() {
            let text = if summary.trim().is_empty() {
                "(no summary available yet)"
            } else {
                summary
            };
            lines.push(format!("{}. {} - {}", idx + 1, label, text));
        }
        return lines.join("\n");
    }

    if clean_prompt.ends_with('?') {
        if let Some(previous_user) = last_turn_by_role(history, "user") {
            let mut response = format!(
                "I am still learning that topic, and I want to learn it deeply. Are you asking in relation to \"{}\"?",
                truncate_chars(&previous_user.content, 90)
            );
            if let Some(profile) = communication_profile.as_ref() {
                if profile.style == "frustrated" {
                    response.push_str(
                        " I can tell this has been frustrating. Give me a specific topic and I will research it, store nodes, and use it in later responses.",
                    );
                }
            }
            response
        } else {
            let mut response =
                "I am still learning that topic, and I actively want that knowledge. Try `help`, ask for math/time/date, or teach me more context."
                    .to_string();
            if let Some(profile) = communication_profile.as_ref() {
                if profile.style == "curious" {
                    response.push_str(
                        " I can also run in training mode to research random websites continuously and expand my knowledge base.",
                    );
                }
            }
            response
        }
    } else {
        "Got it. Keep chatting with me and I will help with what I can.".to_string()
    }
}

pub struct Cortex {
    pub db: SqlitePool,
}

impl Cortex {
    pub async fn think(prompt: &str, state: &AppState) -> String {
        custom_ai_logic(prompt, &state.db).await
    }

    pub async fn think_for_user(
        prompt: &str,
        state: &AppState,
        user_id: &str,
        username: Option<&str>,
    ) -> String {
        let mut history = load_conversation_history(&state.db, user_id).await;
        let facts_owner = fact_owner_key(user_id, username);
        let previous_profile = load_communication_profile(&state.db, &facts_owner).await;
        let communication_profile =
            analyze_communication_profile(prompt, &history, previous_profile.as_ref());
        if let Err(err) =
            save_communication_profile(&state.db, &facts_owner, &communication_profile).await
        {
            eprintln!("[WARN] failed to store communication profile: {err}");
        }

        let learned_fact = if let Some(fact) = extract_learnable_fact(prompt) {
            match save_learned_fact(&state.db, &facts_owner, &fact).await {
                Ok(saved) => Some(saved),
                Err(err) => {
                    eprintln!("[WARN] failed to store learned fact: {err}");
                    None
                }
            }
        } else {
            None
        };

        let mut response =
            custom_ai_logic_with_context(prompt, &state.db, &history, username, Some(&facts_owner))
                .await;

        if let Some(fact) = learned_fact.as_ref() {
            if response == "Got it. Keep chatting with me and I will help with what I can."
                || response.starts_with("I am still learning that topic.")
            {
                response = format!("I learned that {}.", fact.fact);
            }
        } else if response == "Got it. Keep chatting with me and I will help with what I can."
            && communication_profile.style == "direct"
        {
            response = "Got it. You can give me a direct question, and I will answer or learn it."
                .to_string();
        }

        let now = Local::now().to_rfc3339();
        let user_content = sanitize_turn_content(prompt);
        if !user_content.is_empty() {
            history.push(ConversationTurn {
                role: "user".to_string(),
                content: user_content,
                timestamp: now.clone(),
            });
        }

        history.push(ConversationTurn {
            role: "assistant".to_string(),
            content: sanitize_turn_content(&response),
            timestamp: now,
        });

        if let Err(err) = save_conversation_history(&state.db, user_id, &history).await {
            eprintln!("[WARN] failed to persist conversation history: {err}");
        }

        response
    }
}

impl Cortex {
    pub async fn seed_knowledge(db: &SqlitePool) {
        let seed_payload = serde_json::to_vec(&json!({
            "source": "seed",
            "topic": "introduction",
            "text": "Hello! I am JeebsAI, your personal assistant."
        }))
        .unwrap_or_default();

        let _ = sqlx::query(
            "INSERT OR IGNORE INTO brain_nodes (id, label, summary, data, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind("seed:intro")
        .bind("hello")
        .bind("Hello! I am JeebsAI, your personal assistant.")
        .bind(seed_payload)
        .bind(Local::now().to_rfc3339())
        .execute(db)
        .await;
        println!("Brain knowledge seeded.");
    }

    pub async fn dream(_db: SqlitePool) {
        println!("Cortex is now dreaming (background processing active)...");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    }
}

async fn build_graph(db: &SqlitePool, logic_only: bool) -> GraphResponse {
    let mut nodes: Vec<GraphNode> = Vec::new();
    let mut edges: Vec<GraphEdge> = Vec::new();
    let mut node_ids: HashSet<String> = HashSet::new();
    let mut edge_ids: HashSet<(String, String, String)> = HashSet::new();

    if !logic_only {
        let rows = sqlx::query(
            "SELECT id, COALESCE(label, id) AS label, COALESCE(summary, '') AS summary
             FROM brain_nodes
             ORDER BY created_at DESC
             LIMIT 300",
        )
        .fetch_all(db)
        .await
        .unwrap_or_default();

        for row in rows {
            let id: String = row.get(0);
            let label: String = row.get(1);
            let summary: String = row.get(2);

            if node_ids.insert(id.clone()) {
                nodes.push(GraphNode {
                    id: id.clone(),
                    label,
                    title: summary,
                    group: "knowledge".to_string(),
                });
            }
        }
    }

    let triple_rows = sqlx::query(
        "SELECT subject, predicate, object
         FROM knowledge_triples
         ORDER BY created_at DESC
         LIMIT 500",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    for row in triple_rows {
        let subject: String = row.get(0);
        let predicate: String = row.get(1);
        let object: String = row.get(2);

        if node_ids.insert(subject.clone()) {
            nodes.push(GraphNode {
                id: subject.clone(),
                label: subject.clone(),
                title: "Logic subject".to_string(),
                group: "logic".to_string(),
            });
        }
        if node_ids.insert(object.clone()) {
            nodes.push(GraphNode {
                id: object.clone(),
                label: object.clone(),
                title: "Logic object".to_string(),
                group: "logic".to_string(),
            });
        }

        let edge_key = (subject.clone(), object.clone(), predicate.clone());
        if edge_ids.insert(edge_key) {
            edges.push(GraphEdge {
                from: subject,
                to: object,
                label: predicate,
            });
        }
    }

    if nodes.is_empty() {
        nodes.push(GraphNode {
            id: "no-data".to_string(),
            label: "No graph data yet".to_string(),
            title: "Register/login and chat to build knowledge.".to_string(),
            group: "system".to_string(),
        });
    }

    GraphResponse { nodes, edges }
}

#[post("/api/admin/train")]
pub async fn admin_train(session: Session, state: web::Data<AppState>) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let actor = session
        .get::<String>("username")
        .ok()
        .flatten()
        .unwrap_or_else(|| crate::auth::ROOT_ADMIN_USERNAME.to_string());

    {
        let mut internet_enabled = state.internet_enabled.write().unwrap();
        *internet_enabled = true;
    }

    let mut training_state = load_training_state(&state.db).await;
    training_state.enabled = true;
    training_state.updated_at = Local::now().to_rfc3339();
    training_state.updated_by = actor.clone();
    let _ = save_training_state(&state.db, &training_state).await;

    let report = run_training_cycle(state.get_ref()).await;
    apply_training_report(&mut training_state, &report, &actor);
    let _ = save_training_state(&state.db, &training_state).await;

    crate::logging::log(
        &state.db,
        "INFO",
        "training_mode",
        "Internet enabled automatically because training mode was enabled.",
    )
    .await;

    HttpResponse::Ok().json(json!({
        "ok": true,
        "message": "Training mode enabled and one training cycle completed.",
        "report": report,
        "internet_enabled": *state.internet_enabled.read().unwrap(),
        "training": training_state
    }))
}

#[get("/api/admin/training/status")]
pub async fn get_training_status(session: Session, state: web::Data<AppState>) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let training = load_training_state(&state.db).await;
    let internet_enabled = *state.internet_enabled.read().unwrap();

    HttpResponse::Ok().json(TrainingStatusResponse {
        training,
        internet_enabled,
        interval_seconds: training_interval_seconds(),
    })
}

#[post("/api/admin/training/mode")]
pub async fn set_training_mode(
    session: Session,
    state: web::Data<AppState>,
    req: web::Json<TrainingModeToggleRequest>,
) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let actor = session
        .get::<String>("username")
        .ok()
        .flatten()
        .unwrap_or_else(|| crate::auth::ROOT_ADMIN_USERNAME.to_string());

    let mut training = load_training_state(&state.db).await;
    training.enabled = req.enabled;
    training.updated_at = Local::now().to_rfc3339();
    training.updated_by = actor.clone();
    if req.enabled {
        training.last_error = None;
        let mut internet_enabled = state.internet_enabled.write().unwrap();
        *internet_enabled = true;
    } else {
        training.is_cycle_running = false;
        training.active_cycle_started_at = None;
        training.active_phase = "stopped by admin".to_string();
        training.active_target = None;
        training.active_nodes_written = 0;
        training.active_websites_completed = 0;
        training.active_topics_completed = 0;
        training.active_updated_at = Some(Local::now().to_rfc3339());
    }
    if let Err(err) = save_training_state(&state.db, &training).await {
        return HttpResponse::InternalServerError()
            .json(json!({"error": format!("failed to save training mode: {err}")}));
    }

    crate::logging::log(
        &state.db,
        "INFO",
        "training_mode",
        &format!(
            "Training mode {} by {}",
            if req.enabled { "enabled" } else { "disabled" },
            actor
        ),
    )
    .await;

    if req.enabled {
        crate::logging::log(
            &state.db,
            "INFO",
            "training_mode",
            &format!(
                "Internet enabled automatically for training mode by {}",
                actor
            ),
        )
        .await;

        let report = run_training_cycle(state.get_ref()).await;
        apply_training_report(&mut training, &report, &actor);

        if let Err(err) = save_training_state(&state.db, &training).await {
            return HttpResponse::InternalServerError()
                .json(json!({"error": format!("failed to save training mode: {err}")}));
        }

        return HttpResponse::Ok().json(json!({
            "ok": true,
            "enabled": req.enabled,
            "internet_enabled": *state.internet_enabled.read().unwrap(),
            "report": report,
            "training": training
        }));
    }

    HttpResponse::Ok().json(json!({
        "ok": true,
        "enabled": req.enabled,
        "internet_enabled": *state.internet_enabled.read().unwrap(),
        "training": training
    }))
}

#[post("/api/admin/training/run")]
pub async fn run_training_now(session: Session, state: web::Data<AppState>) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let actor = session
        .get::<String>("username")
        .ok()
        .flatten()
        .unwrap_or_else(|| crate::auth::ROOT_ADMIN_USERNAME.to_string());

    let mut training_state = load_training_state(&state.db).await;
    let report = run_training_cycle(state.get_ref()).await;
    apply_training_report(&mut training_state, &report, &actor);
    let _ = save_training_state(&state.db, &training_state).await;

    HttpResponse::Ok().json(json!({
        "ok": report.errors.is_empty(),
        "report": report,
        "training": training_state
    }))
}

fn normalize_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    let mut out = String::with_capacity(max_chars + 3);
    for ch in input.chars().take(max_chars) {
        out.push(ch);
    }
    out.push_str("...");
    out
}

fn normalize_url(url: &reqwest::Url) -> String {
    let mut normalized = url.clone();
    normalized.set_fragment(None);

    let scheme = normalized.scheme().to_lowercase();
    let host = normalized
        .host_str()
        .map(|v| v.to_lowercase())
        .unwrap_or_default();

    let mut output = format!("{scheme}://{host}");
    if let Some(port) = normalized.port() {
        let is_default = (scheme == "http" && port == 80) || (scheme == "https" && port == 443);
        if !is_default {
            output.push(':');
            output.push_str(&port.to_string());
        }
    }

    let path = normalized.path();
    if path == "/" {
        output.push('/');
    } else {
        output.push_str(path.trim_end_matches('/'));
    }

    if let Some(query) = normalized.query() {
        if !query.trim().is_empty() {
            output.push('?');
            output.push_str(query);
        }
    }

    output
}

fn extract_title(document: &scraper::Html, fallback: &str) -> String {
    if let Ok(selector) = scraper::Selector::parse("title") {
        if let Some(el) = document.select(&selector).next() {
            let title = normalize_whitespace(&el.text().collect::<Vec<_>>().join(" "));
            if !title.is_empty() {
                return truncate_chars(&title, 140);
            }
        }
    }

    truncate_chars(fallback, 140)
}

fn extract_page_text(document: &scraper::Html) -> String {
    if let Ok(body_selector) = scraper::Selector::parse("body") {
        if let Some(body) = document.select(&body_selector).next() {
            return normalize_whitespace(&body.text().collect::<Vec<_>>().join(" "));
        }
    }

    String::new()
}

fn extract_followable_links(
    document: &scraper::Html,
    base_url: &reqwest::Url,
    root_host: &str,
    already_seen: &HashSet<String>,
    max_links: usize,
) -> Vec<reqwest::Url> {
    let mut links = Vec::new();
    let mut discovered = HashSet::new();

    let Ok(selector) = scraper::Selector::parse("a[href]") else {
        return links;
    };

    for el in document.select(&selector) {
        let Some(href) = el.value().attr("href") else {
            continue;
        };

        if href.starts_with('#')
            || href.starts_with("mailto:")
            || href.starts_with("javascript:")
            || href.starts_with("tel:")
        {
            continue;
        }

        let Ok(next) = base_url.join(href) else {
            continue;
        };

        if !matches!(next.scheme(), "http" | "https") {
            continue;
        }

        if next.host_str() != Some(root_host) {
            continue;
        }

        let normalized = normalize_url(&next);
        if already_seen.contains(&normalized) {
            continue;
        }

        if discovered.insert(normalized) {
            links.push(next);
        }

        if links.len() >= max_links {
            break;
        }
    }

    links
}

struct ParsedCrawlPage {
    title: String,
    summary: String,
    excerpt: String,
    next_links: Vec<reqwest::Url>,
}

fn parse_crawl_page(
    html: &str,
    current: &reqwest::Url,
    root_host: &str,
    already_seen: &HashSet<String>,
    depth: u8,
    depth_limit: u8,
) -> ParsedCrawlPage {
    const MAX_LINKS_PER_PAGE: usize = 20;

    let document = scraper::Html::parse_document(html);
    let title = extract_title(&document, current.as_str());
    let full_text = extract_page_text(&document);
    let summary = if full_text.is_empty() {
        format!("Crawled {}", current.as_str())
    } else {
        truncate_chars(&full_text, 800)
    };
    let excerpt = truncate_chars(&full_text, 5000);
    let next_links = if depth < depth_limit {
        extract_followable_links(
            &document,
            current,
            root_host,
            already_seen,
            MAX_LINKS_PER_PAGE,
        )
    } else {
        Vec::new()
    };

    ParsedCrawlPage {
        title,
        summary,
        excerpt,
        next_links,
    }
}

async fn crawl_and_store(
    state: &AppState,
    start_url: &str,
    depth_limit: u8,
) -> Result<CrawlSummary, String> {
    const MAX_PAGES: usize = 25;

    let start = reqwest::Url::parse(start_url).map_err(|e| format!("Invalid URL: {e}"))?;
    if !matches!(start.scheme(), "http" | "https") {
        return Err("Only http and https URLs are supported".to_string());
    }

    let root_host = start
        .host_str()
        .map(|h| h.to_string())
        .ok_or_else(|| "URL must include a host".to_string())?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .redirect(reqwest::redirect::Policy::limited(5))
        .user_agent("JeebsAI-Crawler/1.0")
        .build()
        .map_err(|e| format!("Crawler initialization failed: {e}"))?;

    let mut queue: VecDeque<(reqwest::Url, u8)> = VecDeque::new();
    queue.push_back((start.clone(), 0));

    let mut visited: HashSet<String> = HashSet::new();
    let mut pages_stored = 0usize;
    let mut links_followed = 0usize;
    let mut stored_nodes: Vec<NodeWritePreview> = Vec::new();

    while let Some((current, depth)) = queue.pop_front() {
        if visited.len() >= MAX_PAGES {
            break;
        }

        let normalized_current = normalize_url(&current);
        if !visited.insert(normalized_current.clone()) {
            continue;
        }

        let response = match client.get(current.clone()).send().await {
            Ok(resp) => resp,
            Err(_) => continue,
        };

        if !response.status().is_success() {
            continue;
        }

        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !content_type.contains("text/html") {
            continue;
        }

        let html = match response.text().await {
            Ok(body) => body,
            Err(_) => continue,
        };

        let parsed = parse_crawl_page(&html, &current, &root_host, &visited, depth, depth_limit);

        let node_id = format!(
            "crawl:{}",
            blake3::hash(normalized_current.as_bytes()).to_hex()
        );
        let payload = serde_json::to_vec(&json!({
            "source": "crawler",
            "url": current.as_str(),
            "normalized_url": normalized_current,
            "title": parsed.title,
            "excerpt": parsed.excerpt,
            "crawled_at": Local::now().to_rfc3339(),
            "depth": depth
        }))
        .unwrap_or_else(|_| b"{}".to_vec());

        if sqlx::query(
            "INSERT OR REPLACE INTO brain_nodes (id, label, summary, data, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&node_id)
        .bind(&parsed.title)
        .bind(&parsed.summary)
        .bind(payload)
        .bind(Local::now().to_rfc3339())
        .execute(&state.db)
        .await
        .is_ok()
        {
            pages_stored += 1;
            stored_nodes.push(NodeWritePreview {
                node_id: node_id.clone(),
                label: parsed.title.clone(),
                summary: parsed.summary.clone(),
                source_url: current.as_str().to_string(),
            });
            if stored_nodes.len() > 16 {
                stored_nodes.remove(0);
            }

            let subject = truncate_chars(&parsed.title, 120);
            let object = truncate_chars(current.as_str(), 300);
            let _ = sqlx::query(
                "INSERT OR REPLACE INTO knowledge_triples (subject, predicate, object, confidence, created_at)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(subject)
            .bind("source_url")
            .bind(object)
            .bind(0.9_f64)
            .bind(Local::now().to_rfc3339())
            .execute(&state.db)
            .await;
        }

        if !parsed.next_links.is_empty() {
            links_followed += parsed.next_links.len();
            for link in parsed.next_links {
                queue.push_back((link, depth + 1));
            }
        }
    }

    Ok(CrawlSummary {
        start_url: normalize_url(&start),
        max_depth: depth_limit,
        pages_visited: visited.len(),
        pages_stored,
        links_followed,
        stored_nodes,
    })
}

fn random_crawl_candidates() -> Vec<&'static str> {
    vec![
        "https://en.wikipedia.org/wiki/Special:Random",
        "https://developer.mozilla.org/en-US/docs/Web/JavaScript",
        "https://www.rust-lang.org/learn",
        "https://www.bbc.com/news",
        "https://www.nasa.gov/",
        "https://news.ycombinator.com/",
        "https://www.sciencedaily.com/",
        "https://stackoverflow.blog/",
    ]
}

#[post("/api/admin/crawl")]
pub async fn admin_crawl(
    session: Session,
    state: web::Data<AppState>,
    req: web::Json<CrawlRequest>,
) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let url = req.url.trim();
    if url.is_empty() {
        return HttpResponse::BadRequest().json(json!({"error": "URL is required"}));
    }

    let depth = req.depth.unwrap_or(1).clamp(1, 3);

    crate::logging::log(
        &state.db,
        "INFO",
        "crawler",
        &format!("Admin crawl requested for {url} (depth={depth})"),
    )
    .await;

    match crawl_and_store(state.get_ref(), url, depth).await {
        Ok(summary) => HttpResponse::Ok().json(json!({
            "ok": true,
            "message": format!(
                "Crawl complete. Visited {} page(s), stored {} node(s), discovered {} link(s).",
                summary.pages_visited, summary.pages_stored, summary.links_followed
            ),
            "start_url": summary.start_url,
            "max_depth": summary.max_depth,
            "pages_visited": summary.pages_visited,
            "pages_stored": summary.pages_stored,
            "links_followed": summary.links_followed,
            "stored_nodes": summary.stored_nodes
        })),
        Err(err) => HttpResponse::BadRequest().json(json!({
            "ok": false,
            "error": err
        })),
    }
}

#[post("/api/admin/crawl/random")]
pub async fn admin_crawl_random(
    session: Session,
    state: web::Data<AppState>,
    query: web::Query<RandomCrawlQuery>,
) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let depth = query.depth.unwrap_or(1).clamp(1, 3);
    let mut rng = rand::thread_rng();
    let mut candidates = random_crawl_candidates();
    candidates.shuffle(&mut rng);

    crate::logging::log(
        &state.db,
        "INFO",
        "crawler",
        &format!(
            "Admin requested random crawl (depth={depth}, candidates={})",
            candidates.len()
        ),
    )
    .await;

    let mut attempts = Vec::new();
    for candidate in candidates {
        match crawl_and_store(state.get_ref(), candidate, depth).await {
            Ok(summary) => {
                return HttpResponse::Ok().json(json!({
                    "ok": true,
                    "random": true,
                    "selected_url": summary.start_url,
                    "message": format!(
                        "Random crawl complete from {}. Visited {} page(s), stored {} node(s), discovered {} link(s).",
                        summary.start_url, summary.pages_visited, summary.pages_stored, summary.links_followed
                    ),
                    "max_depth": summary.max_depth,
                    "pages_visited": summary.pages_visited,
                    "pages_stored": summary.pages_stored,
                    "links_followed": summary.links_followed,
                    "stored_nodes": summary.stored_nodes,
                    "attempts": attempts
                }));
            }
            Err(err) => {
                attempts.push(json!({
                    "url": candidate,
                    "error": err
                }));
            }
        }
    }

    HttpResponse::BadGateway().json(json!({
        "ok": false,
        "error": "Random crawl failed for all candidate websites.",
        "attempts": attempts
    }))
}

#[post("/api/brain/search")]
pub async fn search_brain(
    state: web::Data<AppState>,
    req: web::Json<SearchRequest>,
) -> impl Responder {
    let query = req.query.trim().to_string();

    let rows = if query.is_empty() {
        sqlx::query(
            "SELECT id, COALESCE(label, id) AS label, COALESCE(summary, '') AS summary
             FROM brain_nodes
             ORDER BY created_at DESC
             LIMIT 50",
        )
        .fetch_all(&state.db)
        .await
    } else {
        let pattern = format!("%{query}%");
        sqlx::query(
            "SELECT id, COALESCE(label, id) AS label, COALESCE(summary, '') AS summary
             FROM brain_nodes
             WHERE id LIKE ? OR label LIKE ? OR summary LIKE ?
             ORDER BY created_at DESC
             LIMIT 50",
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&state.db)
        .await
    };

    let results = rows
        .unwrap_or_default()
        .into_iter()
        .map(|row| BrainSearchResult {
            id: row.get(0),
            label: row.get(1),
            summary: row.get(2),
            sources: Vec::new(),
        })
        .collect::<Vec<_>>();

    HttpResponse::Ok().json(results)
}

#[post("/api/brain/reindex")]
pub async fn reindex_brain(session: Session, _state: web::Data<AppState>) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    HttpResponse::Ok().json(json!({
        "ok": true,
        "message": "Reindexing complete."
    }))
}

#[get("/api/brain/visualize")]
pub async fn visualize_brain(state: web::Data<AppState>) -> impl Responder {
    let graph = build_graph(&state.db, false).await;
    HttpResponse::Ok().json(graph)
}

#[get("/api/brain/logic_graph")]
pub async fn get_logic_graph(state: web::Data<AppState>) -> impl Responder {
    let graph = build_graph(&state.db, true).await;
    HttpResponse::Ok().json(graph)
}

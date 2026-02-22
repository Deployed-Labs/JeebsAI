use actix_web::{get, post, web, HttpResponse, Responder};
// use crate::state::AppState;
use crate::brain::{BrainNode, KnowledgeTriple};
#[get("/api/brain/logic_graph")]
pub async fn logic_graph_endpoint(state: web::Data<AppState>) -> impl Responder {
    let db = &state.db;
    // Fetch all nodes
    let nodes = crate::brain::search_knowledge(db, "").await;
    // Build GraphNode list
    let graph_nodes: Vec<GraphNode> = nodes.iter().map(|n| {
        GraphNode {
            id: n.key.clone(),
            label: n.label.clone(),
            title: n.summary.clone(),
            group: n.label.clone(),
        }
    }).collect();
    // Build GraphEdge list from triples
    let mut graph_edges: Vec<GraphEdge> = Vec::new();
    for node in &nodes {
        let triples = crate::brain::get_triples_for_subject(db, &node.key).await;
        for triple in triples {
            graph_edges.push(GraphEdge {
                from: triple.subject.clone(),
                to: triple.object.clone(),
                label: triple.predicate.clone(),
            });
        }
    }
    let response = GraphResponse {
        nodes: graph_nodes,
        edges: graph_edges,
    };
    HttpResponse::Ok().json(response)
}
// use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};
use std::env;
use std::time::Instant;

use crate::state::AppState;
use crate::utils::decode_all;


#[derive(Debug, Deserialize)]
pub struct AdvancedSearchRequest {
    pub query: String,
    pub max_results: Option<usize>,
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
struct TrainingTrackProgress {
    name: String,
    status: String,
    progress_percent: u8,
    mastered: bool,
    nodes_written: u64,
    threshold: u64,
    current_goal: String,
    last_learned_at: Option<String>,
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
    text_chars_learned: u64,
    focus_topic: Option<String>,
    focus_topic_nodes_written: u64,
    learned_items_count: u64,
    errors: Vec<String>,
}

fn default_active_phase() -> String {
    "idle".to_string()
}

fn normalize_training_topic_input(input: &str) -> Option<String> {
    let compact = crate::evolution::normalize_whitespace(input);
    if compact.is_empty() {
        return None;
    }
    Some(truncate_chars(&compact, 160))
}

fn build_track(name: &str, threshold: u64, goal: &str) -> TrainingTrackProgress {
    TrainingTrackProgress {
        name: name.to_string(),
        status: "active".to_string(),
        progress_percent: 0,
        mastered: false,
        nodes_written: 0,
        threshold,
        current_goal: goal.to_string(),
        last_learned_at: None,
    }
}

fn default_training_tracks() -> Vec<TrainingTrackProgress> {
    vec![
        build_track(
            "conversation skills",
            80,
            "learn clarifying questions, structured replies, and tone adaptation",
        ),
        build_track(
            "rust programming",
            60,
            "master ownership, concurrency, and production-grade Rust patterns",
        ),
        build_track(
            "python programming",
            60,
            "master Python tooling, data workflows, and async application patterns",
        ),
        build_track(
            "data compression and storage efficiency",
            50,
            "learn how to store more information in less space with compression and indexing",
        ),
        TrainingTrackProgress {
            status: "queued".to_string(),
            ..build_track(
                "go programming",
                40,
                "unlock after Rust and Python mastery, then learn idiomatic Go systems design",
            )
        },
        TrainingTrackProgress {
            status: "queued".to_string(),
            ..build_track(
                "javascript and typescript",
                40,
                "unlock after Rust and Python mastery, then learn full-stack JS/TS architecture",
            )
        },
        TrainingTrackProgress {
            status: "queued".to_string(),
            ..build_track(
                "c and c++",
                40,
                "unlock after Rust and Python mastery, then learn low-level optimization",
            )
        },
    ]
}

fn track_hit_score(track_name: &str, item: &TrainingLearnedItem) -> u64 {
    let corpus = format!(
        "{} {} {}",
        item.topic.to_ascii_lowercase(),
        item.title.to_ascii_lowercase(),
        item.summary.to_ascii_lowercase()
    );
    let keywords: &[&str] = match track_name {
        "conversation skills" => &[
            "conversation",
            "dialogue",
            "communication",
            "listening",
            "clarifying",
            "question",
            "language model prompting",
            "interaction design",
        ],
        "rust programming" => &[
            "rust",
            "cargo",
            "borrow checker",
            "ownership",
            "lifetime",
            "actix",
            "tokio",
            "serde",
            "sqlx",
        ],
        "python programming" => &[
            "python", "pandas", "numpy", "fastapi", "asyncio", "flask", "django", "pytest",
        ],
        "data compression and storage efficiency" => &[
            "compression",
            "storage",
            "dedup",
            "columnar",
            "parquet",
            "zstd",
            "lz4",
            "dictionary encoding",
            "delta encoding",
            "data structure",
        ],
        "go programming" => &["go language", "golang", "goroutine", "go programming"],
        "javascript and typescript" => &[
            "javascript",
            "typescript",
            "node.js",
            "nodejs",
            "react",
            "frontend",
        ],
        "c and c++" => &[
            "c programming",
            "c++",
            "memory management",
            "low-level",
            "pointer",
        ],
        _ => &[],
    };

    if keywords.is_empty() {
        return 0;
    }
    keywords.iter().filter(|kw| corpus.contains(**kw)).count() as u64
}

fn refresh_track_statuses(mode: &mut TrainingModeState) {
    let prerequisites_mastered = LANGUAGE_UNLOCK_PREREQUISITES.iter().all(|required| {
        mode.learning_tracks
            .iter()
            .find(|track| track.name == *required)
            .map(|track| track.mastered)
            .unwrap_or(false)
    });

    for track in &mut mode.learning_tracks {
        let threshold = track.threshold.max(1);
        let pct = ((track.nodes_written.saturating_mul(100)) / threshold).min(100);
        track.progress_percent = pct as u8;
        track.mastered = track.nodes_written >= threshold;

        if ADVANCED_LANGUAGE_TRACKS.contains(&track.name.as_str())
            && !prerequisites_mastered
            && !track.mastered
        {
            track.status = "queued".to_string();
            track.current_goal = "waiting for Rust and Python mastery unlock".to_string();
            continue;
        }

        track.status = if track.mastered {
            "mastered".to_string()
        } else {
            "active".to_string()
        };
    }
}

fn smartness_score(mode: &TrainingModeState) -> f64 {
    let mastery = mode
        .learning_tracks
        .iter()
        .filter(|track| track.mastered)
        .count() as f64;
    (mode.total_nodes_written as f64 * 0.45
        + mode.total_topics_processed as f64 * 0.25
        + mode.total_websites_scraped as f64 * 0.2
        + mastery * 25.0)
        .round()
}

fn curriculum_topics_from_state(mode: &TrainingModeState) -> Vec<String> {
    let mut topics = vec![
        "how to store more data in less space".to_string(),
        "lossless compression techniques".to_string(),
        "conversation skills for ai assistants".to_string(),
    ];

    for track in &mode.learning_tracks {
        if track.status != "active" || track.mastered {
            continue;
        }
        topics.push(track.name.clone());
        if !track.current_goal.trim().is_empty() {
            topics.push(track.current_goal.clone());
        }
    }

    let mut dedup = Vec::<String>::new();
    let mut seen = HashSet::<String>::new();
    for topic in topics {
        let normalized = topic.to_ascii_lowercase();
        if seen.insert(normalized) {
            dedup.push(topic);
        }
    }
    dedup
}

fn matches_focus_topic(focus_topic: &str, item: &TrainingLearnedItem) -> bool {
    let focus = focus_topic.to_ascii_lowercase();
    if focus.is_empty() {
        return false;
    }
    let corpus = format!(
        "{} {} {}",
        item.topic.to_ascii_lowercase(),
        item.title.to_ascii_lowercase(),
        item.summary.to_ascii_lowercase()
    );
    corpus.contains(&focus)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrainingModeState {
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
    #[serde(default)]
    focus_topic: Option<String>,
    #[serde(default)]
    smartness_score: f64,
    #[serde(default)]
    total_text_chars_learned: u64,
    #[serde(default)]
    learning_tracks: Vec<TrainingTrackProgress>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrainingModeToggleRequest {
    enabled: bool,
    #[serde(default)]
    focus_topic: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrainingFocusTopicRequest {
    topic: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct TrainingStatusResponse {
    pub training: TrainingModeState,
    pub internet_enabled: bool,
    pub interval_seconds: u64,
}

/// Snapshot of training state for admin/status endpoints.
pub async fn get_training_status(
    db: &SqlitePool,
    internet_enabled: bool,
) -> TrainingStatusResponse {
    let training = load_training_state(db).await;
    TrainingStatusResponse {
        training,
        internet_enabled,
        interval_seconds: training_interval_seconds(),
    }
}

pub async fn set_training_focus_for_trainer(
    db: &SqlitePool,
    topic: &str,
    actor: &str,
) -> Result<(), String> {
    let mut training = load_training_state(db).await;
    training.enabled = true;
    training.focus_topic = Some(topic.trim().to_string());
    training.updated_at = Local::now().to_rfc3339();
    training.updated_by = actor.to_string();
    training.active_phase = "trainer focus set".to_string();
    training.active_target = Some(topic.trim().to_string());
    training.active_updated_at = Some(Local::now().to_rfc3339());

    if let Err(err) = save_training_state(db, &training).await {
        return Err(format!("failed to save training focus: {err}"));
    }

    let _ = crate::toggle_manager::save_training_toggle_state(db, true).await;

    Ok(())
}

pub async fn set_training_enabled_for_trainer(
    db: &SqlitePool,
    enabled: bool,
    actor: &str,
) -> Result<(), String> {
    let mut training = load_training_state(db).await;
    training.enabled = enabled;
    training.updated_at = Local::now().to_rfc3339();
    training.updated_by = actor.to_string();

    if !enabled {
        training.is_cycle_running = false;
        training.active_cycle_started_at = None;
        training.active_phase = "stopped by trainer".to_string();
        training.active_target = None;
        training.active_nodes_written = 0;
        training.active_websites_completed = 0;
        training.active_topics_completed = 0;
        training.active_updated_at = Some(Local::now().to_rfc3339());
    }

    if let Err(err) = save_training_state(db, &training).await {
        return Err(format!("failed to save training mode: {err}"));
    }

    let _ = crate::toggle_manager::save_training_toggle_state(db, enabled).await;

    Ok(())
}

pub async fn sync_training_state_with_toggle(
    db: &SqlitePool,
    enabled: bool,
    actor: &str,
) -> Result<(), String> {
    let mut training = load_training_state(db).await;
    if training.enabled == enabled {
        return Ok(());
    }
    training.enabled = enabled;
    training.updated_at = Local::now().to_rfc3339();
    training.updated_by = actor.to_string();
    if !enabled {
        training.is_cycle_running = false;
        training.active_cycle_started_at = None;
        training.active_phase = "stopped by startup sync".to_string();
        training.active_target = None;
        training.active_nodes_written = 0;
        training.active_websites_completed = 0;
        training.active_topics_completed = 0;
        training.active_updated_at = Some(Local::now().to_rfc3339());
    }
    if let Err(err) = save_training_state(db, &training).await {
        return Err(format!("failed to sync training state: {err}"));
    }
    Ok(())
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
    text_chars_learned: usize,
    track_hits: HashMap<String, u64>,
    focus_topic: Option<String>,
    focus_topic_nodes_written: usize,
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

const LANGUAGE_UNLOCK_PREREQUISITES: &[&str] = &["rust programming", "python programming"];
const ADVANCED_LANGUAGE_TRACKS: &[&str] =
    &["go programming", "javascript and typescript", "c and c++"];

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
        enabled: true, // AUTO-RUN training mode by default
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
        focus_topic: None,
        smartness_score: 0.0,
        total_text_chars_learned: 0,
        learning_tracks: default_training_tracks(),
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
    let mut state = serde_json::from_slice::<TrainingModeState>(&raw)
        .ok()
        .or_else(|| {
            decode_all(&raw)
                .ok()
                .and_then(|decoded| serde_json::from_slice::<TrainingModeState>(&decoded).ok())
        })
        .unwrap_or_else(training_state_default);

    if state.learning_tracks.is_empty() {
        state.learning_tracks = default_training_tracks();
    }
    refresh_track_statuses(&mut state);
    state.smartness_score = smartness_score(&state);
    state
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

    // Log training state updates for monitoring and anomaly detection
    let _ = crate::logging::log(
        db,
        "INFO",
        "TRAINING",
        &format!(
            "Training state saved: enabled={} updated_by={} last_learned_items={} total_nodes_written={}",
            mode.enabled,
            mode.updated_by,
            mode.last_learned_items.len(),
            mode.total_nodes_written
        ),
    )
    .await;
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
        text_chars_learned: report.text_chars_learned as u64,
        focus_topic: report.focus_topic.clone(),
        focus_topic_nodes_written: report.focus_topic_nodes_written as u64,
        learned_items_count: report.learned_items.len() as u64,
        errors: report.errors.clone(),
    }
}

fn apply_training_report(mode: &mut TrainingModeState, report: &TrainingCycleReport, actor: &str) {
    if mode.learning_tracks.is_empty() {
        mode.learning_tracks = default_training_tracks();
    }

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
    mode.total_text_chars_learned = mode
        .total_text_chars_learned
        .saturating_add(report.text_chars_learned as u64);

    mode.last_topics = report.topics.clone();
    mode.last_websites = report.websites_scraped.clone();
    mode.last_learned_items = report.learned_items.clone();
    mode.last_error = report.errors.first().cloned();
    mode.last_cycle_duration_ms = Some(report.duration_ms);
    mode.last_cycle_nodes_written = report.nodes_written as u64;
    mode.last_cycle_errors = report.errors.clone();

    for track in &mut mode.learning_tracks {
        let increment = report.track_hits.get(&track.name).copied().unwrap_or(0);
        if increment > 0 {
            track.nodes_written = track.nodes_written.saturating_add(increment);
            track.last_learned_at = Some(report.cycle_finished_at.clone());
        }
    }
    refresh_track_statuses(mode);

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
    mode.smartness_score = smartness_score(mode);
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

pub async fn collect_training_topics(db: &SqlitePool, limit: usize) -> Vec<String> {
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
pub struct ExternalLearningDoc {
    pub title: String,
    pub url: String,
    pub summary: String,
    pub topic: String,
}

pub async fn query_wikipedia_docs(
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

pub async fn store_external_learning_doc(
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

/// Comprehensive list of websites for random crawling during training
fn random_crawl_candidates() -> Vec<&'static str> {
    vec![
        // Science & Research (Major Universities & Institutions)
        "https://arxiv.org",
        "https://www.nature.com",
        "https://www.science.org",
        "https://www.nasa.gov",
        "https://www.mit.edu",
        "https://www.stanford.edu",
        "https://www.harvard.edu",
        "https://www.caltech.edu",
        "https://www.berkeley.edu",
        "https://www.ox.ac.uk",
        "https://www.cam.ac.uk",
        "https://www.cern.ch",
        // Technology & AI
        "https://github.com",
        "https://www.arxiv.org/list/cs.AI",
        "https://openai.com",
        "https://www.deepmind.com",
        "https://ai.google",
        "https://research.facebook.com",
        "https://www.ibm.com/research",
        "https://www.microsoft.com/research",
        // News & Current Events
        "https://www.bbc.com",
        "https://www.theguardian.com",
        "https://www.nytimes.com",
        "https://www.economist.com",
        "https://www.wired.com",
        "https://news.ycombinator.com",
        "https://www.techcrunch.com",
        "https://www.theverge.com",
        // Developer Resources
        "https://developer.mozilla.org",
        "https://www.w3schools.com",
        "https://stackoverflow.com",
        "https://www.python.org",
        "https://www.rust-lang.org",
        "https://golang.org",
        "https://www.freecodecamp.org",
        "https://docs.microsoft.com",
        // Wikipedia (Knowledge Base)
        "https://en.wikipedia.org",
        "https://en.wikipedia.org/wiki/Artificial_intelligence",
        "https://en.wikipedia.org/wiki/Science",
        "https://en.wikipedia.org/wiki/Technology",
        "https://en.wikipedia.org/wiki/Mathematics",
        "https://en.wikipedia.org/wiki/Physics",
        "https://en.wikipedia.org/wiki/Biology",
        "https://en.wikipedia.org/wiki/Chemistry",
        // Educational Platforms
        "https://www.edx.org",
        "https://www.coursera.org",
        "https://www.khanacademy.org",
        "https://www.udacity.com",
        "https://www.brilliant.org",
        // Science & Nature Journals
        "https://www.cell.com",
        "https://www.sciencedirect.com",
        "https://www.springer.com",
        "https://academic.oup.com",
        "https://www.elsevier.com",
        // Open-Source & Development
        "https://www.linux.org",
        "https://www.apache.org",
        "https://www.eclipse.org",
        "https://www.mozilla.org",
        "https://www.kde.org",
        // Specialized Topics
        "https://phys.org",
        "https://www.space.com",
        "https://www.sciencedaily.com",
        "https://www.medicalnewstoday.com",
        "https://www.psychologytoday.com",
        // Quantum Computing & Advanced Physics
        "https://quantum.ibm.com",
        "https://www.dwavesys.com",
        "https://www.qiskit.org",
        // Machine Learning & Data Science
        "https://www.tensorflow.org",
        "https://pytorch.org",
        "https://www.kaggle.com",
        "https://www.paperswithcode.com",
        // Economics & Finance
        "https://www.imf.org",
        "https://www.worldbank.org",
        "https://www.ecb.europa.eu",
        "https://www.federalreserve.gov",
        // Climate & Environment
        "https://climate.nasa.gov",
        "https://www.ipcc.ch",
        "https://www.un.org/en/climatechange",
        // History & Culture
        "https://www.britannica.com",
        "https://www.historicengland.org.uk",
        "https://www.smithsonianmag.com",
        // Health & Medicine
        "https://www.nih.gov",
        "https://www.cdc.gov",
        "https://www.who.int",
        "https://www.healthline.com",
        // Philosophy & Thought
        "https://plato.stanford.edu",
        "https://www.iep.utm.edu",
        // General Knowledge & Reference
        "https://www.merriam-webster.com",
        "https://dictionary.cambridge.org",
        "https://www.oxforddictionaries.com",
        // Random Topic Exploration
        "https://en.wikipedia.org/wiki/Special:Random",
        "https://www.reddit.com/r/todayilearned",
        "https://www.reddit.com/r/science",
        "https://www.reddit.com/r/AskScience",
    ]
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    input.chars().take(max_chars).collect()
}

fn canonical_prompt_key(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// ===== TEMPLATE PROPOSAL ENDPOINTS =====

#[post("/api/brain/template-proposals/generate")]
pub async fn generate_template_proposals_endpoint(state: web::Data<AppState>) -> impl Responder {
    match crate::proposals::generate_template_proposals(&state.db).await {
        Some(proposals) => {
            let formatted = crate::proposals::format_template_proposals(&proposals);
            HttpResponse::Ok().json(json!({
                "success": true,
                "proposals": proposals.proposals.iter().map(|p| {
                    json!({
                        "id": p.id,
                        "type": p.template_type,
                        "title": p.title,
                        "description": p.description,
                        "steps": p.implementation_steps,
                        "impact": p.expected_impact,
                        "difficulty": p.difficulty_level,
                        "estimated_hours": p.estimated_time_hours,
                        "status": p.status,
                        "created_at": p.created_at,
                    })
                }).collect::<Vec<_>>(),
                "message": formatted,
                "selection_round": proposals.selection_round,
            }))
        }
        None => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": "Failed to generate template proposals"
        })),
    }
}

#[get("/api/brain/template-proposals")]
pub async fn get_template_proposals_endpoint(state: web::Data<AppState>) -> impl Responder {
    match crate::proposals::get_template_proposals(&state.db).await {
        Some(proposals) => {
            let formatted = crate::proposals::format_template_proposals(&proposals);
            HttpResponse::Ok().json(json!({
                "success": true,
                "proposals": proposals.proposals.iter().map(|p| {
                    json!({
                        "id": p.id,
                        "type": p.template_type,
                        "title": p.title,
                        "description": p.description,
                        "steps": p.implementation_steps,
                        "impact": p.expected_impact,
                        "difficulty": p.difficulty_level,
                        "estimated_hours": p.estimated_time_hours,
                        "status": p.status,
                        "created_at": p.created_at,
                    })
                }).collect::<Vec<_>>(),
                "message": formatted,
                "selection_round": proposals.selection_round,
            }))
        }
        None => {
            // No proposals yet, generate them
            match crate::proposals::generate_template_proposals(&state.db).await {
                Some(proposals) => {
                    let formatted = crate::proposals::format_template_proposals(&proposals);
                    HttpResponse::Ok().json(json!({
                        "success": true,
                        "proposals": proposals.proposals.iter().map(|p| {
                            json!({
                                "id": p.id,
                                "type": p.template_type,
                                "title": p.title,
                                "description": p.description,
                                "steps": p.implementation_steps,
                                "impact": p.expected_impact,
                                "difficulty": p.difficulty_level,
                                "estimated_hours": p.estimated_time_hours,
                                "status": p.status,
                                "created_at": p.created_at,
                            })
                        }).collect::<Vec<_>>(),
                        "message": formatted,
                        "selection_round": proposals.selection_round,
                    }))
                }
                None => HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": "Failed to generate template proposals"
                })),
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateProposalStatusRequest {
    pub proposal_id: String,
    pub status: String,
}

#[post("/api/brain/template-proposals/update-status")]
pub async fn update_proposal_status_endpoint(
    req: web::Json<UpdateProposalStatusRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let valid_statuses = vec![
        "proposed",
        "accepted",
        "rejected",
        "in_progress",
        "completed",
    ];

    if !valid_statuses.contains(&req.status.as_str()) {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": "Invalid status. Valid options: proposed, accepted, rejected, in_progress, completed"
        }));
    }

    if crate::proposals::update_template_proposal_status(&state.db, &req.proposal_id, &req.status)
        .await
    {
        HttpResponse::Ok().json(json!({
            "success": true,
            "message": format!("Proposal {} status updated to {}", req.proposal_id, req.status)
        }))
    } else {
        HttpResponse::NotFound().json(json!({
            "success": false,
            "error": "Proposal not found"
        }))
    }
}

#[get("/api/brain/template-proposals/statistics")]
pub async fn get_proposal_statistics_endpoint(state: web::Data<AppState>) -> impl Responder {
    match crate::proposals::get_proposal_statistics(&state.db).await {
        Some(stats) => HttpResponse::Ok().json(json!({
            "success": true,
            "statistics": stats
        })),
        None => HttpResponse::Ok().json(json!({
            "success": true,
            "statistics": {
                "total_proposals": 0,
                "accepted": 0,
                "in_progress": 0,
                "completed": 0,
                "message": "No proposals generated yet"
            }
        })),
    }
}

// ===== DEEP LEARNING ENDPOINTS =====

#[derive(Debug, Deserialize)]
pub struct StartDeepLearningRequest {
    pub topic: String,
}

#[derive(Debug, Deserialize)]
pub struct AddFactRequest {
    pub session_id: String,
    pub fact: String,
    pub source: String,
    pub importance: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct AddProblemRequest {
    pub session_id: String,
    pub problem: String,
    pub solution: String,
    pub explanation: String,
    pub difficulty: String,
}

#[post("/api/learning/start-deep-learning")]
pub async fn start_deep_learning(
    req: web::Json<StartDeepLearningRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    match crate::deep_learning::start_deep_learning_session(&state.db, &req.topic).await {
        Ok(session) => HttpResponse::Ok().json(json!({
            "success": true,
            "session_id": session.id,
            "topic": session.topic,
            "subtopics": session.subtopics,
            "message": format!("Started deep learning session on: {}", session.topic)
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": e
        })),
    }
}

#[post("/api/learning/add-fact")]
pub async fn add_learned_fact(
    req: web::Json<AddFactRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    match crate::deep_learning::add_learned_fact(
        &state.db,
        &req.session_id,
        &req.fact,
        &req.source,
        req.importance.unwrap_or(0.7),
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Fact learned and stored"
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": e
        })),
    }
}

#[post("/api/learning/add-practice-problem")]
pub async fn add_practice_problem(
    req: web::Json<AddProblemRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    match crate::deep_learning::add_practice_problem(
        &state.db,
        &req.session_id,
        &req.problem,
        &req.solution,
        &req.explanation,
        &req.difficulty,
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Practice problem added for deeper learning"
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": e
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct RunExtendedRequest {
    pub session_id: String,
    pub minutes: u32,
    pub inference: Option<bool>,
}

#[post("/api/learning/run-extended")]
pub async fn run_extended_learning(req: web::Json<RunExtendedRequest>, state: web::Data<AppState>) -> impl Responder {
    let inference = req.inference.unwrap_or(false);
    let session_id = req.session_id.clone();
    // create run metadata in jeebs_store
    let run_id = uuid::Uuid::new_v4().to_string();
    let run_key = format!("deeplearn_run:{}", run_id);
    let meta = json!({
        "run_id": run_id,
        "session_id": session_id,
        "minutes": req.minutes,
        "inference": inference,
        "started_at": chrono::Local::now().to_rfc3339(),
        "status": "running",
        "progress_percent": 0,
        "cancelled": false
    });
    let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(&run_key)
        .bind(serde_json::to_vec(&meta).unwrap_or_default())
        .execute(&state.db)
        .await;

    // spawn background task so request returns quickly
    let db = state.db.clone();
    let run_id_clone = run_id.clone();
    actix_web::rt::spawn(async move {
        let _ = crate::deep_learning::run_extended_learning_session(&db, &session_id, req.minutes, inference, &run_id_clone).await;
    });

    HttpResponse::Ok().json(json!({"success": true, "message": "Extended learning started", "run_id": run_id}))
}

#[get("/api/learning/extended-runs")]
pub async fn list_extended_runs(state: web::Data<AppState>) -> impl Responder {
    // Query jeebs_store for keys starting with deeplearn_run:
    match sqlx::query("SELECT key, value FROM jeebs_store WHERE key LIKE ?")
        .bind(format!("deeplearn_run:%"))
        .fetch_all(&state.db)
        .await
    {
        Ok(rows) => {
            let mut runs = Vec::new();
            for r in rows {
                let _k: String = r.get(0);
                let v: Vec<u8> = r.get(1);
                if let Ok(j) = serde_json::from_slice::<serde_json::Value>(&v) {
                    runs.push(j);
                }
            }
            HttpResponse::Ok().json(json!({"success": true, "runs": runs}))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"success": false, "error": e.to_string()})),
    }
}

#[post("/api/learning/extended-run/{id}/cancel")]
pub async fn cancel_extended_run(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let run_id = path.into_inner();
    let run_key = format!("deeplearn_run:{}", run_id);
    match sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&run_key)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(row)) => {
            let mut meta: serde_json::Value = serde_json::from_slice(&row.get::<Vec<u8>, _>(0)).unwrap_or(serde_json::json!({}));
            meta["cancelled"] = serde_json::Value::Bool(true);
            meta["status"] = serde_json::Value::String("cancelling".to_string());
            let _ = sqlx::query("UPDATE jeebs_store SET value = ? WHERE key = ?")
                .bind(serde_json::to_vec(&meta).unwrap_or_default())
                .bind(&run_key)
                .execute(&state.db)
                .await;
            HttpResponse::Ok().json(json!({"success": true}))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({"success": false, "error": "not_found"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"success": false, "error": e.to_string()})),
    }
}

#[get("/api/learning/extended-run/{id}")]
pub async fn get_extended_run(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let run_id = path.into_inner();
    let run_key = format!("deeplearn_run:{}", run_id);
    match sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&run_key)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(row)) => {
            let v: Vec<u8> = row.get(0);
            if let Ok(j) = serde_json::from_slice::<serde_json::Value>(&v) {
                HttpResponse::Ok().json(json!({"success": true, "run": j}))
            } else {
                HttpResponse::InternalServerError().json(json!({"success": false, "error": "invalid_run_data"}))
            }
        }
        Ok(None) => HttpResponse::NotFound().json(json!({"success": false, "error": "not_found"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"success": false, "error": e.to_string()})),
    }
}

#[get("/api/learning/sessions")]
pub async fn get_learning_sessions(state: web::Data<AppState>) -> impl Responder {
    match crate::deep_learning::get_all_learning_sessions(&state.db).await {
        Ok(sessions) => HttpResponse::Ok().json(json!({
            "success": true,
            "sessions": sessions.iter().map(|s| {
                json!({
                    "id": s.id,
                    "topic": s.topic,
                    "status": s.status,
                    "depth_level": s.depth_level,
                    "facts_learned": s.learned_facts.len(),
                    "study_hours": s.study_hours,
                    "confidence": s.confidence,
                    "problems_added": s.practice_problems.len(),
                })
            }).collect::<Vec<_>>(),
            "total_sessions": sessions.len(),
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": e
        })),
    }
}

#[get("/api/learning/statistics")]
pub async fn get_learning_statistics(state: web::Data<AppState>) -> impl Responder {
    match crate::deep_learning::get_learning_stats(&state.db).await {
        Ok(stats) => HttpResponse::Ok().json(json!({
            "success": true,
            "statistics": stats
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": e
        })),
    }
}

#[get("/api/learning/summary")]
pub async fn get_learning_summary_endpoint(state: web::Data<AppState>) -> impl Responder {
    match crate::knowledge_integration::get_learning_summary(&state.db).await {
        Ok(summary) => HttpResponse::Ok().json(json!({
            "success": true,
            "summary": summary
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": e
        })),
    }
}

#[get("/api/learning/session/{id}")]
pub async fn get_learning_session_endpoint(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let session_id = path.into_inner();
    match crate::deep_learning::get_learning_session_by_id(&state.db, &session_id).await {
        Ok(Some(session)) => HttpResponse::Ok().json(json!({
            "success": true,
            "session": session
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "success": false,
            "error": "session_not_found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": e
        })),
    }
}

// ===== CORTEX STRUCT & IMPLEMENTATION =====

/// Intent categories for routing conversation logic
#[derive(Debug, Clone, PartialEq)]
enum Intent {
    Greeting,
    Farewell,
    Thanks,
    SelfIntro,        // "my name is …"
    AboutJeebs,       // "who are you", "what can you do"
    MemoryStore,       // "remember that …"
    MemoryRecall,      // "what do you know about me"
    MemoryLookup,      // "what is my …"
    CommStyle,         // "how am i communicating"
    FollowUp,          // "go on", "continue"
    PluginTime,
    PluginCalc,
    PluginHash,
    PluginBase64,
    PluginPassword,
    PluginSystem,
    PluginLogic,
    KnowledgeQuestion, // general question requiring knowledge lookup
    Conversation,      // default free-form chat
}

fn classify_intent(lower: &str) -> Intent {
    // Greetings
    if matches!(
        lower,
        "hi" | "hello" | "hey" | "yo" | "sup" | "howdy" | "hiya" | "good morning"
            | "good afternoon" | "good evening"
    ) || lower.starts_with("hi ")
        || lower.starts_with("hello ")
        || lower.starts_with("hey ")
    {
        return Intent::Greeting;
    }

    // Farewells
    if matches!(
        lower,
        "bye" | "goodbye" | "see you" | "later" | "see ya" | "cya" | "goodnight"
    ) || lower.starts_with("bye ")
        || lower.starts_with("goodbye ")
    {
        return Intent::Farewell;
    }

    // Thanks
    if lower.starts_with("thank")
        || lower == "thanks"
        || lower == "ty"
        || lower.contains("thank you")
    {
        return Intent::Thanks;
    }

    // Self intro
    if lower.starts_with("my name is ")
        || lower.starts_with("i am ")
        || lower.starts_with("i'm ")
    {
        if extract_name_from_intro(lower).is_some() {
            return Intent::SelfIntro;
        }
    }

    // About Jeebs
    if lower.contains("who are you")
        || lower.contains("what are you")
        || lower.contains("what can you do")
        || lower.contains("tell me about yourself")
        || lower == "help"
        || lower.contains("your name")
        || lower.contains("what is jeebs")
        || lower.contains("what's jeebs")
    {
        return Intent::AboutJeebs;
    }

    // Memory store
    if extract_learnable_fact(lower).is_some() {
        // Don't override other intents
        if !lower.contains('?') {
            return Intent::MemoryStore;
        }
    }

    // Memory overview
    if wants_personal_memory_overview(lower) {
        return Intent::MemoryRecall;
    }

    // Memory lookup
    if wants_personal_memory_lookup(lower) {
        return Intent::MemoryLookup;
    }

    // Communication style
    if wants_communication_reflection(lower) {
        return Intent::CommStyle;
    }

    // Follow up
    if is_follow_up_prompt(lower) {
        return Intent::FollowUp;
    }

    // Plugin: time
    if lower.contains("what time")
        || lower.contains("current time")
        || lower.contains("what date")
        || lower.contains("current date")
        || lower == "time"
        || lower == "date"
        || lower.contains("today's date")
    {
        return Intent::PluginTime;
    }

    // Plugin: calc
    if lower.starts_with("calc ")
        || lower.starts_with("calculate ")
        || lower.starts_with("compute ")
        || lower.starts_with("evaluate ")
        || lower.starts_with("solve ")
        || (lower.contains('+')
            || lower.contains('*')
            || lower.contains('/')
            || (lower.contains('-') && lower.chars().filter(|c| c.is_ascii_digit()).count() >= 2))
            && lower.chars().filter(|c| c.is_ascii_digit()).count() >= 2
    {
        return Intent::PluginCalc;
    }

    // Plugin: hash
    if lower.starts_with("hash ") || lower.starts_with("md5 ") || lower.starts_with("sha") {
        return Intent::PluginHash;
    }

    // Plugin: base64
    if lower.starts_with("base64 ")
        || lower.starts_with("encode ")
        || lower.starts_with("decode ")
    {
        return Intent::PluginBase64;
    }

    // Plugin: password
    if lower.contains("generate password")
        || lower.contains("random password")
        || lower.starts_with("password")
    {
        return Intent::PluginPassword;
    }

    // Plugin: system
    if lower.contains("system status")
        || lower.contains("system info")
        || lower.contains("server status")
        || lower.contains("uptime")
        || lower.contains("cpu usage")
        || lower.contains("memory usage")
    {
        return Intent::PluginSystem;
    }

    // Plugin: logic
    if lower.starts_with("if ")
        || lower.contains(" true ")
        || lower.contains(" false ")
        || lower.contains("boolean")
        || lower.starts_with("logic ")
    {
        return Intent::PluginLogic;
    }

    // Knowledge question
    if lower.contains('?')
        || lower.starts_with("what ")
        || lower.starts_with("who ")
        || lower.starts_with("where ")
        || lower.starts_with("when ")
        || lower.starts_with("why ")
        || lower.starts_with("how ")
        || lower.starts_with("explain ")
        || lower.starts_with("define ")
        || lower.starts_with("describe ")
        || lower.starts_with("tell me about ")
        || lower.starts_with("do you know ")
    {
        return Intent::KnowledgeQuestion;
    }

    Intent::Conversation
}

/// Extract math expression from user input
fn extract_math_expression(input: &str) -> String {
    let lower = input.to_lowercase();
    let stripped = lower
        .trim_start_matches("calc ")
        .trim_start_matches("calculate ")
        .trim_start_matches("compute ")
        .trim_start_matches("evaluate ")
        .trim_start_matches("solve ")
        .trim_start_matches("what is ")
        .trim_start_matches("what's ")
        .trim();

    // Remove trailing question marks and whitespace
    stripped.trim_end_matches('?').trim().to_string()
}

/// Main Cortex struct for AI thinking and responses
pub struct Cortex;

impl Cortex {
    /// Generate a response for a user's prompt with full context
    pub async fn think_for_user(
        prompt: &str,
        state: &web::Data<AppState>,
        user_id: &str,
        username: Option<&str>,
    ) -> String {
        if prompt.trim().is_empty() {
            return "Please provide a prompt so I can help you.".to_string();
        }

        let db = &state.db;
        let lower = prompt.trim().to_lowercase();
        let intent = classify_intent(&lower);

        // ── Phase 1: Load user context ──
        let owner = fact_owner_key(user_id, username);
        let history = load_conversation_history(db, user_id).await;
        let learned_facts = load_learned_facts(db, &owner).await;
        let prev_profile = load_communication_profile(db, &owner).await;
        let profile = analyze_communication_profile(prompt, &history, prev_profile.as_ref());
        let display_name = username
            .filter(|n| !n.is_empty())
            .map(|n| n.to_string())
            .or_else(|| {
                learned_facts
                    .iter()
                    .find(|f| f.canonical.contains("my name is"))
                    .map(|f| {
                        f.fact
                            .split_whitespace()
                            .last()
                            .unwrap_or("friend")
                            .to_string()
                    })
            });

        // ── Phase 2: Route by intent ──
        let response = match intent {
            Intent::Greeting => {
                let name_part = display_name
                    .as_deref()
                    .map(|n| format!(", {n}"))
                    .unwrap_or_default();
                let greetings = [
                    format!("Hey{name_part}! What can I help you with?"),
                    format!("Hello{name_part}! Ready when you are."),
                    format!("Hi{name_part}! What's on your mind?"),
                    format!("Good to see you{name_part}. How can I assist?"),
                ];
                let idx = (chrono::Local::now().timestamp() as usize) % greetings.len();
                greetings[idx].clone()
            }

            Intent::Farewell => {
                let name_part = display_name
                    .as_deref()
                    .map(|n| format!(", {n}"))
                    .unwrap_or_default();
                format!("Goodbye{name_part}! Feel free to come back anytime.")
            }

            Intent::Thanks => {
                "You're welcome! Let me know if there's anything else I can help with."
                    .to_string()
            }

            Intent::SelfIntro => {
                if let Some(name) = extract_name_from_intro(&lower) {
                    let fact_text = format!("my name is {}", name);
                    let _ = save_learned_fact(db, &owner, &fact_text).await;
                    format!(
                        "Nice to meet you, {name}! I'll remember that. What can I do for you?"
                    )
                } else {
                    "Nice to meet you! What can I help you with?".to_string()
                }
            }

            Intent::AboutJeebs => {
                let node_count = sqlx::query("SELECT COUNT(*) FROM brain_nodes")
                    .fetch_optional(db)
                    .await
                    .ok()
                    .flatten()
                    .map(|r| r.get::<i64, _>(0))
                    .unwrap_or(0);

                let fact_count = learned_facts.len();

                format!(
                    "I'm **JeebsAI** — an autonomous AI assistant built in Rust. Here's what I can do:\n\n\
                    🧠 **Knowledge** — I have {node_count} brain nodes and learn from every conversation.\n\
                    🔢 **Calculate** — Math expressions (e.g. `calc 2+2*3`)\n\
                    🕐 **Time** — Current date and time\n\
                    🔐 **Hash** — MD5, SHA-256, BLAKE3 (e.g. `hash hello`)\n\
                    🔑 **Passwords** — Secure password generation\n\
                    📝 **Memory** — I remember facts about you ({fact_count} stored)\n\
                    💻 **System** — Server status and metrics\n\
                    🔎 **Search** — `.google <query>` to learn from the web\n\
                    🧬 **Evolution** — I propose improvements to my own code\n\n\
                    Just ask me anything!"
                )
            }

            Intent::MemoryStore => {
                if let Some(fact) = extract_learnable_fact(prompt.trim()) {
                    match save_learned_fact(db, &owner, &fact).await {
                        Ok(_) => format!("✅ Got it, I'll remember that: *{fact}*"),
                        Err(_) => "I tried to save that but ran into a storage issue.".to_string(),
                    }
                } else {
                    "I wasn't sure what to remember from that. Can you rephrase?".to_string()
                }
            }

            Intent::MemoryRecall => {
                if learned_facts.is_empty() {
                    "I don't have any stored memories about you yet. Tell me things and I'll remember them!".to_string()
                } else {
                    let items: Vec<String> = learned_facts
                        .iter()
                        .take(10)
                        .enumerate()
                        .map(|(i, f)| format!("{}. {}", i + 1, f.fact))
                        .collect();
                    format!(
                        "Here's what I know about you ({} facts):\n\n{}",
                        learned_facts.len(),
                        items.join("\n")
                    )
                }
            }

            Intent::MemoryLookup => {
                let relevant = rank_relevant_facts(&learned_facts, prompt.trim(), 5);
                if relevant.is_empty() {
                    "I don't have any stored info matching that. You can tell me facts and I'll remember them.".to_string()
                } else {
                    let items: Vec<String> = relevant
                        .iter()
                        .take(5)
                        .map(|f| format!("• {}", f.fact))
                        .collect();
                    format!("Here's what I recall:\n\n{}", items.join("\n"))
                }
            }

            Intent::CommStyle => render_communication_reflection(&profile),

            Intent::FollowUp => {
                // Use last assistant turn for context
                if let Some(last) = last_turn_by_role(&history, "assistant") {
                    let topic_summary = recent_conversation_summary(&history)
                        .unwrap_or_else(|| "our conversation".to_string());

                    // Try knowledge retrieval on the topic
                    let kb_response = Self::knowledge_search(db, &topic_summary).await;
                    if !kb_response.is_empty() {
                        format!(
                            "Building on what we discussed: {}\n\nAdditionally, from my knowledge: {}",
                            truncate_chars(&last.content, 200),
                            kb_response
                        )
                    } else {
                        format!(
                            "Continuing from where we left off — I said: \"{}\"\n\nWould you like me to elaborate on a specific part?",
                            truncate_chars(&last.content, 300)
                        )
                    }
                } else {
                    "I don't have context from a previous message. What topic would you like to explore?"
                        .to_string()
                }
            }

            Intent::PluginTime => {
                let now = chrono::Local::now();
                format!(
                    "🕐 Current date and time: **{}**\n📅 Date: {}\n⏰ Time: {}",
                    now.format("%A, %B %e, %Y at %I:%M:%S %p %Z"),
                    now.format("%Y-%m-%d"),
                    now.format("%H:%M:%S")
                )
            }

            Intent::PluginCalc => {
                let expr = extract_math_expression(prompt.trim());
                match meval::eval_str(&expr) {
                    Ok(result) => {
                        if result == result.floor() && result.abs() < 1e15 {
                            format!("🔢 `{}` = **{}**", expr, result as i64)
                        } else {
                            format!("🔢 `{}` = **{:.6}**", expr, result)
                        }
                    }
                    Err(e) => format!(
                        "I couldn't evaluate `{}` — {}\n\nTry expressions like `calc 2+3*4` or `calc sqrt(144)`.",
                        expr, e
                    ),
                }
            }

            Intent::PluginHash => {
                let input_text = lower
                    .trim_start_matches("hash ")
                    .trim_start_matches("md5 ")
                    .trim_start_matches("sha256 ")
                    .trim_start_matches("sha1 ")
                    .trim_start_matches("blake3 ")
                    .trim();

                if input_text.is_empty() {
                    "Provide text to hash. Example: `hash hello world`".to_string()
                } else {
                    use sha2::Digest as Sha2Digest;

                    let md5_hash = {
                        let digest = md5::Md5::digest(input_text.as_bytes());
                        format!("{:x}", digest)
                    };
                    let sha256_hash = {
                        let digest = sha2::Sha256::digest(input_text.as_bytes());
                        format!("{:x}", digest)
                    };
                    let blake3_hash = blake3::hash(input_text.as_bytes()).to_hex().to_string();

                    format!(
                        "🔐 Hashes for `{}`:\n\n• **MD5**: `{}`\n• **SHA-256**: `{}`\n• **BLAKE3**: `{}`",
                        input_text, md5_hash, sha256_hash, blake3_hash
                    )
                }
            }

            Intent::PluginBase64 => {
                use base64::Engine;
                let trimmed = prompt.trim().to_lowercase();
                if trimmed.starts_with("decode ") || trimmed.starts_with("base64 decode ") {
                    let data = trimmed
                        .trim_start_matches("base64 decode ")
                        .trim_start_matches("decode ")
                        .trim();
                    match base64::engine::general_purpose::STANDARD.decode(data) {
                        Ok(bytes) => {
                            let decoded = String::from_utf8_lossy(&bytes);
                            format!("📦 Decoded: `{}`", decoded)
                        }
                        Err(e) => format!("Failed to decode base64: {}", e),
                    }
                } else {
                    let data = trimmed
                        .trim_start_matches("base64 encode ")
                        .trim_start_matches("base64 ")
                        .trim_start_matches("encode ")
                        .trim();
                    let encoded = base64::engine::general_purpose::STANDARD.encode(data.as_bytes());
                    format!("📦 Base64 encoded: `{}`", encoded)
                }
            }

            Intent::PluginPassword => {
                let mut rng = rand::thread_rng();
                let charset: &[u8] =
                    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+";
                let password: String = (0..20)
                    .map(|_| {
                        let idx = rand::Rng::gen_range(&mut rng, 0..charset.len());
                        charset[idx] as char
                    })
                    .collect();
                format!(
                    "🔑 Generated secure password (20 chars):\n\n`{}`\n\n*Store this somewhere safe — I won't remember it.*",
                    password
                )
            }

            Intent::PluginSystem => {
                let (total_mem, used_mem, cpu_count, avg_cpu) = {
                    let mut sys = state.sys.lock().unwrap();
                    sys.refresh_all();
                    let total_mem = sys.total_memory();
                    let used_mem = sys.used_memory();
                    let cpu_count = sys.cpus().len();
                    let avg_cpu = if cpu_count > 0 {
                        sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / cpu_count as f32
                    } else {
                        0.0
                    };
                    (total_mem, used_mem, cpu_count, avg_cpu)
                };

                let mem_pct = if total_mem > 0 {
                    (used_mem as f64 / total_mem as f64) * 100.0
                } else {
                    0.0
                };

                let node_count = sqlx::query("SELECT COUNT(*) FROM brain_nodes")
                    .fetch_optional(db)
                    .await
                    .ok()
                    .flatten()
                    .map(|r| r.get::<i64, _>(0))
                    .unwrap_or(0);

                format!(
                    "💻 **System Status**\n\n\
                    • **CPU**: {:.1}% avg across {} cores\n\
                    • **Memory**: {:.1}% used ({:.0} MB / {:.0} MB)\n\
                    • **Brain nodes**: {}\n\
                    • **Server**: JeebsAI v0.0.1 (Rust/Actix)",
                    avg_cpu,
                    cpu_count,
                    mem_pct,
                    used_mem as f64 / 1_048_576.0,
                    total_mem as f64 / 1_048_576.0,
                    node_count
                )
            }

            Intent::PluginLogic => {
                let expr = lower
                    .trim_start_matches("logic ")
                    .trim();
                match evalexpr::eval(expr) {
                    Ok(val) => format!("🧮 Logic result: `{}` → **{}**", expr, val),
                    Err(e) => format!(
                        "Could not evaluate logic expression `{}`: {}\n\nTry: `logic true && false` or `logic 5 > 3`",
                        expr, e
                    ),
                }
            }

            Intent::KnowledgeQuestion => {
                Self::answer_knowledge_question(db, prompt.trim(), &learned_facts, &history, &profile)
                    .await
            }

            Intent::Conversation => {
                Self::conversational_response(db, prompt.trim(), &learned_facts, &history, &profile)
                    .await
            }
        };

        // ── Phase 3: Persist context ──
        let now = chrono::Local::now().to_rfc3339();
        let mut new_history = history.clone();
        new_history.push(ConversationTurn {
            role: "user".to_string(),
            content: sanitize_turn_content(prompt),
            timestamp: now.clone(),
        });
        new_history.push(ConversationTurn {
            role: "assistant".to_string(),
            content: sanitize_turn_content(&response),
            timestamp: now,
        });
        let _ = save_conversation_history(db, user_id, &new_history).await;
        let _ = save_communication_profile(db, &owner, &profile).await;

        // Learn from conversation passively
        let _ = crate::language_learning::learn_from_input(db, prompt).await;

        response
    }

    /// Stateless think — creates ephemeral context for non-user scenarios
    pub async fn think(prompt: &str, state: &web::Data<AppState>) -> String {
        let ephemeral_id = format!("ephemeral:{}", blake3::hash(b"default").to_hex());
        Self::think_for_user(prompt, state, &ephemeral_id, None).await
    }

    /// Search knowledge base and synthesize a response
    async fn knowledge_search(db: &SqlitePool, query: &str) -> String {
        match crate::knowledge_retrieval::retrieve_knowledge(db, query, 5).await {
            Ok(result) if !result.items.is_empty() => {
                if let Some(ref answer) = result.synthesized_answer {
                    if !answer.is_empty() {
                        return answer.clone();
                    }
                }
                result
                    .items
                    .iter()
                    .take(3)
                    .map(|i| i.summary.clone())
                    .collect::<Vec<_>>()
                    .join(". ")
            }
            _ => String::new(),
        }
    }

    /// Answer a knowledge question using multi-source retrieval
    async fn answer_knowledge_question(
        db: &SqlitePool,
        prompt: &str,
        facts: &[LearnedFact],
        _history: &[ConversationTurn],
        profile: &CommunicationProfile,
    ) -> String {
        let timer = Instant::now();

        // 1. Check personal facts first
        let relevant_personal = rank_relevant_facts(facts, prompt, 3);

        // 2. Knowledge base retrieval
        let kb_result = crate::knowledge_retrieval::retrieve_knowledge(db, prompt, 8)
            .await
            .unwrap_or_else(|_| crate::knowledge_retrieval::RetrievalResult {
                items: Vec::new(),
                total_searched: 0,
                query_terms: Vec::new(),
                synthesized_answer: None,
            });

        // 3. Build response
        let mut parts = Vec::new();

        // Add personal context if relevant
        if !relevant_personal.is_empty() {
            let personal: Vec<String> = relevant_personal
                .iter()
                .take(2)
                .map(|f| f.fact.clone())
                .collect();
            parts.push(format!("From what I know about you: {}", personal.join("; ")));
        }

        // Add synthesized knowledge answer
        if let Some(ref answer) = kb_result.synthesized_answer {
            if !answer.is_empty() {
                parts.push(answer.clone());
            }
        } else if !kb_result.items.is_empty() {
            // Build answer from top items
            let top_summaries: Vec<String> = kb_result
                .items
                .iter()
                .take(3)
                .filter(|i| !i.summary.is_empty())
                .map(|i| {
                    if i.category == "knowledge_triple" {
                        i.content.clone()
                    } else {
                        i.summary.clone()
                    }
                })
                .collect();

            if !top_summaries.is_empty() {
                parts.push(top_summaries.join(". "));
            }
        }

        // Add connected insights
        if kb_result.items.len() >= 2 {
            let linked = crate::knowledge_integration::detect_topics_in_message(prompt);
            if !linked.is_empty() {
                let topics: Vec<String> = linked.iter().take(3).map(|(t, _)| t.clone()).collect();
                parts.push(format!("Related topics: {}", topics.join(", ")));
            }
        }

        if parts.is_empty() {
            // No knowledge found — provide a helpful fallback
            let topic_hint = prompt
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .take(3)
                .collect::<Vec<_>>()
                .join(" ");

            // Adapt tone based on communication profile
            match profile.style.as_str() {
                "frustrated" => format!(
                    "I don't have specific information on that yet, but I'm actively learning. \
                     You can teach me by saying: `remember that <fact>`, or use `.google {}` to have me research it.",
                    topic_hint
                ),
                "curious" => format!(
                    "Great question! I don't have that in my knowledge base yet. \
                     I can learn about it — try `.google {}` and I'll research and store the findings.",
                    topic_hint
                ),
                _ => format!(
                    "I don't have enough information to answer that confidently. \
                     You can help me learn by using `.google {}` or by telling me facts with `remember that ...`.",
                    topic_hint
                ),
            }
        } else {
            let elapsed = timer.elapsed().as_millis();
            let answer = parts.join("\n\n");

            // If response is very short, add context
            if answer.len() < 100 && kb_result.total_searched > 0 {
                format!(
                    "{}\n\n*Searched {} knowledge items in {}ms.*",
                    answer, kb_result.total_searched, elapsed
                )
            } else {
                answer
            }
        }
    }

    /// Generate a conversational response for non-question, non-command input
    async fn conversational_response(
        db: &SqlitePool,
        prompt: &str,
        _facts: &[LearnedFact],
        history: &[ConversationTurn],
        profile: &CommunicationProfile,
    ) -> String {
        // Check for teachable facts
        if let Some(fact) = extract_learnable_fact(prompt) {
            // This was missed by the intent classifier — store it
            let owner_key = "ephemeral:default";
            if save_learned_fact(db, owner_key, &fact).await.is_ok() {
                return format!("✅ Noted: *{fact}*. I'll keep that in mind.");
            }
        }

        // See if there's relevant knowledge to share
        let kb_response = Self::knowledge_search(db, prompt).await;
        if !kb_response.is_empty() && kb_response.len() > 20 {
            return kb_response;
        }

        // Context-aware conversational response
        let recent_topics = if !history.is_empty() {
            let topics = infer_recent_topics(history, 4);
            if !topics.is_empty() {
                format!(
                    " I notice we've been discussing: {}.",
                    topics.join(", ")
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Adapt response based on communication style
        match profile.style.as_str() {
            "frustrated" => format!(
                "I hear you. Let me know if there's something specific I can help with.{recent_topics}"
            ),
            "curious" => format!(
                "Interesting thought! Want me to look into that further? You can also try `.google` to have me research a topic.{recent_topics}"
            ),
            "direct" => format!(
                "Got it. What would you like me to do with that?{recent_topics}"
            ),
            "reflective" => format!(
                "That's a thoughtful perspective.{recent_topics} I can explore related knowledge if you'd like."
            ),
            _ => format!(
                "I see! Is there something specific you'd like me to help with or look up?{recent_topics}"
            ),
        }
    }
}

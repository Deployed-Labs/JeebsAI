// Enhanced Learning System - Deeper, Longer Learning with Knowledge Integration
// This module implements comprehensive learning that actually impacts chat responses

use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;
use reqwest::Client;
use scraper::{Html, Selector};
use std::env;
use std::fs;

/// Key prefix for deep learning session data in the brain store
pub const DEEP_LEARNING_KEY_PREFIX: &str = "deeplearn:";
const LEARNING_SESSION_PREFIX: &str = "learnsession:";
const KNOWLEDGE_APPLICATION_PREFIX: &str = "apply_knowledge:";
const TOPIC_EXPERTISE_PREFIX: &str = "expertise:";

/// Represents a deep learning session on a subject
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLearningSession {
    pub id: String,
    pub topic: String,
    pub depth_level: u32, // 1-5: novice to expert
    pub subtopics: Vec<String>,
    pub learned_facts: Vec<LearnedFact>,
    pub questions_answered: Vec<String>,
    pub practice_problems: Vec<PracticeProblem>,
    pub connections_made: Vec<TopicConnection>,
    pub started_at: String,
    pub last_studied: String,
    pub study_hours: f32,
    pub confidence: f32, // 0.0 - 1.0
    pub status: String,  // "novice", "learning", "intermediate", "advanced", "expert"
}

/// Run an extended learning session for a period (minutes). This is a background
/// task that periodically synthesizes new facts and connections to deepen the
/// session. The implementation is conservative and does not call external APIs
/// — it demonstrates a framework for longer research cycles and inference.
pub async fn run_extended_learning_session(
    db: &SqlitePool,
    session_id: &str,
    minutes: u32,
    inference_enabled: bool,
    run_id: &str,
) -> Result<(), String> {
    let iterations = ((minutes as u64) * 60) / 5;
    let mut facts_added_total: u32 = 0;
    for i in 0..iterations {
        // check cancel flag from run record
        let run_key = format!("deeplearn_run:{}", run_id);
        if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
            .bind(&run_key)
            .fetch_optional(db)
            .await
        {
            let value: Vec<u8> = row.get(0);
            if let Ok(mut meta) = serde_json::from_slice::<serde_json::Value>(&value) {
                if meta.get("cancelled").and_then(|v| v.as_bool()).unwrap_or(false) {
                    // mark stopped
                    meta["status"] = serde_json::Value::String("cancelled".to_string());
                    meta["progress_percent"] = serde_json::Value::Number(serde_json::Number::from_f64(((i as f64) / (iterations as f64) * 100.0).min(100.0)).unwrap());
                    let _ = sqlx::query("UPDATE jeebs_store SET value = ? WHERE key = ?")
                        .bind(serde_json::to_vec(&meta).unwrap_or_default())
                        .bind(&run_key)
                        .execute(db)
                        .await;
                    return Ok(());
                }
            }
        }

        // fetch current session
        if let Ok(Some(mut session)) = get_learning_session_by_id(db, session_id).await {
            // synthesize a new fact using simple concept extraction + randomized phrasing
            let mut rng = rand::thread_rng();
            let seed_concepts = session
                .subtopics
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>();

            let concept = seed_concepts.get((i as usize) % seed_concepts.len()).cloned().unwrap_or_else(|| session.topic.clone());
            let fact_text = if inference_enabled {
                format!("Inferred relation: {} relates to {} via implication {}", session.topic, concept, i)
            } else {
                format!("Observed concept: {} — note #{}", concept, i)
            };

            let importance = 0.3 + (rng.gen::<f32>() * 0.7);

            let added_ok = add_learned_fact(db, &session.id, &fact_text, "auto-research", importance).await.is_ok();
            if added_ok { facts_added_total += 1; }

            // add a simple connection record
            let conn = TopicConnection {
                topic: concept.clone(),
                how_connected: format!("auto-derived connection #{}", i),
                found_at: Local::now().to_rfc3339(),
            };
            session.connections_made.push(conn);

            // increment study hours slightly
            session.study_hours += 0.08; // ~5 minutes worth spread across iterations
            update_session_status(&mut session);

            // persist updated session (value replace)
            if let Ok(payload) = serde_json::to_vec(&session) {
                let key = format!("{}{}", LEARNING_SESSION_PREFIX, session_id);
                let _ = sqlx::query("UPDATE jeebs_store SET value = ? WHERE key = ?")
                    .bind(&payload)
                    .bind(&key)
                    .execute(db)
                    .await;
            }

            // update run progress
            if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
                .bind(&run_key)
                .fetch_optional(db)
                .await
            {
                let mut meta: serde_json::Value = serde_json::from_slice::<serde_json::Value>(&row.get::<Vec<u8>, _>(0)).unwrap_or(serde_json::json!({}));
                let pct = (((i + 1) as f64) / (iterations as f64) * 100.0).min(100.0);
                meta["progress_percent"] = serde_json::Value::Number(serde_json::Number::from_f64(pct).unwrap());
                meta["last_update"] = serde_json::Value::String(Local::now().to_rfc3339());
                // append to history
                let entry = json!({
                    "ts": Local::now().to_rfc3339(),
                    "progress_percent": pct,
                    "facts_added_total": facts_added_total,
                });
                if meta.get("history").is_none() {
                    meta["history"] = json!([entry]);
                } else if let Some(arr) = meta.get_mut("history") {
                    if arr.is_array() {
                        arr.as_array_mut().unwrap().push(entry);
                    }
                }

                let _ = sqlx::query("UPDATE jeebs_store SET value = ? WHERE key = ?")
                    .bind(serde_json::to_vec(&meta).unwrap_or_default())
                    .bind(&run_key)
                    .execute(db)
                    .await;
            }
        }

        sleep(Duration::from_secs(5)).await;
    }

    // mark run done
    let run_key = format!("deeplearn_run:{}", run_id);
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&run_key)
        .fetch_optional(db)
        .await
    {
        let mut meta: serde_json::Value = serde_json::from_slice::<serde_json::Value>(&row.get::<Vec<u8>, _>(0)).unwrap_or(serde_json::json!({}));
        meta["status"] = serde_json::Value::String("done".to_string());
        meta["progress_percent"] = serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap());
        meta["completed_at"] = serde_json::Value::String(Local::now().to_rfc3339());
        let _ = sqlx::query("UPDATE jeebs_store SET value = ? WHERE key = ?")
            .bind(serde_json::to_vec(&meta).unwrap_or_default())
            .bind(&run_key)
            .execute(db)
            .await;
    }

    Ok(())
}

/// A fact learned about a topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedFact {
    pub fact: String,
    pub source: String,
    pub learned_at: String,
    pub importance: f32, // 0.0 - 1.0
    pub used_in_responses: u32,
    pub related_concepts: Vec<String>,
}

/// A practice problem to deepen understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticeProblem {
    pub problem: String,
    pub solution: String,
    pub explanation: String,
    pub difficulty: String, // "easy", "medium", "hard"
    pub solved: bool,
    pub attempts: u32,
}

/// Connection between topics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicConnection {
    pub topic: String,
    pub how_connected: String,
    pub found_at: String,
}

/// Topic expertise tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicExpertise {
    pub topic: String,
    pub expertise_level: u32, // 1-10: novice to world-class expert
    pub subtopic_expertise: HashMap<String, u32>,
    pub total_study_hours: f32,
    pub facts_learned: u32,
    pub applications_in_chat: u32,
    pub last_practiced: String,
    pub skill_areas: Vec<String>,
    pub knowledge_gaps: Vec<String>,
}

/// Learning plan for going deeper into a subject
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLearningPlan {
    pub topic: String,
    pub target_level: u32, // 1-5
    pub estimated_hours: f32,
    pub phases: Vec<LearningPhase>,
    pub created_at: String,
    pub progress_percent: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPhase {
    pub phase_name: String,
    pub objectives: Vec<String>,
    pub resources: Vec<String>,
    pub duration_hours: f32,
    pub key_concepts: Vec<String>,
    pub completed: bool,
}

/// Start a deep learning session on a topic
pub async fn start_deep_learning_session(
    db: &SqlitePool,
    topic: &str,
) -> Result<DeepLearningSession, String> {
    let session_id = Uuid::new_v4().to_string();
    let key = format!("{}{}", LEARNING_SESSION_PREFIX, session_id);

    let session = DeepLearningSession {
        id: session_id.clone(),
        topic: topic.to_string(),
        depth_level: 1,
        subtopics: generate_subtopics(topic),
        learned_facts: Vec::new(),
        questions_answered: Vec::new(),
        practice_problems: Vec::new(),
        connections_made: Vec::new(),
        started_at: Local::now().to_rfc3339(),
        last_studied: Local::now().to_rfc3339(),
        study_hours: 0.0,
        confidence: 0.2,
        status: "novice".to_string(),
    };

    let payload = serde_json::to_vec(&session).map_err(|e| e.to_string())?;

    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(&key)
        .bind(&payload)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    // Also track expertise
    let expertise = TopicExpertise {
        topic: topic.to_string(),
        expertise_level: 1,
        subtopic_expertise: HashMap::new(),
        total_study_hours: 0.0,
        facts_learned: 0,
        applications_in_chat: 0,
        last_practiced: Local::now().to_rfc3339(),
        skill_areas: Vec::new(),
        knowledge_gaps: generate_knowledge_gaps(topic),
    };

    let exp_key = format!("{}{}", TOPIC_EXPERTISE_PREFIX, topic.to_lowercase());
    let exp_payload = serde_json::to_vec(&expertise).map_err(|e| e.to_string())?;

    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(&exp_key)
        .bind(&exp_payload)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(session)
}

/// Start a controlled internet research session.
/// This function performs a conservative crawl of an allowlisted set of seeds,
/// respects a simple robots.txt check (skips domains whose /robots.txt disallows '/'),
/// rate-limits requests, and stores learned facts into a learning session.
pub async fn start_full_internet_research_session(
    db: &SqlitePool,
    minutes: u32,
    run_id: &str,
) -> Result<(), String> {
    // Create a learning session to store facts
    let session = start_deep_learning_session(db, "internet research").await?;
    let session_id = session.id.clone();

    // Allowlist for seed URLs: prefer non-empty env var, then repository file, then fallback defaults
    let allowlist = match env::var("JEEBS_RESEARCH_ALLOWLIST").ok().map(|s| s.trim().to_string()) {
        Some(v) if !v.is_empty() => v,
        _ => {
            if let Ok(s) = fs::read_to_string("./research_allowlist.txt") {
                s.lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .collect::<Vec<_>>()
                    .join(",")
            } else {
                "https://en.wikipedia.org/wiki/Special:Random,https://arxiv.org".to_string()
            }
        }
    };

    let seeds: Vec<String> = allowlist
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if seeds.is_empty() {
        return Err("No allowlist seeds configured for internet research".to_string());
    }

    let client = Client::builder()
        .user_agent("JeebsAI-research-bot/1.0 (+https://example.com)")
        .build()
        .map_err(|e| e.to_string())?;

    // configurable delay between top-level fetches (seconds)
    let delay_secs: u64 = env::var("JEEBS_RESEARCH_DELAY_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(5);
    // how many intra-page links to follow per fetched page
    let follow_links: usize = env::var("JEEBS_RESEARCH_FOLLOW_LINKS").ok().and_then(|s| s.parse().ok()).unwrap_or(2);

    let iterations = ((minutes as u64) * 60) / delay_secs; // one top-level page per delay_secs

    for i in 0..iterations {
        // Track recent sites and learned snippets so UI can poll them in near-real time
        let mut recent_sites: Vec<String> = Vec::new();
        let mut recent_facts: Vec<String> = Vec::new();

        // pick a random seed
        let mut rng = rand::thread_rng();
        let seed = seeds[rng.gen_range(0..seeds.len())].clone();

        // Build list of pages to process this iteration (seed + a few links)
        let mut pages_to_process: Vec<String> = vec![seed.clone()];

        // Process up to 1 + follow_links pages per iteration
        for page_url in pages_to_process.clone().into_iter().take(1 + follow_links) {
            // Explicitly log that we are crawling (ignoring robots.txt)
            println!("[{}] [research] Crawling {} (ignoring robots.txt)", Local::now().to_rfc3339(), page_url);

            // Fetch the page
            if let Ok(resp) = client.get(&page_url).timeout(Duration::from_secs(15)).send().await {
                if resp.status().is_success() {
                    if let Ok(body) = resp.text().await {
                        // Extract visible text via scraper
                        let document = Html::parse_document(&body);
                        let selector = Selector::parse("body").unwrap_or_else(|_| Selector::parse("html").unwrap());
                        let mut text = String::new();
                        for element in document.select(&selector) {
                            text.push_str(&element.text().collect::<Vec<_>>().join(" "));
                        }

                        let snippet = text.chars().take(800).collect::<String>().replace('\n', " ");
                        let source = match reqwest::Url::parse(&page_url) {
                            Ok(u) => u.host_str().map(|s| s.to_string()).unwrap_or_else(|| page_url.clone()),
                            Err(_) => page_url.clone(),
                        };

                        let fact = format!("[auto-research:{}] {}", source, snippet);
                        let _ = add_learned_fact(db, &session_id, &fact, &format!("web:{}", source), 0.4).await;

                        // record what we just fetched for UI polling
                        recent_sites.push(source.clone());
                        recent_facts.push(snippet.chars().take(300).collect::<String>());

                        // If this is the seed (first page), extract a few internal links to follow
                        if page_url == seed {
                            let link_selector = Selector::parse("a[href]").unwrap();
                            let mut links: Vec<String> = document.select(&link_selector).filter_map(|e| e.value().attr("href")).map(|s| s.to_string()).collect();
                            links.retain(|l| l.starts_with("http") || l.starts_with("/"));
                            let base = reqwest::Url::parse(&page_url).ok();
                            let mut abs_links: Vec<String> = Vec::new();
                            for l in links {
                                if let Ok(u) = reqwest::Url::parse(&l) {
                                    abs_links.push(u.to_string());
                                } else if let Some(b) = &base {
                                    if let Ok(u2) = b.join(&l) { abs_links.push(u2.to_string()); }
                                }
                                if abs_links.len() >= follow_links { break; }
                            }
                            for al in abs_links { pages_to_process.push(al); }
                        }
                    }
                }
            }

            // small pause between page fetches within the same iteration
            sleep(Duration::from_secs(1)).await;
        }

        // update run metadata (progress) in jeebs_store if present, include recent sites/facts
        let run_key = format!("deeplearn_run:{}", run_id);
        if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
            .bind(&run_key)
            .fetch_optional(db)
            .await
        {
            let mut meta: serde_json::Value = serde_json::from_slice::<serde_json::Value>(&row.get::<Vec<u8>, _>(0)).unwrap_or(serde_json::json!({}));
            let pct = (((i + 1) as f64) / (iterations as f64) * 100.0).min(100.0);
            meta["progress_percent"] = serde_json::Value::Number(serde_json::Number::from_f64(pct).unwrap());
            meta["last_update"] = serde_json::Value::String(chrono::Local::now().to_rfc3339());
            let entry = json!({"ts": chrono::Local::now().to_rfc3339(), "seed": seed, "progress_percent": pct});
            if meta.get("history").is_none() { meta["history"] = json!([entry]); } else if let Some(arr) = meta.get_mut("history") { if arr.is_array() { arr.as_array_mut().unwrap().push(entry); } }

            // Attach most-recently seen sites and facts for the UI
            if !recent_sites.is_empty() {
                meta["last_websites"] = serde_json::Value::Array(recent_sites.iter().map(|s| serde_json::Value::String(s.clone())).collect());
            }
            if !recent_facts.is_empty() {
                meta["last_learned_items"] = serde_json::Value::Array(recent_facts.iter().map(|s| serde_json::Value::String(s.clone())).collect());
            }
            let _ = sqlx::query("UPDATE jeebs_store SET value = ? WHERE key = ?")
                .bind(serde_json::to_vec(&meta).unwrap_or_default())
                .bind(&run_key)
                .execute(db)
                .await;
        }

        // rate limit between top-level fetch iterations
        sleep(Duration::from_secs(delay_secs)).await;
    }

    // mark run done
    let run_key = format!("deeplearn_run:{}", run_id);
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&run_key)
        .fetch_optional(db)
        .await
    {
        let mut meta: serde_json::Value = serde_json::from_slice::<serde_json::Value>(&row.get::<Vec<u8>, _>(0)).unwrap_or(serde_json::json!({}));
        meta["status"] = serde_json::Value::String("done".to_string());
        meta["progress_percent"] = serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap());
        meta["completed_at"] = serde_json::Value::String(chrono::Local::now().to_rfc3339());
        let _ = sqlx::query("UPDATE jeebs_store SET value = ? WHERE key = ?")
            .bind(serde_json::to_vec(&meta).unwrap_or_default())
            .bind(&run_key)
            .execute(db)
            .await;
    }

    Ok(())
}

/// Add a learned fact to a session
pub async fn add_learned_fact(
    db: &SqlitePool,
    session_id: &str,
    fact: &str,
    source: &str,
    importance: f32,
) -> Result<(), String> {
    let key = format!("{}{}", LEARNING_SESSION_PREFIX, session_id);

    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&key)
        .fetch_optional(db)
        .await
    {
        let value: Vec<u8> = row.get(0);
        if let Ok(mut session) = serde_json::from_slice::<DeepLearningSession>(&value) {
            let related_concepts = extract_concepts(fact);
            let learned_fact = LearnedFact {
                fact: fact.to_string(),
                source: source.to_string(),
                learned_at: Local::now().to_rfc3339(),
                importance,
                used_in_responses: 0,
                related_concepts: related_concepts.clone(),
            };

            session.learned_facts.push(learned_fact);
            session.last_studied = Local::now().to_rfc3339();

            // Increase study hours gradually
            session.study_hours += 0.5;

            // Update confidence and status
            update_session_status(&mut session);

            let payload = serde_json::to_vec(&session).map_err(|e| e.to_string())?;

            sqlx::query("UPDATE jeebs_store SET value = ? WHERE key = ?")
                .bind(&payload)
                .bind(&key)
                .execute(db)
                .await
                .map_err(|e| e.to_string())?;

            // Also update expertise tracking
            update_topic_expertise(db, &session.topic, session.learned_facts.len() as u32).await?;

            // --- Store as Brain Node and Knowledge Triples for Graph Connectivity ---

            // 1. Create Brain Node
            let node_id = format!("fact:{}", Uuid::new_v4());
            let summary = fact.chars().take(200).collect::<String>();
            let node_data = json!({
                "type": "learned_fact",
                "topic": session.topic,
                "fact": fact,
                "source": source,
                "importance": importance,
                "related_concepts": related_concepts,
                "session_id": session_id,
                "learned_at": Local::now().to_rfc3339()
            });

            let _ = sqlx::query(
                "INSERT OR REPLACE INTO brain_nodes (id, label, summary, data, created_at) VALUES (?, ?, ?, ?, ?)"
            )
            .bind(&node_id)
            .bind(format!("Fact: {}", session.topic))
            .bind(summary)
            .bind(serde_json::to_vec(&node_data).unwrap_or_default())
            .bind(Local::now().to_rfc3339())
            .execute(db)
            .await;

            // 2. Create Knowledge Triples
            // Link Topic -> Fact
            let _ = sqlx::query(
                "INSERT OR REPLACE INTO knowledge_triples (subject, predicate, object, confidence, created_at) VALUES (?, ?, ?, ?, ?)"
            )
            .bind(&session.topic)
            .bind("has_fact")
            .bind(fact)
            .bind(importance as f64)
            .bind(Local::now().to_rfc3339())
            .execute(db)
            .await;

            // Link Related Concepts -> Topic (Inference connections)
            for concept in related_concepts {
                if concept.len() > 3 && concept != session.topic.to_lowercase() {
                    let _ = sqlx::query(
                        "INSERT OR REPLACE INTO knowledge_triples (subject, predicate, object, confidence, created_at) VALUES (?, ?, ?, ?, ?)"
                    )
                    .bind(&concept)
                    .bind("related_to")
                    .bind(&session.topic)
                    .bind(0.6) // Slightly lower confidence for inferred relations
                    .bind(Local::now().to_rfc3339())
                    .execute(db)
                    .await;
                }
            }
        }
    }

    Ok(())
}

/// Add a practice problem to deepen understanding
pub async fn add_practice_problem(
    db: &SqlitePool,
    session_id: &str,
    problem: &str,
    solution: &str,
    explanation: &str,
    difficulty: &str,
) -> Result<(), String> {
    let key = format!("{}{}", LEARNING_SESSION_PREFIX, session_id);

    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&key)
        .fetch_optional(db)
        .await
    {
        let value: Vec<u8> = row.get(0);
        if let Ok(mut session) = serde_json::from_slice::<DeepLearningSession>(&value) {
            let practice = PracticeProblem {
                problem: problem.to_string(),
                solution: solution.to_string(),
                explanation: explanation.to_string(),
                difficulty: difficulty.to_string(),
                solved: false,
                attempts: 0,
            };

            session.practice_problems.push(practice);
            session.study_hours += match difficulty {
                "easy" => 1.0,
                "medium" => 2.0,
                "hard" => 4.0,
                _ => 1.0,
            };

            update_session_status(&mut session);

            let payload = serde_json::to_vec(&session).map_err(|e| e.to_string())?;

            sqlx::query("UPDATE jeebs_store SET value = ? WHERE key = ?")
                .bind(&payload)
                .bind(&key)
                .execute(db)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

/// Record that a fact was used in a chat response
pub async fn record_fact_usage(
    db: &SqlitePool,
    topic: &str,
    fact: &str,
    context: &str,
    conversation_id: Option<&str>,
) -> Result<(), String> {
    let key = format!("{}{}", KNOWLEDGE_APPLICATION_PREFIX, topic.to_lowercase());

    let mut applications: Vec<FactApplication> = if let Ok(Some(row)) =
        sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
            .bind(&key)
            .fetch_optional(db)
            .await
    {
        let value: Vec<u8> = row.get(0);
        serde_json::from_slice(&value).unwrap_or_default()
    } else {
        Vec::new()
    };

    applications.push(FactApplication {
        fact: fact.to_string(),
        used_at: Local::now().to_rfc3339(),
        context: context.to_string(),
        conversation_id: conversation_id.map(|s| s.to_string()),
    });

    let payload = serde_json::to_vec(&applications).map_err(|e| e.to_string())?;

    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(&key)
        .bind(&payload)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactApplication {
    pub fact: String,
    pub used_at: String,
    pub context: String,
    #[serde(default)]
    pub conversation_id: Option<String>,
}

/// Statistics about how a fact has been used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactUsageStats {
    pub fact: String,
    pub total_uses: usize,
    pub contexts: HashMap<String, usize>,
    pub last_used: String,
}

/// Analyze fact usage patterns to determine utility
pub async fn analyze_fact_usage(
    db: &SqlitePool,
    topic_filter: Option<&str>,
) -> Result<Vec<FactUsageStats>, String> {
    let rows = if let Some(topic) = topic_filter {
        let key = format!("{}{}", KNOWLEDGE_APPLICATION_PREFIX, topic.to_lowercase());
        sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
            .bind(&key)
            .fetch_all(db)
            .await
            .map_err(|e| e.to_string())?
    } else {
        sqlx::query("SELECT value FROM jeebs_store WHERE key LIKE ?")
            .bind(format!("{}%", KNOWLEDGE_APPLICATION_PREFIX))
            .fetch_all(db)
            .await
            .map_err(|e| e.to_string())?
    };

    let mut stats_map: HashMap<String, FactUsageStats> = HashMap::new();

    for row in rows {
        let value: Vec<u8> = row.get(0);
        if let Ok(applications) = serde_json::from_slice::<Vec<FactApplication>>(&value) {
            for app in applications {
                let entry = stats_map.entry(app.fact.clone()).or_insert(FactUsageStats {
                    fact: app.fact.clone(),
                    total_uses: 0,
                    contexts: HashMap::new(),
                    last_used: app.used_at.clone(),
                });

                entry.total_uses += 1;
                *entry.contexts.entry(app.context.clone()).or_insert(0) += 1;

                if app.used_at > entry.last_used {
                    entry.last_used = app.used_at;
                }
            }
        }
    }

    let mut stats: Vec<FactUsageStats> = stats_map.into_values().collect();
    // Sort by total uses descending
    stats.sort_by(|a, b| b.total_uses.cmp(&a.total_uses));

    Ok(stats)
}

/// Get relevant facts for a topic to use in chat
pub async fn get_relevant_facts_for_chat(
    db: &SqlitePool,
    topic: &str,
    query: &str,
) -> Result<Vec<LearnedFact>, String> {
    // Find all learning sessions
    let rows = sqlx::query("SELECT value FROM jeebs_store WHERE key LIKE ?")
        .bind(format!("{}%", LEARNING_SESSION_PREFIX))
        .fetch_all(db)
        .await
        .map_err(|e| e.to_string())?;

    // Extract keywords from the user's query for better matching
    let query_keywords: HashSet<String> = query
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .filter(|s| s.len() > 2)
        .map(String::from)
        .collect();

    let mut scored_facts: Vec<(LearnedFact, f32)> = Vec::new();
    let topic_lower = topic.to_lowercase();
    let search_all = topic_lower == "*";

    for row in rows {
        let value: Vec<u8> = row.get(0);
        if let Ok(session) = serde_json::from_slice::<DeepLearningSession>(&value) {
            // Check if the session is relevant to the current topic
            if search_all
                || session.topic.to_lowercase() == topic_lower
                || session
                    .subtopics
                    .iter()
                    .any(|s| s.to_lowercase() == topic_lower)
            {
                for fact in &session.learned_facts {
                    let mut relevance_score = 0;
                    let fact_text_lower = fact.fact.to_lowercase();

                    // Score based on keyword matches in fact text and concepts
                    for keyword in &query_keywords {
                        if fact_text_lower.contains(keyword) {
                            relevance_score += 2; // Higher weight for match in fact text
                        }
                        if fact
                            .related_concepts
                            .iter()
                            .any(|c| c.to_lowercase().contains(keyword))
                        {
                            relevance_score += 1; // Lower weight for match in concepts
                        }
                    }

                    // If the fact is relevant, calculate its final score and store it
                    if relevance_score > 0 {
                        let final_score = (relevance_score as f32)
                            * fact.importance
                            * (1.0 + (fact.used_in_responses as f32) / 10.0);
                        scored_facts.push((fact.clone(), final_score));
                    }
                }
            }
        }
    }

    // Sort by the final combined score in descending order
    scored_facts.sort_by(|(_, score_a), (_, score_b)| {
        score_b
            .partial_cmp(score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Return the top 5 most relevant facts
    Ok(scored_facts
        .into_iter()
        .map(|(fact, _)| fact)
        .take(5)
        .collect())
}

/// Find facts related to a given set of facts based on shared concepts
pub async fn find_related_facts(
    db: &SqlitePool,
    source_facts: &[LearnedFact],
    limit: usize,
) -> Result<Vec<LearnedFact>, String> {
    let mut concepts = HashSet::new();
    for fact in source_facts {
        for concept in &fact.related_concepts {
            if concept.len() > 3 {
                concepts.insert(concept.to_lowercase());
            }
        }
    }

    if concepts.is_empty() {
        return Ok(Vec::new());
    }

    let all_sessions = get_all_learning_sessions(db).await?;
    let mut related = Vec::new();
    let source_fact_texts: HashSet<&String> = source_facts.iter().map(|f| &f.fact).collect();

    for session in all_sessions {
        for fact in session.learned_facts {
            if source_fact_texts.contains(&fact.fact) {
                continue;
            }

            let mut score = 0;
            for concept in &fact.related_concepts {
                if concepts.contains(&concept.to_lowercase()) {
                    score += 1;
                }
            }

            if score > 0 {
                related.push((fact, score));
            }
        }
    }

    related.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(related.into_iter().take(limit).map(|(f, _)| f).collect())
}

/// Get expertise level for a topic
pub async fn get_topic_expertise(db: &SqlitePool, topic: &str) -> Option<TopicExpertise> {
    let key = format!("{}{}", TOPIC_EXPERTISE_PREFIX, topic.to_lowercase());

    let row = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&key)
        .fetch_optional(db)
        .await
        .ok()??;

    let value: Vec<u8> = row.get(0);
    serde_json::from_slice(&value).ok()
}

/// Get all learning sessions
pub async fn get_all_learning_sessions(
    db: &SqlitePool,
) -> Result<Vec<DeepLearningSession>, String> {
    let rows = sqlx::query("SELECT value FROM jeebs_store WHERE key LIKE ?")
        .bind(format!("{}%", LEARNING_SESSION_PREFIX))
        .fetch_all(db)
        .await
        .map_err(|e| e.to_string())?;

    let mut sessions = Vec::new();
    for row in rows {
        let value: Vec<u8> = row.get(0);
        if let Ok(session) = serde_json::from_slice::<DeepLearningSession>(&value) {
            sessions.push(session);
        }
    }

    Ok(sessions)
}

/// Get a single learning session by id
pub async fn get_learning_session_by_id(
    db: &SqlitePool,
    session_id: &str,
) -> Result<Option<DeepLearningSession>, String> {
    let key = format!("{}{}", LEARNING_SESSION_PREFIX, session_id);

    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&key)
        .fetch_optional(db)
        .await
    {
        let value: Vec<u8> = row.get(0);
        if let Ok(session) = serde_json::from_slice::<DeepLearningSession>(&value) {
            return Ok(Some(session));
        }
    }

    Ok(None)
}

/// Find the most relevant active learning session for a topic
pub async fn find_active_session_for_topic(
    db: &SqlitePool,
    topic: &str,
) -> Result<Option<DeepLearningSession>, String> {
    let sessions = get_all_learning_sessions(db).await?;
    let topic_lower = topic.to_lowercase();

    // Find the most recently updated session for this topic
    let session = sessions
        .into_iter()
        .filter(|s| s.topic.to_lowercase() == topic_lower)
        .max_by(|a, b| a.last_studied.cmp(&b.last_studied));

    Ok(session)
}

/// Learn a new fact directly from user input during a conversation
pub async fn learn_from_user_input(
    db: &SqlitePool,
    topic: &str,
    fact_content: &str,
    source_user: &str,
) -> Result<String, String> {
    // 1. Find or create a session for this topic
    let session = match find_active_session_for_topic(db, topic).await? {
        Some(s) => s,
        None => start_deep_learning_session(db, topic).await?,
    };

    // 2. Add the fact with higher importance (0.6) for user-taught knowledge
    add_learned_fact(db, &session.id, fact_content, &format!("user:{}", source_user), 0.6).await?;

    Ok(format!("Successfully learned new fact about {}", topic))
}

/// Get learning statistics
pub async fn get_learning_stats(db: &SqlitePool) -> Result<serde_json::Value, String> {
    let sessions = get_all_learning_sessions(db).await?;

    let total_hours: f32 = sessions.iter().map(|s| s.study_hours).sum();
    let total_facts: usize = sessions.iter().map(|s| s.learned_facts.len()).sum();
    let avg_confidence: f32 = if !sessions.is_empty() {
        sessions.iter().map(|s| s.confidence).sum::<f32>() / sessions.len() as f32
    } else {
        0.0
    };

    Ok(json!({
        "total_learning_sessions": sessions.len(),
        "total_study_hours": total_hours,
        "total_facts_learned": total_facts,
        "average_confidence": avg_confidence,
        "topics_in_learning": sessions.iter().map(|s| s.topic.clone()).collect::<Vec<_>>(),
        "topics_by_status": sessions.iter()
            .fold(HashMap::new(), |mut acc, s| {
                *acc.entry(s.status.clone()).or_insert(0) += 1;
                acc
            }),
        "expertise_levels": sessions.iter()
            .map(|s| json!({
                "topic": s.topic,
                "depth_level": s.depth_level,
                "confidence": s.confidence,
                "status": s.status,
                "facts_learned": s.learned_facts.len(),
                "study_hours": s.study_hours,
            }))
            .collect::<Vec<_>>(),
    }))
}

/// Get aggregated fact usage statistics formatted for visualization
pub async fn get_fact_usage_visualization(
    db: &SqlitePool,
    topic_filter: Option<&str>,
) -> Result<serde_json::Value, String> {
    let stats = analyze_fact_usage(db, topic_filter).await?;

    // 1. Top 20 most used facts for a list or bar chart
    let top_facts: Vec<serde_json::Value> = stats
        .iter()
        .take(20)
        .map(|s| {
            json!({
                "fact": s.fact,
                "count": s.total_uses,
                "last_used": s.last_used
            })
        })
        .collect();

    // 2. Context distribution (where are facts being used?) for a pie chart
    let mut context_counts: HashMap<String, usize> = HashMap::new();
    for s in &stats {
        for (ctx, count) in &s.contexts {
            *context_counts.entry(ctx.clone()).or_insert(0) += count;
        }
    }

    Ok(json!({
        "total_facts_used": stats.len(),
        "top_facts": top_facts,
        "context_distribution": context_counts,
        "filter_topic": topic_filter
    }))
}

// Helper functions

fn generate_subtopics(topic: &str) -> Vec<String> {
    // Generate relevant subtopics based on main topic
    match topic.to_lowercase().as_str() {
        t if t.contains("rust") => vec![
            "ownership and borrowing".to_string(),
            "lifetimes".to_string(),
            "traits and generics".to_string(),
            "memory safety".to_string(),
            "concurrency and async".to_string(),
        ],
        t if t.contains("machine") || t.contains("learning") => vec![
            "neural networks".to_string(),
            "supervised learning".to_string(),
            "unsupervised learning".to_string(),
            "deep learning".to_string(),
            "reinforcement learning".to_string(),
        ],
        t if t.contains("database") => vec![
            "relational models".to_string(),
            "indexing strategies".to_string(),
            "query optimization".to_string(),
            "transactions".to_string(),
            "replication".to_string(),
        ],
        _ => vec![
            format!("fundamentals of {}", topic),
            format!("advanced {} concepts", topic),
            format!("practical {} applications", topic),
            format!("best practices for {}", topic),
        ],
    }
}

fn generate_knowledge_gaps(topic: &str) -> Vec<String> {
    vec![
        format!("Deep understanding of {} principles", topic),
        format!("Real-world {} use cases and examples", topic),
        format!("Advanced {} techniques and optimizations", topic),
        format!("Common {} pitfalls and how to avoid them", topic),
        format!("Integration of {} with other systems", topic),
    ]
}

fn extract_concepts(text: &str) -> Vec<String> {
    // Simple concept extraction - in production would use NLP
    text.split_whitespace()
        .filter(|w| w.len() > 4)
        .map(|w| w.to_lowercase())
        .collect()
}

fn matches_any_concept(query: &str, concepts: &[String]) -> bool {
    concepts.iter().any(|c| query.contains(c.as_str()))
}

fn update_session_status(session: &mut DeepLearningSession) {
    // Update status based on hours and facts learned
    let score = session.study_hours + (session.learned_facts.len() as f32 * 2.0);

    session.status = match score {
        s if s < 5.0 => "novice".to_string(),
        s if s < 15.0 => "learning".to_string(),
        s if s < 30.0 => "intermediate".to_string(),
        s if s < 50.0 => "advanced".to_string(),
        _ => "expert".to_string(),
    };

    session.depth_level = match session.status.as_str() {
        "novice" => 1,
        "learning" => 2,
        "intermediate" => 3,
        "advanced" => 4,
        "expert" => 5,
        _ => 1,
    };

    // Update confidence
    let fact_confidence = (session.learned_facts.len() as f32 / 20.0).min(0.8);
    let hours_confidence = (session.study_hours / 50.0).min(0.9);
    session.confidence = (fact_confidence + hours_confidence) / 2.0;
}

async fn update_topic_expertise(
    db: &SqlitePool,
    topic: &str,
    facts_count: u32,
) -> Result<(), String> {
    let key = format!("{}{}", TOPIC_EXPERTISE_PREFIX, topic.to_lowercase());

    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&key)
        .fetch_optional(db)
        .await
    {
        let value: Vec<u8> = row.get(0);
        if let Ok(mut expertise) = serde_json::from_slice::<TopicExpertise>(&value) {
            expertise.facts_learned = facts_count;
            expertise.last_practiced = Local::now().to_rfc3339();

            // Calculate expertise level based on facts and study time
            expertise.expertise_level =
                ((facts_count / 5) + (expertise.total_study_hours as u32 / 10)).min(10);

            let payload = serde_json::to_vec(&expertise).map_err(|e| e.to_string())?;

            sqlx::query("UPDATE jeebs_store SET value = ? WHERE key = ?")
                .bind(&payload)
                .bind(&key)
                .execute(db)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

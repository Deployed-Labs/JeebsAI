// Enhanced Learning System - Deeper, Longer Learning with Knowledge Integration
// This module implements comprehensive learning that actually impacts chat responses

use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use uuid::Uuid;

const DEEP_LEARNING_KEY_PREFIX: &str = "deeplearn:";
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
    pub status: String, // "novice", "learning", "intermediate", "advanced", "expert"
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
            let learned_fact = LearnedFact {
                fact: fact.to_string(),
                source: source.to_string(),
                learned_at: Local::now().to_rfc3339(),
                importance,
                used_in_responses: 0,
                related_concepts: extract_concepts(fact),
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
        context: "chat_response".to_string(),
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
}

/// Get relevant facts for a topic to use in chat
pub async fn get_relevant_facts_for_chat(
    db: &SqlitePool,
    topic: &str,
    query: &str,
) -> Result<Vec<LearnedFact>, String> {
    // Find all learning sessions for this topic
    let rows = sqlx::query("SELECT value FROM jeebs_store WHERE key LIKE ?")
        .bind(format!("{}%", LEARNING_SESSION_PREFIX))
        .fetch_all(db)
        .await
        .map_err(|e| e.to_string())?;

    let mut relevant_facts = Vec::new();
    let query_lower = query.to_lowercase();

    for row in rows {
        let value: Vec<u8> = row.get(0);
        if let Ok(session) = serde_json::from_slice::<DeepLearningSession>(&value) {
            if session.topic.to_lowercase() == topic.to_lowercase()
                || session.subtopics.iter().any(|s| s.to_lowercase() == topic.to_lowercase())
            {
                for fact in &session.learned_facts {
                    // Check if fact is relevant to query
                    if fact.fact.to_lowercase().contains(&query_lower) || matches_any_concept(&query_lower, &fact.related_concepts) {
                        relevant_facts.push(fact.clone());
                    }
                }
            }
        }
    }

    // Sort by importance and usage
    relevant_facts.sort_by(|a, b| {
        let score_a = a.importance * (1.0 + (a.used_in_responses as f32) / 10.0);
        let score_b = b.importance * (1.0 + (b.used_in_responses as f32) / 10.0);
        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(relevant_facts.into_iter().take(5).collect())
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
pub async fn get_all_learning_sessions(db: &SqlitePool) -> Result<Vec<DeepLearningSession>, String> {
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
            expertise.expertise_level = ((facts_count / 5) + (expertise.total_study_hours as u32 / 10)).min(10);

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

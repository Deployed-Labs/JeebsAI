use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use std::collections::HashSet;

const PROPOSAL_KEY: &str = "jeebs:next_proposal";
const PROPOSAL_INTERVAL_SECS: i64 = 1800; // 30 minutes
const MAX_ACTIVE_PROPOSALS: usize = 3;

#[derive(Serialize, Deserialize, Clone)]
pub struct ProactiveProposal {
    pub action_type: String, // "learn", "feature", "experiment", "evolution"
    pub description: String,
    pub reason: String,
    pub created_at: String,
}

#[derive(Deserialize)]
struct FileChangeLite {
    path: String,
    new_content: String,
}

#[derive(Deserialize)]
struct EvolutionUpdateLite {
    #[serde(default)]
    changes: Vec<FileChangeLite>,
    #[serde(default)]
    status: String,
    #[serde(default)]
    created_at: String,
    #[serde(default)]
    title: String,
}

struct ReflectionCandidates {
    actions: Vec<String>,
    learning_topics: Vec<String>,
    scope_topics: Vec<String>,
}

fn parse_markdown_section(content: &str, header: &str) -> Vec<String> {
    let mut in_section = false;
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            in_section = trimmed == header;
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some(item) = trimmed.strip_prefix("- ") {
            if !item.is_empty() {
                out.push(item.to_string());
            }
        }
    }
    out
}

async fn load_reflection_candidates(db: &SqlitePool) -> ReflectionCandidates {
    let rows = sqlx::query("SELECT value FROM jeebs_store WHERE key LIKE 'evolution:update:%' ORDER BY key DESC LIMIT 50")
        .fetch_all(db)
        .await
        .unwrap_or_default();

    let mut actions = Vec::new();
    let mut learning_topics = Vec::new();
    let mut scope_topics = Vec::new();

    for row in rows {
        let raw: Vec<u8> = row.get(0);
        let Ok(update) = serde_json::from_slice::<EvolutionUpdateLite>(&raw) else {
            continue;
        };
        if update.status != "pending" && update.status != "applied" {
            continue;
        }

        for change in update.changes {
            if !change.path.starts_with("evolution/reflections/") {
                continue;
            }

            actions.extend(parse_markdown_section(&change.new_content, "## Suggested Actions"));
            learning_topics.extend(parse_markdown_section(&change.new_content, "## Conversation Gaps To Learn"));
            learning_topics.extend(parse_markdown_section(&change.new_content, "## Search Queries For Knowledge Expansion"));
            scope_topics.extend(parse_markdown_section(&change.new_content, "## Priority Topics"));
        }
    }

    actions.truncate(5);
    learning_topics.truncate(5);
    scope_topics.truncate(5);

    ReflectionCandidates {
        actions,
        learning_topics,
        scope_topics,
    }
}

fn base_action_type(action_type: &str) -> &str {
    if let Some(stripped) = action_type.strip_prefix("insight_") {
        if stripped.is_empty() {
            "insight"
        } else {
            "insight"
        }
    } else {
        action_type
    }
}

async fn load_proposals(db: &SqlitePool) -> Vec<ProactiveProposal> {
    let row = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(PROPOSAL_KEY)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    let Some(row) = row else {
        return Vec::new();
    };

    let value: Vec<u8> = row.get(0);
    if let Ok(list) = serde_json::from_slice::<Vec<ProactiveProposal>>(&value) {
        return list;
    }

    if let Ok(single) = serde_json::from_slice::<ProactiveProposal>(&value) {
        return vec![single];
    }

    Vec::new()
}

async fn save_proposals(db: &SqlitePool, proposals: &[ProactiveProposal]) {
    if let Ok(payload) = serde_json::to_vec(proposals) {
        let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
            .bind(PROPOSAL_KEY)
            .bind(&payload)
            .execute(db)
            .await;
    }
}

/// Actions Jeebs wants to propose
const LEARNING_TOPICS: &[&str] = &[
    "quantum computing fundamentals",
    "distributed systems architecture",
    "graph databases and Neo4j",
    "natural language processing techniques",
    "machine learning inference optimization",
    "blockchain and smart contracts",
    "computer vision and image recognition",
    "cybersecurity best practices",
    "functional programming patterns",
    "microservices design patterns",
    "rust async programming patterns",
    "database query optimization",
    "API design best practices",
    "containerization and Docker",
    "continuous integration pipelines",
];

const FEATURE_IDEAS: &[&str] = &[
    "Add voice input support for chat",
    "Implement real-time collaborative editing",
    "Create an API rate limiter dashboard",
    "Add export conversations to PDF",
    "Implement multi-language support",
    "Add dark/light theme toggle",
    "Create scheduled knowledge refresh",
    "Add user analytics dashboard",
    "Implement semantic code search",
    "Add automated testing framework",
    "Create knowledge graph visualization",
    "Add conversation branching support",
    "Implement plugin marketplace",
    "Add mobile app companion",
];

const EXPERIMENTS: &[&str] = &[
    "Test different knowledge graph traversal algorithms",
    "Benchmark response times with cached vs fresh queries",
    "Experiment with conversation context compression",
    "Test impact of different training data sizes",
    "Measure accuracy improvement from user feedback",
    "Compare SQLite vs PostgreSQL performance",
    "Test different embedding model strategies",
    "Analyze optimal cache invalidation patterns",
    "Experiment with parallel request handling",
    "Test different prompt engineering approaches",
    "Build weekly benchmark of unknown questions",
    "Compare retrieval quality with different summarization templates",
    "Validate training output reusability in conversations",
    "Stress-test response quality with adversarial prompts",
];

pub async fn should_propose_action(db: &SqlitePool) -> bool {
    let proposals = load_proposals(db).await;
    if proposals.len() >= MAX_ACTIVE_PROPOSALS {
        return false;
    }

    let newest_created = proposals
        .iter()
        .filter_map(|proposal| {
            chrono::DateTime::parse_from_rfc3339(&proposal.created_at)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Local))
        })
        .max();

    if let Some(created) = newest_created {
        let elapsed = (Local::now() - created).num_seconds();
        return elapsed >= PROPOSAL_INTERVAL_SECS;
    }

    true
}

pub async fn generate_proactive_proposal(db: &SqlitePool) -> Option<ProactiveProposal> {
    if !should_propose_action(db).await {
        return None;
    }

    let mut proposals = load_proposals(db).await;
    if proposals.len() >= MAX_ACTIVE_PROPOSALS {
        return None;
    }

    let mut existing_types = HashSet::new();
    for proposal in &proposals {
        existing_types.insert(base_action_type(&proposal.action_type).to_string());
    }

    let reflection = load_reflection_candidates(db).await;

    if !existing_types.contains("reflection") {
        if let Some(action) = reflection.actions.first() {
            let proposal = ProactiveProposal {
                action_type: "reflection".to_string(),
                description: format!("Act on reflection: {action}"),
                reason: "Derived from evolution reflection suggested actions.".to_string(),
                created_at: Local::now().to_rfc3339(),
            };
            proposals.push(proposal.clone());
            save_proposals(db, &proposals).await;
            return Some(proposal);
        }
    }

    if !existing_types.contains("learning") {
        if let Some(topic) = reflection.learning_topics.first() {
            let proposal = ProactiveProposal {
                action_type: "learning".to_string(),
                description: format!("Research learning gap: {topic}"),
                reason: "Pulled from evolution learning plan gaps and search queries.".to_string(),
                created_at: Local::now().to_rfc3339(),
            };
            proposals.push(proposal.clone());
            save_proposals(db, &proposals).await;
            return Some(proposal);
        }
    }

    if !existing_types.contains("scope") {
        if let Some(topic) = reflection.scope_topics.first() {
            let proposal = ProactiveProposal {
                action_type: "scope".to_string(),
                description: format!("Expand scope into: {topic}"),
                reason: "Derived from evolution scope roadmap priorities.".to_string(),
                created_at: Local::now().to_rfc3339(),
            };
            proposals.push(proposal.clone());
            save_proposals(db, &proposals).await;
            return Some(proposal);
        }
    }

    // Check if there are pending evolution updates
    let pending_count: i64 = sqlx::query(
        "SELECT COUNT(*) FROM jeebs_store WHERE key LIKE 'evolution:update:%' AND value LIKE '%\"status\":\"pending\"%'"
    )
    .fetch_optional(db)
    .await
    .ok()
    .flatten()
    .map(|row| row.get(0))
    .unwrap_or(0);

    // Try to generate insight-based proposals first (based on learned data)
    if (Local::now().timestamp() % 5) == 0 && !existing_types.contains("insight") {
        if let Ok(insight_proposals) = crate::data_synthesis::generate_insight_proposals(db).await {
            if let Some(insight) = insight_proposals.first() {
                let proposal = ProactiveProposal {
                    action_type: format!("insight_{}", insight.proposal_type),
                    description: insight.title.clone(),
                    reason: insight.description.clone(),
                    created_at: Local::now().to_rfc3339(),
                };
                proposals.push(proposal.clone());
                save_proposals(db, &proposals).await;
                return Some(proposal);
            }
        }
    }

    // Determine which type of action to propose (avoid duplicates)
    let mut candidate_types = Vec::new();
    if pending_count > 0 {
        candidate_types.push("evolution");
    }
    candidate_types.extend(["learn", "feature", "experiment"].iter().copied());

    let action_type = candidate_types
        .into_iter()
        .find(|candidate| !existing_types.contains(*candidate));
    let Some(action_type) = action_type else {
        return None;
    };

    let proposal = match action_type {
        "evolution" if pending_count > 0 => {
            ProactiveProposal {
                action_type: "evolution".to_string(),
                description: format!(
                    "I have {} self-improvement proposal{} waiting for review in my evolution system",
                    pending_count,
                    if pending_count == 1 { "" } else { "s" }
                ),
                reason: "These proposals include learning plans, experiments, and code improvements that could enhance my capabilities. You can review them at /webui/evolution.html".to_string(),
                created_at: Local::now().to_rfc3339(),
            }
        }
        "learn" => {
            // Try data-driven learning suggestions first
            if let Ok(profile) = crate::data_synthesis::generate_knowledge_insights(db).await {
                if let Some(gap) = profile.knowledge_gaps.first() {
                    return Some(ProactiveProposal {
                        action_type: "learn".to_string(),
                        description: format!("Fill knowledge gap: {}", gap),
                        reason: "Based on my analysis of learned content, this gap could improve my understanding".to_string(),
                        created_at: Local::now().to_rfc3339(),
                    });
                }
            }

            // Fallback to predefined topics
            let index = (Local::now().timestamp() as usize) % LEARNING_TOPICS.len();
            let topic = LEARNING_TOPICS[index];
            ProactiveProposal {
                action_type: "learn".to_string(),
                description: format!("I want to learn about {}", topic),
                reason: format!(
                    "Learning {} would expand my knowledge and help me answer more questions in this domain.",
                    topic
                ),
                created_at: Local::now().to_rfc3339(),
            }
        }
        "feature" => {
            let index = (Local::now().timestamp() as usize) % FEATURE_IDEAS.len();
            let feature = FEATURE_IDEAS[index];
            ProactiveProposal {
                action_type: "feature".to_string(),
                description: feature.to_string(),
                reason: "This feature would improve user experience and system capabilities.".to_string(),
                created_at: Local::now().to_rfc3339(),
            }
        }
        "experiment" => {
            let index = (Local::now().timestamp() as usize) % EXPERIMENTS.len();
            let experiment = EXPERIMENTS[index];
            ProactiveProposal {
                action_type: "experiment".to_string(),
                description: experiment.to_string(),
                reason: "Running this experiment would provide data to optimize performance and quality.".to_string(),
                created_at: Local::now().to_rfc3339(),
            }
        }
        _ => return None,
    };

    proposals.push(proposal.clone());
    if proposals.len() > MAX_ACTIVE_PROPOSALS {
        proposals.remove(0);
    }
    save_proposals(db, &proposals).await;

    Some(proposal)
}

pub fn format_proposal(proposal: &ProactiveProposal) -> String {
    match proposal.action_type.as_str() {
        "reflection" => {
            format!(
                "🧭 **Reflection Action**: {}\n\n**Why**: {}\n\nShould I proceed with this reflection-driven action?",
                proposal.description,
                proposal.reason
            )
        }
        "learning" => {
            format!(
                "📚 **Learning Plan Item**: {}\n\n**Why**: {}\n\nShould I research and add this knowledge?",
                proposal.description,
                proposal.reason
            )
        }
        "scope" => {
            format!(
                "🧩 **Scope Expansion**: {}\n\n**Why**: {}\n\nShould I expand into this area?",
                proposal.description,
                proposal.reason
            )
        }
        "learn" => {
            format!(
                "💡 **Proactive Suggestion**: {}\n\n**Why**: {}\n\nWould you like me to research this topic and add it to my knowledge base?",
                proposal.description,
                proposal.reason
            )
        }
        "feature" => {
            format!(
                "🔧 **Feature Idea**: {}\n\n**Why**: {}\n\nShould I create a proposal for this feature?",
                proposal.description,
                proposal.reason
            )
        }
        "experiment" => {
            format!(
                "🧪 **Experiment Proposal**: {}\n\n**Why**: {}\n\nShall I add this to my experiment backlog?",
                proposal.description,
                proposal.reason
            )
        }
        "evolution" => {
            format!(
                "🧬 **Evolution Alert**: {}\n\n**Details**: {}\n\nWould you like to review these proposals?",
                proposal.description,
                proposal.reason
            )
        }
        _ => format!("{}\n\n{}", proposal.description, proposal.reason),
    }
}

pub async fn acknowledge_proposal(db: &SqlitePool) {
    // Update timestamps to prevent immediate re-proposal
    let mut proposals = load_proposals(db).await;
    if proposals.is_empty() {
        return;
    }

    let now = Local::now().to_rfc3339();
    for proposal in &mut proposals {
        proposal.created_at = now.clone();
    }

    save_proposals(db, &proposals).await;
}

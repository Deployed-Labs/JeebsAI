use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};

const PROPOSAL_KEY: &str = "jeebs:next_proposal";
const PROPOSAL_INTERVAL_SECS: i64 = 1800; // 30 minutes

#[derive(Serialize, Deserialize, Clone)]
pub struct ProactiveProposal {
    pub action_type: String, // "learn", "feature", "experiment", "evolution"
    pub description: String,
    pub reason: String,
    pub created_at: String,
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
    let row = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(PROPOSAL_KEY)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    if let Some(row) = row {
        let value: Vec<u8> = row.get(0);
        if let Ok(proposal) = serde_json::from_slice::<ProactiveProposal>(&value) {
            let created = chrono::DateTime::parse_from_rfc3339(&proposal.created_at)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Local));

            if let Some(created) = created {
                let elapsed = (Local::now() - created).num_seconds();
                return elapsed >= PROPOSAL_INTERVAL_SECS;
            }
        }
    }

    true // No previous proposal or error parsing, so we can propose
}

pub async fn generate_proactive_proposal(db: &SqlitePool) -> Option<ProactiveProposal> {
    if !should_propose_action(db).await {
        return None;
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

    // Determine which type of action to propose
    let action_type = if pending_count > 0 && (Local::now().timestamp() % 4) == 0 {
        "evolution"
    } else {
        match (Local::now().timestamp() % 3) {
            0 => "learn",
            1 => "feature",
            _ => "experiment",
        }
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

    // Save the proposal
    let payload = serde_json::to_vec(&proposal).ok()?;
    let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(PROPOSAL_KEY)
        .bind(&payload)
        .execute(db)
        .await;

    Some(proposal)
}

pub fn format_proposal(proposal: &ProactiveProposal) -> String {
    match proposal.action_type.as_str() {
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
    // Update the timestamp to prevent immediate re-proposal
    let proposal_key = PROPOSAL_KEY;
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(proposal_key)
        .fetch_optional(db)
        .await
    {
        let value: Vec<u8> = row.get(0);
        if let Ok(mut proposal) = serde_json::from_slice::<ProactiveProposal>(&value) {
            proposal.created_at = Local::now().to_rfc3339();
            if let Ok(payload) = serde_json::to_vec(&proposal) {
                let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
                    .bind(proposal_key)
                    .bind(&payload)
                    .execute(db)
                    .await;
            }
        }
    }
}

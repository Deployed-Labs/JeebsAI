use chrono::Local;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use std::collections::HashSet;

const PROPOSAL_KEY: &str = "jeebs:next_proposal";
// Allow faster proactive proposal generation; approvals still manual
const PROPOSAL_INTERVAL_SECS: i64 = 900; // 15 minutes
const MAX_ACTIVE_PROPOSALS: usize = 6;

// New: Proposal template types
const TEMPLATE_PROPOSAL_KEY: &str = "jeebs:template_proposals";
const TEMPLATE_PROPOSAL_INTERVAL_SECS: i64 = 1800; // 30 minutes

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TemplateProposal {
    pub id: String,
    pub template_type: String, // "learning_sprint", "feature", "data_storage", "communication"
    pub title: String,
    pub description: String,
    pub implementation_steps: Vec<String>,
    pub expected_impact: String,
    pub difficulty_level: String, // "easy", "medium", "hard"
    pub estimated_time_hours: u32,
    pub created_at: String,
    pub status: String, // "proposed", "accepted", "rejected", "in_progress", "completed"
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TemplateProposalSet {
    pub proposals: Vec<TemplateProposal>,
    pub created_at: String,
    pub selection_round: u32,
}

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

            actions.extend(parse_markdown_section(
                &change.new_content,
                "## Suggested Actions",
            ));
            learning_topics.extend(parse_markdown_section(
                &change.new_content,
                "## Conversation Gaps To Learn",
            ));
            learning_topics.extend(parse_markdown_section(
                &change.new_content,
                "## Search Queries For Knowledge Expansion",
            ));
            scope_topics.extend(parse_markdown_section(
                &change.new_content,
                "## Priority Topics",
            ));
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
                proposal.description, proposal.reason
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

// ===== AUTONOMOUS LEARNING SPRINT TEMPLATES =====
const LEARNING_SPRINT_TEMPLATES: &[(&str, &str, &[&str])] = &[
    (
        "Advanced Rust Concurrency Patterns",
        "Deep dive into tokio, async/await, and concurrent data structures for building highly scalable systems",
        &[
            "Study tokio runtime architecture and task scheduling",
            "Master async/await patterns and future composition",
            "Learn concurrent collections and synchronization primitives",
            "Implement a multi-threaded message queue system",
            "Benchmark and optimize concurrent code paths",
        ],
    ),
    (
        "Machine Learning Fundamentals",
        "Comprehensive understanding of ML algorithms, models, and training techniques",
        &[
            "Study linear regression, decision trees, and ensemble methods",
            "Learn neural network architectures and backpropagation",
            "Understand feature engineering and model validation",
            "Implement ML pipeline from scratch",
            "Analyze trade-offs between accuracy and performance",
        ],
    ),
    (
        "Distributed Systems Architecture",
        "Master patterns for building scalable, fault-tolerant distributed systems",
        &[
            "Study consensus algorithms (Raft, Paxos)",
            "Learn distributed transaction patterns",
            "Understand sharding and replication strategies",
            "Design fault-tolerance mechanisms",
            "Implement a distributed key-value store prototype",
        ],
    ),
    (
        "Natural Language Processing Deep Dive",
        "Advanced NLP techniques for semantic understanding and language generation",
        &[
            "Study tokenization and embedding techniques",
            "Learn transformer architectures and attention mechanisms",
            "Understand prompt engineering and few-shot learning",
            "Analyze semantic similarity and information retrieval",
            "Build custom NLP pipelines for domain-specific tasks",
        ],
    ),
    (
        "Database Performance Optimization",
        "Master query optimization, indexing strategies, and data modeling",
        &[
            "Study query execution plans and optimization strategies",
            "Master index design and composite key strategies",
            "Learn denormalization and caching patterns",
            "Profile and optimize slow queries",
            "Understand columnar vs row-oriented storage trade-offs",
        ],
    ),
    (
        "Cloud Architecture Patterns",
        "Design and implement scalable cloud-native applications",
        &[
            "Study microservices architecture and service mesh",
            "Learn containerization and orchestration",
            "Understand auto-scaling and load balancing",
            "Master infrastructure as code practices",
            "Implement multi-region fault-tolerant system",
        ],
    ),
];

// ===== FEATURE/MODIFICATION TEMPLATES =====
const FEATURE_TEMPLATES: &[(&str, &str, &[&str])] = &[
    (
        "Advanced Knowledge Graph Visualization",
        "Interactive 3D visualization of brain knowledge graphs with relationship filtering and semantic search",
        &[
            "Design graph visualization UI with WebGL or Three.js",
            "Implement relationship filtering and path finding",
            "Add semantic similarity highlighting",
            "Create export functionality for graph snapshots",
            "Optimize rendering for large graphs (10k+ nodes)",
        ],
    ),
    (
        "Multi-Language Support System",
        "Enable JeebsAI to understand and respond in multiple languages with proper context preservation",
        &[
            "Integrate language detection system",
            "Add translation layer with quality verification",
            "Implement language-aware knowledge retrieval",
            "Create language-specific training pipelines",
            "Build multilingual test suite",
        ],
    ),
    (
        "Real-Time Collaborative Brain Editing",
        "Allow multiple users to collaboratively expand and refine JeebsAI's knowledge base",
        &[
            "Design conflict resolution for concurrent edits",
            "Implement operational transform or CRDT",
            "Create audit trail for all modifications",
            "Build permission and role system",
            "Add real-time notification system",
        ],
    ),
    (
        "Conversation Branching and Exploration",
        "Enable users to explore alternative conversation paths and track multiple threads",
        &[
            "Design conversation tree data structure",
            "Implement branch point creation and management",
            "Add comparison view for different branches",
            "Create merge functionality for convergent paths",
            "Build visualization of conversation topology",
        ],
    ),
    (
        "Plugin Architecture Expansion",
        "Modular plugin system for extending JeebsAI with custom integrations and capabilities",
        &[
            "Design plugin interface and lifecycle hooks",
            "Create plugin marketplace with versioning",
            "Implement sandboxed plugin execution",
            "Add plugin dependency resolution",
            "Build developer toolkit and documentation",
        ],
    ),
    (
        "Advanced Scheduling and Task Management",
        "Autonomous task scheduling with dependency management and execution monitoring",
        &[
            "Design task dependency graph and DAG scheduler",
            "Implement priority and resource allocation",
            "Add execution monitoring and failure recovery",
            "Create task result aggregation and reporting",
            "Build human-in-the-loop approval workflow",
        ],
    ),
];

// ===== DATA STORAGE OPTIMIZATION TEMPLATES =====
const DATA_STORAGE_TEMPLATES: &[(&str, &str, &[&str])] = &[
    (
        "Compressed Knowledge Graph Storage",
        "Implement advanced compression for knowledge graphs while maintaining query performance",
        &[
            "Research graph compression algorithms (BP, BitMat, WebGraph)",
            "Design delta encoding for temporal changes",
            "Implement LZ4/Zstd compression layers",
            "Create transparent decompression caching",
            "Benchmark compression ratios vs query latency",
        ],
    ),
    (
        "Smart Data Tiering System",
        "Automatically move data between hot (SSD), warm (HDD), and cold (archive) storage based on access patterns",
        &[
            "Build access pattern tracking and analysis",
            "Design tiering policy engine with machine learning",
            "Implement transparent data migration",
            "Create data locality awareness for queries",
            "Add cost optimization metrics and reporting",
        ],
    ),
    (
        "Incremental Backup and Delta Compression",
        "Efficient backup strategy with deduplication and delta compression for large datasets",
        &[
            "Implement content-addressable storage (CAS)",
            "Design delta compression between versions",
            "Create incremental backup snapshots",
            "Build efficient restoration from incremental backups",
            "Add point-in-time recovery capabilities",
        ],
    ),
    (
        "Vector Database Integration",
        "Leverage specialized vector databases for semantic search and similarity matching",
        &[
            "Evaluate vector databases (Milvus, Weaviate, Pinecone)",
            "Design embedding generation pipeline",
            "Implement approximate nearest neighbor search",
            "Create synchronization between relational and vector storage",
            "Benchmark similarity search performance",
        ],
    ),
    (
        "Schema Evolution and Migration Framework",
        "Seamless schema changes with zero-downtime migration for large datasets",
        &[
            "Design schema versioning system",
            "Implement gradual migration with dual-write",
            "Create automatic data transformation pipeline",
            "Add rollback capabilities for failed migrations",
            "Build schema compatibility verification",
        ],
    ),
    (
        "Columnar Storage for Analytics",
        "Optimize analytical queries with column-oriented storage layout",
        &[
            "Design columnar storage format (Parquet-compatible)",
            "Implement compression per column type",
            "Create efficient aggregation functions",
            "Build query planner for column selection",
            "Add statistics and cost-based optimization",
        ],
    ),
];

// ===== COMMUNICATION AND LOGIC IMPROVEMENT TEMPLATES =====
const COMMUNICATION_LOGIC_TEMPLATES: &[(&str, &str, &[&str])] = &[
    (
        "Enhanced Reasoning Chain Framework",
        "Implement chain-of-thought reasoning with verification and backtracking capabilities",
        &[
            "Design step-by-step reasoning decomposition",
            "Implement intermediate result verification",
            "Create confidence scoring for each step",
            "Add backtracking when reasoning fails",
            "Build reasoning pattern library and templates",
        ],
    ),
    (
        "Semantic Intent Understanding",
        "Deeper understanding of user intent beyond literal questions",
        &[
            "Implement semantic role labeling and intent classification",
            "Design context-aware intent resolution",
            "Create implicit intent inference",
            "Build intent uncertainty handling",
            "Add clarification question generation",
        ],
    ),
    (
        "Multi-Modal Communication Bridge",
        "Support multiple input/output modalities: text, code, diagrams, equations",
        &[
            "Design modality detection and routing",
            "Implement syntax highlighting and rendering",
            "Create mathematical equation parsing and display",
            "Add diagram generation and interpretation",
            "Build modality-specific response formatting",
        ],
    ),
    (
        "Uncertainty and Confidence Quantification",
        "Explicit confidence levels and uncertainty bounds for all responses",
        &[
            "Implement Bayesian uncertainty estimation",
            "Design confidence scoring for facts vs inference",
            "Create uncertainty propagation through chains",
            "Add confidence intervals for predictions",
            "Build transparent uncertainty communication",
        ],
    ),
    (
        "Context-Aware Response Adaptation",
        "Adapt communication style and depth based on user expertise and conversation context",
        &[
            "Build user expertise modeling system",
            "Design adaptive explanation depth",
            "Implement technical vs layman's translation",
            "Create personalization from conversation history",
            "Add response style preferences learning",
        ],
    ),
    (
        "Logical Consistency Checker",
        "Verify responses for logical consistency, contradictions, and knowledge base alignment",
        &[
            "Design fact consistency verification",
            "Implement logical contradiction detection",
            "Create knowledge base alignment checking",
            "Add assumption validation system",
            "Build inconsistency reporting and resolution",
        ],
    ),
];

// ===== TEMPLATE PROPOSAL FUNCTIONS =====

async fn load_template_proposals(db: &SqlitePool) -> Option<TemplateProposalSet> {
    let row = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(TEMPLATE_PROPOSAL_KEY)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()?;

    let value: Vec<u8> = row.get(0);
    serde_json::from_slice::<TemplateProposalSet>(&value).ok()
}

async fn save_template_proposals(db: &SqlitePool, proposals: &TemplateProposalSet) {
    if let Ok(payload) = serde_json::to_vec(proposals) {
        let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
            .bind(TEMPLATE_PROPOSAL_KEY)
            .bind(&payload)
            .execute(db)
            .await;
    }
}

fn generate_template_id(template_type: &str, index: usize, round: u32) -> String {
    format!("template_{}_{}_round{}", template_type, index, round)
}

/// Generate 2 random proposals from the 4 template types
pub async fn generate_template_proposals(db: &SqlitePool) -> Option<TemplateProposalSet> {
    // Check if we should generate new proposals
    if let Some(existing) = load_template_proposals(db).await {
        let created = chrono::DateTime::parse_from_rfc3339(&existing.created_at)
            .ok()?
            .with_timezone(&chrono::Local);

        let elapsed = (Local::now() - created).num_seconds();
        if elapsed < TEMPLATE_PROPOSAL_INTERVAL_SECS {
            return Some(existing); // Still within interval
        }
    }

    let mut rng = rand::thread_rng();

    // Template types available
    let template_types = vec![
        ("learning_sprint", LEARNING_SPRINT_TEMPLATES.len()),
        ("feature", FEATURE_TEMPLATES.len()),
        ("data_storage", DATA_STORAGE_TEMPLATES.len()),
        ("communication", COMMUNICATION_LOGIC_TEMPLATES.len()),
    ];

    // Select 2 different template types randomly
    let selected_types: Vec<_> = template_types.choose_multiple(&mut rng, 2).collect();

    let mut proposals = Vec::new();
    let round = (Local::now().timestamp() / TEMPLATE_PROPOSAL_INTERVAL_SECS) as u32;

    for (template_type, max_index) in selected_types {
        let index = rng.gen_range(0..*max_index);

        let (title, description, steps) = match *template_type {
            "learning_sprint" => LEARNING_SPRINT_TEMPLATES[index],
            "feature" => FEATURE_TEMPLATES[index],
            "data_storage" => DATA_STORAGE_TEMPLATES[index],
            "communication" => COMMUNICATION_LOGIC_TEMPLATES[index],
            _ => continue,
        };

        let (difficulty, time_hours) = match *template_type {
            "learning_sprint" => ("hard", 40),
            "feature" => ("medium", 80),
            "data_storage" => ("hard", 120),
            "communication" => ("medium", 60),
            _ => ("medium", 40),
        };

        let proposal = TemplateProposal {
            id: generate_template_id(template_type, index, round),
            template_type: template_type.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            implementation_steps: steps.iter().map(|s| s.to_string()).collect(),
            expected_impact: format!(
                "Significant improvement in {} capabilities and system effectiveness",
                match *template_type {
                    "learning_sprint" => "learning and knowledge",
                    "feature" => "user experience and functionality",
                    "data_storage" => "storage efficiency and scalability",
                    "communication" => "reasoning and response quality",
                    _ => "system",
                }
            ),
            difficulty_level: difficulty.to_string(),
            estimated_time_hours: time_hours,
            created_at: Local::now().to_rfc3339(),
            status: "proposed".to_string(),
        };

        proposals.push(proposal);
    }

    if proposals.len() != 2 {
        return None;
    }

    let proposal_set = TemplateProposalSet {
        proposals,
        created_at: Local::now().to_rfc3339(),
        selection_round: round,
    };

    save_template_proposals(db, &proposal_set).await;
    Some(proposal_set)
}

/// Get current template proposals
pub async fn get_template_proposals(db: &SqlitePool) -> Option<TemplateProposalSet> {
    load_template_proposals(db).await.or_else(|| {
        // If no proposals exist yet, this will return None
        // The caller should handle generation
        None
    })
}

/// Update status of a template proposal
pub async fn update_template_proposal_status(
    db: &SqlitePool,
    proposal_id: &str,
    new_status: &str,
) -> bool {
    if let Some(mut proposal_set) = load_template_proposals(db).await {
        for proposal in &mut proposal_set.proposals {
            if proposal.id == proposal_id {
                proposal.status = new_status.to_string();
                save_template_proposals(db, &proposal_set).await;
                return true;
            }
        }
    }
    false
}

/// Format template proposals for display
pub fn format_template_proposals(proposal_set: &TemplateProposalSet) -> String {
    let mut output = String::new();
    output.push_str("🎯 **Current Proposal Round**\n\n");
    output.push_str(&format!(
        "*Selection Round: {}*\n\n",
        proposal_set.selection_round
    ));

    for (idx, proposal) in proposal_set.proposals.iter().enumerate() {
        let emoji = match proposal.template_type.as_str() {
            "learning_sprint" => "📚",
            "feature" => "🔧",
            "data_storage" => "💾",
            "communication" => "💡",
            _ => "⭐",
        };

        output.push_str(&format!(
            "{} **{}**: {}\n\n",
            emoji, proposal.title, proposal.description
        ));

        output.push_str(&format!(
            "**Type**: {}\n**Difficulty**: {}\n**Estimated Time**: {} hours\n\n",
            proposal.template_type, proposal.difficulty_level, proposal.estimated_time_hours
        ));

        output.push_str("**Implementation Steps**:\n");
        for step in &proposal.implementation_steps {
            output.push_str(&format!("  • {}\n", step));
        }

        output.push_str(&format!(
            "\n**Expected Impact**: {}\n\n",
            proposal.expected_impact
        ));

        if idx < proposal_set.proposals.len() - 1 {
            output.push_str("---\n\n");
        }
    }

    output.push_str("\n**Would you like to accept any of these proposals for implementation?**");
    output
}

/// Get proposal statistics
pub async fn get_proposal_statistics(db: &SqlitePool) -> Option<serde_json::Value> {
    if let Some(proposal_set) = load_template_proposals(db).await {
        let total_proposals = proposal_set.proposals.len();
        let accepted = proposal_set
            .proposals
            .iter()
            .filter(|p| p.status == "accepted")
            .count();
        let in_progress = proposal_set
            .proposals
            .iter()
            .filter(|p| p.status == "in_progress")
            .count();
        let completed = proposal_set
            .proposals
            .iter()
            .filter(|p| p.status == "completed")
            .count();

        let total_estimated_hours: u32 = proposal_set
            .proposals
            .iter()
            .filter(|p| p.status != "rejected")
            .map(|p| p.estimated_time_hours)
            .sum();

        return Some(json!({
            "total_proposals": total_proposals,
            "accepted": accepted,
            "in_progress": in_progress,
            "completed": completed,
            "rejected": total_proposals - accepted - in_progress - completed,
            "total_estimated_hours": total_estimated_hours,
            "selection_round": proposal_set.selection_round,
            "proposals": proposal_set.proposals.iter().map(|p| {
                json!({
                    "id": p.id,
                    "title": p.title,
                    "type": p.template_type,
                    "status": p.status,
                    "difficulty": p.difficulty_level,
                    "hours": p.estimated_time_hours,
                })
            }).collect::<Vec<_>>(),
        }));
    }

    None
}

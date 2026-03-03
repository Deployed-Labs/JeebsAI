/// Intelligent Inference Engine
///
/// Provides context-aware reasoning by:
/// - Retrieving relevant brain context and learning sessions
/// - Building knowledge graphs from connections
/// - Performing multi-hop reasoning across related concepts
/// - Tracking inference process and confidence scores
/// - Learning from conversation outcomes

use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct InferenceContext {
    pub query: String,
    pub user_id: Option<String>,
    pub related_topics: Vec<String>,
    pub relevant_facts: Vec<FactWithScore>,
    pub knowledge_graph: KnowledgeGraph,
    pub learning_sessions: Vec<SessionContext>,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct FactWithScore {
    pub fact: String,
    pub source: String,
    pub relevance_score: f32,
    pub importance: f32,
    pub usage_count: u32,
    pub related_concepts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct KnowledgeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub weight: f32,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub strength: f32,
}

#[derive(Debug, Clone)]
pub struct SessionContext {
    pub id: String,
    pub topic: String,
    pub confidence: f32,
    pub depth_level: u32,
    pub key_facts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub response: String,
    pub confidence: f32,
    pub reasoning: String,
    pub sources: Vec<String>,
    pub learned_concepts: Vec<String>,
}

/// Build rich inference context from user query
pub async fn build_context(
    pool: &SqlitePool,
    query: &str,
    user_id: Option<&str>,
) -> Result<InferenceContext, String> {
    println!("[Inference] Building context for query: {}", query);

    // Extract key concepts from query
    let key_concepts = extract_concepts(query);

    // Retrieve relevant facts from brain
    let facts = retrieve_relevant_facts(pool, &key_concepts, 10).await?;

    // Build knowledge graph from connections
    let knowledge_graph = build_knowledge_graph(pool, &key_concepts).await?;

    // Find relevant learning sessions
    let learning_sessions = find_relevant_sessions(pool, &key_concepts).await?;

    // Calculate overall confidence
    let confidence = calculate_confidence(&facts, &learning_sessions);

    Ok(InferenceContext {
        query: query.to_string(),
        user_id: user_id.map(|s| s.to_string()),
        related_topics: key_concepts,
        relevant_facts: facts,
        knowledge_graph,
        learning_sessions,
        confidence,
    })
}

/// Extract key concepts from natural language query
fn extract_concepts(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .filter(|word| word.len() > 3)
        .map(|w| w.to_lowercase())
        .collect()
}

/// Retrieve facts relevant to concepts with scoring
async fn retrieve_relevant_facts(
    pool: &SqlitePool,
    concepts: &[String],
    limit: usize,
) -> Result<Vec<FactWithScore>, String> {
    if concepts.is_empty() {
        return Ok(Vec::new());
    }

    let mut all_facts: Vec<FactWithScore> = Vec::new();

    // Query brain_nodes_v2 for matching facts
    for concept in concepts.iter().take(5) {
        let pattern = format!("%{}%", concept);

        let rows = sqlx::query_as::<_, (String, String, String)>(
            "SELECT id, fact, category FROM brain_nodes_v2
             WHERE fact LIKE ? OR category LIKE ?
             LIMIT 20",
        )
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for (id, fact, category) in rows {
            all_facts.push(FactWithScore {
                fact,
                source: category,
                relevance_score: calculate_relevance(&id, concept),
                importance: 0.7,
                usage_count: 0,
                related_concepts: vec![concept.clone()],
            });
        }
    }

    // Query knowledge_triples for structured knowledge
    for concept in concepts.iter().take(3) {
        let rows = sqlx::query_as::<_, (String, String, String, f64)>(
            "SELECT subject, predicate, object, confidence FROM knowledge_triples
             WHERE subject LIKE ? OR object LIKE ?
             LIMIT 15",
        )
        .bind(format!("%{}%", concept))
        .bind(format!("%{}%", concept))
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for (subject, predicate, object, conf) in rows {
            all_facts.push(FactWithScore {
                fact: format!("{} {} {}", subject, predicate, object),
                source: "knowledge_triple".to_string(),
                relevance_score: conf as f32,
                importance: conf as f32,
                usage_count: 0,
                related_concepts: vec![concept.clone()],
            });
        }
    }

    // Sort by relevance and return top N
    all_facts.sort_by(|a, b| {
        (b.relevance_score * b.importance)
            .partial_cmp(&(a.relevance_score * a.importance))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let top_facts: Vec<FactWithScore> = all_facts.into_iter().take(limit).collect();
    println!("[Inference] Retrieved {} relevant facts", top_facts.len());

    Ok(top_facts)
}

/// Build knowledge graph from concept connections
async fn build_knowledge_graph(
    pool: &SqlitePool,
    concepts: &[String],
) -> Result<KnowledgeGraph, String> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut seen_edges = HashSet::new();

    // Add concept nodes
    for concept in concepts {
        nodes.push(GraphNode {
            id: concept.clone(),
            label: concept.clone(),
            node_type: "concept".to_string(),
            weight: 1.0,
        });
    }

    // Query connections between concepts
    for concept in concepts.iter().take(3) {
        let rows = sqlx::query_as::<_, (String, String, f64)>(
            "SELECT from_node, to_node, strength FROM connections
             WHERE from_node LIKE ? OR to_node LIKE ?
             LIMIT 30",
        )
        .bind(format!("%{}%", concept))
        .bind(format!("%{}%", concept))
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for (from, to, strength) in rows {
            let edge_key = format!("{}-{}", from, to);
            if !seen_edges.contains(&edge_key) {
                edges.push(GraphEdge {
                    from: from.clone(),
                    to: to.clone(),
                    relation: "connected".to_string(),
                    strength: strength as f32,
                });
                seen_edges.insert(edge_key);

                // Ensure both nodes exist
                if !nodes.iter().any(|n| n.id == from) {
                    nodes.push(GraphNode {
                        id: from.clone(),
                        label: from.clone(),
                        node_type: "node".to_string(),
                        weight: strength as f32,
                    });
                }
                if !nodes.iter().any(|n| n.id == to) {
                    nodes.push(GraphNode {
                        id: to.clone(),
                        label: to.clone(),
                        node_type: "node".to_string(),
                        weight: strength as f32,
                    });
                }
            }
        }
    }

    println!(
        "[Inference] Built knowledge graph: {} nodes, {} edges",
        nodes.len(),
        edges.len()
    );

    Ok(KnowledgeGraph { nodes, edges })
}

/// Find learning sessions relevant to concepts
async fn find_relevant_sessions(
    pool: &SqlitePool,
    concepts: &[String],
) -> Result<Vec<SessionContext>, String> {
    let mut sessions = Vec::new();

    // Query jeebs_store for learning sessions
    let rows = sqlx::query_as::<_, (String, Vec<u8>)>(
        "SELECT key, value FROM jeebs_store WHERE key LIKE 'learnsession:%' LIMIT 50",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (_key, value) in rows {
        if let Ok(session_json) = serde_json::from_slice::<Value>(&value) {
            if let (Some(topic), Some(conf), Some(depth)) = (
                session_json.get("topic").and_then(|v| v.as_str()),
                session_json.get("confidence").and_then(|v| v.as_f64()),
                session_json.get("depth_level").and_then(|v| v.as_u64()),
            ) {
                // Score session based on concept relevance
                let mut relevance = 0.0;
                for concept in concepts {
                    if topic.to_lowercase().contains(&concept.to_lowercase()) {
                        relevance += 1.0;
                    }
                }

                if relevance > 0.0 {
                    let key_facts = session_json
                        .get("learned_facts")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .take(3)
                                .filter_map(|f| f.get("fact").and_then(|ff| ff.as_str()))
                                .map(|s| s.to_string())
                                .collect()
                        })
                        .unwrap_or_default();

                    sessions.push(SessionContext {
                        id: topic.to_string(),
                        topic: topic.to_string(),
                        confidence: conf as f32,
                        depth_level: depth as u32,
                        key_facts,
                    });
                }
            }
        }
    }

    sessions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    println!("[Inference] Found {} relevant learning sessions", sessions.len());

    Ok(sessions)
}

/// Calculate overall confidence from facts and sessions
fn calculate_confidence(facts: &[FactWithScore], sessions: &[SessionContext]) -> f32 {
    let fact_confidence = if facts.is_empty() {
        0.3
    } else {
        facts.iter().map(|f| f.relevance_score).sum::<f32>() / facts.len() as f32
    };

    let session_confidence = if sessions.is_empty() {
        0.2
    } else {
        sessions.iter().map(|s| s.confidence).sum::<f32>() / sessions.len() as f32
    };

    (fact_confidence * 0.6 + session_confidence * 0.4).min(0.95)
}

/// Calculate relevance score for a fact
fn calculate_relevance(fact_id: &str, concept: &str) -> f32 {
    // Simple heuristic: we could enhance this with vector similarity later
    if fact_id.to_lowercase().contains(&concept.to_lowercase()) {
        0.9
    } else {
        0.5
    }
}

/// Perform intelligent inference with reasoning
pub async fn infer_response(
    context: &InferenceContext,
) -> Result<InferenceResult, String> {
    println!("[Inference] Performing inference with confidence: {}", context.confidence);

    // Build reasoning chain
    let reasoning = build_reasoning_chain(context);

    // Generate response from facts and reasoning
    let response = generate_intelligent_response(context, &reasoning);

    // Extract learned concepts
    let learned_concepts = context
        .related_topics
        .iter()
        .take(5)
        .cloned()
        .collect();

    // Gather sources
    let sources = context
        .relevant_facts
        .iter()
        .map(|f| f.source.clone())
        .collect::<std::collections::HashSet<_>>()
        .iter()
        .cloned()
        .collect();

    Ok(InferenceResult {
        response,
        confidence: context.confidence,
        reasoning,
        sources,
        learned_concepts,
    })
}

/// Build reasoning chain showing inference steps
fn build_reasoning_chain(context: &InferenceContext) -> String {
    let mut chain = vec![
        format!("Query: {}", context.query),
        format!("Key concepts identified: {}", context.related_topics.join(", ")),
        format!("Relevant facts found: {}", context.relevant_facts.len()),
        format!("Knowledge connections: {}", context.knowledge_graph.edges.len()),
        format!(
            "Learning sessions available: {}",
            context.learning_sessions.len()
        ),
    ];

    if !context.learning_sessions.is_empty() {
        chain.push(format!(
            "Highest confidence topic: '{}' at {:.0}%",
            context.learning_sessions[0].topic,
            context.learning_sessions[0].confidence * 100.0
        ));
    }

    chain.join(" → ")
}

/// Generate intelligent response from context
fn generate_intelligent_response(context: &InferenceContext, reasoning: &str) -> String {
    let mut response = String::new();

    // Add confidence indicator
    let confidence_indicator = if context.confidence > 0.8 {
        "I'm confident that..."
    } else if context.confidence > 0.6 {
        "Based on my understanding..."
    } else {
        "From what I can gather..."
    };

    response.push_str(&format!("{}\n\n", confidence_indicator));

    // Add key facts
    if !context.relevant_facts.is_empty() {
        response.push_str("Key points:\n");
        for fact in context.relevant_facts.iter().take(5) {
            response.push_str(&format!("  • {}\n", fact.fact));
        }
        response.push('\n');
    }

    // Add learning session insights if available
    if !context.learning_sessions.is_empty() {
        let session = &context.learning_sessions[0];
        response.push_str(&format!(
            "From my learning on '{}' (Level {}): I've developed {} confidence in this area.\n\n",
            session.topic, session.depth_level, session.confidence as i32
        ));

        if !session.key_facts.is_empty() {
            response.push_str("Core understandings:\n");
            for fact in session.key_facts.iter().take(3) {
                response.push_str(&format!("  • {}\n", fact));
            }
            response.push('\n');
        }
    }

    // Add knowledge graph connections if meaningful
    if context.knowledge_graph.edges.len() > 3 {
        response.push_str("Related concepts and connections:\n");
        for edge in context.knowledge_graph.edges.iter().take(3) {
            response.push_str(&format!(
                "  • {} ↔ {} (strength: {:.1}%)\n",
                edge.from, edge.to, edge.strength * 100.0
            ));
        }
        response.push('\n');
    }

    // Add reasoning trace if confidence is lower
    if context.confidence < 0.7 {
        response.push_str(&format!("Reasoning: {}\n", reasoning));
    }

    response
}

/// Log inference outcome for continuous learning
pub async fn log_inference_outcome(
    pool: &SqlitePool,
    inference: &InferenceResult,
    user_feedback: Option<&str>,
) -> Result<(), String> {
    let outcome_json = json!({
        "response": inference.response,
        "confidence": inference.confidence,
        "reasoning": inference.reasoning,
        "feedback": user_feedback,
        "timestamp": chrono::Local::now().to_rfc3339(),
    });

    let key = format!(
        "inference_outcome:{}",
        uuid::Uuid::new_v4().to_string()
    );

    sqlx::query(
        "INSERT INTO jeebs_store (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(&key)
    .bind(serde_json::to_vec(&outcome_json).map_err(|e| e.to_string())?)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    println!("[Inference] Logged outcome for continuous learning");

    Ok(())
}

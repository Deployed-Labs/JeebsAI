/// Model Context Protocol (MCP) Server for JeebsAI
///
/// Provides Claude and other LLMs with structured access to:
/// - Brain knowledge bases (facts, concepts, relationships)
/// - Learning session data
/// - Inference capabilities
/// - Memory management
///
/// This allows Claude to:
/// - Query the holographic brain for relevant context
/// - Make informed decisions based on learned knowledge
/// - Perform transparent reasoning with access to sources
/// - Track and learn from interactions

use serde_json::{json, Value};
use sqlx::SqlitePool;

/// MCP Resource: Brain Fact Search
///
/// Returns relevant facts from the brain matching a query
pub async fn search_brain_facts(
    pool: &SqlitePool,
    query: &str,
    max_results: usize,
) -> Result<Value, String> {
    let pattern = format!("%{}%", query);

    // Search brain_nodes_v2 for matching facts
    let facts = sqlx::query_as::<_, (String, String, String)>(
        "SELECT id, fact, category FROM brain_nodes_v2
         WHERE fact LIKE ? OR category LIKE ?
         ORDER BY id DESC
         LIMIT ?",
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(max_results as i32)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    let results = facts
        .into_iter()
        .map(|(id, fact, category)| {
            json!({
                "id": id,
                "fact": fact,
                "category": category,
                "type": "brain_fact"
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "type": "resource",
        "content": {
            "query": query,
            "results": results,
            "total": results.len()
        }
    }))
}

/// MCP Tool: Query Knowledge Relationships
///
/// Returns knowledge triples (subject-predicate-object) matching a query
pub async fn query_knowledge_relationships(
    pool: &SqlitePool,
    subject_or_object: &str,
) -> Result<Value, String> {
    let pattern = format!("%{}%", subject_or_object);

    let triples = sqlx::query_as::<_, (String, String, String, f64)>(
        "SELECT subject, predicate, object, confidence
         FROM knowledge_triples
         WHERE subject LIKE ? OR object LIKE ?
         ORDER BY confidence DESC
         LIMIT 50",
    )
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    let results = triples
        .into_iter()
        .map(|(subject, predicate, object, confidence)| {
            json!({
                "subject": subject,
                "predicate": predicate,
                "object": object,
                "confidence": confidence,
                "relationship": format!("{} {} {}", subject, predicate, object)
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "type": "tool_result",
        "name": "query_knowledge_relationships",
        "content": {
            "relationships": results,
            "total": results.len()
        }
    }))
}

/// MCP Tool: Find Knowledge Connections
///
/// Returns how concepts are connected in the knowledge graph
pub async fn find_concept_connections(
    pool: &SqlitePool,
    concept: &str,
    max_hops: usize,
) -> Result<Value, String> {
    let pattern = format!("%{}%", concept);

    // Find direct connections
    let connections = sqlx::query_as::<_, (String, String, f64)>(
        "SELECT from_node, to_node, strength
         FROM connections
         WHERE from_node LIKE ? OR to_node LIKE ?
         ORDER BY strength DESC
         LIMIT ?",
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind((max_hops * 10) as i32)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    let graph = connections
        .iter()
        .map(|(from, to, strength)| {
            json!({
                "from": from,
                "to": to,
                "strength": strength,
                "weight": (strength * 100.0) as i32
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "type": "tool_result",
        "name": "find_concept_connections",
        "content": {
            "concept": concept,
            "connections": graph,
            "total_connections": graph.len()
        }
    }))
}

/// MCP Resource: Learning Sessions
///
/// Returns JeebsAI's learning sessions relevant to a topic
pub async fn get_learning_context(
    pool: &SqlitePool,
    topic: &str,
) -> Result<Value, String> {
    // Query jeebs_store for learning sessions
    let rows = sqlx::query_as::<_, (String, Vec<u8>)>(
        "SELECT key, value FROM jeebs_store WHERE key LIKE 'learnsession:%' LIMIT 50",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    let mut sessions = Vec::new();

    for (_key, value) in rows {
        if let Ok(session_json) = serde_json::from_slice::<Value>(&value) {
            if let (Some(sess_topic), Some(conf), Some(depth), Some(facts)) = (
                session_json.get("topic").and_then(|v| v.as_str()),
                session_json.get("confidence").and_then(|v| v.as_f64()),
                session_json.get("depth_level").and_then(|v| v.as_u64()),
                session_json.get("learned_facts").and_then(|v| v.as_array()),
            ) {
                // Check if topic matches
                if sess_topic.to_lowercase().contains(&topic.to_lowercase()) {
                    let key_facts = facts
                        .iter()
                        .take(5)
                        .filter_map(|f| f.get("fact").and_then(|ff| ff.as_str()))
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>();

                    sessions.push(json!({
                        "topic": sess_topic,
                        "confidence": conf,
                        "depth_level": depth,
                        "facts_learned": facts.len(),
                        "key_facts": key_facts,
                        "status": "completed"
                    }));
                }
            }
        }
    }

    Ok(json!({
        "type": "resource",
        "name": "learning_context",
        "content": {
            "query_topic": topic,
            "sessions": sessions,
            "learning_history": sessions.len()
        }
    }))
}

/// MCP Tool: Brain State Query
///
/// Returns current state of JeebsAI's brain (statistics and metrics)
pub async fn query_brain_state(pool: &SqlitePool) -> Result<Value, String> {
    // Count brain nodes
    let nodes_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM brain_nodes_v2")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    // Count knowledge triples
    let triples_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM knowledge_triples")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    // Count connections
    let connections_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM connections")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    // Get average confidence
    let avg_confidence: Option<(f64,)> =
        sqlx::query_as("SELECT AVG(confidence) FROM knowledge_triples")
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

    let confidence = avg_confidence.map(|(c,)| c).unwrap_or(0.5);

    Ok(json!({
        "type": "tool_result",
        "name": "query_brain_state",
        "content": {
            "brain_metrics": {
                "total_facts": nodes_count.0,
                "knowledge_relationships": triples_count.0,
                "concept_connections": connections_count.0,
                "average_confidence": confidence,
                "knowledge_density": format!("{:.2}%", (confidence * 100.0))
            },
            "timestamp": chrono::Local::now().to_rfc3339()
        }
    }))
}

/// MCP Tool: Retrieve Inference Context
///
/// Returns full context for querying including facts, relationships, and connections
pub async fn get_full_context(
    pool: &SqlitePool,
    query: &str,
) -> Result<Value, String> {
    // Get facts
    let facts = search_brain_facts(pool, query, 10).await?;

    // Get relationships
    let relationships = query_knowledge_relationships(pool, query).await?;

    // Get connections
    let connections = find_concept_connections(pool, query, 2).await?;

    // Get learning
    let learning = get_learning_context(pool, query).await?;

    // Get brain state
    let state = query_brain_state(pool).await?;

    Ok(json!({
        "type": "resource",
        "name": "full_context",
        "query": query,
        "context": {
            "facts": facts.get("content"),
            "relationships": relationships.get("content"),
            "connections": connections.get("content"),
            "learning_sessions": learning.get("content"),
            "brain_state": state.get("content")
        }
    }))
}

/// MCP Instruction: Learn From Interaction
///
/// Records new learning from an interaction for continuous improvement
pub async fn log_interaction_learning(
    pool: &SqlitePool,
    interaction_id: &str,
    topic: &str,
    concepts: Vec<String>,
    confidence: f32,
) -> Result<Value, String> {
    // Store in jeebs_store for later processing
    let learning_record = json!({
        "interaction_id": interaction_id,
        "topic": topic,
        "concepts_learned": concepts,
        "confidence": confidence,
        "timestamp": chrono::Local::now().to_rfc3339(),
        "source": "mcp_interaction"
    });

    let key = format!("interaction_learning:{}", interaction_id);

    sqlx::query(
        "INSERT INTO jeebs_store (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(&key)
    .bind(
        serde_json::to_vec(&learning_record)
            .map_err(|e| format!("Serialization error: {}", e))?,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    Ok(json!({
        "type": "instruction_result",
        "name": "log_interaction_learning",
        "content": {
            "status": "success",
            "recorded": true,
            "interaction_id": interaction_id
        }
    }))
}

/// MCP Integration Error Handler
pub fn handle_mcp_error(error: &str) -> Value {
    json!({
        "type": "error",
        "error": {
            "code": -32603,
            "message": "Internal Error",
            "data": {
                "details": error
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_response_structure() {
        let response = json!({
            "type": "resource",
            "content": {
                "facts": vec![]
            }
        });

        assert_eq!(response["type"], "resource");
        assert!(response["content"]["facts"].is_array());
    }
}

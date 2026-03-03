/// MCP API Endpoints
///
/// Exposes the MCP (Model Context Protocol) server functionality as REST endpoints
/// allowing Claude and external services to query JeebsAI's brain

use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::mcp_server;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct BrainSearchRequest {
    pub query: String,
    #[serde(default)]
    pub max_results: Option<usize>,
}

#[derive(Deserialize)]
pub struct RelationshipQueryRequest {
    pub query: String,
}

#[derive(Deserialize)]
pub struct ConceptConnectionRequest {
    pub concept: String,
    #[serde(default)]
    pub max_hops: Option<usize>,
}

#[derive(Deserialize)]
pub struct InteractionLearningRequest {
    pub topic: String,
    pub concepts: Vec<String>,
    pub confidence: f32,
}

#[derive(Serialize)]
pub struct MCPResponse {
    pub success: bool,
    pub data: serde_json::Value,
}

/// Search brain facts matching a query
#[post("/api/mcp/search-facts")]
pub async fn search_facts(
    data: web::Data<AppState>,
    req: web::Json<BrainSearchRequest>,
) -> impl Responder {
    let max_results = req.max_results.unwrap_or(10);

    match mcp_server::search_brain_facts(&data.db, &req.query, max_results).await {
        Ok(result) => HttpResponse::Ok().json(MCPResponse {
            success: true,
            data: result,
        }),
        Err(e) => {
            eprintln!("[MCP] Search facts error: {}", e);
            HttpResponse::InternalServerError().json(MCPResponse {
                success: false,
                data: json!({
                    "error": e
                }),
            })
        }
    }
}

/// Query knowledge relationships
#[post("/api/mcp/query-relationships")]
pub async fn query_relationships(
    data: web::Data<AppState>,
    req: web::Json<RelationshipQueryRequest>,
) -> impl Responder {
    match mcp_server::query_knowledge_relationships(&data.db, &req.query).await {
        Ok(result) => HttpResponse::Ok().json(MCPResponse {
            success: true,
            data: result,
        }),
        Err(e) => {
            eprintln!("[MCP] Query relationships error: {}", e);
            HttpResponse::InternalServerError().json(MCPResponse {
                success: false,
                data: json!({
                    "error": e
                }),
            })
        }
    }
}

/// Find concept connections in knowledge graph
#[post("/api/mcp/concept-connections")]
pub async fn find_connections(
    data: web::Data<AppState>,
    req: web::Json<ConceptConnectionRequest>,
) -> impl Responder {
    let max_hops = req.max_hops.unwrap_or(2);

    match mcp_server::find_concept_connections(&data.db, &req.concept, max_hops).await {
        Ok(result) => HttpResponse::Ok().json(MCPResponse {
            success: true,
            data: result,
        }),
        Err(e) => {
            eprintln!("[MCP] Find connections error: {}", e);
            HttpResponse::InternalServerError().json(MCPResponse {
                success: false,
                data: json!({
                    "error": e
                }),
            })
        }
    }
}

/// Get learning context for a topic
#[post("/api/mcp/learning-context")]
pub async fn get_learning(
    data: web::Data<AppState>,
    req: web::Json<RelationshipQueryRequest>,
) -> impl Responder {
    match mcp_server::get_learning_context(&data.db, &req.query).await {
        Ok(result) => HttpResponse::Ok().json(MCPResponse {
            success: true,
            data: result,
        }),
        Err(e) => {
            eprintln!("[MCP] Get learning error: {}", e);
            HttpResponse::InternalServerError().json(MCPResponse {
                success: false,
                data: json!({
                    "error": e
                }),
            })
        }
    }
}

/// Get current brain state and statistics
#[get("/api/mcp/brain-state")]
pub async fn brain_state(data: web::Data<AppState>) -> impl Responder {
    match mcp_server::query_brain_state(&data.db).await {
        Ok(result) => HttpResponse::Ok().json(MCPResponse {
            success: true,
            data: result,
        }),
        Err(e) => {
            eprintln!("[MCP] Brain state error: {}", e);
            HttpResponse::InternalServerError().json(MCPResponse {
                success: false,
                data: json!({
                    "error": e
                }),
            })
        }
    }
}

/// Get full context for a query (facts + relationships + connections + learning)
#[post("/api/mcp/full-context")]
pub async fn full_context(
    data: web::Data<AppState>,
    req: web::Json<BrainSearchRequest>,
) -> impl Responder {
    match mcp_server::get_full_context(&data.db, &req.query).await {
        Ok(result) => HttpResponse::Ok().json(MCPResponse {
            success: true,
            data: result,
        }),
        Err(e) => {
            eprintln!("[MCP] Full context error: {}", e);
            HttpResponse::InternalServerError().json(MCPResponse {
                success: false,
                data: json!({
                    "error": e
                }),
            })
        }
    }
}

/// Log interaction learning
#[post("/api/mcp/log-learning")]
pub async fn log_learning(
    data: web::Data<AppState>,
    req: web::Json<InteractionLearningRequest>,
) -> impl Responder {
    let interaction_id = uuid::Uuid::new_v4().to_string();

    match mcp_server::log_interaction_learning(
        &data.db,
        &interaction_id,
        &req.topic,
        req.concepts.clone(),
        req.confidence,
    )
    .await
    {
        Ok(result) => HttpResponse::Ok().json(MCPResponse {
            success: true,
            data: result,
        }),
        Err(e) => {
            eprintln!("[MCP] Log learning error: {}", e);
            HttpResponse::InternalServerError().json(MCPResponse {
                success: false,
                data: json!({
                    "error": e
                }),
            })
        }
    }
}

/// MCP Capabilities endpoint
#[get("/api/mcp/capabilities")]
pub async fn capabilities() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "type": "capabilities",
        "model_context_protocol": {
            "version": "1.0",
            "name": "JeebsAI-MCP",
            "description": "Model Context Protocol server for JeebsAI brain access",
            "resources": [
                {
                    "name": "brain_facts",
                    "description": "Search and retrieve facts from JeebsAI brain",
                    "endpoint": "/api/mcp/search-facts"
                },
                {
                    "name": "learning_context",
                    "description": "Get learning sessions and contextual knowledge",
                    "endpoint": "/api/mcp/learning-context"
                },
                {
                    "name": "full_context",
                    "description": "Get complete context with facts, relationships, and connections",
                    "endpoint": "/api/mcp/full-context"
                },
                {
                    "name": "brain_state",
                    "description": "Query brain statistics and current state",
                    "endpoint": "/api/mcp/brain-state"
                }
            ],
            "tools": [
                {
                    "name": "query_relationships",
                    "description": "Query knowledge relationships and RDF triples",
                    "endpoint": "/api/mcp/query-relationships"
                },
                {
                    "name": "find_connections",
                    "description": "Find how concepts are connected in knowledge graph",
                    "endpoint": "/api/mcp/concept-connections"
                },
                {
                    "name": "log_learning",
                    "description": "Record new learning from interactions",
                    "endpoint": "/api/mcp/log-learning"
                }
            ]
        }
    }))
}

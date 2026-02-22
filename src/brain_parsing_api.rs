use crate::brain_parser::{build_knowledge_graph, BrainParser, ParsedBrainContent};
use crate::state::AppState;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ParseNodeRequest {
    pub node_id: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct QueryGraphRequest {
    pub entity: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ParseResponse {
    pub success: bool,
    pub parsed_content: Option<ParsedBrainContent>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GraphQueryResponse {
    pub success: bool,
    pub results: Vec<String>,
    pub result_count: usize,
    pub query_type: String,
}

#[derive(Debug, Serialize)]
pub struct GraphStatisticsResponse {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub total_categories: usize,
    pub total_entities: usize,
    pub nodes: Vec<NodeSummary>,
}

#[derive(Debug, Serialize)]
pub struct NodeSummary {
    pub id: String,
    pub key: String,
    pub entities_count: usize,
    pub relationships_count: usize,
    pub categories: Vec<String>,
}

/// Parse a single brain node and extract structured information
#[post("/api/brain/parse")]
pub async fn parse_brain_node(
    req: web::Json<ParseNodeRequest>,
    _state: web::Data<AppState>,
) -> impl Responder {
    let parser = BrainParser::new();
    let parsed = parser.parse(req.node_id.clone(), req.key.clone(), req.value.clone());

    HttpResponse::Ok().json(ParseResponse {
        success: true,
        parsed_content: Some(parsed),
        error: None,
    })
}

/// Build a complete knowledge graph from all brain nodes
#[post("/api/brain/graph/build")]
pub async fn build_brain_graph(state: web::Data<AppState>) -> impl Responder {
    let db = &state.db;
    let parser = BrainParser::new();
    match build_knowledge_graph(db, &parser).await {
        Ok(graph) => HttpResponse::Ok().json(json!({
            "success": true,
            "graph_stats": graph.to_json(),
            "node_count": graph.nodes.len(),
            "edge_count": graph.edges.len(),
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": format!("{}", e),
        })),
    }
}

/// Provide a lightweight graph payload suitable for the frontend visualizer (nodes + edges)
#[get("/api/brain/visualize")]
pub async fn visualize(state: web::Data<AppState>) -> impl Responder {
    let db = &state.db;
    let parser = BrainParser::new();
    match build_knowledge_graph(db, &parser).await {
        Ok(graph) => {
            let nodes: Vec<serde_json::Value> = graph
                .nodes
                .iter()
                .map(|(id, node)| {
                    serde_json::json!({
                        "id": id,
                        "label": node.content.original_key.clone(),
                        "title": node.content.original_value.clone(),
                    })
                })
                .collect();

            let edges: Vec<serde_json::Value> = graph
                .edges
                .iter()
                .map(|e| serde_json::json!({ "from": e.from, "to": e.to }))
                .collect();

            HttpResponse::Ok().json(serde_json::json!({ "nodes": nodes, "edges": edges }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

/// Query the knowledge graph by entity
#[post("/api/brain/graph/query/entity")]
pub async fn query_graph_entity(
    req: web::Json<QueryGraphRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Some(entity) = &req.entity {
        let db = &state.db;
        let parser = BrainParser::new();
        match build_knowledge_graph(db, &parser).await {
            Ok(graph) => {
                let results = graph.query_by_entity(entity);
                return HttpResponse::Ok().json(GraphQueryResponse {
                    success: true,
                    result_count: results.len(),
                    results,
                    query_type: "entity".to_string(),
                });
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": format!("{}", e),
                }));
            }
        }
    }

    HttpResponse::BadRequest().json(json!({
        "success": false,
        "error": "entity parameter required",
    }))
}

/// Query the knowledge graph by category
#[post("/api/brain/graph/query/category")]
pub async fn query_graph_category(
    req: web::Json<QueryGraphRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Some(category) = &req.category {
        let db = &state.db;
        let parser = BrainParser::new();
        match build_knowledge_graph(db, &parser).await {
            Ok(graph) => {
                let results = graph.query_by_category(category);
                return HttpResponse::Ok().json(GraphQueryResponse {
                    success: true,
                    result_count: results.len(),
                    results,
                    query_type: "category".to_string(),
                });
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": format!("{}", e),
                }));
            }
        }
    }

    HttpResponse::BadRequest().json(json!({
        "success": false,
        "error": "category parameter required",
    }))
}

/// Get statistics about the complete knowledge graph
#[get("/api/brain/graph/statistics")]
pub async fn get_graph_statistics(state: web::Data<AppState>) -> impl Responder {
    let db = &state.db;
    let parser = BrainParser::new();
    match build_knowledge_graph(db, &parser).await {
        Ok(graph) => {
            let mut nodes_summary = Vec::new();

            for (id, node) in graph.nodes.iter() {
                nodes_summary.push(NodeSummary {
                    id: id.clone(),
                    key: node.content.original_key.clone(),
                    entities_count: node.content.extracted_entities.len(),
                    relationships_count: node.content.relationships.len(),
                    categories: node
                        .content
                        .categories
                        .iter()
                        .map(|c| c.name.clone())
                        .collect(),
                });
            }

            HttpResponse::Ok().json(GraphStatisticsResponse {
                total_nodes: graph.nodes.len(),
                total_edges: graph.edges.len(),
                total_categories: graph.categories.len(),
                total_entities: graph.entity_index.len(),
                nodes: nodes_summary,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": format!("{}", e),
        })),
    }
}

/// Analyze relationships between nodes in the knowledge graph
#[get("/api/brain/graph/relationships")]
pub async fn analyze_relationships(state: web::Data<AppState>) -> impl Responder {
    let db = &state.db;
    let parser = BrainParser::new();
    match build_knowledge_graph(db, &parser).await {
        Ok(graph) => {
            // Collect all relationships
            let mut all_relationships = Vec::new();

            for (_, node) in graph.nodes.iter() {
                for rel in &node.content.relationships {
                    all_relationships.push(json!({
                        "subject": rel.subject,
                        "predicate": rel.predicate,
                        "object": rel.object,
                        "confidence": rel.confidence,
                        "type": format!("{:?}", rel.relationship_type),
                    }));
                }
            }

            HttpResponse::Ok().json(json!({
                "success": true,
                "total_relationships": all_relationships.len(),
                "relationships": all_relationships,
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": format!("{}", e),
        })),
    }
}

/// Get a detailed report on extracted entities across all brain nodes
#[get("/api/brain/graph/entities")]
pub async fn get_entities_report(state: web::Data<AppState>) -> impl Responder {
    let db = &state.db;
    let parser = BrainParser::new();
    match build_knowledge_graph(db, &parser).await {
        Ok(graph) => {
            // Group entities by type
            let mut entities_by_type = std::collections::HashMap::new();

            for (_, node) in graph.nodes.iter() {
                for entity in &node.content.extracted_entities {
                    let type_key = format!("{:?}", entity.entity_type);
                    entities_by_type
                        .entry(type_key)
                        .or_insert_with(Vec::new)
                        .push(json!({
                            "value": entity.value,
                            "confidence": entity.confidence,
                        }));
                }
            }

            HttpResponse::Ok().json(json!({
                "success": true,
                "entities_by_type": entities_by_type,
                "total_entities": graph.entity_index.len(),
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": format!("{}", e),
        })),
    }
}

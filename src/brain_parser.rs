use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};

/// Parser framework for extracting and organizing information from brain nodes
/// Handles structured and unstructured data extraction, relationship inference,
/// and knowledge synthesis

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedBrainContent {
    pub node_id: String,
    pub original_key: String,
    pub original_value: String,
    pub extracted_entities: Vec<Entity>,
    pub relationships: Vec<Relationship>,
    pub topics: Vec<String>,
    pub categories: Vec<Category>,
    pub metadata: BrainMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_type: EntityType,
    pub value: String,
    pub confidence: f64,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Date,
    Concept,
    Technology,
    Product,
    Event,
    Number,
    Definition,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f64,
    pub relationship_type: RelationType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RelationType {
    IsA,      // X is a Y
    PartOf,   // X is part of Y
    Creates,  // X creates Y
    Uses,     // X uses Y
    Knows,    // X knows Y (people)
    Located,  // X is located in Y
    Precedes, // X happens before Y
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub confidence: f64,
    pub subcategories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainMetadata {
    pub source: String,
    pub confidence_overall: f64,
    pub processing_timestamp: String,
    pub word_count: usize,
    pub sentence_count: usize,
    pub language: String,
}

/// Main parser struct for processing brain content
pub struct BrainParser {
    entity_patterns: HashMap<EntityType, Vec<Regex>>,
    relationship_patterns: Vec<(Regex, RelationType)>,
    category_keywords: HashMap<String, Vec<String>>,
}

impl BrainParser {
    /// Create a new brain parser with default patterns
    pub fn new() -> Self {
        Self {
            entity_patterns: Self::build_entity_patterns(),
            relationship_patterns: Self::build_relationship_patterns(),
            category_keywords: Self::build_category_keywords(),
        }
    }

    /// Build regex patterns for entity detection
    fn build_entity_patterns() -> HashMap<EntityType, Vec<Regex>> {
        let mut patterns = HashMap::new();

        // Date patterns
        patterns.insert(
            EntityType::Date,
            vec![
                Regex::new(r"\b(\d{1,2}/\d{1,2}/\d{2,4})\b").unwrap(),
                Regex::new(r"\b(January|February|March|April|May|June|July|August|September|October|November|December)\s+\d{1,2}(?:,?\s+\d{4})?\b").unwrap(),
                // If the simple year regex fails to compile for any reason,
                // fall back to a regex that never matches. `(?!)` is not
                // supported by Rust's regex crate, so use `$^` as a no-match
                // pattern instead.
                Regex::new(r"\b(\d{4})\b").unwrap_or_else(|_| Regex::new(r"$^").unwrap()),
            ],
        );

        // Person patterns (capitalized words)
        patterns.insert(
            EntityType::Person,
            vec![Regex::new(r"\b([A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)\b").unwrap()],
        );

        // Number patterns
        patterns.insert(
            EntityType::Number,
            vec![
                Regex::new(r"\b(\d+(?:,\d{3})*(?:\.\d+)?)\b").unwrap(),
                Regex::new(r"\b(hundred|thousand|million|billion|trillion)\b").unwrap(),
            ],
        );

        // Technology patterns
        patterns.insert(
            EntityType::Technology,
            vec![
                Regex::new(r"\b(Rust|Python|JavaScript|TypeScript|Java|C\+\+|Go|Kotlin|Scala|Haskell|Erlang)\b").unwrap(),
                Regex::new(r"\b(React|Vue|Angular|Node|Express|Django|Flask|Spring|Rails)\b").unwrap(),
                Regex::new(r"\b(AI|ML|NLP|Deep Learning|Machine Learning|Neural Network)\b").unwrap(),
            ],
        );

        patterns
    }

    /// Build relationship detection patterns
    fn build_relationship_patterns() -> Vec<(Regex, RelationType)> {
        vec![
            (
                Regex::new(r"(?i)is\s+a(?:\s+kind\s+of)?").unwrap(),
                RelationType::IsA,
            ),
            (
                Regex::new(r"(?i)(?:is\s+)?part\s+of").unwrap(),
                RelationType::PartOf,
            ),
            (
                Regex::new(r"(?i)(?:creates?|made|builds?)").unwrap(),
                RelationType::Creates,
            ),
            (
                Regex::new(r"(?i)(?:uses?|utilizes?)").unwrap(),
                RelationType::Uses,
            ),
            (
                Regex::new(r"(?i)(?:knows?|knows\s+about)").unwrap(),
                RelationType::Knows,
            ),
            (
                Regex::new(r"(?i)(?:located\s+in|in|at)").unwrap(),
                RelationType::Located,
            ),
            (
                Regex::new(r"(?i)(?:before|after|then)").unwrap(),
                RelationType::Precedes,
            ),
        ]
    }

    /// Build category keyword mappings
    fn build_category_keywords() -> HashMap<String, Vec<String>> {
        let mut keywords = HashMap::new();

        keywords.insert(
            "Technology".to_string(),
            vec![
                "software",
                "hardware",
                "code",
                "program",
                "algorithm",
                "database",
                "api",
                "framework",
                "library",
                "language",
                "compiler",
                "debugger",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        keywords.insert(
            "Science".to_string(),
            vec![
                "experiment",
                "theory",
                "hypothesis",
                "research",
                "study",
                "physics",
                "chemistry",
                "biology",
                "discovery",
                "science",
                "quantum",
                "molecule",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        keywords.insert(
            "Business".to_string(),
            vec![
                "company",
                "business",
                "market",
                "revenue",
                "sales",
                "customer",
                "product",
                "service",
                "enterprise",
                "startup",
                "investment",
                "profit",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        keywords.insert(
            "History".to_string(),
            vec![
                "historical",
                "ancient",
                "medieval",
                "war",
                "revolution",
                "dynasty",
                "civilization",
                "era",
                "period",
                "century",
                "empire",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        keywords.insert(
            "Culture".to_string(),
            vec![
                "art",
                "music",
                "culture",
                "tradition",
                "festival",
                "ceremony",
                "literature",
                "poetry",
                "song",
                "dance",
                "theater",
                "film",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        keywords
    }

    /// Parse brain content and extract structured information
    pub fn parse(&self, node_id: String, key: String, value: String) -> ParsedBrainContent {
        let text = format!("{} {}", key, value);

        let entities = self.extract_entities(&text);
        let relationships = self.extract_relationships(&text, &entities);
        let topics = self.extract_topics(&text);
        let categories = self.infer_categories(&text, &topics);
        let metadata = self.create_metadata(&text);

        ParsedBrainContent {
            node_id,
            original_key: key,
            original_value: value,
            extracted_entities: entities,
            relationships,
            topics,
            categories,
            metadata,
        }
    }

    /// Extract entities from text using pattern matching
    fn extract_entities(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        let mut seen = HashSet::new();

        for (entity_type, patterns) in &self.entity_patterns {
            for pattern in patterns {
                for cap in pattern.captures_iter(text) {
                    if let Some(matched) = cap.get(1) {
                        let value = matched.as_str().to_string();

                        if !seen.contains(&value) && !value.is_empty() {
                            seen.insert(value.clone());
                            entities.push(Entity {
                                entity_type: entity_type.clone(),
                                value,
                                confidence: 0.75,
                                context: Some(self.extract_context(
                                    text,
                                    matched.start(),
                                    matched.end(),
                                )),
                            });
                        }
                    }
                }
            }
        }

        entities
    }

    /// Extract relationships between entities
    fn extract_relationships(&self, text: &str, _entities: &[Entity]) -> Vec<Relationship> {
        let mut relationships = Vec::new();

        // Simple relationship extraction using predicate patterns
        let sentences = self.split_sentences(text);

        for sentence in sentences {
            for (pattern, rel_type) in &self.relationship_patterns {
                if let Some(rel_match) = pattern.find(&sentence) {
                    // Try to find subject and object around the relationship
                    if let (Some(subject), Some(object)) = (
                        self.find_nearest_entity_before(&sentence, rel_match.start()),
                        self.find_nearest_entity_after(&sentence, rel_match.end()),
                    ) {
                        relationships.push(Relationship {
                            subject,
                            predicate: rel_match.as_str().to_string(),
                            object,
                            confidence: 0.65,
                            relationship_type: rel_type.clone(),
                        });
                    }
                }
            }
        }

        relationships
    }

    /// Extract topics from text
    fn extract_topics(&self, text: &str) -> Vec<String> {
        let mut topics = Vec::new();
        let text_lower = text.to_lowercase();

        // Extract from categories
        for category in self.category_keywords.keys() {
            let category_lower = category.to_lowercase();
            if text_lower.contains(&category_lower) {
                topics.push(category.clone());
            }
        }

        // Extract key noun phrases (basic approach)
        let noun_pattern = Regex::new(r"\b([A-Z][a-z]+(?:\s+[A-Z][a-z]+)?)\b").unwrap();
        for cap in noun_pattern.captures_iter(text) {
            if let Some(phrase) = cap.get(1) {
                let topic = phrase.as_str();
                if !topics.contains(&topic.to_string()) && topic.len() > 3 {
                    topics.push(topic.to_string());
                }
            }
        }

        topics
    }

    /// Infer categories based on content and topics
    fn infer_categories(&self, text: &str, topics: &[String]) -> Vec<Category> {
        let mut categories = Vec::new();
        let text_lower = text.to_lowercase();

        for (category_name, keywords) in &self.category_keywords {
            let keyword_matches = keywords
                .iter()
                .filter(|kw| text_lower.contains(kw.as_str()))
                .count();

            if keyword_matches > 0 {
                let confidence = (keyword_matches as f64 / keywords.len() as f64).min(1.0);

                let subcategories = topics
                    .iter()
                    .filter(|topic: &&String| {
                        !categories.iter().any(|c: &Category| &c.name == *topic)
                    })
                    .take(3)
                    .cloned()
                    .collect();

                categories.push(Category {
                    name: category_name.clone(),
                    confidence,
                    subcategories,
                });
            }
        }

        categories
    }

    /// Create metadata about the parsed content
    fn create_metadata(&self, text: &str) -> BrainMetadata {
        let word_count = text.split_whitespace().count();
        let sentence_count = self.split_sentences(text).len();

        BrainMetadata {
            source: "brain_parser".to_string(),
            confidence_overall: 0.70,
            processing_timestamp: chrono::Local::now().to_rfc3339(),
            word_count,
            sentence_count,
            language: "English".to_string(), // Could be improved with language detection
        }
    }

    // Helper functions
    fn split_sentences(&self, text: &str) -> Vec<String> {
        let sentence_pattern = Regex::new(r"[.!?]+").unwrap();
        sentence_pattern
            .split(text)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn extract_context(&self, text: &str, start: usize, end: usize) -> String {
        let context_range = 50;
        let start_pos = start.saturating_sub(context_range);
        let end_pos = (end + context_range).min(text.len());

        text[start_pos..end_pos].to_string()
    }

    fn find_nearest_entity_before(&self, text: &str, position: usize) -> Option<String> {
        text[..position.min(text.len())]
            .split_whitespace()
            .last()
            .map(|s| s.to_string())
    }

    fn find_nearest_entity_after(&self, text: &str, position: usize) -> Option<String> {
        text[position.min(text.len())..]
            .split_whitespace()
            .next()
            .map(|s| s.to_string())
    }
}

/// Analyze all brain nodes and build a comprehensive knowledge graph
pub async fn build_knowledge_graph(
    db: &SqlitePool,
    parser: &BrainParser,
) -> Result<KnowledgeGraph, String> {
    let brain_nodes: Vec<(String, String, String)> =
        sqlx::query_as("SELECT id, key, value FROM brain")
            .fetch_all(db)
            .await
            .map_err(|e| format!("Failed to fetch brain nodes: {}", e))?;

    let mut graph = KnowledgeGraph::new();

    for (id, key, value) in brain_nodes {
        let parsed = parser.parse(id, key, value);
        graph.add_parsed_content(parsed);
    }

    Ok(graph)
}

/// Knowledge graph structure for organizing parsed brain content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: HashMap<String, GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub categories: HashMap<String, Vec<String>>, // category -> node_ids
    pub entity_index: HashMap<String, Vec<String>>, // entity_value -> node_ids
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub content: ParsedBrainContent,
    pub related_nodes: Vec<String>,
    pub similarity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub relationship_type: RelationType,
    pub strength: f64,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            categories: HashMap::new(),
            entity_index: HashMap::new(),
        }
    }

    pub fn add_parsed_content(&mut self, content: ParsedBrainContent) {
        let node_id = content.node_id.clone();

        // Add categories to index
        for category in &content.categories {
            self.categories
                .entry(category.name.clone())
                .or_insert_with(Vec::new)
                .push(node_id.clone());
        }

        // Add entities to index
        for entity in &content.extracted_entities {
            self.entity_index
                .entry(entity.value.clone())
                .or_insert_with(Vec::new)
                .push(node_id.clone());
        }

        // Create graph node
        let graph_node = GraphNode {
            id: node_id.clone(),
            content,
            related_nodes: Vec::new(),
            similarity_score: 1.0,
        };

        self.nodes.insert(node_id, graph_node);
    }

    pub fn query_by_entity(&self, entity: &str) -> Vec<String> {
        self.entity_index
            .get(entity)
            .map(|ids| ids.clone())
            .unwrap_or_default()
    }

    pub fn query_by_category(&self, category: &str) -> Vec<String> {
        self.categories
            .get(category)
            .map(|ids| ids.clone())
            .unwrap_or_default()
    }

    pub fn to_json(&self) -> Value {
        json!({
            "node_count": self.nodes.len(),
            "edge_count": self.edges.len(),
            "categories": self.categories.keys().collect::<Vec<_>>(),
            "entity_count": self.entity_index.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_extraction() {
        let parser = BrainParser::new();
        let text = "Rust is a programming language created by Mozilla in 2010";
        let entities = parser.extract_entities(text);

        assert!(entities
            .iter()
            .any(|e| e.entity_type == EntityType::Technology));
        assert!(entities
            .iter()
            .any(|e| e.entity_type == EntityType::Date || e.entity_type == EntityType::Number));
    }

    #[test]
    fn test_parse_complete() {
        let parser = BrainParser::new();
        let result = parser.parse(
            "test_1".to_string(),
            "Rust".to_string(),
            "A systems programming language focused on safety and performance".to_string(),
        );

        assert!(!result.extracted_entities.is_empty());
        assert!(!result.categories.is_empty());
    }
}

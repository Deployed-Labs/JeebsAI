// Knowledge Integration Module - Uses learned information in chat responses

use crate::{deep_learning, knowledge_retrieval};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
struct LinkedInsight {
    left: String,
    right: String,
    rationale: String,
}

#[derive(Debug, Clone)]
pub struct EnhancedChatContext {
    pub user_message: String,
    pub detected_topics: Vec<(String, f32)>, // topic and confidence
    pub relevant_learned_facts: Vec<String>,
    pub expertise_areas: Vec<String>,
    pub learning_opportunities: Vec<String>,
}

/// Detect topics mentioned in user message
pub fn detect_topics_in_message(message: &str) -> Vec<(String, f32)> {
    let message_lower = message.to_lowercase();
    let mut detected = Vec::new();

    // Check for mentions of learning topics
    let topics = vec![
        ("rust", "programming language"),
        ("machine learning", "ai and ml"),
        ("database", "data management"),
        ("distributed systems", "systems architecture"),
        ("python", "programming language"),
        ("javascript", "programming language"),
        ("java", "programming language"),
        ("kubernetes", "container orchestration"),
        ("docker", "containerization"),
        ("api", "software interface"),
        ("network", "networking"),
        ("security", "cybersecurity"),
        ("testing", "quality assurance"),
        ("performance", "optimization"),
        ("concurrency", "parallel processing"),
    ];

    for (topic, category) in topics {
        if message_lower.contains(topic) {
            let confidence = if message_lower.matches(topic).count() > 1 {
                0.9
            } else {
                0.7
            };
            detected.push((format!("{} ({})", topic, category), confidence));
        }
    }

    detected
}

/// Build enhanced chat context with learned knowledge
pub async fn build_enhanced_context(
    db: &SqlitePool,
    user_message: &str,
) -> Result<EnhancedChatContext, String> {
    let detected_topics = detect_topics_in_message(user_message);
    let mut relevant_facts = Vec::new();
    let mut expertise_areas = Vec::new();

    // For each detected topic, get relevant learned facts
    for (topic, _) in &detected_topics {
        // Extract base topic name
        let base_topic = topic.split('(').next().unwrap_or(topic).trim();

        // Get facts related to this topic
        let facts =
            deep_learning::get_relevant_facts_for_chat(db, base_topic, user_message).await?;
        for fact in facts {
            relevant_facts.push(fact.fact.clone());
        }

        // Get expertise level
        if let Some(expertise) = deep_learning::get_topic_expertise(db, base_topic).await {
            if expertise.expertise_level > 0 {
                expertise_areas.push(format!(
                    "{} (Level {})",
                    base_topic, expertise.expertise_level
                ));
            }
        }
    }

    // Find learning opportunities (topics mentioned but not learned yet)
    let all_sessions = deep_learning::get_all_learning_sessions(db).await?;
    let learned_topics: Vec<String> = all_sessions.iter().map(|s| s.topic.clone()).collect();

    let learning_opportunities = detected_topics
        .iter()
        .filter(|(topic, _)| !learned_topics.iter().any(|lt| topic.contains(lt)))
        .map(|(topic, _)| format!("Could learn more about: {}", topic))
        .collect();

    Ok(EnhancedChatContext {
        user_message: user_message.to_string(),
        detected_topics,
        relevant_learned_facts: relevant_facts,
        expertise_areas,
        learning_opportunities,
    })
}

/// Split a comma-separated string into a Vec of trimmed tag strings
pub fn to_tags(s: &str) -> Vec<String> {
    s.split(',')
        .filter(|t| !t.is_empty())
        .map(|t| t.trim().to_string())
        .collect()
}

fn tokens_lower(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

fn connection_score(
    a: &knowledge_retrieval::KnowledgeItem,
    b: &knowledge_retrieval::KnowledgeItem,
) -> f32 {
    let mut score = 0.0;
    // Tag overlap
    let a_tags: std::collections::HashSet<_> =
        a.tags.iter().map(|t| t.to_ascii_lowercase()).collect();
    let b_tags: std::collections::HashSet<_> =
        b.tags.iter().map(|t| t.to_ascii_lowercase()).collect();
    let overlap = a_tags.intersection(&b_tags).count() as f32;
    if overlap > 0.0 {
        score += overlap * 1.5;
    }

    // Label/content token overlap
    let a_tokens: std::collections::HashSet<_> =
        tokens_lower(&format!("{} {}", a.label, a.summary))
            .into_iter()
            .collect();
    let b_tokens: std::collections::HashSet<_> =
        tokens_lower(&format!("{} {}", b.label, b.summary))
            .into_iter()
            .collect();
    let word_overlap = a_tokens.intersection(&b_tokens).count() as f32;
    if word_overlap > 0.0 {
        score += word_overlap * 0.8;
    }

    // Category affinity: connecting triples to brain nodes is valuable
    if a.category != b.category {
        score += 0.5;
    }

    score
}

fn build_linked_insights(items: &[knowledge_retrieval::KnowledgeItem]) -> Vec<LinkedInsight> {
    let mut out = Vec::new();
    let mut pairs = Vec::new();

    for (i, a) in items.iter().enumerate() {
        for b in items.iter().skip(i + 1) {
            let score = connection_score(a, b);
            if score >= 1.5 {
                pairs.push((score, a, b));
            }
        }
    }

    pairs.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap_or(std::cmp::Ordering::Equal));
    for (score, a, b) in pairs.into_iter().take(3) {
        let rationale = format!(
            "Linking '{}' ({}) with '{}' ({}) based on shared concepts/tags (score {:.2}).",
            a.label, a.category, b.label, b.category, score
        );
        out.push(LinkedInsight {
            left: a.label.clone(),
            right: b.label.clone(),
            rationale,
        });
    }

    out
}

async fn persist_link_if_new(db: &SqlitePool, left: &str, right: &str, confidence: f32) {
    // Store as a knowledge_triples row with predicate related_to; ignore failures quietly.
    let _ = sqlx::query(
        "INSERT INTO knowledge_triples (subject, predicate, object, confidence, created_at)
         VALUES (?, 'related_to', ?, ?, datetime('now'))",
    )
    .bind(left)
    .bind(right)
    .bind(confidence as f64)
    .execute(db)
    .await;
}

/// Enhance a chat response with learned knowledge
pub async fn enhance_response_with_knowledge(
    db: &SqlitePool,
    original_response: &str,
    user_message: &str,
) -> Result<String, String> {
    let context = build_enhanced_context(db, user_message).await?;
    let retrieval = knowledge_retrieval::retrieve_knowledge(db, user_message, 10)
        .await
        .unwrap_or_else(|_| knowledge_retrieval::RetrievalResult {
            items: Vec::new(),
            total_searched: 0,
            query_terms: Vec::new(),
            synthesized_answer: None,
        });
    let linked = build_linked_insights(&retrieval.items);

    if context.relevant_learned_facts.is_empty() && context.expertise_areas.is_empty() {
        // Even if no learned facts, we might surface a connected insight
        if linked.is_empty() {
            return Ok(original_response.to_string());
        }
    }

    let mut enhanced = original_response.to_string();

    // Add relevant facts if we have expertise in this area
    if !context.relevant_learned_facts.is_empty() {
        let facts_section = format!(
            "\n\n**From my knowledge:**\n{}",
            context
                .relevant_learned_facts
                .iter()
                .take(3)
                .enumerate()
                .map(|(i, fact)| format!("{}. {}", i + 1, fact))
                .collect::<Vec<_>>()
                .join("\n")
        );
        enhanced.push_str(&facts_section);

        // Record that these facts were used
        for topic in context.detected_topics.iter() {
            let base_topic = topic.0.split('(').next().unwrap_or(&topic.0).trim();
            for fact in &context.relevant_learned_facts {
                let _ = deep_learning::record_fact_usage(db, base_topic, fact).await;
            }
        }
    }

    // Add expertise level if relevant
    if !context.expertise_areas.is_empty() {
        let expertise_section = format!(
            "\n\n**My expertise in related areas:**\n{}",
            context
                .expertise_areas
                .iter()
                .enumerate()
                .map(|(i, area)| format!("{}. {}", i + 1, area))
                .collect::<Vec<_>>()
                .join("\n")
        );
        enhanced.push_str(&expertise_section);
    }

    // Add learning opportunity suggestions
    if !context.learning_opportunities.is_empty() && rand::random::<f32>() > 0.7 {
        let learning_section = format!("\n\n*Note: {}*", context.learning_opportunities[0]);
        enhanced.push_str(&learning_section);
    }

    // Add connected insights between knowledge pieces
    if !linked.is_empty() {
        let bullet_points = linked
            .iter()
            .map(|l| format!("• {} ↔ {} — {}", l.left, l.right, l.rationale))
            .take(2)
            .collect::<Vec<_>>()
            .join("\n");

        enhanced.push_str(&format!("\n\n**Connected insight:**\n{}", bullet_points));

        for link in linked.iter().take(3) {
            persist_link_if_new(db, &link.left, &link.right, 0.55).await;
        }
    }

    Ok(enhanced)
}

/// Get a summary of what has been learned
pub async fn get_learning_summary(db: &SqlitePool) -> Result<String, String> {
    let stats = deep_learning::get_learning_stats(db).await?;

    let sessions = stats
        .get("total_learning_sessions")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let hours = stats
        .get("total_study_hours")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let facts = stats
        .get("total_facts_learned")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let confidence = stats
        .get("average_confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let summary = format!(
        "📚 **Learning Summary**\n\
         • Learning sessions: {}\n\
         • Total study hours: {:.1}\n\
         • Facts learned: {}\n\
         • Average confidence: {:.1}%\n",
        sessions,
        hours,
        facts,
        confidence * 100.0
    );

    Ok(summary)
}

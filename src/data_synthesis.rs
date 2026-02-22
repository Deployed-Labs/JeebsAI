use chrono::Local;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};

/// Represents synthesized understanding of Jeebs' knowledge
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeProfile {
    pub total_items: u64,
    pub domains: Vec<DomainSummary>,
    pub knowledge_gaps: Vec<String>,
    pub recent_learnings: Vec<String>,
    pub emerging_patterns: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainSummary {
    pub domain: String,
    pub item_count: u64,
    pub key_topics: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InsightProposal {
    pub proposal_type: String, // "gap_filling", "cross_domain", "emerging_trend"
    pub title: String,
    pub description: String,
    pub basis: Vec<String>, // What data this is based on
    pub urgency: f64,       // 0.0 to 1.0
}

/// Analyze all knowledge and generate actionable insights
pub async fn generate_knowledge_insights(db: &SqlitePool) -> Result<KnowledgeProfile, String> {
    let total = count_total_knowledge_items(db).await?;
    let domains = analyze_knowledge_domains(db).await?;
    let gaps = identify_knowledge_gaps(db, &domains).await?;
    let recent = get_recent_learnings(db, 10).await?;
    let patterns = identify_emerging_patterns(db, &domains).await?;

    Ok(KnowledgeProfile {
        total_items: total,
        domains,
        knowledge_gaps: gaps,
        recent_learnings: recent,
        emerging_patterns: patterns,
        created_at: Local::now().to_rfc3339(),
    })
}

/// Count total knowledge items across all sources
async fn count_total_knowledge_items(db: &SqlitePool) -> Result<u64, String> {
    let brain_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM brain_nodes")
        .fetch_one(db)
        .await
        .map_err(|e| e.to_string())?;

    let triple_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_triples")
        .fetch_one(db)
        .await
        .map_err(|e| e.to_string())?;

    let context_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM jeebs_store WHERE key LIKE 'context:%'")
            .fetch_one(db)
            .await
            .map_err(|e| e.to_string())?;

    Ok((brain_count + triple_count + context_count) as u64)
}

/// Analyze knowledge organized by domain
async fn analyze_knowledge_domains(db: &SqlitePool) -> Result<Vec<DomainSummary>, String> {
    let mut domains = HashMap::new();

    // Extract domains from brain nodes
    let rows = sqlx::query(
        "SELECT json_extract(data, '$.domain') as domain, COUNT(*) as count
         FROM brain_nodes
         WHERE json_extract(data, '$.domain') IS NOT NULL
         GROUP BY domain
         ORDER BY count DESC
         LIMIT 20",
    )
    .fetch_all(db)
    .await
    .map_err(|e| e.to_string())?;

    for row in rows {
        let domain: Option<String> = row.get(0);
        let count: i64 = row.get(1);
        if let Some(d) = domain {
            domains.insert(d, count as u64);
        }
    }

    // Extract from knowledge triples by analyzing subject patterns
    let rows =
        sqlx::query("SELECT subject FROM knowledge_triples ORDER BY created_at DESC LIMIT 100")
            .fetch_all(db)
            .await
            .map_err(|e| e.to_string())?;

    for row in rows {
        let subject: String = row.get(0);
        let category = categorize_subject(&subject);
        *domains.entry(category).or_insert(0) += 1;
    }

    // Convert to domain summaries with key topics
    let mut summaries = Vec::new();
    for (domain, count) in domains.iter().take(15) {
        let key_topics = extract_key_topics(db, domain, 5).await?;
        summaries.push(DomainSummary {
            domain: domain.clone(),
            item_count: *count,
            key_topics,
            confidence: (*count as f64 / 1000.0).min(1.0),
        });
    }

    summaries.sort_by(|a, b| b.item_count.cmp(&a.item_count));
    Ok(summaries)
}

/// Categorize a subject into a domain
fn categorize_subject(subject: &str) -> String {
    let lower = subject.to_lowercase();

    if lower.contains("algorithm") || lower.contains("data structure") || lower.contains("sorting")
    {
        "Computer Science".to_string()
    } else if lower.contains("python")
        || lower.contains("rust")
        || lower.contains("javascript")
        || lower.contains("programming")
    {
        "Programming Languages".to_string()
    } else if lower.contains("machine")
        || lower.contains("neural")
        || lower.contains("model")
        || lower.contains("learning")
    {
        "Machine Learning".to_string()
    } else if lower.contains("quantum") || lower.contains("physics") {
        "Physics".to_string()
    } else if lower.contains("biology") || lower.contains("dna") {
        "Biology".to_string()
    } else if lower.contains("math") || lower.contains("equation") {
        "Mathematics".to_string()
    } else if lower.contains("climate") || lower.contains("weather") {
        "Climate Science".to_string()
    } else if lower.contains("history") || lower.contains("war") {
        "History".to_string()
    } else if lower.contains("web") || lower.contains("internet") || lower.contains("api") {
        "Web Technology".to_string()
    } else {
        "General Knowledge".to_string()
    }
}

/// Extract key topics from a domain
async fn extract_key_topics(
    db: &SqlitePool,
    domain: &str,
    limit: usize,
) -> Result<Vec<String>, String> {
    let mut topics = HashSet::new();

    // Get from brain nodes
    let pattern = format!("%{}%", domain);
    let rows = sqlx::query(
        "SELECT label FROM brain_nodes WHERE json_extract(data, '$.domain') LIKE ? LIMIT 20",
    )
    .bind(&pattern)
    .fetch_all(db)
    .await
    .map_err(|e| e.to_string())?;

    for row in rows {
        let label: String = row.get(0);
        if !label.is_empty() {
            topics.insert(label);
        }
    }

    Ok(topics.into_iter().take(limit).collect())
}

/// Identify knowledge gaps based on what Jeebs knows
async fn identify_knowledge_gaps(
    db: &SqlitePool,
    domains: &[DomainSummary],
) -> Result<Vec<String>, String> {
    let mut gaps = Vec::new();

    // Common important topics that might not be well covered
    let important_areas = vec![
        ("Climate Change", "climate"),
        ("Quantum Computing", "quantum"),
        ("Biotechnology", "biotechnology"),
        ("Renewable Energy", "renewable"),
        ("Space Exploration", "space"),
        ("Cybersecurity", "cybersecurity"),
        ("Ethics in AI", "ethics"),
        ("Pandemic Response", "pandemic"),
    ];

    for (topic, keyword) in important_areas {
        let has_coverage = domains
            .iter()
            .any(|d| d.domain.to_lowercase().contains(keyword))
            || sqlx::query("SELECT 1 FROM brain_nodes WHERE label LIKE ?")
                .bind(format!("%{}%", keyword))
                .fetch_optional(db)
                .await
                .map(|o| o.is_some())
                .unwrap_or(false);

        if !has_coverage {
            gaps.push(format!("Need better coverage on: {}", topic));
        }
    }

    // Check for cross-domain connections
    if gaps.len() < 5 && domains.len() > 1 {
        gaps.push("Opportunity: Establish connections between domains".to_string());
    }

    Ok(gaps)
}

/// Get recently learned items
async fn get_recent_learnings(db: &SqlitePool, limit: usize) -> Result<Vec<String>, String> {
    let rows = sqlx::query("SELECT label FROM brain_nodes ORDER BY created_at DESC LIMIT ?")
        .bind(limit as i32)
        .fetch_all(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let label: String = row.get(0);
            if !label.is_empty() {
                Some(label)
            } else {
                None
            }
        })
        .collect())
}

/// Identify emerging patterns in knowledge
async fn identify_emerging_patterns(
    db: &SqlitePool,
    domains: &[DomainSummary],
) -> Result<Vec<String>, String> {
    let mut patterns = Vec::new();

    // Find rapidly growing domains
    if let Ok(Some(row)) = sqlx::query(
        "SELECT COUNT(*) FROM brain_nodes WHERE created_at > datetime('now', '-7 days')",
    )
    .fetch_optional(db)
    .await
    {
        let recent_count: i64 = row.get(0);
        if recent_count > 50 {
            patterns.push(format!(
                "Rapid knowledge acquisition: {} items in past week",
                recent_count
            ));
        }
    }

    // Find most connected topics
    if let Ok(rows) = sqlx::query("SELECT subject, COUNT(*) as count FROM knowledge_triples GROUP BY subject ORDER BY count DESC LIMIT 5")
        .fetch_all(db)
        .await
    {
        for row in rows {
            let subject: String = row.get(0);
            let count: i64 = row.get(1);
            if count > 5 {
                patterns.push(format!("Highly connected concept: {} (mentioned in {} facts)", subject, count));
            }
        }
    }

    // Identify cross-domain patterns
    if domains.len() > 3 {
        patterns.push(format!(
            "Building diverse knowledge across {} domains",
            domains.len()
        ));
    }

    Ok(patterns)
}

/// Generate insight-based proposals from knowledge
pub async fn generate_insight_proposals(db: &SqlitePool) -> Result<Vec<InsightProposal>, String> {
    let profile = generate_knowledge_insights(db).await?;
    let mut proposals = Vec::new();

    // Gap-filling proposals
    for gap in &profile.knowledge_gaps {
        if gap.contains("coverage") {
            proposals.push(InsightProposal {
                proposal_type: "gap_filling".to_string(),
                title: "Expand Knowledge Coverage".to_string(),
                description: gap.clone(),
                basis: vec!["Knowledge analysis showed gaps in coverage".to_string()],
                urgency: 0.7,
            });
        }
    }

    // Cross-domain synthesis proposals
    if profile.domains.len() > 2 {
        proposals.push(InsightProposal {
            proposal_type: "cross_domain".to_string(),
            title: "Synthesize Cross-Domain Insights".to_string(),
            description: format!(
                "I have knowledge across {} domains. Let me identify connections and synthesize new insights.",
                profile.domains.len()
            ),
            basis: profile.domains.iter().map(|d| d.domain.clone()).collect(),
            urgency: 0.6,
        });
    }

    // Emerging trend proposals
    for pattern in &profile.emerging_patterns {
        if pattern.contains("Rapid") || pattern.contains("Highly connected") {
            proposals.push(InsightProposal {
                proposal_type: "emerging_trend".to_string(),
                title: "Explore Emerging Pattern".to_string(),
                description: pattern.clone(),
                basis: vec!["Pattern detected in recent learning".to_string()],
                urgency: 0.8,
            });
        }
    }

    Ok(proposals)
}

/// Generate contextual understanding for chat responses
pub async fn generate_response_context(db: &SqlitePool, query: &str) -> Result<String, String> {
    let profile = generate_knowledge_insights(db).await?;
    let relevant_domains: Vec<String> = profile
        .domains
        .iter()
        .filter(|d| {
            d.domain.to_lowercase().contains(&query.to_lowercase())
                || d.key_topics
                    .iter()
                    .any(|t| t.to_lowercase().contains(&query.to_lowercase()))
        })
        .map(|d| d.domain.clone())
        .collect();

    if relevant_domains.is_empty() {
        return Ok(String::new());
    }

    let context = format!("Based on my knowledge in {}: ", relevant_domains.join(", "));
    Ok(context)
}

/// Get overall knowledge statistics
pub async fn get_knowledge_summary(db: &SqlitePool) -> Result<String, String> {
    let profile = generate_knowledge_insights(db).await?;

    let mut summary = format!("📚 **Knowledge Summary**\n\n");
    summary.push_str(&format!("Total Knowledge Items: {}\n", profile.total_items));
    summary.push_str(&format!("Domains Covered: {}\n\n", profile.domains.len()));

    summary.push_str("**Key Domains:**\n");
    for domain in profile.domains.iter().take(5) {
        summary.push_str(&format!(
            "• {}: {} items\n",
            domain.domain, domain.item_count
        ));
    }

    if !profile.knowledge_gaps.is_empty() {
        summary.push_str("\n**Knowledge Gaps:\n");
        for gap in profile.knowledge_gaps.iter().take(3) {
            summary.push_str(&format!("• {}\n", gap));
        }
    }

    if !profile.emerging_patterns.is_empty() {
        summary.push_str("\n**Emerging Patterns:**\n");
        for pattern in profile.emerging_patterns.iter().take(3) {
            summary.push_str(&format!("• {}\n", pattern));
        }
    }

    Ok(summary)
}

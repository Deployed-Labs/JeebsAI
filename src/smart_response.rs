/// Smart Response Generator
///
/// Produces concise, focused, intelligent responses by:
/// - Selecting only the most relevant facts
/// - Avoiding verbose filler text
/// - Matching response length to question type
/// - Providing clear, direct answers
/// - Including reasoning without overwhelming

#[derive(Debug, Clone)]
pub struct ResponseConfig {
    pub max_facts: usize,
    pub max_length: usize,
    pub include_reasoning: bool,
    pub include_sources: bool,
    pub tone: String, // "concise", "detailed", "friendly"
}

#[derive(Debug, Clone)]
pub struct SmartResponse {
    pub text: String,
    pub summary: String,
    pub confidence: f32,
    pub sources: Vec<String>,
    pub follow_up_suggestion: Option<String>,
}

/// Generate smart response from facts and context
pub fn generate_smart_response(
    facts: Vec<String>,
    confidence: f32,
    user_intent: &str,
    topic: &str,
    config: &ResponseConfig,
) -> SmartResponse {
    // Select best facts - quality over quantity
    let selected_facts = select_best_facts(&facts, config.max_facts, confidence);

    // Build response based on intent
    let response_text = build_response_text(&selected_facts, user_intent, topic, config);

    // Create summary
    let summary = create_summary(&selected_facts, confidence);

    // Generate follow-up suggestion
    let follow_up = generate_follow_up(topic, user_intent);

    // Check response quality
    let response = SmartResponse {
        text: response_text.clone(),
        summary,
        confidence,
        sources: selected_facts.into_iter().take(2).collect(),
        follow_up_suggestion: follow_up,
    };

    // Validate response isn't nonsense
    if is_valid_response(&response) {
        response
    } else {
        SmartResponse {
            text: format!("I'm still learning about {}. Can you tell me more?", topic),
            summary: "Insufficient knowledge".to_string(),
            confidence: 0.3,
            sources: vec![],
            follow_up_suggestion: Some(format!("Share what you know about {}", topic)),
        }
    }
}

/// Select the best facts - not all of them
fn select_best_facts(facts: &[String], max: usize, confidence: f32) -> Vec<String> {
    // At low confidence, be more selective
    let actual_max = if confidence < 0.5 {
        (max as f32 * 0.5) as usize
    } else if confidence < 0.7 {
        (max as f32 * 0.7) as usize
    } else {
        max
    };

    facts
        .iter()
        .take(actual_max)
        .filter(|f| !is_filler_fact(f))
        .cloned()
        .collect()
}

/// Check if fact is just filler
fn is_filler_fact(fact: &str) -> bool {
    let lower = fact.to_lowercase();
    fact.is_empty()
        || fact.len() < 10
        || lower.contains("i don't know")
        || lower.contains("unclear")
        || lower.contains("not sure")
        || lower.contains("based on my knowledge")
        || lower.starts_with("•")
}

/// Build concise response text
fn build_response_text(
    facts: &[String],
    intent: &str,
    topic: &str,
    config: &ResponseConfig,
) -> String {
    if facts.is_empty() {
        return format!("I'm learning about {}. What can you tell me?", topic);
    }

    let mut response = String::new();

    // Opening based on intent
    let opening = match intent {
        "explain" => format!("{} is important because: ", topic),
        "instruct" => format!("To work with {}, ", topic),
        "clarify" => format!("To clarify about {}: ", topic),
        "explore" => format!("Related to {}: ", topic),
        _ => format!("About {}: ", topic),
    };

    response.push_str(&opening);

    // Add facts concisely
    if facts.len() == 1 {
        response.push_str(&facts[0]);
    } else {
        for (i, fact) in facts.iter().enumerate().take(3) {
            if i > 0 {
                response.push_str(" Also, ");
            }
            // Clean up the fact
            let clean_fact = fact
                .trim_start_matches("•")
                .trim_start_matches("-")
                .trim();
            response.push_str(clean_fact);
            if i < facts.len() - 1 {
                response.push('.');
            }
        }
    }

    // Add reasoning only if explicitly needed
    if config.include_reasoning && facts.len() > 1 {
        response.push_str(" This matters because these concepts are connected.");
    }

    // Truncate if too long
    if response.len() > config.max_length {
        let truncated = response.chars().take(config.max_length).collect::<String>();
        response = format!("{}...", truncated);
    }

    response
}

/// Create concise summary
fn create_summary(facts: &[String], confidence: f32) -> String {
    if facts.is_empty() {
        return "Insufficient data".to_string();
    }

    let confidence_level = if confidence > 0.8 {
        "High"
    } else if confidence > 0.6 {
        "Medium"
    } else {
        "Learning"
    };

    format!(
        "{} confidence from {} sources",
        confidence_level,
        facts.len().min(3)
    )
}

/// Generate natural follow-up suggestion
fn generate_follow_up(topic: &str, intent: &str) -> Option<String> {
    let suggestions = match intent {
        "explain" => vec![
            format!("Why does {} matter?", topic),
            format!("How is {} used?", topic),
            format!("What are examples of {}?", topic),
        ],
        "instruct" => vec![
            format!("What's an example of {} practice?", topic),
            format!("What could go wrong with {}?", topic),
        ],
        "explore" => vec![
            format!("What's related to {}?", topic),
            format!("How does {} compare to alternatives?", topic),
        ],
        _ => vec![format!("What else about {}?", topic)],
    };

    Some(
        suggestions
            .get(rand::random::<usize>() % suggestions.len())
            .cloned()
            .unwrap_or_else(|| format!("Tell me more about {}", topic)),
    )
}

/// Validate response isn't nonsense
fn is_valid_response(response: &SmartResponse) -> bool {
    // Don't accept empty responses
    if response.text.is_empty() || response.text.len() < 10 {
        return false;
    }

    // Don't accept responses that are just filler
    let text_lower = response.text.to_lowercase();
    if text_lower.contains("based on my knowledge regarding")
        || text_lower.contains("i don't have")
        || text_lower.contains("i am not sure")
    {
        return false;
    }

    // Need minimum confidence
    if response.confidence < 0.2 {
        return false;
    }

    // Should have content, not just punctuation
    let content: String = response
        .text
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    content.len() > 20
}

/// Implement response config for different question types
pub fn get_response_config_for_intent(intent: &str) -> ResponseConfig {
    match intent {
        "explain" => ResponseConfig {
            max_facts: 3,
            max_length: 300,
            include_reasoning: true,
            include_sources: true,
            tone: "detailed".to_string(),
        },
        "instruct" => ResponseConfig {
            max_facts: 2,
            max_length: 200,
            include_reasoning: false,
            include_sources: false,
            tone: "concise".to_string(),
        },
        "clarify" => ResponseConfig {
            max_facts: 1,
            max_length: 150,
            include_reasoning: false,
            include_sources: false,
            tone: "concise".to_string(),
        },
        "explore" => ResponseConfig {
            max_facts: 4,
            max_length: 250,
            include_reasoning: true,
            include_sources: true,
            tone: "friendly".to_string(),
        },
        _ => ResponseConfig {
            max_facts: 2,
            max_length: 200,
            include_reasoning: false,
            include_sources: false,
            tone: "friendly".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_generation() {
        let facts = vec!["Machine learning enables computers to learn from data".to_string()];
        let config = get_response_config_for_intent("explain");

        let response = generate_smart_response(facts, 0.85, "explain", "machine learning", &config);

        assert!(!response.text.is_empty());
        assert!(response.confidence > 0.0);
        assert!(is_valid_response(&response));
    }

    #[test]
    fn test_filler_detection() {
        assert!(is_filler_fact("• nothing important"));
        assert!(is_filler_fact("I don't know"));
        assert!(!is_filler_fact("Neural networks are inspired by biological neurons"));
    }
}

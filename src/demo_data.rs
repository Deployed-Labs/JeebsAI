/// Demo Data Module: Bootstrap Facts for Empty CDHSC
///
/// Provides curated, clean facts that initialize JeebsAI's knowledge base
/// when the system starts with an empty CDHSC. These facts cover fundamental
/// topics across technology, science, and general knowledge.

use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct DemoFact {
    pub title: String,
    pub content: String,
    pub keywords: Vec<String>,
    pub category: String,
    pub topic: String,
}

impl DemoFact {
    fn new(title: &str, content: &str, keywords: Vec<&str>, category: &str, topic: &str) -> Self {
        DemoFact {
            title: title.to_string(),
            content: content.to_string(),
            keywords: keywords.into_iter().map(|k| k.to_string()).collect(),
            category: category.to_string(),
            topic: topic.to_string(),
        }
    }

    /// Convert to JSON for database storage
    pub fn to_json(&self) -> Value {
        json!({
            "title": self.title,
            "content": self.content,
            "keywords": self.keywords,
            "category": self.category,
            "topic": self.topic,
            "source": "bootstrap",
            "confidence": 0.95
        })
    }
}

/// Get all demo facts for bootstrap
pub fn get_demo_facts() -> Vec<DemoFact> {
    vec![
        // === Machine Learning & AI ===
        DemoFact::new(
            "Machine Learning",
            "Machine learning is a subset of artificial intelligence where systems learn patterns from data without being explicitly programmed. It uses algorithms to find relationships in data and make predictions or decisions based on those patterns. Common applications include recommendation systems, image recognition, and natural language processing.",
            vec!["AI", "ML", "algorithms", "learning", "data"],
            "Technology",
            "Machine Learning"
        ),
        DemoFact::new(
            "Neural Networks",
            "Neural networks are computing systems inspired by biological neurons in animal brains. They consist of interconnected nodes (neurons) that process information. Neural networks are the foundation of deep learning and excel at tasks like image classification, natural language understanding, and game playing.",
            vec!["neural", "deep learning", "AI", "networks", "brain"],
            "Technology",
            "Machine Learning"
        ),
        DemoFact::new(
            "Supervised Learning",
            "Supervised learning is a machine learning approach where models are trained on labeled data - examples where the correct answer is known. The algorithm learns to map inputs to outputs. Common examples include predicting house prices from features or classifying emails as spam or not spam.",
            vec!["learning", "labeled data", "training", "classification"],
            "Technology",
            "Machine Learning"
        ),

        // === Programming ===
        DemoFact::new(
            "Rust Programming Language",
            "Rust is a systems programming language that emphasizes memory safety, concurrency, and performance. It prevents common bugs like null pointer exceptions without requiring a garbage collector. Rust is used for performance-critical applications, web services, and embedded systems.",
            vec!["Rust", "programming", "systems", "safe", "concurrent"],
            "Technology",
            "Programming"
        ),
        DemoFact::new(
            "Object-Oriented Programming",
            "Object-oriented programming (OOP) is a paradigm where code is organized around objects that contain data and methods. Key concepts include classes, inheritance, encapsulation, and polymorphism. OOP promotes code reuse and makes large systems more maintainable.",
            vec!["OOP", "objects", "classes", "inheritance", "programming"],
            "Technology",
            "Programming"
        ),
        DemoFact::new(
            "Git Version Control",
            "Git is a distributed version control system that tracks changes to code over time. It allows multiple developers to collaborate, branch code, and merge changes. Git is the standard tool for software development and is the foundation of platforms like GitHub.",
            vec!["Git", "version control", "code", "collaboration", "GitHub"],
            "Technology",
            "Programming"
        ),

        // === Computer Science Fundamentals ===
        DemoFact::new(
            "Algorithms",
            "An algorithm is a step-by-step procedure for solving a problem or performing a computation. Good algorithms are efficient in terms of time and space. Algorithm analysis uses Big O notation to describe how complexity grows with input size. Common algorithms include sorting, searching, and graph traversal.",
            vec!["algorithm", "efficiency", "complexity", "Big O", "sorting"],
            "Computer Science",
            "Fundamentals"
        ),
        DemoFact::new(
            "Data Structures",
            "Data structures are ways to organize and store data efficiently. Different structures suit different problems: arrays for sequential access, hash maps for fast lookups, trees for hierarchical data, and graphs for relationships. Choosing the right data structure is crucial for performance.",
            vec!["data structure", "arrays", "hash maps", "trees", "efficiency"],
            "Computer Science",
            "Fundamentals"
        ),
        DemoFact::new(
            "Recursion",
            "Recursion is a programming technique where a function calls itself to solve a problem by breaking it into smaller subproblems. Every recursive function needs a base case to stop recursion and a recursive case that calls itself. Recursion is elegant but can be inefficient if not optimized.",
            vec!["recursion", "functions", "base case", "algorithms", "problem solving"],
            "Computer Science",
            "Fundamentals"
        ),

        // === Web Development ===
        DemoFact::new(
            "REST API",
            "REST (Representational State Transfer) is an architectural style for building web services. It uses HTTP methods (GET, POST, PUT, DELETE) to perform operations on resources identified by URLs. REST APIs are stateless and widely used for web and mobile applications.",
            vec!["REST", "API", "HTTP", "web service", "web development"],
            "Technology",
            "Web"
        ),
        DemoFact::new(
            "HTML",
            "HTML (HyperText Markup Language) is the standard markup language for creating web pages. It uses tags to structure content into elements like headings, paragraphs, links, and forms. HTML provides the foundation, while CSS handles styling and JavaScript adds interactivity.",
            vec!["HTML", "web", "markup", "structure", "web development"],
            "Technology",
            "Web"
        ),
        DemoFact::new(
            "CSS",
            "CSS (Cascading Style Sheets) is used to style and layout web pages. It controls colors, fonts, spacing, and positioning of HTML elements. CSS enables responsive design that adapts to different screen sizes, making websites accessible on mobile and desktop.",
            vec!["CSS", "styling", "web", "layout", "responsive"],
            "Technology",
            "Web"
        ),

        // === Science ===
        DemoFact::new(
            "Photosynthesis",
            "Photosynthesis is the process where plants convert light energy into chemical energy stored in glucose. It occurs in chloroplasts and involves two stages: the light-dependent reactions and the Calvin cycle. Photosynthesis is fundamental to life on Earth, producing oxygen and food energy.",
            vec!["photosynthesis", "plants", "energy", "light", "biology"],
            "Science",
            "Biology"
        ),
        DemoFact::new(
            "Evolution",
            "Evolution is the process of change in all forms of life over generations, primarily through natural selection. Organisms with advantageous traits survive and reproduce more successfully, passing traits to offspring. Evolution explains the diversity of life and is supported by fossil records and genetic evidence.",
            vec!["evolution", "natural selection", "adaptation", "genetics", "life"],
            "Science",
            "Biology"
        ),
        DemoFact::new(
            "DNA",
            "DNA (deoxyribonucleic acid) is the molecule that carries genetic instructions for all living organisms. It's structured as a double helix with base pairs that encode genetic information. DNA is passed from parent to offspring and contains the blueprint for proteins and traits.",
            vec!["DNA", "genetics", "molecules", "heredity", "biology"],
            "Science",
            "Biology"
        ),
        DemoFact::new(
            "Quantum Mechanics",
            "Quantum mechanics is the physics of the very small - atoms and subatomic particles. It describes phenomena like superposition and entanglement that seem counter-intuitive but are fundamental to reality. Quantum mechanics is the basis for modern technology like semiconductors and lasers.",
            vec!["quantum", "physics", "particles", "atoms", "superposition"],
            "Science",
            "Physics"
        ),

        // === General Knowledge ===
        DemoFact::new(
            "Gravity",
            "Gravity is the fundamental force that attracts objects with mass. Earth's gravity pulls objects downward at 9.8 m/s². Newton's law of universal gravitation describes it as proportional to mass and inversely proportional to distance squared. Einstein's general relativity explains gravity as the curvature of spacetime.",
            vec!["gravity", "force", "physics", "Newton", "relativity"],
            "Science",
            "Physics"
        ),
        DemoFact::new(
            "Ecosystems",
            "An ecosystem is a community of living organisms (plants, animals, microbes) interacting with their physical environment. Energy flows through ecosystems from the sun to producers, consumers, and decomposers. Healthy ecosystems provide essential services like clean water, air, and food.",
            vec!["ecosystem", "environment", "biology", "organisms", "nature"],
            "Science",
            "Ecology"
        ),
        DemoFact::new(
            "Climate",
            "Climate is the long-term pattern of weather in a region, determined by factors like latitude, altitude, ocean currents, and greenhouse gases. Climate change refers to long-term shifts, primarily caused by human activity increasing CO2. Climate affects agriculture, water, and ecosystems globally.",
            vec!["climate", "weather", "greenhouse gases", "environment", "global"],
            "Science",
            "Ecology"
        ),

        // === History ===
        DemoFact::new(
            "The Industrial Revolution",
            "The Industrial Revolution (1760-1840) transformed production from manual labor to machinery and factories. It began in Britain and spread globally, powered by innovations in textiles, steam power, and iron. It created modern capitalism, urbanization, and dramatically increased productivity but also social challenges.",
            vec!["industrial", "history", "revolution", "machinery", "technology"],
            "History",
            "Modern History"
        ),
        DemoFact::new(
            "Ancient Rome",
            "Ancient Rome was a powerful civilization that lasted over 1000 years (27 BC - 476 AD). Romans pioneered engineering, law, and governance structures. They built an empire spanning three continents, created infrastructure like aqueducts and roads, and their legal system influenced modern law.",
            vec!["Rome", "ancient", "empire", "history", "civilization"],
            "History",
            "Ancient History"
        ),

        // === Philosophy ===
        DemoFact::new(
            "Critical Thinking",
            "Critical thinking is the ability to analyze information objectively, identify biases, evaluate evidence, and form reasoned conclusions. It involves questioning assumptions, considering alternative viewpoints, and checking logical consistency. Critical thinking is essential for problem-solving and informed decision-making.",
            vec!["critical thinking", "logic", "reasoning", "analysis", "philosophy"],
            "Philosophy",
            "Thinking"
        ),
        DemoFact::new(
            "Ethics",
            "Ethics is the study of right and wrong conduct. It addresses questions about morality, values, and how to live well. Major ethical frameworks include consequentialism (judging by outcomes), deontology (judging by rules), and virtue ethics (judging by character). Ethics guides personal and professional behavior.",
            vec!["ethics", "morality", "values", "philosophy", "conduct"],
            "Philosophy",
            "Thinking"
        ),
    ]
}

/// Check if demo facts should be loaded (i.e., CDHSC is empty)
pub fn should_load_demo_facts(node_count: usize) -> bool {
    node_count == 0
}

/// Insert demo facts into the database
pub async fn insert_demo_facts(db: &sqlx::SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let facts = get_demo_facts();

    for (index, fact) in facts.iter().enumerate() {
        let id = format!("demo_fact_{}", index);
        let json_data = fact.to_json();
        let data_blob = serde_json::to_vec(&json_data)?;

        sqlx::query(
            "INSERT OR IGNORE INTO brain_nodes (id, label, summary, data) VALUES (?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&fact.title)
        .bind(&fact.topic)
        .bind(&data_blob)
        .execute(db)
        .await?;
    }

    println!("Loaded {} demo facts into brain_nodes", facts.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_facts_exist() {
        let facts = get_demo_facts();
        assert!(!facts.is_empty(), "Demo facts should not be empty");
        assert!(facts.len() >= 20, "Should have at least 20 demo facts");
    }

    #[test]
    fn test_fact_structure() {
        let facts = get_demo_facts();
        for fact in facts {
            assert!(!fact.title.is_empty(), "Title should not be empty");
            assert!(!fact.content.is_empty(), "Content should not be empty");
            assert!(!fact.keywords.is_empty(), "Keywords should not be empty");
            assert!(!fact.category.is_empty(), "Category should not be empty");
            assert!(!fact.topic.is_empty(), "Topic should not be empty");
        }
    }

    #[test]
    fn test_fact_json_conversion() {
        let fact = DemoFact::new(
            "Test",
            "Test content",
            vec!["test"],
            "Category",
            "Topic"
        );
        let json = fact.to_json();
        assert_eq!(json["title"], "Test");
        assert_eq!(json["content"], "Test content");
        assert_eq!(json["source"], "bootstrap");
    }
}

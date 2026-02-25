/// Filter out code blocks, headers, and technical metadata from input text.
pub fn filter_knowledge_content(raw: &str) -> String {
    let mut filtered = String::new();
    let mut in_code = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        // Skip code blocks (markdown style)
        if trimmed.starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        // Skip headers (e.g., HTTP headers, markdown headers)
        if trimmed.starts_with("#") || trimmed.ends_with(":") {
            continue;
        }
        // Skip lines that look like import/include statements
        if trimmed.starts_with("import ") || trimmed.starts_with("include ") || trimmed.starts_with("use ") {
            continue;
        }
        // Skip lines that look like function/class definitions
        if trimmed.starts_with("fn ") || trimmed.starts_with("class ") || trimmed.starts_with("def ") {
            continue;
        }
        // Skip lines that are empty or only punctuation
        if trimmed.is_empty() || trimmed.chars().all(|c| !c.is_alphanumeric()) {
            continue;
        }
        filtered.push_str(line);
        filtered.push('\n');
    }
    filtered.trim().to_string()
}

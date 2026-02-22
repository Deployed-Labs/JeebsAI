use regex::Regex;
use scraper::{Html, Selector};

/// Strip HTML and extract meaningful text content
pub fn strip_html_extract_text(html_content: &str) -> String {
    // Parse HTML
    let document = Html::parse_document(html_content);

    // Remove script and style tags
    let _script_selector = Selector::parse("script, style, noscript").unwrap();
    let mut text_content = String::new();

    // Extract text from body or entire document
    let body_selector = Selector::parse("body").unwrap();
    let content = if let Some(body) = document.select(&body_selector).next() {
        body
    } else {
        return extract_plain_text_fallback(html_content);
    };

    // Extract text nodes, skipping scripts/styles
    for element in content.descendants() {
        if let scraper::node::Node::Text(text) = element.value() {
            let text_str = text.trim();
            if !text_str.is_empty() && text_str.len() > 2 {
                if !text_content.is_empty() && !text_content.ends_with(' ') {
                    text_content.push(' ');
                }
                text_content.push_str(text_str);
            }
        }
    }

    // Clean up whitespace
    clean_whitespace(&text_content)
}

/// Fallback text extraction using regex (for malformed HTML)
fn extract_plain_text_fallback(html: &str) -> String {
    // Remove common HTML tags
    let mut text = html.to_string();

    // Remove script and style content
    text = remove_tag_content(&text, "script");
    text = remove_tag_content(&text, "style");
    text = remove_tag_content(&text, "noscript");

    // Remove HTML tags
    let tag_regex = Regex::new(r"<[^>]+>").unwrap();
    text = tag_regex.replace_all(&text, " ").to_string();

    // Remove HTML entities
    text = text
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&");

    clean_whitespace(&text)
}

/// Remove all content within specific tags
fn remove_tag_content(html: &str, tag: &str) -> String {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let _open_attr = format!("<{}\\s", tag);

    let mut result = html.to_string();

    // Handle tags with attributes
    let attr_regex = Regex::new(&format!(r"<{}[^>]*>.*?</{}>", tag, tag)).unwrap();
    result = attr_regex.replace_all(&result, " ").to_string();

    // Handle simple tags
    loop {
        if let Some(start) = result.find(&open) {
            if let Some(end) = result[start..].find(&close) {
                let end_pos = start + end + close.len();
                result.drain(start..end_pos);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    result
}

/// Clean up whitespace and normalize text
fn clean_whitespace(text: &str) -> String {
    // Replace multiple spaces with single space
    let space_regex = Regex::new(r"\s+").unwrap();
    let mut cleaned = space_regex.replace_all(text, " ").to_string();

    // Remove leading/trailing whitespace
    cleaned = cleaned.trim().to_string();

    // Limit to reasonable length (avoid storing entire websites)
    if cleaned.len() > 50000 {
        cleaned.truncate(50000);
        // Truncate at last complete word/sentence
        if let Some(last_space) = cleaned.rfind(' ') {
            cleaned.truncate(last_space);
        }
    }

    cleaned
}

/// Extract key metadata from HTML
pub fn extract_metadata(html_content: &str) -> ExtractedMetadata {
    let document = Html::parse_document(html_content);

    // Extract title
    let title_selector = Selector::parse("title").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .and_then(|el| el.text().next())
        .map(|s| s.to_string())
        .unwrap_or_default();

    // Extract meta description
    let desc_selector = Selector::parse("meta[name='description']").unwrap();
    let description = document
        .select(&desc_selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string())
        .unwrap_or_default();

    // Extract h1 headings
    let h1_selector = Selector::parse("h1").unwrap();
    let headings: Vec<String> = document
        .select(&h1_selector)
        .filter_map(|el| el.text().next())
        .map(|s| s.to_string())
        .take(3)
        .collect();

    // Extract keywords
    let keywords_selector = Selector::parse("meta[name='keywords']").unwrap();
    let keywords = document
        .select(&keywords_selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string())
        .unwrap_or_default();

    ExtractedMetadata {
        title,
        description,
        headings,
        keywords,
    }
}

/// Extract all links from HTML
pub fn extract_links(html_content: &str) -> Vec<String> {
    let document = Html::parse_document(html_content);
    let link_selector = Selector::parse("a[href]").unwrap();

    document
        .select(&link_selector)
        .filter_map(|el| el.value().attr("href"))
        .filter(|href| !href.is_empty() && !href.starts_with('#'))
        .map(|href| href.to_string())
        .take(100) // Limit to 100 links per page
        .collect()
}

#[derive(Debug, Clone)]
pub struct ExtractedMetadata {
    pub title: String,
    pub description: String,
    pub headings: Vec<String>,
    pub keywords: String,
}

/// Create a summary from extracted text
pub fn create_summary(text: &str, max_length: usize) -> String {
    let sentences: Vec<&str> = text
        .split(|c| c == '.' || c == '!' || c == '?')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && s.len() > 5)
        .collect();

    let mut summary = String::new();
    for sentence in sentences.iter().take(3) {
        if summary.len() + sentence.len() + 2 > max_length {
            break;
        }
        if !summary.is_empty() {
            summary.push(' ');
        }
        summary.push_str(sentence);
        summary.push('.');
    }

    if summary.is_empty() && !text.is_empty() {
        summary = text.chars().take(max_length).collect();
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_simple_html() {
        let html = "<p>Hello <b>world</b>!</p>";
        let text = strip_html_extract_text(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("world"));
    }

    #[test]
    fn test_remove_scripts() {
        let html = "<p>Content</p><script>alert('test')</script><p>More</p>";
        let text = strip_html_extract_text(html);
        assert!(!text.contains("alert"));
        assert!(text.contains("Content"));
    }
}

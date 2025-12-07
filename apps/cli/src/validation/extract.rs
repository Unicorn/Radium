//! Source URI extraction from text.

use regex::Regex;

/// Extracts source URIs from text content.
///
/// Looks for:
/// - file:// URIs
/// - http:// and https:// URLs
/// - jira:// URIs
/// - braingrid:// URIs
/// - Relative file paths (common patterns like ./file.md, ../file.md, file.md)
pub fn extract_sources(text: &str) -> Vec<String> {
    let mut sources = Vec::new();

    // Pattern for explicit URI schemes
    let uri_patterns = [
        (r"file://[^\s\)\]\}]+", "file://"),
        (r"https?://[^\s\)\]\}]+", "http://"),
        (r"jira://[^\s\)\]\}]+", "jira://"),
        (r"braingrid://[^\s\)\]\}]+", "braingrid://"),
    ];

    for (pattern, _scheme) in &uri_patterns {
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.find_iter(text) {
                sources.push(cap.as_str().to_string());
            }
        }
    }

    // Pattern for relative file paths (common markdown/file reference patterns)
    // Match patterns like: ./file.md, ../file.md, file.md, path/to/file.md
    // But avoid matching URLs, code blocks, or other non-file references
    let file_patterns = [
        r"\./[^\s\)\]\}:]+\.[a-zA-Z0-9]+",  // ./file.md
        r"\.\./[^\s\)\]\}:]+\.[a-zA-Z0-9]+", // ../file.md
        r"(?m)^[^\s:]+\.(md|txt|rs|py|js|ts|json|toml|yaml|yml)$", // file.md at start of line
        r"`([^\s`]+\.(md|txt|rs|py|js|ts|json|toml|yaml|yml))`", // `file.md` in code backticks
    ];

    for pattern in &file_patterns {
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.captures_iter(text) {
                let path = cap.get(1).map(|m| m.as_str()).unwrap_or(cap.get(0).unwrap().as_str());
                // Filter out common non-file patterns
                if !path.contains("://") && !path.starts_with("http") {
                    sources.push(path.to_string());
                }
            }
        }
    }

    // Remove duplicates
    sources.sort();
    sources.dedup();

    sources
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_file_uris() {
        let text = "See file:///path/to/file.md and file:///another/file.txt";
        let sources = extract_sources(text);
        assert!(sources.contains(&"file:///path/to/file.md".to_string()));
        assert!(sources.contains(&"file:///another/file.txt".to_string()));
    }

    #[test]
    fn test_extract_http_urls() {
        let text = "Check https://example.com/doc.md and http://test.com/page";
        let sources = extract_sources(text);
        assert!(sources.iter().any(|s| s.contains("https://example.com")));
        assert!(sources.iter().any(|s| s.contains("http://test.com")));
    }

    #[test]
    fn test_extract_jira_uris() {
        let text = "Related to jira://PROJ-123 and jira://PROJ-456";
        let sources = extract_sources(text);
        assert!(sources.contains(&"jira://PROJ-123".to_string()));
        assert!(sources.contains(&"jira://PROJ-456".to_string()));
    }

    #[test]
    fn test_extract_relative_paths() {
        let text = "See ./file.md and ../parent/file.txt";
        let sources = extract_sources(text);
        assert!(sources.iter().any(|s| s.contains("./file.md")));
    }
}

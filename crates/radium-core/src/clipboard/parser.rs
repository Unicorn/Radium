//! Clipboard format parsing and language detection.

use regex::Regex;

/// Parsed clipboard content with metadata
#[derive(Debug, Clone)]
pub struct ParsedClipboard {
    /// The code content (without annotations)
    pub content: String,
    /// File path if annotated
    pub file_path: Option<String>,
    /// Detected language
    pub language: Option<String>,
}

/// Parse file path annotation from clipboard content
///
/// Supports formats:
/// - `// File: path/to/file.rs` (C-style)
/// - `# File: path/to/file.ext` (Python/Ruby/Shell)
/// - `<!-- File: path/to/file.ext -->` (HTML/XML)
pub fn parse_file_annotation(content: &str) -> Option<String> {
    // C-style comment: // File: path
    if let Some(captures) = Regex::new(r"//\s*File:\s*(.+)")
        .ok()
        .and_then(|re| re.captures(content))
    {
        return Some(captures.get(1)?.as_str().trim().to_string());
    }
    
    // Hash comment: # File: path
    if let Some(captures) = Regex::new(r"#\s*File:\s*(.+)")
        .ok()
        .and_then(|re| re.captures(content))
    {
        return Some(captures.get(1)?.as_str().trim().to_string());
    }
    
    // HTML/XML comment: <!-- File: path -->
    if let Some(captures) = Regex::new(r"<!--\s*File:\s*(.+?)\s*-->")
        .ok()
        .and_then(|re| re.captures(content))
    {
        return Some(captures.get(1)?.as_str().trim().to_string());
    }
    
    None
}

/// Detect language from code patterns or file extension
pub fn detect_language(content: &str, file_path: Option<&str>) -> Option<String> {
    // If file path provided, detect from extension
    if let Some(path) = file_path {
        if let Some(ext) = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
        {
            return Some(ext.to_string());
        }
    }
    
    // Detect from shebang
    if content.starts_with("#!") {
        if content.contains("python") {
            return Some("python".to_string());
        }
        if content.contains("bash") || content.contains("sh") {
            return Some("shell".to_string());
        }
        if content.contains("node") {
            return Some("javascript".to_string());
        }
    }
    
    // Detect from common patterns
    if content.contains("fn main()") || content.contains("use ") {
        return Some("rust".to_string());
    }
    if content.contains("function") && content.contains("=>") {
        return Some("javascript".to_string());
    }
    if content.contains("def ") && content.contains("import ") {
        return Some("python".to_string());
    }
    if content.contains("package ") && content.contains("func ") {
        return Some("go".to_string());
    }
    
    None
}

/// Parse clipboard content and extract code with metadata
pub fn parse_clipboard(content: &str) -> ParsedClipboard {
    let file_path = parse_file_annotation(content);
    
    // Remove annotation lines to get clean code
    let mut clean_content = content.to_string();
    
    // Remove C-style annotation
    clean_content = Regex::new(r"//\s*File:.*")
        .unwrap_or_else(|_| Regex::new("$^").unwrap())
        .replace_all(&clean_content, "")
        .to_string();
    
    // Remove hash annotation
    clean_content = Regex::new(r"#\s*File:.*")
        .unwrap_or_else(|_| Regex::new("$^").unwrap())
        .replace_all(&clean_content, "")
        .to_string();
    
    // Remove HTML/XML annotation
    clean_content = Regex::new(r"<!--\s*File:.*?\s*-->")
        .unwrap_or_else(|_| Regex::new("$^").unwrap())
        .replace_all(&clean_content, "")
        .to_string();
    
    clean_content = clean_content.trim().to_string();
    
    let language = detect_language(&clean_content, file_path.as_deref());
    
    ParsedClipboard {
        content: clean_content,
        file_path,
        language,
    }
}

/// Format code for clipboard with file path annotation
pub fn format_for_clipboard(
    code: &str,
    file_path: Option<&str>,
    language: Option<&str>,
) -> String {
    let mut output = String::new();
    
    if let Some(path) = file_path {
        // Use appropriate comment style based on language
        let comment = match language {
            Some("rust") | Some("c") | Some("cpp") | Some("java") | Some("javascript") | Some("typescript") => {
                format!("// File: {}\n", path)
            }
            Some("python") | Some("ruby") | Some("shell") | Some("yaml") | Some("perl") => {
                format!("# File: {}\n", path)
            }
            Some("html") | Some("xml") => {
                format!("<!-- File: {} -->\n", path)
            }
            _ => {
                format!("// File: {}\n", path)
            }
        };
        output.push_str(&comment);
    }
    
    output.push_str(code);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_annotation_c_style() {
        let content = "// File: src/main.rs\nfn main() {}";
        assert_eq!(parse_file_annotation(content), Some("src/main.rs".to_string()));
    }

    #[test]
    fn test_parse_file_annotation_hash() {
        let content = "# File: main.py\ndef main(): pass";
        assert_eq!(parse_file_annotation(content), Some("main.py".to_string()));
    }

    #[test]
    fn test_detect_language_from_extension() {
        let detected = detect_language("some code", Some("test.rs"));
        assert_eq!(detected, Some("rs".to_string()));
    }

    #[test]
    fn test_detect_language_from_shebang() {
        let content = "#!/usr/bin/env python3\nprint('hello')";
        assert_eq!(detect_language(content, None), Some("python".to_string()));
    }

    #[test]
    fn test_parse_clipboard_with_annotation() {
        let content = "// File: test.rs\nfn main() {}";
        let parsed = parse_clipboard(content);
        assert_eq!(parsed.file_path, Some("test.rs".to_string()));
        assert!(parsed.content.contains("fn main()"));
    }
}


//! Parser for extracting code blocks from markdown text.

use super::CodeBlock;
use std::path::PathBuf;

/// Parser for extracting code blocks from markdown text.
pub struct CodeBlockParser;

impl CodeBlockParser {
    /// Parses markdown text and extracts all code blocks.
    ///
    /// Code blocks are identified by triple backtick fences: ```language\n...\n```
    /// Blocks are assigned sequential indices starting from 1.
    ///
    /// # Arguments
    /// * `markdown` - The markdown text to parse
    ///
    /// # Returns
    /// Vector of extracted code blocks with indices, languages, and content.
    pub fn parse(markdown: &str) -> Vec<CodeBlock> {
        let mut blocks = Vec::new();
        let mut chars = markdown.char_indices().peekable();
        let mut line_number = 1;
        let mut current_index = 1;

        while let Some((_pos, ch)) = chars.next() {
            // Look for opening fence: ``` at line start
            if ch == '`' {
                if let Some((_, '`')) = chars.next() {
                    if let Some((_, '`')) = chars.next() {
                        // Found opening ```
                        let start_line = line_number;
                        
                        // Extract language from fence line
                        let mut language = String::new();
                        let mut after_backticks = String::new();
                        
                        // Read until newline
                        while let Some((_, c)) = chars.next() {
                            if c == '\n' {
                                line_number += 1;
                                break;
                            }
                            after_backticks.push(c);
                        }
                        
                        // Extract language (trim whitespace)
                        let lang_part = after_backticks.trim();
                        if !lang_part.is_empty() {
                            language = lang_part.to_string();
                        }
                        
                        // Accumulate block content until closing fence
                        let mut content = String::new();
                        let mut found_closing = false;
                        
                        while let Some((_, c)) = chars.next() {
                            if c == '\n' {
                                line_number += 1;
                                content.push(c);
                            } else if c == '`' {
                                // Check if this is closing fence
                                let mut peek_chars = chars.clone();
                                if peek_chars.next().map(|(_, c)| c) == Some('`') {
                                    if peek_chars.next().map(|(_, c)| c) == Some('`') {
                                        // Found closing ```
                                        // Consume the closing backticks
                                        chars.next(); // `
                                        chars.next(); // `
                                        
                                        // Check if there's anything after closing fence on same line
                                        let mut after_fence = String::new();
                                        while let Some((_, c)) = chars.peek() {
                                            if *c == '\n' {
                                                break;
                                            }
                                            after_fence.push(*c);
                                            chars.next();
                                        }
                                        
                                        found_closing = true;
                                        break;
                                    }
                                }
                                content.push(c);
                            } else {
                                content.push(c);
                            }
                        }
                        
                        // Trim trailing newline from content
                        let content = content.trim_end().to_string();
                        
                        // Create block (even if closing fence missing - handle gracefully)
                        let file_hint = Self::detect_file_hints(&content);
                        let block = CodeBlock {
                            index: current_index,
                            language: if language.is_empty() { None } else { Some(language) },
                            content,
                            file_hint,
                            start_line,
                        };
                        
                        blocks.push(block);
                        current_index += 1;
                        
                        if !found_closing {
                            // Warn about malformed block (but still capture it)
                            // This is handled gracefully - block is still stored
                        }
                    }
                }
            } else if ch == '\n' {
                line_number += 1;
            }
        }

        blocks
    }

    /// Extracts language identifier from a code fence line.
    ///
    /// # Arguments
    /// * `fence` - The fence line (e.g., "```rust" or "```python")
    ///
    /// # Returns
    /// Language identifier if present, None otherwise.
    pub fn extract_language(fence: &str) -> Option<String> {
        let trimmed = fence.trim();
        if trimmed.starts_with("```") {
            let lang = trimmed[3..].trim();
            if lang.is_empty() {
                None
            } else {
                Some(lang.to_string())
            }
        } else {
            None
        }
    }

    /// Detects file path hints from code block content.
    ///
    /// Looks for common patterns like:
    /// - `// path/to/file.rs` (Rust/C/C++ style)
    /// - `# path/to/file.py` (Python/Ruby style)
    /// - `// file: path/to/file.ts` (explicit file annotation)
    ///
    /// # Arguments
    /// * `content` - The code block content to scan
    ///
    /// # Returns
    /// Detected file path if found, None otherwise.
    pub fn detect_file_hints(content: &str) -> Option<PathBuf> {
        // Check first few lines for file hints
        for line in content.lines().take(5) {
            let trimmed = line.trim();
            
            // Pattern: // file: path/to/file.rs
            if let Some(file_part) = trimmed.strip_prefix("// file:") {
                let path = file_part.trim();
                if !path.is_empty() {
                    return Some(PathBuf::from(path));
                }
            }
            
            // Pattern: # file: path/to/file.py
            if let Some(file_part) = trimmed.strip_prefix("# file:") {
                let path = file_part.trim();
                if !path.is_empty() {
                    return Some(PathBuf::from(path));
                }
            }
            
            // Pattern: // path/to/file.rs (simple comment with path)
            if trimmed.starts_with("//") && !trimmed.starts_with("///") {
                let comment_content = trimmed[2..].trim();
                // Check if it looks like a path (contains / or \)
                if (comment_content.contains('/') || comment_content.contains('\\'))
                    && !comment_content.starts_with("http")
                    && comment_content.len() < 200
                {
                    // Try to extract path
                    let parts: Vec<&str> = comment_content.split_whitespace().collect();
                    if let Some(first_part) = parts.first() {
                        if first_part.contains('/') || first_part.contains('\\') {
                            return Some(PathBuf::from(*first_part));
                        }
                    }
                }
            }
            
            // Pattern: # path/to/file.py (Python/Ruby style)
            if trimmed.starts_with("#") && !trimmed.starts_with("##") {
                let comment_content = trimmed[1..].trim();
                if (comment_content.contains('/') || comment_content.contains('\\'))
                    && !comment_content.starts_with("http")
                    && comment_content.len() < 200
                {
                    let parts: Vec<&str> = comment_content.split_whitespace().collect();
                    if let Some(first_part) = parts.first() {
                        if first_part.contains('/') || first_part.contains('\\') {
                            return Some(PathBuf::from(*first_part));
                        }
                    }
                }
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_block() {
        let markdown = r#"Here's some code:

```rust
fn main() {
    println!("Hello");
}
```

That's it."#;
        
        let blocks = CodeBlockParser::parse(markdown);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].index, 1);
        assert_eq!(blocks[0].language, Some("rust".to_string()));
        assert!(blocks[0].content.contains("fn main()"));
    }

    #[test]
    fn test_parse_multiple_blocks() {
        let markdown = r#"First block:

```rust
fn main() {}
```

Second block:

```python
print("Hello")
```

Third block:

```typescript
console.log("Hi");
```"#;
        
        let blocks = CodeBlockParser::parse(markdown);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].index, 1);
        assert_eq!(blocks[0].language, Some("rust".to_string()));
        assert_eq!(blocks[1].index, 2);
        assert_eq!(blocks[1].language, Some("python".to_string()));
        assert_eq!(blocks[2].index, 3);
        assert_eq!(blocks[2].language, Some("typescript".to_string()));
    }

    #[test]
    fn test_parse_block_without_language() {
        let markdown = r#"Code block:

```
just plain text
```"#;
        
        let blocks = CodeBlockParser::parse(markdown);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].index, 1);
        assert_eq!(blocks[0].language, None);
        assert_eq!(blocks[0].content, "just plain text");
    }

    #[test]
    fn test_parse_empty_response() {
        let markdown = "No code blocks here.";
        let blocks = CodeBlockParser::parse(markdown);
        assert_eq!(blocks.len(), 0);
    }

    #[test]
    fn test_parse_malformed_block() {
        let markdown = r#"Code block:

```rust
fn main() {
    println!("Hello");
}
// Missing closing fence"#;
        
        let blocks = CodeBlockParser::parse(markdown);
        // Should still capture the block even without closing fence
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].content.contains("fn main()"));
    }

    #[test]
    fn test_extract_language() {
        assert_eq!(
            CodeBlockParser::extract_language("```rust"),
            Some("rust".to_string())
        );
        assert_eq!(
            CodeBlockParser::extract_language("```python"),
            Some("python".to_string())
        );
        assert_eq!(
            CodeBlockParser::extract_language("```"),
            None
        );
        assert_eq!(
            CodeBlockParser::extract_language("```  "),
            None
        );
    }

    #[test]
    fn test_detect_file_hints() {
        // Rust style comment
        let content = "// src/main.rs\nfn main() {}";
        assert_eq!(
            CodeBlockParser::detect_file_hints(content),
            Some(PathBuf::from("src/main.rs"))
        );
        
        // Python style comment
        let content = "# app/models.py\nclass Model: pass";
        assert_eq!(
            CodeBlockParser::detect_file_hints(content),
            Some(PathBuf::from("app/models.py"))
        );
        
        // Explicit file annotation
        let content = "// file: path/to/file.ts\nconst x = 1;";
        assert_eq!(
            CodeBlockParser::detect_file_hints(content),
            Some(PathBuf::from("path/to/file.ts"))
        );
        
        // No file hint
        let content = "fn main() {}";
        assert_eq!(CodeBlockParser::detect_file_hints(content), None);
    }

    #[test]
    fn test_parse_large_response() {
        // Create markdown with 50 code blocks
        let mut markdown = String::new();
        for i in 1..=50 {
            markdown.push_str(&format!("Block {}:\n\n```rust\nfn block_{}() {{}}\n```\n\n", i, i));
        }
        
        let start = std::time::Instant::now();
        let blocks = CodeBlockParser::parse(&markdown);
        let elapsed = start.elapsed();
        
        assert_eq!(blocks.len(), 50);
        // Performance check: should parse 50 blocks in <100ms
        assert!(elapsed.as_millis() < 100, "Parsing took {}ms, expected <100ms", elapsed.as_millis());
    }
}


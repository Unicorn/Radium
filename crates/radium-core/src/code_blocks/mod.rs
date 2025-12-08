//! Code block management system.
//!
//! Provides parsing, storage, and manipulation of code blocks extracted from
//! agent responses. Code blocks are automatically detected from markdown,
//! indexed, and stored for easy retrieval and manipulation.

mod error;
mod parser;
mod store;

pub use error::{CodeBlockError, Result};
pub use parser::CodeBlockParser;
pub use store::{BlockSelector, CodeBlockStore};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A code block extracted from markdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    /// Sequential index of this block (1-based).
    pub index: usize,
    
    /// Language identifier if present (e.g., "rust", "python").
    pub language: Option<String>,
    
    /// The code block content.
    pub content: String,
    
    /// Optional file path hint detected from comments.
    pub file_hint: Option<PathBuf>,
    
    /// Line number where this block starts in the original text.
    pub start_line: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_block_serialization() {
        let block = CodeBlock {
            index: 1,
            language: Some("rust".to_string()),
            content: "fn main() {}".to_string(),
            file_hint: Some(PathBuf::from("src/main.rs")),
            start_line: 5,
        };
        
        let json = serde_json::to_string(&block).unwrap();
        let deserialized: CodeBlock = serde_json::from_str(&json).unwrap();
        
        assert_eq!(block.index, deserialized.index);
        assert_eq!(block.language, deserialized.language);
        assert_eq!(block.content, deserialized.content);
    }
}


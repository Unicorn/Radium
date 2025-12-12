//! Tree-sitter parser integration for code analysis.

use std::path::Path;
use tree_sitter::{Language, Parser, Tree};

/// Tree-sitter parser wrapper for code analysis.
pub struct TreeSitterParser {
    parser: Parser,
    rust_language: Language,
}

impl TreeSitterParser {
    /// Create a new tree-sitter parser.
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let rust_language = tree_sitter_rust::language();
        
        parser.set_language(rust_language).expect("Failed to load Rust grammar");

        Self {
            parser,
            rust_language,
        }
    }

    /// Parse a Rust file into an AST.
    pub fn parse_rust(&mut self, source: &str) -> Result<Tree, tree_sitter::Error> {
        self.parser.parse(source, None)
    }

    /// Get the Rust language definition.
    pub fn rust_language(&self) -> Language {
        self.rust_language
    }
}

impl Default for TreeSitterParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rust() {
        let mut parser = TreeSitterParser::new();
        let source = "fn test() {}";
        let tree = parser.parse_rust(source).unwrap();
        assert!(tree.root_node().child_count() > 0);
    }
}

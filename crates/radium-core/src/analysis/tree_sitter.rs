//! Tree-sitter parser integration for code analysis.

use tree_sitter::{Parser, Tree};

/// Tree-sitter parser wrapper for code analysis.
pub struct TreeSitterParser {
    parser: Parser,
}

impl TreeSitterParser {
    /// Create a new tree-sitter parser.
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let rust_language = tree_sitter_rust::language();
        
        parser.set_language(rust_language).expect("Failed to load Rust grammar");

        Self {
            parser,
        }
    }

    /// Parse a Rust file into an AST.
    pub fn parse_rust(&mut self, source: &str) -> Option<Tree> {
        self.parser.parse(source, None)
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
        if let Some(tree) = parser.parse_rust(source) {
            assert!(tree.root_node().child_count() > 0);
        } else {
            panic!("Failed to parse");
        }
    }
}

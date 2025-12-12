//! Code analysis and symbol extraction.
//!
//! This module provides AST-based code analysis using tree-sitter for
//! symbol extraction, search, and definition lookup.

pub mod definitions;
pub mod rust_analyzer;
pub mod symbols;
pub mod tree_sitter;
pub mod typescript_analyzer;

pub use definitions::find_definition;
pub use rust_analyzer::RustAnalyzer;
pub use symbols::{Symbol, SymbolKind, SymbolSearchResult};
pub use tree_sitter::TreeSitterParser;
pub use typescript_analyzer::TypeScriptAnalyzer;

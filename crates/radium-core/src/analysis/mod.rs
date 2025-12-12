//! Code analysis and symbol extraction.
//!
//! This module provides AST-based code analysis using tree-sitter for
//! symbol extraction, search, and definition lookup.

pub mod rust_analyzer;
pub mod symbols;
pub mod tree_sitter;

pub use rust_analyzer::RustAnalyzer;
pub use symbols::{Symbol, SymbolKind, SymbolSearchResult};
pub use tree_sitter::TreeSitterParser;

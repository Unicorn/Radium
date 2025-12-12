//! Symbol extraction and representation.

use std::path::PathBuf;

/// Kind of symbol (function, struct, enum, etc.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
    Type,
    Variable,
    Constant,
}

/// A code symbol extracted from source.
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Symbol kind
    pub kind: SymbolKind,
    /// File path where symbol is defined
    pub file_path: PathBuf,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Visibility (public, private, etc.)
    pub visibility: Option<String>,
    /// Additional metadata (async, const, etc.)
    pub metadata: Vec<String>,
}

/// Result of symbol search.
#[derive(Debug, Clone)]
pub struct SymbolSearchResult {
    /// Matching symbols
    pub symbols: Vec<Symbol>,
    /// Total number of matches (may be more than symbols.len() if truncated)
    pub total: usize,
}

impl Symbol {
    /// Create a new symbol.
    pub fn new(
        name: String,
        kind: SymbolKind,
        file_path: PathBuf,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            name,
            kind,
            file_path,
            line,
            column,
            visibility: None,
            metadata: Vec::new(),
        }
    }

    /// Set visibility.
    pub fn with_visibility(mut self, visibility: String) -> Self {
        self.visibility = Some(visibility);
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, metadata: Vec<String>) -> Self {
        self.metadata = metadata;
        self
    }
}

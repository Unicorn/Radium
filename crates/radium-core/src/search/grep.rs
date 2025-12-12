//! Grep-like content search implementation using grep-searcher.

use std::io;
use std::path::{Path, PathBuf};
use grep_regex::RegexMatcher;
use grep_searcher::{Searcher, SearcherBuilder, Sink, SinkMatch};
use regex::Regex;
use crate::workspace::IgnoreWalker;
use super::filters::FileTypeFilter;

/// Options for content search operations.
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Pattern to search for (regex)
    pub pattern: String,
    /// Number of context lines before each match
    pub context_before: usize,
    /// Number of context lines after each match
    pub context_after: usize,
    /// File type filter
    pub file_filter: FileTypeFilter,
    /// Maximum number of results to return
    pub max_results: usize,
    /// Root directory to search in
    pub root: PathBuf,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            context_before: 0,
            context_after: 0,
            file_filter: FileTypeFilter::All,
            max_results: 100,
            root: PathBuf::from("."),
        }
    }
}

/// A single search result match.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// File path (relative to root)
    pub file_path: PathBuf,
    /// Line number (1-indexed)
    pub line_number: u64,
    /// Matched line content
    pub line: String,
    /// Context lines before the match
    pub context_before: Vec<String>,
    /// Context lines after the match
    pub context_after: Vec<String>,
}

/// Search for content across files in a directory.
///
/// This function searches for a pattern across all files in the given root directory,
/// respecting .gitignore patterns and applying file type filters.
///
/// # Arguments
///
/// * `options` - Search configuration options
///
/// # Returns
///
/// Vector of search results, limited by `max_results`
pub fn search_code(options: SearchOptions) -> io::Result<Vec<SearchResult>> {
    let matcher = RegexMatcher::new(&options.pattern)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid regex pattern: {}", e)))?;

    let mut searcher = SearcherBuilder::new()
        .before_context(options.context_before)
        .after_context(options.context_after)
        .build();

    let mut results = Vec::new();
    let walker = IgnoreWalker::new(&options.root);

    for file_path in walker.build() {
        // Apply file type filter
        if !options.file_filter.matches(&file_path) {
            continue;
        }

        if results.len() >= options.max_results {
            break;
        }

        // Read file content
        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue, // Skip files that can't be read (binary, permissions, etc.)
        };

        // Search in this file
        let mut sink = SearchSink {
            file_path: file_path.clone(),
            results: &mut results,
            context_before: options.context_before,
            context_after: options.context_after,
            max_results: options.max_results,
            content: &content,
        };

        if let Err(e) = searcher.search_path(&matcher, &file_path, &mut sink) {
            // Continue on individual file errors
            tracing::debug!("Error searching file {}: {}", file_path.display(), e);
            continue;
        }

        if results.len() >= options.max_results {
            break;
        }
    }

    Ok(results)
}

/// Sink implementation for collecting search results.
struct SearchSink<'a> {
    file_path: PathBuf,
    results: &'a mut Vec<SearchResult>,
    context_before: usize,
    context_after: usize,
    max_results: usize,
    content: &'a str,
}

impl<'a> Sink for SearchSink<'a> {
    type Error = io::Error;

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch<'_>) -> Result<bool, Self::Error> {
        if self.results.len() >= self.max_results {
            return Ok(false); // Stop searching
        }

        let line_number = mat.line_number().unwrap_or(0);
        let line_bytes = mat.bytes();
        let line = String::from_utf8_lossy(line_bytes).to_string();

        // Extract context lines
        let lines: Vec<&str> = self.content.lines().collect();
        let line_idx = (line_number as usize).saturating_sub(1);
        
        let context_before: Vec<String> = if line_idx >= self.context_before {
            lines[line_idx - self.context_before..line_idx]
                .iter()
                .map(|s| s.to_string())
                .collect()
        } else {
            lines[..line_idx]
                .iter()
                .map(|s| s.to_string())
                .collect()
        };

        let context_after: Vec<String> = if line_idx + 1 + self.context_after <= lines.len() {
            lines[line_idx + 1..line_idx + 1 + self.context_after]
                .iter()
                .map(|s| s.to_string())
                .collect()
        } else {
            lines[line_idx + 1..]
                .iter()
                .map(|s| s.to_string())
                .collect()
        };

        // Make path relative to root if possible
        let relative_path = self.file_path.clone();

        self.results.push(SearchResult {
            file_path: relative_path,
            line_number,
            line,
            context_before,
            context_after,
        });

        Ok(true) // Continue searching
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_search_code_basic() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create test files
        fs::write(root.join("file1.rs"), "fn test() {\n    println!(\"hello\");\n}").unwrap();
        fs::write(root.join("file2.ts"), "function test() {\n    console.log('hello');\n}").unwrap();

        let options = SearchOptions {
            pattern: "test".to_string(),
            context_before: 0,
            context_after: 0,
            file_filter: FileTypeFilter::All,
            max_results: 10,
            root: root.to_path_buf(),
        };

        let results = search_code(options).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_code_with_filter() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("file1.rs"), "fn test() {}").unwrap();
        fs::write(root.join("file2.ts"), "function test() {}").unwrap();

        let options = SearchOptions {
            pattern: "test".to_string(),
            context_before: 0,
            context_after: 0,
            file_filter: FileTypeFilter::Language("rust".to_string()),
            max_results: 10,
            root: root.to_path_buf(),
        };

        let results = search_code(options).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].file_path.ends_with("file1.rs"));
    }

    #[test]
    fn test_search_code_max_results() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        for i in 0..5 {
            fs::write(root.join(format!("file{}.rs", i)), "fn test() {}").unwrap();
        }

        let options = SearchOptions {
            pattern: "test".to_string(),
            context_before: 0,
            context_after: 0,
            file_filter: FileTypeFilter::All,
            max_results: 3,
            root: root.to_path_buf(),
        };

        let results = search_code(options).unwrap();
        assert!(results.len() <= 3);
    }
}

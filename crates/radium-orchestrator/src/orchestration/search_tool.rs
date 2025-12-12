//! Content search tool for orchestration.
//!
//! This module provides the `search_code` tool that allows agents to search
//! file contents with context lines, glob filters, and .gitignore support.

use async_trait::async_trait;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use grep_regex::RegexMatcher;
use grep_searcher::{Searcher, SearcherBuilder, Sink, SinkMatch};
use ignore::WalkBuilder;
use glob::Pattern;

use super::file_tools::WorkspaceRootProvider;
use super::tool::{Tool, ToolArguments, ToolHandler, ToolParameters, ToolResult};
use crate::error::{OrchestrationError, Result};

/// Search result structure
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: PathBuf,
    pub line_number: u64,
    pub line: String,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
}

/// File type filter
enum FileTypeFilter {
    All,
    Glob(String),
    Language(String),
}

impl FileTypeFilter {
    fn from_str(s: &str) -> Self {
        let s = s.trim();
        if s == "*" || s.eq_ignore_ascii_case("all") {
            return FileTypeFilter::All;
        }
        if s.starts_with("language:") {
            let lang = s.strip_prefix("language:").unwrap_or("").trim();
            return FileTypeFilter::Language(lang.to_string());
        }
        if s.contains('*') || s.contains('?') || s.contains('[') {
            return FileTypeFilter::Glob(s.to_string());
        }
        FileTypeFilter::Language(s.to_string())
    }

    fn matches(&self, path: &Path) -> bool {
        match self {
            FileTypeFilter::All => true,
            FileTypeFilter::Glob(pattern) => {
                let path_str = path.to_string_lossy();
                Pattern::new(pattern)
                    .ok()
                    .map(|p| p.matches(&path_str))
                    .unwrap_or(false)
            }
            FileTypeFilter::Language(lang) => {
                let ext = path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                match lang.to_lowercase().as_str() {
                    "rust" => ext == "rs",
                    "typescript" | "ts" => ext == "ts" || ext == "tsx",
                    "javascript" | "js" => ext == "js" || ext == "jsx",
                    "python" | "py" => ext == "py",
                    "go" => ext == "go",
                    "java" => ext == "java",
                    "cpp" | "c++" => ext == "cpp" || ext == "cc" || ext == "cxx",
                    "c" => ext == "c",
                    "ruby" | "rb" => ext == "rb",
                    "php" => ext == "php",
                    "swift" => ext == "swift",
                    "kotlin" => ext == "kt",
                    "scala" => ext == "scala",
                    _ => false,
                }
            }
        }
    }
}

/// Search code tool handler
struct SearchCodeHandler {
    workspace_root: Arc<dyn WorkspaceRootProvider>,
}

#[async_trait]
impl ToolHandler for SearchCodeHandler {
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult> {
        let workspace_root = self.workspace_root.workspace_root().ok_or_else(|| {
            OrchestrationError::Other("Workspace root not available".to_string())
        })?;

        let pattern = args.get_string("pattern").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "search_code".to_string(),
                reason: "Missing required 'pattern' argument".to_string(),
            }
        })?;

        let context_lines = args.get_i64("context_lines").unwrap_or(0) as usize;
        let file_types = args.get_string("file_types").unwrap_or_else(|| "*".to_string());
        let max_results = args.get_i64("max_results").unwrap_or(100) as usize;

        // Perform search
        let results = search_code_internal(&workspace_root, &pattern, context_lines, &file_types, max_results)
            .map_err(|e| OrchestrationError::Other(format!("Search failed: {}", e)))?;

        if results.is_empty() {
            return Ok(ToolResult::success("No matches found".to_string()));
        }

        // Format results
        let mut output = String::new();
        output.push_str(&format!("# Search Results ({} found)\n\n", results.len()));

        for result in results {
            output.push_str(&format!("## {}\n", result.file_path.display()));
            output.push_str(&format!("**Line {}:**\n\n", result.line_number));

            // Add context before
            if !result.context_before.is_empty() {
                output.push_str("```\n");
                for line in &result.context_before {
                    output.push_str(&format!("  {}\n", line));
                }
                output.push_str("```\n\n");
            }

            // Add matched line
            output.push_str("```\n");
            output.push_str(&format!("> {}\n", result.line));
            output.push_str("```\n\n");

            // Add context after
            if !result.context_after.is_empty() {
                output.push_str("```\n");
                for line in &result.context_after {
                    output.push_str(&format!("  {}\n", line));
                }
                output.push_str("```\n\n");
            }

            output.push_str("---\n\n");
        }

        Ok(ToolResult::success(output))
    }
}

/// Internal search implementation (public for use by other tools)
pub fn search_code_internal(
    root: &Path,
    pattern: &str,
    context_lines: usize,
    file_types: &str,
    max_results: usize,
) -> io::Result<Vec<SearchResult>> {
    let matcher = RegexMatcher::new(pattern)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid regex pattern: {}", e)))?;

    let mut searcher = SearcherBuilder::new()
        .before_context(context_lines)
        .after_context(context_lines)
        .build();

    let file_filter = FileTypeFilter::from_str(file_types);
    let mut results = Vec::new();

    // Use ignore crate for .gitignore support
    let mut walker = WalkBuilder::new(root);
    walker.add_custom_ignore_filename(".nxignore");

    for entry in walker.build() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }

        let file_path = entry.path().to_path_buf();

        // Apply file type filter
        if !file_filter.matches(&file_path) {
            continue;
        }

        if results.len() >= max_results {
            break;
        }

        // Read file content
        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue, // Skip files that can't be read
        };

        // Search in this file
        let mut sink = SearchSink {
            file_path: file_path.clone(),
            results: &mut results,
            context_before: context_lines,
            context_after: context_lines,
            max_results,
            content: &content,
        };

        if let Err(_) = searcher.search_path(&matcher, &file_path, &mut sink) {
            continue;
        }

        if results.len() >= max_results {
            break;
        }
    }

    Ok(results)
}

/// Sink implementation for collecting search results
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

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch<'_>) -> io::Result<bool> {
        if self.results.len() >= self.max_results {
            return Ok(false);
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

        self.results.push(SearchResult {
            file_path: self.file_path.clone(),
            line_number,
            line,
            context_before,
            context_after,
        });

        Ok(true)
    }
}

/// Create the search_code tool
pub fn create_search_code_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property(
            "pattern",
            "string",
            "Regex pattern to search for in file contents",
            true,
        )
        .add_property(
            "context_lines",
            "number",
            "Number of context lines before and after each match (default: 0)",
            false,
        )
        .add_property(
            "file_types",
            "string",
            "File type filter: '*' (all), '*.rs' (glob), 'language:rust' (language), or 'rust' (shorthand). Default: '*'",
            false,
        )
        .add_property(
            "max_results",
            "number",
            "Maximum number of results to return (default: 100)",
            false,
        );

    let handler = Arc::new(SearchCodeHandler { workspace_root });

    Tool::new(
        "search_code",
        "search_code",
        "Search file contents for a pattern with context lines, file type filters, and .gitignore support",
        parameters,
        handler,
    )
}

/// Create all search tools
pub fn create_search_tools(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Vec<Tool> {
    vec![create_search_code_tool(workspace_root)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct TestWorkspaceRoot {
        root: PathBuf,
    }

    impl WorkspaceRootProvider for TestWorkspaceRoot {
        fn workspace_root(&self) -> Option<PathBuf> {
            Some(self.root.clone())
        }
    }

    #[tokio::test]
    async fn test_search_code_tool_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_search_code_tool(workspace_root);
        assert_eq!(tool.name, "search_code");
    }
}

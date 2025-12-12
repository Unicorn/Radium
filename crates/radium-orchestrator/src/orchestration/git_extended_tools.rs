//! Extended Git and code search tools
//!
//! This module provides advanced Git and code search capabilities:
//! - find_references: Find all references to a symbol using ripgrep
//! - git_blame: Show git blame for a file
//! - git_show: Show git commit details

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::process::Command;

use super::file_tools::WorkspaceRootProvider;
use super::tool::{Tool, ToolArguments, ToolHandler, ToolParameters, ToolResult};
use crate::error::{OrchestrationError, Result};

// ============================================================================
// Find References Tool
// ============================================================================

/// Find references tool handler
struct FindReferencesHandler {
    workspace_root: Arc<dyn WorkspaceRootProvider>,
}

#[async_trait]
impl ToolHandler for FindReferencesHandler {
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult> {
        let workspace_root = self.workspace_root.workspace_root().ok_or_else(|| {
            OrchestrationError::Other("Workspace root not available".to_string())
        })?;

        let symbol = args.get_string("symbol").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "find_references".to_string(),
                reason: "Missing required 'symbol' argument".to_string(),
            }
        })?;

        let file_type = args.get_string("file_type");
        let max_results = args.get_i64("max_results").unwrap_or(100) as usize;

        let references = find_symbol_references(&workspace_root, &symbol, file_type.as_deref(), max_results).await?;

        Ok(ToolResult::success(references))
    }
}

/// Find all references to a symbol using internal search
pub async fn find_symbol_references(
    workspace_root: &Path,
    symbol: &str,
    file_type: Option<&str>,
    max_results: usize,
) -> Result<String> {
    use super::search_tool;
    
    // Use word boundary pattern for exact symbol matches
    let pattern = format!(r"\b{}\b", regex::escape(symbol));
    
    // Convert file_type to filter string
    let file_types = if let Some(ftype) = file_type {
        format!("language:{}", ftype)
    } else {
        "*".to_string()
    };

    // Use internal search implementation
    let results = search_tool::search_code_internal(
        workspace_root,
        &pattern,
        0, // No context for find_references
        &file_types,
        max_results,
    ).map_err(|e| OrchestrationError::Other(format!("Search failed: {}", e)))?;

    if results.is_empty() {
        return Ok(format!("No references found for symbol '{}'", symbol));
    }

    // Format results similar to ripgrep output
    let mut formatted = format!("# References to '{}' ({} found)\n\n", symbol, results.len());
    formatted.push_str("```\n");
    
    for result in results {
        formatted.push_str(&format!("{}:{}:{}\n", 
            result.file_path.display(), 
            result.line_number, 
            result.line.trim()));
    }
    
    formatted.push_str("```\n");
    Ok(formatted)
}

/// Create the find_references tool
pub fn create_find_references_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property(
            "symbol",
            "string",
            "Symbol name to search for (function, type, variable, etc.)",
            true,
        )
        .add_property(
            "file_type",
            "string",
            "File type filter (e.g., 'rust', 'js', 'py', 'go'). Optional.",
            false,
        )
        .add_property(
            "max_results",
            "number",
            "Maximum number of results to return (default: 100)",
            false,
        );

    let handler = Arc::new(FindReferencesHandler { workspace_root });

    Tool::new(
        "find_references",
        "find_references",
        "Find all references to a symbol (function, type, variable) in the codebase using ripgrep",
        parameters,
        handler,
    )
}

// ============================================================================
// Git Blame Tool
// ============================================================================

/// Git blame tool handler
struct GitBlameHandler {
    workspace_root: Arc<dyn WorkspaceRootProvider>,
}

#[async_trait]
impl ToolHandler for GitBlameHandler {
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult> {
        let workspace_root = self.workspace_root.workspace_root().ok_or_else(|| {
            OrchestrationError::Other("Workspace root not available".to_string())
        })?;

        let file_path = args.get_string("file_path").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "git_blame".to_string(),
                reason: "Missing required 'file_path' argument".to_string(),
            }
        })?;

        let start_line = args.get_i64("start_line");
        let end_line = args.get_i64("end_line");

        let blame_output = git_blame(&workspace_root, &file_path, start_line, end_line).await?;

        Ok(ToolResult::success(blame_output))
    }
}

/// Execute git blame on a file
pub async fn git_blame(
    workspace_root: &Path,
    file_path: &str,
    start_line: Option<i64>,
    end_line: Option<i64>,
) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.arg("blame");

    // Add line range if specified
    if let (Some(start), Some(end)) = (start_line, end_line) {
        cmd.arg("-L");
        cmd.arg(format!("{},{}", start, end));
    }

    cmd.arg(file_path);
    cmd.current_dir(workspace_root);

    let output = cmd.output().await.map_err(|e| {
        OrchestrationError::Other(format!("Failed to run git blame: {}", e))
    })?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        let mut formatted = format!("# Git Blame: {}\n\n", file_path);
        formatted.push_str("```\n");
        formatted.push_str(&result);
        formatted.push_str("```\n");
        Ok(formatted)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(OrchestrationError::Other(format!("git blame failed: {}", error)))
    }
}

/// Create the git_blame tool
pub fn create_git_blame_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property(
            "file_path",
            "string",
            "Path to the file (relative to workspace root)",
            true,
        )
        .add_property(
            "start_line",
            "number",
            "Start line number (optional, for line range)",
            false,
        )
        .add_property(
            "end_line",
            "number",
            "End line number (optional, for line range)",
            false,
        );

    let handler = Arc::new(GitBlameHandler { workspace_root });

    Tool::new(
        "git_blame",
        "git_blame",
        "Show git blame for a file (who changed which lines). Optionally specify line range.",
        parameters,
        handler,
    )
}

// ============================================================================
// Git Show Tool
// ============================================================================

/// Git show tool handler
struct GitShowHandler {
    workspace_root: Arc<dyn WorkspaceRootProvider>,
}

#[async_trait]
impl ToolHandler for GitShowHandler {
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult> {
        let workspace_root = self.workspace_root.workspace_root().ok_or_else(|| {
            OrchestrationError::Other("Workspace root not available".to_string())
        })?;

        let commit_ref = args.get_string("commit").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "git_show".to_string(),
                reason: "Missing required 'commit' argument".to_string(),
            }
        })?;

        let file_path = args.get_string("file_path");
        let stat_only = args.get_bool("stat_only").unwrap_or(false);

        let show_output = git_show(&workspace_root, &commit_ref, file_path.as_deref(), stat_only).await?;

        Ok(ToolResult::success(show_output))
    }
}

/// Execute git show for a commit
pub async fn git_show(
    workspace_root: &Path,
    commit: &str,
    file_path: Option<&str>,
    stat_only: bool,
) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.arg("show");

    if stat_only {
        cmd.arg("--stat");
    }

    cmd.arg(commit);

    if let Some(path) = file_path {
        cmd.arg("--");
        cmd.arg(path);
    }

    cmd.current_dir(workspace_root);

    let output = cmd.output().await.map_err(|e| {
        OrchestrationError::Other(format!("Failed to run git show: {}", e))
    })?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        let mut formatted = format!("# Git Show: {}\n\n", commit);
        if let Some(path) = file_path {
            formatted.push_str(&format!("**File:** {}\n\n", path));
        }
        formatted.push_str("```\n");
        formatted.push_str(&result);
        formatted.push_str("```\n");
        Ok(formatted)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(OrchestrationError::Other(format!("git show failed: {}", error)))
    }
}

/// Create the git_show tool
pub fn create_git_show_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property(
            "commit",
            "string",
            "Commit reference (hash, HEAD, branch name, etc.)",
            true,
        )
        .add_property(
            "file_path",
            "string",
            "Optional file path to show changes for specific file only",
            false,
        )
        .add_property(
            "stat_only",
            "boolean",
            "Show only statistics (--stat) instead of full diff (default: false)",
            false,
        );

    let handler = Arc::new(GitShowHandler { workspace_root });

    Tool::new(
        "git_show",
        "git_show",
        "Show git commit details including diff. Can filter by file path or show stats only.",
        parameters,
        handler,
    )
}

// ============================================================================
// Public API
// ============================================================================

/// Create all extended Git and search tools
pub fn create_git_extended_tools(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Vec<Tool> {
    vec![
        create_find_references_tool(Arc::clone(&workspace_root)),
        create_git_blame_tool(Arc::clone(&workspace_root)),
        create_git_show_tool(workspace_root),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    struct TestWorkspaceRoot {
        root: PathBuf,
    }

    impl WorkspaceRootProvider for TestWorkspaceRoot {
        fn workspace_root(&self) -> Option<PathBuf> {
            Some(self.root.clone())
        }
    }

    #[tokio::test]
    async fn test_find_references_tool_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_find_references_tool(workspace_root);
        assert_eq!(tool.name, "find_references");
    }

    #[tokio::test]
    async fn test_git_blame_tool_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_git_blame_tool(workspace_root);
        assert_eq!(tool.name, "git_blame");
    }

    #[tokio::test]
    async fn test_git_show_tool_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_git_show_tool(workspace_root);
        assert_eq!(tool.name, "git_show");
    }

    #[tokio::test]
    async fn test_create_all_extended_tools() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tools = create_git_extended_tools(workspace_root);
        assert_eq!(tools.len(), 3);
        assert_eq!(tools[0].name, "find_references");
        assert_eq!(tools[1].name, "git_blame");
        assert_eq!(tools[2].name, "git_show");
    }
}

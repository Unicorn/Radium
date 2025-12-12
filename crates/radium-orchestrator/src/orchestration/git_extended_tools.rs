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
// Git Status Tool
// ============================================================================

/// Git status tool handler
struct GitStatusHandler {
    workspace_root: Arc<dyn WorkspaceRootProvider>,
}

#[async_trait]
impl ToolHandler for GitStatusHandler {
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult> {
        let workspace_root = self.workspace_root.workspace_root().ok_or_else(|| {
            OrchestrationError::Other("Workspace root not available".to_string())
        })?;

        let format = args.get_string("format").unwrap_or_else(|| "short".to_string());
        let show_untracked = args.get_bool("show_untracked").unwrap_or(true);

        // Check if this is a git repository
        let git_check = Command::new("git")
            .arg("rev-parse")
            .arg("--git-dir")
            .current_dir(&workspace_root)
            .output()
            .await;

        if git_check.is_err() || !git_check.as_ref().unwrap().status.success() {
            return Ok(ToolResult::error("Not a git repository".to_string()));
        }

        // Get git status using porcelain v2 format for machine-readable output
        let mut status_cmd = Command::new("git");
        status_cmd.arg("status").arg("--porcelain=v2");
        if !show_untracked {
            status_cmd.arg("--untracked-files=no");
        }
        status_cmd.current_dir(&workspace_root);

        let output = status_cmd.output().await.map_err(|e| {
            OrchestrationError::Other(format!("Failed to execute git status: {}", e))
        })?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(OrchestrationError::Other(format!("git status failed: {}", error)));
        }

        let status_output = String::from_utf8_lossy(&output.stdout);
        
        // Get branch information
        let branch_output = Command::new("git")
            .arg("branch")
            .arg("--show-current")
            .current_dir(&workspace_root)
            .output()
            .await
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "detached HEAD".to_string());

        // Parse porcelain v2 output
        let status_info = parse_git_status_porcelain_v2(&status_output, &branch_output, &format)?;

        Ok(ToolResult::success(status_info)
            .with_metadata("branch", branch_output)
            .with_metadata("format", format))
    }
}

/// Parse git status porcelain v2 output
fn parse_git_status_porcelain_v2(
    porcelain_output: &str,
    branch: &str,
    format: &str,
) -> Result<String> {
    let mut staged = Vec::new();
    let mut modified = Vec::new();
    let mut deleted = Vec::new();
    let mut untracked = Vec::new();
    let mut renamed = Vec::new();

    for line in porcelain_output.lines() {
        if line.is_empty() {
            continue;
        }

        // Porcelain v2 format: 
        // Regular: XY <sub> <mH> <mI> <mW> <hH> <hI> <path>
        // Renamed: XY <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path1><sep><path2>
        // X = status of index, Y = status of work tree
        // X and Y can be: ' ' (unmodified), M (modified), A (added), D (deleted), R (renamed), C (copied), U (unmerged), ? (untracked)
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let status = parts[0];
        if status.len() < 2 {
            continue;
        }

        let index_status = status.chars().nth(0).unwrap_or(' ');
        let worktree_status = status.chars().nth(1).unwrap_or(' ');

        // Handle renamed files (format: R <score> <old> -> <new>)
        if index_status == 'R' || worktree_status == 'R' {
            // Porcelain v2 renamed format: R <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path1><sep><path2>
            // The separator is usually \t (tab) between old and new path
            if parts.len() >= 8 {
                let path_part = parts[7..].join(" ");
                if let Some(sep_pos) = path_part.find('\t') {
                    let old_path = path_part[..sep_pos].to_string();
                    let new_path = path_part[sep_pos + 1..].to_string();
                    renamed.push((old_path, new_path));
                } else if path_part.contains(" -> ") {
                    // Fallback for " -> " separator
                    let paths: Vec<&str> = path_part.split(" -> ").collect();
                    if paths.len() == 2 {
                        renamed.push((paths[0].to_string(), paths[1].to_string()));
                    }
                }
            }
            continue;
        }

        // Get path (last part after status and metadata)
        let path = if parts.len() >= 8 {
            // Full format with metadata
            parts[7..].join(" ")
        } else {
            // Simplified format
            parts[1..].join(" ")
        };

        // Categorize by index status (staged) and worktree status (modified)
        match (index_status, worktree_status) {
            ('A', ' ') => staged.push(("A", path)),
            ('M', ' ') => staged.push(("M", path)),
            ('D', ' ') => staged.push(("D", path)),
            (' ', 'M') => modified.push(("M", path)),
            (' ', 'D') => deleted.push(("D", path)),
            (' ', 'A') => modified.push(("A", path)),
            ('?', '?') => untracked.push(("??", path)),
            ('?', _) => untracked.push(("??", path)),
            (_, '?') => untracked.push(("??", path)),
            ('M', 'M') => {
                // Modified in both index and worktree
                staged.push(("M", path.clone()));
                modified.push(("M", path));
            }
            _ => {}
        }
    }

    // Format output
    let mut output = format!("Branch: {}\n", branch);
    
    let total_changes = staged.len() + modified.len() + deleted.len() + untracked.len() + renamed.len();
    if total_changes == 0 {
        output.push_str("Status: clean\n");
        return Ok(output);
    }

    output.push_str(&format!("Status: {} change(s)\n\n", total_changes));

    if format == "detailed" {
        if !staged.is_empty() {
            output.push_str("Staged:\n");
            for (status, path) in &staged {
                output.push_str(&format!("  {}  {}\n", status, path));
            }
            output.push('\n');
        }

        if !modified.is_empty() {
            output.push_str("Modified:\n");
            for (status, path) in &modified {
                output.push_str(&format!("  {}  {}\n", status, path));
            }
            output.push('\n');
        }

        if !deleted.is_empty() {
            output.push_str("Deleted:\n");
            for (status, path) in &deleted {
                output.push_str(&format!("  {}  {}\n", status, path));
            }
            output.push('\n');
        }

        if !renamed.is_empty() {
            output.push_str("Renamed:\n");
            for (old, new) in &renamed {
                output.push_str(&format!("  R  {} -> {}\n", old, new));
            }
            output.push('\n');
        }

        if !untracked.is_empty() {
            output.push_str("Untracked:\n");
            for (status, path) in &untracked {
                output.push_str(&format!("  {}  {}\n", status, path));
            }
        }
    } else {
        // Short format
        if !staged.is_empty() {
            output.push_str(&format!("Staged ({}):\n", staged.len()));
            for (status, path) in &staged {
                output.push_str(&format!("  {}  {}\n", status, path));
            }
        }
        if !modified.is_empty() {
            output.push_str(&format!("Modified ({}):\n", modified.len()));
            for (status, path) in &modified {
                output.push_str(&format!("  {}  {}\n", status, path));
            }
        }
        if !deleted.is_empty() {
            output.push_str(&format!("Deleted ({}):\n", deleted.len()));
            for (status, path) in &deleted {
                output.push_str(&format!("  {}  {}\n", status, path));
            }
        }
        if !untracked.is_empty() {
            output.push_str(&format!("Untracked ({}):\n", untracked.len()));
            for (status, path) in &untracked {
                output.push_str(&format!("  {}  {}\n", status, path));
            }
        }
    }

    Ok(output)
}

/// Create the git_status tool
pub fn create_git_status_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property(
            "format",
            "string",
            "Output format: 'short' (default) or 'detailed'",
            false,
        )
        .add_property(
            "show_untracked",
            "boolean",
            "Show untracked files (default: true)",
            false,
        );

    let handler = Arc::new(GitStatusHandler { workspace_root });

    Tool::new(
        "git_status",
        "git_status",
        "Show git repository status with staged, modified, deleted, and untracked files",
        parameters,
        handler,
    )
}

// ============================================================================
// Git Diff Tool
// ============================================================================

/// Git diff tool handler
struct GitDiffHandler {
    workspace_root: Arc<dyn WorkspaceRootProvider>,
}

#[async_trait]
impl ToolHandler for GitDiffHandler {
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult> {
        let workspace_root = self.workspace_root.workspace_root().ok_or_else(|| {
            OrchestrationError::Other("Workspace root not available".to_string())
        })?;

        let commit = args.get_string("commit");
        let file_path = args.get_string("file_path");
        let staged_only = args.get_bool("staged_only").unwrap_or(false);

        let mut diff_cmd = Command::new("git");
        diff_cmd.arg("diff");

        if staged_only {
            diff_cmd.arg("--cached");
        }

        if let Some(ref commit_ref) = commit {
            diff_cmd.arg(commit_ref);
        }

        if let Some(ref path) = file_path {
            diff_cmd.arg("--").arg(path);
        }

        diff_cmd.current_dir(&workspace_root);

        let output = diff_cmd.output().await.map_err(|e| {
            OrchestrationError::Other(format!("Failed to execute git diff: {}", e))
        })?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(OrchestrationError::Other(format!("git diff failed: {}", error)));
        }

        let diff_output = String::from_utf8_lossy(&output.stdout);
        
        if diff_output.trim().is_empty() {
            Ok(ToolResult::success("No differences found".to_string()))
        } else {
            Ok(ToolResult::success(diff_output.to_string()))
        }
    }
}

/// Create the git_diff tool
pub fn create_git_diff_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property(
            "commit",
            "string",
            "Optional commit reference to diff against (default: working directory vs HEAD)",
            false,
        )
        .add_property(
            "file_path",
            "string",
            "Optional file path to show diff for specific file only",
            false,
        )
        .add_property(
            "staged_only",
            "boolean",
            "Show only staged changes (--cached, default: false)",
            false,
        );

    let handler = Arc::new(GitDiffHandler { workspace_root });

    Tool::new(
        "git_diff",
        "git_diff",
        "Show git diff for changes. Can filter by commit, file path, or show staged changes only.",
        parameters,
        handler,
    )
}

// ============================================================================
// Git Log Tool
// ============================================================================

/// Git log tool handler
struct GitLogHandler {
    workspace_root: Arc<dyn WorkspaceRootProvider>,
}

#[async_trait]
impl ToolHandler for GitLogHandler {
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult> {
        let workspace_root = self.workspace_root.workspace_root().ok_or_else(|| {
            OrchestrationError::Other("Workspace root not available".to_string())
        })?;

        let max_entries = args.get_i64("max_entries").unwrap_or(10) as usize;
        let file_path = args.get_string("file_path");
        let format = args.get_string("format").unwrap_or_else(|| "short".to_string());

        let mut log_cmd = Command::new("git");
        log_cmd.arg("log");

        match format.as_str() {
            "oneline" => {
                log_cmd.arg("--oneline");
            }
            "short" => {
                log_cmd.arg("--pretty=format:%h - %an, %ar : %s");
            }
            "full" => {
                log_cmd.arg("--pretty=full");
            }
            _ => {
                log_cmd.arg("--pretty=format:%h - %an, %ar : %s");
            }
        }

        log_cmd.arg(format!("-{}", max_entries));

        if let Some(ref path) = file_path {
            log_cmd.arg("--").arg(path);
        }

        log_cmd.current_dir(&workspace_root);

        let output = log_cmd.output().await.map_err(|e| {
            OrchestrationError::Other(format!("Failed to execute git log: {}", e))
        })?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(OrchestrationError::Other(format!("git log failed: {}", error)));
        }

        let log_output = String::from_utf8_lossy(&output.stdout);
        
        if log_output.trim().is_empty() {
            Ok(ToolResult::success("No commits found".to_string()))
        } else {
            Ok(ToolResult::success(log_output.to_string()))
        }
    }
}

/// Create the git_log tool
pub fn create_git_log_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property(
            "max_entries",
            "integer",
            "Maximum number of log entries to return (default: 10)",
            false,
        )
        .add_property(
            "file_path",
            "string",
            "Optional file path to show log for specific file only",
            false,
        )
        .add_property(
            "format",
            "string",
            "Output format: 'short' (default), 'oneline', or 'full'",
            false,
        );

    let handler = Arc::new(GitLogHandler { workspace_root });

    Tool::new(
        "git_log",
        "git_log",
        "Show git commit log. Can filter by file path and limit number of entries.",
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
        create_git_show_tool(Arc::clone(&workspace_root)),
        create_git_status_tool(Arc::clone(&workspace_root)),
        create_git_diff_tool(Arc::clone(&workspace_root)),
        create_git_log_tool(workspace_root),
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
        assert_eq!(tools.len(), 6);
        assert_eq!(tools[0].name, "find_references");
        assert_eq!(tools[1].name, "git_blame");
        assert_eq!(tools[2].name, "git_show");
        assert_eq!(tools[3].name, "git_status");
        assert_eq!(tools[4].name, "git_diff");
        assert_eq!(tools[5].name, "git_log");
    }

    #[tokio::test]
    async fn test_git_status_tool_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_git_status_tool(workspace_root);
        assert_eq!(tool.name, "git_status");
    }

    #[tokio::test]
    async fn test_git_status_not_a_repo() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_git_status_tool(workspace_root);
        let args = ToolArguments::new(serde_json::json!({}));
        let result = tool.execute(&args).await.unwrap();
        
        // Should return error for non-git repo
        assert!(!result.success || result.output.contains("Not a git repository"));
    }

    #[tokio::test]
    async fn test_git_status_in_git_repo() {
        let temp_dir = TempDir::new().unwrap();
        
        // Initialize git repo
        tokio::process::Command::new("git")
            .arg("init")
            .current_dir(temp_dir.path())
            .output()
            .await
            .unwrap();
        
        // Create a test file
        fs::write(temp_dir.path().join("test.txt"), "test").await.unwrap();
        
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_git_status_tool(workspace_root);
        let args = ToolArguments::new(serde_json::json!({}));
        let result = tool.execute(&args).await.unwrap();
        
        // Should succeed and show status
        assert!(result.success);
        assert!(result.output.contains("Branch:") || result.output.contains("Status:"));
    }

    #[tokio::test]
    async fn test_git_diff_tool_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_git_diff_tool(workspace_root);
        assert_eq!(tool.name, "git_diff");
    }

    #[tokio::test]
    async fn test_git_log_tool_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_git_log_tool(workspace_root);
        assert_eq!(tool.name, "git_log");
    }
}

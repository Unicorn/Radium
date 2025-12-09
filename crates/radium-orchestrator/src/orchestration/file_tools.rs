//! File operation tools for orchestration
//!
//! This module provides file operation tools (read_file, write_file, search_replace, etc.)
//! that can be used by the orchestrator to manipulate files in the workspace.

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

use super::tool::{Tool, ToolArguments, ToolHandler, ToolParameters, ToolResult};
use crate::error::{OrchestrationError, Result};

/// Trait for workspace root resolution to avoid direct dependency on radium-core
pub trait WorkspaceRootProvider: Send + Sync {
    /// Get the workspace root path
    fn workspace_root(&self) -> Option<PathBuf>;
}

/// File operation tool handler
struct FileOperationHandler {
    /// Workspace root provider
    workspace_root: Arc<dyn WorkspaceRootProvider>,
    /// Operation type
    operation: FileOperation,
}

/// File operation types
enum FileOperation {
    ReadFile,
    WriteFile,
    SearchReplace,
    ListDir,
    GlobFileSearch,
    ReadLints,
}

#[async_trait]
impl ToolHandler for FileOperationHandler {
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult> {
        let workspace_root = self.workspace_root.workspace_root().ok_or_else(|| {
            OrchestrationError::Other("Workspace root not available".to_string())
        })?;

        match self.operation {
            FileOperation::ReadFile => self.handle_read_file(args, &workspace_root).await,
            FileOperation::WriteFile => self.handle_write_file(args, &workspace_root).await,
            FileOperation::SearchReplace => self.handle_search_replace(args, &workspace_root).await,
            FileOperation::ListDir => self.handle_list_dir(args, &workspace_root).await,
            FileOperation::GlobFileSearch => self.handle_glob_file_search(args, &workspace_root).await,
            FileOperation::ReadLints => self.handle_read_lints(args, &workspace_root).await,
        }
    }
}

impl FileOperationHandler {
    /// Resolve a file path relative to workspace root
    fn resolve_path(&self, path_str: &str, workspace_root: &Path) -> PathBuf {
        let path = PathBuf::from(path_str);
        if path.is_absolute() {
            path
        } else {
            workspace_root.join(path)
        }
    }

    /// Handle read_file operation
    async fn handle_read_file(&self, args: &ToolArguments, workspace_root: &Path) -> Result<ToolResult> {
        let file_path = args.get_string("file_path").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "read_file".to_string(),
                reason: "Missing required 'file_path' argument".to_string(),
            }
        })?;

        let resolved_path = self.resolve_path(&file_path, workspace_root);

        match fs::read_to_string(&resolved_path).await {
            Ok(content) => Ok(ToolResult::success(content)
                .with_metadata("file_path", resolved_path.display().to_string())),
            Err(e) => Ok(ToolResult::error(format!(
                "Failed to read file {}: {}",
                resolved_path.display(),
                e
            ))),
        }
    }

    /// Handle write_file operation
    async fn handle_write_file(&self, args: &ToolArguments, workspace_root: &Path) -> Result<ToolResult> {
        let file_path = args.get_string("file_path").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "write_file".to_string(),
                reason: "Missing required 'file_path' argument".to_string(),
            }
        })?;

        let contents = args.get_string("contents").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "write_file".to_string(),
                reason: "Missing required 'contents' argument".to_string(),
            }
        })?;

        let resolved_path = self.resolve_path(&file_path, workspace_root);

        // Ensure parent directory exists
        if let Some(parent) = resolved_path.parent() {
            if let Err(e) = fs::create_dir_all(parent).await {
                return Ok(ToolResult::error(format!(
                    "Failed to create parent directory for {}: {}",
                    resolved_path.display(),
                    e
                )));
            }
        }

        match fs::write(&resolved_path, contents).await {
            Ok(_) => Ok(ToolResult::success(format!(
                "Successfully wrote {} bytes to {}",
                resolved_path.metadata().map(|m| m.len()).unwrap_or(0),
                resolved_path.display()
            ))
            .with_metadata("file_path", resolved_path.display().to_string())),
            Err(e) => Ok(ToolResult::error(format!(
                "Failed to write file {}: {}",
                resolved_path.display(),
                e
            ))),
        }
    }

    /// Handle search_replace operation
    async fn handle_search_replace(&self, args: &ToolArguments, workspace_root: &Path) -> Result<ToolResult> {
        let file_path = args.get_string("file_path").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "search_replace".to_string(),
                reason: "Missing required 'file_path' argument".to_string(),
            }
        })?;

        let old_string = args.get_string("old_string").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "search_replace".to_string(),
                reason: "Missing required 'old_string' argument".to_string(),
            }
        })?;

        let new_string = args.get_string("new_string").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "search_replace".to_string(),
                reason: "Missing required 'new_string' argument".to_string(),
            }
        })?;

        let resolved_path = self.resolve_path(&file_path, workspace_root);

        // Read current file content
        let content = match fs::read_to_string(&resolved_path).await {
            Ok(c) => c,
            Err(e) => {
                return Ok(ToolResult::error(format!(
                    "Failed to read file {}: {}",
                    resolved_path.display(),
                    e
                )));
            }
        };

        // Perform replacement
        if !content.contains(&old_string) {
            return Ok(ToolResult::error(format!(
                "Pattern '{}' not found in file {}",
                old_string,
                resolved_path.display()
            )));
        }

        let new_content = content.replace(&old_string, &new_string);
        let replacements = (content.matches(&old_string).count()) as u64;

        // Write back to file
        match fs::write(&resolved_path, new_content).await {
            Ok(_) => Ok(ToolResult::success(format!(
                "Successfully replaced {} occurrence(s) in {}",
                replacements,
                resolved_path.display()
            ))
            .with_metadata("file_path", resolved_path.display().to_string())
            .with_metadata("replacements", replacements.to_string())),
            Err(e) => Ok(ToolResult::error(format!(
                "Failed to write file {}: {}",
                resolved_path.display(),
                e
            ))),
        }
    }

    /// Handle list_dir operation
    async fn handle_list_dir(&self, args: &ToolArguments, workspace_root: &Path) -> Result<ToolResult> {
        let dir_path = args.get_string("dir_path").unwrap_or_else(|| ".".to_string());
        let resolved_path = self.resolve_path(&dir_path, workspace_root);

        match fs::read_dir(&resolved_path).await {
            Ok(mut entries) => {
                let mut files = Vec::new();
                let mut dirs = Vec::new();

                while let Some(entry) = entries.next_entry().await.map_err(|e| {
                    OrchestrationError::Other(format!("Failed to read directory entry: {}", e))
                })? {
                    let path = entry.path();
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    if path.is_dir() {
                        dirs.push(name);
                    } else {
                        files.push(name);
                    }
                }

                dirs.sort();
                files.sort();

                let mut output = String::new();
                if !dirs.is_empty() {
                    output.push_str("Directories:\n");
                    for dir in &dirs {
                        output.push_str(&format!("  {}/\n", dir));
                    }
                }
                if !files.is_empty() {
                    output.push_str("Files:\n");
                    for file in &files {
                        output.push_str(&format!("  {}\n", file));
                    }
                }
                if dirs.is_empty() && files.is_empty() {
                    output = "Directory is empty".to_string();
                }

                Ok(ToolResult::success(output)
                    .with_metadata("dir_path", resolved_path.display().to_string())
                    .with_metadata("file_count", files.len().to_string())
                    .with_metadata("dir_count", dirs.len().to_string()))
            }
            Err(e) => Ok(ToolResult::error(format!(
                "Failed to read directory {}: {}",
                resolved_path.display(),
                e
            ))),
        }
    }

    /// Handle glob_file_search operation
    async fn handle_glob_file_search(&self, args: &ToolArguments, workspace_root: &Path) -> Result<ToolResult> {
        let pattern = args.get_string("pattern").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "glob_file_search".to_string(),
                reason: "Missing required 'pattern' argument".to_string(),
            }
        })?;

        // Simple glob matching - for full glob support, we'd need a crate like glob
        // For now, support basic patterns: *.ext, **/*.ext, filename*
        let mut matches = Vec::new();

        fn walk_dir_sync(dir: &Path, pattern: &str, matches: &mut Vec<String>, workspace_root: &Path) -> std::io::Result<()> {
            use std::fs;
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                let relative_path = path.strip_prefix(workspace_root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();

                if path.is_dir() {
                    // Recursively search subdirectories if pattern contains **
                    if pattern.contains("**") {
                        walk_dir_sync(&path, pattern, matches, workspace_root)?;
                    }
                } else if matches_pattern(&relative_path, pattern) {
                    matches.push(relative_path);
                }
            }
            Ok(())
        }

        walk_dir_sync(workspace_root, &pattern, &mut matches, workspace_root)?;

        matches.sort();
        let output = if matches.is_empty() {
            format!("No files found matching pattern: {}", pattern)
        } else {
            format!("Found {} file(s) matching '{}':\n{}", 
                matches.len(), 
                pattern,
                matches.iter().map(|m| format!("  {}\n", m)).collect::<String>())
        };
        Ok(ToolResult::success(output)
            .with_metadata("pattern", pattern)
            .with_metadata("match_count", matches.len().to_string()))
    }

    /// Handle read_lints operation
    async fn handle_read_lints(&self, args: &ToolArguments, workspace_root: &Path) -> Result<ToolResult> {
        // For now, return a placeholder - actual linting would require integration
        // with the linting system (which may be in radium-core)
        let file_path = args.get_string("file_path");
        
        if let Some(path) = file_path {
            let resolved_path = self.resolve_path(&path, workspace_root);
            Ok(ToolResult::success(format!(
                "Linting for {}: No linter configured. This feature requires integration with the linting system.",
                resolved_path.display()
            ))
            .with_metadata("file_path", resolved_path.display().to_string())
            .with_metadata("note", "linting_not_implemented"))
        } else {
            Ok(ToolResult::success(
                "No file specified. Linting requires a file_path argument."
            ))
        }
    }
}

/// Simple pattern matching for glob (supports * and basic patterns)
fn matches_pattern(path: &str, pattern: &str) -> bool {
    // Convert glob pattern to simple matching
    if pattern == "*" {
        return true;
    }

    if pattern.starts_with("*.") {
        // *.ext pattern
        let ext = &pattern[2..];
        return path.ends_with(ext);
    }

    if pattern.ends_with("*") {
        // prefix* pattern
        let prefix = &pattern[..pattern.len() - 1];
        return path.starts_with(prefix);
    }

    // Exact match or contains
    path.contains(pattern) || path == pattern
}

/// Create file operation tools
///
/// # Arguments
/// * `workspace_root` - Provider for workspace root path
///
/// # Returns
/// Vector of file operation tools
pub fn create_file_operation_tools(
    workspace_root: Arc<dyn WorkspaceRootProvider>,
) -> Vec<Tool> {
    vec![
        create_read_file_tool(Arc::clone(&workspace_root)),
        create_write_file_tool(Arc::clone(&workspace_root)),
        create_search_replace_tool(Arc::clone(&workspace_root)),
        create_list_dir_tool(Arc::clone(&workspace_root)),
        create_glob_file_search_tool(Arc::clone(&workspace_root)),
        create_read_lints_tool(Arc::clone(&workspace_root)),
    ]
}

fn create_read_file_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property("file_path", "string", "Path to the file to read (relative to workspace root)", true);

    let handler = Arc::new(FileOperationHandler {
        workspace_root,
        operation: FileOperation::ReadFile,
    });

    Tool::new("read_file", "read_file", "Read the contents of a file", parameters, handler)
}

fn create_write_file_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property("file_path", "string", "Path to the file to write (relative to workspace root)", true)
        .add_property("contents", "string", "Contents to write to the file", true);

    let handler = Arc::new(FileOperationHandler {
        workspace_root,
        operation: FileOperation::WriteFile,
    });

    Tool::new("write_file", "write_file", "Write contents to a file (creates file if it doesn't exist)", parameters, handler)
}

fn create_search_replace_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property("file_path", "string", "Path to the file to modify (relative to workspace root)", true)
        .add_property("old_string", "string", "String to search for", true)
        .add_property("new_string", "string", "String to replace with", true);

    let handler = Arc::new(FileOperationHandler {
        workspace_root,
        operation: FileOperation::SearchReplace,
    });

    Tool::new("search_replace", "search_replace", "Replace occurrences of a string in a file", parameters, handler)
}

fn create_list_dir_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property("dir_path", "string", "Path to the directory to list (relative to workspace root, defaults to '.')", false);

    let handler = Arc::new(FileOperationHandler {
        workspace_root,
        operation: FileOperation::ListDir,
    });

    Tool::new("list_dir", "list_dir", "List files and directories in a directory", parameters, handler)
}

fn create_glob_file_search_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property("pattern", "string", "Glob pattern to search for (e.g., '*.rs', '**/*.md')", true);

    let handler = Arc::new(FileOperationHandler {
        workspace_root,
        operation: FileOperation::GlobFileSearch,
    });

    Tool::new("glob_file_search", "glob_file_search", "Search for files matching a glob pattern", parameters, handler)
}

fn create_read_lints_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property("file_path", "string", "Path to the file to lint (relative to workspace root)", false);

    let handler = Arc::new(FileOperationHandler {
        workspace_root,
        operation: FileOperation::ReadLints,
    });

    Tool::new("read_lints", "read_lints", "Read linting errors for a file", parameters, handler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
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
    async fn test_read_file_tool() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        tokio::fs::write(&test_file, "Hello, world!").await.unwrap();

        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_read_file_tool(workspace_root);
        let args = ToolArguments::new(serde_json::json!({
            "file_path": "test.txt"
        }));

        let result = tool.execute(&args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "Hello, world!");
    }

    #[tokio::test]
    async fn test_write_file_tool() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_write_file_tool(workspace_root);
        let args = ToolArguments::new(serde_json::json!({
            "file_path": "new_file.txt",
            "contents": "Test content"
        }));

        let result = tool.execute(&args).await.unwrap();
        assert!(result.success);

        let content = tokio::fs::read_to_string(temp_dir.path().join("new_file.txt")).await.unwrap();
        assert_eq!(content, "Test content");
    }

    #[tokio::test]
    async fn test_search_replace_tool() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        tokio::fs::write(&test_file, "Hello, world! Hello again!").await.unwrap();

        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_search_replace_tool(workspace_root);
        let args = ToolArguments::new(serde_json::json!({
            "file_path": "test.txt",
            "old_string": "Hello",
            "new_string": "Hi"
        }));

        let result = tool.execute(&args).await.unwrap();
        assert!(result.success);

        let content = tokio::fs::read_to_string(&test_file).await.unwrap();
        assert_eq!(content, "Hi, world! Hi again!");
    }
}


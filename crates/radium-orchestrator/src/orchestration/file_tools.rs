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
    CreateDir,
    DeleteFile,
    RenameFile,
}

/// Validate that a resolved path is within workspace boundaries.
///
/// # Errors
/// Returns OrchestrationError if path escapes workspace
fn validate_workspace_boundary(path: &Path, workspace_root: &Path) -> Result<()> {
    // Canonicalize both paths for comparison
    let canonical_root = workspace_root.canonicalize().map_err(|e| {
        OrchestrationError::Other(format!(
            "Failed to canonicalize workspace root: {}",
            e
        ))
    })?;

    // For non-existent paths, validate parent directory chain
    let canonical_path = if path.exists() {
        path.canonicalize().map_err(|e| {
            OrchestrationError::Other(format!(
                "Failed to canonicalize path {}: {}",
                path.display(),
                e
            ))
        })?
    } else {
        // Validate parent directories that exist
        let mut check_path = path.to_path_buf();
        while !check_path.exists() {
            if let Some(parent) = check_path.parent() {
                check_path = parent.to_path_buf();
            } else {
                break;
            }
        }

        if check_path.exists() {
            let canonical_parent = check_path.canonicalize().map_err(|e| {
                OrchestrationError::Other(format!(
                    "Failed to canonicalize parent path: {}",
                    e
                ))
            })?;

            // Reconstruct full path from canonical parent
            let remaining = path.strip_prefix(&check_path).unwrap_or(path);
            canonical_parent.join(remaining)
        } else {
            path.to_path_buf()
        }
    };

    // Verify path is within workspace
    if !canonical_path.starts_with(&canonical_root) {
        return Err(OrchestrationError::Other(format!(
            "Path '{}' is outside workspace boundary '{}'. All file operations must be within the workspace.",
            canonical_path.display(),
            canonical_root.display()
        )));
    }

    Ok(())
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
            FileOperation::CreateDir => self.handle_create_dir(args, &workspace_root).await,
            FileOperation::DeleteFile => self.handle_delete_file(args, &workspace_root).await,
            FileOperation::RenameFile => self.handle_rename_file(args, &workspace_root).await,
        }
    }
}

impl FileOperationHandler {
    /// Resolve a file path relative to workspace root.
    ///
    /// All paths are treated as workspace-relative. Leading slashes are automatically
    /// stripped to prevent accidental absolute path usage. This ensures AI-provided
    /// paths like "/docs/file.md" are interpreted as "docs/file.md" within workspace.
    ///
    /// # Security
    /// After resolution, paths are validated to ensure they remain within workspace boundaries.
    ///
    /// # Arguments
    /// * `path_str` - Path string (leading slashes auto-stripped)
    /// * `workspace_root` - Workspace root directory
    ///
    /// # Returns
    /// Resolved path within workspace, or error if outside boundaries
    fn resolve_path(&self, path_str: &str, workspace_root: &Path) -> Result<PathBuf> {
        // Strip leading slashes to treat all paths as workspace-relative
        let normalized = path_str.trim_start_matches('/').trim_start_matches('\\');

        // Log when we auto-fix a path for debugging
        if normalized != path_str {
            tracing::debug!(
                "Auto-normalized path: '{}' -> '{}'",
                path_str,
                normalized
            );
        }

        // Resolve relative to workspace root
        let resolved = workspace_root.join(normalized);

        // Validate workspace boundary
        validate_workspace_boundary(&resolved, workspace_root)?;

        Ok(resolved)
    }

    /// Handle read_file operation
    async fn handle_read_file(&self, args: &ToolArguments, workspace_root: &Path) -> Result<ToolResult> {
        let file_path = args.get_string("file_path").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "read_file".to_string(),
                reason: "Missing required 'file_path' argument".to_string(),
            }
        })?;

        let resolved_path = self.resolve_path(&file_path, workspace_root)?;

        // Get optional line range parameters
        let start_line: Option<usize> = args
            .get_i64("start_line")
            .map(|n| n as usize);
        let end_line: Option<usize> = args
            .get_i64("end_line")
            .map(|n| n as usize);

        match fs::read_to_string(&resolved_path).await {
            Ok(content) => {
                let (result_content, metadata) = if let (Some(start), Some(end)) = (start_line, end_line) {
                    // Read specific line range (1-indexed, inclusive)
                    let lines: Vec<&str> = content.lines().collect();
                    let total_lines = lines.len();
                    
                    // Validate line numbers (1-indexed)
                    if start < 1 || end < start || end > total_lines {
                        return Ok(ToolResult::error(format!(
                            "Invalid line range: start_line={}, end_line={}, file has {} lines",
                            start, end, total_lines
                        )));
                    }
                    
                    // Extract range (convert to 0-indexed)
                    let start_idx = start - 1;
                    let end_idx = end; // end is inclusive, so we use it directly
                    let range_lines = &lines[start_idx..end_idx.min(total_lines)];
                    let range_content = range_lines.join("\n");
                    
                    (
                        range_content,
                        vec![
                            ("file_path".to_string(), resolved_path.display().to_string()),
                            ("start_line".to_string(), start.to_string()),
                            ("end_line".to_string(), end.to_string()),
                            ("total_lines".to_string(), total_lines.to_string()),
                        ],
                    )
                } else if let Some(start) = start_line {
                    // Read from start_line to end of file
                    let lines: Vec<&str> = content.lines().collect();
                    let total_lines = lines.len();
                    
                    if start < 1 || start > total_lines {
                        return Ok(ToolResult::error(format!(
                            "Invalid start_line: {}, file has {} lines",
                            start, total_lines
                        )));
                    }
                    
                    let start_idx = start - 1;
                    let range_lines = &lines[start_idx..];
                    let range_content = range_lines.join("\n");
                    
                    (
                        range_content,
                        vec![
                            ("file_path".to_string(), resolved_path.display().to_string()),
                            ("start_line".to_string(), start.to_string()),
                            ("total_lines".to_string(), total_lines.to_string()),
                        ],
                    )
                } else {
                    // Read entire file
                    (
                        content,
                        vec![("file_path".to_string(), resolved_path.display().to_string())],
                    )
                };
                
                let mut result = ToolResult::success(result_content);
                for (key, value) in metadata {
                    result = result.with_metadata(key, value);
                }
                Ok(result)
            }
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

        let resolved_path = self.resolve_path(&file_path, workspace_root)?;

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
            Ok(()) => Ok(ToolResult::success(format!(
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

        let resolved_path = self.resolve_path(&file_path, workspace_root)?;

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
            Ok(()) => Ok(ToolResult::success(format!(
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
        let resolved_path = self.resolve_path(&dir_path, workspace_root)?;

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
            let resolved_path = self.resolve_path(&path, workspace_root)?;
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

    /// Handle create_dir operation
    async fn handle_create_dir(&self, args: &ToolArguments, workspace_root: &Path) -> Result<ToolResult> {
        let dir_path = args.get_string("dir_path").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "create_dir".to_string(),
                reason: "Missing required 'dir_path' argument".to_string(),
            }
        })?;

        let resolved_path = self.resolve_path(&dir_path, workspace_root)?;

        // Check if directory already exists
        if resolved_path.exists() {
            if resolved_path.is_dir() {
                return Ok(ToolResult::success(format!(
                    "Directory already exists: {}",
                    resolved_path.display()
                ))
                .with_metadata("dir_path", resolved_path.display().to_string())
                .with_metadata("already_existed", "true"));
            }
            return Ok(ToolResult::error(format!(
                "Path exists but is not a directory: {}",
                resolved_path.display()
            )));
        }

        // Create directory and all parent directories
        match fs::create_dir_all(&resolved_path).await {
            Ok(()) => Ok(ToolResult::success(format!(
                "Successfully created directory: {}",
                resolved_path.display()
            ))
            .with_metadata("dir_path", resolved_path.display().to_string())
            .with_metadata("created", "true")),
            Err(e) => Ok(ToolResult::error(format!(
                "Failed to create directory {}: {}",
                resolved_path.display(),
                e
            ))),
        }
    }

    /// Handle delete_file operation
    async fn handle_delete_file(&self, args: &ToolArguments, workspace_root: &Path) -> Result<ToolResult> {
        let file_path = args.get_string("file_path").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "delete_file".to_string(),
                reason: "Missing required 'file_path' argument".to_string(),
            }
        })?;

        let resolved_path = self.resolve_path(&file_path, workspace_root)?;

        // Check if file exists
        if !resolved_path.exists() {
            return Ok(ToolResult::error(format!(
                "File not found: {}",
                resolved_path.display()
            )));
        }

        // Check if it's a file (not a directory)
        if !resolved_path.is_file() {
            return Ok(ToolResult::error(format!(
                "Path is not a file: {} (use remove_dir for directories)",
                resolved_path.display()
            )));
        }

        // Delete the file
        match fs::remove_file(&resolved_path).await {
            Ok(()) => Ok(ToolResult::success(format!(
                "Successfully deleted file: {}",
                resolved_path.display()
            ))
            .with_metadata("file_path", resolved_path.display().to_string())
            .with_metadata("deleted", "true")),
            Err(e) => Ok(ToolResult::error(format!(
                "Failed to delete file {}: {}",
                resolved_path.display(),
                e
            ))),
        }
    }

    /// Handle rename_file operation
    async fn handle_rename_file(&self, args: &ToolArguments, workspace_root: &Path) -> Result<ToolResult> {
        let old_path = args.get_string("old_path").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "rename_file".to_string(),
                reason: "Missing required 'old_path' argument".to_string(),
            }
        })?;

        let new_path = args.get_string("new_path").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "rename_file".to_string(),
                reason: "Missing required 'new_path' argument".to_string(),
            }
        })?;

        let resolved_old = self.resolve_path(&old_path, workspace_root)?;
        let resolved_new = self.resolve_path(&new_path, workspace_root)?;

        // Check if source exists
        if !resolved_old.exists() {
            return Ok(ToolResult::error(format!(
                "Source path not found: {}",
                resolved_old.display()
            )));
        }

        // Check if destination already exists
        if resolved_new.exists() {
            return Ok(ToolResult::error(format!(
                "Destination path already exists: {}",
                resolved_new.display()
            )));
        }

        // Ensure destination parent directory exists
        if let Some(parent) = resolved_new.parent() {
            if let Err(e) = fs::create_dir_all(parent).await {
                return Ok(ToolResult::error(format!(
                    "Failed to create parent directory for {}: {}",
                    resolved_new.display(),
                    e
                )));
            }
        }

        // Perform the rename/move
        match fs::rename(&resolved_old, &resolved_new).await {
            Ok(()) => {
                let is_file = resolved_new.is_file();
                Ok(ToolResult::success(format!(
                    "Successfully renamed {} from {} to {}",
                    if is_file { "file" } else { "directory" },
                    resolved_old.display(),
                    resolved_new.display()
                ))
                .with_metadata("old_path", resolved_old.display().to_string())
                .with_metadata("new_path", resolved_new.display().to_string())
                .with_metadata("is_file", is_file.to_string()))
            }
            Err(e) => Ok(ToolResult::error(format!(
                "Failed to rename {} to {}: {}",
                resolved_old.display(),
                resolved_new.display(),
                e
            ))),
        }
    }
}

/// Simple pattern matching for glob (supports * and basic patterns)
fn matches_pattern(path: &str, pattern: &str) -> bool {
    // Convert glob pattern to simple matching
    if pattern == "*" {
        return true;
    }

    if let Some(ext) = pattern.strip_prefix("*.") {
        // *.ext pattern
        return path.ends_with(ext);
    }

    if let Some(prefix) = pattern.strip_suffix('*') {
        // prefix* pattern
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
        create_create_dir_tool(Arc::clone(&workspace_root)),
        create_delete_file_tool(Arc::clone(&workspace_root)),
        create_rename_file_tool(Arc::clone(&workspace_root)),
    ]
}

fn create_read_file_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property("file_path", "string", "Path to the file to read (relative to workspace root)", true)
        .add_property("start_line", "integer", "Optional start line number (1-indexed) for reading a line range", false)
        .add_property("end_line", "integer", "Optional end line number (1-indexed, inclusive) for reading a line range", false);

    let handler = Arc::new(FileOperationHandler {
        workspace_root,
        operation: FileOperation::ReadFile,
    });

    Tool::new(
        "read_file",
        "read_file",
        "Read file contents from workspace. Supports line ranges (start_line, end_line). ALWAYS read files before modifying them. Returns full content or specified range.",
        parameters,
        handler
    )
}

fn create_write_file_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property("file_path", "string", "Path to the file to write (relative to workspace root)", true)
        .add_property("contents", "string", "Contents to write to the file", true);

    let handler = Arc::new(FileOperationHandler {
        workspace_root,
        operation: FileOperation::WriteFile,
    });

    Tool::new(
        "write_file",
        "write_file",
        "Write contents to a file. Creates parent directories automatically. Overwrites existing files. Use after reading to ensure accurate modifications.",
        parameters,
        handler
    )
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

    Tool::new(
        "glob_file_search",
        "glob_file_search",
        "Search for files matching a glob pattern. Use ** for recursive: **/*.rs finds all Rust files. Examples: *.md, src/**/*.toml, **/test/*.rs",
        parameters,
        handler
    )
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

fn create_create_dir_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property(
            "dir_path",
            "string",
            "Path to the directory to create (workspace-relative, auto-strips leading slashes). Creates parent directories automatically.",
            true
        );

    let handler = Arc::new(FileOperationHandler {
        workspace_root,
        operation: FileOperation::CreateDir,
    });

    Tool::new(
        "create_dir",
        "create_dir",
        "Create a directory and all necessary parent directories. Idempotent - succeeds if directory already exists. \
         Use this before creating files in new directories. Paths are workspace-relative and leading slashes are auto-stripped.",
        parameters,
        handler
    )
}

fn create_delete_file_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property(
            "file_path",
            "string",
            "Path to the file to delete (workspace-relative, auto-strips leading slashes). Only deletes files, not directories.",
            true
        );

    let handler = Arc::new(FileOperationHandler {
        workspace_root,
        operation: FileOperation::DeleteFile,
    });

    Tool::new(
        "delete_file",
        "delete_file",
        "Delete a file from the workspace. Only works on files, not directories. \
         Returns error if file doesn't exist or if path points to a directory. \
         WARNING: This operation cannot be undone. Paths are workspace-relative and leading slashes are auto-stripped.",
        parameters,
        handler
    )
}

fn create_rename_file_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property(
            "old_path",
            "string",
            "Current path of the file or directory (workspace-relative, auto-strips leading slashes)",
            true
        )
        .add_property(
            "new_path",
            "string",
            "New path for the file or directory (workspace-relative, auto-strips leading slashes)",
            true
        );

    let handler = Arc::new(FileOperationHandler {
        workspace_root,
        operation: FileOperation::RenameFile,
    });

    Tool::new(
        "rename_file",
        "rename_file",
        "Rename or move a file or directory within the workspace. Works on both files and directories. \
         Creates parent directories for destination if needed. Returns error if source doesn't exist \
         or destination already exists. Both paths are workspace-relative and leading slashes are auto-stripped.",
        parameters,
        handler
    )
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

    // Path resolution tests
    #[tokio::test]
    async fn test_resolve_path_strips_leading_slash() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let handler = FileOperationHandler {
            workspace_root: workspace_root.clone(),
            operation: FileOperation::ReadFile,
        };

        // Test Unix-style leading slash
        let resolved = handler.resolve_path("/docs/file.md", temp_dir.path()).unwrap();
        assert_eq!(
            resolved,
            temp_dir.path().join("docs/file.md")
        );

        // Test Windows-style leading backslash (use forward slashes after stripping)
        let resolved = handler.resolve_path("\\docs/file.md", temp_dir.path()).unwrap();
        assert_eq!(
            resolved,
            temp_dir.path().join("docs/file.md")
        );

        // Test without leading slash (no change)
        let resolved = handler.resolve_path("docs/file.md", temp_dir.path()).unwrap();
        assert_eq!(
            resolved,
            temp_dir.path().join("docs/file.md")
        );

        // Test multiple leading slashes (all stripped)
        let resolved = handler.resolve_path("///docs/file.md", temp_dir.path()).unwrap();
        assert_eq!(
            resolved,
            temp_dir.path().join("docs/file.md")  // All leading slashes are stripped
        );
    }

    #[tokio::test]
    async fn test_resolve_path_rejects_traversal() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let handler = FileOperationHandler {
            workspace_root: workspace_root.clone(),
            operation: FileOperation::ReadFile,
        };

        // Should reject path traversal outside workspace
        let result = handler.resolve_path("../outside", temp_dir.path());
        assert!(result.is_err());
    }

    // New tool tests
    #[tokio::test]
    async fn test_create_dir_tool() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_create_dir_tool(workspace_root);

        // Test creating a new directory
        let args = ToolArguments::new(serde_json::json!({
            "dir_path": "new_dir"
        }));
        let result = tool.execute(&args).await.unwrap();
        assert!(result.success);
        assert!(temp_dir.path().join("new_dir").is_dir());

        // Test idempotency - creating existing directory
        let result = tool.execute(&args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.metadata.get("already_existed"), Some(&"true".to_string()));
    }

    #[tokio::test]
    async fn test_create_dir_with_leading_slash() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_create_dir_tool(workspace_root);

        // Test with leading slash (should be stripped)
        let args = ToolArguments::new(serde_json::json!({
            "dir_path": "/nested/dir/path"
        }));
        let result = tool.execute(&args).await.unwrap();
        assert!(result.success);
        assert!(temp_dir.path().join("nested/dir/path").is_dir());
    }

    #[tokio::test]
    async fn test_delete_file_tool() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "content").unwrap();

        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_delete_file_tool(workspace_root);

        // Delete the file
        let args = ToolArguments::new(serde_json::json!({
            "file_path": "test.txt"
        }));
        let result = tool.execute(&args).await.unwrap();
        assert!(result.success);
        assert!(!test_file.exists());

        // Try to delete non-existent file
        let result = tool.execute(&args).await.unwrap();
        assert!(!result.success);
        assert!(result.output.contains("not found"));
    }

    #[tokio::test]
    async fn test_delete_file_rejects_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path().join("subdir");
        std::fs::create_dir(&dir).unwrap();

        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_delete_file_tool(workspace_root);

        // Try to delete a directory
        let args = ToolArguments::new(serde_json::json!({
            "file_path": "subdir"
        }));
        let result = tool.execute(&args).await.unwrap();
        assert!(!result.success);
        assert!(result.output.contains("not a file"));
    }

    #[tokio::test]
    async fn test_rename_file_tool() {
        let temp_dir = TempDir::new().unwrap();
        let old_file = temp_dir.path().join("old.txt");
        std::fs::write(&old_file, "content").unwrap();

        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_rename_file_tool(workspace_root);

        // Rename the file
        let args = ToolArguments::new(serde_json::json!({
            "old_path": "old.txt",
            "new_path": "new.txt"
        }));
        let result = tool.execute(&args).await.unwrap();
        assert!(result.success);
        assert!(!old_file.exists());
        assert!(temp_dir.path().join("new.txt").exists());

        // Verify content preserved
        let content = std::fs::read_to_string(temp_dir.path().join("new.txt")).unwrap();
        assert_eq!(content, "content");
    }

    #[tokio::test]
    async fn test_rename_file_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let old_file = temp_dir.path().join("file.txt");
        std::fs::write(&old_file, "content").unwrap();

        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_rename_file_tool(workspace_root);

        // Rename to nested path that doesn't exist
        let args = ToolArguments::new(serde_json::json!({
            "old_path": "file.txt",
            "new_path": "new/nested/path/file.txt"
        }));
        let result = tool.execute(&args).await.unwrap();
        assert!(result.success);
        assert!(temp_dir.path().join("new/nested/path/file.txt").exists());
    }

    #[tokio::test]
    async fn test_rename_file_rejects_existing_destination() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("old.txt"), "old").unwrap();
        std::fs::write(temp_dir.path().join("new.txt"), "new").unwrap();

        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_rename_file_tool(workspace_root);

        // Try to rename to existing file
        let args = ToolArguments::new(serde_json::json!({
            "old_path": "old.txt",
            "new_path": "new.txt"
        }));
        let result = tool.execute(&args).await.unwrap();
        assert!(!result.success);
        assert!(result.output.contains("already exists"));
    }
}


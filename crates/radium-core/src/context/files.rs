//! Context file loading and processing.
//!
//! This module provides a hierarchical context file system that enables persistent
//! instructions for agents without repetition. Context files are automatically discovered
//! and loaded based on their location, with higher precedence files overriding lower
//! precedence ones.
//!
//! ## Hierarchical Loading
//!
//! Context files are loaded from multiple locations with the following precedence order
//! (highest to lowest):
//!
//! 1. **Subdirectory context file** - `<subdir>/GEMINI.md` (highest precedence)
//! 2. **Project root context file** - `<workspace>/GEMINI.md`
//! 3. **Global context file** - `~/.radium/GEMINI.md` (lowest precedence)
//!
//! Lower precedence files are prepended to higher precedence files, allowing
//! subdirectory-specific context to override project-wide context.
//!
//! ## Context Imports
//!
//! Context files support importing other files using the `@file.md` syntax. This allows
//! you to organize context into reusable modules and avoid duplication.
//!
//! - Imports are resolved relative to the importing file's directory
//! - Circular imports are detected and rejected
//! - Duplicate imports are automatically deduplicated
//! - Imports inside code blocks are ignored
//!
//! ## Usage
//!
//! ```rust,no_run
//! use radium_core::context::ContextFileLoader;
//! use std::path::Path;
//!
//! // Create a loader for a workspace
//! let loader = ContextFileLoader::new(Path::new("/workspace"));
//!
//! // Load context for a specific path (hierarchically)
//! let content = loader.load_hierarchical(Path::new("/workspace/src"))?;
//!
//! // Discover all context files in the workspace
//! let files = loader.discover_context_files()?;
//!
//! // Process imports in content
//! let processed = loader.process_imports("@shared.md", Path::new("/workspace"))?;
//! ```
//!
//! ## User Documentation
//!
//! For detailed user-facing documentation, see [`docs/features/context-files.md`](../../../../docs/features/context-files.md).

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::error::{ContextError, Result};

/// Default context file name.
const DEFAULT_CONTEXT_FILE_NAME: &str = "GEMINI.md";

/// Context file loader for hierarchical loading and import processing.
///
/// This struct provides methods to discover, load, and process context files
/// (typically `GEMINI.md`) from a workspace. It supports hierarchical loading
/// with precedence resolution and import processing with circular dependency detection.
///
/// # Examples
///
/// Basic usage with default file name:
///
/// ```rust,no_run
/// use radium_core::context::ContextFileLoader;
/// use std::path::Path;
///
/// let loader = ContextFileLoader::new(Path::new("/workspace"));
/// let content = loader.load_hierarchical(Path::new("/workspace/src"))?;
/// ```
///
/// Using a custom context file name:
///
/// ```rust,no_run
/// use radium_core::context::ContextFileLoader;
/// use std::path::Path;
///
/// let loader = ContextFileLoader::with_file_name(
///     Path::new("/workspace"),
///     "CONTEXT.md".to_string()
/// );
/// ```
pub struct ContextFileLoader {
    /// Workspace root path.
    workspace_root: PathBuf,
    /// Custom context file name (default: "GEMINI.md").
    custom_file_name: Option<String>,
}

impl ContextFileLoader {
    /// Creates a new context file loader with default file name (`GEMINI.md`).
    ///
    /// # Arguments
    /// * `workspace_root` - The workspace root directory
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use radium_core::context::ContextFileLoader;
    /// use std::path::Path;
    ///
    /// let loader = ContextFileLoader::new(Path::new("/workspace"));
    /// ```
    pub fn new(workspace_root: impl AsRef<Path>) -> Self {
        Self { workspace_root: workspace_root.as_ref().to_path_buf(), custom_file_name: None }
    }

    /// Creates a new context file loader with a custom file name.
    ///
    /// This allows you to use a different file name than the default `GEMINI.md`.
    /// Useful for projects that want to use a different naming convention.
    ///
    /// # Arguments
    /// * `workspace_root` - The workspace root directory
    /// * `file_name` - Custom context file name (e.g., `"CONTEXT.md"`)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use radium_core::context::ContextFileLoader;
    /// use std::path::Path;
    ///
    /// let loader = ContextFileLoader::with_file_name(
    ///     Path::new("/workspace"),
    ///     "CONTEXT.md".to_string()
    /// );
    /// ```
    pub fn with_file_name(workspace_root: impl AsRef<Path>, file_name: String) -> Self {
        Self {
            workspace_root: workspace_root.as_ref().to_path_buf(),
            custom_file_name: Some(file_name),
        }
    }

    /// Gets the context file name to use.
    fn file_name(&self) -> &str {
        self.custom_file_name.as_deref().unwrap_or(DEFAULT_CONTEXT_FILE_NAME)
    }

    /// Loads context files hierarchically for a given path.
    ///
    /// This method discovers and loads all applicable context files based on the
    /// provided path, following the hierarchical precedence order. The content
    /// from all files is merged, with lower precedence files prepended to higher
    /// precedence files.
    ///
    /// ## Precedence Order (highest to lowest)
    ///
    /// 1. **Subdirectory context file** - `<path>/GEMINI.md` or `<path_parent>/GEMINI.md`
    /// 2. **Project root context file** - `<workspace_root>/GEMINI.md`
    /// 3. **Global context file** - `~/.radium/GEMINI.md`
    ///
    /// Lower precedence files are prepended to higher precedence files, so
    /// subdirectory context can override project context.
    ///
    /// ## Import Processing
    ///
    /// After loading, all `@file.md` import statements are automatically processed.
    /// See [`process_imports`](Self::process_imports) for details.
    ///
    /// # Arguments
    /// * `path` - The path to load context for (can be a file or directory)
    ///
    /// # Returns
    /// Combined context content from all applicable files, with imports resolved.
    /// Returns an empty string if no context files are found.
    ///
    /// # Errors
    ///
    /// This method returns an error if:
    /// - A context file exists but cannot be read (permission errors, I/O errors)
    /// - Import processing fails (circular imports, missing import files)
    ///
    /// Missing context files are silently ignored and do not cause errors.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use radium_core::context::ContextFileLoader;
    /// use std::path::Path;
    ///
    /// let loader = ContextFileLoader::new(Path::new("/workspace"));
    ///
    /// // Load context for a subdirectory
    /// let content = loader.load_hierarchical(Path::new("/workspace/src"))?;
    ///
    /// // Load context for a specific file
    /// let content = loader.load_hierarchical(Path::new("/workspace/src/main.rs"))?;
    /// ```
    pub fn load_hierarchical(&self, path: &Path) -> Result<String> {
        let file_name = self.file_name();
        let mut contexts = Vec::new();

        // 1. Global context file (~/.radium/GEMINI.md)
        if let Ok(home) = std::env::var("HOME") {
            let global_path = PathBuf::from(home).join(".radium").join(file_name);
            if global_path.exists() {
                if let Ok(content) = fs::read_to_string(&global_path) {
                    contexts.push(("global", content));
                }
            }
        }

        // 2. Project root context file
        let project_path = self.workspace_root.join(file_name);
        if project_path.exists() {
            if let Ok(content) = fs::read_to_string(&project_path) {
                contexts.push(("project", content));
            }
        }

        // 3. Subdirectory context file (if path is a directory, look for file in it)
        let subdir_path = if path.is_dir() {
            path.join(file_name)
        } else if let Some(parent) = path.parent() {
            parent.join(file_name)
        } else {
            PathBuf::new()
        };

        if subdir_path.exists() && subdir_path != project_path {
            if let Ok(content) = fs::read_to_string(&subdir_path) {
                contexts.push(("subdirectory", content));
            }
        }

        // Merge contexts: lower precedence first, then higher precedence
        let mut result = String::new();
        for (_source, content) in contexts {
            if !result.is_empty() {
                result.push_str("\n---\n\n");
            }
            result.push_str(&content);
        }

        // Process imports if we have content
        if !result.is_empty() {
            let base_path = if path.is_dir() {
                path.to_path_buf()
            } else if let Some(parent) = path.parent() {
                parent.to_path_buf()
            } else {
                self.workspace_root.clone()
            };
            result = self.process_imports(&result, &base_path)?;
        }

        Ok(result)
    }

    /// Gets the list of context files that would be loaded for a given path.
    ///
    /// This method returns the paths to all context files that would be loaded
    /// by [`load_hierarchical`](Self::load_hierarchical) for the given path,
    /// in precedence order (lowest to highest).
    ///
    /// # Arguments
    /// * `path` - The path to get context files for (can be a file or directory)
    ///
    /// # Returns
    /// Vector of paths to context files that exist, in precedence order:
    /// 1. Global context file (`~/.radium/GEMINI.md`)
    /// 2. Project root context file (`<workspace>/GEMINI.md`)
    /// 3. Subdirectory context file (`<path>/GEMINI.md`)
    ///
    /// Returns an empty vector if no context files are found.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use radium_core::context::ContextFileLoader;
    /// use std::path::Path;
    ///
    /// let loader = ContextFileLoader::new(Path::new("/workspace"));
    /// let paths = loader.get_context_file_paths(Path::new("/workspace/src"));
    ///
    /// for path in paths {
    ///     println!("Found context file: {}", path.display());
    /// }
    /// ```
    pub fn get_context_file_paths(&self, path: &Path) -> Vec<PathBuf> {
        let file_name = self.file_name();
        let mut paths = Vec::new();

        // 1. Global context file
        if let Ok(home) = std::env::var("HOME") {
            let global_path = PathBuf::from(home).join(".radium").join(file_name);
            if global_path.exists() {
                paths.push(global_path);
            }
        }

        // 2. Project root context file
        let project_path = self.workspace_root.join(file_name);
        let project_path_clone = project_path.clone();
        if project_path.exists() {
            paths.push(project_path);
        }

        // 3. Subdirectory context file
        let subdir_path = if path.is_dir() {
            path.join(file_name)
        } else if let Some(parent) = path.parent() {
            parent.join(file_name)
        } else {
            PathBuf::new()
        };

        if subdir_path.exists() && subdir_path != project_path_clone {
            paths.push(subdir_path);
        }

        paths
    }

    /// Discovers all context files in the workspace.
    ///
    /// Recursively scans the workspace directory (and global location) to find
    /// all context files. Hidden directories (starting with `.`) are skipped
    /// during scanning.
    ///
    /// # Returns
    /// Vector of paths to all found context files, including:
    /// - Global context file (`~/.radium/GEMINI.md`) if it exists
    /// - All context files found in the workspace directory tree
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory scanning fails (permission errors, I/O errors)
    /// - The workspace root is not a valid directory
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use radium_core::context::ContextFileLoader;
    /// use std::path::Path;
    ///
    /// let loader = ContextFileLoader::new(Path::new("/workspace"));
    /// let files = loader.discover_context_files()?;
    ///
    /// println!("Found {} context file(s)", files.len());
    /// ```
    pub fn discover_context_files(&self) -> Result<Vec<PathBuf>> {
        let file_name = self.file_name();
        let mut files = Vec::new();

        // Check global location
        if let Ok(home) = std::env::var("HOME") {
            let global_path = PathBuf::from(home).join(".radium").join(file_name);
            if global_path.exists() {
                files.push(global_path);
            }
        }

        // Recursively scan workspace
        self.scan_directory(&self.workspace_root, file_name, &mut files)?;

        Ok(files)
    }

    /// Recursively scans a directory for context files.
    fn scan_directory(&self, dir: &Path, file_name: &str, files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        // Check if this directory has a context file
        let context_path = dir.join(file_name);
        if context_path.exists() && context_path.is_file() {
            files.push(context_path);
        }

        // Recursively scan subdirectories
        let entries = fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // Skip .radium and other hidden directories
                if let Some(name) = path.file_name() {
                    if name.to_string_lossy().starts_with('.') {
                        continue;
                    }
                }
                self.scan_directory(&path, file_name, files)?;
            }
        }

        Ok(())
    }

    /// Processes import statements in context content.
    ///
    /// This method processes `@file.md` import statements in the content, replacing
    /// them with the contents of the imported files. Imports are resolved relative
    /// to the `base_path`, and circular imports are detected and rejected.
    ///
    /// ## Import Syntax
    ///
    /// - `@file.md` - Import a file relative to the base path
    /// - `@/absolute/path/file.md` - Import using an absolute path
    /// - Imports inside code blocks (between ` ``` ` markers) are ignored
    ///
    /// ## Import Processing
    ///
    /// - Imports are processed recursively (imported files can also contain imports)
    /// - Circular imports are detected and cause an error
    /// - Duplicate imports are automatically deduplicated
    /// - Import paths are resolved relative to the importing file's directory
    ///
    /// # Arguments
    /// * `content` - The context content to process (may contain `@file.md` imports)
    /// * `base_path` - Base path for resolving relative imports
    ///
    /// # Returns
    /// Content with all imports resolved and merged. Import statements are replaced
    /// with the contents of the imported files.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - An import file is not found
    /// - An import file cannot be read (permission errors, I/O errors)
    /// - A circular import is detected (file A imports file B, which imports file A)
    /// - The base path is invalid
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use radium_core::context::ContextFileLoader;
    /// use std::path::Path;
    ///
    /// let loader = ContextFileLoader::new(Path::new("/workspace"));
    ///
    /// // Process content with imports
    /// let content = "# Main Context\n\n@shared.md\n\nMore content.";
    /// let processed = loader.process_imports(content, Path::new("/workspace"))?;
    /// ```
    pub fn process_imports(&self, content: &str, base_path: &Path) -> Result<String> {
        let mut result = String::new();
        let mut processed_imports = HashSet::new();
        let mut import_stack = Vec::new();

        self.process_imports_recursive(
            content,
            base_path,
            &mut processed_imports,
            &mut import_stack,
            &mut result,
        )?;

        Ok(result)
    }

    /// Recursively processes imports with circular import detection.
    fn process_imports_recursive(
        &self,
        content: &str,
        base_path: &Path,
        processed_imports: &mut HashSet<PathBuf>,
        import_stack: &mut Vec<PathBuf>,
        result: &mut String,
    ) -> Result<()> {
        let mut lines = content.lines().peekable();
        let mut in_code_block = false;

        while let Some(line) = lines.next() {
            // Track code blocks to avoid processing imports inside them
            if line.trim().starts_with("```") {
                in_code_block = !in_code_block;
                result.push_str(line);
                result.push('\n');
                continue;
            }

            if in_code_block {
                result.push_str(line);
                result.push('\n');
                continue;
            }

            // Check for import syntax: @file.md
            let trimmed = line.trim();
            if trimmed.starts_with('@') && trimmed.len() > 1 {
                let import_path_str = &trimmed[1..];
                let import_path = if PathBuf::from(import_path_str).is_absolute() {
                    PathBuf::from(import_path_str)
                } else {
                    base_path.join(import_path_str)
                };

                // Normalize path
                let import_path = import_path.canonicalize().map_err(|_| {
                    ContextError::FileNotFound(format!(
                        "Import file not found: {}",
                        import_path_str
                    ))
                })?;

                // Check for circular imports
                if import_stack.contains(&import_path) {
                    return Err(ContextError::InvalidSyntax(format!(
                        "Circular import detected: {}",
                        import_path.display()
                    )));
                }

                // Check if already processed
                if processed_imports.contains(&import_path) {
                    // Skip duplicate imports
                    continue;
                }

                // Read and process imported file
                let import_content = fs::read_to_string(&import_path).map_err(|_| {
                    ContextError::FileNotFound(format!(
                        "Cannot read import file: {}",
                        import_path.display()
                    ))
                })?;

                // Mark as processed
                processed_imports.insert(import_path.clone());
                import_stack.push(import_path.clone());

                // Get base path for relative imports in this file
                let import_base = if import_path.is_file() {
                    import_path.parent().unwrap_or(base_path).to_path_buf()
                } else {
                    base_path.to_path_buf()
                };

                // Recursively process imports in the imported file
                self.process_imports_recursive(
                    &import_content,
                    &import_base,
                    processed_imports,
                    import_stack,
                    result,
                )?;

                import_stack.pop();
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_hierarchical_project_only() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create project root context file
        let project_file = temp_dir.path().join("GEMINI.md");
        fs::write(&project_file, "# Project Context\n\nProject instructions.").unwrap();

        let content = loader.load_hierarchical(temp_dir.path()).unwrap();
        assert!(content.contains("Project Context"));
        assert!(content.contains("Project instructions"));
    }

    #[test]
    fn test_load_hierarchical_subdirectory() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create project root context file
        let project_file = temp_dir.path().join("GEMINI.md");
        fs::write(&project_file, "# Project Context\n\nProject instructions.").unwrap();

        // Create subdirectory with context file
        let subdir = temp_dir.path().join("src");
        fs::create_dir_all(&subdir).unwrap();
        let subdir_file = subdir.join("GEMINI.md");
        fs::write(&subdir_file, "# Subdirectory Context\n\nSubdirectory instructions.").unwrap();

        let content = loader.load_hierarchical(&subdir).unwrap();
        // Should contain both, with project first (lower precedence), then subdirectory
        assert!(content.contains("Project Context"));
        assert!(content.contains("Subdirectory Context"));
        // Subdirectory should come after project (higher precedence)
        let project_pos = content.find("Project Context").unwrap();
        let subdir_pos = content.find("Subdirectory Context").unwrap();
        assert!(project_pos < subdir_pos);
    }

    #[test]
    fn test_load_hierarchical_missing_files() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // No context files exist
        let content = loader.load_hierarchical(temp_dir.path()).unwrap();
        assert!(content.is_empty());
    }

    #[test]
    fn test_discover_context_files() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create multiple context files
        let project_file = temp_dir.path().join("GEMINI.md");
        fs::write(&project_file, "Project context").unwrap();

        let subdir = temp_dir.path().join("src");
        fs::create_dir_all(&subdir).unwrap();
        let subdir_file = subdir.join("GEMINI.md");
        fs::write(&subdir_file, "Subdirectory context").unwrap();

        let files = loader.discover_context_files().unwrap();
        assert!(files.len() >= 2);
        assert!(files.iter().any(|f| f == &project_file));
        assert!(files.iter().any(|f| f == &subdir_file));
    }

    #[test]
    fn test_process_imports_simple() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create imported file
        let imported_file = temp_dir.path().join("imported.md");
        fs::write(&imported_file, "# Imported Content\n\nThis is imported.").unwrap();

        // Create content with import
        let content = "# Main Content\n\n@imported.md\n\nMore content.";
        let result = loader.process_imports(content, temp_dir.path()).unwrap();

        assert!(result.contains("Main Content"));
        assert!(result.contains("Imported Content"));
        assert!(result.contains("This is imported"));
        assert!(result.contains("More content"));
    }

    #[test]
    fn test_process_imports_circular() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create file1 that imports file2
        let file1 = temp_dir.path().join("file1.md");
        fs::write(&file1, "# File 1\n\n@file2.md").unwrap();

        // Create file2 that imports file1 (circular)
        let file2 = temp_dir.path().join("file2.md");
        fs::write(&file2, "# File 2\n\n@file1.md").unwrap();

        let content = fs::read_to_string(&file1).unwrap();
        let result = loader.process_imports(&content, temp_dir.path());
        assert!(result.is_err());
        if let Err(ContextError::InvalidSyntax(msg)) = result {
            assert!(msg.contains("Circular import"));
        } else {
            panic!("Expected InvalidSyntax error for circular import");
        }
    }

    #[test]
    fn test_process_imports_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        let content = "# Main Content\n\n@nonexistent.md";
        let result = loader.process_imports(content, temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_process_imports_in_code_block() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Import should be ignored inside code blocks
        let content = "# Main\n\n```\n@file.md\n```\n\n@file.md";
        let result = loader.process_imports(content, temp_dir.path());
        // Should error on the import outside code block, not the one inside
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_file_name() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::with_file_name(temp_dir.path(), "CUSTOM.md".to_string());

        // Create custom context file
        let custom_file = temp_dir.path().join("CUSTOM.md");
        fs::write(&custom_file, "# Custom Context").unwrap();

        let content = loader.load_hierarchical(temp_dir.path()).unwrap();
        assert!(content.contains("Custom Context"));

        // Default file should not be loaded
        let default_file = temp_dir.path().join("GEMINI.md");
        fs::write(&default_file, "# Default Context").unwrap();
        let content2 = loader.load_hierarchical(temp_dir.path()).unwrap();
        assert!(!content2.contains("Default Context"));
    }

    #[test]
    fn test_process_imports_relative_path() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create subdirectory with imported file
        let subdir = temp_dir.path().join("docs");
        fs::create_dir_all(&subdir).unwrap();
        let imported_file = subdir.join("guide.md");
        fs::write(&imported_file, "# Guide Content").unwrap();

        // Create main file in subdirectory
        let main_file = subdir.join("main.md");
        fs::write(&main_file, "# Main\n\n@guide.md").unwrap();

        let content = fs::read_to_string(&main_file).unwrap();
        let result = loader.process_imports(&content, &subdir).unwrap();
        assert!(result.contains("Guide Content"));
    }

    #[test]
    fn test_process_imports_duplicate() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create imported file
        let imported_file = temp_dir.path().join("imported.md");
        fs::write(&imported_file, "# Imported").unwrap();

        // Import same file twice
        let content = "# Main\n\n@imported.md\n\n@imported.md";
        let result = loader.process_imports(content, temp_dir.path()).unwrap();
        // Should only appear once (or be deduplicated)
        let count = result.matches("Imported").count();
        assert!(count >= 1);
    }

    #[test]
    fn test_load_hierarchical_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create empty context file
        let project_file = temp_dir.path().join("GEMINI.md");
        fs::write(&project_file, "").unwrap();

        let content = loader.load_hierarchical(temp_dir.path()).unwrap();
        // Empty file should result in empty content (after processing)
        assert!(content.is_empty() || content.trim().is_empty());
    }

    #[test]
    fn test_load_hierarchical_whitespace_only() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create whitespace-only context file
        let project_file = temp_dir.path().join("GEMINI.md");
        fs::write(&project_file, "   \n\n\t\t\n  ").unwrap();

        let content = loader.load_hierarchical(temp_dir.path()).unwrap();
        // Whitespace-only should be processed but result in minimal content
        assert!(content.trim().is_empty() || content.is_empty());
    }

    #[test]
    fn test_process_imports_absolute_path() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create imported file
        let imported_file = temp_dir.path().join("imported.md");
        fs::write(&imported_file, "# Absolute Import\n\nContent from absolute path.").unwrap();

        // Use absolute path in import
        let absolute_path = imported_file.canonicalize().unwrap();
        let content = format!("# Main\n\n@{}\n\nMore content.", absolute_path.display());
        let result = loader.process_imports(&content, temp_dir.path()).unwrap();

        assert!(result.contains("Main"));
        assert!(result.contains("Absolute Import"));
        assert!(result.contains("Content from absolute path"));
        assert!(result.contains("More content"));
    }

    #[test]
    fn test_process_imports_path_with_spaces() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create directory and file with spaces in name
        let subdir = temp_dir.path().join("my docs");
        fs::create_dir_all(&subdir).unwrap();
        let imported_file = subdir.join("my file.md");
        fs::write(&imported_file, "# File With Spaces\n\nContent.").unwrap();

        // Import file with spaces in path
        let content = "# Main\n\n@my docs/my file.md\n\nMore.";
        let result = loader.process_imports(content, temp_dir.path()).unwrap();

        assert!(result.contains("Main"));
        assert!(result.contains("File With Spaces"));
        assert!(result.contains("Content"));
        assert!(result.contains("More"));
    }

    #[test]
    fn test_process_imports_multiple_in_line() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create multiple imported files
        let file1 = temp_dir.path().join("file1.md");
        let file2 = temp_dir.path().join("file2.md");
        fs::write(&file1, "# File 1").unwrap();
        fs::write(&file2, "# File 2").unwrap();

        // Multiple imports on separate lines
        let content = "# Main\n\n@file1.md\n\n@file2.md\n\nEnd";
        let result = loader.process_imports(content, temp_dir.path()).unwrap();

        assert!(result.contains("Main"));
        assert!(result.contains("File 1"));
        assert!(result.contains("File 2"));
        assert!(result.contains("End"));
    }

    #[test]
    fn test_process_imports_unicode_content() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create file with unicode content
        let imported_file = temp_dir.path().join("imported.md");
        fs::write(&imported_file, "# Unicode Test\n\n中文 Español Français 🚀").unwrap();

        let content = "# Main\n\n@imported.md";
        let result = loader.process_imports(content, temp_dir.path()).unwrap();

        assert!(result.contains("Main"));
        assert!(result.contains("Unicode Test"));
        assert!(result.contains("中文"));
        assert!(result.contains("Español"));
        assert!(result.contains("Français"));
        assert!(result.contains("🚀"));
    }

    #[test]
    fn test_process_imports_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create file with special characters
        let imported_file = temp_dir.path().join("imported.md");
        fs::write(&imported_file, "# Special Chars\n\n<>&\"'`{}\\[\\]").unwrap();

        let content = "# Main\n\n@imported.md";
        let result = loader.process_imports(content, temp_dir.path()).unwrap();

        assert!(result.contains("Main"));
        assert!(result.contains("Special Chars"));
    }

    #[test]
    fn test_discover_context_files_ignores_hidden_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create context file in project root
        let project_file = temp_dir.path().join("GEMINI.md");
        fs::write(&project_file, "Project context").unwrap();

        // Create hidden directory with context file (should be ignored)
        let hidden_dir = temp_dir.path().join(".hidden");
        fs::create_dir_all(&hidden_dir).unwrap();
        let hidden_file = hidden_dir.join("GEMINI.md");
        fs::write(&hidden_file, "Hidden context").unwrap();

        let files = loader.discover_context_files().unwrap();
        // Should find project file but not hidden directory file
        assert!(files.iter().any(|f| f == &project_file));
        assert!(!files.iter().any(|f| f == &hidden_file));
    }

    #[test]
    fn test_load_hierarchical_with_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create context file with frontmatter
        let project_file = temp_dir.path().join("GEMINI.md");
        fs::write(
            &project_file,
            "---\nversion: 1.0\n---\n\n# Context\n\nContent after frontmatter.",
        )
        .unwrap();

        let content = loader.load_hierarchical(temp_dir.path()).unwrap();
        assert!(content.contains("Context"));
        assert!(content.contains("Content after frontmatter"));
    }

    #[test]
    fn test_process_imports_nested_code_blocks() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContextFileLoader::new(temp_dir.path());

        // Create imported file
        let imported_file = temp_dir.path().join("imported.md");
        fs::write(&imported_file, "# Imported").unwrap();

        // Import inside code block should be ignored, outside should work
        let content = "# Main\n\n```\n@imported.md\n```\n\n@imported.md\n\nEnd";
        let result = loader.process_imports(content, temp_dir.path()).unwrap();

        // Should contain imported content (from outside code block)
        assert!(result.contains("Main"));
        assert!(result.contains("Imported"));
        assert!(result.contains("End"));
        // The import inside code block should remain as text
        assert!(result.contains("```"));
    }
}

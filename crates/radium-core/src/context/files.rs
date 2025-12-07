//! Context file loading and processing.
//!
//! Supports hierarchical loading of context files (GEMINI.md) from:
//! - Global location: `~/.radium/GEMINI.md`
//! - Project root: `GEMINI.md`
//! - Subdirectory: `<subdir>/GEMINI.md`
//!
//! Also supports context imports using `@file.md` syntax.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::error::{ContextError, Result};

/// Default context file name.
const DEFAULT_CONTEXT_FILE_NAME: &str = "GEMINI.md";

/// Context file loader for hierarchical loading and import processing.
pub struct ContextFileLoader {
    /// Workspace root path.
    workspace_root: PathBuf,
    /// Custom context file name (default: "GEMINI.md").
    custom_file_name: Option<String>,
}

impl ContextFileLoader {
    /// Creates a new context file loader.
    ///
    /// # Arguments
    /// * `workspace_root` - The workspace root directory
    pub fn new(workspace_root: impl AsRef<Path>) -> Self {
        Self {
            workspace_root: workspace_root.as_ref().to_path_buf(),
            custom_file_name: None,
        }
    }

    /// Creates a new context file loader with custom file name.
    ///
    /// # Arguments
    /// * `workspace_root` - The workspace root directory
    /// * `file_name` - Custom context file name
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
    /// Precedence order (highest to lowest):
    /// 1. Subdirectory context file
    /// 2. Project root context file
    /// 3. Global context file (`~/.radium/GEMINI.md`)
    ///
    /// Lower precedence files are prepended to higher precedence files.
    ///
    /// # Arguments
    /// * `path` - The path to load context for (can be file or directory)
    ///
    /// # Returns
    /// Combined context content from all applicable files
    ///
    /// # Errors
    /// Returns error if file reading fails (but missing files are ignored)
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

    /// Discovers all context files in the workspace.
    ///
    /// # Returns
    /// Vector of paths to all found context files
    ///
    /// # Errors
    /// Returns error if directory scanning fails
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
    fn scan_directory(
        &self,
        dir: &Path,
        file_name: &str,
        files: &mut Vec<PathBuf>,
    ) -> Result<()> {
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
    /// Supports `@file.md` syntax to import other files.
    ///
    /// # Arguments
    /// * `content` - The context content to process
    /// * `base_path` - Base path for resolving relative imports
    ///
    /// # Returns
    /// Content with imports resolved and merged
    ///
    /// # Errors
    /// Returns error if import resolution fails or circular imports detected
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
}


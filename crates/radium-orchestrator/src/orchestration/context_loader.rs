//! Context file loader for orchestration
//!
//! This module provides a trait-based interface for loading context files
//! (GEMINI.md) to avoid circular dependencies with radium-core.

use std::path::PathBuf;

/// Trait for loading context files to avoid direct dependency on radium-core
pub trait ContextFileLoaderTrait: Send + Sync {
    /// Load hierarchical context files for a given path
    ///
    /// This should load context files following the hierarchical precedence:
    /// 1. Subdirectory context file (highest precedence)
    /// 2. Project root context file
    /// 3. Global context file (lowest precedence)
    ///
    /// # Arguments
    /// * `path` - The path to load context for (can be a file or directory)
    ///
    /// # Returns
    /// Combined context content from all applicable files, with imports resolved.
    /// Returns an empty string if no context files are found.
    fn load_hierarchical(&self, path: &std::path::Path) -> Result<String, String>;
}

/// Simple implementation that uses radium-core's ContextFileLoader
///
/// This is a wrapper that can be created from a workspace root.
/// The actual implementation should be provided by the application layer (TUI/CLI).
pub struct ContextFileLoaderAdapter {
    /// Workspace root path
    workspace_root: PathBuf,
}

impl ContextFileLoaderAdapter {
    /// Create a new context file loader adapter
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

impl ContextFileLoaderTrait for ContextFileLoaderAdapter {
    fn load_hierarchical(&self, _path: &std::path::Path) -> Result<String, String> {
        // This implementation would use radium_core::context::ContextFileLoader
        // but we can't import it here due to circular dependencies.
        // Instead, the application layer (TUI) should provide an implementation
        // that wraps radium_core::context::ContextFileLoader.
        
        // For now, return empty string - actual implementation will be provided
        // by the TUI layer that has access to radium-core
        Ok(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_context_file_loader_adapter_creation() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = ContextFileLoaderAdapter::new(temp_dir.path().to_path_buf());
        
        // Should not panic
        let result = adapter.load_hierarchical(temp_dir.path());
        assert!(result.is_ok());
    }
}


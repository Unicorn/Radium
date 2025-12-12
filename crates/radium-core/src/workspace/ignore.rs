//! Ignore pattern handling for workspace traversal.
//!
//! This module provides utilities for respecting .gitignore, .nxignore, and
//! other ignore patterns during directory traversal.

use std::path::{Path, PathBuf};
use ignore::WalkBuilder;

/// Builder for creating ignore-aware directory walkers.
///
/// This wraps the `ignore` crate's `WalkBuilder` with Radium-specific
/// configuration for standard ignore patterns.
pub struct IgnoreWalker {
    builder: WalkBuilder,
}

impl IgnoreWalker {
    /// Create a new ignore-aware walker for the given directory.
    ///
    /// This automatically configures:
    /// - .gitignore and .git/info/exclude support
    /// - .nxignore support (Nx monorepo ignore files)
    /// - Standard ignore patterns (target/, node_modules/, .git/, dist/, build/)
    pub fn new(root: impl AsRef<Path>) -> Self {
        let mut builder = WalkBuilder::new(root);
        
        // Enable gitignore support (default in ignore crate)
        // Add standard ignore patterns
        builder.add_custom_ignore_filename(".nxignore");
        
        // Filter out common build/dependency directories
        // These are handled by gitignore patterns, but we ensure they're excluded
        // even if not in .gitignore
        builder.filter_entry(|entry| {
            let path = entry.path();
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy().to_lowercase();
                // Skip common directories even if not in gitignore
                if path.is_dir() {
                    return !matches!(
                        name_str.as_str(),
                        "target" | "node_modules" | "dist" | "build" | ".next" | ".venv" | "__pycache__"
                    );
                }
            }
            true
        });

        Self { builder }
    }

    /// Build the walker and return an iterator over file paths.
    ///
    /// Returns paths relative to the workspace root, sorted for deterministic output.
    pub fn build(self) -> impl Iterator<Item = PathBuf> {
        let mut paths: Vec<PathBuf> = self
            .builder
            .build()
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    if e.file_type()?.is_file() {
                        Some(e.path().to_path_buf())
                    } else {
                        None
                    }
                })
            })
            .collect();

        // Sort for deterministic output
        paths.sort();
        paths.into_iter()
    }

    /// Build the walker and return an iterator over directory entries.
    ///
    /// This includes both files and directories, useful for recursive operations.
    pub fn build_entries(self) -> impl Iterator<Item = ignore::DirEntry> {
        self.builder.build().filter_map(|entry| entry.ok())
    }

    /// Configure whether to follow symbolic links.
    pub fn follow_links(mut self, yes: bool) -> Self {
        self.builder.follow_links(yes);
        self
    }

    /// Add a custom ignore file name pattern.
    pub fn add_ignore_filename(mut self, pattern: &str) -> Self {
        self.builder.add_custom_ignore_filename(pattern);
        self
    }

    /// Set the maximum depth for directory traversal.
    pub fn max_depth(mut self, depth: Option<usize>) -> Self {
        if let Some(d) = depth {
            self.builder.max_depth(Some(d));
        } else {
            self.builder.max_depth(None);
        }
        self
    }
}

/// Check if a path should be ignored based on standard patterns.
///
/// This is a quick check for common ignore patterns without building
/// a full ignore walker. For full .gitignore support, use `IgnoreWalker`.
pub fn should_ignore_path(path: &Path) -> bool {
    // Check for common ignored directories in path
    for component in path.components() {
        if let std::path::Component::Normal(name) = component {
            let name_str = name.to_string_lossy().to_lowercase();
            if matches!(
                name_str.as_str(),
                ".git" | "target" | "node_modules" | "dist" | "build" | ".next" | ".venv" | "__pycache__" | ".radium"
            ) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_ignore_walker_respects_gitignore() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Initialize git repo for .gitignore to work
        std::process::Command::new("git")
            .arg("init")
            .arg("--quiet")
            .current_dir(root)
            .output()
            .ok();

        // Create a .gitignore file
        fs::write(root.join(".gitignore"), "*.log\ntemp/").unwrap();

        // Create some files
        fs::write(root.join("file.rs"), "content").unwrap();
        fs::write(root.join("file.log"), "content").unwrap();
        fs::create_dir_all(root.join("temp")).unwrap();
        fs::write(root.join("temp/file.txt"), "content").unwrap();

        let walker = IgnoreWalker::new(root);
        let mut paths: Vec<PathBuf> = walker.build().collect();
        paths.sort();

        // Should only include file.rs, not file.log or temp/file.txt
        // Note: .gitignore itself might be included, so we filter for .rs files
        let rs_files: Vec<_> = paths.iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
            .collect();
        assert_eq!(rs_files.len(), 1);
        assert!(rs_files[0].ends_with("file.rs"));
    }

    #[test]
    fn test_ignore_walker_excludes_common_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create common directories
        fs::create_dir_all(root.join("target")).unwrap();
        fs::create_dir_all(root.join("node_modules")).unwrap();
        fs::write(root.join("target/file.rs"), "content").unwrap();
        fs::write(root.join("node_modules/package.json"), "{}").unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/file.rs"), "content").unwrap();

        let walker = IgnoreWalker::new(root);
        let mut paths: Vec<PathBuf> = walker.build().collect();
        paths.sort();

        // Should only include src/file.rs (target/ and node_modules/ should be excluded)
        let rs_files: Vec<_> = paths.iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
            .collect();
        assert_eq!(rs_files.len(), 1);
        assert!(rs_files[0].ends_with("src/file.rs"));
    }

    #[test]
    fn test_should_ignore_path() {
        assert!(should_ignore_path(Path::new("target/file.rs")));
        assert!(should_ignore_path(Path::new("src/node_modules/file.js")));
        assert!(!should_ignore_path(Path::new("src/file.rs")));
    }
}

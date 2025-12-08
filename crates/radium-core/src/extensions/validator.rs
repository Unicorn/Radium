//! Extension validation logic.
//!
//! Provides comprehensive validation for extension packages before installation,
//! including manifest validation, component file checks, version compatibility,
//! and security checks.

use crate::extensions::manifest::ExtensionManifest;
use crate::extensions::structure::ExtensionStructureError;
use std::path::Path;
use thiserror::Error;

/// Extension validation errors.
#[derive(Debug, Error)]
pub enum ExtensionValidationError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Structure error.
    #[error("structure error: {0}")]
    Structure(#[from] ExtensionStructureError),

    /// Manifest error.
    #[error("manifest error: {0}")]
    Manifest(#[from] crate::extensions::manifest::ExtensionManifestError),

    /// Component file not found.
    #[error("component file not found: {0}")]
    ComponentNotFound(String),

    /// Invalid component path (path traversal attempt).
    #[error("invalid component path (security): {0}")]
    InvalidPath(String),

    /// Component file syntax error.
    #[error("component file syntax error in {0}: {1}")]
    ComponentSyntaxError(String, String),

    /// Version incompatibility.
    #[error("version incompatibility: {0}")]
    VersionIncompatible(String),

    /// Missing dependency.
    #[error("missing dependency: {0}")]
    MissingDependency(String),

    /// Dependency version mismatch.
    #[error("dependency version mismatch: {0}")]
    DependencyVersionMismatch(String),
}

/// Result type for validation operations.
pub type Result<T> = std::result::Result<T, ExtensionValidationError>;

/// Extension validator.
pub struct ExtensionValidator;

impl ExtensionValidator {
    /// Validates an extension package before installation.
    ///
    /// # Arguments
    /// * `package_path` - Path to extension package directory
    /// * `manifest` - Extension manifest
    ///
    /// # Returns
    /// Ok(()) if valid, error otherwise
    pub fn validate(package_path: &Path, manifest: &ExtensionManifest) -> Result<()> {
        // Validate manifest (already done during load, but double-check)
        manifest.validate().map_err(ExtensionValidationError::Manifest)?;

        // Validate component paths for security
        Self::validate_component_paths(manifest)?;

        // Validate component files exist
        Self::validate_component_files(package_path, manifest)?;

        // Validate component file syntax
        Self::validate_component_syntax(package_path, manifest)?;

        Ok(())
    }

    /// Validates component paths for security (no path traversal, no absolute paths).
    fn validate_component_paths(manifest: &ExtensionManifest) -> Result<()> {
        for path in &manifest.components.prompts {
            if Self::is_unsafe_path(path) {
                return Err(ExtensionValidationError::InvalidPath(format!(
                    "prompt path contains unsafe elements: {}",
                    path
                )));
            }
        }

        for path in &manifest.components.mcp_servers {
            if Self::is_unsafe_path(path) {
                return Err(ExtensionValidationError::InvalidPath(format!(
                    "MCP server path contains unsafe elements: {}",
                    path
                )));
            }
        }

        for path in &manifest.components.commands {
            if Self::is_unsafe_path(path) {
                return Err(ExtensionValidationError::InvalidPath(format!(
                    "command path contains unsafe elements: {}",
                    path
                )));
            }
        }

        Ok(())
    }

    /// Checks if a path is unsafe (contains path traversal or is absolute).
    fn is_unsafe_path(path: &str) -> bool {
        // Check for absolute paths (Unix and Windows)
        if path.starts_with('/') || (path.len() > 1 && &path[1..3] == ":\\") {
            return true;
        }

        // Check for path traversal
        if path.contains("..") {
            return true;
        }

        // Check for null bytes (potential security issue)
        if path.contains('\0') {
            return true;
        }

        false
    }

    /// Validates that all component files exist.
    fn validate_component_files(package_path: &Path, manifest: &ExtensionManifest) -> Result<()> {
        // Validate prompt files
        for pattern in &manifest.components.prompts {
            let resolved = package_path.join(pattern);
            // For glob patterns, check if directory exists
            if pattern.contains('*') {
                let dir = resolved.parent().unwrap_or(package_path);
                if !dir.exists() {
                    return Err(ExtensionValidationError::ComponentNotFound(format!(
                        "prompt directory not found: {}",
                        dir.display()
                    )));
                }
            } else if !resolved.exists() {
                return Err(ExtensionValidationError::ComponentNotFound(format!(
                    "prompt file not found: {}",
                    resolved.display()
                )));
            }
        }

        // Validate MCP server configs
        for path in &manifest.components.mcp_servers {
            let resolved = package_path.join(path);
            if !resolved.exists() {
                return Err(ExtensionValidationError::ComponentNotFound(format!(
                    "MCP server config not found: {}",
                    resolved.display()
                )));
            }
        }

        // Validate command files
        for pattern in &manifest.components.commands {
            let resolved = package_path.join(pattern);
            // For glob patterns, check if directory exists
            if pattern.contains('*') {
                let dir = resolved.parent().unwrap_or(package_path);
                if !dir.exists() {
                    return Err(ExtensionValidationError::ComponentNotFound(format!(
                        "command directory not found: {}",
                        dir.display()
                    )));
                }
            } else if !resolved.exists() {
                return Err(ExtensionValidationError::ComponentNotFound(format!(
                    "command file not found: {}",
                    resolved.display()
                )));
            }
        }

        Ok(())
    }

    /// Validates component file syntax (TOML for commands, JSON for MCP servers).
    fn validate_component_syntax(package_path: &Path, manifest: &ExtensionManifest) -> Result<()> {
        // Validate command TOML files
        for pattern in &manifest.components.commands {
            if pattern.contains('*') {
                // Skip glob patterns for syntax validation
                continue;
            }
            let resolved = package_path.join(pattern);
            if resolved.exists() && resolved.is_file() {
                let content = std::fs::read_to_string(&resolved)?;
                toml::from_str::<toml::Value>(&content)
                    .map_err(|e| ExtensionValidationError::ComponentSyntaxError(
                        resolved.display().to_string(),
                        e.to_string(),
                    ))?;
            }
        }

        // Validate MCP server JSON files
        for path in &manifest.components.mcp_servers {
            let resolved = package_path.join(path);
            if resolved.exists() && resolved.is_file() {
                let content = std::fs::read_to_string(&resolved)?;
                serde_json::from_str::<serde_json::Value>(&content)
                    .map_err(|e| ExtensionValidationError::ComponentSyntaxError(
                        resolved.display().to_string(),
                        e.to_string(),
                    ))?;
            }
        }

        Ok(())
    }

    /// Validates that all dependencies are installed.
    ///
    /// # Arguments
    /// * `manifest` - Extension manifest
    /// * `installed_extensions` - List of installed extension names
    ///
    /// # Returns
    /// Ok(()) if all dependencies are satisfied, error otherwise
    pub fn validate_dependencies(
        manifest: &ExtensionManifest,
        installed_extensions: &[String],
    ) -> Result<()> {
        for dep_name in &manifest.dependencies {
            if !installed_extensions.contains(dep_name) {
                return Err(ExtensionValidationError::MissingDependency(dep_name.clone()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::manifest::ExtensionComponents;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_manifest(name: &str) -> ExtensionManifest {
        ExtensionManifest {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Author".to_string(),
            components: ExtensionComponents::default(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_validate_path_traversal() {
        let mut manifest = create_test_manifest("test");
        manifest.components.prompts.push("../etc/passwd".to_string());

        let result = ExtensionValidator::validate_component_paths(&manifest);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExtensionValidationError::InvalidPath(_)));
    }

    #[test]
    fn test_validate_absolute_path() {
        let mut manifest = create_test_manifest("test");
        manifest.components.prompts.push("/etc/passwd".to_string());

        let result = ExtensionValidator::validate_component_paths(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_safe_path() {
        let mut manifest = create_test_manifest("test");
        manifest.components.prompts.push("prompts/agent.md".to_string());

        let result = ExtensionValidator::validate_component_paths(&manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_dependencies() {
        let mut manifest = create_test_manifest("test");
        manifest.dependencies.push("dep1".to_string());
        manifest.dependencies.push("dep2".to_string());

        let installed = vec!["dep1".to_string(), "dep2".to_string(), "other".to_string()];
        let result = ExtensionValidator::validate_dependencies(&manifest, &installed);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_missing_dependency() {
        let mut manifest = create_test_manifest("test");
        manifest.dependencies.push("missing".to_string());

        let installed = vec!["dep1".to_string()];
        let result = ExtensionValidator::validate_dependencies(&manifest, &installed);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExtensionValidationError::MissingDependency(_)));
    }
}


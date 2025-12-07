//! Extension structure and directory organization.
//!
//! Defines the directory structure for extension packages and
//! provides utilities for path resolution and component discovery.

use crate::extensions::manifest::ExtensionManifest;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Extension structure errors.
#[derive(Debug, Error)]
pub enum ExtensionStructureError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid extension structure.
    #[error("invalid extension structure: {0}")]
    InvalidStructure(String),

    /// Manifest error.
    #[error("manifest error: {0}")]
    Manifest(#[from] crate::extensions::manifest::ExtensionManifestError),
}

/// Result type for structure operations.
pub type Result<T> = std::result::Result<T, ExtensionStructureError>;

/// Component directory names.
pub const COMPONENT_PROMPTS: &str = "prompts";
pub const COMPONENT_MCP: &str = "mcp";
pub const COMPONENT_COMMANDS: &str = "commands";

/// Manifest file name.
pub const MANIFEST_FILE: &str = "radium-extension.json";

/// Represents an installed extension with its metadata and location.
#[derive(Debug, Clone)]
pub struct Extension {
    /// Extension manifest.
    pub manifest: ExtensionManifest,

    /// Installation directory path.
    pub install_path: PathBuf,

    /// Extension name (from manifest).
    pub name: String,

    /// Extension version (from manifest).
    pub version: String,
}

impl Extension {
    /// Creates a new Extension from a manifest and installation path.
    ///
    /// # Arguments
    /// * `manifest` - Extension manifest
    /// * `install_path` - Installation directory path
    ///
    /// # Returns
    /// New Extension instance
    pub fn new(manifest: ExtensionManifest, install_path: PathBuf) -> Self {
        let name = manifest.name.clone();
        let version = manifest.version.clone();

        Self {
            manifest,
            install_path,
            name,
            version,
        }
    }

    /// Gets the prompts directory path.
    pub fn prompts_dir(&self) -> PathBuf {
        self.install_path.join(COMPONENT_PROMPTS)
    }

    /// Gets the MCP servers directory path.
    pub fn mcp_dir(&self) -> PathBuf {
        self.install_path.join(COMPONENT_MCP)
    }

    /// Gets the commands directory path.
    pub fn commands_dir(&self) -> PathBuf {
        self.install_path.join(COMPONENT_COMMANDS)
    }

    /// Gets the manifest file path.
    pub fn manifest_path(&self) -> PathBuf {
        self.install_path.join(MANIFEST_FILE)
    }

    /// Validates the extension structure.
    ///
    /// Checks that:
    /// - Manifest file exists
    /// - Component directories exist if components are declared
    ///
    /// # Returns
    /// Ok(()) if valid, error otherwise
    pub fn validate_structure(&self) -> Result<()> {
        // Check manifest exists
        if !self.manifest_path().exists() {
            return Err(ExtensionStructureError::InvalidStructure(format!(
                "manifest file not found: {}",
                self.manifest_path().display()
            )));
        }

        // Validate component directories if components are declared
        if !self.manifest.components.prompts.is_empty() {
            let prompts_dir = self.prompts_dir();
            if !prompts_dir.exists() {
                return Err(ExtensionStructureError::InvalidStructure(format!(
                    "prompts directory not found: {}",
                    prompts_dir.display()
                )));
            }
        }

        if !self.manifest.components.mcp_servers.is_empty() {
            let mcp_dir = self.mcp_dir();
            if !mcp_dir.exists() {
                return Err(ExtensionStructureError::InvalidStructure(format!(
                    "MCP directory not found: {}",
                    mcp_dir.display()
                )));
            }
        }

        if !self.manifest.components.commands.is_empty() {
            let commands_dir = self.commands_dir();
            if !commands_dir.exists() {
                return Err(ExtensionStructureError::InvalidStructure(format!(
                    "commands directory not found: {}",
                    commands_dir.display()
                )));
            }
        }

        Ok(())
    }

    /// Resolves a component path relative to the extension installation directory.
    ///
    /// # Arguments
    /// * `component_path` - Component path from manifest (may include glob patterns)
    ///
    /// # Returns
    /// Resolved absolute path
    pub fn resolve_component_path(&self, component_path: &str) -> PathBuf {
        self.install_path.join(component_path)
    }

    /// Gets all prompt file paths based on manifest patterns.
    pub fn get_prompt_paths(&self) -> Result<Vec<PathBuf>> {
        let prompts_dir = self.prompts_dir();
        let mut paths = Vec::new();

        for pattern in &self.manifest.components.prompts {
            let resolved = self.resolve_component_path(pattern);
            if resolved.exists() && resolved.is_file() {
                paths.push(resolved);
            } else if prompts_dir.exists() {
                // Try to match pattern in prompts directory
                let glob_pattern = prompts_dir.join(pattern);
                if let Ok(matched) = glob::glob(&glob_pattern.to_string_lossy()) {
                    for entry in matched.flatten() {
                        if entry.is_file() {
                            paths.push(entry);
                        }
                    }
                }
            }
        }

        Ok(paths)
    }

    /// Gets all MCP server configuration paths.
    pub fn get_mcp_paths(&self) -> Result<Vec<PathBuf>> {
        let mcp_dir = self.mcp_dir();
        let mut paths = Vec::new();

        for pattern in &self.manifest.components.mcp_servers {
            let resolved = self.resolve_component_path(pattern);
            if resolved.exists() && resolved.is_file() {
                paths.push(resolved);
            } else if mcp_dir.exists() {
                let glob_pattern = mcp_dir.join(pattern);
                if let Ok(matched) = glob::glob(&glob_pattern.to_string_lossy()) {
                    for entry in matched.flatten() {
                        if entry.is_file() {
                            paths.push(entry);
                        }
                    }
                }
            }
        }

        Ok(paths)
    }

    /// Gets all command file paths.
    pub fn get_command_paths(&self) -> Result<Vec<PathBuf>> {
        let commands_dir = self.commands_dir();
        let mut paths = Vec::new();

        for pattern in &self.manifest.components.commands {
            let resolved = self.resolve_component_path(pattern);
            if resolved.exists() && resolved.is_file() {
                paths.push(resolved);
            } else if commands_dir.exists() {
                let glob_pattern = commands_dir.join(pattern);
                if let Ok(matched) = glob::glob(&glob_pattern.to_string_lossy()) {
                    for entry in matched.flatten() {
                        if entry.is_file() {
                            paths.push(entry);
                        }
                    }
                }
            }
        }

        Ok(paths)
    }
}

/// Gets the default user-level extensions directory.
///
/// # Returns
/// Path to `~/.radium/extensions/`
///
/// # Errors
/// Returns error if HOME environment variable is not set
pub fn default_extensions_dir() -> std::result::Result<PathBuf, ExtensionStructureError> {
    #[allow(clippy::disallowed_methods)]
    let home = std::env::var("HOME")
        .map_err(|_| ExtensionStructureError::InvalidStructure("HOME not set".to_string()))?;
    Ok(Path::new(&home).join(".radium").join("extensions"))
}

/// Gets the workspace-level extensions directory.
///
/// # Arguments
/// * `workspace_root` - Workspace root directory
///
/// # Returns
/// Path to `.radium/extensions/` in workspace
pub fn workspace_extensions_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".radium").join("extensions")
}

/// Validates an extension package structure before installation.
///
/// # Arguments
/// * `package_path` - Path to extension package directory
///
/// # Returns
/// Ok(()) if structure is valid, error otherwise
pub fn validate_package_structure(package_path: &Path) -> Result<()> {
    // Check manifest exists
    let manifest_path = package_path.join(MANIFEST_FILE);
    if !manifest_path.exists() {
        return Err(ExtensionStructureError::InvalidStructure(format!(
            "manifest file not found: {}",
            manifest_path.display()
        )));
    }

    // Load and validate manifest
    let manifest = ExtensionManifest::load(&manifest_path)?;

    // Check component directories exist if declared
    if !manifest.components.prompts.is_empty() {
        let prompts_dir = package_path.join(COMPONENT_PROMPTS);
        if !prompts_dir.exists() {
            return Err(ExtensionStructureError::InvalidStructure(format!(
                "prompts directory not found: {}",
                prompts_dir.display()
            )));
        }
    }

    if !manifest.components.mcp_servers.is_empty() {
        let mcp_dir = package_path.join(COMPONENT_MCP);
        if !mcp_dir.exists() {
            return Err(ExtensionStructureError::InvalidStructure(format!(
                "MCP directory not found: {}",
                mcp_dir.display()
            )));
        }
    }

    if !manifest.components.commands.is_empty() {
        let commands_dir = package_path.join(COMPONENT_COMMANDS);
        if !commands_dir.exists() {
            return Err(ExtensionStructureError::InvalidStructure(format!(
                "commands directory not found: {}",
                commands_dir.display()
            )));
        }
    }

    Ok(())
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
            description: "Test extension".to_string(),
            author: "Test Author".to_string(),
            components: ExtensionComponents::default(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_extension_new() {
        let manifest = create_test_manifest("test-ext");
        let install_path = PathBuf::from("/extensions/test-ext");

        let extension = Extension::new(manifest.clone(), install_path.clone());

        assert_eq!(extension.name, "test-ext");
        assert_eq!(extension.version, "1.0.0");
        assert_eq!(extension.install_path, install_path);
    }

    #[test]
    fn test_extension_dir_paths() {
        let manifest = create_test_manifest("test-ext");
        let install_path = PathBuf::from("/extensions/test-ext");
        let extension = Extension::new(manifest, install_path);

        assert_eq!(extension.prompts_dir(), PathBuf::from("/extensions/test-ext/prompts"));
        assert_eq!(extension.mcp_dir(), PathBuf::from("/extensions/test-ext/mcp"));
        assert_eq!(extension.commands_dir(), PathBuf::from("/extensions/test-ext/commands"));
        assert_eq!(
            extension.manifest_path(),
            PathBuf::from("/extensions/test-ext/radium-extension.json")
        );
    }

    #[test]
    fn test_extension_resolve_component_path() {
        let manifest = create_test_manifest("test-ext");
        let install_path = PathBuf::from("/extensions/test-ext");
        let extension = Extension::new(manifest, install_path);

        let resolved = extension.resolve_component_path("prompts/agent.md");
        assert_eq!(resolved, PathBuf::from("/extensions/test-ext/prompts/agent.md"));
    }

    #[test]
    fn test_default_extensions_dir() {
        // Test with a mock HOME - this will fail if HOME is not set in test environment
        // which is acceptable as the function requires HOME to be set
        // TODO: Use a test helper that can set env vars safely
        if std::env::var("HOME").is_ok() {
            let dir = default_extensions_dir();
            // Should succeed if HOME is set
            assert!(dir.is_ok());
        } else {
            // If HOME is not set, the function should fail
            let dir = default_extensions_dir();
            assert!(dir.is_err());
        }
    }

    #[test]
    fn test_workspace_extensions_dir() {
        let workspace = PathBuf::from("/workspace");
        let dir = workspace_extensions_dir(&workspace);
        assert_eq!(dir, PathBuf::from("/workspace/.radium/extensions"));
    }

    #[test]
    fn test_validate_package_structure_valid() {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path();

        // Create manifest
        let manifest_path = package_path.join(MANIFEST_FILE);
        let manifest_json = r#"{
            "name": "test-ext",
            "version": "1.0.0",
            "description": "Test",
            "author": "Author",
            "components": {
                "prompts": ["prompts/*.md"]
            }
        }"#;
        std::fs::write(&manifest_path, manifest_json).unwrap();

        // Create prompts directory
        let prompts_dir = package_path.join("prompts");
        std::fs::create_dir(&prompts_dir).unwrap();

        let result = validate_package_structure(package_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_package_structure_missing_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path();

        let result = validate_package_structure(package_path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExtensionStructureError::InvalidStructure(_)
        ));
    }

    #[test]
    fn test_validate_package_structure_missing_component_dir() {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path();

        // Create manifest with prompts but no directory
        let manifest_path = package_path.join(MANIFEST_FILE);
        let manifest_json = r#"{
            "name": "test-ext",
            "version": "1.0.0",
            "description": "Test",
            "author": "Author",
            "components": {
                "prompts": ["prompts/*.md"]
            }
        }"#;
        std::fs::write(&manifest_path, manifest_json).unwrap();

        let result = validate_package_structure(package_path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExtensionStructureError::InvalidStructure(_)
        ));
    }

    #[test]
    fn test_extension_validate_structure() {
        let temp_dir = TempDir::new().unwrap();
        let install_path = temp_dir.path().to_path_buf();

        // Create manifest
        let manifest_path = install_path.join(MANIFEST_FILE);
        let manifest_json = r#"{
            "name": "test-ext",
            "version": "1.0.0",
            "description": "Test",
            "author": "Author"
        }"#;
        std::fs::write(&manifest_path, manifest_json).unwrap();

        let manifest = ExtensionManifest::load(&manifest_path).unwrap();
        let extension = Extension::new(manifest, install_path);

        let result = extension.validate_structure();
        assert!(result.is_ok());
    }

    #[test]
    fn test_extension_get_prompt_paths_empty() {
        let manifest = create_test_manifest("test-ext");
        let install_path = PathBuf::from("/extensions/test-ext");
        let extension = Extension::new(manifest, install_path);

        let paths = extension.get_prompt_paths().unwrap();
        assert!(paths.is_empty());
    }

    #[test]
    fn test_extension_get_mcp_paths_empty() {
        let manifest = create_test_manifest("test-ext");
        let install_path = PathBuf::from("/extensions/test-ext");
        let extension = Extension::new(manifest, install_path);

        let paths = extension.get_mcp_paths().unwrap();
        assert!(paths.is_empty());
    }

    #[test]
    fn test_extension_get_command_paths_empty() {
        let manifest = create_test_manifest("test-ext");
        let install_path = PathBuf::from("/extensions/test-ext");
        let extension = Extension::new(manifest, install_path);

        let paths = extension.get_command_paths().unwrap();
        assert!(paths.is_empty());
    }
}


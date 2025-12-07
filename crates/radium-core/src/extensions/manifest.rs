//! Extension manifest format and validation.
//!
//! Defines the structure and validation for extension manifest files
//! (radium-extension.json).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Extension manifest errors.
#[derive(Debug, Error)]
pub enum ExtensionManifestError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing error.
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// Invalid manifest format.
    #[error("invalid manifest format: {0}")]
    InvalidFormat(String),

    /// Missing required field.
    #[error("missing required field: {0}")]
    MissingField(String),

    /// Invalid version format.
    #[error("invalid version format: {0}")]
    InvalidVersion(String),

    /// Invalid component path.
    #[error("invalid component path: {0}")]
    InvalidComponentPath(String),

    /// Manifest file not found.
    #[error("manifest file not found: {0}")]
    NotFound(String),
}

/// Result type for manifest operations.
pub type Result<T> = std::result::Result<T, ExtensionManifestError>;

/// Extension component definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionComponents {
    /// Prompt file paths (glob patterns).
    #[serde(default)]
    pub prompts: Vec<String>,

    /// MCP server configuration file paths.
    #[serde(default)]
    pub mcp_servers: Vec<String>,

    /// Custom command file paths (glob patterns).
    #[serde(default)]
    pub commands: Vec<String>,
}

impl Default for ExtensionComponents {
    fn default() -> Self {
        Self { prompts: Vec::new(), mcp_servers: Vec::new(), commands: Vec::new() }
    }
}

/// Extension manifest structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    /// Extension name (must be unique).
    pub name: String,

    /// Extension version (semver format).
    pub version: String,

    /// Extension description.
    pub description: String,

    /// Extension author.
    pub author: String,

    /// Extension components (prompts, MCP servers, commands).
    #[serde(default)]
    pub components: ExtensionComponents,

    /// Extension dependencies (other extension names).
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// Optional extension metadata.
    #[serde(default, flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ExtensionManifest {
    /// Loads an extension manifest from a JSON file.
    ///
    /// # Arguments
    /// * `path` - Path to the manifest file (radium-extension.json)
    ///
    /// # Returns
    /// The parsed and validated manifest
    ///
    /// # Errors
    /// Returns error if file cannot be read, parsed, or validated
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(ExtensionManifestError::NotFound(path.to_string_lossy().to_string()));
        }

        let content = std::fs::read_to_string(path)?;
        let manifest: ExtensionManifest = serde_json::from_str(&content)?;

        manifest.validate()?;

        Ok(manifest)
    }

    /// Loads an extension manifest from JSON string.
    ///
    /// # Arguments
    /// * `json` - JSON string content
    ///
    /// # Returns
    /// The parsed and validated manifest
    ///
    /// # Errors
    /// Returns error if JSON cannot be parsed or validated
    pub fn from_json(json: &str) -> Result<Self> {
        let manifest: ExtensionManifest = serde_json::from_str(json)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validates the manifest structure and content.
    ///
    /// # Returns
    /// Ok(()) if valid, error otherwise
    ///
    /// # Errors
    /// Returns error if validation fails
    pub fn validate(&self) -> Result<()> {
        // Validate required fields
        if self.name.is_empty() {
            return Err(ExtensionManifestError::MissingField("name".to_string()));
        }

        if self.version.is_empty() {
            return Err(ExtensionManifestError::MissingField("version".to_string()));
        }

        if self.description.is_empty() {
            return Err(ExtensionManifestError::MissingField("description".to_string()));
        }

        if self.author.is_empty() {
            return Err(ExtensionManifestError::MissingField("author".to_string()));
        }

        // Validate version format (basic semver check)
        if !Self::is_valid_version(&self.version) {
            return Err(ExtensionManifestError::InvalidVersion(self.version.clone()));
        }

        // Validate extension name format (alphanumeric, dash, underscore only)
        if !Self::is_valid_name(&self.name) {
            return Err(ExtensionManifestError::InvalidFormat(format!(
                "invalid extension name: '{}' (must be alphanumeric with dashes/underscores)",
                self.name
            )));
        }

        // Validate component paths
        for path in &self.components.prompts {
            if path.trim().is_empty() {
                return Err(ExtensionManifestError::InvalidComponentPath(
                    "empty prompt path".to_string(),
                ));
            }
        }

        for path in &self.components.mcp_servers {
            if path.trim().is_empty() {
                return Err(ExtensionManifestError::InvalidComponentPath(
                    "empty MCP server path".to_string(),
                ));
            }
        }

        for path in &self.components.commands {
            if path.trim().is_empty() {
                return Err(ExtensionManifestError::InvalidComponentPath(
                    "empty command path".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Checks if a version string is valid (basic semver validation).
    fn is_valid_version(version: &str) -> bool {
        // Basic semver pattern: major.minor.patch or major.minor.patch-prerelease
        // Allow simple version numbers like "1.0.0", "1.0", "1"
        let parts: Vec<&str> = version.split('.').collect();
        if parts.is_empty() || parts.len() > 3 {
            return false;
        }

        // Check for prerelease/build metadata
        if let Some(first_part) = parts.first() {
            if first_part.contains('-') || first_part.contains('+') {
                return false; // Prerelease/build in first part is invalid
            }
        }

        // Validate each part is numeric (or has valid prerelease)
        for part in &parts {
            let num_part =
                if part.contains('-') { part.split('-').next().unwrap_or(part) } else { part };

            if num_part.is_empty() || !num_part.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
        }

        true
    }

    /// Checks if an extension name is valid.
    fn is_valid_name(name: &str) -> bool {
        // Allow alphanumeric, dash, underscore
        // Must start with a letter (not a digit)
        if name.is_empty() {
            return false;
        }

        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() {
            return false;
        }

        name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_valid_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("radium-extension.json");

        let manifest_json = r#"{
            "name": "test-extension",
            "version": "1.0.0",
            "description": "Test extension",
            "author": "Test Author",
            "components": {
                "prompts": ["prompts/*.md"],
                "mcp_servers": ["mcp/*.json"],
                "commands": ["commands/*.toml"]
            },
            "dependencies": []
        }"#;

        fs::write(&manifest_path, manifest_json).unwrap();

        let manifest = ExtensionManifest::load(&manifest_path).unwrap();
        assert_eq!(manifest.name, "test-extension");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.description, "Test extension");
        assert_eq!(manifest.author, "Test Author");
        assert_eq!(manifest.components.prompts.len(), 1);
        assert_eq!(manifest.components.mcp_servers.len(), 1);
        assert_eq!(manifest.components.commands.len(), 1);
    }

    #[test]
    fn test_load_nonexistent_manifest() {
        let path = Path::new("/nonexistent/radium-extension.json");
        let result = ExtensionManifest::load(path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExtensionManifestError::NotFound(_)));
    }

    #[test]
    fn test_from_json_valid() {
        let json = r#"{
            "name": "json-extension",
            "version": "2.0.0",
            "description": "JSON extension",
            "author": "JSON Author"
        }"#;

        let manifest = ExtensionManifest::from_json(json).unwrap();
        assert_eq!(manifest.name, "json-extension");
        assert_eq!(manifest.version, "2.0.0");
    }

    #[test]
    fn test_from_json_invalid() {
        let json = r#"{"invalid": "json"}"#;
        let result = ExtensionManifest::from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_missing_name() {
        let manifest = ExtensionManifest {
            name: String::new(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Author".to_string(),
            components: ExtensionComponents::default(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        };

        let result = manifest.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExtensionManifestError::MissingField(ref f) if f == "name"
        ));
    }

    #[test]
    fn test_validate_missing_version() {
        let manifest = ExtensionManifest {
            name: "test".to_string(),
            version: String::new(),
            description: "Test".to_string(),
            author: "Author".to_string(),
            components: ExtensionComponents::default(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        };

        let result = manifest.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExtensionManifestError::MissingField(ref f) if f == "version"
        ));
    }

    #[test]
    fn test_validate_missing_description() {
        let manifest = ExtensionManifest {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            author: "Author".to_string(),
            components: ExtensionComponents::default(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        };

        let result = manifest.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExtensionManifestError::MissingField(ref f) if f == "description"
        ));
    }

    #[test]
    fn test_validate_missing_author() {
        let manifest = ExtensionManifest {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: String::new(),
            components: ExtensionComponents::default(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        };

        let result = manifest.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExtensionManifestError::MissingField(ref f) if f == "author"
        ));
    }

    #[test]
    fn test_validate_invalid_version() {
        let manifest = ExtensionManifest {
            name: "test".to_string(),
            version: "invalid-version".to_string(),
            description: "Test".to_string(),
            author: "Author".to_string(),
            components: ExtensionComponents::default(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        };

        let result = manifest.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExtensionManifestError::InvalidVersion(_)));
    }

    #[test]
    fn test_validate_valid_versions() {
        let valid_versions = vec!["1", "1.0", "1.0.0", "2.1.3"];

        for version in valid_versions {
            let manifest = ExtensionManifest {
                name: "test".to_string(),
                version: version.to_string(),
                description: "Test".to_string(),
                author: "Author".to_string(),
                components: ExtensionComponents::default(),
                dependencies: Vec::new(),
                metadata: HashMap::new(),
            };

            assert!(manifest.validate().is_ok(), "Version {} should be valid", version);
        }
    }

    #[test]
    fn test_validate_invalid_name() {
        let invalid_names = vec!["", "123start", "-start", "_start", "name with spaces"];

        for name in invalid_names {
            let manifest = ExtensionManifest {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                description: "Test".to_string(),
                author: "Author".to_string(),
                components: ExtensionComponents::default(),
                dependencies: Vec::new(),
                metadata: HashMap::new(),
            };

            let result = manifest.validate();
            assert!(result.is_err(), "Name '{}' should be invalid", name);
        }
    }

    #[test]
    fn test_validate_valid_names() {
        let valid_names = vec!["test", "test-extension", "test_extension", "test123"];

        for name in valid_names {
            let manifest = ExtensionManifest {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                description: "Test".to_string(),
                author: "Author".to_string(),
                components: ExtensionComponents::default(),
                dependencies: Vec::new(),
                metadata: HashMap::new(),
            };

            assert!(manifest.validate().is_ok(), "Name '{}' should be valid", name);
        }
    }

    #[test]
    fn test_validate_empty_component_paths() {
        let mut components = ExtensionComponents::default();
        components.prompts.push("".to_string());

        let manifest = ExtensionManifest {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Author".to_string(),
            components,
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        };

        let result = manifest.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExtensionManifestError::InvalidComponentPath(_)));
    }

    #[test]
    fn test_validate_with_dependencies() {
        let manifest = ExtensionManifest {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Author".to_string(),
            components: ExtensionComponents::default(),
            dependencies: vec!["dep1".to_string(), "dep2".to_string()],
            metadata: HashMap::new(),
        };

        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.dependencies.len(), 2);
    }

    #[test]
    fn test_validate_default_components() {
        let manifest = ExtensionManifest {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Author".to_string(),
            components: ExtensionComponents::default(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        };

        assert!(manifest.validate().is_ok());
        assert!(manifest.components.prompts.is_empty());
        assert!(manifest.components.mcp_servers.is_empty());
        assert!(manifest.components.commands.is_empty());
    }

    #[test]
    fn test_manifest_serialization() {
        let manifest = ExtensionManifest {
            name: "test-extension".to_string(),
            version: "1.0.0".to_string(),
            description: "Test extension".to_string(),
            author: "Test Author".to_string(),
            components: ExtensionComponents {
                prompts: vec!["prompts/*.md".to_string()],
                mcp_servers: vec!["mcp/*.json".to_string()],
                commands: vec!["commands/*.toml".to_string()],
            },
            dependencies: vec!["dep1".to_string()],
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: ExtensionManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(manifest.name, deserialized.name);
        assert_eq!(manifest.version, deserialized.version);
        assert_eq!(manifest.components.prompts.len(), deserialized.components.prompts.len());
    }
}

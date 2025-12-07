//! Extension discovery and enumeration.
//!
//! Provides functionality for discovering and listing installed extensions.

use crate::extensions::manifest::{ExtensionManifest, ExtensionManifestError};
use crate::extensions::structure::{Extension, ExtensionStructureError, MANIFEST_FILE};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Extension discovery errors.
#[derive(Debug, Error)]
pub enum ExtensionDiscoveryError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Manifest error.
    #[error("manifest error: {0}")]
    Manifest(#[from] ExtensionManifestError),

    /// Structure error.
    #[error("structure error: {0}")]
    Structure(#[from] ExtensionStructureError),

    /// Extension not found.
    #[error("extension not found: {0}")]
    NotFound(String),
}

/// Result type for discovery operations.
pub type Result<T> = std::result::Result<T, ExtensionDiscoveryError>;

/// Extension discovery options.
#[derive(Debug, Clone, Default)]
pub struct DiscoveryOptions {
    /// Extension directories to search.
    ///
    /// If empty, uses default user directory.
    pub search_paths: Vec<PathBuf>,

    /// Whether to validate extension structure during discovery.
    pub validate_structure: bool,
}

/// Extension discovery service.
pub struct ExtensionDiscovery {
    options: DiscoveryOptions,
}

impl ExtensionDiscovery {
    /// Creates a new extension discovery with default options.
    pub fn new() -> Self {
        Self {
            options: DiscoveryOptions::default(),
        }
    }

    /// Creates a new extension discovery with custom options.
    pub fn with_options(options: DiscoveryOptions) -> Self {
        Self { options }
    }

    /// Gets the search paths to use.
    fn get_search_paths(&self) -> std::result::Result<Vec<PathBuf>, ExtensionDiscoveryError> {
        if self.options.search_paths.is_empty() {
            // Use default user directory
            let default_dir = crate::extensions::structure::default_extensions_dir()
                .map_err(|e| ExtensionDiscoveryError::Structure(e))?;
            Ok(vec![default_dir])
        } else {
            Ok(self.options.search_paths.clone())
        }
    }

    /// Discovers all installed extensions.
    ///
    /// # Returns
    /// Vector of discovered extensions
    ///
    /// # Errors
    /// Returns error if discovery fails
    pub fn discover_all(&self) -> Result<Vec<Extension>> {
        let search_paths = self.get_search_paths()?;
        let mut extensions = Vec::new();

        for search_path in search_paths {
            if !search_path.exists() {
                continue;
            }

            let entries = std::fs::read_dir(&search_path)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if !path.is_dir() {
                    continue;
                }

                // Check for manifest file
                let manifest_path = path.join(MANIFEST_FILE);
                if !manifest_path.exists() {
                    continue;
                }

                // Try to load extension
                if let Ok(extension) = self.load_extension_from_dir(&path) {
                    extensions.push(extension);
                }
            }
        }

        Ok(extensions)
    }

    /// Discovers extensions in a specific directory.
    ///
    /// # Arguments
    /// * `dir` - Directory to search
    ///
    /// # Returns
    /// Vector of discovered extensions
    pub fn discover_in_directory(&self, dir: &Path) -> Result<Vec<Extension>> {
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut extensions = Vec::new();
        let entries = std::fs::read_dir(dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join(MANIFEST_FILE);
            if !manifest_path.exists() {
                continue;
            }

            if let Ok(extension) = self.load_extension_from_dir(&path) {
                extensions.push(extension);
            }
        }

        Ok(extensions)
    }

    /// Loads an extension from a directory.
    ///
    /// # Arguments
    /// * `dir` - Extension directory
    ///
    /// # Returns
    /// Loaded extension
    fn load_extension_from_dir(&self, dir: &Path) -> Result<Extension> {
        let manifest_path = dir.join(MANIFEST_FILE);
        let manifest = ExtensionManifest::load(&manifest_path)?;

        let extension = Extension::new(manifest, dir.to_path_buf());

        // Validate structure if requested
        if self.options.validate_structure {
            extension.validate_structure().map_err(|e| {
                ExtensionDiscoveryError::Structure(e)
            })?;
        }

        Ok(extension)
    }

    /// Gets an extension by name.
    ///
    /// # Arguments
    /// * `name` - Extension name
    ///
    /// # Returns
    /// Extension if found, None otherwise
    pub fn get(&self, name: &str) -> Result<Option<Extension>> {
        let extensions = self.discover_all()?;

        for extension in extensions {
            if extension.name == name {
                return Ok(Some(extension));
            }
        }

        Ok(None)
    }

    /// Lists all installed extensions.
    ///
    /// # Returns
    /// Vector of extension names and versions
    pub fn list(&self) -> Result<Vec<(String, String)>> {
        let extensions = self.discover_all()?;
        Ok(extensions
            .into_iter()
            .map(|ext| (ext.name, ext.version))
            .collect())
    }

    /// Searches for extensions by name or description.
    ///
    /// # Arguments
    /// * `query` - Search query
    ///
    /// # Returns
    /// Matching extensions
    pub fn search(&self, query: &str) -> Result<Vec<Extension>> {
        let extensions = self.discover_all()?;
        let query_lower = query.to_lowercase();

        Ok(extensions
            .into_iter()
            .filter(|ext| {
                ext.name.to_lowercase().contains(&query_lower)
                    || ext.manifest.description.to_lowercase().contains(&query_lower)
            })
            .collect())
    }

    /// Validates all discovered extensions.
    ///
    /// # Returns
    /// Vector of (extension name, validation result)
    pub fn validate_all(&self) -> Result<Vec<(String, std::result::Result<(), ExtensionStructureError>)>> {
        let extensions = self.discover_all()?;
        Ok(extensions
            .into_iter()
            .map(|ext| {
                let name = ext.name.clone();
                let result = ext.validate_structure();
                (name, result)
            })
            .collect())
    }
}

impl Default for ExtensionDiscovery {
    fn default() -> Self {
        Self::new()
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
            description: format!("Test extension: {}", name),
            author: "Test Author".to_string(),
            components: ExtensionComponents::default(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    fn create_test_extension(temp_dir: &TempDir, name: &str) -> PathBuf {
        let ext_dir = temp_dir.path().join(name);
        std::fs::create_dir_all(&ext_dir).unwrap();

        let manifest_path = ext_dir.join(MANIFEST_FILE);
        let manifest = create_test_manifest(name);
        let manifest_json = serde_json::to_string(&manifest).unwrap();
        std::fs::write(&manifest_path, manifest_json).unwrap();

        ext_dir
    }

    #[test]
    fn test_discover_all_empty() {
        let temp_dir = TempDir::new().unwrap();
        let discovery = ExtensionDiscovery::with_options(DiscoveryOptions {
            search_paths: vec![temp_dir.path().to_path_buf()],
            validate_structure: false,
        });

        let extensions = discovery.discover_all().unwrap();
        assert!(extensions.is_empty());
    }

    #[test]
    fn test_discover_all_single() {
        let temp_dir = TempDir::new().unwrap();
        create_test_extension(&temp_dir, "test-ext");

        let discovery = ExtensionDiscovery::with_options(DiscoveryOptions {
            search_paths: vec![temp_dir.path().to_path_buf()],
            validate_structure: false,
        });

        let extensions = discovery.discover_all().unwrap();
        assert_eq!(extensions.len(), 1);
        assert_eq!(extensions[0].name, "test-ext");
    }

    #[test]
    fn test_discover_all_multiple() {
        let temp_dir = TempDir::new().unwrap();
        create_test_extension(&temp_dir, "ext1");
        create_test_extension(&temp_dir, "ext2");
        create_test_extension(&temp_dir, "ext3");

        let discovery = ExtensionDiscovery::with_options(DiscoveryOptions {
            search_paths: vec![temp_dir.path().to_path_buf()],
            validate_structure: false,
        });

        let extensions = discovery.discover_all().unwrap();
        assert_eq!(extensions.len(), 3);
    }

    #[test]
    fn test_discover_in_directory() {
        let temp_dir = TempDir::new().unwrap();
        create_test_extension(&temp_dir, "test-ext");

        let discovery = ExtensionDiscovery::new();
        let extensions = discovery.discover_in_directory(temp_dir.path()).unwrap();

        assert_eq!(extensions.len(), 1);
        assert_eq!(extensions[0].name, "test-ext");
    }

    #[test]
    fn test_get_extension() {
        let temp_dir = TempDir::new().unwrap();
        create_test_extension(&temp_dir, "test-ext");

        let discovery = ExtensionDiscovery::with_options(DiscoveryOptions {
            search_paths: vec![temp_dir.path().to_path_buf()],
            validate_structure: false,
        });

        let extension = discovery.get("test-ext").unwrap();
        assert!(extension.is_some());
        assert_eq!(extension.unwrap().name, "test-ext");
    }

    #[test]
    fn test_get_nonexistent_extension() {
        let temp_dir = TempDir::new().unwrap();

        let discovery = ExtensionDiscovery::with_options(DiscoveryOptions {
            search_paths: vec![temp_dir.path().to_path_buf()],
            validate_structure: false,
        });

        let extension = discovery.get("nonexistent").unwrap();
        assert!(extension.is_none());
    }

    #[test]
    fn test_list_extensions() {
        let temp_dir = TempDir::new().unwrap();
        create_test_extension(&temp_dir, "ext1");
        create_test_extension(&temp_dir, "ext2");

        let discovery = ExtensionDiscovery::with_options(DiscoveryOptions {
            search_paths: vec![temp_dir.path().to_path_buf()],
            validate_structure: false,
        });

        let list = discovery.list().unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|(name, _)| name == "ext1"));
        assert!(list.iter().any(|(name, _)| name == "ext2"));
    }

    #[test]
    fn test_search_by_name() {
        let temp_dir = TempDir::new().unwrap();
        create_test_extension(&temp_dir, "test-extension");
        create_test_extension(&temp_dir, "other-extension");

        let discovery = ExtensionDiscovery::with_options(DiscoveryOptions {
            search_paths: vec![temp_dir.path().to_path_buf()],
            validate_structure: false,
        });

        let results = discovery.search("test").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test-extension");
    }

    #[test]
    fn test_search_by_description() {
        let temp_dir = TempDir::new().unwrap();
        create_test_extension(&temp_dir, "ext1");
        create_test_extension(&temp_dir, "ext2");

        let discovery = ExtensionDiscovery::with_options(DiscoveryOptions {
            search_paths: vec![temp_dir.path().to_path_buf()],
            validate_structure: false,
        });

        let results = discovery.search("ext1").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "ext1");
    }

    #[test]
    fn test_search_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();
        create_test_extension(&temp_dir, "TestExtension");

        let discovery = ExtensionDiscovery::with_options(DiscoveryOptions {
            search_paths: vec![temp_dir.path().to_path_buf()],
            validate_structure: false,
        });

        let results = discovery.search("test").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_discover_skips_non_directories() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a file (not a directory)
        std::fs::write(temp_dir.path().join("not-a-dir"), "content").unwrap();
        
        // Create a valid extension
        create_test_extension(&temp_dir, "valid-ext");

        let discovery = ExtensionDiscovery::with_options(DiscoveryOptions {
            search_paths: vec![temp_dir.path().to_path_buf()],
            validate_structure: false,
        });

        let extensions = discovery.discover_all().unwrap();
        assert_eq!(extensions.len(), 1);
        assert_eq!(extensions[0].name, "valid-ext");
    }

    #[test]
    fn test_discover_skips_directories_without_manifest() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create directory without manifest
        std::fs::create_dir(temp_dir.path().join("no-manifest")).unwrap();
        
        // Create valid extension
        create_test_extension(&temp_dir, "valid-ext");

        let discovery = ExtensionDiscovery::with_options(DiscoveryOptions {
            search_paths: vec![temp_dir.path().to_path_buf()],
            validate_structure: false,
        });

        let extensions = discovery.discover_all().unwrap();
        assert_eq!(extensions.len(), 1);
        assert_eq!(extensions[0].name, "valid-ext");
    }

    #[test]
    fn test_validate_all() {
        let temp_dir = TempDir::new().unwrap();
        create_test_extension(&temp_dir, "valid-ext");

        let discovery = ExtensionDiscovery::with_options(DiscoveryOptions {
            search_paths: vec![temp_dir.path().to_path_buf()],
            validate_structure: false,
        });

        let validations = discovery.validate_all().unwrap();
        assert_eq!(validations.len(), 1);
        assert_eq!(validations[0].0, "valid-ext");
        // Validation should pass for extensions with no components
        assert!(validations[0].1.is_ok());
    }
}


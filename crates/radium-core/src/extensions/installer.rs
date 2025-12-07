//! Extension installation and management.
//!
//! Provides functionality for installing, uninstalling, and updating
//! extension packages.

use crate::extensions::discovery::{DiscoveryOptions, ExtensionDiscovery, ExtensionDiscoveryError};
use crate::extensions::manifest::ExtensionManifest;
use crate::extensions::structure::{
    Extension, ExtensionStructureError, MANIFEST_FILE, default_extensions_dir,
    validate_package_structure,
};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Extension installer errors.
#[derive(Debug, Error)]
pub enum ExtensionInstallerError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Manifest error.
    #[error("manifest error: {0}")]
    Manifest(#[from] crate::extensions::manifest::ExtensionManifestError),

    /// Structure error.
    #[error("structure error: {0}")]
    Structure(#[from] ExtensionStructureError),

    /// Discovery error.
    #[error("discovery error: {0}")]
    Discovery(#[from] ExtensionDiscoveryError),

    /// Extension already installed.
    #[error("extension already installed: {0}")]
    AlreadyInstalled(String),

    /// Extension not found.
    #[error("extension not found: {0}")]
    NotFound(String),

    /// Dependency error.
    #[error("dependency error: {0}")]
    Dependency(String),

    /// Installation conflict.
    #[error("installation conflict: {0}")]
    Conflict(String),
}

/// Result type for installer operations.
pub type Result<T> = std::result::Result<T, ExtensionInstallerError>;

/// Extension installation options.
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    /// Whether to overwrite existing installation.
    pub overwrite: bool,

    /// Whether to install dependencies automatically.
    pub install_dependencies: bool,

    /// Whether to validate structure after installation.
    pub validate_after_install: bool,
}

/// Extension manager for installing and managing extensions.
pub struct ExtensionManager {
    extensions_dir: PathBuf,
    discovery: ExtensionDiscovery,
}

impl ExtensionManager {
    /// Creates a new extension manager with default extensions directory.
    ///
    /// # Errors
    /// Returns error if HOME environment variable is not set
    pub fn new() -> Result<Self> {
        let extensions_dir =
            default_extensions_dir().map_err(|e| ExtensionInstallerError::Structure(e))?;

        let discovery = ExtensionDiscovery::with_options(DiscoveryOptions {
            search_paths: vec![extensions_dir.clone()],
            validate_structure: false,
        });

        Ok(Self { extensions_dir, discovery })
    }

    /// Creates a new extension manager with custom extensions directory.
    ///
    /// # Arguments
    /// * `extensions_dir` - Custom extensions directory path
    pub fn with_directory(extensions_dir: PathBuf) -> Self {
        let discovery = ExtensionDiscovery::with_options(DiscoveryOptions {
            search_paths: vec![extensions_dir.clone()],
            validate_structure: false,
        });

        Self { extensions_dir, discovery }
    }

    /// Gets the extensions directory.
    pub fn extensions_dir(&self) -> &Path {
        &self.extensions_dir
    }

    /// Ensures the extensions directory exists.
    fn ensure_extensions_dir(&self) -> Result<()> {
        if !self.extensions_dir.exists() {
            std::fs::create_dir_all(&self.extensions_dir)?;
        }
        Ok(())
    }

    /// Installs an extension from a local directory.
    ///
    /// # Arguments
    /// * `package_path` - Path to extension package directory
    /// * `options` - Installation options
    ///
    /// # Returns
    /// Installed extension
    ///
    /// # Errors
    /// Returns error if installation fails
    pub fn install(&self, package_path: &Path, options: InstallOptions) -> Result<Extension> {
        // Validate package structure
        validate_package_structure(package_path)
            .map_err(|e| ExtensionInstallerError::Structure(e))?;

        // Load manifest
        let manifest_path = package_path.join(MANIFEST_FILE);
        let manifest = ExtensionManifest::load(&manifest_path)?;

        // Check if already installed
        if self.discovery.get(&manifest.name)?.is_some() {
            if !options.overwrite {
                return Err(ExtensionInstallerError::AlreadyInstalled(manifest.name.clone()));
            }
            // Uninstall existing first
            self.uninstall(&manifest.name)?;
        }

        // Check dependencies
        if options.install_dependencies {
            self.check_dependencies(&manifest)?;
        } else {
            self.validate_dependencies(&manifest)?;
        }

        // Ensure extensions directory exists
        self.ensure_extensions_dir()?;

        // Create installation directory
        let install_path = self.extensions_dir.join(&manifest.name);

        // Copy extension files
        self.copy_extension_files(package_path, &install_path)?;

        // Load installed extension
        let extension = Extension::new(manifest, install_path);

        // Validate if requested
        if options.validate_after_install {
            extension.validate_structure().map_err(|e| ExtensionInstallerError::Structure(e))?;
        }

        Ok(extension)
    }

    /// Installs an extension from a URL (skeleton implementation).
    ///
    /// # Arguments
    /// * `url` - Extension URL
    /// * `options` - Installation options
    ///
    /// # Returns
    /// Installed extension
    ///
    /// # Errors
    /// Returns error if installation fails
    ///
    /// # Note
    /// This is a basic skeleton. Full URL installation would require:
    /// - HTTP client to download
    /// - Archive extraction (zip, tar.gz)
    /// - Temporary directory management
    pub fn install_from_url(&self, _url: &str, _options: InstallOptions) -> Result<Extension> {
        // TODO: Implement URL-based installation
        // This would involve:
        // 1. Downloading from URL
        // 2. Extracting archive
        // 3. Installing from extracted directory
        Err(ExtensionInstallerError::Conflict("URL installation not yet implemented".to_string()))
    }

    /// Uninstalls an extension by name.
    ///
    /// # Arguments
    /// * `name` - Extension name
    ///
    /// # Errors
    /// Returns error if uninstallation fails
    pub fn uninstall(&self, name: &str) -> Result<()> {
        let extension = self
            .discovery
            .get(name)?
            .ok_or_else(|| ExtensionInstallerError::NotFound(name.to_string()))?;

        // Check for dependent extensions
        let all_extensions = self.discovery.discover_all()?;
        for ext in all_extensions {
            if ext.name != name && ext.manifest.dependencies.contains(&name.to_string()) {
                return Err(ExtensionInstallerError::Dependency(format!(
                    "Cannot uninstall '{}': extension '{}' depends on it",
                    name, ext.name
                )));
            }
        }

        // Remove extension directory
        if extension.install_path.exists() {
            std::fs::remove_dir_all(&extension.install_path)?;
        }

        Ok(())
    }

    /// Updates an extension by reinstalling it.
    ///
    /// # Arguments
    /// * `name` - Extension name
    /// * `package_path` - Path to updated extension package
    /// * `options` - Installation options
    ///
    /// # Returns
    /// Updated extension
    ///
    /// # Errors
    /// Returns error if update fails
    pub fn update(
        &self,
        name: &str,
        package_path: &Path,
        options: InstallOptions,
    ) -> Result<Extension> {
        // Uninstall existing
        if self.discovery.get(name)?.is_some() {
            self.uninstall(name)?;
        }

        // Install new version
        let mut install_options = options;
        install_options.overwrite = true;
        self.install(package_path, install_options)
    }

    /// Lists all installed extensions.
    ///
    /// # Returns
    /// Vector of installed extensions
    pub fn list(&self) -> Result<Vec<Extension>> {
        self.discovery.discover_all().map_err(ExtensionInstallerError::Discovery)
    }

    /// Gets an extension by name.
    ///
    /// # Arguments
    /// * `name` - Extension name
    ///
    /// # Returns
    /// Extension if found, None otherwise
    pub fn get(&self, name: &str) -> Result<Option<Extension>> {
        self.discovery.get(name).map_err(ExtensionInstallerError::Discovery)
    }

    /// Validates that all dependencies are installed.
    ///
    /// # Arguments
    /// * `manifest` - Extension manifest
    ///
    /// # Errors
    /// Returns error if dependencies are missing
    fn validate_dependencies(&self, manifest: &ExtensionManifest) -> Result<()> {
        for dep_name in &manifest.dependencies {
            if self.discovery.get(dep_name)?.is_none() {
                return Err(ExtensionInstallerError::Dependency(format!(
                    "Missing dependency: '{}'",
                    dep_name
                )));
            }
        }
        Ok(())
    }

    /// Checks dependencies and installs missing ones (future enhancement).
    ///
    /// # Arguments
    /// * `manifest` - Extension manifest
    ///
    /// # Errors
    /// Returns error if dependency installation fails
    ///
    /// # Note
    /// Currently just validates. Full implementation would recursively install.
    fn check_dependencies(&self, manifest: &ExtensionManifest) -> Result<()> {
        // For now, just validate dependencies exist
        // Future: recursively install missing dependencies
        self.validate_dependencies(manifest)
    }

    /// Copies extension files from package to installation directory.
    ///
    /// # Arguments
    /// * `source` - Source package directory
    /// * `dest` - Destination installation directory
    ///
    /// # Errors
    /// Returns error if copy fails
    fn copy_extension_files(&self, source: &Path, dest: &Path) -> Result<()> {
        // Create destination directory
        if dest.exists() {
            std::fs::remove_dir_all(dest)?;
        }
        std::fs::create_dir_all(dest)?;

        // Copy all files recursively
        copy_dir_all(source, dest)?;

        Ok(())
    }
}

/// Recursively copies a directory.
fn copy_dir_all(source: &Path, dest: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dest)?;

    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if path.is_dir() {
            copy_dir_all(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
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
            description: format!("Test extension: {}", name),
            author: "Test Author".to_string(),
            components: ExtensionComponents::default(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    fn create_test_package(temp_dir: &TempDir, name: &str) -> PathBuf {
        let package_dir = temp_dir.path().join(format!("package-{}", name));
        std::fs::create_dir_all(&package_dir).unwrap();

        // Create manifest
        let manifest_path = package_dir.join(MANIFEST_FILE);
        let manifest = create_test_manifest(name);
        let manifest_json = serde_json::to_string(&manifest).unwrap();
        std::fs::write(&manifest_path, manifest_json).unwrap();

        package_dir
    }

    #[test]
    fn test_install_extension() {
        let temp_dir = TempDir::new().unwrap();
        let extensions_dir = temp_dir.path().join("extensions");
        std::fs::create_dir_all(&extensions_dir).unwrap();

        let package_path = create_test_package(&temp_dir, "test-ext");

        let manager = ExtensionManager::with_directory(extensions_dir);
        let options = InstallOptions::default();

        let extension = manager.install(&package_path, options).unwrap();
        assert_eq!(extension.name, "test-ext");
        assert_eq!(extension.version, "1.0.0");
    }

    #[test]
    fn test_install_already_installed() {
        let temp_dir = TempDir::new().unwrap();
        let extensions_dir = temp_dir.path().join("extensions");
        std::fs::create_dir_all(&extensions_dir).unwrap();

        let package_path = create_test_package(&temp_dir, "test-ext");

        let manager = ExtensionManager::with_directory(extensions_dir);
        let options = InstallOptions::default();

        // Install first time
        manager.install(&package_path, options.clone()).unwrap();

        // Try to install again (should fail)
        let result = manager.install(&package_path, options);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExtensionInstallerError::AlreadyInstalled(_)));
    }

    #[test]
    fn test_install_with_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let extensions_dir = temp_dir.path().join("extensions");
        std::fs::create_dir_all(&extensions_dir).unwrap();

        let package_path = create_test_package(&temp_dir, "test-ext");

        let manager = ExtensionManager::with_directory(extensions_dir);
        let mut options = InstallOptions::default();

        // Install first time
        manager.install(&package_path, options.clone()).unwrap();

        // Install again with overwrite
        options.overwrite = true;
        let extension = manager.install(&package_path, options).unwrap();
        assert_eq!(extension.name, "test-ext");
    }

    #[test]
    fn test_uninstall_extension() {
        let temp_dir = TempDir::new().unwrap();
        let extensions_dir = temp_dir.path().join("extensions");
        std::fs::create_dir_all(&extensions_dir).unwrap();

        let package_path = create_test_package(&temp_dir, "test-ext");

        let manager = ExtensionManager::with_directory(extensions_dir.clone());
        let options = InstallOptions::default();

        // Install
        manager.install(&package_path, options).unwrap();

        // Uninstall
        manager.uninstall("test-ext").unwrap();

        // Verify uninstalled
        let extension = manager.get("test-ext").unwrap();
        assert!(extension.is_none());
    }

    #[test]
    fn test_uninstall_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let extensions_dir = temp_dir.path().join("extensions");
        std::fs::create_dir_all(&extensions_dir).unwrap();

        let manager = ExtensionManager::with_directory(extensions_dir);
        let result = manager.uninstall("nonexistent");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExtensionInstallerError::NotFound(_)));
    }

    #[test]
    fn test_list_extensions() {
        let temp_dir = TempDir::new().unwrap();
        let extensions_dir = temp_dir.path().join("extensions");
        std::fs::create_dir_all(&extensions_dir).unwrap();

        let package1 = create_test_package(&temp_dir, "ext1");
        let package2 = create_test_package(&temp_dir, "ext2");

        let manager = ExtensionManager::with_directory(extensions_dir);
        let options = InstallOptions::default();

        manager.install(&package1, options.clone()).unwrap();
        manager.install(&package2, options).unwrap();

        let extensions = manager.list().unwrap();
        assert_eq!(extensions.len(), 2);
    }

    #[test]
    fn test_get_extension() {
        let temp_dir = TempDir::new().unwrap();
        let extensions_dir = temp_dir.path().join("extensions");
        std::fs::create_dir_all(&extensions_dir).unwrap();

        let package_path = create_test_package(&temp_dir, "test-ext");

        let manager = ExtensionManager::with_directory(extensions_dir);
        let options = InstallOptions::default();

        manager.install(&package_path, options).unwrap();

        let extension = manager.get("test-ext").unwrap();
        assert!(extension.is_some());
        assert_eq!(extension.unwrap().name, "test-ext");
    }

    #[test]
    fn test_validate_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let extensions_dir = temp_dir.path().join("extensions");
        std::fs::create_dir_all(&extensions_dir).unwrap();

        // Create package with dependency
        let package_dir = temp_dir.path().join("package-dep");
        std::fs::create_dir_all(&package_dir).unwrap();

        let mut manifest = create_test_manifest("dependent-ext");
        manifest.dependencies.push("missing-dep".to_string());

        let manifest_path = package_dir.join(MANIFEST_FILE);
        let manifest_json = serde_json::to_string(&manifest).unwrap();
        std::fs::write(&manifest_path, manifest_json).unwrap();

        let manager = ExtensionManager::with_directory(extensions_dir);
        let options = InstallOptions::default();

        let result = manager.install(&package_dir, options);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExtensionInstallerError::Dependency(_)));
    }

    #[test]
    fn test_update_extension() {
        let temp_dir = TempDir::new().unwrap();
        let extensions_dir = temp_dir.path().join("extensions");
        std::fs::create_dir_all(&extensions_dir).unwrap();

        let package_path = create_test_package(&temp_dir, "test-ext");

        let manager = ExtensionManager::with_directory(extensions_dir);
        let options = InstallOptions::default();

        // Install initial version
        manager.install(&package_path, options.clone()).unwrap();

        // Update (same package, but tests the update path)
        let extension = manager.update("test-ext", &package_path, options).unwrap();
        assert_eq!(extension.name, "test-ext");
    }

    #[test]
    fn test_copy_extension_files() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source");
        let dest = temp_dir.path().join("dest");

        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("file.txt"), "content").unwrap();
        std::fs::create_dir_all(source.join("subdir")).unwrap();
        std::fs::write(source.join("subdir").join("subfile.txt"), "subcontent").unwrap();

        copy_dir_all(&source, &dest).unwrap();

        assert!(dest.join("file.txt").exists());
        assert!(dest.join("subdir").join("subfile.txt").exists());
    }
}

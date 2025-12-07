//! Extension installation and management.
//!
//! Provides functionality for installing, uninstalling, and updating
//! extension packages.

#[cfg(feature = "workflow")]
use crate::extensions::conflict::{ConflictDetector, ConflictError};
use crate::extensions::discovery::{DiscoveryOptions, ExtensionDiscovery, ExtensionDiscoveryError};
use crate::extensions::manifest::ExtensionManifest;
use crate::extensions::structure::{
    Extension, ExtensionStructureError, MANIFEST_FILE, default_extensions_dir,
    validate_package_structure,
};
use crate::extensions::signing::{SignatureVerifier, TrustedKeysManager};
use crate::extensions::validator::{ExtensionValidator, ExtensionValidationError};
use crate::extensions::versioning::{UpdateChecker, VersionComparator};
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

    /// Invalid format error.
    #[error("invalid format: {0}")]
    InvalidFormat(String),

    /// Installation conflict.
    #[error("installation conflict: {0}")]
    Conflict(String),

    /// Validation error.
    #[error("validation error: {0}")]
    Validation(#[from] ExtensionValidationError),

    /// Conflict detection error.
    #[cfg(feature = "workflow")]
    #[error("conflict error: {0}")]
    ConflictDetection(#[from] ConflictError),
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

    /// Installs an extension from a local directory, archive, or URL.
    ///
    /// # Arguments
    /// * `source` - Path to extension package directory, archive file, or URL
    /// * `options` - Installation options
    ///
    /// # Returns
    /// Installed extension
    ///
    /// # Errors
    /// Returns error if installation fails
    pub fn install_from_source(&self, source: &str, options: InstallOptions) -> Result<Extension> {
        // Check if source is a URL
        if source.starts_with("http://") || source.starts_with("https://") {
            return self.install_from_url(source, options);
        }

        let source_path = Path::new(source);
        if !source_path.exists() {
            return Err(ExtensionInstallerError::NotFound(format!(
                "Extension source not found: {}",
                source
            )));
        }

        // Check if source is an archive file
        if source_path.is_file() {
            let extension = source_path.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            
            if extension == "gz" || extension == "tgz" || source.ends_with(".tar.gz") {
                return self.install_from_archive(source_path, options);
            }
        }

        // Otherwise, treat as directory
        self.install(source_path, options)
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

        // Validate extension
        ExtensionValidator::validate(package_path, &manifest)?;

        // Check for conflicts
        #[cfg(feature = "workflow")]
        ConflictDetector::check_conflicts(&manifest, package_path)?;

        // Verify signature if present (warning mode - non-blocking)
        if SignatureVerifier::has_signature(package_path) {
            if let Ok(keys_manager) = TrustedKeysManager::new() {
                // Try to verify with any trusted key
                let mut verified = false;
                if let Ok(trusted_keys) = keys_manager.list_trusted_keys() {
                    for key_name in trusted_keys {
                        if let Ok(public_key) = keys_manager.get_trusted_key(&key_name) {
                            if SignatureVerifier::verify(package_path, &public_key).is_ok() {
                                verified = true;
                                break;
                            }
                        }
                    }
                }
                if !verified {
                    // Log warning but don't block installation
                    eprintln!("Warning: Extension signature could not be verified. Proceeding with installation anyway.");
                }
            }
        }

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

    /// Installs an extension from a URL.
    ///
    /// # Arguments
    /// * `url` - Extension URL (must point to a .tar.gz archive)
    /// * `options` - Installation options
    ///
    /// # Returns
    /// Installed extension
    ///
    /// # Errors
    /// Returns error if installation fails
    pub fn install_from_url(&self, url: &str, options: InstallOptions) -> Result<Extension> {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Download the archive to a temporary file
        let response = reqwest::blocking::get(url)
            .map_err(|e| ExtensionInstallerError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to download extension: {}", e),
            )))?;

        if !response.status().is_success() {
            return Err(ExtensionInstallerError::Conflict(format!(
                "Failed to download extension: HTTP {}",
                response.status()
            )));
        }

        // Create temporary file for download
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| ExtensionInstallerError::Io(e))?;
        
        let bytes = response.bytes()
            .map_err(|e| ExtensionInstallerError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to read response: {}", e),
            )))?;

        temp_file.write_all(&bytes)
            .map_err(|e| ExtensionInstallerError::Io(e))?;
        
        temp_file.flush()
            .map_err(|e| ExtensionInstallerError::Io(e))?;

        // Install from the temporary archive file
        let temp_path = temp_file.path();
        let result = self.install_from_archive(temp_path, options);
        
        // Keep temp file alive until installation completes
        drop(temp_file);
        
        result
    }

    /// Installs an extension from an archive file (.tar.gz).
    ///
    /// # Arguments
    /// * `archive_path` - Path to .tar.gz archive file
    /// * `options` - Installation options
    ///
    /// # Returns
    /// Installed extension
    ///
    /// # Errors
    /// Returns error if installation fails
    pub fn install_from_archive(&self, archive_path: &Path, options: InstallOptions) -> Result<Extension> {
        use flate2::read::GzDecoder;
        use std::fs::File;
        use tar::Archive;
        use tempfile::TempDir;

        // Create temporary directory for extraction
        let temp_dir = TempDir::new()
            .map_err(|e| ExtensionInstallerError::Io(e))?;

        // Open and extract the archive
        let file = File::open(archive_path)
            .map_err(|e| ExtensionInstallerError::Io(e))?;
        
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);
        
        // Extract with security checks (prevent path traversal)
        archive.set_unpack_xattrs(false);
        archive.set_preserve_permissions(false);
        
        archive.unpack(temp_dir.path())
            .map_err(|e| ExtensionInstallerError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to extract archive: {}", e),
            )))?;

        // Find the extension directory in the extracted archive
        // It might be at the root or in a subdirectory
        let extracted_path = if temp_dir.path().join(MANIFEST_FILE).exists() {
            temp_dir.path().to_path_buf()
        } else {
            // Look for a subdirectory containing the manifest
            let entries = std::fs::read_dir(temp_dir.path())
                .map_err(|e| ExtensionInstallerError::Io(e))?;
            
            let mut found_path = None;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join(MANIFEST_FILE).exists() {
                    found_path = Some(path);
                    break;
                }
            }
            
            found_path.ok_or_else(|| ExtensionInstallerError::Structure(
                ExtensionStructureError::InvalidStructure(
                    "Archive does not contain a valid extension structure".to_string()
                )
            ))?
        };

        // Install from extracted directory
        self.install(&extracted_path, options)
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

    /// Updates an extension by reinstalling it with rollback support.
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
    /// Returns error if update fails (extension will be rolled back if backup exists)
    pub fn update(
        &self,
        name: &str,
        package_path: &Path,
        options: InstallOptions,
    ) -> Result<Extension> {
        // Get existing extension if it exists
        let existing_extension = self.discovery.get(name)?;
        
        // Load new manifest to check version
        let new_manifest_path = if package_path.is_dir() {
            package_path.join(MANIFEST_FILE)
        } else {
            // For archives, we'd need to extract first, but for now assume directory
            return Err(ExtensionInstallerError::InvalidFormat(
                "Update from archive not yet supported".to_string(),
            ));
        };
        
        let new_manifest = ExtensionManifest::load(&new_manifest_path)?;
        
        // Validate new version is newer if extension exists
        if let Some(ref existing) = existing_extension {
            if let Err(e) = UpdateChecker::check_for_update(existing, &new_manifest.version) {
                return Err(ExtensionInstallerError::InvalidFormat(format!(
                    "New version {} is not newer than current version {}: {}",
                    new_manifest.version, existing.version, e
                )));
            }
            
            // Create backup before update
            let backup_path = self.create_backup(name, &existing.install_path)?;
            
            // Attempt update
            match self.perform_update(name, package_path, options.clone()) {
                Ok(extension) => {
                    // Update successful, remove backup
                    if backup_path.exists() {
                        let _ = std::fs::remove_dir_all(&backup_path);
                    }
                    Ok(extension)
                }
                Err(e) => {
                    // Update failed, attempt rollback
                    if backup_path.exists() {
                        if let Err(rollback_err) = self.restore_backup(name, &backup_path, &existing.install_path) {
                            return Err(ExtensionInstallerError::InvalidFormat(format!(
                                "Update failed and rollback also failed: {} (original error: {})",
                                rollback_err, e
                            )));
                        }
                    }
                    Err(e)
                }
            }
        } else {
            // Extension doesn't exist, just install it
            let mut install_options = options;
            install_options.overwrite = true;
            self.install(package_path, install_options)
        }
    }
    
    /// Performs the actual update operation.
    fn perform_update(
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
    
    /// Creates a backup of an extension before update.
    fn create_backup(&self, name: &str, install_path: &Path) -> Result<PathBuf> {
        let backup_dir = self.extensions_dir.join(".backups");
        std::fs::create_dir_all(&backup_dir)?;
        
        let backup_path = backup_dir.join(format!("{}-backup", name));
        
        // Remove existing backup if any
        if backup_path.exists() {
            std::fs::remove_dir_all(&backup_path)?;
        }
        
        // Copy extension to backup
        copy_dir_all(install_path, &backup_path)?;
        
        Ok(backup_path)
    }
    
    /// Restores an extension from a backup.
    fn restore_backup(
        &self,
        _name: &str,
        backup_path: &Path,
        install_path: &Path,
    ) -> Result<()> {
        // Remove current (failed) installation if it exists
        if install_path.exists() {
            std::fs::remove_dir_all(install_path)?;
        }
        
        // Restore from backup
        copy_dir_all(backup_path, install_path)?;
        
        Ok(())
    }
    
    /// Rolls back an extension to its previous version.
    ///
    /// # Arguments
    /// * `name` - Extension name
    ///
    /// # Errors
    /// Returns error if rollback fails (e.g., no backup exists)
    pub fn rollback(&self, name: &str) -> Result<()> {
        let extension = self
            .discovery
            .get(name)?
            .ok_or_else(|| ExtensionInstallerError::NotFound(name.to_string()))?;
        
        let backup_path = self.extensions_dir.join(".backups").join(format!("{}-backup", name));
        
        if !backup_path.exists() {
            return Err(ExtensionInstallerError::NotFound(format!(
                "No backup found for extension '{}'",
                name
            )));
        }
        
        self.restore_backup(name, &backup_path, &extension.install_path)?;
        
        Ok(())
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
        // Get list of installed extension names
        let installed_extensions: Vec<String> = self.discovery.discover_all()?
            .into_iter()
            .map(|ext| ext.name)
            .collect();

        // Validate dependencies using validator
        ExtensionValidator::validate_dependencies(manifest, &installed_extensions)
            .map_err(|e| ExtensionInstallerError::Validation(e))?;

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
        // Now returns Validation error instead of Dependency error
        let err = result.unwrap_err();
        assert!(
            matches!(err, ExtensionInstallerError::Validation(_)) || 
            matches!(err, ExtensionInstallerError::Dependency(_)),
            "Expected Validation or Dependency error, got: {:?}", err
        );
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

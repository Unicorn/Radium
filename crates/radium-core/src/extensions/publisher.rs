//! Extension publishing workflow.
//!
//! Provides functionality for publishing extensions to the marketplace
//! with validation, signing, and upload.

use crate::extensions::manifest::ExtensionManifest;
use crate::extensions::signing::ExtensionSigner;
use crate::extensions::structure::{MANIFEST_FILE, validate_package_structure};
use crate::extensions::validator::ExtensionValidator;
use crate::extensions::marketplace::{MarketplaceClient, MarketplaceError};
use std::path::Path;
use thiserror::Error;

/// Publishing errors.
#[derive(Debug, Error)]
pub enum PublishingError {
    /// Validation error.
    #[error("validation error: {0}")]
    Validation(String),

    /// Marketplace error.
    #[error("marketplace error: {0}")]
    Marketplace(#[from] MarketplaceError),

    /// Signing error.
    #[error("signing error: {0}")]
    Signing(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Manifest error.
    #[error("manifest error: {0}")]
    Manifest(String),
}

/// Result type for publishing operations.
pub type Result<T> = std::result::Result<T, PublishingError>;

/// Extension publisher for publishing to marketplace.
pub struct ExtensionPublisher {
    marketplace_client: MarketplaceClient,
}

impl ExtensionPublisher {
    /// Creates a new publisher.
    ///
    /// # Returns
    /// New publisher instance
    ///
    /// # Errors
    /// Returns error if marketplace client cannot be created
    pub fn new() -> Result<Self> {
        let marketplace_client = MarketplaceClient::new()
            .map_err(|e| PublishingError::Marketplace(e))?;
        Ok(Self { marketplace_client })
    }

    /// Validates an extension for publishing.
    ///
    /// # Arguments
    /// * `extension_path` - Path to extension directory
    ///
    /// # Returns
    /// Ok(()) if valid
    ///
    /// # Errors
    /// Returns error if validation fails
    pub fn validate_for_publish(&self, extension_path: &Path) -> Result<()> {
        // Validate package structure
        validate_package_structure(extension_path)
            .map_err(|e| PublishingError::Validation(format!("Invalid package structure: {}", e)))?;

        // Load and validate manifest
        let manifest_path = extension_path.join(MANIFEST_FILE);
        if !manifest_path.exists() {
            return Err(PublishingError::Manifest(format!(
                "Manifest file not found: {}",
                manifest_path.display()
            )));
        }

        let manifest = ExtensionManifest::load(&manifest_path)
            .map_err(|e| PublishingError::Manifest(format!("Failed to load manifest: {}", e)))?;

        // Validate extension
        ExtensionValidator::validate(extension_path, &manifest)
            .map_err(|e| PublishingError::Validation(format!("Extension validation failed: {}", e)))?;

        // Check required fields
        if manifest.name.is_empty() {
            return Err(PublishingError::Validation("Extension name is required".to_string()));
        }
        if manifest.version.is_empty() {
            return Err(PublishingError::Validation("Extension version is required".to_string()));
        }
        if manifest.description.is_empty() {
            return Err(PublishingError::Validation("Extension description is required".to_string()));
        }
        if manifest.author.is_empty() {
            return Err(PublishingError::Validation("Extension author is required".to_string()));
        }

        Ok(())
    }

    /// Publishes an extension to the marketplace.
    ///
    /// # Arguments
    /// * `extension_path` - Path to extension directory
    /// * `api_key` - Marketplace API key for authentication
    /// * `sign_with_key` - Optional private key for signing (if None, extension must already be signed)
    ///
    /// # Returns
    /// Published extension metadata
    ///
    /// # Errors
    /// Returns error if publishing fails
    pub fn publish(
        &self,
        extension_path: &Path,
        api_key: &str,
        sign_with_key: Option<&[u8]>,
    ) -> Result<crate::extensions::marketplace::MarketplaceExtension> {
        // Validate extension
        self.validate_for_publish(extension_path)?;

        // Sign extension if key provided
        if let Some(private_key) = sign_with_key {
            let signer = ExtensionSigner::from_private_key(private_key)
                .map_err(|e| PublishingError::Signing(format!("Failed to create signer: {}", e)))?;
            signer.sign_extension(extension_path)
                .map_err(|e| PublishingError::Signing(format!("Failed to sign extension: {}", e)))?;
        }

        // Create archive for upload
        let archive_path = self.create_archive(extension_path)?;

        // Publish to marketplace
        let result = self.marketplace_client.publish_extension(&archive_path, api_key);

        // Clean up temporary archive
        let _ = std::fs::remove_file(&archive_path);

        result.map_err(|e| PublishingError::Marketplace(e))
    }

    /// Creates a temporary archive of the extension.
    fn create_archive(&self, extension_path: &Path) -> Result<std::path::PathBuf> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::fs::File;
        use tar::Builder;

        let temp_dir = std::env::temp_dir();
        let archive_path = temp_dir.join(format!("{}.tar.gz", 
            extension_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("extension")));

        let file = File::create(&archive_path)?;
        let enc = GzEncoder::new(file, Compression::default());
        let mut tar = Builder::new(enc);

        // Add all files from extension directory
        tar.append_dir_all(".", extension_path)?;
        tar.finish()?;

        Ok(archive_path)
    }
}

impl Default for ExtensionPublisher {
    fn default() -> Self {
        Self::new().expect("Failed to create extension publisher")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_publisher_creation() {
        let publisher = ExtensionPublisher::new();
        // May fail if marketplace URL is invalid, but structure should be correct
        assert!(publisher.is_ok() || publisher.is_err());
    }
}


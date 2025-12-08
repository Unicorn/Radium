//! Extension signing and verification.
//!
//! Provides functionality for cryptographically signing extensions
//! and verifying their authenticity using Ed25519 signatures.

use crate::extensions::structure::MANIFEST_FILE;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Signing errors.
#[derive(Debug, Error)]
pub enum SigningError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid key format.
    #[error("invalid key format: {0}")]
    InvalidKey(String),

    /// Signature verification failed.
    #[error("signature verification failed: {0}")]
    VerificationFailed(String),

    /// Signature file not found.
    #[error("signature file not found: {0}")]
    SignatureNotFound(String),

    /// Manifest error.
    #[error("manifest error: {0}")]
    Manifest(String),
}

/// Result type for signing operations.
pub type Result<T> = std::result::Result<T, SigningError>;

/// Extension signer for generating signatures.
pub struct ExtensionSigner {
    signing_key: SigningKey,
}

impl ExtensionSigner {
    /// Creates a new signer from a private key.
    ///
    /// # Arguments
    /// * `private_key_bytes` - Private key bytes (64 bytes for Ed25519)
    ///
    /// # Returns
    /// New signer instance
    ///
    /// # Errors
    /// Returns error if key format is invalid
    pub fn from_private_key(private_key_bytes: &[u8]) -> Result<Self> {
        let key_array: [u8; 32] = private_key_bytes
            .try_into()
            .map_err(|_| SigningError::InvalidKey("Invalid key length (expected 32 bytes)".to_string()))?;
        let signing_key = SigningKey::from_bytes(&key_array);
        Ok(Self { signing_key })
    }

    /// Generates a new keypair and creates a signer.
    ///
    /// # Returns
    /// Tuple of (signer, public_key_bytes)
    pub fn generate() -> (Self, Vec<u8>) {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let public_key_bytes = signing_key.verifying_key().to_bytes().to_vec();
        (Self { signing_key }, public_key_bytes)
    }

    /// Signs an extension package.
    ///
    /// # Arguments
    /// * `extension_path` - Path to extension directory
    ///
    /// # Returns
    /// Path to signature file
    ///
    /// # Errors
    /// Returns error if signing fails
    pub fn sign_extension(&self, extension_path: &Path) -> Result<PathBuf> {
        // Load manifest
        let manifest_path = extension_path.join(MANIFEST_FILE);
        if !manifest_path.exists() {
            return Err(SigningError::Manifest(format!(
                "Manifest not found: {}",
                manifest_path.display()
            )));
        }

        let manifest_content = fs::read_to_string(&manifest_path)?;

        // Sign the manifest content
        let signature = self.signing_key.sign(manifest_content.as_bytes());

        // Write signature to .sig file
        let sig_path = extension_path.join(format!("{}.sig", MANIFEST_FILE));
        fs::write(&sig_path, signature.to_bytes())?;

        Ok(sig_path)
    }

    /// Gets the public key for this signer.
    ///
    /// # Returns
    /// Public key bytes
    pub fn public_key(&self) -> Vec<u8> {
        self.signing_key.verifying_key().to_bytes().to_vec()
    }
}

/// Signature verifier for validating extension signatures.
pub struct SignatureVerifier;

impl SignatureVerifier {
    /// Verifies an extension signature.
    ///
    /// # Arguments
    /// * `extension_path` - Path to extension directory
    /// * `public_key_bytes` - Public key bytes for verification
    ///
    /// # Returns
    /// Ok(()) if signature is valid
    ///
    /// # Errors
    /// Returns error if verification fails
    pub fn verify(extension_path: &Path, public_key_bytes: &[u8]) -> Result<()> {
        // Load manifest
        let manifest_path = extension_path.join(MANIFEST_FILE);
        if !manifest_path.exists() {
            return Err(SigningError::Manifest(format!(
                "Manifest not found: {}",
                manifest_path.display()
            )));
        }

        let manifest_content = fs::read_to_string(&manifest_path)?;

        // Load signature
        let sig_path = extension_path.join(format!("{}.sig", MANIFEST_FILE));
        if !sig_path.exists() {
            return Err(SigningError::SignatureNotFound(format!(
                "Signature file not found: {}",
                sig_path.display()
            )));
        }

        let signature_bytes = fs::read(&sig_path)?;
        let signature_array: [u8; 64] = signature_bytes
            .try_into()
            .map_err(|v: Vec<u8>| SigningError::InvalidKey(format!("Invalid signature length (expected 64 bytes, got {})", v.len())))?;
        let signature = Signature::from_bytes(&signature_array);

        // Parse public key
        let public_key_array: [u8; 32] = public_key_bytes
            .try_into()
            .map_err(|_| SigningError::InvalidKey("Invalid public key length (expected 32 bytes)".to_string()))?;
        let verifying_key = VerifyingKey::from_bytes(&public_key_array)
            .map_err(|e| SigningError::InvalidKey(format!("Invalid public key: {}", e)))?;

        // Verify signature
        verifying_key
            .verify(manifest_content.as_bytes(), &signature)
            .map_err(|e| SigningError::VerificationFailed(format!("Signature verification failed: {}", e)))?;

        Ok(())
    }

    /// Checks if an extension has a signature file.
    ///
    /// # Arguments
    /// * `extension_path` - Path to extension directory
    ///
    /// # Returns
    /// True if signature file exists
    pub fn has_signature(extension_path: &Path) -> bool {
        let sig_path = extension_path.join(format!("{}.sig", MANIFEST_FILE));
        sig_path.exists()
    }
}

/// Manages trusted public keys for signature verification.
pub struct TrustedKeysManager {
    keys_dir: PathBuf,
}

impl TrustedKeysManager {
    /// Creates a new trusted keys manager with default directory.
    ///
    /// # Returns
    /// New manager instance
    ///
    /// # Errors
    /// Returns error if keys directory cannot be created
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME")
            .map_err(|_| SigningError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "HOME environment variable not set",
            )))?;
        let keys_dir = PathBuf::from(home).join(".radium").join("trusted-keys");
        std::fs::create_dir_all(&keys_dir)?;
        Ok(Self { keys_dir })
    }

    /// Creates a new trusted keys manager with custom directory.
    ///
    /// # Arguments
    /// * `keys_dir` - Directory for trusted keys
    pub fn with_directory(keys_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&keys_dir).ok();
        Self { keys_dir }
    }

    /// Adds a trusted public key.
    ///
    /// # Arguments
    /// * `name` - Key name/identifier
    /// * `public_key_bytes` - Public key bytes
    ///
    /// # Errors
    /// Returns error if key cannot be saved
    pub fn add_trusted_key(&self, name: &str, public_key_bytes: &[u8]) -> Result<()> {
        let key_path = self.keys_dir.join(format!("{}.pub", name));
        fs::write(&key_path, public_key_bytes)?;
        Ok(())
    }

    /// Gets a trusted public key by name.
    ///
    /// # Arguments
    /// * `name` - Key name/identifier
    ///
    /// # Returns
    /// Public key bytes if found
    ///
    /// # Errors
    /// Returns error if key cannot be loaded
    pub fn get_trusted_key(&self, name: &str) -> Result<Vec<u8>> {
        let key_path = self.keys_dir.join(format!("{}.pub", name));
        if !key_path.exists() {
            return Err(SigningError::InvalidKey(format!("Trusted key not found: {}", name)));
        }
        Ok(fs::read(&key_path)?)
    }

    /// Lists all trusted key names.
    ///
    /// # Returns
    /// Vector of key names
    ///
    /// # Errors
    /// Returns error if directory cannot be read
    pub fn list_trusted_keys(&self) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        if !self.keys_dir.exists() {
            return Ok(keys);
        }

        for entry in fs::read_dir(&self.keys_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("pub") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    keys.push(name.to_string());
                }
            }
        }

        Ok(keys)
    }

    /// Removes a trusted key.
    ///
    /// # Arguments
    /// * `name` - Key name/identifier
    ///
    /// # Errors
    /// Returns error if key cannot be removed
    pub fn remove_trusted_key(&self, name: &str) -> Result<()> {
        let key_path = self.keys_dir.join(format!("{}.pub", name));
        if key_path.exists() {
            fs::remove_file(&key_path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_keypair() {
        let (signer, public_key) = ExtensionSigner::generate();
        assert_eq!(public_key.len(), 32);
        assert_eq!(signer.public_key().len(), 32);
    }

    #[test]
    fn test_sign_and_verify() {
        let temp_dir = TempDir::new().unwrap();
        let extension_path = temp_dir.path().join("test-extension");
        std::fs::create_dir_all(&extension_path).unwrap();

        // Create manifest
        let manifest_path = extension_path.join(MANIFEST_FILE);
        let manifest = r#"{"name":"test","version":"1.0.0","description":"Test","author":"Test"}"#;
        std::fs::write(&manifest_path, manifest).unwrap();

        // Generate keypair and sign
        let (signer, public_key) = ExtensionSigner::generate();
        signer.sign_extension(&extension_path).unwrap();

        // Verify signature
        SignatureVerifier::verify(&extension_path, &public_key).unwrap();
    }

    #[test]
    fn test_verify_fails_on_tampered_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let extension_path = temp_dir.path().join("test-extension");
        std::fs::create_dir_all(&extension_path).unwrap();

        // Create manifest
        let manifest_path = extension_path.join(MANIFEST_FILE);
        let manifest = r#"{"name":"test","version":"1.0.0","description":"Test","author":"Test"}"#;
        std::fs::write(&manifest_path, manifest).unwrap();

        // Generate keypair and sign
        let (signer, public_key) = ExtensionSigner::generate();
        signer.sign_extension(&extension_path).unwrap();

        // Tamper with manifest
        std::fs::write(&manifest_path, r#"{"name":"test","version":"2.0.0","description":"Test","author":"Test"}"#).unwrap();

        // Verification should fail
        assert!(SignatureVerifier::verify(&extension_path, &public_key).is_err());
    }

    #[test]
    fn test_trusted_keys_manager() {
        let temp_dir = TempDir::new().unwrap();
        let keys_dir = temp_dir.path().join("trusted-keys");
        let manager = TrustedKeysManager::with_directory(keys_dir.clone());

        // Add trusted key
        let (_, public_key) = ExtensionSigner::generate();
        manager.add_trusted_key("test-key", &public_key).unwrap();

        // Get trusted key
        let retrieved_key = manager.get_trusted_key("test-key").unwrap();
        assert_eq!(retrieved_key, public_key);

        // List keys
        let keys = manager.list_trusted_keys().unwrap();
        assert!(keys.contains(&"test-key".to_string()));

        // Remove key
        manager.remove_trusted_key("test-key").unwrap();
        assert!(manager.get_trusted_key("test-key").is_err());
    }
}


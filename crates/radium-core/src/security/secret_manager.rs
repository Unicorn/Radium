//! Secret manager for encrypted credential storage.
//!
//! Provides AES-256-GCM encryption with PBKDF2 key derivation
//! for secure storage of sensitive credentials.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chrono::{DateTime, Utc};
use pbkdf2::pbkdf2_hmac;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::error::{SecurityError, SecurityResult};

/// Current version of the vault file format.
const VAULT_VERSION: &str = "1.0";

/// Minimum master password length.
const MIN_PASSWORD_LENGTH: usize = 12;

/// PBKDF2 iterations for key derivation.
const PBKDF2_ITERATIONS: u32 = 100_000;

/// Salt length in bytes.
const SALT_LENGTH: usize = 32;

/// Nonce length for AES-GCM.
const NONCE_LENGTH: usize = 12;

/// Encrypted secret entry in the vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SecretEntry {
    /// Encrypted secret value (base64 encoded).
    encrypted_value: String,
    /// Nonce used for encryption (base64 encoded).
    nonce: String,
    /// Version number for rotation tracking.
    version: u32,
    /// Creation timestamp.
    created_at: String,
    /// Last update timestamp.
    updated_at: String,
}

/// Vault file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VaultFile {
    /// Version of the vault file format.
    version: String,
    /// PBKDF2 salt (base64 encoded).
    salt: String,
    /// Map of secret name to encrypted entry.
    secrets: HashMap<String, SecretEntry>,
}

/// Secret manager for encrypted credential storage.
///
/// Provides secure storage and retrieval of secrets using AES-256-GCM
/// encryption with PBKDF2 key derivation from a master password.
///
/// # Security
///
/// - Uses AES-256-GCM for authenticated encryption
/// - PBKDF2 with 100,000 iterations for key derivation
/// - Master password must be at least 12 characters
/// - Vault file has 0600 permissions (owner read/write only)
/// - Sensitive data cleared from memory using zeroize
///
/// # Example
///
/// ```no_run
/// use radium_core::security::SecretManager;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut manager = SecretManager::new(
///     std::path::PathBuf::from("~/.radium/auth/secrets.vault"),
///     "my-secure-master-password"
/// )?;
///
/// // Store a secret
/// manager.store_secret("api_key", "sk-test123")?;
///
/// // Retrieve a secret
/// let value = manager.get_secret("api_key")?;
///
/// // List all secret names
/// let names = manager.list_secrets()?;
/// # Ok(())
/// # }
/// ```
pub struct SecretManager {
    /// Path to the vault file.
    vault_path: PathBuf,
    /// Derived encryption key (zeroized on drop).
    #[allow(dead_code)]
    encryption_key: EncryptionKey,
}

/// Encryption key wrapper that zeroizes on drop.
#[derive(ZeroizeOnDrop)]
struct EncryptionKey {
    key: Key<Aes256Gcm>,
}

impl SecretManager {
    /// Creates a new secret manager with the specified vault path and master password.
    ///
    /// # Arguments
    ///
    /// * `vault_path` - Path to the encrypted vault file
    /// * `master_password` - Master password for encryption key derivation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Master password is too short (< 12 characters)
    /// - Key derivation fails
    /// - Vault file cannot be created or accessed
    pub fn new(vault_path: PathBuf, master_password: &str) -> SecurityResult<Self> {
        Self::validate_password(master_password)?;

        let encryption_key = Self::derive_key(master_password, None)?;

        let manager = Self {
            vault_path,
            encryption_key,
        };

        // Ensure vault directory exists with proper permissions
        manager.ensure_vault_dir()?;

        Ok(manager)
    }

    /// Creates a secret manager from an existing vault.
    ///
    /// Derives the encryption key using the salt stored in the vault file.
    ///
    /// # Arguments
    ///
    /// * `vault_path` - Path to the existing vault file
    /// * `master_password` - Master password for encryption key derivation
    ///
    /// # Errors
    ///
    /// Returns an error if the vault file doesn't exist or is corrupted.
    pub fn from_existing(vault_path: PathBuf, master_password: &str) -> SecurityResult<Self> {
        Self::validate_password(master_password)?;

        // Load vault to get salt
        let vault = Self::load_vault(&vault_path)?;
        let salt = base64::engine::general_purpose::STANDARD
            .decode(&vault.salt)
            .map_err(|e| SecurityError::VaultCorruption(format!("Invalid salt: {}", e)))?;

        let encryption_key = Self::derive_key(master_password, Some(&salt))?;

        Ok(Self {
            vault_path,
            encryption_key,
        })
    }

    /// Validates that the master password meets security requirements.
    fn validate_password(password: &str) -> SecurityResult<()> {
        if password.len() < MIN_PASSWORD_LENGTH {
            return Err(SecurityError::InvalidPassword(format!(
                "Password must be at least {} characters",
                MIN_PASSWORD_LENGTH
            )));
        }

        // Check for basic complexity (at least one letter and one number or special char)
        let has_letter = password.chars().any(|c| c.is_alphabetic());
        let has_number_or_special = password.chars().any(|c| c.is_numeric() || "!@#$%^&*".contains(c));

        if !has_letter || !has_number_or_special {
            return Err(SecurityError::InvalidPassword(
                "Password must contain at least one letter and one number or special character".to_string(),
            ));
        }

        Ok(())
    }

    /// Derives an encryption key from the master password using PBKDF2.
    ///
    /// # Arguments
    ///
    /// * `password` - Master password
    /// * `salt` - Optional salt (if None, generates a new one)
    ///
    /// # Returns
    ///
    /// Derived encryption key for AES-256-GCM
    fn derive_key(password: &str, salt: Option<&[u8]>) -> SecurityResult<EncryptionKey> {
        let salt_bytes = if let Some(s) = salt {
            s.to_vec()
        } else {
            let mut s = vec![0u8; SALT_LENGTH];
            rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut s);
            s
        };

        // Derive 32-byte key for AES-256
        let mut key_bytes = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt_bytes, PBKDF2_ITERATIONS, &mut key_bytes);

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes).clone();

        // Zeroize the key bytes
        key_bytes.zeroize();

        Ok(EncryptionKey { key })
    }

    /// Ensures the vault directory exists with proper permissions.
    fn ensure_vault_dir(&self) -> SecurityResult<()> {
        let dir = self.vault_path.parent().ok_or(SecurityError::PermissionDenied(
            "Invalid vault path".to_string(),
        ))?;

        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        // Set directory permissions to 0700 (rwx------)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o700);
            fs::set_permissions(dir, perms)?;
        }

        Ok(())
    }

    /// Loads the vault file from disk.
    fn load_vault(path: &Path) -> SecurityResult<VaultFile> {
        if !path.exists() {
            // Create new vault with random salt
            let salt_bytes = {
                let mut s = vec![0u8; SALT_LENGTH];
                rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut s);
                s
            };
            let salt = base64::engine::general_purpose::STANDARD.encode(salt_bytes);

            return Ok(VaultFile {
                version: VAULT_VERSION.to_string(),
                salt,
                secrets: HashMap::new(),
            });
        }

        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let vault: VaultFile = serde_json::from_str(&contents)
            .map_err(|e| SecurityError::VaultCorruption(format!("Invalid vault format: {}", e)))?;

        // Validate version
        if vault.version != VAULT_VERSION {
            return Err(SecurityError::InvalidVaultVersion {
                expected: VAULT_VERSION.to_string(),
                found: vault.version,
            });
        }

        Ok(vault)
    }

    /// Saves the vault file to disk with proper permissions.
    fn save_vault(&self, vault: &VaultFile) -> SecurityResult<()> {
        self.ensure_vault_dir()?;

        let json = serde_json::to_string_pretty(vault)
            .map_err(|e| SecurityError::Serialization(e))?;

        let mut file = File::create(&self.vault_path)?;
        file.write_all(json.as_bytes())?;

        // Set file permissions to 0600 (rw-------)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&self.vault_path, perms)?;
        }

        Ok(())
    }

    /// Encrypts a secret value using AES-256-GCM.
    fn encrypt(&self, value: &str) -> SecurityResult<(String, String)> {
        let cipher = Aes256Gcm::new(&self.encryption_key.key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, value.as_bytes())
            .map_err(|e| SecurityError::EncryptionError(format!("Encryption failed: {}", e)))?;

        let encrypted_value = base64::engine::general_purpose::STANDARD.encode(ciphertext);
        let nonce_str = base64::engine::general_purpose::STANDARD.encode(nonce);

        Ok((encrypted_value, nonce_str))
    }

    /// Decrypts a secret value using AES-256-GCM.
    fn decrypt(&self, encrypted_value: &str, nonce: &str) -> SecurityResult<String> {
        let cipher = Aes256Gcm::new(&self.encryption_key.key);

        let ciphertext = base64::engine::general_purpose::STANDARD
            .decode(encrypted_value)
            .map_err(|e| SecurityError::EncryptionError(format!("Invalid base64: {}", e)))?;

        let nonce_bytes = base64::engine::general_purpose::STANDARD
            .decode(nonce)
            .map_err(|e| SecurityError::EncryptionError(format!("Invalid nonce: {}", e)))?;

        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| SecurityError::EncryptionError(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| SecurityError::EncryptionError(format!("Invalid UTF-8: {}", e)))
    }

    /// Stores a secret in the encrypted vault.
    ///
    /// # Arguments
    ///
    /// * `name` - Secret name/identifier
    /// * `value` - Secret value to encrypt and store
    ///
    /// # Errors
    ///
    /// Returns an error if encryption or file operations fail.
    pub fn store_secret(&mut self, name: &str, value: &str) -> SecurityResult<()> {
        let mut vault = Self::load_vault(&self.vault_path)?;

        // Encrypt the value
        let (encrypted_value, nonce) = self.encrypt(value)?;

        // Get current version or start at 1
        let version = vault
            .secrets
            .get(name)
            .map(|e| e.version + 1)
            .unwrap_or(1);

        let now = Utc::now().to_rfc3339();

        let entry = if let Some(existing) = vault.secrets.get(name) {
            SecretEntry {
                encrypted_value,
                nonce,
                version,
                created_at: existing.created_at.clone(),
                updated_at: now,
            }
        } else {
            SecretEntry {
                encrypted_value,
                nonce,
                version,
                created_at: now.clone(),
                updated_at: now,
            }
        };

        vault.secrets.insert(name.to_string(), entry);
        self.save_vault(&vault)?;

        Ok(())
    }

    /// Retrieves a secret from the encrypted vault.
    ///
    /// # Arguments
    ///
    /// * `name` - Secret name/identifier
    ///
    /// # Returns
    ///
    /// Decrypted secret value
    ///
    /// # Errors
    ///
    /// Returns an error if the secret is not found or decryption fails.
    pub fn get_secret(&self, name: &str) -> SecurityResult<String> {
        let vault = Self::load_vault(&self.vault_path)?;

        let entry = vault
            .secrets
            .get(name)
            .ok_or_else(|| SecurityError::SecretNotFound(name.to_string()))?;

        self.decrypt(&entry.encrypted_value, &entry.nonce)
    }

    /// Lists all secret names in the vault.
    ///
    /// # Returns
    ///
    /// Vector of secret names (values are never exposed)
    ///
    /// # Errors
    ///
    /// Returns an error if the vault cannot be loaded.
    pub fn list_secrets(&self) -> SecurityResult<Vec<String>> {
        let vault = Self::load_vault(&self.vault_path)?;
        Ok(vault.secrets.keys().cloned().collect())
    }

    /// Rotates a secret by storing a new value and incrementing the version.
    ///
    /// # Arguments
    ///
    /// * `name` - Secret name/identifier
    /// * `new_value` - New secret value
    ///
    /// # Errors
    ///
    /// Returns an error if the secret doesn't exist or encryption fails.
    pub fn rotate_secret(&mut self, name: &str, new_value: &str) -> SecurityResult<()> {
        // Verify secret exists
        self.get_secret(name)?;

        // Store new value (will increment version)
        self.store_secret(name, new_value)
    }

    /// Removes a secret from the vault.
    ///
    /// # Arguments
    ///
    /// * `name` - Secret name/identifier
    ///
    /// # Errors
    ///
    /// Returns an error if the vault cannot be loaded or saved.
    pub fn remove_secret(&mut self, name: &str) -> SecurityResult<()> {
        let mut vault = Self::load_vault(&self.vault_path)?;

        if vault.secrets.remove(name).is_none() {
            return Err(SecurityError::SecretNotFound(name.to_string()));
        }

        self.save_vault(&vault)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_manager() -> (SecretManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("secrets.vault");
        let manager = SecretManager::new(vault_path, "TestPassword123!").unwrap();
        (manager, temp_dir)
    }

    #[test]
    fn test_password_validation_too_short() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("secrets.vault");
        let result = SecretManager::new(vault_path, "short");
        assert!(matches!(result, Err(SecurityError::InvalidPassword(_))));
    }

    #[test]
    fn test_password_validation_no_complexity() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("secrets.vault");
        let result = SecretManager::new(vault_path, "alllowercaseletters");
        assert!(matches!(result, Err(SecurityError::InvalidPassword(_))));
    }

    #[test]
    fn test_store_and_retrieve_secret() {
        let (mut manager, _temp_dir) = create_test_manager();

        manager.store_secret("test_key", "secret_value").unwrap();
        let value = manager.get_secret("test_key").unwrap();

        assert_eq!(value, "secret_value");
    }

    #[test]
    fn test_get_nonexistent_secret() {
        let (manager, _temp_dir) = create_test_manager();

        let result = manager.get_secret("nonexistent");
        assert!(matches!(result, Err(SecurityError::SecretNotFound(_))));
    }

    #[test]
    fn test_list_secrets() {
        let (mut manager, _temp_dir) = create_test_manager();

        manager.store_secret("key1", "value1").unwrap();
        manager.store_secret("key2", "value2").unwrap();
        manager.store_secret("key3", "value3").unwrap();

        let names = manager.list_secrets().unwrap();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"key1".to_string()));
        assert!(names.contains(&"key2".to_string()));
        assert!(names.contains(&"key3".to_string()));
    }

    #[test]
    fn test_rotate_secret() {
        let (mut manager, _temp_dir) = create_test_manager();

        manager.store_secret("api_key", "old_value").unwrap();
        let entry1 = manager.get_secret("api_key").unwrap();
        assert_eq!(entry1, "old_value");

        manager.rotate_secret("api_key", "new_value").unwrap();
        let entry2 = manager.get_secret("api_key").unwrap();
        assert_eq!(entry2, "new_value");
    }

    #[test]
    fn test_remove_secret() {
        let (mut manager, _temp_dir) = create_test_manager();

        manager.store_secret("temp_key", "temp_value").unwrap();
        manager.remove_secret("temp_key").unwrap();

        let result = manager.get_secret("temp_key");
        assert!(matches!(result, Err(SecurityError::SecretNotFound(_))));
    }

    #[test]
    fn test_encryption_decryption() {
        let (manager, _temp_dir) = create_test_manager();

        let original = "sensitive_data_12345";
        let (encrypted, nonce) = manager.encrypt(original).unwrap();
        let decrypted = manager.decrypt(&encrypted, &nonce).unwrap();

        assert_eq!(original, decrypted);
        assert_ne!(encrypted, original);
    }

    #[test]
    fn test_vault_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("secrets.vault");

        {
            let mut manager = SecretManager::new(vault_path.clone(), "TestPassword123!").unwrap();
            manager.store_secret("persistent_key", "persistent_value").unwrap();
        }

        // Create new manager from existing vault
        let manager = SecretManager::from_existing(vault_path, "TestPassword123!").unwrap();
        let value = manager.get_secret("persistent_key").unwrap();
        assert_eq!(value, "persistent_value");
    }
}


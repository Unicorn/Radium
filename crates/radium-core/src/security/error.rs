//! Error types for security operations.

use thiserror::Error;

/// Security-related errors.
#[derive(Error, Debug)]
pub enum SecurityError {
    /// Secret not found.
    #[error("Secret not found: {0}")]
    SecretNotFound(String),

    /// Invalid master password (too short or weak).
    #[error("Invalid master password: {0}")]
    InvalidPassword(String),

    /// Encryption/decryption error.
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    /// Key derivation error.
    #[error("Key derivation error: {0}")]
    KeyDerivationError(String),

    /// I/O error occurred during file operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Permission denied error.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Vault file corruption or invalid format.
    #[error("Vault corruption: {0}")]
    VaultCorruption(String),

    /// Invalid vault version.
    #[error("Invalid vault version: expected {expected}, found {found}")]
    InvalidVaultVersion { expected: String, found: String },
}

/// Result type alias for security operations.
pub type SecurityResult<T> = std::result::Result<T, SecurityError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_error_secret_not_found() {
        let err = SecurityError::SecretNotFound("api_key".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Secret not found"));
        assert!(msg.contains("api_key"));
    }

    #[test]
    fn test_security_error_invalid_password() {
        let err = SecurityError::InvalidPassword("Password too short".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid master password"));
        assert!(msg.contains("Password too short"));
    }

    #[test]
    fn test_security_error_encryption() {
        let err = SecurityError::EncryptionError("AES-GCM error".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Encryption error"));
        assert!(msg.contains("AES-GCM error"));
    }

    #[test]
    fn test_security_error_vault_corruption() {
        let err = SecurityError::VaultCorruption("Invalid JSON".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Vault corruption"));
        assert!(msg.contains("Invalid JSON"));
    }
}

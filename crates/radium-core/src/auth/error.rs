//! Error types for authentication operations.

use thiserror::Error;

/// Authentication-related errors.
#[derive(Error, Debug)]
pub enum AuthError {
    /// Credential not found for the specified provider.
    #[error("Credential not found for provider: {0}")]
    CredentialNotFound(String),

    /// Invalid credential format.
    #[error("Invalid credential format")]
    InvalidFormat,

    /// Provider is not supported.
    #[error("Provider not supported: {0}")]
    UnsupportedProvider(String),

    /// I/O error occurred during file operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Permission denied error.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

/// Result type alias for authentication operations.
pub type AuthResult<T> = std::result::Result<T, AuthError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_credential_not_found() {
        let err = AuthError::CredentialNotFound("gemini".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Credential not found"));
        assert!(msg.contains("gemini"));
    }

    #[test]
    fn test_auth_error_invalid_format() {
        let err = AuthError::InvalidFormat;
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid credential format"));
    }

    #[test]
    fn test_auth_error_unsupported_provider() {
        let err = AuthError::UnsupportedProvider("unknown".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Provider not supported"));
        assert!(msg.contains("unknown"));
    }

    #[test]
    fn test_auth_error_permission_denied() {
        let err = AuthError::PermissionDenied("HOME not set".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Permission denied"));
        assert!(msg.contains("HOME not set"));
    }
}

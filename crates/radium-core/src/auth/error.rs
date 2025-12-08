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

    /// Connection failed during validation.
    #[error("Connection failed for {provider}: {reason}")]
    ConnectionFailed { provider: String, reason: String },

    /// API key is unauthorized.
    #[error("Unauthorized for {provider}")]
    Unauthorized { provider: String },

    /// Rate limited by provider.
    #[error("Rate limited for {provider}")]
    RateLimited {
        provider: String,
        retry_after: Option<std::time::Duration>,
    },

    /// Provider service is unavailable.
    #[error("Service unavailable for {provider}")]
    ServiceUnavailable { provider: String },

    /// Validation timeout.
    #[error("Validation timeout for {provider}")]
    Timeout { provider: String },
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

    #[test]
    fn test_auth_error_connection_failed() {
        let err = AuthError::ConnectionFailed {
            provider: "Gemini (Google)".to_string(),
            reason: "Network error".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Connection failed"));
        assert!(msg.contains("Gemini (Google)"));
        assert!(msg.contains("Network error"));
    }

    #[test]
    fn test_auth_error_unauthorized() {
        let err = AuthError::Unauthorized {
            provider: "OpenAI (GPT)".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Unauthorized"));
        assert!(msg.contains("OpenAI (GPT)"));
    }

    #[test]
    fn test_auth_error_rate_limited() {
        let err = AuthError::RateLimited {
            provider: "Claude (Anthropic)".to_string(),
            retry_after: Some(std::time::Duration::from_secs(60)),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Rate limited"));
        assert!(msg.contains("Claude (Anthropic)"));
    }

    #[test]
    fn test_auth_error_service_unavailable() {
        let err = AuthError::ServiceUnavailable {
            provider: "Gemini (Google)".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Service unavailable"));
        assert!(msg.contains("Gemini (Google)"));
    }

    #[test]
    fn test_auth_error_timeout() {
        let err = AuthError::Timeout {
            provider: "OpenAI (GPT)".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Validation timeout"));
        assert!(msg.contains("OpenAI (GPT)"));
    }
}

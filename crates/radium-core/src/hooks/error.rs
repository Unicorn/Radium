//! Error types for the hooks system.

use thiserror::Error;

/// Errors that can occur in the hooks system.
#[derive(Error, Debug)]
pub enum HookError {
    /// Hook registration error.
    #[error("Failed to register hook: {0}")]
    RegistrationFailed(String),

    /// Hook execution error.
    #[error("Hook execution failed: {0}")]
    ExecutionFailed(String),

    /// Hook not found.
    #[error("Hook not found: {0}")]
    NotFound(String),

    /// Invalid hook configuration.
    #[error("Invalid hook configuration: {0}")]
    InvalidConfig(String),

    /// Hook validation error.
    #[error("Hook validation failed: {0}")]
    ValidationFailed(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration parsing error.
    #[error("Configuration parsing error: {0}")]
    ConfigParse(#[from] toml::de::Error),

    /// Hook discovery error.
    #[error("Hook discovery error: {0}")]
    Discovery(String),
}

/// Result type for hook operations.
pub type Result<T> = std::result::Result<T, HookError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_error_display_all_variants() {
        // Test RegistrationFailed
        let err = HookError::RegistrationFailed("test".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Failed to register hook"));
        assert!(msg.contains("test"));

        // Test ExecutionFailed
        let err = HookError::ExecutionFailed("test".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Hook execution failed"));

        // Test NotFound
        let err = HookError::NotFound("test".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Hook not found"));

        // Test InvalidConfig
        let err = HookError::InvalidConfig("test".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid hook configuration"));

        // Test ValidationFailed
        let err = HookError::ValidationFailed("test".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Hook validation failed"));

        // Test Discovery
        let err = HookError::Discovery("test".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Hook discovery error"));
    }

    #[test]
    fn test_hook_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let hook_err: HookError = io_err.into();
        assert!(matches!(hook_err, HookError::Io(_)));
    }

    #[test]
    fn test_hook_error_from_serialization_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let hook_err: HookError = json_err.into();
        assert!(matches!(hook_err, HookError::Serialization(_)));
    }

    #[test]
    fn test_hook_error_from_config_parse_error() {
        let toml_err = toml::from_str::<toml::Value>("invalid toml {").unwrap_err();
        let hook_err: HookError = toml_err.into();
        assert!(matches!(hook_err, HookError::ConfigParse(_)));
    }

    #[test]
    fn test_hook_error_debug() {
        let err = HookError::RegistrationFailed("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("RegistrationFailed"));
    }
}

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

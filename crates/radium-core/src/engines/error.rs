//! Error types for engine operations.

use std::io;
use thiserror::Error;

/// Result type for engine operations.
pub type Result<T> = std::result::Result<T, EngineError>;

/// Errors that can occur during engine operations.
#[derive(Debug, Error)]
pub enum EngineError {
    /// Engine not found.
    #[error("Engine not found: {0}")]
    NotFound(String),

    /// Engine binary not available.
    #[error("Engine binary not available: {0}")]
    BinaryNotFound(String),

    /// Engine not authenticated.
    #[error("Engine not authenticated: {0}")]
    NotAuthenticated(String),

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Engine execution error.
    #[error("Engine execution error: {0}")]
    ExecutionError(String),

    /// Invalid engine configuration.
    #[error("Invalid engine configuration: {0}")]
    InvalidConfig(String),

    /// Engine registry error.
    #[error("Engine registry error: {0}")]
    RegistryError(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

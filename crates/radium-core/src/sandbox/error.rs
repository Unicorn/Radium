//! Error types for sandbox operations.

use std::io;
use thiserror::Error;

/// Result type for sandbox operations.
pub type Result<T> = std::result::Result<T, SandboxError>;

/// Errors that can occur during sandbox operations.
#[derive(Debug, Error)]
pub enum SandboxError {
    /// Sandbox initialization failed.
    #[error("Sandbox initialization failed: {0}")]
    InitFailed(String),

    /// Sandbox execution failed.
    #[error("Sandbox execution failed: {0}")]
    ExecutionFailed(String),

    /// Sandbox not available on this platform.
    #[error("Sandbox type '{0}' not available on this platform")]
    NotAvailable(String),

    /// Sandbox configuration error.
    #[error("Sandbox configuration error: {0}")]
    ConfigError(String),

    /// Docker/Podman not found.
    #[error("Docker/Podman not found: {0}")]
    ContainerRuntimeNotFound(String),

    /// Seatbelt not available (macOS only).
    #[error("macOS Seatbelt not available: {0}")]
    SeatbeltNotAvailable(String),

    /// Invalid sandbox profile.
    #[error("Invalid sandbox profile: {0}")]
    InvalidProfile(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

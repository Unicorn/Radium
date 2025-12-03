//! Monitoring system error types.

use std::io;

/// Monitoring system errors.
#[derive(Debug, thiserror::Error)]
pub enum MonitoringError {
    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// I/O error during monitoring operations.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Agent not found.
    #[error("agent not found: {0}")]
    AgentNotFound(String),

    /// Invalid agent status.
    #[error("invalid agent status: {0}")]
    InvalidStatus(String),

    /// Telemetry parsing error.
    #[error("telemetry parsing error: {0}")]
    TelemetryParse(String),

    /// Checkpoint error.
    #[error("checkpoint error: {0}")]
    Checkpoint(String),
}

/// Result type for monitoring operations.
pub type Result<T> = std::result::Result<T, MonitoringError>;

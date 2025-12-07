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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitoring_error_display() {
        let error = MonitoringError::AgentNotFound("test-agent".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("agent not found"));
        assert!(msg.contains("test-agent"));
    }

    #[test]
    fn test_monitoring_error_invalid_status() {
        let error = MonitoringError::InvalidStatus("invalid".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("invalid agent status"));
    }

    #[test]
    fn test_monitoring_error_telemetry_parse() {
        let error = MonitoringError::TelemetryParse("parse error".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("telemetry parsing error"));
    }

    #[test]
    fn test_monitoring_error_checkpoint() {
        let error = MonitoringError::Checkpoint("checkpoint error".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("checkpoint error"));
    }

    #[test]
    fn test_monitoring_error_from_database_error() {
        let db_error = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
            None,
        );
        let monitoring_error: MonitoringError = db_error.into();
        assert!(matches!(monitoring_error, MonitoringError::Database(_)));
    }

    #[test]
    fn test_monitoring_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let monitoring_error: MonitoringError = io_error.into();
        assert!(matches!(monitoring_error, MonitoringError::Io(_)));
    }
}

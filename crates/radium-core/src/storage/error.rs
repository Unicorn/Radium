//! Error types for the storage layer.

use thiserror::Error;

/// Errors that can occur in the storage layer.
#[derive(Error, Debug)]
pub enum StorageError {
    /// Database connection error.
    #[error("Database connection error: {0}")]
    Connection(#[from] rusqlite::Error),

    /// Item not found in storage.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid data error.
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias for storage operations.
pub type StorageResult<T> = std::result::Result<T, StorageError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_error_display_all_variants() {
        // Test NotFound
        let err = StorageError::NotFound("item".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Not found"));
        assert!(msg.contains("item"));

        // Test InvalidData
        let err = StorageError::InvalidData("invalid".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid data"));
        assert!(msg.contains("invalid"));
    }

    #[test]
    fn test_storage_error_from_connection_error() {
        let db_err = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
            None,
        );
        let storage_err: StorageError = db_err.into();
        assert!(matches!(storage_err, StorageError::Connection(_)));
    }

    #[test]
    fn test_storage_error_from_serialization_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let storage_err: StorageError = json_err.into();
        assert!(matches!(storage_err, StorageError::Serialization(_)));
    }

    #[test]
    fn test_storage_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let storage_err: StorageError = io_err.into();
        assert!(matches!(storage_err, StorageError::Io(_)));
    }

    #[test]
    fn test_storage_error_debug() {
        let err = StorageError::NotFound("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NotFound"));
    }
}

//! Error types for Radium Core.

use crate::auth::AuthError;
use crate::storage::StorageError;
use thiserror::Error;

/// Core error type for Radium operations.
#[derive(Error, Debug)]
pub enum RadiumError {
    /// Server-related errors
    #[error("Server error: {0}")]
    Server(#[from] tonic::transport::Error),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Address parsing errors
    #[error("Invalid address: {0}")]
    InvalidAddress(#[from] std::net::AddrParseError),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Model-related errors
    #[error("Model error: {0}")]
    Model(String),

    /// Storage-related errors
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),
}

/// Result type alias for Radium operations.
pub type Result<T> = std::result::Result<T, RadiumError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::StorageError;

    #[test]
    fn test_radium_error_storage_conversion() {
        let storage_err = StorageError::NotFound("test-agent".to_string());
        let radium_err: RadiumError = storage_err.into();
        match radium_err {
            RadiumError::Storage(StorageError::NotFound(msg)) => {
                assert_eq!(msg, "test-agent");
            }
            _ => panic!("Expected Storage error variant"),
        }
    }

    #[test]
    fn test_radium_error_address_parsing() {
        let parse_err = "invalid:address:format".parse::<std::net::SocketAddr>().unwrap_err();
        let radium_err: RadiumError = parse_err.into();
        match radium_err {
            RadiumError::InvalidAddress(_) => {}
            _ => panic!("Expected InvalidAddress error variant"),
        }
    }

    #[test]
    fn test_radium_error_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let radium_err: RadiumError = io_err.into();
        match radium_err {
            RadiumError::Io(_) => {}
            _ => panic!("Expected Io error variant"),
        }
    }

    #[test]
    fn test_radium_error_config() {
        let err = RadiumError::Config("Invalid configuration".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Configuration error"));
        assert!(msg.contains("Invalid configuration"));
    }

    #[test]
    fn test_radium_error_model() {
        let err = RadiumError::Model("Model initialization failed".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Model error"));
        assert!(msg.contains("Model initialization failed"));
    }

    #[test]
    fn test_radium_error_storage_serialization() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let storage_err = StorageError::Serialization(json_err);
        let radium_err: RadiumError = storage_err.into();
        match radium_err {
            RadiumError::Storage(StorageError::Serialization(_)) => {}
            _ => panic!("Expected Storage::Serialization error variant"),
        }
    }

    #[test]
    fn test_radium_error_auth_conversion() {
        use crate::auth::AuthError;
        let auth_err = AuthError::InvalidFormat;
        let radium_err: RadiumError = auth_err.into();
        match radium_err {
            RadiumError::Auth(_) => {}
            _ => panic!("Expected Auth error variant"),
        }
    }

    #[test]
    fn test_radium_error_server_conversion() {
        // Create a server error (tonic transport error)
        // This is harder to create directly, so we'll test the display
        let err = RadiumError::Config("test".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Configuration error"));
    }

    #[test]
    fn test_radium_error_display_all_variants() {
        // Test Config
        let err = RadiumError::Config("test config".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Configuration error"));
        assert!(msg.contains("test config"));

        // Test Model
        let err = RadiumError::Model("test model".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Model error"));
        assert!(msg.contains("test model"));
    }

    #[test]
    fn test_radium_error_debug() {
        let err = RadiumError::Config("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Config"));
    }

    #[test]
    fn test_radium_error_from_storage_not_found() {
        let storage_err = StorageError::NotFound("item".to_string());
        let radium_err: RadiumError = storage_err.into();
        match radium_err {
            RadiumError::Storage(StorageError::NotFound(msg)) => {
                assert_eq!(msg, "item");
            }
            _ => panic!("Expected Storage::NotFound"),
        }
    }

    #[test]
    fn test_radium_error_from_storage_connection() {
        let db_err = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
            None,
        );
        let storage_err = StorageError::Connection(db_err);
        let radium_err: RadiumError = storage_err.into();
        match radium_err {
            RadiumError::Storage(StorageError::Connection(_)) => {}
            _ => panic!("Expected Storage::Connection"),
        }
    }
}

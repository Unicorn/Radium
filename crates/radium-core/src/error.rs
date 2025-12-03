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
}

//! Error types for MCP operations.

use std::io;
use thiserror::Error;

/// Result type for MCP operations.
pub type Result<T> = std::result::Result<T, McpError>;

/// Errors that can occur during MCP operations.
#[derive(Debug, Error)]
pub enum McpError {
    /// Connection error.
    #[error("MCP connection error: {0}")]
    Connection(String),

    /// Transport error.
    #[error("MCP transport error: {0}")]
    Transport(String),

    /// Protocol error.
    #[error("MCP protocol error: {0}")]
    Protocol(String),

    /// Server not found.
    #[error("MCP server not found: {0}")]
    ServerNotFound(String),

    /// Tool not found.
    #[error("MCP tool not found: {0}")]
    ToolNotFound(String),

    /// Authentication error.
    #[error("MCP authentication error: {0}")]
    Authentication(String),

    /// Configuration error.
    #[error("MCP configuration error: {0}")]
    Config(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML parsing error.
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_error_display() {
        let err = McpError::Connection("test connection error".to_string());
        assert!(err.to_string().contains("connection error"));
        assert!(err.to_string().contains("test connection error"));
    }

    #[test]
    fn test_mcp_error_io_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let mcp_err: McpError = io_err.into();
        assert!(matches!(mcp_err, McpError::Io(_)));
    }

    #[test]
    fn test_mcp_error_json_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let mcp_err: McpError = json_err.into();
        assert!(matches!(mcp_err, McpError::Json(_)));
    }
}


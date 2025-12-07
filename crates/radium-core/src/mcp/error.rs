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
    fn test_mcp_error_connection_display() {
        let err = McpError::Connection("test connection error".to_string());
        let display = err.to_string();
        assert!(display.contains("connection error"));
        assert!(display.contains("test connection error"));
    }

    #[test]
    fn test_mcp_error_transport_display() {
        let err = McpError::Transport("transport failed".to_string());
        let display = err.to_string();
        assert!(display.contains("transport error"));
        assert!(display.contains("transport failed"));
    }

    #[test]
    fn test_mcp_error_protocol_display() {
        let err = McpError::Protocol("invalid protocol".to_string());
        let display = err.to_string();
        assert!(display.contains("protocol error"));
        assert!(display.contains("invalid protocol"));
    }

    #[test]
    fn test_mcp_error_server_not_found_display() {
        let err = McpError::ServerNotFound("my-server".to_string());
        let display = err.to_string();
        assert!(display.contains("server not found"));
        assert!(display.contains("my-server"));
    }

    #[test]
    fn test_mcp_error_tool_not_found_display() {
        let err = McpError::ToolNotFound("my-tool".to_string());
        let display = err.to_string();
        assert!(display.contains("tool not found"));
        assert!(display.contains("my-tool"));
    }

    #[test]
    fn test_mcp_error_authentication_display() {
        let err = McpError::Authentication("auth failed".to_string());
        let display = err.to_string();
        assert!(display.contains("authentication error"));
        assert!(display.contains("auth failed"));
    }

    #[test]
    fn test_mcp_error_config_display() {
        let err = McpError::Config("invalid config".to_string());
        let display = err.to_string();
        assert!(display.contains("configuration error"));
        assert!(display.contains("invalid config"));
    }

    #[test]
    fn test_mcp_error_io_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let mcp_err: McpError = io_err.into();
        assert!(matches!(mcp_err, McpError::Io(_)));
    }

    #[test]
    fn test_mcp_error_io_display() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let mcp_err: McpError = io_err.into();
        let display = mcp_err.to_string();
        assert!(display.contains("I/O error"));
    }

    #[test]
    fn test_mcp_error_json_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let mcp_err: McpError = json_err.into();
        assert!(matches!(mcp_err, McpError::Json(_)));
    }

    #[test]
    fn test_mcp_error_json_display() {
        let json_err = serde_json::from_str::<serde_json::Value>("{invalid}").unwrap_err();
        let mcp_err: McpError = json_err.into();
        let display = mcp_err.to_string();
        assert!(display.contains("JSON error"));
    }

    #[test]
    fn test_mcp_error_toml_conversion() {
        let toml_err = toml::from_str::<toml::Value>("invalid = toml").unwrap_err();
        let mcp_err: McpError = toml_err.into();
        assert!(matches!(mcp_err, McpError::TomlParse(_)));
    }

    #[test]
    fn test_mcp_error_toml_display() {
        let toml_err = toml::from_str::<toml::Value>("[invalid").unwrap_err();
        let mcp_err: McpError = toml_err.into();
        let display = mcp_err.to_string();
        assert!(display.contains("TOML parse error"));
    }

    #[test]
    fn test_mcp_error_source_chain() {
        use std::error::Error;
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let mcp_err: McpError = io_err.into();

        // Test that source() returns the underlying error
        let source = mcp_err.source();
        assert!(source.is_some());
    }

    #[test]
    fn test_all_error_variants() {
        // Test that all error variants can be created and displayed
        let variants = vec![
            McpError::Connection("test".to_string()),
            McpError::Transport("test".to_string()),
            McpError::Protocol("test".to_string()),
            McpError::ServerNotFound("test".to_string()),
            McpError::ToolNotFound("test".to_string()),
            McpError::Authentication("test".to_string()),
            McpError::Config("test".to_string()),
        ];

        for err in variants {
            let display = err.to_string();
            assert!(!display.is_empty());
            assert!(display.contains("test"));
        }
    }
}

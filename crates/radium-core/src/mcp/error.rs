//! Error types for MCP operations.

use std::io;
use thiserror::Error;

/// Result type for MCP operations.
pub type Result<T> = std::result::Result<T, McpError>;

/// Errors that can occur during MCP operations.
#[derive(Debug, Error)]
pub enum McpError {
    /// Connection error.
    #[error("MCP connection error: {message}\n\n{suggestion}")]
    Connection {
        message: String,
        suggestion: String,
    },

    /// Transport error.
    #[error("MCP transport error: {message}\n\n{suggestion}")]
    Transport {
        message: String,
        suggestion: String,
    },

    /// Protocol error.
    #[error("MCP protocol error: {message}\n\n{suggestion}")]
    Protocol {
        message: String,
        suggestion: String,
    },

    /// Server not found.
    #[error("MCP server not found: {server_name}\n\n{suggestion}")]
    ServerNotFound {
        server_name: String,
        suggestion: String,
    },

    /// Tool not found.
    #[error("MCP tool not found: {tool_name}\n\n{suggestion}")]
    ToolNotFound {
        tool_name: String,
        suggestion: String,
    },

    /// Authentication error.
    #[error("MCP authentication error: {message}\n\n{suggestion}")]
    Authentication {
        message: String,
        suggestion: String,
    },

    /// Configuration error.
    #[error("MCP configuration error: {message}\n\n{suggestion}")]
    Config {
        message: String,
        suggestion: String,
    },

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

impl McpError {
    /// Create a connection error with a helpful suggestion.
    pub fn connection(message: impl Into<String>, suggestion: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
            suggestion: suggestion.into(),
        }
    }

    /// Create a transport error with a helpful suggestion.
    pub fn transport(message: impl Into<String>, suggestion: impl Into<String>) -> Self {
        Self::Transport {
            message: message.into(),
            suggestion: suggestion.into(),
        }
    }

    /// Create a protocol error with a helpful suggestion.
    pub fn protocol(message: impl Into<String>, suggestion: impl Into<String>) -> Self {
        Self::Protocol {
            message: message.into(),
            suggestion: suggestion.into(),
        }
    }

    /// Create a server not found error with a helpful suggestion.
    pub fn server_not_found(server_name: impl Into<String>, suggestion: impl Into<String>) -> Self {
        Self::ServerNotFound {
            server_name: server_name.into(),
            suggestion: suggestion.into(),
        }
    }

    /// Create a tool not found error with a helpful suggestion.
    pub fn tool_not_found(tool_name: impl Into<String>, suggestion: impl Into<String>) -> Self {
        Self::ToolNotFound {
            tool_name: tool_name.into(),
            suggestion: suggestion.into(),
        }
    }

    /// Create an authentication error with a helpful suggestion.
    pub fn authentication(message: impl Into<String>, suggestion: impl Into<String>) -> Self {
        Self::Authentication {
            message: message.into(),
            suggestion: suggestion.into(),
        }
    }

    /// Create a configuration error with a helpful suggestion.
    pub fn config(message: impl Into<String>, suggestion: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            suggestion: suggestion.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_error_connection_display() {
        let err = McpError::connection("test connection error", "test suggestion");
        let display = err.to_string();
        assert!(display.contains("connection error"));
        assert!(display.contains("test connection error"));
        assert!(display.contains("test suggestion"));
    }

    #[test]
    fn test_mcp_error_transport_display() {
        let err = McpError::transport("transport failed", "test suggestion");
        let display = err.to_string();
        assert!(display.contains("transport error"));
        assert!(display.contains("transport failed"));
        assert!(display.contains("test suggestion"));
    }

    #[test]
    fn test_mcp_error_protocol_display() {
        let err = McpError::protocol("invalid protocol", "test suggestion");
        let display = err.to_string();
        assert!(display.contains("protocol error"));
        assert!(display.contains("invalid protocol"));
        assert!(display.contains("test suggestion"));
    }

    #[test]
    fn test_mcp_error_server_not_found_display() {
        let err = McpError::server_not_found("my-server", "test suggestion");
        let display = err.to_string();
        assert!(display.contains("server not found"));
        assert!(display.contains("my-server"));
        assert!(display.contains("test suggestion"));
    }

    #[test]
    fn test_mcp_error_tool_not_found_display() {
        let err = McpError::tool_not_found("my-tool", "test suggestion");
        let display = err.to_string();
        assert!(display.contains("tool not found"));
        assert!(display.contains("my-tool"));
        assert!(display.contains("test suggestion"));
    }

    #[test]
    fn test_mcp_error_authentication_display() {
        let err = McpError::authentication("auth failed", "test suggestion");
        let display = err.to_string();
        assert!(display.contains("authentication error"));
        assert!(display.contains("auth failed"));
        assert!(display.contains("test suggestion"));
    }

    #[test]
    fn test_mcp_error_config_display() {
        let err = McpError::config("invalid config", "test suggestion");
        let display = err.to_string();
        assert!(display.contains("configuration error"));
        assert!(display.contains("invalid config"));
        assert!(display.contains("test suggestion"));
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
            McpError::connection("test", "suggestion"),
            McpError::transport("test", "suggestion"),
            McpError::protocol("test", "suggestion"),
            McpError::server_not_found("test", "suggestion"),
            McpError::tool_not_found("test", "suggestion"),
            McpError::authentication("test", "suggestion"),
            McpError::config("test", "suggestion"),
        ];

        for err in variants {
            let display = err.to_string();
            assert!(!display.is_empty());
            assert!(display.contains("test"));
            assert!(display.contains("suggestion"));
        }
    }
}

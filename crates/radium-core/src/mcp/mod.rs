//! Model Context Protocol (MCP) integration for Radium.
//!
//! This module provides MCP client functionality for connecting to and communicating
//! with MCP servers, enabling external tool discovery and execution.

pub mod auth;
pub mod client;
pub mod config;
pub mod content;
pub mod error;
pub mod integration;
pub mod messages;
pub mod prompts;
pub mod tools;
pub mod transport;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use error::{McpError, Result};

/// MCP transport types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    /// Standard input/output transport for local servers.
    Stdio,
    /// Server-Sent Events transport for HTTP streaming.
    Sse,
    /// HTTP streaming transport for remote servers.
    Http,
}

/// MCP server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Server name/identifier.
    pub name: String,
    /// Transport type to use.
    pub transport: TransportType,
    /// Command to execute (for stdio transport).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Command arguments (for stdio transport).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    /// Server URL (for SSE/HTTP transports).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Authentication configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<McpAuthConfig>,
}

/// MCP authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpAuthConfig {
    /// Authentication type (e.g., "oauth", "bearer").
    pub auth_type: String,
    /// Authentication parameters.
    #[serde(flatten)]
    pub params: HashMap<String, String>,
}

/// MCP server information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    /// Server name.
    pub name: String,
    /// Server version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Server capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<McpCapabilities>,
}

/// MCP server capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCapabilities {
    /// Tools capability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<McpToolsCapability>,
    /// Prompts capability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<McpPromptsCapability>,
}

/// MCP tools capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolsCapability {
    /// List of available tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// MCP prompts capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptsCapability {
    /// List of available prompts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// MCP transport trait for different transport implementations.
#[async_trait::async_trait]
pub trait McpTransport: Send + Sync {
    /// Connect to the MCP server.
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from the MCP server.
    async fn disconnect(&mut self) -> Result<()>;

    /// Send a message to the server.
    async fn send(&mut self, message: &[u8]) -> Result<()>;

    /// Receive a message from the server.
    async fn receive(&mut self) -> Result<Vec<u8>>;

    /// Check if the transport is connected.
    fn is_connected(&self) -> bool;
}

// Re-export for convenience
pub use auth::OAuthTokenManager;
pub use client::McpClient;
pub use config::McpConfigManager;
pub use content::ContentHandler;
pub use integration::McpIntegration;
pub use prompts::SlashCommandRegistry;
pub use tools::McpToolRegistry;

/// MCP tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// Tool name.
    pub name: String,
    /// Tool description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tool input schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
}

/// MCP tool execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    /// Tool execution result content.
    pub content: Vec<McpContent>,
    /// Whether the tool execution was successful.
    #[serde(default)]
    pub is_error: bool,
}

/// MCP content (text, image, audio).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpContent {
    /// Text content.
    Text {
        /// Text content.
        text: String,
    },
    /// Image content.
    Image {
        /// Image data (base64 encoded or URL).
        data: String,
        /// MIME type.
        mime_type: String,
    },
    /// Audio content.
    Audio {
        /// Audio data (base64 encoded or URL).
        data: String,
        /// MIME type.
        mime_type: String,
    },
}

/// MCP prompt definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    /// Prompt name.
    pub name: String,
    /// Prompt description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Prompt arguments schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<McpPromptArgument>>,
}

/// MCP prompt argument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
    /// Argument name.
    pub name: String,
    /// Argument description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the argument is required.
    #[serde(default)]
    pub required: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_type_serialization() {
        let transport = TransportType::Stdio;
        let json = serde_json::to_string(&transport).unwrap();
        assert_eq!(json, "\"stdio\"");
    }

    #[test]
    fn test_transport_type_deserialization() {
        let json = "\"sse\"";
        let transport: TransportType = serde_json::from_str(json).unwrap();
        assert_eq!(transport, TransportType::Sse);
    }

    #[test]
    fn test_mcp_server_config() {
        let config = McpServerConfig {
            name: "test-server".to_string(),
            transport: TransportType::Stdio,
            command: Some("mcp-server".to_string()),
            args: Some(vec!["--config".to_string(), "config.json".to_string()]),
            url: None,
            auth: None,
        };

        assert_eq!(config.name, "test-server");
        assert_eq!(config.transport, TransportType::Stdio);
        assert!(config.command.is_some());
        assert!(config.args.is_some());
    }

    #[test]
    fn test_mcp_content_text() {
        let content = McpContent::Text {
            text: "Hello, world!".to_string(),
        };

        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("text"));
        assert!(json.contains("Hello, world!"));
    }

    #[test]
    fn test_mcp_content_image() {
        let content = McpContent::Image {
            data: "base64data".to_string(),
            mime_type: "image/png".to_string(),
        };

        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("image"));
        assert!(json.contains("base64data"));
        assert!(json.contains("image/png"));
    }
}


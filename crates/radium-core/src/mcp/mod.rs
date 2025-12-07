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
pub mod orchestration_bridge;
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
    fn test_transport_type_serialization_stdio() {
        let transport = TransportType::Stdio;
        let json = serde_json::to_string(&transport).unwrap();
        assert_eq!(json, "\"stdio\"");
    }

    #[test]
    fn test_transport_type_serialization_sse() {
        let transport = TransportType::Sse;
        let json = serde_json::to_string(&transport).unwrap();
        assert_eq!(json, "\"sse\"");
    }

    #[test]
    fn test_transport_type_serialization_http() {
        let transport = TransportType::Http;
        let json = serde_json::to_string(&transport).unwrap();
        assert_eq!(json, "\"http\"");
    }

    #[test]
    fn test_transport_type_deserialization_stdio() {
        let json = "\"stdio\"";
        let transport: TransportType = serde_json::from_str(json).unwrap();
        assert_eq!(transport, TransportType::Stdio);
    }

    #[test]
    fn test_transport_type_deserialization_sse() {
        let json = "\"sse\"";
        let transport: TransportType = serde_json::from_str(json).unwrap();
        assert_eq!(transport, TransportType::Sse);
    }

    #[test]
    fn test_transport_type_deserialization_http() {
        let json = "\"http\"";
        let transport: TransportType = serde_json::from_str(json).unwrap();
        assert_eq!(transport, TransportType::Http);
    }

    #[test]
    fn test_transport_type_equality() {
        assert_eq!(TransportType::Stdio, TransportType::Stdio);
        assert_ne!(TransportType::Stdio, TransportType::Sse);
    }

    #[test]
    fn test_mcp_server_config_stdio() {
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
    fn test_mcp_server_config_sse() {
        let config = McpServerConfig {
            name: "sse-server".to_string(),
            transport: TransportType::Sse,
            command: None,
            args: None,
            url: Some("http://localhost:8080/sse".to_string()),
            auth: None,
        };

        assert_eq!(config.name, "sse-server");
        assert_eq!(config.transport, TransportType::Sse);
        assert!(config.url.is_some());
    }

    #[test]
    fn test_mcp_server_config_http() {
        let config = McpServerConfig {
            name: "http-server".to_string(),
            transport: TransportType::Http,
            command: None,
            args: None,
            url: Some("https://api.example.com/mcp".to_string()),
            auth: None,
        };

        assert_eq!(config.name, "http-server");
        assert_eq!(config.transport, TransportType::Http);
        assert!(config.url.is_some());
    }

    #[test]
    fn test_mcp_server_config_with_auth() {
        let mut auth_params = HashMap::new();
        auth_params.insert("client_id".to_string(), "test-id".to_string());
        auth_params.insert("client_secret".to_string(), "test-secret".to_string());

        let config = McpServerConfig {
            name: "auth-server".to_string(),
            transport: TransportType::Http,
            command: None,
            args: None,
            url: Some("https://api.example.com/mcp".to_string()),
            auth: Some(McpAuthConfig { auth_type: "oauth".to_string(), params: auth_params }),
        };

        assert!(config.auth.is_some());
        assert_eq!(config.auth.as_ref().unwrap().auth_type, "oauth");
    }

    #[test]
    fn test_mcp_server_config_serialization() {
        let config = McpServerConfig {
            name: "test-server".to_string(),
            transport: TransportType::Stdio,
            command: Some("mcp-server".to_string()),
            args: Some(vec!["--config".to_string()]),
            url: None,
            auth: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("test-server"));
        assert!(json.contains("stdio"));
    }

    #[test]
    fn test_mcp_server_info() {
        let info = McpServerInfo {
            name: "test-server".to_string(),
            version: Some("1.0.0".to_string()),
            capabilities: Some(McpCapabilities {
                tools: Some(McpToolsCapability { list_changed: Some(true) }),
                prompts: Some(McpPromptsCapability { list_changed: Some(false) }),
            }),
        };

        assert_eq!(info.name, "test-server");
        assert_eq!(info.version, Some("1.0.0".to_string()));
        assert!(info.capabilities.is_some());
    }

    #[test]
    fn test_mcp_server_info_serialization() {
        let info = McpServerInfo {
            name: "test-server".to_string(),
            version: Some("1.0.0".to_string()),
            capabilities: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("test-server"));
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn test_mcp_tool() {
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: Some(serde_json::json!({"type": "object"})),
        };

        assert_eq!(tool.name, "test_tool");
        assert!(tool.description.is_some());
        assert!(tool.input_schema.is_some());
    }

    #[test]
    fn test_mcp_tool_serialization() {
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: None,
        };

        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("test_tool"));
        assert!(json.contains("A test tool"));
    }

    #[test]
    fn test_mcp_tool_result() {
        let result = McpToolResult {
            content: vec![McpContent::Text { text: "Success".to_string() }],
            is_error: false,
        };

        assert_eq!(result.content.len(), 1);
        assert!(!result.is_error);
    }

    #[test]
    fn test_mcp_tool_result_serialization() {
        let result = McpToolResult {
            content: vec![McpContent::Text { text: "Success".to_string() }],
            is_error: false,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Success"));
    }

    #[test]
    fn test_mcp_content_text() {
        let content = McpContent::Text { text: "Hello, world!".to_string() };

        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("text"));
        assert!(json.contains("Hello, world!"));
    }

    #[test]
    fn test_mcp_content_text_deserialization() {
        let json = r#"{"type":"text","text":"Hello, world!"}"#;
        let content: McpContent = serde_json::from_str(json).unwrap();
        match content {
            McpContent::Text { text } => assert_eq!(text, "Hello, world!"),
            _ => panic!("Expected text content"),
        }
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

    #[test]
    fn test_mcp_content_image_deserialization() {
        let json = r#"{"type":"image","data":"base64data","mime_type":"image/png"}"#;
        let content: McpContent = serde_json::from_str(json).unwrap();
        match content {
            McpContent::Image { data, mime_type } => {
                assert_eq!(data, "base64data");
                assert_eq!(mime_type, "image/png");
            }
            _ => panic!("Expected image content"),
        }
    }

    #[test]
    fn test_mcp_content_audio() {
        let content = McpContent::Audio {
            data: "audiodata".to_string(),
            mime_type: "audio/mpeg".to_string(),
        };

        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("audio"));
        assert!(json.contains("audiodata"));
        assert!(json.contains("audio/mpeg"));
    }

    #[test]
    fn test_mcp_content_audio_deserialization() {
        let json = r#"{"type":"audio","data":"audiodata","mime_type":"audio/mpeg"}"#;
        let content: McpContent = serde_json::from_str(json).unwrap();
        match content {
            McpContent::Audio { data, mime_type } => {
                assert_eq!(data, "audiodata");
                assert_eq!(mime_type, "audio/mpeg");
            }
            _ => panic!("Expected audio content"),
        }
    }

    #[test]
    fn test_mcp_prompt() {
        let prompt = McpPrompt {
            name: "test_prompt".to_string(),
            description: Some("A test prompt".to_string()),
            arguments: None,
        };

        assert_eq!(prompt.name, "test_prompt");
        assert!(prompt.description.is_some());
    }

    #[test]
    fn test_mcp_prompt_with_arguments() {
        let prompt = McpPrompt {
            name: "test_prompt".to_string(),
            description: Some("A test prompt".to_string()),
            arguments: Some(vec![McpPromptArgument {
                name: "arg1".to_string(),
                description: Some("First argument".to_string()),
                required: true,
            }]),
        };

        assert!(prompt.arguments.is_some());
        assert_eq!(prompt.arguments.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_mcp_prompt_serialization() {
        let prompt = McpPrompt {
            name: "test_prompt".to_string(),
            description: Some("A test prompt".to_string()),
            arguments: None,
        };

        let json = serde_json::to_string(&prompt).unwrap();
        assert!(json.contains("test_prompt"));
    }

    #[test]
    fn test_mcp_prompt_argument() {
        let arg = McpPromptArgument {
            name: "arg1".to_string(),
            description: Some("First argument".to_string()),
            required: true,
        };

        assert_eq!(arg.name, "arg1");
        assert!(arg.required);
    }

    #[test]
    fn test_mcp_prompt_argument_serialization() {
        let arg = McpPromptArgument {
            name: "arg1".to_string(),
            description: Some("First argument".to_string()),
            required: true,
        };

        let json = serde_json::to_string(&arg).unwrap();
        assert!(json.contains("arg1"));
        assert!(json.contains("required"));
    }

    #[test]
    fn test_mcp_capabilities() {
        let caps = McpCapabilities {
            tools: Some(McpToolsCapability { list_changed: Some(true) }),
            prompts: Some(McpPromptsCapability { list_changed: Some(false) }),
        };

        assert!(caps.tools.is_some());
        assert!(caps.prompts.is_some());
    }

    #[test]
    fn test_mcp_capabilities_serialization() {
        let caps = McpCapabilities {
            tools: Some(McpToolsCapability { list_changed: Some(true) }),
            prompts: None,
        };

        let json = serde_json::to_string(&caps).unwrap();
        assert!(json.contains("tools"));
    }
}

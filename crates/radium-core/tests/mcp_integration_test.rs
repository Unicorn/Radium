//! Integration tests for MCP functionality.

use radium_core::mcp::{
    config::McpConfigManager, content::ContentHandler, McpContent, McpError, McpServerConfig,
    McpTool, McpToolRegistry, TransportType,
};
use serde_json::json;
use std::collections::HashMap;
use tempfile::TempDir;

#[tokio::test]
async fn test_mcp_tool_registry_integration() {
    // Test tool registry with multiple servers
    let mut registry1 = McpToolRegistry::new("server1".to_string());
    let mut registry2 = McpToolRegistry::new("server2".to_string());

    // Register tools from server1
    let tool1 = McpTool {
        name: "query".to_string(),
        description: Some("Query tool from server1".to_string()),
        input_schema: None,
    };
    registry1.register_tool(tool1);

    // Register tools from server2
    let tool2 = McpTool {
        name: "query".to_string(),
        description: Some("Query tool from server2".to_string()),
        input_schema: None,
    };
    registry2.register_tool(tool2);

    // Verify conflict resolution
    assert!(registry1.has_tool("query"));
    assert!(registry2.has_tool("query"));

    // Verify prefixed names work
    assert!(registry1.has_tool("server1:query"));
    assert!(registry2.has_tool("server2:query"));
}

#[tokio::test]
async fn test_mcp_config_manager_integration() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("mcp-servers.toml");
    let mut manager = McpConfigManager::new(config_path.clone());

    // Add multiple servers with different transports
    manager.servers.push(McpServerConfig {
        name: "stdio-server".to_string(),
        transport: TransportType::Stdio,
        command: Some("mcp-server".to_string()),
        args: Some(vec!["--config".to_string(), "config.json".to_string()]),
        url: None,
        auth: None,
    });

    manager.servers.push(McpServerConfig {
        name: "http-server".to_string(),
        transport: TransportType::Http,
        command: None,
        args: None,
        url: Some("https://api.example.com/mcp".to_string()),
        auth: None,
    });

    manager.servers.push(McpServerConfig {
        name: "sse-server".to_string(),
        transport: TransportType::Sse,
        command: None,
        args: None,
        url: Some("http://localhost:8080/sse".to_string()),
        auth: None,
    });

    // Save and reload
    manager.save().unwrap();

    let mut new_manager = McpConfigManager::new(config_path);
    new_manager.load().unwrap();

    assert_eq!(new_manager.get_servers().len(), 3);
    assert!(new_manager.get_server("stdio-server").is_some());
    assert!(new_manager.get_server("http-server").is_some());
    assert!(new_manager.get_server("sse-server").is_some());
}

#[tokio::test]
async fn test_mcp_content_handler_integration() {
    // Test text content serialization
    let text_content = McpContent::Text {
        text: "Hello, world!".to_string(),
    };
    let text_json = ContentHandler::serialize_content(&text_content).unwrap();
    assert_eq!(text_json["type"], "text");
    assert_eq!(text_json["text"], "Hello, world!");

    // Test image content serialization
    let image_content = McpContent::Image {
        data: "base64data".to_string(),
        mime_type: "image/png".to_string(),
    };
    let image_json = ContentHandler::serialize_content(&image_content).unwrap();
    assert_eq!(image_json["type"], "image");
    assert_eq!(image_json["mime_type"], "image/png");

    // Test audio content serialization
    let audio_content = McpContent::Audio {
        data: "audiodata".to_string(),
        mime_type: "audio/wav".to_string(),
    };
    let audio_json = ContentHandler::serialize_content(&audio_content).unwrap();
    assert_eq!(audio_json["type"], "audio");
    assert_eq!(audio_json["mime_type"], "audio/wav");

    // Test content parsing round-trip
    let parsed_text = ContentHandler::parse_content(&text_json).unwrap();
    match parsed_text {
        McpContent::Text { text } => assert_eq!(text, "Hello, world!"),
        _ => panic!("Expected text content"),
    }
}

#[tokio::test]
async fn test_mcp_multi_server_configuration() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("mcp-servers.toml");
    let mut manager = McpConfigManager::new(config_path.clone());

    // Create a complex multi-server configuration
    let mut auth_params = HashMap::new();
    auth_params.insert("client_id".to_string(), "test-client".to_string());
    auth_params.insert("client_secret".to_string(), "test-secret".to_string());

    manager.servers.push(McpServerConfig {
        name: "local-server".to_string(),
        transport: TransportType::Stdio,
        command: Some("local-mcp".to_string()),
        args: Some(vec!["--verbose".to_string()]),
        url: None,
        auth: None,
    });

    manager.servers.push(McpServerConfig {
        name: "remote-server".to_string(),
        transport: TransportType::Http,
        command: None,
        args: None,
        url: Some("https://remote.example.com/mcp".to_string()),
        auth: Some(radium_core::mcp::McpAuthConfig {
            auth_type: "oauth".to_string(),
            params: auth_params,
        }),
    });

    manager.save().unwrap();

    // Reload and verify
    let mut new_manager = McpConfigManager::new(config_path);
    new_manager.load().unwrap();

    let local_server = new_manager.get_server("local-server").unwrap();
    assert_eq!(local_server.transport, TransportType::Stdio);
    assert!(local_server.command.is_some());

    let remote_server = new_manager.get_server("remote-server").unwrap();
    assert_eq!(remote_server.transport, TransportType::Http);
    assert!(remote_server.auth.is_some());
    assert_eq!(
        remote_server.auth.as_ref().unwrap().auth_type,
        "oauth"
    );
}

#[tokio::test]
async fn test_mcp_tool_registry_conflict_resolution() {
    // Simulate a scenario where multiple servers provide tools with the same name
    let mut registry = McpToolRegistry::new("server1".to_string());

    // Register first tool
    let tool1 = McpTool {
        name: "search".to_string(),
        description: Some("Search tool".to_string()),
        input_schema: Some(json!({
            "type": "object",
            "properties": {
                "query": {"type": "string"}
            }
        })),
    };
    registry.register_tool(tool1);

    // Register second tool with same name (should be prefixed)
    let tool2 = McpTool {
        name: "search".to_string(),
        description: Some("Another search tool".to_string()),
        input_schema: Some(json!({
            "type": "object",
            "properties": {
                "term": {"type": "string"}
            }
        })),
    };
    registry.register_tool(tool2);

    // Verify both tools exist
    assert_eq!(registry.get_all_tools().len(), 2);
    assert!(registry.has_tool("search"));
    assert!(registry.has_tool("server1:search"));

    // Verify we can retrieve both
    let tool1_retrieved = registry.get_tool("search");
    assert!(tool1_retrieved.is_some());

    let tool2_retrieved = registry.get_tool("server1:search");
    assert!(tool2_retrieved.is_some());
}

#[tokio::test]
async fn test_mcp_config_round_trip() {
    // Test that configuration can be saved and loaded correctly
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("mcp-servers.toml");
    let mut manager = McpConfigManager::new(config_path.clone());

    // Create a server config with all fields
    let mut auth_params = HashMap::new();
    auth_params.insert("token".to_string(), "test-token".to_string());

    manager.servers.push(McpServerConfig {
        name: "full-config-server".to_string(),
        transport: TransportType::Http,
        command: None,
        args: None,
        url: Some("https://example.com/mcp".to_string()),
        auth: Some(radium_core::mcp::McpAuthConfig {
            auth_type: "bearer".to_string(),
            params: auth_params,
        }),
    });

    // Save
    manager.save().unwrap();

    // Load
    let mut new_manager = McpConfigManager::new(config_path);
    new_manager.load().unwrap();

    // Verify all fields are preserved
    let server = new_manager.get_server("full-config-server").unwrap();
    assert_eq!(server.name, "full-config-server");
    assert_eq!(server.transport, TransportType::Http);
    assert_eq!(server.url, Some("https://example.com/mcp".to_string()));
    assert!(server.auth.is_some());
    assert_eq!(
        server.auth.as_ref().unwrap().auth_type,
        "bearer"
    );
    assert_eq!(
        server.auth.as_ref().unwrap().params.get("token"),
        Some(&"test-token".to_string())
    );
}


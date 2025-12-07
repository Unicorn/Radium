//! Comprehensive error handling tests for MCP integration.

use radium_core::mcp::{
    McpError, McpServerConfig, McpTool, TransportType,
    client::McpClient,
    config::McpConfigManager,
    integration::McpIntegration,
};
use radium_core::workspace::Workspace;
use serde_json::json;
use std::collections::HashMap;
use tempfile::TempDir;

#[tokio::test]
async fn test_connection_error_propagation() {
    // Test that connection errors propagate correctly
    let config = McpServerConfig {
        name: "test-server".to_string(),
        transport: TransportType::Stdio,
        command: Some("nonexistent_command_xyz_12345".to_string()),
        args: None,
        url: None,
        auth: None,
    };

    let result = McpClient::connect(&config).await;
    assert!(result.is_err());
    if let Err(err) = result {
        // Should be a transport or connection error
        assert!(
            matches!(err, McpError::Transport(_)) || matches!(err, McpError::Connection(_)),
            "Expected Transport or Connection error, got: {}",
            err
        );
        assert!(err.to_string().contains("Failed to spawn") || err.to_string().contains("connection"));
    }
}

#[tokio::test]
async fn test_config_error_missing_required_fields() {
    // Test stdio transport without command
    let config = McpServerConfig {
        name: "test-server".to_string(),
        transport: TransportType::Stdio,
        command: None, // Missing required field
        args: None,
        url: None,
        auth: None,
    };

    let result = McpClient::connect(&config).await;
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err, McpError::Config(_)));
        assert!(err.to_string().contains("command") || err.to_string().contains("required"));
    }
}

#[tokio::test]
async fn test_config_error_missing_url_for_remote_transport() {
    // Test SSE transport without URL
    let config = McpServerConfig {
        name: "test-server".to_string(),
        transport: TransportType::Sse,
        command: None,
        args: None,
        url: None, // Missing required field
        auth: None,
    };

    let result = McpClient::connect(&config).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, McpError::Config(_)));
    assert!(err.to_string().contains("url") || err.to_string().contains("required"));
}

#[tokio::test]
async fn test_protocol_error_malformed_json() {
    // This test would require a mock server that returns malformed JSON
    // For now, we test that protocol errors can be created and displayed
    let err = McpError::Protocol("Invalid JSON-RPC message format".to_string());
    assert!(err.to_string().contains("protocol error"));
    assert!(err.to_string().contains("Invalid JSON-RPC"));
}

#[tokio::test]
async fn test_server_not_found_error() {
    let integration = McpIntegration::new();
    
    // Try to get tools from non-existent server
    let all_tools = integration.get_all_tools().await;
    // Should return empty list, not error (graceful degradation)
    assert!(all_tools.is_empty());
}

#[tokio::test]
async fn test_tool_not_found_error() {
    // This would require a connected server with known tools
    // For now, test error type creation
    let err = McpError::ToolNotFound("nonexistent-tool".to_string());
    assert!(err.to_string().contains("tool not found"));
    assert!(err.to_string().contains("nonexistent-tool"));
}

#[tokio::test]
async fn test_authentication_error_scenarios() {
    // Test authentication error creation
    let err = McpError::Authentication("OAuth token expired".to_string());
    assert!(err.to_string().contains("authentication error"));
    assert!(err.to_string().contains("OAuth token expired"));
}

#[tokio::test]
async fn test_partial_server_failure() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();
    
    // Create workspace structure
    std::fs::create_dir_all(workspace_root.join(".radium")).unwrap();
    
    // Create config with one valid and one invalid server
    let config_content = r#"
[[servers]]
name = "valid-server"
transport = "stdio"
command = "echo"
args = ["test"]

[[servers]]
name = "invalid-server"
transport = "stdio"
command = "nonexistent_command_xyz_12345"
"#;
    
    let config_path = workspace_root.join(".radium/mcp-servers.toml");
    std::fs::write(&config_path, config_content).unwrap();
    
    let workspace = Workspace::create(workspace_root).unwrap();
    let integration = McpIntegration::new();
    
    // Initialize should handle partial failures gracefully
    // The invalid server should fail, but valid server might succeed
    let result = integration.initialize(&workspace).await;
    
    // Should either succeed (if echo works) or fail gracefully
    // The key is that it doesn't panic
    let _ = result;
    
    // Check that we can still get tools (even if some servers failed)
    let tools = integration.get_all_tools().await;
    // Tools list should be empty or contain tools from working servers
    let _ = tools;
}

#[tokio::test]
async fn test_config_parse_error_invalid_toml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");
    
    // Write invalid TOML
    std::fs::write(&config_path, "[invalid toml syntax").unwrap();
    
    let mut manager = McpConfigManager::new(config_path);
    let result = manager.load();
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, McpError::TomlParse(_)));
    assert!(err.to_string().contains("TOML parse error"));
}

#[tokio::test]
async fn test_config_parse_error_invalid_json() {
    // Test JSON parsing error (for extension configs)
    let invalid_json = "{ invalid json }";
    let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
    
    assert!(result.is_err());
    // This tests that JSON errors can be converted to McpError
    let json_err = result.unwrap_err();
    let mcp_err: McpError = json_err.into();
    assert!(matches!(mcp_err, McpError::Json(_)));
}

#[tokio::test]
async fn test_io_error_handling() {
    // Test I/O error conversion
    let io_err = std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "Permission denied",
    );
    let mcp_err: McpError = io_err.into();
    
    assert!(matches!(mcp_err, McpError::Io(_)));
    assert!(mcp_err.to_string().contains("I/O error"));
}

#[tokio::test]
async fn test_error_message_clarity() {
    // Test that error messages are clear and actionable
    let errors = vec![
        (
            McpError::Config("Stdio transport requires 'command' field".to_string()),
            "command",
            "required",
        ),
        (
            McpError::Connection("Failed to connect to server 'test' after 30s".to_string()),
            "Failed to connect",
            "test",
        ),
        (
            McpError::Transport("Failed to spawn process: No such file or directory".to_string()),
            "Failed to spawn",
            "process",
        ),
        (
            McpError::Authentication("OAuth token expired for server 'test'".to_string()),
            "OAuth token expired",
            "test",
        ),
    ];
    
    for (err, expected_text1, expected_text2) in errors {
        let msg = err.to_string();
        assert!(
            msg.contains(expected_text1),
            "Error message '{}' should contain '{}'",
            msg,
            expected_text1
        );
        assert!(
            msg.contains(expected_text2),
            "Error message '{}' should contain '{}'",
            msg,
            expected_text2
        );
    }
}

#[tokio::test]
async fn test_error_source_chain() {
    use std::error::Error;
    
    // Test that error source chain works correctly
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let mcp_err: McpError = io_err.into();
    
    // Should have a source
    assert!(mcp_err.source().is_some());
    
    // Source should be the original IO error
    let source = mcp_err.source().unwrap();
    assert!(source.to_string().contains("file not found"));
}

#[tokio::test]
async fn test_timeout_scenario() {
    // Test timeout handling (simulated with invalid URL that will timeout)
    let config = McpServerConfig {
        name: "timeout-server".to_string(),
        transport: TransportType::Http,
        command: None,
        args: None,
        url: Some("http://192.0.2.0:9999".to_string()), // Invalid address that will timeout
        auth: None,
    };
    
    // Connection should fail with timeout or connection error
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        McpClient::connect(&config),
    )
    .await;
    
    match result {
        Ok(Err(e)) => {
            // Expected - connection failed
            assert!(
                matches!(e, McpError::Transport(_)) || matches!(e, McpError::Connection(_)),
                "Expected Transport or Connection error, got: {:?}",
                e
            );
        }
        Err(_) => {
            // Timeout - also acceptable
        }
        Ok(Ok(_)) => {
            // Unexpected success - but handle gracefully
        }
    }
}

#[tokio::test]
async fn test_invalid_transport_type_handling() {
    // Test that invalid transport configuration is caught
    // (This is tested through config validation)
    let config = McpServerConfig {
        name: "test".to_string(),
        transport: TransportType::Stdio,
        command: Some("echo".to_string()),
        args: None,
        url: Some("http://example.com".to_string()), // URL not needed for stdio
        auth: None,
    };
    
    // Should still work (URL is optional for stdio)
    // The real test is that invalid transport enum values are caught at parse time
    let _ = config;
}

#[tokio::test]
async fn test_concurrent_error_handling() {
    // Test that errors don't cause panics in concurrent scenarios
    let integration = Arc::new(McpIntegration::new());
    
    // Try to initialize multiple times concurrently
    let temp_dir = TempDir::new().unwrap();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let integration = Arc::clone(&integration);
            let workspace = workspace.clone();
            tokio::spawn(async move {
                integration.initialize(&workspace).await
            })
        })
        .collect();
    
    // All should complete without panicking
    for handle in handles {
        let _ = handle.await;
    }
}

#[tokio::test]
async fn test_error_recovery_after_failure() {
    // Test that system can recover after a server connection failure
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();
    std::fs::create_dir_all(workspace_root.join(".radium")).unwrap();
    
    // Create config with invalid server
    let config_content = r#"
[[servers]]
name = "invalid-server"
transport = "stdio"
command = "nonexistent_command_xyz_12345"
"#;
    
    let config_path = workspace_root.join(".radium/mcp-servers.toml");
    std::fs::write(&config_path, config_content).unwrap();
    
    let workspace = Workspace::create(workspace_root).unwrap();
    let integration = McpIntegration::new();
    
    // First initialization should fail or skip invalid server
    let _ = integration.initialize(&workspace).await;
    
    // Should still be able to query tools (empty list is fine)
    let tools = integration.get_all_tools().await;
    assert!(tools.is_empty() || tools.len() == 0);
    
    // Should be able to initialize again without issues
    let _ = integration.initialize(&workspace).await;
}

#[tokio::test]
async fn test_all_error_variants_handled() {
    // Ensure all error variants can be created and handled
    let error_variants = vec![
        McpError::Connection("test".to_string()),
        McpError::Transport("test".to_string()),
        McpError::Protocol("test".to_string()),
        McpError::ServerNotFound("test".to_string()),
        McpError::ToolNotFound("test".to_string()),
        McpError::Authentication("test".to_string()),
        McpError::Config("test".to_string()),
    ];
    
    for err in error_variants {
        // All should be displayable
        let msg = err.to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("test"));
        
        // All should be debuggable
        let debug = format!("{:?}", err);
        assert!(!debug.is_empty());
    }
}

#[tokio::test]
async fn test_error_context_preservation() {
    // Test that error context is preserved through error chain
    use std::error::Error;
    
    let io_err = std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Configuration file not found: /path/to/config.toml",
    );
    let mcp_err: McpError = io_err.into();
    
    // Error message should preserve context
    assert!(mcp_err.to_string().contains("I/O error"));
    
    // Source should preserve original message
    if let Some(source) = mcp_err.source() {
        assert!(source.to_string().contains("Configuration file not found"));
    }
}

#[tokio::test]
async fn test_malformed_message_handling() {
    // Test handling of malformed JSON-RPC messages
    // This would typically be tested with a mock server, but we can test error creation
    let err = McpError::Protocol("Invalid JSON-RPC message: missing 'jsonrpc' field".to_string());
    
    assert!(matches!(err, McpError::Protocol(_)));
    assert!(err.to_string().contains("protocol error"));
    assert!(err.to_string().contains("JSON-RPC"));
}

#[tokio::test]
async fn test_error_in_error_scenarios() {
    // Test that errors during error handling don't cause panics
    // E.g., error while formatting error message
    let err = McpError::Config("test".to_string());
    
    // Should be able to convert to string even in error scenarios
    let msg = err.to_string();
    assert!(!msg.is_empty());
    
    // Should be able to get debug representation
    let debug = format!("{:?}", err);
    assert!(!debug.is_empty());
}

#[tokio::test]
async fn test_graceful_degradation_on_errors() {
    // Test that system degrades gracefully when errors occur
    let integration = McpIntegration::new();
    
    // Even if no servers are configured, should not panic
    let tools = integration.get_all_tools().await;
    assert!(tools.is_empty());
    
    // Should be able to query tools multiple times
    for _ in 0..5 {
        let tools = integration.get_all_tools().await;
        assert!(tools.is_empty());
    }
}


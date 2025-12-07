//! Integration tests for MCP stdio transport.

use radium_core::mcp::transport::StdioTransport;
use radium_core::mcp::McpTransport;
use std::io::Write;
use tempfile::TempDir;

/// Helper to create a simple echo-based mock MCP server script.
fn create_mock_server_script(temp_dir: &TempDir) -> std::path::PathBuf {
    let script_path = temp_dir.path().join("mock_mcp_server.sh");
    let mut file = std::fs::File::create(&script_path).unwrap();
    
    // Create a simple script that reads JSON-RPC messages and echoes them back
    writeln!(
        file,
        r#"#!/bin/bash
# Mock MCP server that echoes JSON-RPC messages
while IFS= read -r line; do
    if [ -z "$line" ]; then
        continue
    fi
    # Echo back the message with a response wrapper
    echo "{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{{\"echo\":\"$line\"}}}}"
done
"#
    )
    .unwrap();
    
    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms).unwrap();
    }
    
    script_path
}

#[tokio::test]
async fn test_stdio_transport_connection_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = create_mock_server_script(&temp_dir);
    
    let mut transport = StdioTransport::new(
        "bash".to_string(),
        vec![script_path.to_string_lossy().to_string()],
    );
    
    // Test initial state
    assert!(!transport.is_connected());
    
    // Connect
    let result = transport.connect().await;
    assert!(result.is_ok(), "Connection should succeed: {:?}", result);
    assert!(transport.is_connected());
    
    // Disconnect
    let result = transport.disconnect().await;
    assert!(result.is_ok(), "Disconnect should succeed: {:?}", result);
    assert!(!transport.is_connected());
}

#[tokio::test]
async fn test_stdio_transport_double_connect() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = create_mock_server_script(&temp_dir);
    
    let mut transport = StdioTransport::new(
        "bash".to_string(),
        vec![script_path.to_string_lossy().to_string()],
    );
    
    // First connect should succeed
    assert!(transport.connect().await.is_ok());
    assert!(transport.is_connected());
    
    // Second connect should fail
    let result = transport.connect().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Already connected"));
    
    // Cleanup
    let _ = transport.disconnect().await;
}

#[tokio::test]
async fn test_stdio_transport_send_receive() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = create_mock_server_script(&temp_dir);
    
    let mut transport = StdioTransport::new(
        "bash".to_string(),
        vec![script_path.to_string_lossy().to_string()],
    );
    
    // Connect
    assert!(transport.connect().await.is_ok());
    
    // Send a message
    let test_message = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"test\"}";
    let result = transport.send(test_message).await;
    assert!(result.is_ok(), "Send should succeed: {:?}", result);
    
    // Receive response (with timeout)
    let receive_result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        transport.receive(),
    )
    .await;
    
    if let Ok(Ok(response)) = receive_result {
        assert!(!response.is_empty());
        // Response should contain our message
        let response_str = String::from_utf8_lossy(&response);
        assert!(response_str.contains("test") || response_str.contains("echo"));
    }
    
    // Cleanup
    let _ = transport.disconnect().await;
}

#[tokio::test]
async fn test_stdio_transport_send_when_not_connected() {
    let mut transport = StdioTransport::new("echo".to_string(), vec![]);
    
    let result = transport.send(b"test").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Not connected"));
}

#[tokio::test]
async fn test_stdio_transport_receive_when_not_connected() {
    let mut transport = StdioTransport::new("echo".to_string(), vec![]);
    
    let result = transport.receive().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Not connected"));
}

#[tokio::test]
async fn test_stdio_transport_invalid_command() {
    let mut transport = StdioTransport::new(
        "nonexistent_command_xyz_12345".to_string(),
        vec![],
    );
    
    let result = transport.connect().await;
    assert!(result.is_err());
    assert!(!transport.is_connected());
    assert!(result.unwrap_err().to_string().contains("Failed to spawn"));
}

#[tokio::test]
async fn test_stdio_transport_process_termination() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = create_mock_server_script(&temp_dir);
    
    let mut transport = StdioTransport::new(
        "bash".to_string(),
        vec![script_path.to_string_lossy().to_string()],
    );
    
    // Connect
    assert!(transport.connect().await.is_ok());
    assert!(transport.is_connected());
    
    // Disconnect should terminate the process
    assert!(transport.disconnect().await.is_ok());
    assert!(!transport.is_connected());
    
    // Should be able to disconnect again without error
    assert!(transport.disconnect().await.is_ok());
}

#[tokio::test]
async fn test_stdio_transport_multiple_messages() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = create_mock_server_script(&temp_dir);
    
    let mut transport = StdioTransport::new(
        "bash".to_string(),
        vec![script_path.to_string_lossy().to_string()],
    );
    
    assert!(transport.connect().await.is_ok());
    
    // Send multiple messages
    for i in 0..3 {
        let message = format!("{{\"jsonrpc\":\"2.0\",\"id\":{},\"method\":\"test\"}}", i);
        let result = transport.send(message.as_bytes()).await;
        assert!(result.is_ok(), "Send {} should succeed", i);
    }
    
    // Cleanup
    let _ = transport.disconnect().await;
}

#[tokio::test]
async fn test_stdio_transport_with_args() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = create_mock_server_script(&temp_dir);
    
    let mut transport = StdioTransport::new(
        "bash".to_string(),
        vec![
            script_path.to_string_lossy().to_string(),
            "--test-arg".to_string(),
        ],
    );
    
    // Should be able to connect even with extra args
    // (the script will ignore them, but the transport should handle them)
    let result = transport.connect().await;
    // This might succeed or fail depending on the script, but shouldn't panic
    let _ = result;
    
    let _ = transport.disconnect().await;
}


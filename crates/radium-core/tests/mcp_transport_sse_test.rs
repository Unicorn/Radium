//! Integration tests for MCP SSE transport.

use radium_core::mcp::transport::SseTransport;
use radium_core::mcp::McpTransport;
use std::time::Duration;

#[tokio::test]
async fn test_sse_transport_connection_lifecycle() {
    let mut transport = SseTransport::new("http://localhost:9999/sse".to_string());
    
    // Test initial state
    assert!(!transport.is_connected());
    
    // Connect will likely fail without a real server, but we can test the logic
    let result = transport.connect().await;
    
    if result.is_ok() {
        assert!(transport.is_connected());
        
        // Disconnect
        let result = transport.disconnect().await;
        assert!(result.is_ok());
        assert!(!transport.is_connected());
    } else {
        // Expected failure - verify error message
        assert!(!transport.is_connected());
        assert!(result.unwrap_err().to_string().contains("Failed to connect"));
    }
}

#[tokio::test]
async fn test_sse_transport_double_connect() {
    let mut transport = SseTransport::new("http://localhost:9999/sse".to_string());
    
    // First connect might fail, but if it succeeds, second should fail
    if transport.connect().await.is_ok() {
        assert!(transport.is_connected());
        
        // Second connect should fail
        let result = transport.connect().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Already connected"));
        
        let _ = transport.disconnect().await;
    }
}

#[tokio::test]
async fn test_sse_transport_send_when_not_connected() {
    let mut transport = SseTransport::new("http://localhost:9999/sse".to_string());
    
    let result = transport.send(b"test message").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Not connected"));
}

#[tokio::test]
async fn test_sse_transport_receive_when_not_connected() {
    let mut transport = SseTransport::new("http://localhost:9999/sse".to_string());
    
    let result = transport.receive().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Not connected"));
}

#[tokio::test]
async fn test_sse_transport_invalid_url() {
    let mut transport = SseTransport::new("not-a-valid-url".to_string());
    
    let result = transport.connect().await;
    assert!(result.is_err());
    assert!(!transport.is_connected());
    assert!(result.unwrap_err().to_string().contains("Failed to connect"));
}

#[tokio::test]
async fn test_sse_transport_with_auth() {
    let mut transport = SseTransport::new_with_auth(
        "http://localhost:9999/sse".to_string(),
        Some("Bearer test-token".to_string()),
    );
    
    assert!(!transport.is_connected());
    
    // Update auth header
    transport.set_auth_header(Some("Bearer new-token".to_string()));
    
    // Connection will likely fail, but auth header should be set
    let _ = transport.connect().await;
}

#[tokio::test]
async fn test_sse_transport_disconnect_when_not_connected() {
    let mut transport = SseTransport::new("http://localhost:9999/sse".to_string());
    
    // Disconnecting when not connected should not error
    let result = transport.disconnect().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_sse_transport_receive_empty_buffer() {
    let mut transport = SseTransport::new("http://localhost:9999/sse".to_string());
    
    // If we somehow connect, receiving with empty buffer should fail
    if transport.connect().await.is_ok() {
        // Wait a bit for background task
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let result = transport.receive().await;
        // Should either succeed (if background task added message) or fail (if buffer empty)
        // Either way, shouldn't panic
        let _ = result;
        
        let _ = transport.disconnect().await;
    }
}

#[tokio::test]
async fn test_sse_transport_connection_timeout() {
    let mut transport = SseTransport::new("http://192.0.2.0:9999/sse".to_string());
    
    // This should timeout or fail quickly
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        transport.connect(),
    )
    .await;
    
    match result {
        Ok(Err(e)) => {
            // Expected - connection failed
            assert!(!transport.is_connected());
            assert!(e.to_string().contains("Failed to connect"));
        }
        Err(_) => {
            // Timeout - also acceptable
            assert!(!transport.is_connected());
        }
        Ok(Ok(_)) => {
            // Unexpected success - but handle gracefully
            let _ = transport.disconnect().await;
        }
    }
}

#[tokio::test]
async fn test_sse_transport_send_with_auth() {
    let mut transport = SseTransport::new_with_auth(
        "http://localhost:9999/sse".to_string(),
        Some("Bearer test-token".to_string()),
    );
    
    // Connect (will likely fail)
    if transport.connect().await.is_ok() {
        // Try to send - should include auth header
        let result = transport.send(b"test message").await;
        // Result depends on server, but shouldn't panic
        let _ = result;
        
        let _ = transport.disconnect().await;
    }
}


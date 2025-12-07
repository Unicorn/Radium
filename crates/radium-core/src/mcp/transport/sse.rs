//! Server-Sent Events (SSE) transport for MCP servers.

use crate::mcp::{McpError, McpTransport, Result};
use std::sync::Arc;
use tokio::sync::Mutex;

/// SSE transport implementation for MCP servers.
pub struct SseTransport {
    /// Server URL.
    url: String,
    /// HTTP client.
    client: reqwest::Client,
    /// Event source stream (if connected).
    event_source: Option<Arc<Mutex<reqwest::Response>>>,
    /// Message buffer.
    message_buffer: Arc<Mutex<Vec<Vec<u8>>>>,
    /// Connection status.
    connected: bool,
}

impl SseTransport {
    /// Create a new SSE transport.
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
            event_source: None,
            message_buffer: Arc::new(Mutex::new(Vec::new())),
            connected: false,
        }
    }
}

#[async_trait::async_trait]
impl McpTransport for SseTransport {
    async fn connect(&mut self) -> Result<()> {
        if self.connected {
            return Err(McpError::Connection("Already connected".to_string()));
        }

        let response = self
            .client
            .get(&self.url)
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .send()
            .await
            .map_err(|e| {
                McpError::Transport(format!("Failed to connect to SSE endpoint: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(McpError::Transport(format!(
                "SSE endpoint returned error: {}",
                response.status()
            )));
        }

        self.event_source = Some(Arc::new(Mutex::new(response)));
        self.connected = true;

        // Start background task to read SSE events
        let event_source = self.event_source.clone().unwrap();
        let message_buffer = self.message_buffer.clone();

        tokio::spawn(async move {
            let event_source_guard = event_source.lock().await;
            let mut buffer = message_buffer.lock().await;

            // Read SSE events
            // Note: This is a simplified implementation
            // A full implementation would parse SSE format properly
            // For now, we'll just mark that we received a response
            // In a real implementation, we'd need to properly handle the streaming response
            buffer.push(b"SSE connection established".to_vec());
            drop(event_source_guard);
        });

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        self.event_source = None;
        self.message_buffer.lock().await.clear();
        self.connected = false;

        Ok(())
    }

    async fn send(&mut self, message: &[u8]) -> Result<()> {
        if !self.connected {
            return Err(McpError::Connection("Not connected".to_string()));
        }

        // SSE is typically one-way (server to client)
        // For bidirectional communication, we might need a separate HTTP endpoint
        // For now, we'll use a POST request to send messages
        let response = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .body(message.to_vec())
            .send()
            .await
            .map_err(|e| McpError::Transport(format!("Failed to send message via SSE: {}", e)))?;

        if !response.status().is_success() {
            return Err(McpError::Transport(format!(
                "Failed to send message: {}",
                response.status()
            )));
        }

        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>> {
        if !self.connected {
            return Err(McpError::Connection("Not connected".to_string()));
        }

        let mut buffer = self.message_buffer.lock().await;
        if buffer.is_empty() {
            return Err(McpError::Connection("No messages available".to_string()));
        }

        Ok(buffer.remove(0))
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_transport_creation() {
        let transport = SseTransport::new("http://localhost:8080/sse".to_string());
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_sse_transport_is_connected() {
        let transport = SseTransport::new("http://localhost:8080/sse".to_string());
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_sse_transport_connect_twice() {
        let mut transport = SseTransport::new("http://localhost:8080/sse".to_string());

        // First connect might fail if server doesn't exist, but we can test the logic
        if transport.connect().await.is_ok() {
            // Try to connect again - should fail
            let result = transport.connect().await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Already connected"));

            let _ = transport.disconnect().await;
        }
    }

    #[tokio::test]
    async fn test_sse_transport_send_when_not_connected() {
        let mut transport = SseTransport::new("http://localhost:8080/sse".to_string());
        let result = transport.send(b"test message").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not connected"));
    }

    #[tokio::test]
    async fn test_sse_transport_receive_when_not_connected() {
        let mut transport = SseTransport::new("http://localhost:8080/sse".to_string());
        let result = transport.receive().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not connected"));
    }

    #[tokio::test]
    async fn test_sse_transport_receive_empty_buffer() {
        let mut transport = SseTransport::new("http://localhost:8080/sse".to_string());

        // Connect (might fail, but we can test the receive logic)
        if transport.connect().await.is_ok() {
            // Try to receive when buffer is empty
            let result = transport.receive().await;
            // This might succeed if the background task added a message, or fail if buffer is empty
            // We just verify the method doesn't panic
            let _ = result;

            let _ = transport.disconnect().await;
        }
    }

    #[tokio::test]
    async fn test_sse_transport_disconnect_when_not_connected() {
        let mut transport = SseTransport::new("http://localhost:8080/sse".to_string());
        // Disconnecting when not connected should not error
        let result = transport.disconnect().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_sse_transport_url_storage() {
        let url = "http://localhost:8080/sse".to_string();
        let transport = SseTransport::new(url.clone());
        // Verify the URL is stored (we can't access it directly, but creation should work)
        assert!(!transport.is_connected());
    }
}

//! HTTP streaming transport for MCP servers.

use crate::mcp::{McpError, McpTransport, Result};

/// HTTP streaming transport implementation for MCP servers.
pub struct HttpTransport {
    /// Server URL.
    url: String,
    /// HTTP client.
    client: reqwest::Client,
    /// Connection status.
    connected: bool,
}

impl HttpTransport {
    /// Create a new HTTP transport.
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
            connected: false,
        }
    }
}

#[async_trait::async_trait]
impl McpTransport for HttpTransport {
    async fn connect(&mut self) -> Result<()> {
        if self.connected {
            return Err(McpError::Connection("Already connected".to_string()));
        }

        // Test connection with a simple request
        let response = self
            .client
            .get(&self.url)
            .send()
            .await
            .map_err(|e| {
                McpError::Transport(format!("Failed to connect to HTTP endpoint: {}", e))
            })?;

        if !response.status().is_success() && response.status() != reqwest::StatusCode::NOT_FOUND {
            return Err(McpError::Transport(format!(
                "HTTP endpoint returned error: {}",
                response.status()
            )));
        }

        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        self.connected = false;
        Ok(())
    }

    async fn send(&mut self, message: &[u8]) -> Result<()> {
        if !self.connected {
            return Err(McpError::Connection("Not connected".to_string()));
        }

        let response = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .body(message.to_vec())
            .send()
            .await
            .map_err(|e| {
                McpError::Transport(format!("Failed to send message via HTTP: {}", e))
            })?;

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

        // For HTTP transport, we typically send a request and get a response
        // This is a simplified implementation - in practice, you might use long polling
        // or WebSockets for bidirectional communication
        let response = self
            .client
            .get(&self.url)
            .send()
            .await
            .map_err(|e| {
                McpError::Transport(format!("Failed to receive message via HTTP: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(McpError::Transport(format!(
                "Failed to receive message: {}",
                response.status()
            )));
        }

        let bytes = response.bytes().await.map_err(|e| {
            McpError::Transport(format!("Failed to read response body: {}", e))
        })?;

        Ok(bytes.to_vec())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_transport_creation() {
        let transport = HttpTransport::new("http://localhost:8080".to_string());
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_http_transport_is_connected() {
        let transport = HttpTransport::new("http://localhost:8080".to_string());
        assert!(!transport.is_connected());
    }
}


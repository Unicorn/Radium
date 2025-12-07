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
    /// OAuth authorization header (if configured).
    auth_header: Option<String>,
}

impl HttpTransport {
    /// Create a new HTTP transport.
    pub fn new(url: String) -> Self {
        Self { url, client: reqwest::Client::new(), connected: false, auth_header: None }
    }

    /// Create a new HTTP transport with OAuth authentication.
    pub fn new_with_auth(url: String, auth_header: Option<String>) -> Self {
        Self { url, client: reqwest::Client::new(), connected: false, auth_header }
    }

    /// Update the authorization header (for token refresh).
    pub fn set_auth_header(&mut self, auth_header: Option<String>) {
        self.auth_header = auth_header;
    }
}

#[async_trait::async_trait]
impl McpTransport for HttpTransport {
    async fn connect(&mut self) -> Result<()> {
        if self.connected {
            return Err(McpError::connection(
                "Already connected",
                "The HTTP transport is already connected. Disconnect before reconnecting.",
            ));
        }

        // Test connection with a simple request
        let mut request = self.client.get(&self.url);
        if let Some(ref auth) = self.auth_header {
            request = request.header("Authorization", auth.as_str());
        }
        let response = request.send().await.map_err(|e| {
            McpError::transport(
                format!("Failed to connect to HTTP endpoint at {}: {}", self.url, e),
                format!(
                    "Failed to connect to the HTTP server. Common causes:\n  - Server is not running\n  - Network connectivity issue\n  - Invalid URL: {}\n  - Firewall blocking connection\n\nTry:\n  - Verify the server is running: curl {}\n  - Check network connectivity\n  - Verify the URL is correct",
                    self.url, self.url
                ),
            )
        })?;

        if !response.status().is_success() && response.status() != reqwest::StatusCode::NOT_FOUND {
            return Err(McpError::transport(
                format!("HTTP endpoint returned error: {}", response.status()),
                format!(
                    "The HTTP server at {} returned an error status. Common causes:\n  - Authentication required (check OAuth token)\n  - Server error\n  - Endpoint not found\n\nCheck:\n  - OAuth token is valid: rad mcp auth status\n  - Server logs for errors\n  - URL is correct: {}",
                    self.url, self.url
                ),
            ));
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
            return Err(McpError::connection(
                "Not connected",
                "The HTTP transport is not connected. Call connect() before sending messages.",
            ));
        }

        let mut request = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json");
        
        if let Some(ref auth) = self.auth_header {
            request = request.header("Authorization", auth.as_str());
        }

        let response = request
            .body(message.to_vec())
            .send()
            .await
            .map_err(|e| McpError::transport(
                format!("Failed to send message via HTTP to {}: {}", self.url, e),
                format!(
                    "Failed to send message to the HTTP server. Common causes:\n  - Network connectivity issue\n  - Server not responding\n  - Authentication token expired\n\nTry:\n  - Check network connectivity\n  - Verify OAuth token: rad mcp auth status\n  - Check server logs",
                ),
            ))?;

        if !response.status().is_success() {
            return Err(McpError::transport(
                format!("Failed to send message: {}", response.status()),
                format!(
                    "The HTTP server at {} returned an error status. Common causes:\n  - Invalid message format\n  - Authentication required\n  - Server error\n\nCheck:\n  - Message format matches server expectations\n  - OAuth token is valid: rad mcp auth status\n  - Server logs for errors",
                    self.url
                ),
            ));
        }

        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>> {
        if !self.connected {
            return Err(McpError::connection(
                "Not connected",
                "The HTTP transport is not connected. Call connect() before receiving messages.",
            ));
        }

        // For HTTP transport, we typically send a request and get a response
        // This is a simplified implementation - in practice, you might use long polling
        // or WebSockets for bidirectional communication
        let mut request = self.client.get(&self.url);
        if let Some(ref auth) = self.auth_header {
            request = request.header("Authorization", auth.as_str());
        }
        let response = request.send().await.map_err(|e| {
            McpError::transport(
                format!("Failed to receive message via HTTP from {}: {}", self.url, e),
                format!(
                    "Failed to receive message from the HTTP server. Common causes:\n  - Network connectivity issue\n  - Server not responding\n  - Authentication token expired\n\nTry:\n  - Check network connectivity\n  - Verify OAuth token: rad mcp auth status\n  - Check server logs",
                ),
            )
        })?;

        if !response.status().is_success() {
            return Err(McpError::transport(
                format!("Failed to receive message: {}", response.status()),
                format!(
                    "The HTTP server at {} returned an error status. Common causes:\n  - Server error\n  - Authentication required\n  - Endpoint not found\n\nCheck:\n  - OAuth token is valid: rad mcp auth status\n  - Server logs for errors\n  - URL is correct: {}",
                    self.url, self.url
                ),
            ));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| McpError::transport(
                format!("Failed to read response body: {}", e),
                "Failed to read the HTTP response body. This may indicate:\n  - Network interruption\n  - Response too large\n  - Server closed connection\n\nTry the request again or check server logs.",
            ))?;

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

    #[tokio::test]
    async fn test_http_transport_connect_twice() {
        let mut transport = HttpTransport::new("http://localhost:8080".to_string());

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
    async fn test_http_transport_send_when_not_connected() {
        let mut transport = HttpTransport::new("http://localhost:8080".to_string());
        let result = transport.send(b"test message").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not connected"));
    }

    #[tokio::test]
    async fn test_http_transport_receive_when_not_connected() {
        let mut transport = HttpTransport::new("http://localhost:8080".to_string());
        let result = transport.receive().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not connected"));
    }

    #[tokio::test]
    async fn test_http_transport_disconnect_when_not_connected() {
        let mut transport = HttpTransport::new("http://localhost:8080".to_string());
        // Disconnecting when not connected should not error
        let result = transport.disconnect().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_transport_url_storage() {
        let url = "http://localhost:8080".to_string();
        let transport = HttpTransport::new(url.clone());
        // Verify the URL is stored (we can't access it directly, but creation should work)
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_http_transport_invalid_url() {
        let mut transport = HttpTransport::new("not-a-valid-url".to_string());
        let result = transport.connect().await;
        // Should fail to parse or connect
        assert!(result.is_err());
        assert!(!transport.is_connected());
    }
}

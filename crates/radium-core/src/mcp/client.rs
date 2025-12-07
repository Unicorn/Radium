//! MCP client implementation.

use crate::mcp::messages::{InitializeParams, InitializeResult, JsonRpcRequest, JsonRpcResponse};
use crate::mcp::transport::{HttpTransport, SseTransport, StdioTransport};
use crate::mcp::McpTransport;
use crate::mcp::{McpError, McpServerConfig, McpServerInfo, Result, TransportType};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

/// MCP client for communicating with MCP servers.
pub struct McpClient {
    /// Transport implementation.
    transport: Arc<Mutex<Box<dyn McpTransport>>>,
    /// Server information.
    server_info: McpServerInfo,
    /// Request ID counter.
    request_id: Arc<Mutex<u64>>,
}

impl McpClient {
    /// Create a new MCP client and connect to the server.
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be created or connected.
    pub async fn connect(server_config: &McpServerConfig) -> Result<Self> {
        // Create transport based on configuration
        let transport: Box<dyn McpTransport> = match server_config.transport {
            TransportType::Stdio => {
                let command = server_config.command.clone().ok_or_else(|| {
                    McpError::Config("Stdio transport requires 'command' field".to_string())
                })?;
                let args = server_config.args.clone().unwrap_or_default();
                let mut stdio_transport = StdioTransport::new(command, args);
                stdio_transport.connect().await?;
                Box::new(stdio_transport)
            }
            TransportType::Sse => {
                let url = server_config.url.clone().ok_or_else(|| {
                    McpError::Config("SSE transport requires 'url' field".to_string())
                })?;
                let mut sse_transport = SseTransport::new(url);
                sse_transport.connect().await?;
                Box::new(sse_transport)
            }
            TransportType::Http => {
                let url = server_config.url.clone().ok_or_else(|| {
                    McpError::Config("HTTP transport requires 'url' field".to_string())
                })?;
                let mut http_transport = HttpTransport::new(url);
                http_transport.connect().await?;
                Box::new(http_transport)
            }
        };

        let transport = Arc::new(Mutex::new(transport));

        // Initialize the MCP connection
        let init_result = Self::initialize_connection(&transport).await?;

        let server_info = McpServerInfo {
            name: init_result
                .server_info
                .as_ref()
                .map(|info| info.name.clone())
                .unwrap_or_else(|| server_config.name.clone()),
            version: init_result
                .server_info
                .as_ref()
                .and_then(|info| info.version.clone()),
            capabilities: Some(crate::mcp::McpCapabilities {
                tools: None, // Will be populated when tools are discovered
                prompts: None, // Will be populated when prompts are discovered
            }),
        };

        Ok(Self {
            transport,
            server_info,
            request_id: Arc::new(Mutex::new(0)),
        })
    }

    /// Initialize the MCP connection with the server.
    async fn initialize_connection(
        transport: &Arc<Mutex<Box<dyn McpTransport>>>,
    ) -> Result<InitializeResult> {
        let init_params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: crate::mcp::messages::ClientCapabilities {
                experimental: None,
            },
            client_info: Some(crate::mcp::messages::ClientInfo {
                name: "radium".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        };

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: Some(serde_json::to_value(init_params)?),
            id: Some(json!(1)),
        };

        let request_bytes = serde_json::to_vec(&request)?;

        let mut transport = transport.lock().await;
        transport.send(&request_bytes).await?;

        // Receive response
        let response_bytes = transport.receive().await?;
        let response: JsonRpcResponse = serde_json::from_slice(&response_bytes)?;

        if let Some(error) = response.error {
            return Err(McpError::Protocol(format!(
                "Initialize failed: {} (code: {})",
                error.message, error.code
            )));
        }

        let result = response.result.ok_or_else(|| {
            McpError::Protocol("Initialize response missing result".to_string())
        })?;

        let init_result: InitializeResult = serde_json::from_value(result)?;

        // Send initialized notification
        let initialized = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        });
        transport.send(&serde_json::to_vec(&initialized)?).await?;

        Ok(init_result)
    }

    /// Send a request and wait for a response.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn send_request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let mut request_id = self.request_id.lock().await;
        *request_id += 1;
        let id = *request_id;
        drop(request_id);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: Some(json!(id)),
        };

        let request_bytes = serde_json::to_vec(&request)?;

        let mut transport = self.transport.lock().await;
        transport.send(&request_bytes).await?;

        // Receive response
        let response_bytes = transport.receive().await?;
        let response: JsonRpcResponse = serde_json::from_slice(&response_bytes)?;

        if let Some(error) = response.error {
            return Err(McpError::Protocol(format!(
                "Request failed: {} (code: {})",
                error.message, error.code
            )));
        }

        response.result.ok_or_else(|| {
            McpError::Protocol("Response missing result".to_string())
        })
    }

    /// Get server information.
    pub fn server_info(&self) -> &McpServerInfo {
        &self.server_info
    }

    /// Check if the client is connected.
    pub fn is_connected(&self) -> bool {
        // Note: This is a simplified check
        // In a full implementation, we'd check the transport state
        true
    }

    /// Disconnect from the server.
    ///
    /// # Errors
    ///
    /// Returns an error if disconnection fails.
    pub async fn disconnect(&mut self) -> Result<()> {
        let mut transport = self.transport.lock().await;
        transport.disconnect().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::{McpError, McpTransport, Result};

    /// Mock transport implementation for testing.
    struct MockTransport {
        connected: bool,
        sent_messages: Vec<Vec<u8>>,
        receive_queue: Vec<Vec<u8>>,
        should_fail_connect: bool,
        should_fail_send: bool,
        should_fail_receive: bool,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                connected: false,
                sent_messages: Vec::new(),
                receive_queue: Vec::new(),
                should_fail_connect: false,
                should_fail_send: false,
                should_fail_receive: false,
            }
        }

        fn with_receive_response(mut self, response: Vec<u8>) -> Self {
            self.receive_queue.push(response);
            self
        }

        fn with_fail_connect(mut self) -> Self {
            self.should_fail_connect = true;
            self
        }

        fn with_fail_send(mut self) -> Self {
            self.should_fail_send = true;
            self
        }

        fn with_fail_receive(mut self) -> Self {
            self.should_fail_receive = true;
            self
        }

        fn get_sent_messages(&self) -> &[Vec<u8>] {
            &self.sent_messages
        }
    }

    #[async_trait::async_trait]
    impl McpTransport for MockTransport {
        async fn connect(&mut self) -> Result<()> {
            if self.should_fail_connect {
                return Err(McpError::Connection("Mock connection failure".to_string()));
            }
            self.connected = true;
            Ok(())
        }

        async fn disconnect(&mut self) -> Result<()> {
            self.connected = false;
            Ok(())
        }

        async fn send(&mut self, message: &[u8]) -> Result<()> {
            if !self.connected {
                return Err(McpError::Connection("Not connected".to_string()));
            }
            if self.should_fail_send {
                return Err(McpError::Transport("Mock send failure".to_string()));
            }
            self.sent_messages.push(message.to_vec());
            Ok(())
        }

        async fn receive(&mut self) -> Result<Vec<u8>> {
            if !self.connected {
                return Err(McpError::Connection("Not connected".to_string()));
            }
            if self.should_fail_receive {
                return Err(McpError::Transport("Mock receive failure".to_string()));
            }
            if self.receive_queue.is_empty() {
                return Err(McpError::Connection("No messages available".to_string()));
            }
            Ok(self.receive_queue.remove(0))
        }

        fn is_connected(&self) -> bool {
            self.connected
        }
    }

    #[test]
    fn test_mcp_client_creation() {
        // Test that we can create a config (client creation requires actual connection)
        let config = McpServerConfig {
            name: "test-server".to_string(),
            transport: TransportType::Stdio,
            command: Some("echo".to_string()),
            args: Some(vec![]),
            url: None,
            auth: None,
        };

        assert_eq!(config.name, "test-server");
        assert_eq!(config.transport, TransportType::Stdio);
    }

    #[tokio::test]
    async fn test_mock_transport_connect() {
        let mut transport = MockTransport::new();
        assert!(!transport.is_connected());
        
        transport.connect().await.unwrap();
        assert!(transport.is_connected());
    }

    #[tokio::test]
    async fn test_mock_transport_send_receive() {
        let mut transport = MockTransport::new();
        transport.connect().await.unwrap();

        let message = b"test message";
        transport.send(message).await.unwrap();
        
        assert_eq!(transport.get_sent_messages().len(), 1);
        assert_eq!(transport.get_sent_messages()[0], message);
    }

    #[tokio::test]
    async fn test_mock_transport_receive_queue() {
        let mut transport = MockTransport::new()
            .with_receive_response(b"response1".to_vec())
            .with_receive_response(b"response2".to_vec());
        
        transport.connect().await.unwrap();

        let msg1 = transport.receive().await.unwrap();
        assert_eq!(msg1, b"response1");

        let msg2 = transport.receive().await.unwrap();
        assert_eq!(msg2, b"response2");
    }

    #[tokio::test]
    async fn test_mock_transport_disconnect() {
        let mut transport = MockTransport::new();
        transport.connect().await.unwrap();
        assert!(transport.is_connected());

        transport.disconnect().await.unwrap();
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_mock_transport_error_handling() {
        let mut transport = MockTransport::new().with_fail_connect();
        let result = transport.connect().await;
        assert!(result.is_err());
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_mock_transport_send_when_not_connected() {
        let mut transport = MockTransport::new();
        let result = transport.send(b"test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_transport_receive_when_not_connected() {
        let mut transport = MockTransport::new();
        let result = transport.receive().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_transport_receive_empty_queue() {
        let mut transport = MockTransport::new();
        transport.connect().await.unwrap();
        
        let result = transport.receive().await;
        assert!(result.is_err());
    }
}


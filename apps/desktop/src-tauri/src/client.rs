//! gRPC client for connecting to Radium server

use radium_core::radium_client::RadiumClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::{Channel, Endpoint};
use tracing::info;

/// Default server address
const DEFAULT_SERVER_ADDRESS: &str = "http://127.0.0.1:50051";

/// gRPC client manager for Radium server connections
pub struct ClientManager {
    /// Cached client connection
    client: Arc<Mutex<Option<RadiumClient<Channel>>>>,
    /// Server address
    server_address: String,
}

impl ClientManager {
    /// Create a new client manager with default server address
    pub fn new() -> Self {
        Self::with_address(DEFAULT_SERVER_ADDRESS.to_string())
    }

    /// Create a new client manager with a custom server address
    pub fn with_address(server_address: String) -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            server_address,
        }
    }

    /// Get or create a connected client
    ///
    /// This will reuse an existing connection if available, or create a new one.
    pub async fn get_client(&self) -> Result<RadiumClient<Channel>, String> {
        // Check if we have a cached client
        let mut client_guard = self.client.lock().await;
        
        if let Some(ref client) = *client_guard {
            // Try to use existing client - in a real implementation, we might want to
            // check if the connection is still alive, but for simplicity we'll just
            // create a new one if needed
            return Ok(client.clone());
        }

        // Create new connection
        info!(address = %self.server_address, "Connecting to Radium server");
        
        let endpoint = Endpoint::from_shared(self.server_address.clone())
            .map_err(|e| format!("Invalid server address: {}", e))?;
        
        let channel = endpoint
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to server: {}", e))?;
        
        let client = RadiumClient::new(channel);
        info!("Connected to Radium server");
        
        // Cache the client
        *client_guard = Some(client.clone());
        
        Ok(client)
    }

    /// Clear the cached client connection
    pub async fn disconnect(&self) {
        let mut client_guard = self.client.lock().await;
        *client_guard = None;
        info!("Disconnected from Radium server");
    }

    /// Get the server address
    pub fn server_address(&self) -> &str {
        &self.server_address
    }
}

impl Default for ClientManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_manager_new() {
        let manager = ClientManager::new();
        assert_eq!(manager.server_address(), DEFAULT_SERVER_ADDRESS);
    }

    #[test]
    fn test_client_manager_with_address() {
        let custom_address = "http://127.0.0.1:9999".to_string();
        let manager = ClientManager::with_address(custom_address.clone());
        assert_eq!(manager.server_address(), custom_address);
    }

    #[test]
    fn test_client_manager_default() {
        let manager = ClientManager::default();
        assert_eq!(manager.server_address(), DEFAULT_SERVER_ADDRESS);
    }

    #[tokio::test]
    async fn test_client_manager_invalid_address() {
        let manager = ClientManager::with_address("invalid://address".to_string());
        let result = manager.get_client().await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        // The error might be from Endpoint::from_shared or from connect
        // Both are acceptable - the important thing is that it fails
        assert!(
            error_msg.contains("Invalid server address") || 
            error_msg.contains("Failed to connect") ||
            error_msg.contains("invalid"),
            "Error message should indicate connection failure, got: {}",
            error_msg
        );
    }

    #[tokio::test]
    async fn test_client_manager_connection_failure() {
        // Use a port that's unlikely to have a server running
        let manager = ClientManager::with_address("http://127.0.0.1:65535".to_string());
        let result = manager.get_client().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to connect"));
    }

    #[tokio::test]
    async fn test_client_manager_disconnect() {
        let manager = ClientManager::new();
        // Disconnect should not panic even if no connection exists
        manager.disconnect().await;
        
        // After disconnect, get_client should try to reconnect
        // (This will fail without a server, but that's expected)
        let result = manager.get_client().await;
        assert!(result.is_err());
    }
}

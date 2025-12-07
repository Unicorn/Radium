//! Client helper for managing embedded server and gRPC client connections.
//!
//! This module provides utilities for automatically managing server lifecycle
//! when creating gRPC client connections, making it easy for CLI/TUI applications
//! to use the server without manual management.

use std::net::SocketAddr;
use std::time::Duration;

use tonic::transport::{Channel, Endpoint};
use tracing::info;

use crate::config::Config;
use crate::error::{RadiumError, Result};
use crate::proto::radium_client::RadiumClient;
#[cfg(feature = "orchestrator-integration")]
use crate::server::manager::EmbeddedServer;

/// Default timeout for server readiness check
const DEFAULT_READINESS_TIMEOUT: Duration = Duration::from_secs(10);

/// Helper for managing embedded server and gRPC client connections.
///
/// This helper automatically starts an embedded server if needed and provides
/// a convenient interface for creating gRPC clients.
///
/// The embedded server is managed per-instance and will be automatically
/// cleaned up when the helper is dropped (for CLI commands) or can be
/// explicitly shut down (for long-running apps).
pub struct ClientHelper {
    /// Server configuration
    config: Config,
    /// Whether to use embedded server (can be disabled via env var)
    use_embedded: bool,
    /// Embedded server instance (if using embedded mode)
    #[cfg(feature = "orchestrator-integration")]
    embedded_server: Option<EmbeddedServer>,
}

impl ClientHelper {
    /// Create a new client helper with default configuration.
    ///
    /// # Returns
    /// A new `ClientHelper` instance.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(Config::default())
    }

    /// Create a new client helper with custom configuration.
    ///
    /// # Arguments
    /// * `config` - Server configuration
    ///
    /// # Returns
    /// A new `ClientHelper` instance.
    #[must_use]
    pub fn with_config(config: Config) -> Self {
        // Check if embedded server is disabled via environment variable
        let use_embedded = !std::env::var("RADIUM_DISABLE_EMBEDDED_SERVER")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        Self { config, use_embedded, embedded_server: None }
    }

    /// Ensure the server is running and return a connected client.
    ///
    /// This will:
    /// 1. Check if an external server is already running (if embedded is disabled)
    /// 2. Start an embedded server if needed (if enabled)
    /// 3. Wait for server to be ready
    /// 4. Create and return a gRPC client connection
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Server fails to start
    /// - Server doesn't become ready within timeout
    /// - Client connection fails
    pub async fn connect(&mut self) -> Result<RadiumClient<Channel>> {
        if !self.use_embedded {
            // Use external server - just connect
            return self.connect_to_external().await;
        }

        // Initialize embedded server if not already done
        if self.embedded_server.is_none() {
            let mut server = EmbeddedServer::new(self.config.clone());
            server.start().await?;
            server.wait_for_ready(DEFAULT_READINESS_TIMEOUT).await?;
            self.embedded_server = Some(server);
        }

        // Get server address
        let address = self.embedded_server.as_ref().unwrap().address();

        // Create client connection
        self.create_client(address).await
    }

    /// Connect to an external server (embedded server disabled).
    ///
    /// # Errors
    ///
    /// Returns an error if connection fails.
    async fn connect_to_external(&self) -> Result<RadiumClient<Channel>> {
        let address = self.config.server.address;
        let server_url = format!("http://{}", address);

        info!(
            address = %server_url,
            "Connecting to external Radium server"
        );

        self.create_client(address).await
    }

    /// Create a gRPC client connection to the specified address.
    ///
    /// # Arguments
    /// * `address` - Server socket address
    ///
    /// # Errors
    ///
    /// Returns an error if connection fails.
    async fn create_client(&self, address: SocketAddr) -> Result<RadiumClient<Channel>> {
        let server_url = format!("http://{}", address);

        let endpoint = Endpoint::from_shared(server_url.clone())
            .map_err(|e| RadiumError::Config(format!("Invalid server address: {}", e)))?;

        let channel = endpoint.connect().await.map_err(|e| RadiumError::Server(e))?;

        info!(address = %server_url, "Connected to Radium server");

        Ok(RadiumClient::new(channel))
    }

    /// Get the server address being used.
    ///
    /// # Returns
    /// The socket address of the server.
    #[must_use]
    pub fn server_address(&self) -> SocketAddr {
        self.config.server.address
    }

    /// Check if embedded server is enabled.
    ///
    /// # Returns
    /// `true` if embedded server will be used, `false` if external server is expected.
    #[must_use]
    pub fn is_embedded_enabled(&self) -> bool {
        self.use_embedded
    }

    /// Shutdown the embedded server (if running).
    ///
    /// This is useful for cleanup in CLI/TUI applications.
    /// The server will also be automatically cleaned up when the helper is dropped.
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails.
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(mut server) = self.embedded_server.take() {
            server.shutdown().await?;
        }
        Ok(())
    }
}

impl Default for ClientHelper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_helper_new() {
        let helper = ClientHelper::new();
        assert_eq!(helper.server_address().port(), 50051);
    }

    #[test]
    fn test_client_helper_with_config() {
        let mut config = Config::default();
        config.server.address = "127.0.0.1:8080".parse().unwrap();
        let helper = ClientHelper::with_config(config);
        assert_eq!(helper.server_address().port(), 8080);
    }

    #[test]
    #[allow(unsafe_code)] // Test-only: setting env vars for isolated test
    fn test_client_helper_embedded_disabled() {
        unsafe {
            std::env::set_var("RADIUM_DISABLE_EMBEDDED_SERVER", "true");
        }
        let helper = ClientHelper::new();
        assert!(!helper.is_embedded_enabled());
        unsafe {
            std::env::remove_var("RADIUM_DISABLE_EMBEDDED_SERVER");
        }
    }

    #[tokio::test]
    #[allow(unsafe_code)] // Test-only: setting env vars for isolated test
    async fn test_client_helper_connect_with_embedded() {
        // Find an available port
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let mut config = Config::default();
        config.server.address = format!("127.0.0.1:{}", port).parse().unwrap();
        config.server.enable_grpc_web = false;

        // Ensure embedded is enabled
        unsafe {
            std::env::remove_var("RADIUM_DISABLE_EMBEDDED_SERVER");
        }

        let mut helper = ClientHelper::with_config(config);
        let client = helper.connect().await.unwrap();
        // Client should be connected
        assert!(std::mem::size_of_val(&client) > 0);

        // Cleanup
        helper.shutdown().await.unwrap();
    }

    #[tokio::test]
    #[allow(unsafe_code)] // Test-only: setting env vars for isolated test
    async fn test_client_helper_shutdown() {
        // Find an available port
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let mut config = Config::default();
        config.server.address = format!("127.0.0.1:{}", port).parse().unwrap();
        config.server.enable_grpc_web = false;

        unsafe {
            std::env::remove_var("RADIUM_DISABLE_EMBEDDED_SERVER");
        }

        let mut helper = ClientHelper::with_config(config);
        let _client = helper.connect().await.unwrap();

        // Shutdown should work
        helper.shutdown().await.unwrap();
    }
}

//! Embedded server manager for automatic server lifecycle management.
//!
//! This module provides utilities for embedding the Radium server within client applications,
//! automatically starting and stopping the server as needed.

use std::net::SocketAddr;
use std::time::Duration;

use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::{interval, sleep};
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::error::{RadiumError, Result};
use crate::proto::{PingRequest, radium_client::RadiumClient};
use crate::server;

/// Manages an embedded Radium server running in a background task.
///
/// The server runs in a separate tokio task and can be started, checked for readiness,
/// and gracefully shut down.
pub struct EmbeddedServer {
    /// Server configuration
    config: Config,
    /// Shutdown signal sender
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// Server task handle
    server_handle: Option<JoinHandle<Result<()>>>,
    /// Server address (may differ from config if port was auto-assigned)
    address: SocketAddr,
}

impl EmbeddedServer {
    /// Create a new embedded server with the given configuration.
    ///
    /// # Arguments
    /// * `config` - Server configuration
    ///
    /// # Returns
    /// A new `EmbeddedServer` instance (not started yet).
    #[must_use]
    pub fn new(config: Config) -> Self {
        let address = config.server.address;
        Self { config, shutdown_tx: None, server_handle: None, address }
    }

    /// Start the server in a background task.
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to start or if it's already running.
    pub async fn start(&mut self) -> Result<()> {
        if self.is_running() {
            return Err(RadiumError::Config("Server is already running".to_string()));
        }

        info!(
            address = %self.address,
            "Starting embedded Radium server"
        );

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // Clone config for the server task
        let config = self.config.clone();

        // Spawn server in background task
        let server_handle =
            tokio::spawn(async move { server::run_with_shutdown(&config, shutdown_rx).await });

        self.shutdown_tx = Some(shutdown_tx);
        self.server_handle = Some(server_handle);

        // Give server a moment to start binding
        sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    /// Wait for the server to be ready (accepting connections).
    ///
    /// This polls the server's Ping endpoint until it responds successfully.
    ///
    /// # Arguments
    /// * `timeout` - Maximum time to wait for server to be ready
    ///
    /// # Errors
    ///
    /// Returns an error if the server doesn't become ready within the timeout,
    /// or if the server task has failed.
    pub async fn wait_for_ready(&self, timeout: Duration) -> Result<()> {
        let start_time = std::time::Instant::now();
        let mut poll_interval = interval(Duration::from_millis(100));
        poll_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // Check if server task is still alive
        if let Some(ref handle) = self.server_handle {
            if handle.is_finished() {
                return Err(RadiumError::Config(
                    "Server task completed unexpectedly before becoming ready".to_string(),
                ));
            }
        } else {
            return Err(RadiumError::Config("Server is not running".to_string()));
        }

        // Try to connect and ping the server
        let server_url = format!("http://{}", self.address);
        let endpoint = tonic::transport::Endpoint::from_shared(server_url.clone())
            .map_err(|e| RadiumError::Config(format!("Invalid server address: {}", e)))?;

        loop {
            // Check timeout
            if start_time.elapsed() > timeout {
                return Err(RadiumError::Config(format!(
                    "Server did not become ready within {:?}",
                    timeout
                )));
            }

            // Check if server task failed
            if let Some(ref handle) = self.server_handle {
                if handle.is_finished() {
                    return Err(RadiumError::Config(
                        "Server task completed unexpectedly while waiting for ready".to_string(),
                    ));
                }
            }

            // Try to connect and ping
            match endpoint.connect().await {
                Ok(channel) => {
                    let mut client = RadiumClient::new(channel);
                    let request =
                        tonic::Request::new(PingRequest { message: "health-check".to_string() });

                    match client.ping(request).await {
                        Ok(_) => {
                            info!(
                                address = %self.address,
                                elapsed_ms = start_time.elapsed().as_millis(),
                                "Embedded server is ready"
                            );
                            return Ok(());
                        }
                        Err(e) => {
                            debug!(
                                error = %e,
                                "Server not ready yet (ping failed), retrying..."
                            );
                        }
                    }
                }
                Err(e) => {
                    debug!(
                        error = %e,
                        "Server not ready yet (connection failed), retrying..."
                    );
                }
            }

            // Wait before next attempt
            poll_interval.tick().await;
        }
    }

    /// Check if the server is currently running.
    ///
    /// # Returns
    /// `true` if the server task exists and is still running, `false` otherwise.
    #[must_use]
    pub fn is_running(&self) -> bool {
        if let Some(ref handle) = self.server_handle { !handle.is_finished() } else { false }
    }

    /// Get the server address.
    ///
    /// # Returns
    /// The socket address the server is bound to.
    #[must_use]
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// Gracefully shutdown the server.
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown signal fails to send, or if waiting for server
    /// to stop times out.
    pub async fn shutdown(&mut self) -> Result<()> {
        if !self.is_running() {
            return Ok(());
        }

        info!(address = %self.address, "Shutting down embedded server");

        // Send shutdown signal
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            if shutdown_tx.send(()).is_err() {
                warn!("Shutdown signal receiver already dropped");
            }
        }

        // Wait for server task to complete
        if let Some(handle) = self.server_handle.take() {
            // Give server time to shutdown gracefully
            match tokio::time::timeout(Duration::from_secs(5), handle).await {
                Ok(Ok(_)) => {
                    info!("Embedded server stopped gracefully");
                    Ok(())
                }
                Ok(Err(e)) => {
                    warn!(error = %e, "Server task returned error during shutdown");
                    Err(RadiumError::Config(format!("Server shutdown error: {}", e)))
                }
                Err(_) => {
                    warn!("Server shutdown timed out, task may still be running");
                    Err(RadiumError::Config("Server shutdown timed out".to_string()))
                }
            }
        } else {
            Ok(())
        }
    }
}

impl Drop for EmbeddedServer {
    fn drop(&mut self) {
        if self.is_running() {
            warn!("EmbeddedServer dropped while running - attempting graceful shutdown");
            // Try to shutdown, but we can't use async in drop
            // The shutdown signal will be sent when shutdown_tx is dropped
            if let Some(shutdown_tx) = self.shutdown_tx.take() {
                let _ = shutdown_tx.send(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedded_server_new() {
        let config = Config::default();
        let server = EmbeddedServer::new(config);
        assert!(!server.is_running());
        assert_eq!(server.address().port(), 50051);
    }

    #[tokio::test]
    async fn test_embedded_server_start() {
        // Find an available port to avoid conflicts
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let mut config = Config::default();
        config.server.address = format!("127.0.0.1:{}", port).parse().unwrap();
        config.server.enable_grpc_web = false;

        let mut server = EmbeddedServer::new(config);
        server.start().await.unwrap();
        assert!(server.is_running());

        // Cleanup
        server.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_embedded_server_wait_for_ready() {
        // Find an available port
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let mut config = Config::default();
        config.server.address = format!("127.0.0.1:{}", port).parse().unwrap();
        config.server.enable_grpc_web = false;

        let mut server = EmbeddedServer::new(config);
        server.start().await.unwrap();

        // Wait for server to be ready
        server.wait_for_ready(Duration::from_secs(5)).await.unwrap();

        assert!(server.is_running());

        // Cleanup
        server.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_embedded_server_shutdown() {
        // Find an available port
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let mut config = Config::default();
        config.server.address = format!("127.0.0.1:{}", port).parse().unwrap();
        config.server.enable_grpc_web = false;

        let mut server = EmbeddedServer::new(config);
        server.start().await.unwrap();
        assert!(server.is_running());

        server.shutdown().await.unwrap();
        assert!(!server.is_running());
    }

    #[tokio::test]
    async fn test_embedded_server_double_start() {
        // Find an available port
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let mut config = Config::default();
        config.server.address = format!("127.0.0.1:{}", port).parse().unwrap();
        config.server.enable_grpc_web = false;

        let mut server = EmbeddedServer::new(config);
        server.start().await.unwrap();

        // Try to start again - should fail
        let result = server.start().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already running"));

        // Cleanup
        server.shutdown().await.unwrap();
    }
}

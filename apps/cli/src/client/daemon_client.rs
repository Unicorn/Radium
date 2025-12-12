//! Daemon client with connection management and retry logic.

use anyhow::{Context, Result};
use radium_core::radium_client::RadiumClient;
use std::time::Duration;
use tokio::time::sleep;
use tonic::transport::{Channel, Endpoint};
use tracing::{debug, info, warn};

/// Execution mode for CLI commands.
#[derive(Debug, Clone)]
pub enum ExecutionMode {
    /// Local in-process execution (current default).
    Local,
    /// Remote daemon execution.
    Daemon(String), // URL
}

impl ExecutionMode {
    /// Determine execution mode from CLI arguments.
    pub fn from_args(daemon: Option<String>, local: bool) -> Self {
        if local {
            Self::Local
        } else if let Some(url) = daemon {
            Self::Daemon(url)
        } else {
            Self::Local // Default to local
        }
    }
}

/// Daemon client wrapper with retry logic and connection management.
#[allow(dead_code)]
pub struct DaemonClient {
    /// Server URL
    url: String,
    /// Cached client connection
    client: Option<RadiumClient<Channel>>,
}

impl DaemonClient {
    /// Create a new daemon client.
    ///
    /// # Arguments
    /// * `url` - Daemon server URL (e.g., "http://localhost:50051")
    pub fn new(url: String) -> Self {
        Self { url, client: None }
    }

    /// Get or create a connected client with retry logic.
    ///
    /// # Arguments
    /// * `max_retries` - Maximum number of retry attempts (default: 3)
    ///
    /// # Returns
    /// Connected RadiumClient or error if connection fails after retries.
    pub async fn connect(&mut self, max_retries: Option<usize>) -> Result<RadiumClient<Channel>> {
        let max_retries = max_retries.unwrap_or(3);

        // Check if we have a cached client
        if let Some(ref client) = self.client {
            // TODO: Add health check to verify connection is still alive
            return Ok(client.clone());
        }

        // Create new connection with retry logic
        info!(url = %self.url, "Connecting to daemon");

        let endpoint = Endpoint::from_shared(self.url.clone())
            .context("Invalid daemon URL")?;

        let mut retry_delay = Duration::from_secs(1);
        let mut last_error = None;

        for attempt in 0..max_retries {
            match endpoint.connect().await {
                Ok(channel) => {
                    let client = RadiumClient::new(channel);
                    info!("Connected to daemon");
                    self.client = Some(client.clone());
                    return Ok(client);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries - 1 {
                        debug!(
                            attempt = attempt + 1,
                            max_retries = max_retries,
                            delay_secs = retry_delay.as_secs(),
                            "Connection failed, retrying..."
                        );
                        sleep(retry_delay).await;
                        retry_delay = Duration::from_secs(retry_delay.as_secs() * 2); // Exponential backoff
                    }
                }
            }
        }

        Err(anyhow::anyhow!(
            "Failed to connect to daemon after {} attempts: {}",
            max_retries,
            last_error.unwrap()
        ))
    }

    /// Perform a health check by pinging the daemon.
    ///
    /// # Returns
    /// True if daemon is healthy, false otherwise.
    pub async fn health_check(&mut self) -> Result<bool> {
        let mut client = self.connect(None).await?;
        match client.ping(radium_core::proto::PingRequest {
            message: "health_check".to_string(),
        }).await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get the daemon URL.
    pub fn url(&self) -> &str {
        &self.url
    }
}

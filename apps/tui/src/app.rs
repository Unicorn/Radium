//! Application state management

use radium_core::radium_client::RadiumClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;

/// Application state
#[derive(Clone)]
pub struct AppState {
    /// gRPC client (if connected)
    pub client: Arc<Mutex<Option<RadiumClient<Channel>>>>,
    /// Server address
    pub server_addr: String,
    /// Connection status
    pub connection_status: Arc<Mutex<String>>,
}

impl AppState {
    pub fn new(server_addr: String) -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            server_addr,
            connection_status: Arc::new(Mutex::new("Disconnected".to_string())),
        }
    }

    pub async fn connect(&self) -> anyhow::Result<()> {
        *self.connection_status.lock().await = format!("Connecting to {}...", self.server_addr);

        let channel = Channel::from_shared(self.server_addr.clone())
            .map_err(|e| anyhow::anyhow!("Invalid URI: {}", e))?
            .connect()
            .await?;

        let client = RadiumClient::new(channel);
        *self.client.lock().await = Some(client);
        *self.connection_status.lock().await = format!("Connected to {}", self.server_addr);

        Ok(())
    }
}

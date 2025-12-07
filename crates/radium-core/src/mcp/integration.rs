//! MCP integration with agent system.

use crate::mcp::client::McpClient;
use crate::mcp::config::McpConfigManager;
use crate::mcp::tools::McpToolRegistry;
use crate::mcp::{McpError, Result};
use crate::workspace::Workspace;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// MCP integration manager for connecting MCP servers and making tools available.
pub struct McpIntegration {
    /// Connected MCP clients.
    clients: Arc<Mutex<HashMap<String, Arc<Mutex<McpClient>>>>>,
    /// Tool registries for each server.
    tool_registries: Arc<Mutex<HashMap<String, McpToolRegistry>>>,
}

impl McpIntegration {
    /// Create a new MCP integration manager.
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            tool_registries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Initialize MCP integration by loading and connecting to configured servers.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration loading or connection fails.
    pub async fn initialize(&self, workspace: &Workspace) -> Result<()> {
        let config_path = McpConfigManager::default_config_path(workspace.root());
        let mut config_manager = McpConfigManager::new(config_path);
        config_manager.load()?;

        let mut clients = self.clients.lock().await;
        let mut tool_registries = self.tool_registries.lock().await;

        for server_config in config_manager.get_servers() {
            match McpClient::connect(server_config).await {
                Ok(client) => {
                    let client = Arc::new(Mutex::new(client));
                    clients.insert(server_config.name.clone(), client.clone());

                    // Discover tools
                    let mut tool_registry = McpToolRegistry::new(server_config.name.clone());
                    let tools = client.lock().await.discover_tools().await?;
                    for tool in tools {
                        tool_registry.register_tool(tool);
                    }

                    tool_registries.insert(server_config.name.clone(), tool_registry);
                }
                Err(e) => {
                    // Log error but continue with other servers
                    tracing::warn!(
                        "Failed to connect to MCP server '{}': {}",
                        server_config.name,
                        e
                    );
                }
            }
        }

        Ok(())
    }

    /// Get all available tools from all MCP servers.
    pub async fn get_all_tools(&self) -> Vec<(String, Vec<String>)> {
        let tool_registries = self.tool_registries.lock().await;
        tool_registries
            .iter()
            .map(|(server_name, registry)| {
                let tool_names: Vec<String> = registry
                    .get_all_tools()
                    .iter()
                    .map(|t| t.name.clone())
                    .collect();
                (server_name.clone(), tool_names)
            })
            .collect()
    }

    /// Execute a tool from an MCP server.
    ///
    /// # Errors
    ///
    /// Returns an error if the tool cannot be found or executed.
    pub async fn execute_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<crate::mcp::McpToolResult> {
        let clients = self.clients.lock().await;
        let client = clients
            .get(server_name)
            .ok_or_else(|| McpError::ServerNotFound(server_name.to_string()))?;

        let client = client.lock().await;
        client.execute_tool(tool_name, arguments).await
    }

    /// Check if an MCP server is connected.
    pub async fn is_server_connected(&self, server_name: &str) -> bool {
        let clients = self.clients.lock().await;
        clients.contains_key(server_name)
    }

    /// Get the number of connected servers.
    pub async fn connected_server_count(&self) -> usize {
        let clients = self.clients.lock().await;
        clients.len()
    }
}

impl Default for McpIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_integration_creation() {
        let _integration = McpIntegration::new();
        // Can't easily test async methods without a real workspace
        // But we can verify the struct is created
        assert!(true);
    }
}


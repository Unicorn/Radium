//! MCP integration with agent system.

use crate::mcp::client::McpClient;
use crate::mcp::config::McpConfigManager;
use crate::mcp::tools::McpToolRegistry;
use crate::mcp::prompts::SlashCommandRegistry;
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
    /// Slash command registry for MCP prompts.
    slash_registry: Arc<Mutex<SlashCommandRegistry>>,
}

impl McpIntegration {
    /// Create a new MCP integration manager.
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            tool_registries: Arc::new(Mutex::new(HashMap::new())),
            slash_registry: Arc::new(Mutex::new(SlashCommandRegistry::new())),
        }
    }

    /// Initialize MCP integration by loading and connecting to configured servers.
    ///
    /// Loads MCP server configurations from:
    /// 1. Workspace MCP config file (.radium/mcp-servers.toml) - highest precedence
    /// 2. Extension MCP configs (from installed extensions) - lower precedence
    ///
    /// Extension configs are loaded after workspace configs, so workspace configs
    /// take precedence if there are name conflicts.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration loading or connection fails.
    pub async fn initialize(&self, workspace: &Workspace) -> Result<()> {
        let config_path = McpConfigManager::default_config_path(workspace.root());
        let mut config_manager = McpConfigManager::new(config_path);
        config_manager.load()?;

        // Collect all server configs (workspace + extensions)
        let mut all_servers: Vec<crate::mcp::McpServerConfig> = config_manager.get_servers().to_vec();

        // Load extension MCP configs and add to server list
        // Extension configs are loaded after workspace configs (lower precedence)
        if let Ok(extension_config_paths) = crate::extensions::integration::get_extension_mcp_configs() {
            for config_path in extension_config_paths {
                // Try to load each extension MCP config
                // Extension configs may be in JSON format, so we need to handle both
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    // Try to parse as TOML first (workspace format)
                    if let Ok(toml_table) = toml::from_str::<toml::Table>(&content) {
                        if let Ok(extension_servers) = McpConfigManager::parse_servers(&toml_table) {
                            // Add extension servers (workspace servers take precedence)
                            for server_config in extension_servers {
                                // Only add if not already present (workspace configs take precedence)
                                if !all_servers.iter().any(|s| s.name == server_config.name) {
                                    all_servers.push(server_config);
                                }
                            }
                        }
                    }
                    // If TOML parsing fails, try JSON (extension format)
                    else if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                        // Convert JSON to McpServerConfig
                        if let Some(server_config) = Self::parse_json_mcp_config(&json_value) {
                            // Only add if not already present
                            if !all_servers.iter().any(|s| s.name == server_config.name) {
                                all_servers.push(server_config);
                            }
                        }
                    }
                }
            }
        }

        let mut clients = self.clients.lock().await;
        let mut tool_registries = self.tool_registries.lock().await;

        for server_config in all_servers {
            match McpClient::connect(&server_config).await {
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

                    // Discover prompts and register as slash commands
                    let mut slash_registry = self.slash_registry.lock().await;
                    if let Ok(prompts) = client.lock().await.list_prompts().await {
                        for prompt in prompts {
                            slash_registry.register_prompt_with_server(
                                server_config.name.clone(),
                                prompt,
                            );
                        }
                    }
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

    /// Parse a JSON MCP server configuration into McpServerConfig.
    ///
    /// This allows extensions to provide MCP configs in JSON format.
    fn parse_json_mcp_config(json: &serde_json::Value) -> Option<crate::mcp::McpServerConfig> {
        use crate::mcp::{McpServerConfig, TransportType};

        let name = json.get("name")?.as_str()?.to_string();
        let transport_str = json.get("transport")?.as_str()?;
        let transport = match transport_str {
            "stdio" => TransportType::Stdio,
            "sse" => TransportType::Sse,
            "http" => TransportType::Http,
            _ => return None,
        };

        let command = json.get("command").and_then(|v| v.as_str()).map(|s| s.to_string());
        let args = json.get("args")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
        let url = json.get("url").and_then(|v| v.as_str()).map(|s| s.to_string());

        // Validate transport-specific requirements
        match transport {
            TransportType::Stdio => {
                if command.is_none() {
                    return None;
                }
            }
            TransportType::Sse | TransportType::Http => {
                if url.is_none() {
                    return None;
                }
            }
        }

        Some(McpServerConfig { name, transport, command, args, url, auth: None })
    }

    /// Get all available tools from all MCP servers.
    pub async fn get_all_tools(&self) -> Vec<(String, Vec<String>)> {
        let tool_registries = self.tool_registries.lock().await;
        tool_registries
            .iter()
            .map(|(server_name, registry)| {
                let tool_names: Vec<String> =
                    registry.get_all_tools().iter().map(|t| t.name.clone()).collect();
                (server_name.clone(), tool_names)
            })
            .collect()
    }

    /// Get all tool definitions with their full metadata from all MCP servers.
    ///
    /// Returns a vector of tuples: (server_name, tool_name, tool_definition)
    /// where tool_name is the name used in the registry (may be prefixed for conflicts).
    pub async fn get_all_tool_definitions(&self) -> Vec<(String, String, crate::mcp::McpTool)> {
        let tool_registries = self.tool_registries.lock().await;
        let mut result = Vec::new();

        for (server_name, registry) in tool_registries.iter() {
            for (tool_name, tool) in registry.get_all_tools_with_names() {
                result.push((server_name.clone(), tool_name, tool.clone()));
            }
        }

        result
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

    /// Get all prompts from all MCP servers.
    ///
    /// Returns a vector of tuples: (server_name, prompt)
    pub async fn get_all_prompts(&self) -> Vec<(String, crate::mcp::McpPrompt)> {
        let clients = self.clients.lock().await;
        let mut result = Vec::new();

        for (server_name, client) in clients.iter() {
            let client = client.lock().await;
            if let Ok(prompts) = client.list_prompts().await {
                for prompt in prompts {
                    result.push((server_name.clone(), prompt));
                }
            }
        }

        result
    }

    /// Execute a prompt from an MCP server.
    ///
    /// # Arguments
    /// * `server_name` - Name of the server that has the prompt
    /// * `prompt_name` - Name of the prompt to execute
    /// * `arguments` - Optional arguments for the prompt
    ///
    /// # Errors
    ///
    /// Returns an error if the prompt cannot be found or executed.
    pub async fn execute_prompt(
        &self,
        server_name: &str,
        prompt_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let clients = self.clients.lock().await;
        let client = clients
            .get(server_name)
            .ok_or_else(|| McpError::ServerNotFound(server_name.to_string()))?;

        let client = client.lock().await;
        client.execute_prompt(prompt_name, arguments).await
    }

    /// Get the slash command registry.
    pub fn slash_registry(&self) -> Arc<Mutex<SlashCommandRegistry>> {
        Arc::clone(&self.slash_registry)
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

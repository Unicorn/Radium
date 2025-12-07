//! MCP prompts as slash commands.

use crate::mcp::client::McpClient;
use crate::mcp::{McpError, McpPrompt, Result};
use serde_json::{Value, json};

impl McpClient {
    /// List prompts from the MCP server.
    ///
    /// # Errors
    ///
    /// Returns an error if prompt listing fails.
    pub async fn list_prompts(&self) -> Result<Vec<McpPrompt>> {
        let result = self.send_request("prompts/list", None).await?;

        let prompts_value = result.get("prompts").ok_or_else(|| {
            McpError::protocol(
                "prompts/list response missing 'prompts' field",
                "The MCP server did not return a 'prompts' field in the prompts/list response. This may indicate:\n  - Server protocol version mismatch\n  - Server implementation error\n\nCheck the server logs and ensure it supports the MCP prompts/list method.",
            )
        })?;

        let prompts: Vec<McpPrompt> = serde_json::from_value(prompts_value.clone())
            .map_err(|e| McpError::protocol(
                format!("Failed to parse prompts: {}", e),
                "The MCP server returned prompts in an invalid format. This may indicate:\n  - Server protocol version mismatch\n  - Malformed server response\n\nCheck the server logs and verify it follows the MCP protocol specification.",
            ))?;

        Ok(prompts)
    }

    /// Get a prompt by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the prompt cannot be retrieved.
    pub async fn get_prompt(&self, prompt_name: &str) -> Result<McpPrompt> {
        let prompts = self.list_prompts().await?;
        prompts
            .into_iter()
            .find(|p| p.name == prompt_name)
            .ok_or_else(|| McpError::protocol(
                format!("Prompt '{}' not found", prompt_name),
                format!(
                    "The prompt '{}' is not available from this MCP server. Try:\n  - List available prompts: rad mcp prompts\n  - Check the prompt name spelling\n  - Verify the server supports this prompt",
                    prompt_name
                ),
            ))
    }

    /// Execute a prompt with arguments.
    ///
    /// # Errors
    ///
    /// Returns an error if prompt execution fails.
    pub async fn execute_prompt(
        &self,
        prompt_name: &str,
        arguments: Option<Value>,
    ) -> Result<Value> {
        let params = json!({
            "name": prompt_name,
            "arguments": arguments.unwrap_or_else(|| json!({}))
        });

        let result = self.send_request("prompts/get", Some(params)).await?;

        Ok(result)
    }
}

/// Slash command registry for MCP prompts.
pub struct SlashCommandRegistry {
    /// Map of command names to prompts.
    commands: std::collections::HashMap<String, McpPrompt>,
    /// Map of command names to server names.
    command_to_server: std::collections::HashMap<String, String>,
}

impl SlashCommandRegistry {
    /// Create a new slash command registry.
    pub fn new() -> Self {
        Self {
            commands: std::collections::HashMap::new(),
            command_to_server: std::collections::HashMap::new(),
        }
    }

    /// Register a prompt as a slash command.
    pub fn register_prompt(&mut self, prompt: McpPrompt) {
        let command_name = format!("/{}", prompt.name.replace(' ', "_").to_lowercase());
        self.commands.insert(command_name.clone(), prompt);
    }

    /// Register a prompt with its server name.
    pub fn register_prompt_with_server(&mut self, server_name: String, prompt: McpPrompt) {
        let command_name = format!("/{}", prompt.name.replace(' ', "_").to_lowercase());
        self.commands.insert(command_name.clone(), prompt);
        self.command_to_server.insert(command_name, server_name);
    }

    /// Get the server name for a command.
    pub fn get_server_for_command(&self, command_name: &str) -> Option<&String> {
        self.command_to_server.get(command_name)
    }

    /// Get a command by name.
    pub fn get_command(&self, command_name: &str) -> Option<&McpPrompt> {
        self.commands.get(command_name)
    }

    /// Get all registered commands.
    pub fn get_all_commands(&self) -> Vec<(&String, &McpPrompt)> {
        self.commands.iter().collect()
    }

    /// Check if a command exists.
    pub fn has_command(&self, command_name: &str) -> bool {
        self.commands.contains_key(command_name)
    }
}

impl Default for SlashCommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slash_command_registry_creation() {
        let registry = SlashCommandRegistry::new();
        assert_eq!(registry.get_all_commands().len(), 0);
    }

    #[test]
    fn test_slash_command_registry_register() {
        let mut registry = SlashCommandRegistry::new();
        let prompt = McpPrompt {
            name: "test prompt".to_string(),
            description: Some("A test prompt".to_string()),
            arguments: None,
        };

        registry.register_prompt(prompt);
        assert_eq!(registry.get_all_commands().len(), 1);
        assert!(registry.has_command("/test_prompt"));
    }

    #[test]
    fn test_slash_command_registry_get() {
        let mut registry = SlashCommandRegistry::new();
        let prompt = McpPrompt {
            name: "test prompt".to_string(),
            description: Some("A test prompt".to_string()),
            arguments: None,
        };

        registry.register_prompt(prompt);
        let retrieved = registry.get_command("/test_prompt");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test prompt");
    }

    #[test]
    fn test_slash_command_registry_with_server() {
        let mut registry = SlashCommandRegistry::new();
        let prompt = McpPrompt {
            name: "test prompt".to_string(),
            description: Some("A test prompt".to_string()),
            arguments: None,
        };

        registry.register_prompt_with_server("test-server".to_string(), prompt);
        assert!(registry.has_command("/test_prompt"));
        assert_eq!(
            registry.get_server_for_command("/test_prompt"),
            Some(&"test-server".to_string())
        );
    }
}

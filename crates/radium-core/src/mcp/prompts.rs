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
            McpError::Protocol("prompts/list response missing 'prompts' field".to_string())
        })?;

        let prompts: Vec<McpPrompt> = serde_json::from_value(prompts_value.clone())
            .map_err(|e| McpError::Protocol(format!("Failed to parse prompts: {}", e)))?;

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
            .ok_or_else(|| McpError::Protocol(format!("Prompt not found: {}", prompt_name)))
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
}

impl SlashCommandRegistry {
    /// Create a new slash command registry.
    pub fn new() -> Self {
        Self { commands: std::collections::HashMap::new() }
    }

    /// Register a prompt as a slash command.
    pub fn register_prompt(&mut self, prompt: McpPrompt) {
        let command_name = format!("/{}", prompt.name.replace(' ', "_").to_lowercase());
        self.commands.insert(command_name, prompt);
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
}

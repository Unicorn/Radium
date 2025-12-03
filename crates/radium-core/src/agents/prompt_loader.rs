//! Agent prompt loading and management.
//!
//! Provides high-level integration between agent configuration and prompt templates.

use crate::agents::config::AgentConfig;
use crate::prompts::{
    PromptCache, PromptContext, PromptError, PromptTemplate, RenderOptions, ValidationResult,
    validate_prompt,
};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Agent prompt loader errors.
#[derive(Debug, Error)]
pub enum PromptLoaderError {
    /// Prompt error.
    #[error("prompt error: {0}")]
    Prompt(#[from] PromptError),

    /// Prompt file not found.
    #[error("prompt file not found: {0}")]
    PromptNotFound(String),

    /// Invalid agent configuration.
    #[error("invalid agent configuration: {0}")]
    InvalidConfig(String),
}

/// Result type for prompt loader operations.
pub type Result<T> = std::result::Result<T, PromptLoaderError>;

/// Agent prompt loader.
///
/// Loads and manages prompts for agents, integrating agent configuration
/// with prompt templates.
pub struct AgentPromptLoader {
    cache: PromptCache,
    base_path: Option<PathBuf>,
}

impl AgentPromptLoader {
    /// Create a new agent prompt loader.
    pub fn new() -> Self {
        Self { cache: PromptCache::new(), base_path: None }
    }

    /// Create a new agent prompt loader with a base path.
    ///
    /// The base path is used to resolve relative prompt paths and file injection paths.
    pub fn with_base_path(base_path: impl Into<PathBuf>) -> Self {
        Self {
            cache: PromptCache::new(),
            base_path: Some(base_path.into()),
        }
    }

    /// Load a prompt template for an agent.
    ///
    /// # Arguments
    ///
    /// * `agent_config` - The agent configuration
    ///
    /// # Errors
    ///
    /// Returns error if the prompt file cannot be loaded.
    pub fn load_prompt(&self, agent_config: &AgentConfig) -> Result<PromptTemplate> {
        let prompt_path = self.resolve_prompt_path(agent_config)?;

        if !prompt_path.exists() {
            return Err(PromptLoaderError::PromptNotFound(
                prompt_path.display().to_string(),
            ));
        }

        self.cache.load(&prompt_path).map_err(PromptLoaderError::from)
    }

    /// Render a prompt for an agent with the given context.
    ///
    /// # Arguments
    ///
    /// * `agent_config` - The agent configuration
    /// * `context` - The prompt context with variable values
    ///
    /// # Errors
    ///
    /// Returns error if the prompt cannot be loaded or rendered.
    pub fn render_prompt(
        &self,
        agent_config: &AgentConfig,
        context: &PromptContext,
    ) -> Result<String> {
        let template = self.load_prompt(agent_config)?;

        let options = RenderOptions {
            base_path: self.base_path.clone(),
            strict: false,
            default_value: None,
        };

        template.render_with_options(context, &options).map_err(PromptLoaderError::from)
    }

    /// Validate an agent's prompt.
    ///
    /// # Arguments
    ///
    /// * `agent_config` - The agent configuration
    /// * `required_placeholders` - List of placeholders that must be present
    ///
    /// # Returns
    ///
    /// Validation result with any issues found.
    pub fn validate_prompt(
        &self,
        agent_config: &AgentConfig,
        required_placeholders: &[String],
    ) -> Result<ValidationResult> {
        let template = self.load_prompt(agent_config)?;
        Ok(validate_prompt(&template, required_placeholders))
    }

    /// Resolve the prompt path for an agent.
    ///
    /// Handles both absolute and relative paths, using the base path if set.
    fn resolve_prompt_path(&self, agent_config: &AgentConfig) -> Result<PathBuf> {
        let prompt_path = &agent_config.prompt_path;

        // If path is absolute, use it directly
        if prompt_path.is_absolute() {
            return Ok(prompt_path.clone());
        }

        // If we have a base path, resolve relative to it
        if let Some(base) = &self.base_path {
            return Ok(base.join(prompt_path));
        }

        // Try relative to current directory
        if let Ok(cwd) = std::env::current_dir() {
            let resolved = cwd.join(prompt_path);
            if resolved.exists() {
                return Ok(resolved);
            }
        }

        // Try relative to agent config file location
        if let Some(config_file) = &agent_config.file_path {
            if let Some(config_dir) = config_file.parent() {
                let resolved = config_dir.join(prompt_path);
                if resolved.exists() {
                    return Ok(resolved);
                }
            }
        }

        // Return the path as-is (will fail on load if doesn't exist)
        Ok(prompt_path.clone())
    }

    /// Clear the prompt cache.
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}

impl Default for AgentPromptLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::config::AgentConfig;
    use std::fs;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    fn create_test_agent_config(
        temp_dir: &TempDir,
        prompt_content: &str,
    ) -> (AgentConfig, PathBuf) {
        let prompt_file = temp_dir.path().join("prompt.md");
        fs::write(&prompt_file, prompt_content).unwrap();

        let config = AgentConfig::new("test-agent", "Test Agent", prompt_file.clone())
            .with_description("A test agent");

        (config, prompt_file)
    }

    #[test]
    fn test_load_prompt() {
        let temp_dir = TempDir::new().unwrap();
        let (config, _) = create_test_agent_config(&temp_dir, "Hello {{name}}!");

        let loader = AgentPromptLoader::with_base_path(temp_dir.path());
        let template = loader.load_prompt(&config).unwrap();

        assert_eq!(template.content(), "Hello {{name}}!");
    }

    #[test]
    fn test_render_prompt() {
        let temp_dir = TempDir::new().unwrap();
        let (config, _) = create_test_agent_config(&temp_dir, "Hello {{name}}!");

        let loader = AgentPromptLoader::with_base_path(temp_dir.path());
        let mut context = PromptContext::new();
        context.set("name", "World");

        let result = loader.render_prompt(&config, &context).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_validate_prompt() {
        let temp_dir = TempDir::new().unwrap();
        let (config, _) = create_test_agent_config(&temp_dir, "Hello {{name}}!");

        let loader = AgentPromptLoader::with_base_path(temp_dir.path());
        let required = vec!["name".to_string()];
        let result = loader.validate_prompt(&config, &required).unwrap();

        assert!(result.is_valid);
    }

    #[test]
    fn test_prompt_not_found() {
        let config = AgentConfig::new("test-agent", "Test Agent", PathBuf::from("nonexistent.md"));

        let loader = AgentPromptLoader::new();
        let result = loader.load_prompt(&config);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PromptLoaderError::PromptNotFound(_)));
    }
}

//! Engine trait definition and metadata.

use super::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Engine metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMetadata {
    /// Unique engine identifier.
    pub id: String,

    /// Human-readable engine name.
    pub name: String,

    /// Engine description.
    pub description: String,

    /// CLI command to execute.
    pub cli_command: Option<String>,

    /// Supported models.
    pub models: Vec<String>,

    /// Whether engine requires authentication.
    pub requires_auth: bool,

    /// Engine version (if detectable).
    pub version: Option<String>,
}

impl EngineMetadata {
    /// Creates new engine metadata.
    pub fn new(id: String, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            cli_command: None,
            models: Vec::new(),
            requires_auth: false,
            version: None,
        }
    }

    /// Sets the CLI command.
    #[must_use]
    pub fn with_cli_command(mut self, command: String) -> Self {
        self.cli_command = Some(command);
        self
    }

    /// Sets supported models.
    #[must_use]
    pub fn with_models(mut self, models: Vec<String>) -> Self {
        self.models = models;
        self
    }

    /// Sets authentication requirement.
    #[must_use]
    pub fn with_auth_required(mut self, required: bool) -> Self {
        self.requires_auth = required;
        self
    }

    /// Sets version.
    #[must_use]
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }
}

/// Engine execution request.
#[derive(Debug, Clone)]
pub struct ExecutionRequest {
    /// Model to use.
    pub model: String,

    /// Prompt or messages.
    pub prompt: String,

    /// Optional system message.
    pub system: Option<String>,

    /// Temperature (0.0-1.0).
    pub temperature: Option<f32>,

    /// Maximum tokens to generate.
    pub max_tokens: Option<usize>,

    /// Additional parameters.
    pub params: std::collections::HashMap<String, serde_json::Value>,
}

impl ExecutionRequest {
    /// Creates a new execution request.
    pub fn new(model: String, prompt: String) -> Self {
        Self {
            model,
            prompt,
            system: None,
            temperature: None,
            max_tokens: None,
            params: std::collections::HashMap::new(),
        }
    }

    /// Sets the system message.
    #[must_use]
    pub fn with_system(mut self, system: String) -> Self {
        self.system = Some(system);
        self
    }

    /// Sets the temperature.
    #[must_use]
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets max tokens.
    #[must_use]
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
}

/// Engine execution response.
#[derive(Debug, Clone)]
pub struct ExecutionResponse {
    /// Generated content.
    pub content: String,

    /// Token usage information.
    pub usage: Option<TokenUsage>,

    /// Model used.
    pub model: String,

    /// Raw response (for debugging).
    pub raw: Option<String>,
}

/// Token usage information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Input/prompt tokens.
    pub input_tokens: u64,

    /// Output/completion tokens.
    pub output_tokens: u64,

    /// Total tokens.
    pub total_tokens: u64,
}

/// Engine trait for AI provider abstraction.
#[async_trait]
pub trait Engine: Send + Sync {
    /// Gets engine metadata.
    fn metadata(&self) -> &EngineMetadata;

    /// Checks if the engine is available (binary exists, etc.).
    async fn is_available(&self) -> bool;

    /// Checks if the engine is authenticated.
    async fn is_authenticated(&self) -> Result<bool>;

    /// Executes a request.
    ///
    /// # Arguments
    /// * `request` - Execution request
    ///
    /// # Returns
    /// Execution response
    ///
    /// # Errors
    /// Returns error if execution fails
    async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResponse>;

    /// Gets the default model for this engine.
    fn default_model(&self) -> String;

    /// Lists available models.
    fn available_models(&self) -> Vec<String> {
        self.metadata().models.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_metadata_new() {
        let metadata = EngineMetadata::new(
            "test-engine".to_string(),
            "Test Engine".to_string(),
            "A test engine".to_string(),
        );

        assert_eq!(metadata.id, "test-engine");
        assert_eq!(metadata.name, "Test Engine");
        assert!(!metadata.requires_auth);
    }

    #[test]
    fn test_engine_metadata_builder() {
        let metadata = EngineMetadata::new(
            "claude".to_string(),
            "Claude".to_string(),
            "Anthropic Claude".to_string(),
        )
        .with_cli_command("claude".to_string())
        .with_models(vec!["claude-3-opus".to_string(), "claude-3-sonnet".to_string()])
        .with_auth_required(true)
        .with_version("1.0.0".to_string());

        assert_eq!(metadata.cli_command, Some("claude".to_string()));
        assert_eq!(metadata.models.len(), 2);
        assert!(metadata.requires_auth);
        assert_eq!(metadata.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_execution_request_new() {
        let request = ExecutionRequest::new("gpt-4".to_string(), "Hello".to_string());

        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.prompt, "Hello");
        assert!(request.system.is_none());
    }

    #[test]
    fn test_execution_request_builder() {
        let request = ExecutionRequest::new("gpt-4".to_string(), "Hello".to_string())
            .with_system("You are helpful".to_string())
            .with_temperature(0.7)
            .with_max_tokens(1000);

        assert_eq!(request.system, Some("You are helpful".to_string()));
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.max_tokens, Some(1000));
    }
}

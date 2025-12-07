// Orchestration configuration
//
// Manages configuration for orchestration providers, including model selection,
// temperature settings, and provider-specific options.

use serde::{Deserialize, Serialize};

/// Orchestration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationConfig {
    /// Whether orchestration is enabled
    pub enabled: bool,
    /// Default provider to use
    pub default_provider: ProviderType,
    /// Gemini provider configuration
    pub gemini: GeminiConfig,
    /// Claude provider configuration
    pub claude: ClaudeConfig,
    /// OpenAI provider configuration
    pub openai: OpenAIConfig,
    /// Prompt-based provider configuration
    pub prompt_based: PromptBasedConfig,
    /// Fallback configuration
    pub fallback: FallbackConfig,
}

impl Default for OrchestrationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_provider: ProviderType::Gemini,
            gemini: GeminiConfig::default(),
            claude: ClaudeConfig::default(),
            openai: OpenAIConfig::default(),
            prompt_based: PromptBasedConfig::default(),
            fallback: FallbackConfig::default(),
        }
    }
}

impl OrchestrationConfig {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration with a specific default provider
    pub fn with_provider(mut self, provider: ProviderType) -> Self {
        self.default_provider = provider;
        self
    }

    /// Enable or disable orchestration
    pub fn set_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Get API key for a provider from environment
    pub fn get_api_key(&self, provider: ProviderType) -> Option<String> {
        match provider {
            ProviderType::Gemini => std::env::var("GEMINI_API_KEY").ok(),
            ProviderType::Claude => std::env::var("ANTHROPIC_API_KEY").ok(),
            ProviderType::OpenAI => std::env::var("OPENAI_API_KEY").ok(),
            ProviderType::PromptBased => None, // No API key needed for prompt-based
        }
    }
}

/// Provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    /// Gemini provider
    Gemini,
    /// Claude provider
    Claude,
    /// OpenAI provider
    OpenAI,
    /// Prompt-based provider
    PromptBased,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gemini => write!(f, "gemini"),
            Self::Claude => write!(f, "claude"),
            Self::OpenAI => write!(f, "openai"),
            Self::PromptBased => write!(f, "prompt_based"),
        }
    }
}

/// Gemini provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    /// Model to use
    pub model: String,
    /// Temperature (0.0-1.0)
    pub temperature: f32,
    /// Maximum tool execution iterations
    pub max_tool_iterations: usize,
    /// API endpoint (optional override)
    pub api_endpoint: Option<String>,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            model: "gemini-2.0-flash-thinking-exp".to_string(),
            temperature: 0.7,
            max_tool_iterations: 5,
            api_endpoint: None,
        }
    }
}

/// Claude provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    /// Model to use
    pub model: String,
    /// Temperature (0.0-1.0)
    pub temperature: f32,
    /// Maximum tool execution iterations
    pub max_tool_iterations: usize,
    /// Maximum output tokens
    pub max_tokens: u32,
    /// API endpoint (optional override)
    pub api_endpoint: Option<String>,
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            model: "claude-3-5-sonnet-20241022".to_string(),
            temperature: 0.7,
            max_tool_iterations: 5,
            max_tokens: 4096,
            api_endpoint: None,
        }
    }
}

/// OpenAI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    /// Model to use
    pub model: String,
    /// Temperature (0.0-1.0)
    pub temperature: f32,
    /// Maximum tool execution iterations
    pub max_tool_iterations: usize,
    /// API endpoint (optional override)
    pub api_endpoint: Option<String>,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4-turbo-preview".to_string(),
            temperature: 0.7,
            max_tool_iterations: 5,
            api_endpoint: None,
        }
    }
}

/// Prompt-based provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptBasedConfig {
    /// Temperature (0.0-1.0)
    pub temperature: f32,
    /// Maximum tool execution iterations
    pub max_tool_iterations: usize,
}

impl Default for PromptBasedConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tool_iterations: 5,
        }
    }
}

/// Fallback configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    /// Enable automatic fallback to other providers
    pub enabled: bool,
    /// Fallback chain (order of providers to try)
    pub chain: Vec<ProviderType>,
    /// Maximum retries per provider
    pub max_retries: usize,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            chain: vec![
                ProviderType::Gemini,
                ProviderType::Claude,
                ProviderType::OpenAI,
                ProviderType::PromptBased,
            ],
            max_retries: 2,
        }
    }
}

/// Configuration builder for fluent API
pub struct ConfigBuilder {
    config: OrchestrationConfig,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: OrchestrationConfig::default(),
        }
    }

    /// Set default provider
    pub fn default_provider(mut self, provider: ProviderType) -> Self {
        self.config.default_provider = provider;
        self
    }

    /// Enable/disable orchestration
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Configure Gemini provider
    pub fn gemini_model(mut self, model: impl Into<String>) -> Self {
        self.config.gemini.model = model.into();
        self
    }

    /// Configure Claude provider
    pub fn claude_model(mut self, model: impl Into<String>) -> Self {
        self.config.claude.model = model.into();
        self
    }

    /// Configure OpenAI provider
    pub fn openai_model(mut self, model: impl Into<String>) -> Self {
        self.config.openai.model = model.into();
        self
    }

    /// Set temperature for all providers
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.config.gemini.temperature = temperature;
        self.config.claude.temperature = temperature;
        self.config.openai.temperature = temperature;
        self.config.prompt_based.temperature = temperature;
        self
    }

    /// Set max tool iterations for all providers
    pub fn max_tool_iterations(mut self, iterations: usize) -> Self {
        self.config.gemini.max_tool_iterations = iterations;
        self.config.claude.max_tool_iterations = iterations;
        self.config.openai.max_tool_iterations = iterations;
        self.config.prompt_based.max_tool_iterations = iterations;
        self
    }

    /// Enable/disable fallback
    pub fn fallback_enabled(mut self, enabled: bool) -> Self {
        self.config.fallback.enabled = enabled;
        self
    }

    /// Set fallback chain
    pub fn fallback_chain(mut self, chain: Vec<ProviderType>) -> Self {
        self.config.fallback.chain = chain;
        self
    }

    /// Build the configuration
    pub fn build(self) -> OrchestrationConfig {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OrchestrationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.default_provider, ProviderType::Gemini);
        assert_eq!(config.gemini.model, "gemini-2.0-flash-thinking-exp");
        assert_eq!(config.claude.model, "claude-3-5-sonnet-20241022");
        assert_eq!(config.openai.model, "gpt-4-turbo-preview");
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .default_provider(ProviderType::Claude)
            .temperature(0.9)
            .max_tool_iterations(10)
            .build();

        assert_eq!(config.default_provider, ProviderType::Claude);
        assert!((config.gemini.temperature - 0.9).abs() < f32::EPSILON);
        assert_eq!(config.gemini.max_tool_iterations, 10);
    }

    #[test]
    fn test_with_provider() {
        let config = OrchestrationConfig::new().with_provider(ProviderType::OpenAI);
        assert_eq!(config.default_provider, ProviderType::OpenAI);
    }

    #[test]
    fn test_set_enabled() {
        let config = OrchestrationConfig::new().set_enabled(false);
        assert!(!config.enabled);
    }

    #[test]
    fn test_provider_type_display() {
        assert_eq!(ProviderType::Gemini.to_string(), "gemini");
        assert_eq!(ProviderType::Claude.to_string(), "claude");
        assert_eq!(ProviderType::OpenAI.to_string(), "openai");
        assert_eq!(ProviderType::PromptBased.to_string(), "prompt_based");
    }

    #[test]
    fn test_fallback_config() {
        let config = FallbackConfig::default();
        assert!(config.enabled);
        assert_eq!(config.chain.len(), 4);
        assert_eq!(config.chain[0], ProviderType::Gemini);
        assert_eq!(config.max_retries, 2);
    }

    #[test]
    fn test_gemini_config_defaults() {
        let config = GeminiConfig::default();
        assert_eq!(config.model, "gemini-2.0-flash-thinking-exp");
        assert!((config.temperature - 0.7).abs() < f32::EPSILON);
        assert_eq!(config.max_tool_iterations, 5);
        assert!(config.api_endpoint.is_none());
    }

    #[test]
    fn test_claude_config_defaults() {
        let config = ClaudeConfig::default();
        assert_eq!(config.model, "claude-3-5-sonnet-20241022");
        assert_eq!(config.max_tokens, 4096);
        assert!((config.temperature - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_builder_fluent_api() {
        let config = ConfigBuilder::new()
            .enabled(false)
            .gemini_model("custom-model")
            .fallback_enabled(false)
            .build();

        assert!(!config.enabled);
        assert_eq!(config.gemini.model, "custom-model");
        assert!(!config.fallback.enabled);
    }
}

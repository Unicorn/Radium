//! Agent component schema
//!
//! The Agent component invokes AI models (Claude, GPT, etc.) for intelligent processing.
//! Supports multiple providers, tool calling, and streaming.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// AI providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AIProvider {
    /// Anthropic (Claude)
    #[default]
    Anthropic,
    /// OpenAI (GPT)
    OpenAI,
    /// Google (Gemini)
    Google,
    /// Azure OpenAI
    Azure,
    /// AWS Bedrock
    Bedrock,
    /// Custom endpoint
    Custom,
}

impl AIProvider {
    /// Convert to TypeScript representation
    pub fn to_typescript(&self) -> &'static str {
        match self {
            AIProvider::Anthropic => "'anthropic'",
            AIProvider::OpenAI => "'openai'",
            AIProvider::Google => "'google'",
            AIProvider::Azure => "'azure'",
            AIProvider::Bedrock => "'bedrock'",
            AIProvider::Custom => "'custom'",
        }
    }
}

/// Anthropic model variants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum AnthropicModel {
    /// Claude 3.5 Sonnet
    #[serde(rename = "claude-3-5-sonnet-20241022")]
    #[default]
    Claude35Sonnet,
    /// Claude 3 Opus
    #[serde(rename = "claude-3-opus-20240229")]
    Claude3Opus,
    /// Claude 3 Haiku
    #[serde(rename = "claude-3-haiku-20240307")]
    Claude3Haiku,
    /// Claude 3.5 Haiku
    #[serde(rename = "claude-3-5-haiku-20241022")]
    Claude35Haiku,
}

impl AnthropicModel {
    /// Get the model ID string
    pub fn model_id(&self) -> &'static str {
        match self {
            AnthropicModel::Claude35Sonnet => "claude-3-5-sonnet-20241022",
            AnthropicModel::Claude3Opus => "claude-3-opus-20240229",
            AnthropicModel::Claude3Haiku => "claude-3-haiku-20240307",
            AnthropicModel::Claude35Haiku => "claude-3-5-haiku-20241022",
        }
    }
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub enum ModelConfig {
    /// Anthropic model config
    Anthropic {
        model: AnthropicModel,
        #[serde(default = "default_max_tokens")]
        max_tokens: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        temperature: Option<f32>,
    },
    /// OpenAI model config
    OpenAI {
        model: String,
        #[serde(default = "default_max_tokens")]
        max_tokens: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        temperature: Option<f32>,
    },
    /// Custom model config
    Custom {
        endpoint: String,
        model: String,
        #[serde(default)]
        config: HashMap<String, serde_json::Value>,
    },
}

fn default_max_tokens() -> u32 {
    4096
}

impl ModelConfig {
    /// Create Anthropic config with Claude 3.5 Sonnet
    pub fn claude_sonnet() -> Self {
        ModelConfig::Anthropic {
            model: AnthropicModel::Claude35Sonnet,
            max_tokens: default_max_tokens(),
            temperature: None,
        }
    }

    /// Create Anthropic config with Claude 3 Opus
    pub fn claude_opus() -> Self {
        ModelConfig::Anthropic {
            model: AnthropicModel::Claude3Opus,
            max_tokens: default_max_tokens(),
            temperature: None,
        }
    }

    /// Create Anthropic config with Claude 3 Haiku
    pub fn claude_haiku() -> Self {
        ModelConfig::Anthropic {
            model: AnthropicModel::Claude3Haiku,
            max_tokens: default_max_tokens(),
            temperature: None,
        }
    }

    /// Create OpenAI config with GPT-4
    pub fn gpt4() -> Self {
        ModelConfig::OpenAI {
            model: "gpt-4-turbo-preview".to_string(),
            max_tokens: default_max_tokens(),
            temperature: None,
        }
    }

    /// Create OpenAI config with GPT-3.5
    pub fn gpt35() -> Self {
        ModelConfig::OpenAI {
            model: "gpt-3.5-turbo".to_string(),
            max_tokens: default_max_tokens(),
            temperature: None,
        }
    }

    /// Get provider type
    pub fn provider(&self) -> AIProvider {
        match self {
            ModelConfig::Anthropic { .. } => AIProvider::Anthropic,
            ModelConfig::OpenAI { .. } => AIProvider::OpenAI,
            ModelConfig::Custom { .. } => AIProvider::Custom,
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self::claude_sonnet()
    }
}

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message
    System,
    /// User message
    User,
    /// Assistant message
    Assistant,
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// Message role
    pub role: MessageRole,

    /// Message content
    pub content: String,
}

impl Message {
    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }
}

/// Tool definition for the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Input schema (JSON Schema)
    pub input_schema: serde_json::Value,
}

impl Tool {
    /// Create a new tool
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

/// Agent component input
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AgentInput {
    /// Model configuration
    #[serde(default)]
    pub model_config: ModelConfig,

    /// System prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,

    /// Messages
    #[validate(length(min = 1, message = "At least one message required"))]
    pub messages: Vec<Message>,

    /// Available tools
    #[serde(default)]
    pub tools: Vec<Tool>,

    /// Whether to stream response
    #[serde(default)]
    pub stream: bool,

    /// Timeout in milliseconds
    #[serde(default = "default_agent_timeout")]
    pub timeout_ms: u64,

    /// Variable to store response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_variable: Option<String>,

    /// Stop sequences
    #[serde(default)]
    pub stop_sequences: Vec<String>,
}

fn default_agent_timeout() -> u64 {
    120000 // 2 minutes
}

impl AgentInput {
    /// Create a new agent input with a single user message
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            model_config: ModelConfig::default(),
            system_prompt: None,
            messages: vec![Message::user(message)],
            tools: Vec::new(),
            stream: false,
            timeout_ms: default_agent_timeout(),
            output_variable: None,
            stop_sequences: Vec::new(),
        }
    }

    /// Set model configuration
    pub fn with_model(mut self, config: ModelConfig) -> Self {
        self.model_config = config;
        self
    }

    /// Set system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Add a message
    pub fn add_message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    /// Add a tool
    pub fn add_tool(mut self, tool: Tool) -> Self {
        self.tools.push(tool);
        self
    }

    /// Enable streaming
    pub fn stream(mut self) -> Self {
        self.stream = true;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Set output variable
    pub fn with_output_variable(mut self, variable: impl Into<String>) -> Self {
        self.output_variable = Some(variable.into());
        self
    }
}

impl Default for AgentInput {
    fn default() -> Self {
        Self::new("Hello")
    }
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    /// Input tokens
    pub input_tokens: u32,

    /// Output tokens
    pub output_tokens: u32,

    /// Total tokens
    pub total_tokens: u32,
}

impl TokenUsage {
    /// Create new token usage
    pub fn new(input: u32, output: u32) -> Self {
        Self {
            input_tokens: input,
            output_tokens: output,
            total_tokens: input + output,
        }
    }
}

/// Tool call made by the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,

    /// Tool name
    pub name: String,

    /// Tool input
    pub input: serde_json::Value,
}

impl ToolCall {
    /// Create a new tool call
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        input: serde_json::Value,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            input,
        }
    }
}

/// Reason the model stopped generating
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Normal completion
    #[default]
    EndTurn,
    /// Hit max tokens
    MaxTokens,
    /// Hit stop sequence
    StopSequence,
    /// Model wants to use tools
    ToolUse,
    /// Error occurred
    Error,
}

/// Agent component output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentOutput {
    /// Response text
    pub response: String,

    /// Model used
    pub model: String,

    /// Provider used
    pub provider: String,

    /// Token usage
    pub usage: TokenUsage,

    /// Tool calls made
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,

    /// Why the model stopped
    pub finish_reason: FinishReason,

    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl AgentOutput {
    /// Create a successful completion
    pub fn success(
        response: impl Into<String>,
        model: impl Into<String>,
        provider: impl Into<String>,
        usage: TokenUsage,
        duration_ms: u64,
    ) -> Self {
        Self {
            response: response.into(),
            model: model.into(),
            provider: provider.into(),
            usage,
            tool_calls: Vec::new(),
            finish_reason: FinishReason::EndTurn,
            duration_ms,
        }
    }

    /// Create a tool use response
    pub fn tool_use(
        tool_calls: Vec<ToolCall>,
        model: impl Into<String>,
        provider: impl Into<String>,
        usage: TokenUsage,
        duration_ms: u64,
    ) -> Self {
        Self {
            response: String::new(),
            model: model.into(),
            provider: provider.into(),
            usage,
            tool_calls,
            finish_reason: FinishReason::ToolUse,
            duration_ms,
        }
    }
}

impl Default for AgentOutput {
    fn default() -> Self {
        Self::success(
            "",
            "claude-3-5-sonnet-20241022",
            "anthropic",
            TokenUsage::default(),
            0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_provider_serialization() {
        assert_eq!(
            serde_json::to_string(&AIProvider::Anthropic).unwrap(),
            "\"anthropic\""
        );
        assert_eq!(
            serde_json::to_string(&AIProvider::OpenAI).unwrap(),
            "\"openai\""
        );
    }

    #[test]
    fn test_anthropic_model() {
        let model = AnthropicModel::Claude35Sonnet;
        assert_eq!(model.model_id(), "claude-3-5-sonnet-20241022");
    }

    #[test]
    fn test_model_config_anthropic() {
        let config = ModelConfig::claude_sonnet();
        assert_eq!(config.provider(), AIProvider::Anthropic);
    }

    #[test]
    fn test_model_config_openai() {
        let config = ModelConfig::gpt4();
        assert_eq!(config.provider(), AIProvider::OpenAI);
    }

    #[test]
    fn test_message_creation() {
        let system = Message::system("You are a helpful assistant.");
        assert_eq!(system.role, MessageRole::System);

        let user = Message::user("Hello!");
        assert_eq!(user.role, MessageRole::User);

        let assistant = Message::assistant("Hi there!");
        assert_eq!(assistant.role, MessageRole::Assistant);
    }

    #[test]
    fn test_tool_creation() {
        let tool = Tool::new(
            "get_weather",
            "Get the current weather",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                },
                "required": ["location"]
            }),
        );

        assert_eq!(tool.name, "get_weather");
    }

    #[test]
    fn test_agent_input() {
        let input = AgentInput::new("What is the weather in Paris?")
            .with_model(ModelConfig::claude_sonnet())
            .with_system_prompt("You are a weather assistant.");

        assert_eq!(input.messages.len(), 1);
        assert!(input.system_prompt.is_some());
    }

    #[test]
    fn test_agent_input_with_tools() {
        let tool = Tool::new(
            "search",
            "Search the web",
            serde_json::json!({"type": "object"}),
        );
        let input = AgentInput::new("Search for Rust tutorials")
            .add_tool(tool)
            .stream();

        assert_eq!(input.tools.len(), 1);
        assert!(input.stream);
    }

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage::new(100, 50);
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_tool_call() {
        let call = ToolCall::new(
            "call_123",
            "get_weather",
            serde_json::json!({"location": "Paris"}),
        );

        assert_eq!(call.name, "get_weather");
    }

    #[test]
    fn test_agent_output_success() {
        let output = AgentOutput::success(
            "The weather in Paris is sunny.",
            "claude-3-5-sonnet-20241022",
            "anthropic",
            TokenUsage::new(50, 20),
            1500,
        );

        assert!(!output.response.is_empty());
        assert_eq!(output.finish_reason, FinishReason::EndTurn);
    }

    #[test]
    fn test_agent_output_tool_use() {
        let calls = vec![ToolCall::new(
            "call_1",
            "search",
            serde_json::json!({"query": "rust"}),
        )];
        let output = AgentOutput::tool_use(
            calls,
            "claude-3-5-sonnet-20241022",
            "anthropic",
            TokenUsage::new(30, 10),
            500,
        );

        assert_eq!(output.finish_reason, FinishReason::ToolUse);
        assert_eq!(output.tool_calls.len(), 1);
    }

    #[test]
    fn test_serialization() {
        let input = AgentInput::new("Hello")
            .with_model(ModelConfig::claude_haiku())
            .with_timeout(30000);

        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("modelConfig"));
        assert!(json.contains("messages"));
    }
}

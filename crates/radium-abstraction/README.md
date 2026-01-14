# radium-abstraction

Core trait definitions and abstractions for the Radium platform.

## Overview

`radium-abstraction` provides the foundational traits, types, and interfaces that define how components interact in the Radium ecosystem. It serves as the contract layer that enables loose coupling and extensibility across the entire platform.

## Purpose

This crate exists to:

1. **Define Core Interfaces**: Establish the contracts that all implementations must follow
2. **Enable Extensibility**: Allow custom implementations without modifying core code
3. **Avoid Circular Dependencies**: Provide shared abstractions that other crates depend on
4. **Ensure Type Safety**: Leverage Rust's type system for compile-time guarantees
5. **Support Testing**: Enable easy mocking and testing with trait objects

## Core Abstractions

### Model Trait

The `Model` trait defines the interface for AI model providers:

```rust
#[async_trait]
pub trait Model: Send + Sync {
    /// Get the model's unique identifier
    fn id(&self) -> &str;

    /// Get the model's name
    fn name(&self) -> &str;

    /// Generate a completion from the model
    async fn complete(
        &self,
        messages: &[ChatMessage],
        params: &ModelParameters,
    ) -> Result<ModelResponse, ModelError>;
}
```

**Implementations:**
- `GeminiModel` - Google Gemini models
- `ClaudeModel` - Anthropic Claude models
- `OpenAIModel` - OpenAI GPT models
- `OllamaModel` - Local models via Ollama
- `UniversalModel` - Universal API adapter
- `MockModel` - Testing implementation

### StreamingModel Trait

For models that support streaming responses:

```rust
#[async_trait]
pub trait StreamingModel: Model {
    /// Stream completion chunks
    async fn stream(
        &self,
        messages: &[ChatMessage],
        params: &ModelParameters,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ModelChunk, ModelError>> + Send>>, ModelError>;
}
```

### Tool Trait

Defines executable tools that agents can use:

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name
    fn name(&self) -> &str;

    /// Tool description
    fn description(&self) -> &str;

    /// JSON schema for parameters
    fn parameters_schema(&self) -> serde_json::Value;

    /// Execute the tool
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult, ToolError>;
}
```

### ToolProvider Trait

Manages collections of tools:

```rust
#[async_trait]
pub trait ToolProvider: Send + Sync {
    /// Get all available tools
    async fn get_tools(&self) -> Result<Vec<Box<dyn Tool>>, ToolError>;

    /// Get a specific tool by name
    async fn get_tool(&self, name: &str) -> Result<Option<Box<dyn Tool>>, ToolError>;

    /// Execute a tool
    async fn execute_tool(
        &self,
        name: &str,
        params: serde_json::Value,
    ) -> Result<ToolResult, ToolError>;
}
```

### BatchProcessor Trait

For processing batches of requests efficiently:

```rust
#[async_trait]
pub trait BatchProcessor: Send + Sync {
    type Input;
    type Output;
    type Error;

    /// Process a batch of inputs
    async fn process_batch(
        &self,
        inputs: Vec<Self::Input>,
        config: &BatchConfig,
    ) -> Vec<Result<Self::Output, Self::Error>>;
}
```

## Type Definitions

### ChatMessage

Represents a message in a conversation:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub name: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}
```

### ModelParameters

Configuration for model inference:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParameters {
    pub temperature: Option<f64>,
    pub max_tokens: Option<usize>,
    pub top_p: Option<f64>,
    pub top_k: Option<usize>,
    pub stop_sequences: Option<Vec<String>>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
```

### ModelResponse

Response from a model:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub content: String,
    pub finish_reason: FinishReason,
    pub usage: ModelUsage,
    pub model: String,
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Error,
}
```

### ModelUsage

Token usage statistics:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub cached_tokens: Option<usize>,
}
```

### ToolCall

Represents a function call from the model:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}
```

### ToolResult

Result from tool execution:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

## Error Types

### ModelError

Errors that can occur during model operations:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Rate limit exceeded")]
    RateLimitError,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Timeout after {0}s")]
    Timeout(u64),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
```

### ToolError

Errors during tool execution:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Tool error: {0}")]
    ToolError(String),
}
```

## Usage Examples

### Implementing a Custom Model

```rust
use radium_abstraction::{Model, ModelError, ModelResponse, ChatMessage, ModelParameters};
use async_trait::async_trait;

pub struct CustomModel {
    id: String,
    name: String,
}

#[async_trait]
impl Model for CustomModel {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn complete(
        &self,
        messages: &[ChatMessage],
        params: &ModelParameters,
    ) -> Result<ModelResponse, ModelError> {
        // Your implementation here
        todo!()
    }
}
```

### Implementing a Custom Tool

```rust
use radium_abstraction::{Tool, ToolResult, ToolError};
use async_trait::async_trait;

pub struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Performs basic arithmetic operations"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"]
                },
                "a": { "type": "number" },
                "b": { "type": "number" }
            },
            "required": ["operation", "a", "b"]
        })
    }

    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult, ToolError> {
        // Parse parameters
        let op = params["operation"].as_str().ok_or_else(|| {
            ToolError::InvalidParameters("Missing operation".to_string())
        })?;

        let a = params["a"].as_f64().ok_or_else(|| {
            ToolError::InvalidParameters("Invalid parameter a".to_string())
        })?;

        let b = params["b"].as_f64().ok_or_else(|| {
            ToolError::InvalidParameters("Invalid parameter b".to_string())
        })?;

        // Perform operation
        let result = match op {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err(ToolError::ExecutionFailed("Division by zero".to_string()));
                }
                a / b
            }
            _ => return Err(ToolError::InvalidParameters(format!("Unknown operation: {}", op))),
        };

        Ok(ToolResult {
            success: true,
            output: result.to_string(),
            error: None,
            metadata: Default::default(),
        })
    }
}
```

### Using Trait Objects

```rust
use radium_abstraction::{Model, ChatMessage, ModelParameters};

async fn call_any_model(
    model: &dyn Model,
    messages: Vec<ChatMessage>,
) -> Result<String, Box<dyn std::error::Error>> {
    let params = ModelParameters::default();
    let response = model.complete(&messages, &params).await?;
    Ok(response.content)
}
```

## Design Principles

1. **Async First**: All I/O operations are async for maximum concurrency
2. **Send + Sync**: All traits require Send + Sync for thread safety
3. **Error Handling**: Use Result types and custom error enums
4. **Serialization**: Support serde for all data types
5. **Extensibility**: Use `#[serde(flatten)]` for custom fields
6. **Documentation**: Comprehensive doc comments on all public items

## Batch Processing

The `BatchProcessor` trait enables efficient batch processing:

```rust
use radium_abstraction::batch::{BatchProcessor, BatchConfig};

struct ModelBatchProcessor {
    model: Box<dyn Model>,
}

#[async_trait]
impl BatchProcessor for ModelBatchProcessor {
    type Input = Vec<ChatMessage>;
    type Output = ModelResponse;
    type Error = ModelError;

    async fn process_batch(
        &self,
        inputs: Vec<Self::Input>,
        config: &BatchConfig,
    ) -> Vec<Result<Self::Output, Self::Error>> {
        // Process requests in parallel with rate limiting
        futures::stream::iter(inputs)
            .map(|messages| async {
                self.model.complete(&messages, &Default::default()).await
            })
            .buffer_unordered(config.max_concurrent)
            .collect()
            .await
    }
}
```

## Testing Support

The crate provides test utilities:

```rust
#[cfg(test)]
mod tests {
    use radium_abstraction::{Model, MockModel};

    #[tokio::test]
    async fn test_with_mock_model() {
        let model = MockModel::new("test-model".to_string());

        // MockModel returns predictable responses
        let response = model.complete(&[], &Default::default()).await.unwrap();
        assert!(!response.content.is_empty());
    }
}
```

## Dependencies

This crate has minimal dependencies to avoid bloat:

- **async-trait** - Async trait support
- **serde** - Serialization framework
- **thiserror** - Error derive macros
- **futures** - Async utilities

No runtime dependencies like tokio are included.

## Versioning

This crate follows semantic versioning:
- MAJOR: Breaking changes to trait signatures
- MINOR: New traits or methods added
- PATCH: Bug fixes, documentation updates

## Contributing

When adding new traits:
1. Define clear contracts with comprehensive documentation
2. Include usage examples in doc comments
3. Add corresponding error types
4. Ensure Send + Sync bounds where appropriate
5. Add tests demonstrating usage

## License

MIT - see [LICENSE](../../LICENSE) for details

## Links

- [Architecture Overview](../../website/docs/developer-guide/architecture/)
- [API Documentation](https://docs.rs/radium-abstraction)
- [Contributing Guide](../../CONTRIBUTING.md)

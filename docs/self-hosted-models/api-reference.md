# Model Trait and Provider Abstraction API Reference

## Overview

This document provides a complete reference for Radium's model abstraction layer, enabling developers to understand how to implement custom model providers or extend existing ones. The abstraction consists of the `Model` trait, `ModelFactory`, and related types defined in the `radium-abstraction` and `radium-models` crates.

## Model Trait

The `Model` trait is the core interface for all AI model implementations in Radium. It provides a unified API for text generation and chat completions.

### Trait Definition

```rust
#[async_trait]
pub trait Model: Send + Sync {
    async fn generate_text(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError>;

    async fn generate_chat_completion(
        &self,
        messages: &[ChatMessage],
        parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError>;

    fn model_id(&self) -> &str;
}
```

### Method: `generate_text`

Generates a text completion based on a single prompt string.

**Parameters:**
- `prompt: &str` - The input prompt for text generation
- `parameters: Option<ModelParameters>` - Optional parameters to control generation (temperature, max_tokens, etc.)

**Returns:**
- `Result<ModelResponse, ModelError>` - The generated response or an error

**Example:**
```rust
use radium_abstraction::{Model, ModelParameters};

let response = model.generate_text(
    "Write a haiku about programming",
    Some(ModelParameters {
        temperature: Some(0.7),
        max_tokens: Some(100),
        ..Default::default()
    })
).await?;

println!("{}", response.content);
```

### Method: `generate_chat_completion`

Generates a chat completion based on a conversation history.

**Parameters:**
- `messages: &[ChatMessage]` - The conversation history as a slice of chat messages
- `parameters: Option<ModelParameters>` - Optional parameters to control generation

**Returns:**
- `Result<ModelResponse, ModelError>` - The generated response or an error

**Example:**
```rust
use radium_abstraction::{Model, ChatMessage, ModelParameters};

let messages = vec![
    ChatMessage {
        role: "system".to_string(),
        content: "You are a helpful assistant.".to_string(),
    },
    ChatMessage {
        role: "user".to_string(),
        content: "What is Rust?".to_string(),
    },
];

let response = model.generate_chat_completion(&messages, None).await?;
println!("{}", response.content);
```

### Method: `model_id`

Returns the identifier of the model instance.

**Returns:**
- `&str` - The model ID (e.g., "llama-3-70b", "gpt-4")

**Example:**
```rust
let id = model.model_id();
println!("Using model: {}", id);
```

## StreamingModel Trait

The `StreamingModel` trait enables real-time token-by-token streaming of model responses.

### Trait Definition

```rust
#[async_trait]
pub trait StreamingModel: Send + Sync {
    async fn generate_stream(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, ModelError>> + Send>>, ModelError>;
}
```

### Method: `generate_stream`

Generates a streaming text completion, yielding tokens as they're generated.

**Parameters:**
- `prompt: &str` - The input prompt
- `parameters: Option<ModelParameters>` - Optional generation parameters

**Returns:**
- `Result<Pin<Box<dyn Stream<Item = Result<String, ModelError>> + Send>>, ModelError>` - A stream of tokens or an error

**Example:**
```rust
use radium_abstraction::StreamingModel;
use futures::StreamExt;

let mut stream = model.generate_stream("Tell me a story", None).await?;
while let Some(result) = stream.next().await {
    match result {
        Ok(token) => print!("{}", token),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Data Types

### ChatMessage

Represents a message in a conversation.

```rust
pub struct ChatMessage {
    pub role: String,    // "user", "assistant", or "system"
    pub content: String,  // The message content
}
```

**Roles:**
- `"system"` - System instructions or context
- `"user"` - User messages
- `"assistant"` - Assistant responses

### ModelParameters

Parameters for controlling model generation.

```rust
pub struct ModelParameters {
    pub temperature: Option<f32>,        // 0.0-2.0, higher = more creative
    pub top_p: Option<f32>,             // 0.0-1.0, nucleus sampling
    pub max_tokens: Option<u32>,        // Maximum tokens to generate
    pub stop_sequences: Option<Vec<String>>, // Stop generation on these sequences
}
```

**Default Values:**
- `temperature`: `0.7`
- `top_p`: `1.0`
- `max_tokens`: `512`
- `stop_sequences`: `None`

### ModelResponse

The response from a model generation.

```rust
pub struct ModelResponse {
    pub content: String,              // The generated text
    pub model_id: Option<String>,    // The model ID used
    pub usage: Option<ModelUsage>,   // Token usage statistics
}
```

### ModelUsage

Token usage statistics for a request.

```rust
pub struct ModelUsage {
    pub prompt_tokens: u32,      // Tokens in the prompt
    pub completion_tokens: u32,  // Tokens in the completion
    pub total_tokens: u32,       // Total tokens used
}
```

### ModelError

Errors that can occur when interacting with models.

```rust
pub enum ModelError {
    RequestError(String),                    // Network or request errors
    ModelResponseError(String),              // Model returned an error
    SerializationError(String),              // JSON serialization errors
    UnsupportedModelProvider(String),        // Provider not supported
    QuotaExceeded {                          // Rate limit or quota exceeded
        provider: String,
        message: Option<String>,
    },
    Other(String),                          // Other unexpected errors
}
```

## ModelFactory

The `ModelFactory` provides a unified way to create model instances from configuration.

### Creating Models

#### `create`

Creates a model instance from a `ModelConfig`.

```rust
use radium_models::{ModelConfig, ModelFactory, ModelType};

let config = ModelConfig::new(
    ModelType::Universal,
    "llama-3-70b".to_string(),
)
.with_base_url("http://localhost:8000/v1".to_string());

let model = ModelFactory::create(config)?;
```

#### `create_from_str`

Creates a model from a string representation of the model type.

```rust
let model = ModelFactory::create_from_str(
    "universal",
    "llama-3-70b".to_string(),
)?;
```

**Supported Model Type Strings:**
- `"mock"` - Mock model for testing
- `"claude"` or `"anthropic"` - Anthropic Claude
- `"gemini"` - Google Gemini
- `"openai"` - OpenAI GPT models
- `"universal"`, `"openai-compatible"`, or `"local"` - Universal provider
- `"ollama"` - Ollama (not yet implemented in factory, use Universal)

#### `create_with_api_key`

Creates a model with an explicit API key.

```rust
let model = ModelFactory::create_with_api_key(
    "openai",
    "gpt-4".to_string(),
    "sk-...".to_string(),
)?;
```

### ModelConfig

Configuration for creating model instances.

```rust
pub struct ModelConfig {
    pub model_type: ModelType,
    pub model_id: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,  // Required for Universal models
}
```

**Builder Methods:**
- `new(model_type, model_id)` - Create a new config
- `with_api_key(api_key)` - Set the API key
- `with_base_url(base_url)` - Set the base URL (required for Universal)

### ModelType

Enumeration of supported model types.

```rust
pub enum ModelType {
    Mock,      // Testing model
    Claude,    // Anthropic Claude
    Gemini,    // Google Gemini
    OpenAI,    // OpenAI GPT
    Universal, // OpenAI-compatible (vLLM, LocalAI, Ollama, etc.)
    Ollama,    // Ollama (factory integration pending)
}
```

## UniversalModel

The `UniversalModel` is the primary way to use self-hosted models. It implements the OpenAI Chat Completions API specification.

### Constructors

#### `new`

Creates a UniversalModel, loading API key from environment variables.

```rust
use radium_models::UniversalModel;

// Loads API key from UNIVERSAL_API_KEY or OPENAI_COMPATIBLE_API_KEY
let model = UniversalModel::new(
    "llama-3-70b".to_string(),
    "http://localhost:8000/v1".to_string(),
)?;
```

**Environment Variables:**
- `UNIVERSAL_API_KEY` (primary)
- `OPENAI_COMPATIBLE_API_KEY` (fallback)

#### `with_api_key`

Creates a UniversalModel with an explicit API key.

```rust
let model = UniversalModel::with_api_key(
    "llama-3-70b".to_string(),
    "http://localhost:8000/v1".to_string(),
    "optional-api-key".to_string(),
);
```

#### `without_auth`

Creates a UniversalModel without authentication (most common for local servers).

```rust
let model = UniversalModel::without_auth(
    "llama-3-70b".to_string(),
    "http://localhost:8000/v1".to_string(),
);
```

### Supported Servers

UniversalModel works with any server implementing the OpenAI Chat Completions API:

- **vLLM**: `http://localhost:8000/v1`
- **LocalAI**: `http://localhost:8080/v1`
- **Ollama**: `http://localhost:11434/v1` (OpenAI-compatible endpoint)
- **LM Studio**: `http://localhost:1234/v1`
- **Any OpenAI-compatible server**

## Ollama Implementation Status

### Current State

A native `OllamaModel` implementation exists in `crates/radium-models/src/ollama.rs`, but it is not yet integrated into the `ModelFactory`. The factory returns an error when attempting to create an Ollama model:

```rust
ModelType::Ollama => {
    Err(ModelError::UnsupportedModelProvider(
        "Ollama model type is not yet implemented. Use UniversalModel with base_url 'http://localhost:11434/v1' instead.".to_string(),
    ))
}
```

### Recommended Approach

Use `UniversalModel` with Ollama's OpenAI-compatible endpoint:

```rust
use radium_models::UniversalModel;

let model = UniversalModel::without_auth(
    "llama3.2".to_string(),
    "http://localhost:11434/v1".to_string(),
);
```

This works because Ollama provides an OpenAI-compatible API endpoint at `/v1/chat/completions`.

## Implementing a Custom Provider

To implement a custom model provider, you need to:

1. **Implement the `Model` trait:**
```rust
use async_trait::async_trait;
use radium_abstraction::{Model, ModelError, ModelParameters, ModelResponse, ChatMessage};

#[derive(Debug)]
pub struct CustomModel {
    model_id: String,
    // ... your fields
}

#[async_trait]
impl Model for CustomModel {
    async fn generate_text(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        // Your implementation
    }

    async fn generate_chat_completion(
        &self,
        messages: &[ChatMessage],
        parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        // Your implementation
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}
```

2. **Optionally implement `StreamingModel`** for streaming support

3. **Handle errors appropriately** using `ModelError` variants

4. **Return proper `ModelResponse`** with content, model_id, and usage statistics

### Reference Implementations

- **MockModel**: `crates/radium-models/src/lib.rs` - Simple testing implementation
- **OpenAIModel**: `crates/radium-models/src/openai.rs` - HTTP client pattern
- **UniversalModel**: `crates/radium-models/src/universal.rs` - OpenAI-compatible API pattern
- **OllamaModel**: `crates/radium-models/src/ollama.rs` - Custom API pattern

## Error Handling

### Common Error Patterns

**Connection Errors:**
```rust
if e.is_connect() {
    ModelError::RequestError(format!(
        "Server not reachable at {}. Is it running?",
        base_url
    ))
}
```

**API Errors:**
```rust
if !status.is_success() {
    ModelError::ModelResponseError(format!(
        "API returned error: {}",
        status
    ))
}
```

**Quota/Rate Limits:**
```rust
if status == 429 {
    ModelError::QuotaExceeded {
        provider: "custom".to_string(),
        message: Some("Rate limit exceeded".to_string()),
    }
}
```

## Best Practices

1. **Use `UniversalModel` for self-hosted models** - It's the simplest and most compatible approach
2. **Handle errors gracefully** - Provide user-friendly error messages
3. **Include usage statistics** - Return `ModelUsage` when available
4. **Support streaming** - Implement `StreamingModel` for better UX
5. **Test thoroughly** - Use `MockModel` for testing your integration
6. **Document your provider** - Add setup guides and examples

## Related Documentation

- [Universal Provider Guide](../universal-provider-guide.md) - Detailed UniversalModel usage
- [Setup Guides](setup/) - Provider-specific setup instructions
- [Configuration Guide](configuration/agent-config.md) - Agent configuration examples
- [Source Code](../../crates/radium-abstraction/src/lib.rs) - Model trait definition
- [Factory Source Code](../../crates/radium-models/src/factory.rs) - ModelFactory implementation


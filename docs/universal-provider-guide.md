# Universal OpenAI-Compatible Provider Guide

## Introduction

The Universal provider enables Radium to work with any server that implements the OpenAI Chat Completions API specification. This includes local inference servers like vLLM, LocalAI, LM Studio, and Ollama, allowing you to:

- Run models locally for privacy and cost savings
- Use self-hosted inference servers
- Experiment with open-source models
- Avoid vendor lock-in

## Quick Start

The simplest way to get started is with a local server that doesn't require authentication:

```rust
use radium_models::UniversalModel;

let model = UniversalModel::without_auth(
    "llama-2-7b".to_string(),
    "http://localhost:1234/v1".to_string(),
);

let messages = vec![radium_abstraction::ChatMessage {
    role: "user".to_string(),
    content: "Say hello".to_string(),
}];

let response = model.generate_chat_completion(&messages, None).await?;
println!("{}", response.content);
```

## Supported Servers

### vLLM

vLLM is a high-performance LLM inference server with OpenAI-compatible API.

#### Installation

```bash
pip install vllm
```

#### Starting the Server

```bash
vllm serve meta-llama/Llama-3-8B-Instruct --port 8000
```

#### Usage

```rust
use radium_models::UniversalModel;

// With API key (if configured)
let model = UniversalModel::with_api_key(
    "meta-llama/Llama-3-8B-Instruct".to_string(),
    "http://localhost:8000/v1".to_string(),
    "optional-api-key".to_string(),
);

// Without authentication (default)
let model = UniversalModel::without_auth(
    "meta-llama/Llama-3-8B-Instruct".to_string(),
    "http://localhost:8000/v1".to_string(),
);
```

### LocalAI

LocalAI is a local inference server that can run various models with an OpenAI-compatible API.

#### Installation (Docker)

```bash
docker run -p 8080:8080 localai/localai
```

#### Configuration

LocalAI requires model configuration files. See the [LocalAI documentation](https://localai.io/) for details.

#### Usage

```rust
use radium_models::UniversalModel;

// LocalAI may require authentication
let model = UniversalModel::with_api_key(
    "gpt-3.5-turbo".to_string(),
    "http://localhost:8080/v1".to_string(),
    "local-api-key".to_string(),
);
```

### LM Studio

LM Studio is a desktop application for running local models with an easy-to-use interface.

#### Installation

1. Download LM Studio from [lmstudio.ai](https://lmstudio.ai/)
2. Install and launch the application
3. Download a model via the UI
4. Start the local server in Settings → Local Server

#### Usage

```rust
use radium_models::UniversalModel;

// LM Studio typically doesn't require authentication
let model = UniversalModel::without_auth(
    "llama-2-7b".to_string(),
    "http://localhost:1234/v1".to_string(),
);
```

### Ollama

Ollama is a local model runner with OpenAI-compatible endpoints.

#### Installation

```bash
curl -fsSL https://ollama.com/install.sh | sh
```

#### Pulling Models

```bash
ollama pull llama2
```

#### Usage

```rust
use radium_models::UniversalModel;

let model = UniversalModel::without_auth(
    "llama2".to_string(),
    "http://localhost:11434/v1".to_string(),
);
```

## Factory Integration

You can also create Universal models through the `ModelFactory`:

```rust
use radium_models::{ModelConfig, ModelFactory, ModelType};

let config = ModelConfig::new(
    ModelType::Universal,
    "llama-2-7b".to_string(),
)
.with_base_url("http://localhost:1234/v1".to_string());

let model = ModelFactory::create(config)?;
```

Or using string parsing:

```rust
use radium_models::{ModelConfig, ModelFactory, ModelType};

// "universal", "openai-compatible", or "local" all work
let config = ModelConfig::new(
    ModelType::from_str("universal")?,
    "llama-2-7b".to_string(),
)
.with_base_url("http://localhost:1234/v1".to_string());

let model = ModelFactory::create(config)?;
```

## Streaming Support

The Universal provider supports Server-Sent Events (SSE) streaming for real-time token generation:

```rust
use futures::StreamExt;
use radium_models::UniversalModel;

let model = UniversalModel::without_auth(
    "llama-2-7b".to_string(),
    "http://localhost:1234/v1".to_string(),
);

let messages = vec![radium_abstraction::ChatMessage {
    role: "user".to_string(),
    content: "Tell me a story".to_string(),
}];

let mut stream = model.generate_chat_completion_stream(&messages, None).await?;

while let Some(result) = stream.next().await {
    let content = result?;
    print!("{}", content);
    // Content accumulates as tokens arrive
}
```

## Environment Variables

The Universal provider supports loading API keys from environment variables:

- `UNIVERSAL_API_KEY` - Primary environment variable for API key
- `OPENAI_COMPATIBLE_API_KEY` - Alternative environment variable name

When using `UniversalModel::new()`, the API key will be loaded from these environment variables. For servers that don't require authentication, use `without_auth()` instead.

## Troubleshooting

### Connection Refused Errors

**Problem**: `RequestError: Network error: ... connection refused`

**Solutions**:
- Verify the server is running: `curl http://localhost:8000/v1/models`
- Check the base URL includes the `/v1` path
- Ensure the port matches your server configuration
- Check firewall settings

### Authentication Failures

**Problem**: `UnsupportedModelProvider: Authentication failed (401): ...`

**Solutions**:
- Verify the API key is correct (if required)
- Check if the server requires authentication (some local servers don't)
- Use `without_auth()` for servers that don't require API keys
- Ensure the `Authorization: Bearer <key>` header is being sent (check logs)

### Timeout Issues

**Problem**: Requests timeout after 60 seconds

**Solutions**:
- The default timeout is 60 seconds
- For slower models, consider using streaming to see progress
- Check server logs for processing delays
- Verify network connectivity

### Malformed Response Errors

**Problem**: `SerializationError: Failed to parse response`

**Solutions**:
- Verify the server implements the OpenAI API specification correctly
- Check server logs for error responses
- Ensure the server is returning valid JSON
- Some servers may return non-standard formats - check compatibility

### Empty Content in Response

**Problem**: `ModelResponseError: No content in API response`

**Solutions**:
- Check if the model generated an empty response
- Verify the request format matches OpenAI specification
- Check server logs for warnings or errors
- Try a different prompt to test

### Streaming Issues

**Problem**: Streaming doesn't work or returns incomplete content

**Solutions**:
- Verify the server supports SSE streaming (check `/v1/chat/completions` with `stream: true`)
- Check that the server sends `data: [DONE]` to terminate the stream
- Some servers may not support streaming - use non-streaming `generate_chat_completion()` instead
- Check network stability for long-running streams

## Migration from OpenAI Provider

If you're currently using the OpenAI provider and want to switch to Universal:

### Before (OpenAI)

```rust
use radium_models::OpenAIModel;

let model = OpenAIModel::new("gpt-4".to_string())?;
```

### After (Universal)

```rust
use radium_models::UniversalModel;

let model = UniversalModel::new(
    "gpt-4".to_string(),
    "https://api.openai.com/v1".to_string(),
)?;
```

### Key Differences

1. **Base URL**: Universal requires explicit `base_url` parameter
2. **API Key**: Uses `UNIVERSAL_API_KEY` or `OPENAI_COMPATIBLE_API_KEY` instead of `OPENAI_API_KEY`
3. **Compatibility**: Works with any OpenAI-compatible server, not just OpenAI

### Factory Migration

```rust
// Before
let config = ModelConfig::new(ModelType::OpenAI, "gpt-4".to_string());

// After
let config = ModelConfig::new(ModelType::Universal, "gpt-4".to_string())
    .with_base_url("https://api.openai.com/v1".to_string());
```

## Best Practices

1. **Use `without_auth()` for local servers**: Most local inference servers don't require API keys
2. **Set appropriate timeouts**: The default 60s timeout works for most cases, but adjust if needed
3. **Use streaming for long responses**: Streaming provides better UX for long generations
4. **Handle errors gracefully**: Check error types and provide user-friendly messages
5. **Test server compatibility**: Not all servers implement the OpenAI spec perfectly - test your setup

## Examples

See the integration tests in `crates/radium-models/tests/universal_integration_test.rs` for complete examples with vLLM, LocalAI, and LM Studio.

## API Reference

For detailed API documentation, see the [UniversalModel documentation](../../crates/radium-models/src/universal.rs).


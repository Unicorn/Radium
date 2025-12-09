# Universal OpenAI-Compatible Provider Guide

## Introduction

The Universal OpenAI-Compatible Provider enables Radium to connect to any server that implements the OpenAI Chat Completions API specification. This includes popular local inference servers like vLLM, LocalAI, LM Studio, and Ollama, allowing you to:

- Run models locally for privacy and cost savings
- Use self-hosted inference servers
- Experiment with open-source models
- Avoid vendor lock-in

## Quick Start

The simplest way to get started is with a local server that doesn't require authentication:

```rust
use radium_models::UniversalModel;

// Connect to LM Studio (no authentication required)
let model = UniversalModel::without_auth(
    "llama-2-7b".to_string(),
    "http://localhost:1234/v1".to_string(),
);

// Generate text
let response = model.generate_text("Say hello", None).await?;
println!("{}", response.content);
```

## Supported Servers

The Universal provider works with any server implementing the OpenAI Chat Completions API, including:

- **vLLM**: High-performance LLM inference server
- **LocalAI**: Local inference server with OpenAI-compatible API
- **LM Studio**: Desktop app for running local models
- **Ollama**: Local model runner with OpenAI-compatible endpoints
- **Any custom server**: As long as it implements the OpenAI API spec

## Server Setup Guides

### vLLM

vLLM is a high-performance inference server optimized for large language models.

#### Installation

```bash
pip install vllm
```

#### Starting the Server

```bash
# Serve a model (replace with your model name)
vllm serve meta-llama/Llama-3-8B-Instruct --port 8000

# With API key authentication (optional)
vllm serve meta-llama/Llama-3-8B-Instruct --port 8000 --api-key your-api-key
```

#### Usage with Radium

```rust
use radium_models::UniversalModel;

// Without authentication
let model = UniversalModel::without_auth(
    "meta-llama/Llama-3-8B-Instruct".to_string(),
    "http://localhost:8000/v1".to_string(),
);

// With authentication
let model = UniversalModel::with_api_key(
    "meta-llama/Llama-3-8B-Instruct".to_string(),
    "http://localhost:8000/v1".to_string(),
    "your-api-key".to_string(),
);
```

#### API Endpoint

- Default: `http://localhost:8000/v1`
- Chat completions: `http://localhost:8000/v1/chat/completions`

### LocalAI

LocalAI is a drop-in replacement for OpenAI that runs locally using consumer-grade hardware.

#### Installation (Docker)

```bash
docker run -p 8080:8080 localai/localai
```

#### Configuration

LocalAI uses YAML configuration files. Create a `models.yaml` file:

```yaml
models:
  - name: gpt-3.5-turbo
    backend: llama
    parameters:
      model: /path/to/model.gguf
```

#### Usage with Radium

```rust
use radium_models::UniversalModel;

// Without authentication (default)
let model = UniversalModel::without_auth(
    "gpt-3.5-turbo".to_string(),
    "http://localhost:8080/v1".to_string(),
);

// With authentication (if configured)
let model = UniversalModel::with_api_key(
    "gpt-3.5-turbo".to_string(),
    "http://localhost:8080/v1".to_string(),
    "local-api-key".to_string(),
);
```

#### API Endpoint

- Default: `http://localhost:8080/v1`
- Chat completions: `http://localhost:8080/v1/chat/completions`

### LM Studio

LM Studio is a user-friendly desktop application for running local models.

#### Installation

1. Download LM Studio from [lmstudio.ai](https://lmstudio.ai)
2. Install and launch the application
3. Download a model through the UI

#### Enabling the Local Server

1. Open LM Studio
2. Go to Settings → Local Server
3. Enable "Local Server"
4. Note the port (default: 1234)

#### Usage with Radium

```rust
use radium_models::UniversalModel;

// LM Studio doesn't require authentication
let model = UniversalModel::without_auth(
    "llama-2-7b".to_string(),  // Use the model name from LM Studio
    "http://localhost:1234/v1".to_string(),
);

let response = model.generate_text("Say hello", None).await?;
```

#### API Endpoint

- Default: `http://localhost:1234/v1`
- Chat completions: `http://localhost:1234/v1/chat/completions`

### Ollama

Ollama is a simple tool for running large language models locally.

#### Installation

```bash
# macOS/Linux
curl -fsSL https://ollama.com/install.sh | sh

# Or download from https://ollama.com
```

#### Pulling a Model

```bash
ollama pull llama2
```

#### Usage with Radium

```rust
use radium_models::UniversalModel;

// Ollama uses OpenAI-compatible endpoints
let model = UniversalModel::without_auth(
    "llama2".to_string(),
    "http://localhost:11434/v1".to_string(),
);
```

#### API Endpoint

- Default: `http://localhost:11434/v1`
- Chat completions: `http://localhost:11434/v1/chat/completions`

## Constructor Patterns

The Universal provider supports three constructor patterns:

### 1. `new()` - Environment Variable Authentication

Loads API key from environment variables:

```rust
// Set environment variable
std::env::set_var("UNIVERSAL_API_KEY", "your-api-key");

// Or use OPENAI_COMPATIBLE_API_KEY
std::env::set_var("OPENAI_COMPATIBLE_API_KEY", "your-api-key");

let model = UniversalModel::new(
    "model-name".to_string(),
    "http://localhost:8000/v1".to_string(),
)?;
```

### 2. `with_api_key()` - Explicit API Key

Provides API key directly:

```rust
let model = UniversalModel::with_api_key(
    "model-name".to_string(),
    "http://localhost:8000/v1".to_string(),
    "your-api-key".to_string(),
);
```

### 3. `without_auth()` - No Authentication

For servers that don't require authentication:

```rust
let model = UniversalModel::without_auth(
    "model-name".to_string(),
    "http://localhost:1234/v1".to_string(),
);
```

## Environment Variables

The Universal provider supports the following environment variables:

- `UNIVERSAL_API_KEY`: Primary API key environment variable
- `OPENAI_COMPATIBLE_API_KEY`: Fallback API key environment variable
- `UNIVERSAL_BASE_URL`: Default base URL (not currently used, specify in constructor)

## Factory Integration

You can also create Universal models through the ModelFactory:

```rust
use radium_models::{ModelConfig, ModelFactory, ModelType};

let config = ModelConfig::new(
    ModelType::Universal,
    "model-name".to_string(),
)
.with_base_url("http://localhost:8000/v1".to_string())
.with_api_key("your-api-key".to_string());  // Optional

let model = ModelFactory::create(config)?;
```

## Streaming Support

The Universal provider supports Server-Sent Events (SSE) streaming:

```rust
use futures::StreamExt;
use radium_models::UniversalModel;
use radium_abstraction::ChatMessage;

let model = UniversalModel::without_auth(
    "model-name".to_string(),
    "http://localhost:8000/v1".to_string(),
);

let messages = vec![ChatMessage {
    role: "user".to_string(),
    content: "Tell me a story".to_string(),
}];

let mut stream = model.generate_chat_completion_stream(&messages, None).await?;

while let Some(result) = stream.next().await {
    let content = result?;
    print!("{}", content);
}
```

## Troubleshooting

### Connection Refused Errors

**Error**: `Network error: Connection refused`

**Solutions**:
1. Verify the server is running: `curl http://localhost:8000/v1/models`
2. Check the port number matches your server configuration
3. Ensure the base URL includes `/v1` suffix
4. Check firewall settings

### Authentication Failures

**Error**: `Authentication failed (401)`

**Solutions**:
1. Verify your API key is correct
2. Check if the server requires authentication (some don't)
3. Use `without_auth()` for servers that don't require keys
4. Check server logs for authentication errors

### Timeout Issues

**Error**: `Request timeout`

**Solutions**:
1. The default timeout is 60 seconds
2. Large models or slow hardware may need more time
3. Check server logs for processing delays
4. Consider using a faster model or hardware

### Malformed Response Errors

**Error**: `Failed to parse response`

**Solutions**:
1. Verify the server implements the OpenAI API specification correctly
2. Check server logs for error responses
3. Test the server directly with `curl`:
   ```bash
   curl http://localhost:8000/v1/chat/completions \
     -H "Content-Type: application/json" \
     -d '{"model":"test","messages":[{"role":"user","content":"hello"}]}'
   ```

### Empty or Missing Content

**Error**: `No content in API response`

**Solutions**:
1. Check that the model is loaded and ready
2. Verify the model name matches what the server expects
3. Check server logs for generation errors
4. Try a simpler prompt to test basic functionality

### Common Configuration Mistakes

1. **Missing `/v1` suffix**: Base URL should be `http://localhost:8000/v1`, not `http://localhost:8000`
2. **Wrong model name**: Use the exact model identifier the server expects
3. **Port mismatch**: Verify the port matches your server configuration
4. **Authentication when not needed**: Some servers (LM Studio, Ollama) don't require API keys

## Migration from OpenAI Provider

If you're currently using the OpenAI provider and want to switch to Universal:

### Before (OpenAI)

```rust
use radium_models::OpenAIModel;

let model = OpenAIModel::new("gpt-4".to_string())?;
```

### After (Universal with OpenAI)

```rust
use radium_models::UniversalModel;

let model = UniversalModel::with_api_key(
    "gpt-4".to_string(),
    "https://api.openai.com/v1".to_string(),
    std::env::var("OPENAI_API_KEY")?,
);
```

### Benefits

- Same API, works with any OpenAI-compatible server
- Can switch between local and cloud servers easily
- No code changes needed when switching providers

## Advanced Usage

### Custom Parameters

```rust
use radium_abstraction::ModelParameters;

let params = ModelParameters {
    temperature: Some(0.7),
    top_p: Some(0.9),
    max_tokens: Some(100),
    stop_sequences: Some(vec!["\n\n".to_string()]),
};

let response = model.generate_chat_completion(&messages, Some(params)).await?;
```

### Multiple Messages

```rust
let messages = vec![
    ChatMessage {
        role: "system".to_string(),
        content: "You are a helpful assistant".to_string(),
    },
    ChatMessage {
        role: "user".to_string(),
        content: "What is the weather?".to_string(),
    },
];

let response = model.generate_chat_completion(&messages, None).await?;
```

## Performance Tips

1. **Use streaming** for long responses to see output incrementally
2. **Batch requests** when possible to reduce overhead
3. **Choose appropriate models** - smaller models are faster but less capable
4. **Monitor server resources** - local servers are limited by your hardware
5. **Use connection pooling** - the HTTP client reuses connections automatically

## Security Considerations

1. **API Keys**: Never commit API keys to version control
2. **Local Servers**: Local servers may not have the same security as cloud providers
3. **Network**: Use HTTPS in production, HTTP is acceptable for localhost
4. **Authentication**: Enable authentication on production servers

## Examples

See the integration tests in `crates/radium-models/tests/universal_integration_test.rs` for complete examples with real servers.

## Further Reading

- [OpenAI API Reference](https://platform.openai.com/docs/api-reference/chat)
- [vLLM Documentation](https://docs.vllm.ai/)
- [LocalAI Documentation](https://localai.io/)
- [LM Studio Documentation](https://lmstudio.ai/docs)
- [Ollama Documentation](https://ollama.com/docs)

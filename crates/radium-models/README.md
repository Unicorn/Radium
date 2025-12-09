# Radium Models

Model implementations for Radium, providing a unified interface for interacting with various AI model providers.

## Supported Providers

- **Mock**: Testing and development
- **Claude**: Anthropic's Claude models (API key required)
- **Gemini**: Google's Gemini models (API key required)
- **OpenAI**: OpenAI's GPT models (API key required)
- **Ollama**: Local models via Ollama (no API key, local execution)

## Ollama Setup

Ollama enables local model execution without API costs or internet connectivity.

### Installation

1. Install Ollama:
   ```bash
   curl https://ollama.ai/install.sh | sh
   ```

2. Start Ollama server:
   ```bash
   ollama serve
   ```

3. Pull a model:
   ```bash
   ollama pull llama2
   ```

### Usage Examples

#### Non-Streaming Text Generation

```rust
use radium_models::OllamaModel;
use radium_abstraction::Model;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = OllamaModel::new("llama2".to_string())?;
    let response = model.generate_text("Hello!", None).await?;
    println!("{}", response.content);
    Ok(())
}
```

#### Streaming Text Generation

```rust
use radium_models::OllamaModel;
use radium_abstraction::StreamingModel;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = OllamaModel::new("llama2".to_string())?;
    let mut stream = model.generate_stream("Write a story", None).await?;
    
    while let Some(token) = stream.next().await {
        print!("{}", token?);
    }
    Ok(())
}
```

#### Remote Ollama Server

```rust
use radium_models::OllamaModel;

let model = OllamaModel::with_base_url(
    "llama2".to_string(),
    "http://192.168.1.100:11434".to_string()
)?;
```

#### Using ModelFactory

```rust
use radium_models::{ModelFactory, ModelConfig, ModelType};

let config = ModelConfig::new(
    ModelType::Ollama,
    "llama2".to_string()
);
let model = ModelFactory::create(config)?;
```

## Configuration

### Ollama Configuration Options

- `model_id`: Model identifier (e.g., "llama2", "codellama:13b")
- `base_url`: Ollama server URL (default: "http://localhost:11434")

### Environment Variables

- `OLLAMA_BASE_URL`: Override default Ollama server URL (optional)

## Troubleshooting

### "Ollama server not reachable"

- Ensure Ollama is running: `ollama serve`
- Check the server is listening on port 11434
- For remote servers, set custom base_url using `OllamaModel::with_base_url()`

### "Model not found"

- Pull the model: `ollama pull llama2`
- List available models: `ollama list`
- Verify the model name matches exactly

### "Insufficient memory"

- Try a smaller model variant (e.g., `llama2:7b` instead of `llama2:13b`)
- Close other applications to free memory
- Check available system memory: `free -h` (Linux) or `vm_stat` (macOS)

## Testing

### Unit Tests

Run unit tests with mocked HTTP:
```bash
cargo test -p radium-models
```

### Integration Tests

Run integration tests with real Ollama server:
```bash
# Ensure Ollama is running and llama2 is pulled
cargo test -p radium-models -- --ignored
```

## API Documentation

For detailed API documentation, run:
```bash
cargo doc -p radium-models --open
```


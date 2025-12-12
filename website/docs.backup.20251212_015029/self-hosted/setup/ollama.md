# Ollama Setup Guide

## Overview

Ollama is a local model runner that makes it easy to download and run open-source models. It's the fastest way to get started with self-hosted models in Radium, with setup taking approximately 5-10 minutes.

## Prerequisites

- **macOS** or **Linux** (Windows support via WSL)
- **8GB RAM minimum** (16GB recommended)
- **curl** or **wget** for installation
- **Docker** (optional, for containerized deployment)

## Installation

### macOS

#### Option 1: Homebrew (Recommended)

```bash
brew install ollama
```

#### Option 2: Direct Download

```bash
curl -fsSL https://ollama.com/install.sh | sh
```

### Linux

#### Installation Script

```bash
curl -fsSL https://ollama.com/install.sh | sh
```

This script will:
- Download the Ollama binary
- Install it to `/usr/local/bin/ollama`
- Create a systemd service (if systemd is available)

#### Manual Installation

1. Download the binary for your architecture:
   ```bash
   # For x86_64
   curl -L https://ollama.com/download/ollama-linux-amd64 -o /usr/local/bin/ollama
   chmod +x /usr/local/bin/ollama
   ```

2. Start the Ollama service:
   ```bash
   ollama serve
   ```

### Docker

For containerized deployment:

```bash
docker run -d -v ollama:/root/.ollama -p 11434:11434 --name ollama ollama/ollama
```

**Note**: Models will be stored in the Docker volume `ollama`.

## Starting Ollama

### macOS / Linux

After installation, start the Ollama service:

```bash
ollama serve
```

The service will run in the foreground. For production, you may want to run it as a background service or use systemd.

### Systemd Service (Linux)

Create `/etc/systemd/system/ollama.service`:

```ini
[Unit]
Description=Ollama Service
After=network.target

[Service]
ExecStart=/usr/local/bin/ollama serve
User=ollama
Group=ollama
Restart=always

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable ollama
sudo systemctl start ollama
```

### Docker

If using Docker, the service starts automatically:

```bash
docker start ollama
```

## Verifying Installation

Test that Ollama is running:

```bash
curl http://localhost:11434/api/tags
```

You should see a JSON response with available models (may be empty initially).

## Model Management

### Downloading Models

Pull models using the `ollama pull` command:

```bash
# Popular models
ollama pull llama3.2          # Llama 3.2 (3B parameters, ~2GB)
ollama pull llama3.2:13b      # Llama 3.2 13B (~7GB)
ollama pull codellama        # CodeLlama (7B, optimized for code)
ollama pull mistral          # Mistral 7B
ollama pull mixtral          # Mixtral 8x7B MoE
```

### Listing Models

View all downloaded models:

```bash
ollama list
```

### Removing Models

Delete a model to free up disk space:

```bash
ollama rm llama3.2
```

### Model Recommendations

| Model | Size | Use Case | RAM Required |
|-------|------|----------|--------------|
| `llama3.2` | ~2GB | General purpose, fast | 8GB |
| `llama3.2:13b` | ~7GB | Better quality | 16GB |
| `codellama` | ~4GB | Code generation | 8GB |
| `mistral` | ~4GB | Balanced performance | 8GB |
| `mixtral` | ~26GB | High quality, reasoning | 32GB |

## Hardware Requirements

### Minimum Requirements

- **CPU**: Modern x86_64 or ARM64 processor
- **RAM**: 8GB (for 3B-7B models)
- **Storage**: 10GB free space (for models)

### Recommended Requirements

- **CPU**: Multi-core processor (4+ cores)
- **RAM**: 16GB+ (for 13B+ models)
- **Storage**: 50GB+ free space (for multiple models)
- **GPU**: Optional, but significantly improves performance (NVIDIA with CUDA support)

### GPU Support

Ollama automatically uses GPU if available:

```bash
# Check if GPU is detected
ollama ps
```

For NVIDIA GPUs, ensure CUDA drivers are installed. Ollama will use the GPU automatically if detected.

## Configuration

### Default Settings

Ollama runs on:
- **Host**: `localhost`
- **Port**: `11434`
- **API Endpoint**: `http://localhost:11434`

### Custom Port

To run on a different port:

```bash
OLLAMA_HOST=0.0.0.0:11435 ollama serve
```

### Remote Access

To allow remote access:

```bash
OLLAMA_HOST=0.0.0.0:11434 ollama serve
```

**Security Note**: Only enable remote access on trusted networks or with proper firewall rules.

## Using with Radium

### Current Implementation Status

⚠️ **Important**: While a native `OllamaModel` implementation exists in the Radium codebase, it is not yet integrated into the `ModelFactory`. Use the Universal provider as the recommended approach.

### Configuration via Universal Provider

Ollama provides an OpenAI-compatible API endpoint. Configure Radium to use it:

**Agent Configuration (TOML):**
```toml
[agent]
id = "my-agent"
name = "My Agent"
description = "Agent using Ollama"
prompt_path = "prompts/agents/my-agent.md"
engine = "universal"
model = "llama3.2"
```

**Environment Variables:**
```bash
export UNIVERSAL_BASE_URL="http://localhost:11434/v1"
export UNIVERSAL_MODEL_ID="llama3.2"
```

Or in your agent configuration, you'll need to set the base URL through environment variables or configuration files that support it.

### Testing the Connection

Test that Radium can connect to Ollama:

```bash
# Using curl to test the OpenAI-compatible endpoint
curl http://localhost:11434/v1/models
```

You should see a list of available models in OpenAI format.

### Example Agent Configuration

Create `agents/my-agents/ollama-agent.toml`:

```toml
[agent]
id = "ollama-agent"
name = "Ollama Agent"
description = "Agent using local Ollama model"
prompt_path = "prompts/agents/my-agents/ollama-agent.md"
engine = "universal"
model = "llama3.2"

[agent.persona.models]
primary = "llama3.2"
fallback = "llama3.2:13b"
```

**Note**: The exact configuration format may vary based on how Radium's engine system resolves Universal provider endpoints. Check the [agent configuration guide](../configuration/agent-config.md) for the latest patterns.

## Troubleshooting

### Connection Refused

**Problem**: `curl http://localhost:11434/api/tags` returns connection refused.

**Solutions**:
1. Ensure Ollama is running: `ollama serve`
2. Check the port: `netstat -an | grep 11434`
3. Verify firewall settings

### Model Not Found

**Problem**: Model not available when making requests.

**Solutions**:
1. Pull the model: `ollama pull llama3.2`
2. List available models: `ollama list`
3. Verify model name matches exactly

### Out of Memory

**Problem**: Model fails to load or runs very slowly.

**Solutions**:
1. Use a smaller model (e.g., `llama3.2` instead of `llama3.2:13b`)
2. Close other applications to free RAM
3. Consider using a model with quantization (smaller memory footprint)

### Slow Performance

**Problem**: Model inference is slow.

**Solutions**:
1. Use GPU if available (Ollama detects automatically)
2. Use a smaller/faster model
3. Reduce `max_tokens` in generation parameters
4. Ensure sufficient RAM (swap usage indicates memory pressure)

## Next Steps

1. **Configure Your Agent**: See the [agent configuration guide](../configuration/agent-config.md)
2. **Test Your Setup**: Run a simple agent execution to verify connectivity
3. **Explore Models**: Try different models to find the best fit for your use case
4. **Optimize Performance**: Tune model parameters and hardware configuration

## Additional Resources

- [Ollama Official Documentation](https://ollama.com/docs)
- [Ollama Model Library](https://ollama.com/library)
- [Radium Universal Provider Guide](../../universal-provider-guide.md)
- [Troubleshooting Guide](../troubleshooting.md)


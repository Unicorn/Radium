# LocalAI Setup Guide

## Overview

LocalAI is a versatile local inference server that supports multiple model backends (llama.cpp, transformers, etc.) and provides an OpenAI-compatible API. It's ideal for flexible deployments with CPU or GPU support.

**Setup Time**: ~15 minutes  
**Best For**: Flexible deployments, multiple backends, CPU/GPU support

## Prerequisites

- **Docker** and **Docker Compose** (recommended)
- **8GB+ RAM** (for 7B models), **16GB+ RAM** (for 13B+ models)
- **CPU**: Modern x86_64 or ARM64 processor
- **GPU**: Optional, but improves performance (NVIDIA with CUDA or Apple Silicon)

## Installation

### Docker Compose (Recommended)

The easiest way to run LocalAI is with Docker Compose:

```yaml
version: '3.8'

services:
  localai:
    image: localai/localai:latest-aio-cuda
    ports:
      - "8080:8080"
    volumes:
      - ./models:/models
      - ./config:/config
    environment:
      - MODELS_PATH=/models
      - CONFIG_PATH=/config
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: all
              capabilities: [gpu]
```

Save as `docker-compose.yml` and run:

```bash
docker-compose up -d
```

### Docker (Standalone)

For a simple Docker deployment:

```bash
docker run -d \
  -p 8080:8080 \
  -v $(pwd)/models:/models \
  -v $(pwd)/config:/config \
  -e MODELS_PATH=/models \
  -e CONFIG_PATH=/config \
  localai/localai:latest-aio-cuda
```

### Standalone Binary

For systems without Docker:

1. Download the binary from [LocalAI releases](https://github.com/go-skynet/LocalAI/releases)
2. Extract and make executable:
   ```bash
   chmod +x localai
   ```
3. Run:
   ```bash
   ./localai
   ```

## Model Configuration

### Model Gallery

LocalAI supports a model gallery for easy model installation:

```bash
# List available models
curl http://localhost:8080/models/available

# Install a model from the gallery
curl http://localhost:8080/models/apply -d '{
  "id": "ggml-gpt4all-j"
}'
```

### Manual Model Configuration

Create model configuration files in the `config` directory:

**Example: `config/gpt-3.5-turbo.yaml`**
```yaml
name: gpt-3.5-turbo
backend: llama-cpp
parameters:
  model: gpt-3.5-turbo.gguf
  context_size: 4096
  f16: true
  threads: 4
  gpu_layers: 35
```

### Downloading Models

Download models manually:

```bash
# Create models directory
mkdir -p models

# Download a model (example: GPT-4All)
wget https://gpt4all.io/models/ggml-gpt4all-j-v1.3-groovy.bin -O models/gpt-3.5-turbo.gguf
```

### Supported Model Formats

- **GGUF** (llama.cpp) - Recommended for CPU inference
- **GGML** (legacy llama.cpp format)
- **Transformers** (Hugging Face) - Requires GPU

## Backend Selection

### llama.cpp (CPU, Recommended)

Best for CPU inference and most models:

```yaml
backend: llama-cpp
parameters:
  model: model.gguf
  threads: 4
  f16: true
```

### Transformers (GPU)

For GPU acceleration with Hugging Face models:

```yaml
backend: transformers
parameters:
  model: meta-llama/Llama-3-8B-Instruct
  gpu_layers: 35
```

### Whisper (Audio)

For audio transcription:

```yaml
backend: whisper
parameters:
  model: whisper-base
```

## Hardware Requirements

### CPU-Only Setup

| Model Size | RAM Required | CPU Cores | Performance |
|------------|--------------|-----------|-------------|
| 7B | 8GB | 4+ | Slow (5-10 tokens/s) |
| 13B | 16GB | 8+ | Very Slow (2-5 tokens/s) |

### GPU Setup

| Model Size | VRAM Required | GPU Examples |
|------------|---------------|--------------|
| 7B | 8GB | RTX 3060, RTX 4060 |
| 13B | 16GB | RTX 3090, RTX 4090 |
| 30B+ | 24GB+ | RTX 4090, A100 |

## Configuration Options

### Environment Variables

```bash
# Model storage path
MODELS_PATH=/models

# Configuration path
CONFIG_PATH=/config

# Backend selection
BACKEND=llama-cpp

# GPU settings
CUDA_VISIBLE_DEVICES=0

# Thread count (CPU)
THREADS=4
```

### Model Parameters

Common parameters in model YAML files:

| Parameter | Description | Default | Recommended |
|-----------|-------------|---------|-------------|
| `threads` | CPU threads | Auto | 4-8 |
| `gpu_layers` | GPU layers (llama.cpp) | 0 | 35 (for 7B models) |
| `context_size` | Context window | 512 | 4096 or 8192 |
| `f16` | Use FP16 | false | true (if supported) |
| `batch_size` | Batch size | 512 | 512 |

## Verifying Installation

Test that LocalAI is running:

```bash
# Check server health
curl http://localhost:8080/readyz

# List available models
curl http://localhost:8080/v1/models

# Test a completion
curl http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

## Using with Radium

### Configuration via Universal Provider

LocalAI provides an OpenAI-compatible API. Configure Radium to use it:

**Environment Variables:**
```bash
export UNIVERSAL_BASE_URL="http://localhost:8080/v1"
export UNIVERSAL_MODEL_ID="gpt-3.5-turbo"
```

**Agent Configuration (TOML):**
```toml
[agent]
id = "localai-agent"
name = "LocalAI Agent"
description = "Agent using LocalAI for flexible inference"
prompt_path = "prompts/agents/my-agents/localai-agent.md"
engine = "universal"
model = "gpt-3.5-turbo"
```

### Example Agent Configuration

Create `agents/my-agents/localai-agent.toml`:

```toml
[agent]
id = "localai-agent"
name = "LocalAI Agent"
description = "Flexible agent using LocalAI"
prompt_path = "prompts/agents/my-agents/localai-agent.md"
engine = "universal"
model = "gpt-3.5-turbo"

[agent.persona.models]
primary = "gpt-3.5-turbo"
fallback = "gpt-4"
```

**Note**: The exact configuration format may vary based on how Radium's engine system resolves Universal provider endpoints. Check the [agent configuration guide](../configuration/agent-config.md) for the latest patterns.

## Multi-Model Serving

LocalAI can serve multiple models simultaneously. Configure each model in separate YAML files:

**`config/model1.yaml`:**
```yaml
name: model1
backend: llama-cpp
parameters:
  model: model1.gguf
```

**`config/model2.yaml`:**
```yaml
name: model2
backend: llama-cpp
parameters:
  model: model2.gguf
```

Both models will be available via the API.

## Troubleshooting

### Model Not Found

**Problem**: Model not available when making requests.

**Solutions**:
1. Verify model file exists in `MODELS_PATH`
2. Check model configuration YAML is correct
3. Verify model name matches configuration
4. Check LocalAI logs: `docker logs localai`

### Out of Memory

**Problem**: Model fails to load due to insufficient RAM/VRAM.

**Solutions**:
1. Use a smaller model
2. Reduce `context_size` in model config
3. Use quantization (GGUF Q4 or Q8)
4. Close other applications

### Slow Performance

**Problem**: Inference is very slow.

**Solutions**:
1. Use GPU if available (set `gpu_layers`)
2. Increase `threads` for CPU inference
3. Use a smaller/faster model
4. Enable `f16` if supported
5. Check CPU/GPU utilization

### Connection Refused

**Problem**: Can't connect to LocalAI server.

**Solutions**:
1. Verify server is running: `docker ps` or check process
2. Check port is correct: `netstat -an | grep 8080`
3. Verify firewall settings
4. Check Docker port mapping

## Next Steps

1. **Configure Your Agent**: See the [agent configuration guide](../configuration/agent-config.md)
2. **Optimize Performance**: Tune backend and model parameters
3. **Add More Models**: Configure additional models for different use cases
4. **Set Up Monitoring**: Monitor resource usage and performance

## Additional Resources

- [LocalAI Official Documentation](https://localai.io/)
- [LocalAI GitHub Repository](https://github.com/go-skynet/LocalAI)
- [Model Gallery](https://github.com/go-skynet/model-gallery)
- [Radium Universal Provider Guide](../../universal-provider-guide.md)
- [Troubleshooting Guide](../troubleshooting.md)


---
id: "vllm"
title: "vLLM Setup Guide"
sidebar_label: "vLLM Setup Guide"
---

# vLLM Setup Guide

## Overview

vLLM is a high-performance LLM inference server optimized for throughput and low latency. It's ideal for production deployments requiring GPU acceleration and high throughput.

**Setup Time**: ~15 minutes  
**Best For**: Production deployments, GPU inference, high throughput

## Prerequisites

- **NVIDIA GPU** with CUDA support (required)
- **CUDA 11.8+** or **CUDA 12.1+**
- **Python 3.8+** (if not using Docker)
- **Docker** (recommended) or **Docker Compose**
- **16GB+ VRAM** (for 7B models), **40GB+ VRAM** (for 13B+ models)

## Installation

### Docker (Recommended)

The easiest way to run vLLM is with Docker:

```bash
docker run --gpus all \
  -p 8000:8000 \
  -v ~/.cache/huggingface:/root/.cache/huggingface \
  vllm/vllm-openai:latest \
  --model meta-llama/Llama-3-8B-Instruct \
  --port 8000
```

This command:
- Uses all available GPUs (`--gpus all`)
- Exposes port 8000
- Mounts Hugging Face cache for model persistence
- Loads the Llama-3-8B-Instruct model

### Docker Compose

For easier management, use Docker Compose:

```yaml
version: '3.8'

services:
  vllm:
    image: vllm/vllm-openai:latest
    ports:
      - "8000:8000"
    volumes:
      - ~/.cache/huggingface:/root/.cache/huggingface
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: all
              capabilities: [gpu]
    command: >
      --model meta-llama/Llama-3-8B-Instruct
      --port 8000
      --max-num-seqs 256
      --gpu-memory-utilization 0.9
```

Save as `docker-compose.yml` and run:

```bash
docker-compose up -d
```

### Python Installation (Advanced)

For bare metal installation:

```bash
# Install CUDA toolkit first (if not already installed)
# Then install vLLM
pip install vllm

# Or from source
git clone https://github.com/vllm-project/vllm.git
cd vllm
pip install -e .
```

## Starting the Server

### Basic Command

```bash
vllm serve meta-llama/Llama-3-8B-Instruct --port 8000
```

### With Custom Parameters

```bash
vllm serve meta-llama/Llama-3-8B-Instruct \
  --port 8000 \
  --max-num-seqs 256 \
  --gpu-memory-utilization 0.9 \
  --tensor-parallel-size 1
```

### Common Parameters

| Parameter | Description | Default | Recommended |
|-----------|-------------|---------|-------------|
| `--port` | Server port | 8000 | 8000 |
| `--max-num-seqs` | Max concurrent sequences | 256 | 256 (adjust based on VRAM) |
| `--gpu-memory-utilization` | GPU memory usage (0-1) | 0.9 | 0.9 |
| `--tensor-parallel-size` | Number of GPUs for tensor parallelism | 1 | 1 (single GPU) or 2+ (multi-GPU) |
| `--max-model-len` | Max sequence length | Auto | 4096 or 8192 |
| `--quantization` | Quantization method | None | `awq` or `gptq` for smaller VRAM |

## Kubernetes Deployment

For production Kubernetes deployments:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: vllm
spec:
  replicas: 1
  selector:
    matchLabels:
      app: vllm
  template:
    metadata:
      labels:
        app: vllm
    spec:
      containers:
      - name: vllm
        image: vllm/vllm-openai:latest
        ports:
        - containerPort: 8000
        resources:
          limits:
            nvidia.com/gpu: 1
          requests:
            nvidia.com/gpu: 1
        command:
        - python
        - -m
        - vllm.entrypoints.openai.api_server
        args:
        - --model
        - meta-llama/Llama-3-8B-Instruct
        - --port
        - "8000"
        - --max-num-seqs
        - "256"
        volumeMounts:
        - name: model-cache
          mountPath: /root/.cache/huggingface
      volumes:
      - name: model-cache
        persistentVolumeClaim:
          claimName: vllm-model-cache
---
apiVersion: v1
kind: Service
metadata:
  name: vllm
spec:
  selector:
    app: vllm
  ports:
  - port: 8000
    targetPort: 8000
  type: LoadBalancer
```

## GPU Requirements

### Minimum Requirements

| Model Size | VRAM Required | GPU Examples |
|------------|---------------|--------------|
| 7B | 16GB | RTX 3090, RTX 4090, A100 40GB |
| 13B | 24GB | RTX 4090, A100 40GB |
| 30B+ | 40GB+ | A100 40GB, A100 80GB |

### Recommended GPUs

- **NVIDIA A100** (40GB or 80GB) - Best for production
- **NVIDIA RTX 4090** (24GB) - Good for 7B-13B models
- **NVIDIA RTX 3090** (24GB) - Budget option for 7B models

### Multi-GPU Setup

For models requiring multiple GPUs:

```bash
vllm serve meta-llama/Llama-3-70B-Instruct \
  --tensor-parallel-size 4 \
  --port 8000
```

This distributes the model across 4 GPUs using tensor parallelism.

## Model Loading

### Supported Models

vLLM supports models from Hugging Face:

- **Llama**: `meta-llama/Llama-3-8B-Instruct`, `meta-llama/Llama-3-70B-Instruct`
- **Mistral**: `mistralai/Mistral-7B-Instruct-v0.2`
- **CodeLlama**: `codellama/CodeLlama-7b-Instruct-hf`
- **Mixtral**: `mistralai/Mixtral-8x7B-Instruct-v0.1`

### Model Download

Models are automatically downloaded from Hugging Face on first use. They're cached in `~/.cache/huggingface/`.

To pre-download a model:

```bash
python -c "from transformers import AutoModel; AutoModel.from_pretrained('meta-llama/Llama-3-8B-Instruct')"
```

## Performance Tuning

### Throughput Optimization

Increase concurrent requests:

```bash
vllm serve meta-llama/Llama-3-8B-Instruct \
  --max-num-seqs 512 \
  --gpu-memory-utilization 0.95
```

### Latency Optimization

Reduce concurrent requests for lower latency:

```bash
vllm serve meta-llama/Llama-3-8B-Instruct \
  --max-num-seqs 64 \
  --gpu-memory-utilization 0.8
```

### Memory Optimization

Use quantization for smaller VRAM footprint:

```bash
vllm serve meta-llama/Llama-3-8B-Instruct \
  --quantization awq \
  --gpu-memory-utilization 0.9
```

## Verifying Installation

Test that vLLM is running:

```bash
# Check server health
curl http://localhost:8000/health

# List available models
curl http://localhost:8000/v1/models

# Test a completion
curl http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "meta-llama/Llama-3-8B-Instruct",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

## Using with Radium

### Configuration via Universal Provider

vLLM provides an OpenAI-compatible API. Configure Radium to use it:

**Environment Variables:**
```bash
export UNIVERSAL_BASE_URL="http://localhost:8000/v1"
export UNIVERSAL_MODEL_ID="meta-llama/Llama-3-8B-Instruct"
```

**Agent Configuration (TOML):**
```toml
[agent]
id = "vllm-agent"
name = "vLLM Agent"
description = "Agent using vLLM for high-performance inference"
prompt_path = "prompts/agents/my-agents/vllm-agent.md"
engine = "universal"
model = "meta-llama/Llama-3-8B-Instruct"
```

### Example Agent Configuration

Create `agents/my-agents/vllm-agent.toml`:

```toml
[agent]
id = "vllm-agent"
name = "vLLM Agent"
description = "High-performance agent using vLLM"
prompt_path = "prompts/agents/my-agents/vllm-agent.md"
engine = "universal"
model = "meta-llama/Llama-3-8B-Instruct"

[agent.persona.models]
primary = "meta-llama/Llama-3-8B-Instruct"
fallback = "meta-llama/Llama-3-70B-Instruct"
```

**Note**: The exact configuration format may vary based on how Radium's engine system resolves Universal provider endpoints. Check the [agent configuration guide](../configuration/agent-config.md) for the latest patterns.

## Troubleshooting

### GPU Not Detected

**Problem**: vLLM doesn't detect GPU.

**Solutions**:
1. Verify CUDA is installed: `nvidia-smi`
2. Check Docker GPU access: `docker run --gpus all nvidia/cuda:11.8.0-base-ubuntu22.04 nvidia-smi`
3. Ensure NVIDIA Container Toolkit is installed

### Out of Memory

**Problem**: Model fails to load due to insufficient VRAM.

**Solutions**:
1. Use a smaller model
2. Reduce `--gpu-memory-utilization` (e.g., 0.7)
3. Use quantization: `--quantization awq`
4. Reduce `--max-num-seqs`

### Slow Performance

**Problem**: Inference is slower than expected.

**Solutions**:
1. Check GPU utilization: `nvidia-smi`
2. Increase `--max-num-seqs` for better throughput
3. Verify model is loaded on GPU (not CPU)
4. Check for thermal throttling

### Connection Refused

**Problem**: Can't connect to vLLM server.

**Solutions**:
1. Verify server is running: `curl http://localhost:8000/health`
2. Check port is correct: `netstat -an | grep 8000`
3. Verify firewall settings
4. Check Docker port mapping

## Next Steps

1. **Configure Your Agent**: See the [agent configuration guide](../configuration/agent-config.md)
2. **Optimize Performance**: Tune parameters based on your workload
3. **Set Up Monitoring**: Monitor GPU usage and throughput
4. **Scale Deployment**: Consider Kubernetes for production

## Additional Resources

- [vLLM Official Documentation](https://docs.vllm.ai/)
- [vLLM GitHub Repository](https://github.com/vllm-project/vllm)
- [Radium Universal Provider Guide](../../universal-provider-guide.md)
- [Troubleshooting Guide](../troubleshooting.md)


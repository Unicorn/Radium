---
id: "examples"
title: "Configuration Examples"
sidebar_label: "Configuration Examples"
---

# Configuration Examples

This document provides comprehensive configuration examples for using self-hosted models with Radium agents.

## Table of Contents

- [Basic Examples](#basic-examples)
- [Multi-Tier Examples](#multi-tier-examples)
- [Mixed Cloud/Self-Hosted](#mixed-cloudself-hosted)
- [Docker Compose Examples](#docker-compose-examples)
- [Production Examples](#production-examples)

## Basic Examples

### Ollama - Simple Agent

**File: `agents/my-agents/ollama-simple.toml`**
```toml
[agent]
id = "ollama-simple"
name = "Simple Ollama Agent"
description = "Basic agent using Ollama"
prompt_path = "prompts/agents/my-agents/ollama-simple.md"
engine = "universal"
model = "llama3.2"
```

**Environment Setup:**
```bash
export UNIVERSAL_BASE_URL="http://localhost:11434/v1"
```

### vLLM - High-Performance Agent

**File: `agents/my-agents/vllm-agent.toml`**
```toml
[agent]
id = "vllm-agent"
name = "vLLM Agent"
description = "High-performance agent using vLLM"
prompt_path = "prompts/agents/my-agents/vllm-agent.md"
engine = "universal"
model = "meta-llama/Llama-3-8B-Instruct"
reasoning_effort = "high"
```

**Environment Setup:**
```bash
export UNIVERSAL_BASE_URL="http://localhost:8000/v1"
```

### LocalAI - Flexible Agent

**File: `agents/my-agents/localai-agent.toml`**
```toml
[agent]
id = "localai-agent"
name = "LocalAI Agent"
description = "Flexible agent using LocalAI"
prompt_path = "prompts/agents/my-agents/localai-agent.md"
engine = "universal"
model = "gpt-3.5-turbo"
```

**Environment Setup:**
```bash
export UNIVERSAL_BASE_URL="http://localhost:8080/v1"
```

## Multi-Tier Examples

### Local Models Only

**File: `agents/my-agents/local-tier.toml`**
```toml
[agent]
id = "local-tier"
name = "Local Tier Agent"
description = "Agent with multiple local model tiers"
prompt_path = "prompts/agents/my-agents/local-tier.md"
engine = "universal"
model = "llama3.2"

[agent.persona.models]
primary = "llama3.2"           # Fast 3B model (Ollama)
fallback = "llama3.2:13b"      # Better 13B model (Ollama)
premium = "meta-llama/Llama-3-70B-Instruct"  # Best quality (vLLM)
```

**Environment Setup:**
```bash
# Primary and fallback use Ollama
export UNIVERSAL_BASE_URL="http://localhost:11434/v1"

# Premium uses vLLM (may need separate configuration)
# This example assumes the engine system can handle multiple endpoints
```

### Local Primary with Cloud Fallback

**File: `agents/my-agents/hybrid-fallback.toml`**
```toml
[agent]
id = "hybrid-fallback"
name = "Hybrid Fallback Agent"
description = "Local primary with cloud fallback"
prompt_path = "prompts/agents/my-agents/hybrid-fallback.md"
engine = "universal"
model = "llama3.2"

[agent.persona.models]
primary = "llama3.2"           # Local Ollama (fast, free)
fallback = "gemini-2.0-flash-exp"  # Cloud Gemini (reliable)
premium = "gpt-4o"             # Cloud OpenAI (best quality)
```

### Cost-Optimized Strategy

**File: `agents/my-agents/cost-optimized.toml`**
```toml
[agent]
id = "cost-optimized"
name = "Cost-Optimized Agent"
description = "Maximize local usage, minimize cloud costs"
prompt_path = "prompts/agents/my-agents/cost-optimized.md"
engine = "universal"
model = "llama3.2"

[agent.persona.models]
primary = "llama3.2"           # Local (free)
fallback = "llama3.2:13b"      # Local (free)
premium = "gpt-4o-mini"        # Cloud (cheap fallback)
```

## Mixed Cloud/Self-Hosted

### Development vs Production

**Development Agent (Local):**
```toml
# agents/my-agents/dev-agent.toml
[agent]
id = "dev-agent"
name = "Development Agent"
description = "Local agent for development"
prompt_path = "prompts/agents/my-agents/dev-agent.md"
engine = "universal"
model = "llama3.2"

[agent.persona.models]
primary = "llama3.2"
fallback = "llama3.2:13b"
```

**Production Agent (Cloud):**
```toml
# agents/my-agents/prod-agent.toml
[agent]
id = "prod-agent"
name = "Production Agent"
description = "Cloud agent for production"
prompt_path = "prompts/agents/my-agents/prod-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"

[agent.persona.models]
primary = "gemini-2.0-flash-exp"
fallback = "gpt-4o-mini"
premium = "gpt-4o"
```

### Different Agents, Different Providers

**Code Agent (Local for Speed):**
```toml
# agents/my-agents/code-local.toml
[agent]
id = "code-local"
name = "Local Code Agent"
description = "Fast local agent for code tasks"
prompt_path = "prompts/agents/my-agents/code-local.md"
engine = "universal"
model = "codellama"
```

**Reasoning Agent (Cloud for Quality):**
```toml
# agents/my-agents/reasoning-cloud.toml
[agent]
id = "reasoning-cloud"
name = "Cloud Reasoning Agent"
description = "High-quality cloud agent for reasoning"
prompt_path = "prompts/agents/my-agents/reasoning-cloud.md"
engine = "gemini"
model = "gemini-2.0-flash-thinking-exp"
```

## Docker Compose Examples

### Radium + Ollama

**File: `docker-compose.yml`**
```yaml
version: '3.8'

services:
  ollama:
    image: ollama/ollama:latest
    ports:
      - "11434:11434"
    volumes:
      - ollama-data:/root/.ollama
    environment:
      - OLLAMA_HOST=0.0.0.0:11434

  radium:
    # Your Radium service configuration
    # Ensure it can access ollama:11434
    environment:
      - UNIVERSAL_BASE_URL=http://ollama:11434/v1
    depends_on:
      - ollama

volumes:
  ollama-data:
```

### Radium + vLLM

**File: `docker-compose.yml`**
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

  radium:
    # Your Radium service configuration
    environment:
      - UNIVERSAL_BASE_URL=http://vllm:8000/v1
    depends_on:
      - vllm
```

### Radium + LocalAI

**File: `docker-compose.yml`**
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

  radium:
    # Your Radium service configuration
    environment:
      - UNIVERSAL_BASE_URL=http://localai:8080/v1
    depends_on:
      - localai
```

## Production Examples

### High Availability Setup

**Agent with Multiple Fallbacks:**
```toml
[agent]
id = "ha-agent"
name = "High Availability Agent"
description = "Agent with multiple fallback options"
prompt_path = "prompts/agents/my-agents/ha-agent.md"
engine = "universal"
model = "llama3.2"

[agent.persona.models]
primary = "llama3.2"           # Primary local model
fallback = "llama3.2:13b"      # Better local model
premium = "gpt-4o-mini"        # Cloud fallback
```

**Environment with Health Checks:**
```bash
# Primary endpoint
export UNIVERSAL_BASE_URL="http://ollama-primary:11434/v1"

# Fallback endpoint (if primary fails)
# Note: This may require custom engine configuration
```

### Performance-Tuned Agent

**File: `agents/my-agents/perf-tuned.toml`**
```toml
[agent]
id = "perf-tuned"
name = "Performance-Tuned Agent"
description = "Optimized for speed and throughput"
prompt_path = "prompts/agents/my-agents/perf-tuned.md"
engine = "universal"
model = "llama3.2"
reasoning_effort = "low"

[agent.persona.models]
primary = "llama3.2"           # Fastest local model
fallback = "llama3.2:13b"      # Slightly slower but better
premium = "gpt-4o-mini"        # Fast cloud option

[agent.persona.performance]
profile = "speed"
estimated_tokens = 2000
```

### Quality-Focused Agent

**File: `agents/my-agents/quality-focused.toml`**
```toml
[agent]
id = "quality-focused"
name = "Quality-Focused Agent"
description = "Optimized for output quality"
prompt_path = "prompts/agents/my-agents/quality-focused.md"
engine = "universal"
model = "llama3.2:13b"
reasoning_effort = "high"

[agent.persona.models]
primary = "llama3.2:13b"       # Better local model
fallback = "mixtral"           # Best local model (if available)
premium = "gpt-4o"             # Best cloud model

[agent.persona.performance]
profile = "thinking"
estimated_tokens = 8000
```

## Environment Variable Examples

### Single Provider Setup

```bash
# .env file
UNIVERSAL_BASE_URL=http://localhost:11434/v1
UNIVERSAL_MODEL_ID=llama3.2
```

### Multiple Providers (Switching)

```bash
# Switch to Ollama
export UNIVERSAL_BASE_URL="http://localhost:11434/v1"
export UNIVERSAL_MODEL_ID="llama3.2"

# Switch to vLLM
export UNIVERSAL_BASE_URL="http://localhost:8000/v1"
export UNIVERSAL_MODEL_ID="meta-llama/Llama-3-8B-Instruct"

# Switch to LocalAI
export UNIVERSAL_BASE_URL="http://localhost:8080/v1"
export UNIVERSAL_MODEL_ID="gpt-3.5-turbo"
```

### Remote Server Setup

```bash
# Remote Ollama server
export UNIVERSAL_BASE_URL="http://192.168.1.100:11434/v1"
export UNIVERSAL_MODEL_ID="llama3.2"

# Remote vLLM server
export UNIVERSAL_BASE_URL="http://vllm.example.com:8000/v1"
export UNIVERSAL_MODEL_ID="meta-llama/Llama-3-8B-Instruct"
```

## Complete Working Example

### Full Stack: Agent + Ollama + Docker

**1. Agent Configuration: `agents/my-agents/example.toml`**
```toml
[agent]
id = "example"
name = "Example Agent"
description = "Complete example agent"
prompt_path = "prompts/agents/my-agents/example.md"
engine = "universal"
model = "llama3.2"
reasoning_effort = "medium"

[agent.persona.models]
primary = "llama3.2"
fallback = "llama3.2:13b"
```

**2. Docker Compose: `docker-compose.yml`**
```yaml
version: '3.8'

services:
  ollama:
    image: ollama/ollama:latest
    ports:
      - "11434:11434"
    volumes:
      - ollama-data:/root/.ollama

volumes:
  ollama-data:
```

**3. Environment: `.env`**
```bash
UNIVERSAL_BASE_URL=http://localhost:11434/v1
UNIVERSAL_MODEL_ID=llama3.2
```

**4. Setup Script: `setup.sh`**
```bash
#!/bin/bash
# Start services
docker-compose up -d

# Wait for Ollama
sleep 5

# Pull model
docker exec ollama ollama pull llama3.2

# Verify
curl http://localhost:11434/api/tags
```

**5. Test:**
```bash
# Run the agent
rad run example "Hello, how are you?"
```

## Next Steps

- See [Agent Configuration Guide](agent-config.md) for detailed explanations
- Check [Advanced Configuration](advanced.md) for production patterns
- Review [Troubleshooting Guide](../troubleshooting.md) for common issues
- Explore [Setup Guides](../setup/) for provider installation


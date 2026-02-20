---
id: "agent-config"
title: "Agent Configuration for Self-Hosted Models"
sidebar_label: "Agent Configuration for Self-Hosted Mode..."
---

# Agent Configuration for Self-Hosted Models

## Overview

This guide explains how to configure Radium agents to use self-hosted model providers (Ollama, vLLM, LocalAI) through the Universal provider. All self-hosted models are accessed via the Universal provider, which implements the OpenAI Chat Completions API specification.

## Basic Configuration

### Universal Provider Setup

All self-hosted models use the `universal` engine type. The configuration requires:

1. **Engine**: Set to `"universal"`
2. **Model**: The model identifier (e.g., `"llama3.2"`, `"meta-llama/Llama-3-8B-Instruct"`)
3. **Base URL**: Configured via environment variables or system configuration

### Minimal Configuration

```toml
[agent]
id = "self-hosted-agent"
name = "Self-Hosted Agent"
description = "Agent using local Ollama model"
prompt_path = "prompts/agents/my-agents/self-hosted-agent.md"
engine = "universal"
model = "llama3.2"
```

**Note**: The base URL for the Universal provider must be configured via environment variables or system settings. See [Environment Variables](#environment-variables) below.

## Provider-Specific Configurations

### Ollama Configuration

**Agent TOML:**
```toml
[agent]
id = "ollama-agent"
name = "Ollama Agent"
description = "Agent using local Ollama model"
prompt_path = "prompts/agents/my-agents/ollama-agent.md"
engine = "universal"
model = "llama3.2"
```

**Environment Variables:**
```bash
export UNIVERSAL_BASE_URL="http://localhost:11434/v1"
export UNIVERSAL_MODEL_ID="llama3.2"
```

**Full Example:**
```toml
[agent]
id = "ollama-agent"
name = "Ollama Agent"
description = "Agent using local Ollama model"
prompt_path = "prompts/agents/my-agents/ollama-agent.md"
engine = "universal"
model = "llama3.2"
reasoning_effort = "medium"

[agent.persona.models]
primary = "llama3.2"
fallback = "llama3.2:13b"
```

### vLLM Configuration

**Agent TOML:**
```toml
[agent]
id = "vllm-agent"
name = "vLLM Agent"
description = "Agent using vLLM for high-performance inference"
prompt_path = "prompts/agents/my-agents/vllm-agent.md"
engine = "universal"
model = "meta-llama/Llama-3-8B-Instruct"
```

**Environment Variables:**
```bash
export UNIVERSAL_BASE_URL="http://localhost:8000/v1"
export UNIVERSAL_MODEL_ID="meta-llama/Llama-3-8B-Instruct"
```

**Full Example:**
```toml
[agent]
id = "vllm-agent"
name = "vLLM Agent"
description = "High-performance agent using vLLM"
prompt_path = "prompts/agents/my-agents/vllm-agent.md"
engine = "universal"
model = "meta-llama/Llama-3-8B-Instruct"
reasoning_effort = "high"

[agent.persona.models]
primary = "meta-llama/Llama-3-8B-Instruct"
fallback = "meta-llama/Llama-3-70B-Instruct"
premium = "meta-llama/Llama-3-70B-Instruct"
```

### LocalAI Configuration

**Agent TOML:**
```toml
[agent]
id = "localai-agent"
name = "LocalAI Agent"
description = "Agent using LocalAI for flexible inference"
prompt_path = "prompts/agents/my-agents/localai-agent.md"
engine = "universal"
model = "gpt-3.5-turbo"
```

**Environment Variables:**
```bash
export UNIVERSAL_BASE_URL="http://localhost:8080/v1"
export UNIVERSAL_MODEL_ID="gpt-3.5-turbo"
```

**Full Example:**
```toml
[agent]
id = "localai-agent"
name = "LocalAI Agent"
description = "Flexible agent using LocalAI"
prompt_path = "prompts/agents/my-agents/localai-agent.md"
engine = "universal"
model = "gpt-3.5-turbo"
reasoning_effort = "medium"

[agent.persona.models]
primary = "gpt-3.5-turbo"
fallback = "gpt-4"
```

## Environment Variables

### Universal Provider Variables

The Universal provider uses these environment variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `UNIVERSAL_BASE_URL` | Base URL for the API endpoint | `http://localhost:11434/v1` |
| `UNIVERSAL_MODEL_ID` | Default model ID (optional) | `llama3.2` |
| `UNIVERSAL_API_KEY` | API key (if required) | `your-api-key` |
| `OPENAI_COMPATIBLE_API_KEY` | Alternative API key variable | `your-api-key` |

### Provider-Specific Variables

Some providers may use provider-specific environment variables:

| Provider | Variable | Default | Description |
|----------|----------|---------|-------------|
| Ollama | `OLLAMA_HOST` | `localhost:11434` | Ollama server address |
| vLLM | `VLLM_ENDPOINT` | `http://localhost:8000/v1` | vLLM API endpoint |
| LocalAI | `LOCALAI_ENDPOINT` | `http://localhost:8080/v1` | LocalAI API endpoint |

**Note**: These provider-specific variables may be used by the engine system to automatically configure the Universal provider. Check your Radium configuration for details.

## Multi-Tier Model Strategy

Radium supports a multi-tier model strategy with primary, fallback, and premium models. This is useful for self-hosted models where you want to:

- Use a fast local model as primary
- Fall back to a more capable model if needed
- Use a premium cloud model for critical tasks

### Example: Local Primary with Cloud Fallback

```toml
[agent]
id = "hybrid-agent"
name = "Hybrid Agent"
description = "Agent with local primary and cloud fallback"
prompt_path = "prompts/agents/my-agents/hybrid-agent.md"
engine = "universal"
model = "llama3.2"

[agent.persona.models]
primary = "llama3.2"           # Local Ollama (fast, free)
fallback = "gpt-4o-mini"       # Cloud OpenAI (reliable)
premium = "gpt-4o"             # Cloud OpenAI (best quality)
```

### Example: Multiple Local Models

```toml
[agent]
id = "local-tier-agent"
name = "Local Tier Agent"
description = "Agent with multiple local model tiers"
prompt_path = "prompts/agents/my-agents/local-tier-agent.md"
engine = "universal"
model = "llama3.2"

[agent.persona.models]
primary = "llama3.2"           # Fast 3B model
fallback = "llama3.2:13b"      # Better 13B model
premium = "mixtral"             # Best quality (if available)
```

## Mixed Cloud and Self-Hosted

You can configure agents to use a mix of cloud and self-hosted models:

```toml
[agent]
id = "mixed-agent"
name = "Mixed Agent"
description = "Agent mixing cloud and self-hosted models"
prompt_path = "prompts/agents/my-agents/mixed-agent.md"
engine = "gemini"              # Default to cloud
model = "gemini-2.0-flash-exp"

[agent.persona.models]
primary = "llama3.2"           # Self-hosted (Ollama)
fallback = "gemini-2.0-flash-exp"  # Cloud (Gemini)
premium = "gpt-4o"             # Cloud (OpenAI)
```

**Note**: When mixing providers, ensure the engine system can resolve models from different providers. The multi-tier strategy will attempt to use models in order: primary → fallback → premium.

## Model Parameters

### Reasoning Effort

Control the reasoning effort level:

```toml
[agent]
reasoning_effort = "low"    # Fast, less thorough
# reasoning_effort = "medium"  # Balanced (default)
# reasoning_effort = "high"   # Slow, more thorough
```

### Performance Profile

Configure performance characteristics:

```toml
[agent.persona.performance]
profile = "balanced"        # speed, balanced, thinking, expert
estimated_tokens = 4000
```

## Configuration Validation

### Testing Your Configuration

1. **Verify Model Server is Running:**
   ```bash
   # Ollama
   curl http://localhost:11434/api/tags
   
   # vLLM
   curl http://localhost:8000/v1/models
   
   # LocalAI
   curl http://localhost:8080/v1/models
   ```

2. **Test Agent Discovery:**
   ```bash
   rad agents list
   ```

3. **Test Agent Execution:**
   ```bash
   rad run ollama-agent "Test prompt"
   ```

### Common Configuration Issues

**Issue**: Agent can't connect to model server
- **Solution**: Verify environment variables are set correctly
- **Solution**: Check model server is running and accessible
- **Solution**: Verify base URL includes `/v1` path

**Issue**: Model not found
- **Solution**: Verify model name matches exactly (case-sensitive)
- **Solution**: Check model is available on the server
- **Solution**: For Ollama, run `ollama list` to see available models

**Issue**: Authentication errors
- **Solution**: Most local servers don't require API keys
- **Solution**: If using `UNIVERSAL_API_KEY`, ensure it's correct
- **Solution**: Try removing API key for local servers

## Advanced Configuration

### Custom Endpoints

For remote or custom-configured servers:

```bash
# Remote Ollama server
export UNIVERSAL_BASE_URL="http://192.168.1.100:11434/v1"

# Custom vLLM endpoint
export UNIVERSAL_BASE_URL="http://vllm.example.com:8000/v1"

# LocalAI with custom port
export UNIVERSAL_BASE_URL="http://localhost:9090/v1"
```

### Multiple Agents with Different Models

Configure different agents to use different self-hosted models:

```toml
# agents/my-agents/fast-agent.toml
[agent]
id = "fast-agent"
engine = "universal"
model = "llama3.2"  # Fast 3B model

# agents/my-agents/quality-agent.toml
[agent]
id = "quality-agent"
engine = "universal"
model = "llama3.2:13b"  # Better 13B model
```

## Next Steps

- See [Configuration Examples](examples.md) for more detailed examples
- Check [Troubleshooting Guide](../troubleshooting.md) for common issues
- Review [Advanced Configuration](advanced.md) for production setups
- Explore [Setup Guides](../setup/) for provider-specific installation


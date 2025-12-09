# Self-Hosted Model Integration

## Overview

Radium supports self-hosted AI models through the Universal provider, enabling you to run agents locally for privacy, cost savings, and air-gapped environments. This guide covers setting up and configuring Ollama, vLLM, and LocalAI as alternatives to cloud-based providers (Gemini, OpenAI, Claude).

## Benefits of Self-Hosted Models

- **Cost Savings**: Eliminate API costs by using local compute resources
- **Data Privacy**: Keep all prompts and responses on-premises
- **Air-Gapped Environments**: Run agents in isolated networks without internet access
- **Open-Source Models**: Access to a wide variety of open-source models
- **No Rate Limits**: Full control over throughput and usage
- **Customization**: Fine-tune models and optimize for your specific use cases

## Quick Start

The fastest way to get started is with Ollama:

```bash
# 1. Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# 2. Pull a model
ollama pull llama3.2

# 3. Configure your agent to use Universal provider
# See configuration guide for details
```

**Estimated Setup Time**: 5-10 minutes for Ollama, 15-30 minutes for vLLM/LocalAI

## Supported Providers

### Ollama
- **Best for**: Quick setup, CPU inference, development and testing
- **Setup Time**: ~5 minutes
- **Hardware**: 8GB+ RAM (16GB recommended)
- **Guide**: [Ollama Setup Guide](setup/ollama.md)

### vLLM
- **Best for**: High-performance production deployments, GPU inference
- **Setup Time**: ~15 minutes
- **Hardware**: NVIDIA GPU with 16GB+ VRAM
- **Guide**: [vLLM Setup Guide](setup/vllm.md)

### LocalAI
- **Best for**: Flexible deployments, multiple model backends, CPU/GPU support
- **Setup Time**: ~15 minutes
- **Hardware**: 8GB+ RAM (CPU) or GPU (optional)
- **Guide**: [LocalAI Setup Guide](setup/localai.md)

## Documentation Structure

- **[Setup Guides](setup/)**: Installation and deployment instructions for each provider
- **[Configuration](configuration/)**: Agent configuration examples and advanced patterns
- **[Troubleshooting](troubleshooting.md)**: Common issues and solutions
- **[Migration Guide](migration.md)**: Migrating from cloud providers to self-hosted
- **[API Reference](api-reference.md)**: Model trait and provider abstraction documentation

## Prerequisites

### Software Requirements

- **Docker** (for containerized deployments)
- **Docker Compose** (for LocalAI and multi-service setups)
- **curl** or **wget** (for installation scripts)
- **Python 3.8+** (for vLLM if not using Docker)

### Hardware Requirements

| Provider | Minimum RAM | Recommended RAM | GPU Required |
|----------|------------|-----------------|--------------|
| Ollama   | 8GB        | 16GB            | No (optional) |
| vLLM     | 16GB       | 32GB+           | Yes (NVIDIA) |
| LocalAI  | 8GB        | 16GB            | No (optional) |

### Network Requirements

- **Local Access**: Models run on localhost by default
- **Remote Access**: Configure firewall rules if accessing from other machines
- **Ports**: 
  - Ollama: `11434`
  - vLLM: `8000`
  - LocalAI: `8080`

## Current Implementation Status

### Universal Provider

All self-hosted models are accessed through Radium's Universal provider, which implements the OpenAI Chat Completions API specification. This means:

- ✅ **vLLM**: Fully supported via Universal provider
- ✅ **LocalAI**: Fully supported via Universal provider
- ⚠️ **Ollama**: Supported via Universal provider (native OllamaModel exists but factory integration pending)

**Note**: While a native `OllamaModel` implementation exists in the codebase, the ModelFactory currently doesn't support it. Use the Universal provider with `base_url = "http://localhost:11434/v1"` as the recommended approach.

## Next Steps

1. **Choose a Provider**: Review the [setup guides](setup/) to select the best option for your needs
2. **Install and Configure**: Follow the provider-specific setup guide
3. **Configure Agents**: See the [agent configuration guide](configuration/agent-config.md) for TOML examples
4. **Test Your Setup**: Use `rad doctor` to verify connectivity (when available)
5. **Explore Examples**: Check out [code examples](../examples/self-hosted-models/) for working configurations

## Getting Help

- **Troubleshooting**: See the [troubleshooting guide](troubleshooting.md) for common issues
- **API Reference**: Check the [API reference](api-reference.md) for implementation details
- **Universal Provider Guide**: See [Universal Provider Guide](../universal-provider-guide.md) for technical details
- **Issues**: Report problems on GitHub or consult the community

## Related Documentation

- [Universal Provider Guide](../universal-provider-guide.md) - Technical API documentation
- [Agent Configuration Guide](../user-guide/agent-configuration.md) - General agent configuration
- [CLI Commands](../cli/commands/agents.md) - Agent management commands


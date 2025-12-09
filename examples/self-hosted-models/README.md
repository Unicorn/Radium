# Self-Hosted Models Examples

This directory contains working examples for integrating self-hosted AI models (Ollama, vLLM, LocalAI) with Radium agents.

## Directory Structure

```
examples/self-hosted-models/
├── README.md           # This file
├── ollama/             # Ollama examples
│   ├── agent.toml
│   ├── docker-compose.yml
│   └── setup.sh
├── vllm/               # vLLM examples
│   ├── agent.toml
│   ├── docker-compose.yml
│   └── setup.sh
├── localai/            # LocalAI examples
│   ├── agent.toml
│   ├── docker-compose.yml
│   └── setup.sh
└── mixed/              # Mixed cloud/self-hosted examples
    ├── agent.toml
    └── README.md
```

## Quick Start

### Ollama Example

```bash
cd ollama
./setup.sh
docker-compose up -d
export UNIVERSAL_BASE_URL="http://localhost:11434/v1"
rad run ollama-agent "Hello!"
```

### vLLM Example

```bash
cd vllm
./setup.sh
docker-compose up -d
export UNIVERSAL_BASE_URL="http://localhost:8000/v1"
rad run vllm-agent "Hello!"
```

### LocalAI Example

```bash
cd localai
./setup.sh
docker-compose up -d
export UNIVERSAL_BASE_URL="http://localhost:8080/v1"
rad run localai-agent "Hello!"
```

## Requirements

- Docker and Docker Compose
- Radium CLI installed
- Sufficient hardware (see provider-specific requirements)

## See Also

- [Self-Hosted Models Documentation](../../docs/self-hosted-models/)
- [Setup Guides](../../docs/self-hosted-models/setup/)
- [Configuration Guide](../../docs/self-hosted-models/configuration/)


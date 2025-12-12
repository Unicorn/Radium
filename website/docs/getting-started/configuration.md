---
id: "configuration"
title: "Configuration Guide"
sidebar_label: "Configuration"
---

# Configuration Guide

This guide covers all configuration options for Radium, including workspace settings, agent configuration, orchestration settings, and more.

## Configuration Locations

Radium uses multiple configuration locations:

- **Workspace Config**: `.radium/config.toml` (project-specific)
- **User Config**: `~/.radium/config.toml` (user-specific)
- **Orchestration Config**: `~/.radium/orchestration.toml` (orchestration settings)
- **Agent Configs**: `agents/**/*.toml` (agent definitions)

## Workspace Configuration

Workspace configuration is stored in `.radium/config.toml`:

```toml
[workspace]
name = "my-project"
description = "My project description"

[engines]
default = "gemini"

[orchestration]
enabled = true
provider = "gemini"

[memory]
enabled = true
retention_days = 30

[policy]
approval_mode = "ask"
```

### Workspace Settings

- **name**: Workspace name
- **description**: Workspace description
- **engines.default**: Default AI provider
- **orchestration.enabled**: Enable/disable orchestration
- **orchestration.provider**: Default orchestration provider
- **memory.enabled**: Enable memory system
- **memory.retention_days**: Memory retention period
- **policy.approval_mode**: Default policy approval mode

## Agent Configuration

Agent configuration is stored in `agents/**/*.toml`:

```toml
[agent]
id = "my-agent"
name = "My Agent"
description = "Agent description"
prompt_path = "prompts/my-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"

[agent.persona]
[agent.persona.models]
primary = "gemini-2.0-flash-exp"
fallback = "gemini-1.5-pro"
premium = "gemini-1.5-pro"

[agent.persona.performance]
profile = "balanced"
estimated_tokens = 1500
```

### Agent Settings

- **id**: Unique agent identifier
- **name**: Human-readable name
- **description**: Agent description
- **prompt_path**: Path to prompt file
- **engine**: AI provider (gemini, claude, openai)
- **model**: Specific model to use
- **persona**: Persona system configuration

**Learn more**: [Agent Configuration Guide](../user-guide/agent-configuration.md)

## Orchestration Configuration

Orchestration configuration is stored in `~/.radium/orchestration.toml`:

```toml
[orchestration]
enabled = true
provider = "gemini"
model = "gemini-2.0-flash-exp"

[orchestration.fallback]
enabled = true
provider = "claude"
model = "claude-3-haiku"

[orchestration.routing]
strategy = "intelligent"
max_agents = 3
```

### Orchestration Settings

- **enabled**: Enable/disable orchestration
- **provider**: Default AI provider for orchestration
- **model**: Model to use for orchestration
- **fallback**: Fallback provider configuration
- **routing.strategy**: Routing strategy (intelligent, round-robin, etc.)
- **routing.max_agents**: Maximum agents per workflow

**Learn more**: [Orchestration Configuration](../user-guide/orchestration-configuration.md)

## Policy Configuration

Policy configuration is stored in `.radium/policy.toml`:

```toml
approval_mode = "ask"

[[rules]]
name = "Allow safe file operations"
priority = "user"
action = "allow"
tool_pattern = "read_*"

[[rules]]
name = "Require approval for file writes"
priority = "user"
action = "ask_user"
tool_pattern = "write_*"

[[rules]]
name = "Deny dangerous shell commands"
priority = "admin"
action = "deny"
tool_pattern = "run_terminal_cmd"
arg_pattern = "rm -rf *"
```

### Policy Settings

- **approval_mode**: Default approval mode (yolo, autoedit, ask)
- **rules**: Policy rules array
  - **name**: Rule name
  - **priority**: Rule priority (admin, user, default)
  - **action**: Rule action (allow, deny, ask_user)
  - **tool_pattern**: Tool name pattern
  - **arg_pattern**: Argument pattern (optional)

**Learn more**: [Policy Engine](../features/policy-engine.md)

## Environment Variables

### API Keys

```bash
# Google AI (Gemini)
export GOOGLE_AI_API_KEY="your-key-here"

# Anthropic (Claude)
export ANTHROPIC_API_KEY="your-key-here"

# OpenAI (GPT)
export OPENAI_API_KEY="your-key-here"
```

### Configuration Overrides

```bash
# Override default engine
export RADIUM_DEFAULT_ENGINE="claude"

# Override orchestration provider
export RADIUM_ORCHESTRATION_PROVIDER="gemini"

# Enable debug logging
export RUST_LOG="debug"
```

## Self-Hosted Model Configuration

### Ollama

```bash
export UNIVERSAL_BASE_URL="http://localhost:11434/v1"
export OLLAMA_MODEL="llama3.2"
```

### vLLM

```bash
export UNIVERSAL_BASE_URL="http://localhost:8000/v1"
export VLLM_MODEL="meta-llama/Llama-2-7b-chat-hf"
```

### LocalAI

```bash
export UNIVERSAL_BASE_URL="http://localhost:8080/v1"
export LOCALAI_MODEL="gpt-3.5-turbo"
```

**Learn more**: [Self-Hosted Models](../self-hosted/README.md)

## Extension Configuration

Extensions are configured via `radium-extension.json`:

```json
{
  "name": "my-extension",
  "version": "1.0.0",
  "description": "My extension",
  "author": "Your Name",
  "components": {
    "prompts": ["prompts/**/*.md"],
    "mcp_servers": ["mcp/*.json"],
    "commands": ["commands/*.toml"],
    "hooks": ["hooks/*.toml"]
  }
}
```

**Learn more**: [Extension System](../extensions/README.md)

## Context Sources Configuration

Context sources are configured in workspace or agent configs:

```toml
[context_sources]
files = ["docs/**/*.md", "README.md"]
http = ["https://api.example.com/docs"]
jira = { url = "https://jira.example.com", project = "PROJ" }
braingrid = { project_id = "PROJ-14" }
```

**Learn more**: [Context Sources](../user-guide/context-sources.md)

## Memory Configuration

Memory settings control how agent outputs are stored:

```toml
[memory]
enabled = true
retention_days = 30
max_entries_per_plan = 100
truncate_length = 2000
```

### Memory Settings

- **enabled**: Enable/disable memory system
- **retention_days**: How long to keep memory entries
- **max_entries_per_plan**: Maximum entries per plan
- **truncate_length**: Maximum length per entry

**Learn more**: [Memory & Context](../user-guide/memory-and-context.md)

## CLI Configuration

CLI behavior can be configured via environment variables:

```bash
# Output format
export RADIUM_OUTPUT_FORMAT="json"  # json, table, plain

# Verbose output
export RADIUM_VERBOSE="true"

# Color output
export RADIUM_COLOR="auto"  # auto, always, never
```

## Validation

Validate your configuration:

```bash
# Validate workspace config
rad workspace doctor

# Validate agent configs
rad agents validate

# Validate policy config
rad policy validate

# Validate extension configs
rad extension validate
```

## Configuration Precedence

Configuration is loaded in this order (later overrides earlier):

1. Default values
2. User config (`~/.radium/config.toml`)
3. Workspace config (`.radium/config.toml`)
4. Environment variables
5. Command-line arguments

## Best Practices

### Workspace-Specific Settings

- Store workspace-specific settings in `.radium/config.toml`
- Use environment variables for sensitive data (API keys)
- Version control workspace config (exclude sensitive data)

### Agent Organization

- Organize agents by category in subdirectories
- Use descriptive agent IDs and names
- Document agent purposes in descriptions

### Policy Configuration

- Start with restrictive policies
- Gradually relax as needed
- Use approval modes appropriately
- Document policy decisions

## Troubleshooting

### Configuration Not Loading

```bash
# Check config file location
ls -la .radium/config.toml

# Validate config syntax
rad workspace doctor
```

### Environment Variables Not Working

```bash
# Check if variables are set
echo $GOOGLE_AI_API_KEY

# Verify shell profile
cat ~/.bashrc | grep RADIUM
```

### Agent Not Found

```bash
# Check agent discovery
rad agents list

# Verify agent config location
find . -name "*.toml" -path "*/agents/*"
```

## Next Steps

- **[Core Concepts](./core-concepts.md)** - Understand Radium concepts
- **[Agent Configuration](../user-guide/agent-configuration.md)** - Detailed agent config
- **[Orchestration Configuration](../user-guide/orchestration-configuration.md)** - Orchestration setup
- **[Policy Engine](../features/policy-engine.md)** - Policy configuration

---

**Need help?** Check the [Troubleshooting Guide](../cli/troubleshooting.md) or [open an issue](https://github.com/clay-curry/RAD/issues).


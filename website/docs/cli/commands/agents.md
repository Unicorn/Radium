---
id: "agents"
title: "Agent Management Commands"
sidebar_label: "Agent Management Commands"
---

# Agent Management Commands

Commands for managing agents and workflow templates.

## `rad agents`

Manage agents in the workspace.

### Subcommands

#### `list`

List all discovered agents.

```bash
rad agents list [options]
```

Options:
- `--json` - Output as JSON

#### `search <query>`

Search agents by name or description.

```bash
rad agents search <query> [--json]
```

#### `info <id>`

Show detailed information about an agent.

```bash
rad agents info `<id>` [--json]
```

#### `validate <id>`

Validate agent configuration.

```bash
rad agents validate `<id>`
```

#### `create <id>`

Create a new agent template.

```bash
rad agents create `<id>`
```

### Examples

```bash
# List all agents
rad agents list

# Search for agents
rad agents search "code"

# Get agent info
rad agents info arch-agent

# Validate agent
rad agents validate arch-agent

# Create new agent
rad agents create my-agent
```

### Self-Hosted Model Configuration

Radium supports self-hosted models (Ollama, vLLM, LocalAI) via the Universal provider. Configure agents to use local models:

```toml
[agent]
id = "local-agent"
name = "Local Agent"
description = "Agent using self-hosted model"
prompt_path = "prompts/agents/my-agents/local-agent.md"
engine = "universal"
model = "llama3.2"
```

**Environment Setup:**
```bash
# Ollama
export UNIVERSAL_BASE_URL="http://localhost:11434/v1"

# vLLM
export UNIVERSAL_BASE_URL="http://localhost:8000/v1"

# LocalAI
export UNIVERSAL_BASE_URL="http://localhost:8080/v1"
```

See the [Self-Hosted Models Documentation](../../self-hosted-models/README.md) for complete setup guides and examples.

## `rad templates`

Manage workflow templates.

### Subcommands

#### `list`

List all workflow templates.

```bash
rad templates list [--json]
```

#### `info <id>`

Show template details.

```bash
rad templates info `<id>` [--json]
```

#### `validate <id>`

Validate template structure.

```bash
rad templates validate `<id>`
```

### Examples

```bash
# List templates
rad templates list

# Get template info
rad templates info basic-workflow

# Validate template
rad templates validate basic-workflow
```


---
id: "execution"
title: "Agent Execution Commands"
sidebar_label: "Agent Execution Commands"
---

# Agent Execution Commands

Commands for executing individual agents or agent scripts.

## `rad step`

Execute a single agent with optional input.

### Usage

```bash
rad step `<id>` [prompt...] [options]
```

### Arguments

- `id` - Agent ID from configuration
- `prompt` - Additional prompt to append (optional)

### Options

- `--model <model>` - Model to use (overrides agent config)
- `--engine <engine>` - Engine to use (overrides agent config)
- `--reasoning <level>` - Reasoning effort level (low, medium, high)

### Examples

```bash
# Execute agent with default prompt
rad step arch-agent

# Execute with additional prompt
rad step code-agent "Implement a REST API endpoint"

# Override model
rad step code-agent --model gpt-4

# Override engine
rad step code-agent --engine claude

# Set reasoning level
rad step code-agent --reasoning high
```

## `rad run`

Execute agent script with parallel (&) and sequential (&&) support.

### Usage

```bash
rad run <script> [options]
```

### Arguments

- `script` - Agent script (e.g., "agent-id 'prompt'" or "agent1 & agent2")

### Options

- `--model <model>` - Model to use
- `-d, --dir <path>` - Working directory

### Examples

```bash
# Run single agent
rad run "code-agent 'Implement feature X'"

# Run agents in parallel
rad run "agent1 'task1' & agent2 'task2'"

# Run agents sequentially
rad run "agent1 'task1' && agent2 'task2'"

# Run with specific model
rad run "agent-id 'prompt'" --model gpt-4

# Run in specific directory
rad run "agent-id 'prompt'" --dir /path/to/project
```

## `rad chat`

Interactive chat mode with session management.

### Usage

```bash
rad chat [agent-id] [options]
```

### Arguments

- `agent-id` - Agent ID to chat with (optional with --list)

### Options

- `--session <name>` - Session name (defaults to timestamp)
- `--resume` - Resume an existing session
- `--list` - List available sessions

### Examples

```bash
# Start chat session
rad chat code-agent

# Start named session
rad chat code-agent --session my-session

# Resume session
rad chat code-agent --resume

# List sessions
rad chat --list
```


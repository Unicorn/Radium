# Agent Configuration Guide

This guide explains how to configure AI agents in Radium using TOML configuration files.

## Introduction

Radium uses a declarative TOML-based configuration system for defining AI agents. Each agent is defined in a `.toml` file that specifies the agent's identity, capabilities, and behavior. Agents are automatically discovered from configured directories and can be managed through the CLI.

## Agent Configuration Format

### Basic Structure

An agent configuration file follows this structure:

```toml
[agent]
id = "my-agent"
name = "My Agent"
description = "A description of what this agent does"
prompt_path = "prompts/agents/my-category/my-agent.md"
```

### Required Fields

- **`id`** (string): Unique identifier for the agent (e.g., "arch-agent", "code-agent")
- **`name`** (string): Human-readable name for the agent
- **`description`** (string): Brief description of the agent's purpose
- **`prompt_path`** (PathBuf): Path to the markdown file containing the agent's prompt template

### Optional Fields

- **`engine`** (string): Default AI engine to use (e.g., "gemini", "openai", "claude", "universal" for self-hosted)
- **`model`** (string): Default model to use (e.g., "gemini-2.0-flash-exp", "gpt-4", "llama3.2" for self-hosted)
- **`reasoning_effort`** (string): Default reasoning effort level - `"low"`, `"medium"`, or `"high"` (default: `"medium"`)
- **`mirror_path`** (PathBuf): Optional mirror path for RAD-agents
- **`capabilities`** (object): Agent capabilities for dynamic model selection (see [Capabilities](#capabilities) section)
- **`sandbox`** (object): Sandbox configuration for safe command execution (see [Sandbox Configuration](#sandbox-configuration) section)

**Note**: For self-hosted models (Ollama, vLLM, LocalAI), use `engine = "universal"` and configure the base URL via environment variables. See the [Self-Hosted Models Guide](../self-hosted-models/README.md) for complete setup instructions.

### Example: Minimal Configuration

```toml
[agent]
id = "simple-agent"
name = "Simple Agent"
description = "A basic agent with minimal configuration"
prompt_path = "prompts/agents/core/simple-agent.md"
```

### Example: Full Configuration

```toml
[agent]
id = "arch-agent"
name = "Architecture Agent"
description = "Defines system architecture and technical design decisions"
prompt_path = "prompts/agents/core/arch-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"
reasoning_effort = "medium"
```

## Agent Behaviors

Agents can be configured with special behaviors that affect how they interact with workflows.

### Loop Behavior

Loop behavior allows an agent to request looping back to previous steps during workflow execution.

```toml
[agent]
id = "review-agent"
name = "Review Agent"
description = "Reviews code and can request revisions"
prompt_path = "prompts/agents/core/review-agent.md"

[agent.loop_behavior]
steps = 2              # Number of steps to go back
max_iterations = 5     # Maximum number of iterations (optional)
skip = ["step-1"]      # Step IDs to skip during loop (optional)
```

**Fields:**
- **`steps`** (usize, required): Number of steps to go back when looping
- **`max_iterations`** (usize, optional): Maximum number of iterations before stopping
- **`skip`** (array of strings, optional): List of step IDs to skip during loop

### Trigger Behavior

Trigger behavior allows an agent to dynamically trigger other agents during workflow execution.

```toml
[agent]
id = "coordinator-agent"
name = "Coordinator Agent"
description = "Coordinates work and can trigger other agents"
prompt_path = "prompts/agents/core/coordinator-agent.md"

[agent.trigger_behavior]
trigger_agent_id = "fallback-agent"  # Default agent to trigger (optional)
```

**Fields:**
- **`trigger_agent_id`** (string, optional): Default agent ID to trigger (can be overridden in behavior.json)

## Capabilities

Agent capabilities define the agent's model class, cost tier, and concurrency limits for dynamic model selection.

```toml
[agent]
id = "fast-agent"
name = "Fast Agent"
description = "Optimized for speed"
prompt_path = "prompts/agents/core/fast-agent.md"

[agent.capabilities]
model_class = "fast"        # Options: "fast", "balanced", "reasoning"
cost_tier = "low"           # Options: "low", "medium", "high"
max_concurrent_tasks = 10   # Maximum concurrent tasks (default: 5)
```

**Fields:**
- **`model_class`** (string, required): Model category - `"fast"` (speed-optimized), `"balanced"` (balanced speed/quality), or `"reasoning"` (deep reasoning)
- **`cost_tier`** (string, required): Cost tier - `"low"`, `"medium"`, or `"high"`
- **`max_concurrent_tasks`** (integer, optional): Maximum number of concurrent tasks (default: 5)

**Model Class Examples:**
- `"fast"`: Use with Flash, Mini, or other speed-optimized models
- `"balanced"`: Use with Pro, 4o, or other balanced models
- `"reasoning"`: Use with o1, Thinking, or other reasoning-focused models

If `capabilities` is not specified, defaults to `balanced/medium/5`.

## Sandbox Configuration

Sandbox configuration enables safe command execution in isolated environments. This is useful for agents that need to run potentially unsafe commands.

```toml
[agent]
id = "code-exec-agent"
name = "Code Execution Agent"
description = "Executes code in a sandbox"
prompt_path = "prompts/agents/core/code-exec-agent.md"

[agent.sandbox]
sandbox_type = "docker"            # Options: "docker", "podman", "seatbelt", "none"
network = "closed"                 # Network mode: "open", "closed", or "proxied"
profile = "restrictive"             # Sandbox profile: "permissive", "restrictive", or "custom(path)"
image = "rust:latest"              # Docker/Podman image (required for docker/podman)
working_dir = "/app"               # Working directory inside sandbox (optional)
volumes = ["/host:/container"]     # Volume mounts in host:container format (optional)
env = { "KEY" = "value" }          # Environment variables (optional)
custom_flags = ["--cap-add=SYS_ADMIN"]  # Custom flags for container execution (optional)
```

**Fields:**
- **`sandbox_type`** (string, required): Sandbox type - `"docker"`, `"podman"`, `"seatbelt"` (macOS only), or `"none"`
- **`network`** (string, optional): Network mode - `"open"`, `"closed"`, or `"proxied"` (default: `"open"`)
- **`profile`** (string, optional): Sandbox profile - `"permissive"`, `"restrictive"`, or `"custom(path)"` (default: `"permissive"`)
- **`image`** (string, optional): Container image for Docker/Podman sandboxes
- **`working_dir`** (string, optional): Working directory inside sandbox
- **`volumes`** (array of strings, optional): Volume mounts in `host:container` format
- **`env`** (table, optional): Environment variables as key-value pairs
- **`custom_flags`** (array of strings, optional): Additional flags for container execution

**Sandbox Types:**
- **`docker`**: Uses Docker containers for isolation (requires Docker)
- **`podman`**: Uses Podman containers for isolation (requires Podman)
- **`seatbelt`**: Uses macOS Seatbelt sandboxing (macOS only)
- **`none`**: No sandboxing (commands execute directly)

If `sandbox` is not specified, commands execute directly without sandboxing.

## Prompt Templates

Agent prompts are stored in markdown files and support placeholder replacement using `{{KEY}}` syntax.

### Placeholder Syntax

Placeholders in the format `{{KEY}}` are replaced with values from the execution context:

```markdown
# Architecture Agent

Hello {{user_name}}!

Your task is to {{task_description}}.

Please complete this by {{deadline}}.
```

### Template Loading

Prompt templates are automatically loaded from the path specified in `prompt_path`. The path can be:

- **Absolute**: `/absolute/path/to/prompt.md`
- **Relative**: `prompts/agents/core/arch-agent.md` (relative to workspace root)

## Creating Custom Agents

### Step 1: Create Agent Configuration

Create a new TOML file in the appropriate directory:

```bash
# Project-local agents
./agents/my-category/my-agent.toml

# User-level agents
~/.radium/agents/my-category/my-agent.toml
```

### Step 2: Define Agent Configuration

```toml
[agent]
id = "my-agent"
name = "My Custom Agent"
description = "Does something useful"
prompt_path = "prompts/agents/my-category/my-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"
```

### Step 3: Create Prompt Template

Create the prompt file at the specified path:

```markdown
# My Custom Agent

## Role

Define the agent's role and primary responsibilities here.

## Capabilities

- List the agent's core capabilities
- Include what tasks it can perform
- Specify any constraints or limitations

## Instructions

Provide step-by-step instructions for the agent:

1. First step - explain what to do
2. Second step - detail the process
3. Continue as needed...
```

### Step 4: Validate Configuration

Use the CLI to validate your agent:

```bash
rad agents validate
```

Or validate a specific agent:

```bash
rad agents info my-agent
```

### Step 5: Test the Agent

List all agents to verify discovery:

```bash
rad agents list
```

## Using the CLI

### List All Agents

```bash
rad agents list
```

Show detailed information:

```bash
rad agents list --verbose
```

### Search Agents

```bash
rad agents search "architecture"
```

### Get Agent Information

```bash
rad agents info arch-agent
```

### Validate Agents

```bash
rad agents validate
```

### Create New Agent

```bash
rad agents create my-agent "My Agent" \
  --description "Agent description" \
  --category custom \
  --engine gemini \
  --model gemini-2.0-flash-exp \
  --reasoning medium
```

This command will:
1. Create the agent configuration file
2. Create a prompt template file
3. Set up the directory structure

## Agent Discovery

Agents are automatically discovered from multiple directories in this order (precedence from highest to lowest):

1. **Project-local agents**: `./agents/`
2. **User agents**: `~/.radium/agents/`
3. **Workspace agents**: (if applicable)
4. **Project-level extension agents**: `./.radium/extensions/*/agents/`
5. **User-level extension agents**: `~/.radium/extensions/*/agents/`

### Category Derivation

The agent's category is automatically derived from the directory structure:

- `agents/core/arch-agent.toml` → category: `"core"`
- `agents/custom/my-agent.toml` → category: `"custom"`
- `agents/rad-agents/design/design-agent.toml` → category: `"rad-agents/design"`

### Duplicate Agent IDs

If multiple agents have the same ID, the agent from the directory with higher precedence will be used. Later entries override earlier ones.

## Common Patterns

### Pattern 1: Simple Task Agent

```toml
[agent]
id = "task-agent"
name = "Task Agent"
description = "Performs a specific task"
prompt_path = "prompts/agents/core/task-agent.md"
```

### Pattern 2: Agent with Default Model

```toml
[agent]
id = "model-agent"
name = "Model Agent"
description = "Uses a specific model"
prompt_path = "prompts/agents/core/model-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"
reasoning_effort = "high"
```

### Pattern 3: Agent with Loop Behavior

```toml
[agent]
id = "iterative-agent"
name = "Iterative Agent"
description = "Can loop back to previous steps"
prompt_path = "prompts/agents/core/iterative-agent.md"

[agent.loop_behavior]
steps = 2
max_iterations = 3
```

### Pattern 4: Agent with Trigger Behavior

```toml
[agent]
id = "coordinator"
name = "Coordinator Agent"
description = "Can trigger other agents"
prompt_path = "prompts/agents/core/coordinator.md"

[agent.trigger_behavior]
trigger_agent_id = "worker-agent"
```

### Pattern 5: Agent with Capabilities

```toml
[agent]
id = "fast-code-gen"
name = "Fast Code Generator"
description = "Generates code quickly"
prompt_path = "prompts/agents/core/fast-code-gen.md"

[agent.capabilities]
model_class = "fast"
cost_tier = "low"
max_concurrent_tasks = 20
```

### Pattern 6: Agent with Sandbox

```toml
[agent]
id = "safe-exec"
name = "Safe Execution Agent"
description = "Executes commands in sandbox"
prompt_path = "prompts/agents/core/safe-exec.md"

[agent.sandbox]
type = "docker"
image = "ubuntu:latest"
profile = "restricted"
```

## Best Practices

1. **Use descriptive IDs**: Choose IDs that clearly indicate the agent's purpose (e.g., `arch-agent` not `agent1`)

2. **Organize by category**: Group related agents in the same category directory

3. **Write clear descriptions**: Help users understand when to use each agent

4. **Specify default models**: Set `engine` and `model` if the agent requires a specific AI provider

5. **Use placeholder templates**: Make prompts reusable with `{{KEY}}` placeholders

6. **Validate regularly**: Run `rad agents validate` to catch configuration errors early

7. **Version control**: Commit agent configurations and prompts to version control

## Troubleshooting

### Agent Not Found

**Problem**: Agent doesn't appear in `rad agents list`

**Solutions**:
- Check that the agent file is in a discovery directory (`./agents/` or `~/.radium/agents/`)
- Verify the file has a `.toml` extension
- Check that the `id` field is unique
- Run `rad agents validate` to check for errors

### Prompt File Not Found

**Problem**: Validation error: "Prompt file not found"

**Solutions**:
- Verify the `prompt_path` in the agent config is correct
- Check if the path is relative (relative to workspace root) or absolute
- Ensure the prompt file exists at the specified path
- Check file permissions

### Invalid TOML Syntax

**Problem**: Error parsing TOML file

**Solutions**:
- Validate TOML syntax using a TOML validator
- Check for missing quotes around string values
- Ensure all required fields are present
- Verify array syntax for `skip` field in loop behavior

### Duplicate Agent IDs

**Problem**: Agent from one directory is overriding another

**Solutions**:
- Use unique IDs for each agent
- Check discovery order (project-local has highest precedence)
- Use different categories to organize agents
- Consider using namespaced IDs (e.g., `my-extension:agent-id`)

### Placeholder Not Replaced

**Problem**: `{{KEY}}` placeholders remain in rendered prompt

**Solutions**:
- Verify placeholder syntax uses double braces: `{{KEY}}`
- Check that the context provides values for all placeholders
- Use non-strict mode if placeholders are optional
- Verify placeholder names match exactly (case-sensitive)

### Invalid Capabilities Configuration

**Problem**: Validation error with capabilities section

**Solutions**:
- Verify `model_class` is one of: "fast", "balanced", "reasoning"
- Verify `cost_tier` is one of: "low", "medium", "high"
- Ensure `max_concurrent_tasks` is a positive integer
- Check TOML syntax for the `[agent.capabilities]` section

### Sandbox Configuration Issues

**Problem**: Sandbox not working or validation errors

**Solutions**:
- Verify sandbox type is supported on your platform (seatbelt is macOS-only)
- For Docker/Podman: Ensure the container runtime is installed and running
- Check that the specified image exists and is accessible
- Verify network_mode is valid: "isolated", "bridged", or "host"
- Test sandbox configuration with a simple command first

## Examples

See the `examples/agents/` directory for complete example configurations:

- `simple-agent.toml` - Minimal configuration
- `full-agent.toml` - All optional fields
- `loop-behavior-agent.toml` - Agent with loop behavior
- `trigger-behavior-agent.toml` - Agent with trigger behavior

## Further Reading

- [Self-Hosted Models Guide](../self-hosted-models/README.md) - Setup and configuration for Ollama, vLLM, and LocalAI
- [Developer Guide: Agent System Architecture](../developer-guide/agent-system-architecture.md) - Technical details for developers
- [CLI Reference](../../README.md#agent-configuration) - Command-line interface documentation


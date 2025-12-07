# Orchestration User Guide

## Introduction

Radium's orchestration system allows you to interact naturally with your AI assistants without manually selecting which agent to use for each task. The orchestrator automatically analyzes your input, routes tasks to appropriate specialist agents, coordinates multi-agent workflows, and synthesizes results - all while remaining completely model-agnostic.

### What is Orchestration?

Orchestration is Radium's intelligent task routing system that:
- **Automatically selects** the right agent(s) for each task
- **Coordinates workflows** involving multiple agents
- **Synthesizes results** from agent interactions
- **Works across providers** (Gemini, Claude, OpenAI, prompt-based fallback)

Instead of typing `/chat senior-developer` and then your request, you can simply type your request naturally, and the orchestrator will route it to the appropriate agent automatically.

## Getting Started

### First-Time Setup

Orchestration is **enabled by default** when you start Radium TUI. On first run, Radium will create a default configuration file at `~/.radium/orchestration.toml`.

If orchestration isn't working:

1. **Check API keys**: Ensure you have at least one API key set:
   ```bash
   export GEMINI_API_KEY="your-key-here"
   # or
   export ANTHROPIC_API_KEY="your-key-here"
   # or
   export OPENAI_API_KEY="your-key-here"
   ```

2. **Verify orchestration status**:
   ```
   /orchestrator
   ```

3. **Enable if needed**:
   ```
   /orchestrator toggle
   ```

### Quick Start Example

```
You: I need to refactor the authentication module

🤔 Analyzing...
📋 Invoking: senior-developer
✅ Complete (2.3s)

Assistant: I've analyzed your authentication module and identified several areas for improvement...
```

## Natural Conversation

### How It Works

When orchestration is enabled, you can type naturally in the TUI without any command prefixes:

- ✅ **"I need help with authentication"** → Routes to appropriate agent
- ✅ **"Refactor the user service module"** → Routes to developer agent
- ✅ **"Create tests for the API endpoints"** → Routes to testing agent
- ✅ **"Document the database schema"** → Routes to documentation agent

The orchestrator automatically:
1. Analyzes your intent
2. Selects the best agent(s) for the task
3. Executes the task
4. Returns synthesized results

### Commands vs. Orchestration

**Commands** (starting with `/`) always bypass orchestration:
- `/chat agent-name` - Direct chat with specific agent
- `/agents` - List available agents
- `/orchestrator` - Orchestration configuration

**Natural input** (no `/`) routes through orchestration if enabled.

## Commands

### `/orchestrator` - Show Status

Display current orchestration configuration:

```
/orchestrator
```

Output shows:
- Enabled/disabled state
- Current provider and model
- Service initialization status

### `/orchestrator toggle` - Enable/Disable

Toggle orchestration on or off:

```
/orchestrator toggle
```

When disabled, natural language input will fall back to regular chat mode. Use this if you prefer explicit agent selection.

### `/orchestrator switch <provider>` - Change Provider

Switch between AI providers:

```
/orchestrator switch gemini
/orchestrator switch claude
/orchestrator switch openai
/orchestrator switch prompt_based
```

Available providers:
- **gemini** - Google Gemini models (default)
- **claude** - Anthropic Claude models
- **openai** - OpenAI GPT models
- **prompt_based** - Prompt-based fallback (no API key required)

Provider changes are automatically saved to your configuration file.

### `/orchestrator config` - Show Full Configuration

Display complete configuration details:

```
/orchestrator config
```

Shows all provider settings, model configurations, temperature, iteration limits, and fallback settings.

### `/orchestrator refresh` - Reload Agent Registry

Reload all agent tool definitions:

```
/orchestrator refresh
```

Use this after adding, modifying, or removing agent configuration files. The orchestrator will discover and use the updated agents.

## Configuration

Orchestration configuration supports both workspace-based and home directory settings. See the [Configuration Guide](./orchestration-configuration.md) for complete details.

### Configuration File Locations

Orchestration settings are stored in (in priority order):
1. **Workspace config:** `.radium/config/orchestration.toml` (preferred)
2. **Home directory:** `~/.radium/orchestration.toml` (fallback)

### Quick Configuration Reference

```toml
[orchestration]
enabled = true
default_provider = "gemini"

[orchestration.gemini]
model = "gemini-2.0-flash-thinking-exp"
temperature = 0.7
max_tool_iterations = 5

[orchestration.claude]
model = "claude-3-5-sonnet-20241022"
temperature = 0.7
max_tool_iterations = 5
max_tokens = 4096

[orchestration.openai]
model = "gpt-4-turbo-preview"
temperature = 0.7
max_tool_iterations = 5

[orchestration.prompt_based]
temperature = 0.7
max_tool_iterations = 5

[orchestration.fallback]
enabled = true
chain = ["gemini", "claude", "openai", "prompt_based"]
max_retries = 2
```

### Configuration Options

**Global Settings:**
- `enabled` - Enable/disable orchestration (boolean)
- `default_provider` - Primary provider to use (`gemini`, `claude`, `openai`, `prompt_based`)

**Provider Settings:**
- `model` - Model identifier (provider-specific)
- `temperature` - Generation temperature (0.0-1.0)
- `max_tool_iterations` - Maximum tool execution iterations (default: 5)
- `max_tokens` - Maximum output tokens (Claude only)

**Fallback Settings:**
- `enabled` - Enable automatic fallback (boolean)
- `chain` - Fallback provider order (array)
- `max_retries` - Maximum retries per provider (default: 2)

## Providers

### Gemini (Google)

**Best for:** Fast responses, general tasks, cost-effective

**Requirements:** `GEMINI_API_KEY` environment variable

**Default Model:** `gemini-2.0-flash-thinking-exp`

**Configuration:**
```toml
[orchestration.gemini]
model = "gemini-2.0-flash-thinking-exp"
temperature = 0.7
max_tool_iterations = 5
```

### Claude (Anthropic)

**Best for:** Complex reasoning, code analysis, high-quality outputs

**Requirements:** `ANTHROPIC_API_KEY` environment variable

**Default Model:** `claude-3-5-sonnet-20241022`

**Configuration:**
```toml
[orchestration.claude]
model = "claude-3-5-sonnet-20241022"
temperature = 0.7
max_tool_iterations = 5
max_tokens = 4096
```

### OpenAI

**Best for:** GPT-4 capabilities, general purpose

**Requirements:** `OPENAI_API_KEY` environment variable

**Default Model:** `gpt-4-turbo-preview`

**Configuration:**
```toml
[orchestration.openai]
model = "gpt-4-turbo-preview"
temperature = 0.7
max_tool_iterations = 5
```

### Prompt-Based Fallback

**Best for:** Offline use, testing, when API keys unavailable

**Requirements:** None (uses local model abstraction)

**Configuration:**
```toml
[orchestration.prompt_based]
temperature = 0.7
max_tool_iterations = 5
```

## Troubleshooting

For detailed troubleshooting, see the [Troubleshooting Guide](./orchestration-troubleshooting.md).

### Quick Troubleshooting

**Symptoms:** Natural input doesn't trigger orchestration

**Solutions:**
1. Check orchestration is enabled: `/orchestrator`
2. Verify API keys are set: `echo $GEMINI_API_KEY`
3. Check service initialization in status output
4. Try toggling: `/orchestrator toggle` (off then on)
5. Check logs for initialization errors

### Wrong Agent Selected

**Symptoms:** Orchestrator routes to incorrect agent

**Solutions:**
1. Be more specific in your request
2. Include context about what you need
3. Try different phrasings
4. Use explicit `/chat agent-name` if needed
5. Check available agents: `/agents`

### Provider Errors

**Symptoms:** "Orchestration error" or API failures

**Solutions:**
1. Verify API key is valid and set
2. Check API service status
3. Switch to different provider: `/orchestrator switch claude`
4. Enable fallback in config (should be enabled by default)
5. Try prompt-based fallback: `/orchestrator switch prompt_based`

### Timeout Errors

**Symptoms:** "Finished with: max_iterations" or timeouts

**Solutions:**
1. Break task into smaller requests
2. Increase `max_tool_iterations` in config
3. Simplify your request
4. Check network connectivity

### Slow Performance

**Symptoms:** Orchestration takes too long

**Solutions:**
1. Switch to faster provider (Gemini Flash)
2. Reduce `max_tool_iterations` if excessive
3. Check network latency
4. Simplify requests
5. Check if multiple agents are being invoked unnecessarily

## Tips for Best Results

1. **Be Specific**: Clear, specific requests route better
   - ✅ "Refactor the authentication service to use JWT tokens"
   - ❌ "Fix auth"

2. **Provide Context**: Include relevant information
   - ✅ "Update the User model to add email verification field"
   - ❌ "Add email to user"

3. **One Task at a Time**: Break complex workflows into steps
   - ✅ "Design the database schema for user profiles"
   - ❌ "Build the entire user profile system with auth, database, API, and frontend"

4. **Use Commands for Explicit Control**: When you know exactly which agent you want
   - `/chat senior-developer` for development tasks
   - `/chat tester` for testing tasks

## Examples

See [orchestration-workflows.md](../examples/orchestration-workflows.md) for detailed workflow examples.

## Advanced Usage

### Multi-Agent Workflows

The orchestrator can coordinate multiple agents for complex tasks:

```
You: Create a new feature for task templates

🤔 Analyzing...
1. 📐 product-manager - Define feature requirements
2. 🏗️ architect - Design implementation approach
3. 💻 senior-developer - Implement feature
4. 🧪 tester - Create test suite

Executing 4 agents...
✅ Complete (15.2s)
```

### Custom Agent Routing

Agents are automatically discovered from:
- `./agents/` - Project-local agents
- `~/.radium/agents/` - User agents

Agent descriptions and capabilities are used by the orchestrator to route tasks. See [agent-creation-guide.md](../guides/agent-creation-guide.md) for creating custom agents.

### Performance Optimization

- **Provider Selection**: Choose faster providers (Gemini Flash) for quick tasks
- **Iteration Limits**: Adjust `max_tool_iterations` to prevent long-running workflows
- **Fallback Chain**: Configure fallback order for reliability

## Related Documentation

- [Orchestration Configuration Guide](./orchestration-configuration.md) - Complete configuration reference
- [Orchestration Troubleshooting Guide](./orchestration-troubleshooting.md) - Common issues and solutions
- [Orchestration Testing Guide](./orchestration-testing.md) - Manual testing procedures
- [Agent Configuration](./agent-configuration.md) - Agent setup
- [Orchestration Workflows](../examples/orchestration-workflows.md) - Example workflows
- [Agent Creation Guide](../guides/agent-creation-guide.md) - Creating custom agents


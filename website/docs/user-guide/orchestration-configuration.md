---
id: "orchestration-configuration"
title: "Orchestration Configuration Guide"
sidebar_label: "Orchestration Configuration Guide"
---

# Orchestration Configuration Guide

This guide provides detailed information about configuring Radium's orchestration system.

## Configuration File Locations

Radium supports two configuration file locations, with workspace-based config taking precedence:

1. **Workspace Configuration** (preferred): `.radium/config/orchestration.toml`
   - Used when working within a Radium workspace
   - Project-specific settings
   - Shared with team members (if committed to version control)

2. **Home Directory Configuration** (fallback): `~/.radium/orchestration.toml`
   - Used when no workspace is found
   - User-specific settings
   - Applies to all projects

### Configuration Priority

When Radium starts:
1. First, attempts to load from workspace: `.radium/config/orchestration.toml`
2. If workspace config not found, loads from home directory: `~/.radium/orchestration.toml`
3. If neither exists, uses default configuration

Configuration changes via `/orchestrator` commands are saved to the workspace config if available, otherwise to the home directory config.

## Configuration Format

The orchestration configuration uses TOML format:

```toml
[orchestration]
enabled = true
default_provider = "gemini"

[orchestration.gemini]
model = "gemini-2.0-flash-thinking-exp"
temperature = 0.7
max_tool_iterations = 5
api_endpoint = null

[orchestration.claude]
model = "claude-3-5-sonnet-20241022"
temperature = 0.7
max_tool_iterations = 5
max_tokens = 4096
api_endpoint = null

[orchestration.openai]
model = "gpt-4-turbo-preview"
temperature = 0.7
max_tool_iterations = 5
api_endpoint = null

[orchestration.prompt_based]
temperature = 0.7
max_tool_iterations = 5

[orchestration.fallback]
enabled = true
chain = ["gemini", "claude", "openai", "prompt_based"]
max_retries = 2
```

## Global Settings

### `enabled`

Enable or disable orchestration globally.

- **Type:** `boolean`
- **Default:** `true`
- **Example:** `enabled = false`

When disabled, natural language input falls back to regular chat mode. Use `/orchestrator toggle` to change at runtime.

### `default_provider`

Primary AI provider to use for orchestration.

- **Type:** `string` (one of: `gemini`, `claude`, `openai`, `prompt_based`)
- **Default:** `gemini`
- **Example:** `default_provider = "claude"`

This provider is used unless function calling fails and fallback is enabled.

## Provider-Specific Settings

### Gemini Configuration

```toml
[orchestration.gemini]
model = "gemini-2.0-flash-thinking-exp"
temperature = 0.7
max_tool_iterations = 5
api_endpoint = null
```

#### `model`

Gemini model identifier.

- **Type:** `string`
- **Default:** `"gemini-2.0-flash-thinking-exp"`
- **Options:**
  - `gemini-2.0-flash-thinking-exp` - Fast, cost-effective (recommended)
  - `gemini-1.5-pro` - More capable, slower
  - `gemini-1.5-flash` - Fast, good balance

#### `temperature`

Generation temperature (creativity/randomness).

- **Type:** `float` (0.0 to 1.0)
- **Default:** `0.7`
- **Lower values (0.0-0.5):** More deterministic, focused
- **Higher values (0.7-1.0):** More creative, varied

#### `max_tool_iterations`

Maximum number of tool execution iterations per request.

- **Type:** `integer`
- **Default:** `5`
- **Range:** 1-10 (recommended: 3-5)

Higher values allow more complex multi-step workflows but increase latency and cost.

#### `api_endpoint`

Optional API endpoint override (for custom deployments).

- **Type:** `string` or `null`
- **Default:** `null`
- **Example:** `api_endpoint = "https://custom-gemini-api.example.com"`

### Claude Configuration

```toml
[orchestration.claude]
model = "claude-3-5-sonnet-20241022"
temperature = 0.7
max_tool_iterations = 5
max_tokens = 4096
api_endpoint = null
```

#### `model`

Claude model identifier.

- **Type:** `string`
- **Default:** `"claude-3-5-sonnet-20241022"`
- **Options:**
  - `claude-3-5-sonnet-20241022` - Best balance (recommended)
  - `claude-3-opus-20240229` - Most capable, expensive
  - `claude-3-haiku-20240307` - Fastest, least capable

#### `max_tokens`

Maximum output tokens for Claude responses.

- **Type:** `integer`
- **Default:** `4096`
- **Range:** 1-4096

Controls response length. Higher values allow longer outputs but increase cost.

### OpenAI Configuration

```toml
[orchestration.openai]
model = "gpt-4-turbo-preview"
temperature = 0.7
max_tool_iterations = 5
api_endpoint = null
```

#### `model`

OpenAI model identifier.

- **Type:** `string`
- **Default:** `"gpt-4-turbo-preview"`
- **Options:**
  - `gpt-4-turbo-preview` - Latest GPT-4 (recommended)
  - `gpt-4` - Standard GPT-4
  - `gpt-3.5-turbo` - Faster, less capable

### Prompt-Based Configuration

```toml
[orchestration.prompt_based]
temperature = 0.7
max_tool_iterations = 5
```

Prompt-based orchestration doesn't require API keys and uses local model abstraction. Useful for:
- Offline development
- Testing orchestration logic
- When API keys are unavailable

**Note:** Prompt-based orchestration has limited capabilities compared to native function calling providers.

## Fallback Configuration

```toml
[orchestration.fallback]
enabled = true
chain = ["gemini", "claude", "openai", "prompt_based"]
max_retries = 2
```

### `enabled`

Enable automatic fallback when primary provider fails.

- **Type:** `boolean`
- **Default:** `true`

When enabled, if the primary provider fails (e.g., function calling error), Radium automatically tries the next provider in the fallback chain.

### `chain`

Order of providers to try during fallback.

- **Type:** `array` of provider names
- **Default:** `["gemini", "claude", "openai", "prompt_based"]`
- **Example:** `chain = ["claude", "gemini", "prompt_based"]`

Providers are tried in order until one succeeds or all fail.

### `max_retries`

Maximum retry attempts per provider in fallback chain.

- **Type:** `integer`
- **Default:** `2`
- **Range:** 1-5

## Example Configurations

### Fast and Cost-Effective

```toml
[orchestration]
enabled = true
default_provider = "gemini"

[orchestration.gemini]
model = "gemini-2.0-flash-thinking-exp"
temperature = 0.5
max_tool_iterations = 3
```

**Use case:** Quick tasks, high-volume usage, cost-sensitive projects.

### High Quality

```toml
[orchestration]
enabled = true
default_provider = "claude"

[orchestration.claude]
model = "claude-3-5-sonnet-20241022"
temperature = 0.7
max_tool_iterations = 5
max_tokens = 8192
```

**Use case:** Complex reasoning, code analysis, high-quality outputs.

### Balanced

```toml
[orchestration]
enabled = true
default_provider = "gemini"

[orchestration.gemini]
model = "gemini-2.0-flash-thinking-exp"
temperature = 0.7
max_tool_iterations = 5

[orchestration.fallback]
enabled = true
chain = ["gemini", "claude", "prompt_based"]
max_retries = 2
```

**Use case:** General development, good balance of speed and quality.

### Development/Testing

```toml
[orchestration]
enabled = true
default_provider = "prompt_based"

[orchestration.prompt_based]
temperature = 0.7
max_tool_iterations = 3

[orchestration.fallback]
enabled = false
```

**Use case:** Offline development, testing orchestration logic without API costs.

## Environment Variables

API keys are configured via environment variables (not in config file for security):

- **Gemini:** `GEMINI_API_KEY`
- **Claude:** `ANTHROPIC_API_KEY`
- **OpenAI:** `OPENAI_API_KEY`

Set these before starting Radium:

```bash
export GEMINI_API_KEY="your-key-here"
export ANTHROPIC_API_KEY="your-key-here"
export OPENAI_API_KEY="your-key-here"
```

Or add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
# Add to ~/.zshrc or ~/.bashrc
export GEMINI_API_KEY="your-key-here"
export ANTHROPIC_API_KEY="your-key-here"
export OPENAI_API_KEY="your-key-here"
```

## Configuration Management

### Viewing Configuration

View current configuration in TUI:

```
/orchestrator config
```

This displays all configuration options including provider settings, fallback chain, and current values.

### Changing Configuration

#### Via TUI Commands

1. **Switch provider:**
   ```
   /orchestrator switch claude
   ```

2. **Toggle orchestration:**
   ```
   /orchestrator toggle
   ```

3. **Refresh agent registry:**
   ```
   /orchestrator refresh
   ```

Changes are automatically saved to the configuration file.

#### Via Configuration File

1. Edit the configuration file:
   ```bash
   # Workspace config (preferred)
   nano .radium/config/orchestration.toml
   
   # Or home directory config
   nano ~/.radium/orchestration.toml
   ```

2. Restart Radium TUI for changes to take effect.

**Note:** Changes via TUI commands override manual file edits. Always use TUI commands when possible.

## Performance Tuning

### Reducing Latency

1. **Use faster models:**
   - Gemini: `gemini-2.0-flash-thinking-exp`
   - Claude: `claude-3-haiku-20240307`
   - OpenAI: `gpt-3.5-turbo`

2. **Lower iteration limits:**
   ```toml
   max_tool_iterations = 3
   ```

3. **Reduce temperature:**
   ```toml
   temperature = 0.5
   ```

### Improving Quality

1. **Use more capable models:**
   - Claude: `claude-3-5-sonnet-20241022`
   - OpenAI: `gpt-4-turbo-preview`

2. **Increase iteration limits:**
   ```toml
   max_tool_iterations = 7
   ```

3. **Higher temperature for creativity:**
   ```toml
   temperature = 0.9
   ```

### Cost Optimization

1. **Use cost-effective models:**
   - Gemini Flash (lowest cost)
   - Claude Haiku
   - GPT-3.5 Turbo

2. **Limit iterations:**
   ```toml
   max_tool_iterations = 3
   ```

3. **Disable fallback if not needed:**
   ```toml
   [orchestration.fallback]
   enabled = false
   ```

## Troubleshooting Configuration

### Configuration Not Loading

**Symptoms:** Changes not taking effect

**Solutions:**
1. Check file location (workspace vs home directory)
2. Verify TOML syntax is valid
3. Check file permissions (should be readable)
4. Restart Radium TUI

### Invalid Configuration Values

**Symptoms:** Errors on startup or runtime

**Solutions:**
1. Check TOML syntax
2. Verify provider names are correct
3. Ensure numeric values are in valid ranges
4. Check for typos in model names

### Configuration File Not Found

**Symptoms:** Using defaults instead of custom config

**Solutions:**
1. Create configuration file manually
2. Use `/orchestrator` commands to generate default config
3. Check file path is correct

## Related Documentation

- [Orchestration User Guide](./orchestration.md) - Complete user guide
- [Orchestration Troubleshooting](./orchestration-troubleshooting.md) - Common issues and solutions
- [Orchestration Workflows](../examples/orchestration-workflows.md) - Example workflows


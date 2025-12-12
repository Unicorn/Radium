---
id: "configuration"
title: "Hooks Configuration Reference"
sidebar_label: "Hooks Config Ref"
---

# Hooks Configuration Reference

Complete reference for configuring hooks in Radium using TOML configuration files.

## Overview

Hooks can be configured in two locations:
1. **Workspace Configuration**: `.radium/hooks.toml` in your workspace root
2. **Extension Configuration**: `hooks.toml` in extension directories (automatically discovered)

Workspace configurations take precedence over extension configurations for hooks with the same name.

## Configuration File Format

Hooks are configured using TOML format with a `[[hooks]]` array of hook definitions:

```toml
[[hooks]]
name = "my-hook"
type = "before_model"
priority = 100
enabled = true
script = "hooks/my-hook.rs"

[hooks.config]
custom_option = "value"
```

## Hook Definition Fields

### Required Fields

#### `name` (string)
Unique identifier for the hook within the configuration context.

- **Required**: Yes
- **Example**: `"logging-hook"`, `"metrics-collector"`
- **Constraints**: Cannot be empty
- **Validation**: Must be unique within the configuration file

#### `type` (string)
Hook type that determines when and how the hook executes.

- **Required**: Yes
- **Valid Values**:
  - `"before_model"` - Execute before model calls
  - `"after_model"` - Execute after model calls
  - `"before_tool"` - Execute before tool execution
  - `"after_tool"` - Execute after tool execution
  - `"tool_selection"` - Execute during tool selection
  - `"error_interception"` - Intercept errors before propagation
  - `"error_transformation"` - Transform error messages
  - `"error_recovery"` - Attempt error recovery
  - `"error_logging"` - Log errors with custom formatting
  - `"telemetry_collection"` - Collect and aggregate telemetry
  - `"custom_logging"` - Custom logging hooks
  - `"metrics_aggregation"` - Aggregate metrics
  - `"performance_monitoring"` - Monitor performance

### Optional Fields

#### `priority` (integer)
Execution priority for the hook. Higher priority hooks execute first.

- **Required**: No
- **Default**: `100`
- **Range**: Any positive integer
- **Conventions**:
  - `200+` - Critical hooks that must run first
  - `100-199` - Standard hooks (default)
  - `<100` - Optional hooks

#### `enabled` (boolean)
Whether the hook is enabled and will be executed.

- **Required**: No
- **Default**: `true`
- **Note**: Disabled hooks are still registered but not executed

#### `script` (string)
Path to the hook implementation script or library.

- **Required**: If `config` is not provided (at least one of `script` or `config` must be set)
- **Example**: `"hooks/logging.rs"`, `"extensions/my-extension/hooks/cache.rs"`
- **Note**: For programmatic hooks, this can be omitted if using `config` with inline configuration

#### `config` (table)
Inline configuration for the hook. Used for hooks that don't require external scripts.

- **Required**: If `script` is not provided (at least one of `script` or `config` must be set)
- **Type**: TOML table (`[hooks.config]`)
- **Example**:
  ```toml
  [hooks.config]
  log_level = "info"
  max_retries = 3
  timeout = 5000
  ```

## Configuration Examples

### Basic Model Hook

```toml
[[hooks]]
name = "input-validator"
type = "before_model"
priority = 200
enabled = true
script = "hooks/validate-input.rs"

[hooks.config]
max_length = 10000
allow_empty = false
```

### Telemetry Collection Hook

```toml
[[hooks]]
name = "cost-tracker"
type = "telemetry_collection"
priority = 100
enabled = true

[hooks.config]
track_tokens = true
track_costs = true
output_file = "logs/costs.json"
```

### Error Handling Hook

```toml
[[hooks]]
name = "error-recovery"
type = "error_recovery"
priority = 150
enabled = true
script = "hooks/recover-errors.rs"

[hooks.config]
max_retries = 3
backoff_multiplier = 2.0
retryable_errors = ["timeout", "rate_limit"]
```

### Multiple Hooks in One File

```toml
# Before model hook
[[hooks]]
name = "request-logger"
type = "before_model"
priority = 100
enabled = true
script = "hooks/logging.rs"

[hooks.config]
log_level = "info"

# After model hook
[[hooks]]
name = "response-processor"
type = "after_model"
priority = 100
enabled = true
script = "hooks/process-response.rs"

# Tool execution hook
[[hooks]]
name = "tool-validator"
type = "before_tool"
priority = 150
enabled = true
script = "hooks/validate-tool.rs"
```

### Extension Configuration

In extension directories (e.g., `extensions/my-extension/hooks.toml`):

```toml
[[hooks]]
name = "extension-metrics"
type = "metrics_aggregation"
priority = 50
enabled = true

[hooks.config]
namespace = "my-extension"
export_to = "prometheus"
```

## Validation Rules

The configuration system validates hooks when they are loaded. Validation errors prevent the hook from being registered.

### Name Validation
- ❌ Empty string
- ✅ Non-empty string
- ⚠️ Duplicate names in the same file will overwrite previous definitions

### Type Validation
- ❌ Empty string
- ❌ Invalid hook type (not in the 13 valid types)
- ✅ One of the 13 valid hook types

### Script/Config Validation
- ❌ Neither `script` nor `config` provided
- ✅ At least one of `script` or `config` provided
- ✅ Both provided (script takes precedence)

### Priority Validation
- ✅ Any positive integer (no maximum enforced)
- ⚠️ Negative numbers are accepted but not recommended

## Configuration Discovery

Hooks are automatically discovered from:

1. **Workspace Configuration**: `.radium/hooks.toml`
   - Highest priority
   - Applies to all agents in the workspace

2. **Extension Configurations**: `extensions/*/hooks.toml`
   - Discovered automatically
   - Lower priority than workspace config
   - Hooks with same name override extension hooks

3. **Programmatic Registration**: Hooks registered directly in code
   - Can be enabled/disabled via configuration
   - Name must match configuration entry

## Enabling and Disabling Hooks

Hooks can be enabled or disabled in two ways:

### Via Configuration File

Edit `.radium/hooks.toml` and set `enabled = false`:

```toml
[[hooks]]
name = "my-hook"
type = "before_model"
enabled = false  # Disabled
```

### Via CLI Commands

```bash
# Disable a hook
rad hooks disable my-hook

# Enable a hook
rad hooks enable my-hook
```

The CLI updates both the registry and the configuration file automatically.

## Validation Command

Validate all hook configurations:

```bash
rad hooks validate
```

This command:
- Loads all hook configurations from workspace and extensions
- Runs validation checks on each hook
- Reports any validation errors
- Provides helpful error messages for fixing issues

## Configuration Best Practices

### Naming Conventions
- Use descriptive names: `"model-request-logger"` not `"hook1"`
- Include hook type in name if useful: `"before-tool-validator"`
- Use kebab-case for consistency

### Priority Guidelines
- Reserve priorities 200+ for critical system hooks
- Use 100-199 for standard application hooks
- Use &lt;100 for optional or experimental hooks

### Organization
- Group related hooks in the same file
- Use comments to document hook purpose
- Keep workspace hooks minimal (use extensions for complex hooks)

### Testing
- Test hooks with `rad hooks test <name>` before enabling
- Validate configuration with `rad hooks validate`
- Use descriptive `config` sections for hook-specific options

## Troubleshooting

### Hook Not Executing

1. Check if hook is enabled: `rad hooks info <name>`
2. Verify hook type matches execution point
3. Check priority isn't being overridden
4. Ensure script path is correct (if using `script`)

### Validation Errors

Common validation errors and fixes:

- **"Hook name cannot be empty"**: Provide a `name` field
- **"Invalid hook type"**: Check `type` against valid values list
- **"Either script or config must be provided"**: Add `script` or `config` field

### Configuration Not Loading

1. Check file location (`.radium/hooks.toml` for workspace)
2. Verify TOML syntax is correct
3. Use `rad hooks validate` to check for errors
4. Check file permissions

## Related Documentation

- [Getting Started Guide](getting-started.md) - Basic usage examples
- [Creating Hooks](creating-hooks.md) - How to implement hooks
- [Hook Types](hook-types.md) - Detailed hook type reference
- [Examples](examples.md) - Configuration examples


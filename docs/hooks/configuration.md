# Hook Configuration Guide

Complete guide to configuring hooks via TOML configuration files.

## Configuration File Location

Hooks are configured in `.radium/hooks.toml`:

```toml
[[hooks]]
name = "my-hook"
type = "before_model_call"
priority = 100
enabled = true

[hooks.config]
# Hook-specific configuration
log_level = "info"
```

## Configuration Schema

### Hook Entry

Each hook entry has the following fields:

- **`name`** (required): Unique identifier for the hook
- **`type`** (required): Hook type (see below)
- **`priority`** (optional, default: 100): Execution priority (0-1000)
- **`enabled`** (optional, default: true): Whether the hook is enabled
- **`config`** (optional): Hook-specific configuration

### Hook Types

Available hook types:

- `before_model_call` - Before model API call
- `after_model_call` - After model API call
- `before_tool_execution` - Before tool execution
- `after_tool_execution` - After tool execution
- `tool_selection` - Tool selection
- `error_interception` - Error handling
- `telemetry_collection` - Telemetry collection
- `workflow_step` - Workflow step completion

## Examples

### Basic Configuration

```toml
[[hooks]]
name = "request-logger"
type = "before_model_call"
priority = 100
enabled = true
```

### Multiple Hooks

```toml
[[hooks]]
name = "request-logger"
type = "before_model_call"
priority = 100

[[hooks]]
name = "response-logger"
type = "after_model_call"
priority = 100

[[hooks]]
name = "error-handler"
type = "error_interception"
priority = 200
```

### Hook-Specific Configuration

```toml
[[hooks]]
name = "telemetry-collector"
type = "telemetry_collection"
priority = 50

[hooks.config]
endpoint = "https://api.example.com/telemetry"
api_key = "${TELEMETRY_API_KEY}"
batch_size = 100
```

### Conditional Hooks

```toml
[[hooks]]
name = "dev-logger"
type = "before_model_call"
priority = 100
enabled = false  # Disabled by default

# Enable via environment variable or runtime configuration
```

## Loading Configuration

Configuration is automatically loaded from `.radium/hooks.toml` when the workspace is initialized:

```rust
use radium_core::hooks::config::HookConfigFile;
use radium_core::hooks::registry::HookRegistry;

let config = HookConfigFile::load_from_file(".radium/hooks.toml")?;
let registry = Arc::new(HookRegistry::new());

// Register hooks from configuration
for hook_config in config.hooks {
    // Register hook based on configuration
    // ...
}
```

## Validation

Configuration is validated on load:

- Hook names must be unique
- Hook types must be valid
- Priorities must be 0-1000
- Required fields must be present

## Environment Variables

You can use environment variables in configuration:

```toml
[hooks.config]
api_key = "${API_KEY}"
endpoint = "${TELEMETRY_ENDPOINT:-https://default.endpoint.com}"
```

## Dynamic Configuration

Hooks can be enabled/disabled at runtime:

```rust
registry.enable_hook("my-hook").await?;
registry.disable_hook("my-hook").await?;
```

## Best Practices

1. **Use descriptive names**: Make hook names clear and descriptive
2. **Set appropriate priorities**: Higher priority for critical hooks
3. **Group related hooks**: Use consistent naming patterns
4. **Document configuration**: Add comments for complex configurations
5. **Version control**: Keep configuration files in version control


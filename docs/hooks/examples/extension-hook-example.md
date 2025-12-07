# Extension Hook Example

This guide demonstrates how to create and use hooks in Radium extensions.

## Overview

Hooks in extensions allow you to customize agent behavior without modifying core code. Extensions can provide hooks that are automatically discovered and loaded when the extension is installed.

## Extension Structure

An extension with hooks should have the following structure:

```
my-extension/
├── radium-extension.json
├── hooks/
│   └── my-hooks.toml
└── README.md
```

## Step 1: Create Hook Configuration

Create a `hooks` directory in your extension and add a TOML configuration file:

```toml
# hooks/my-hooks.toml

[[hooks]]
name = "my-model-logger"
type = "before_model"
priority = 100
enabled = true

[hooks.config]
log_level = "info"
```

## Step 2: Update Extension Manifest

Add hooks to your extension manifest:

```json
{
  "name": "my-extension",
  "version": "1.0.0",
  "components": {
    "hooks": ["hooks/my-hooks.toml"]
  }
}
```

## Step 3: Hook Types

Available hook types:

- `before_model` - Before model API call
- `after_model` - After model API call
- `before_tool` - Before tool execution
- `after_tool` - After tool execution
- `tool_selection` - Tool selection
- `error_interception` - Error interception
- `error_transformation` - Error transformation
- `error_recovery` - Error recovery
- `error_logging` - Error logging
- `telemetry_collection` - Telemetry collection

## Step 4: Using Hooks from CLI

Once your extension is installed, hooks are automatically discovered:

```bash
# List all hooks
rad hooks list

# List hooks by type
rad hooks list --type before_model

# Get hook information
rad hooks info my-model-logger

# Enable/disable hooks
rad hooks enable my-model-logger
rad hooks disable my-model-logger
```

## Example: Complete Extension with Hook

See `examples/extensions/example-extension/` for a complete example extension that includes:

- Hook configuration file
- Extension manifest with hooks declared
- README with usage instructions

## Hook Discovery

Hooks are automatically discovered from:

1. **Extension hooks**: All hooks in installed extensions
2. **Workspace hooks**: Hooks in `.radium/hooks.toml`

Hooks are loaded in this order:
1. Extension hooks (lowest priority)
2. Workspace hooks (highest priority, can override extension hooks)

## Best Practices

1. **Use descriptive names**: Make hook names clear and unique
2. **Set appropriate priorities**: Higher priority hooks execute first
3. **Document your hooks**: Include documentation in your extension README
4. **Test hooks**: Verify hooks work correctly before publishing
5. **Version hooks**: Update hook versions when making breaking changes

## Advanced: Custom Hook Implementations

For advanced use cases, you can create custom hook implementations in Rust. However, note that hooks must be compiled into the Radium binary. Extension hooks are primarily configuration-based.

For custom Rust hook implementations, see the main hooks documentation:
- `docs/hooks/getting-started.md`
- `docs/hooks/api-reference.md`


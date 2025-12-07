# Validation Hook Example

This is an example hook implementation that demonstrates how to validate tool arguments before execution in Radium.

## Overview

The validation hook performs security checks and input validation on tool arguments before they are executed. It demonstrates:

- Blocking dangerous tools
- Path traversal detection
- Command validation
- Argument schema validation

## Features

- Blocks dangerous tools (delete_file, rm, format_disk, etc.)
- Detects and blocks path traversal attempts
- Validates file operation arguments
- Blocks dangerous shell commands
- High priority (200) to ensure it runs before other hooks

## Usage

### Building

```bash
cd examples/hooks/validation-hook
cargo build --release
```

### Integration

To use this hook in your Radium workspace:

1. Register the hook programmatically
2. Configure it in `.radium/hooks.toml`
3. The hook will automatically validate tool executions

### Example Configuration

```toml
[[hooks]]
name = "validation-hook"
type = "before_tool"
priority = 200
enabled = true
```

## Implementation Details

The hook implements the `ToolHook` trait and validates tool arguments in the `before_tool_execution` method. It uses a high priority (200) to ensure it runs before other hooks that might modify arguments.

## Security Considerations

This is an example implementation. For production use, consider:

- More comprehensive dangerous tool/command lists
- Configurable allow/deny lists
- Integration with policy engine
- More sophisticated path validation
- Command parsing and validation

## Customization

You can customize the hook by:

- Adjusting the dangerous tools list
- Modifying path validation rules
- Adding custom validation logic
- Integrating with external security systems


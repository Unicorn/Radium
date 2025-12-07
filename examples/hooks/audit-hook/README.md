# Audit Hook Example

This is an example hook implementation that demonstrates audit logging for compliance in Radium.

## Overview

The audit hook logs all model calls and tool executions to a structured audit log file. It demonstrates:

- Comprehensive audit logging
- Structured log format (JSON)
- Model call tracking
- Tool execution tracking
- Timestamp tracking

## Features

- Logs all model calls (before and after)
- Logs all tool executions (before and after)
- Structured JSON log format
- Timestamp tracking
- Input/output previews (first 100 characters)
- Thread-safe file writing

## Usage

### Building

```bash
cd examples/hooks/audit-hook
cargo build --release
```

### Integration

To use this hook in your Radium workspace:

1. Register the hook programmatically with a log file path
2. Configure it in `.radium/hooks.toml`
3. The hook will automatically log all model calls and tool executions

### Example Configuration

```toml
[[hooks]]
name = "audit-hook"
type = "before_model"
priority = 100
enabled = true

[hooks.config]
log_file = ".radium/audit.log"
```

## Implementation Details

The hook implements both `ModelHook` and `ToolHook` traits to capture all execution points. It writes structured JSON entries to a log file for easy parsing and analysis.

## Log Format

Each audit entry is a JSON object:

```json
{
  "timestamp": "2024-01-01T12:00:00Z",
  "event": "before_model_call",
  "hook": "audit-hook",
  "model_id": "gpt-4",
  "input_length": 100,
  "input_preview": "..."
}
```

## Compliance Considerations

This is an example implementation. For production compliance use, consider:

- Log rotation
- Secure log storage
- Encryption at rest
- Access controls
- Retention policies
- Integration with compliance systems

## Customization

You can customize the hook by:

- Adjusting log format
- Adding additional metadata
- Filtering sensitive data
- Integrating with external audit systems
- Adding log rotation


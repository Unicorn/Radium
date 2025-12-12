# Custom Commands

Custom commands allow you to define reusable operations using TOML configuration files. Commands support template substitution, shell execution, file injection, and integration with the sandbox and hook systems.

## Overview

Custom commands provide:
- **TOML-based definitions**: Easy to create and maintain
- **Template substitution**: Dynamic content generation
- **Shell command execution**: Run shell commands safely
- **File content injection**: Include file contents in output
- **Sandbox integration**: Safe command execution
- **Hook integration**: Approval/denial workflows

## Command Definition

Commands are defined in TOML files located in:
- Project commands: `.radium/commands/*.toml`
- User commands: `~/.radium/commands/*.toml`
- Extension commands: Extension-specific directories

### Basic Command Structure

```toml
[command]
name = "command-name"
description = "Command description"
template = "Command template with {{args}} and !{shell} and @{file}"
```

### Example: Simple Command

```toml
[command]
name = "hello"
description = "Say hello"
template = "Hello, {{arg1}}!"
```

## Template Substitution

Commands support three types of template substitution:

### Argument Substitution

Arguments are substituted using `{{args}}` or `{{arg1}}`, `{{arg2}}`, etc.

```toml
[command]
name = "greet"
description = "Greet someone"
template = "Hello {{arg1}}, you said: {{args}}"
args = ["name", "message"]
```

Usage:
```bash
rad custom greet "Alice" "Welcome!"
# Output: Hello Alice, you said: Alice Welcome!
```

### Shell Command Execution

Shell commands are executed using `!{command}` syntax:

```toml
[command]
name = "git-status"
description = "Show git status"
template = "Git Status:\n!{git status}"
```

The shell command output replaces the `!{command}` placeholder.

### File Content Injection

File contents are injected using `@{file}` syntax:

```toml
[command]
name = "show-readme"
description = "Display README"
template = "# README\n\n@{README.md}"
```

Files are resolved relative to the command's base directory.

### Combined Template

You can combine all substitution types:

```toml
[command]
name = "build-report"
description = "Generate build report"
template = "Building {{arg1}}...\n!{cargo build --release}\n\nNotes:\n@{notes.md}"
```

## Command Discovery

Commands are discovered from multiple locations with precedence:

1. **Project commands** (`.radium/commands/`) - Highest precedence
2. **User commands** (`~/.radium/commands/`) - Medium precedence
3. **Extension commands** - Lowest precedence

### Namespaced Commands

Commands can be organized in namespaces using subdirectories:

```
.radium/commands/
  git/
    status.toml
    commit.toml
  build/
    test.toml
    release.toml
```

Namespaced commands are referenced as `namespace:command-name`:
```bash
rad custom git:status
rad custom build:test
```

## Command Execution

### Via CLI

```bash
# List all commands
rad custom list

# Execute a command
rad custom command-name arg1 arg2

# Execute namespaced command
rad custom namespace:command-name arg1
```

### Programmatically

```rust
use radium_core::commands::custom::CustomCommand;

// Load command
let command = CustomCommand::load("command-name")?;

// Execute with arguments
let output = command.execute(&["arg1", "arg2"], &workspace_root)?;
```

## Sandbox Integration

Commands can execute within a sandbox for security:

```rust
use radium_core::sandbox::{Sandbox, SandboxConfig};

let mut sandbox = Sandbox::new(&SandboxConfig::default())?;
let output = command.execute_with_sandbox(
    &args,
    &workspace_root,
    Some(&mut sandbox)
)?;
```

### Sandbox Configuration

Configure sandbox behavior in `.radium/config.toml`:

```toml
[sandbox]
enabled = true
network_enabled = false
allow_write = true
```

## Hook Integration

Commands integrate with the hook system for approval workflows:

```rust
use radium_core::hooks::registry::HookRegistry;

let registry = Arc::new(HookRegistry::new());
let output = command.execute_with_hooks(
    &args,
    &workspace_root,
    Some(registry)
).await?;
```

### Hook Approval Flow

1. **Tool Selection Hook**: Approve or deny command execution
2. **Before Tool Hook**: Modify arguments before execution
3. **After Tool Hook**: Modify output after execution

If a hook denies execution, the command fails with a `ToolDenied` error.

## Examples

### Example: Git Status Command

```toml
[command]
name = "git-status"
description = "Show git status with context"
template = "# Git Status\n\n!{git status}\n\n# Recent Commits\n\n!{git log --oneline -5}"
```

Usage:
```bash
rad custom git-status
```

### Example: Test Command with Arguments

```toml
[command]
name = "test"
description = "Run tests with filter"
template = "Running tests for {{arg1}}...\n\n!{cargo test --lib {{arg1}}}"
args = ["filter"]
```

Usage:
```bash
rad custom test "my_module"
```

### Example: Documentation Command

```toml
[command]
name = "docs"
description = "Show project documentation"
template = "# Project Documentation\n\n@{README.md}\n\n## API Reference\n\n@{docs/API.md}"
```

Usage:
```bash
rad custom docs
```

## Best Practices

### Command Organization

- Use namespaces for related commands
- Keep command templates simple and readable
- Document commands with clear descriptions

### Security

- Use sandbox for untrusted commands
- Configure hooks to approve/deny dangerous operations
- Validate arguments before execution

### Performance

- Avoid long-running commands in templates
- Cache expensive operations
- Use file injection for large static content

## Troubleshooting

### Command Not Found

- Verify command file exists in `.radium/commands/`
- Check command name matches file name (without .toml)
- Verify workspace is initialized

### Template Substitution Fails

- Check argument count matches template placeholders
- Verify file paths are correct (relative to workspace root)
- Ensure shell commands are available in PATH

### Sandbox Issues

- Verify sandbox is enabled in config
- Check sandbox permissions allow required operations
- Review sandbox logs for blocked operations

### Hook Denial

- Check hook configuration
- Review hook logs for denial reasons
- Verify command matches hook approval criteria


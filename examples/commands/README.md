# Custom Commands Examples

This directory contains example custom commands demonstrating various features of the Radium custom commands system.

## Examples

### Simple Commands

- **git-status.toml**: Shows git status with recent commits using shell command injection (`!{}`)
- **test.toml**: Runs tests with a filter argument using argument substitution (`{{arg1}}`)
- **docs.toml**: Displays project documentation using file content injection (`@{}`)

### Namespaced Commands

Commands can be organized in namespaces using subdirectories:

- **git/commit.toml**: Namespaced as `git:commit`, creates a git commit with a message
- **build/test.toml**: Namespaced as `build:test`, runs build tests

## Usage

To use these examples, copy them to your project's `.radium/commands/` directory:

```bash
# Copy all examples
cp -r examples/commands/* .radium/commands/

# Or copy individual commands
cp examples/commands/git-status.toml .radium/commands/
```

Then use them with the `rad custom` command:

```bash
# Simple command
rad custom git-status

# Command with arguments
rad custom test "my_module"

# Namespaced command
rad custom git:commit "Fix bug in authentication"

# Another namespaced command
rad custom build:test
```

## Template Syntax

### Shell Command Injection

Use `!{command}` to execute shell commands:

```toml
template = "Current time: !{date}"
```

### File Content Injection

Use `@{file}` to inject file contents:

```toml
template = "README:\n\n@{README.md}"
```

### Argument Substitution

Use `{{args}}` for all arguments or `{{arg1}}`, `{{arg2}}`, etc. for specific arguments:

```toml
template = "Hello {{arg1}}, you said: {{args}}"
args = ["name", "message"]
```

### Combined Example

```toml
[command]
name = "build-report"
description = "Generate build report"
template = "Building {{arg1}}...\n!{cargo build --release}\n\nNotes:\n@{notes.md}"
args = ["target"]
```

## Best Practices

1. **Keep commands focused**: Each command should do one thing well
2. **Use namespaces**: Organize related commands in subdirectories
3. **Document clearly**: Provide helpful descriptions
4. **Handle errors**: Commands should fail gracefully
5. **Test locally**: Verify commands work before committing

## See Also

- [Custom Commands User Guide](../../docs/user-guide/custom-commands.md) - Complete documentation
- [Command System Implementation](../../crates/radium-core/src/commands/custom.rs) - Technical details


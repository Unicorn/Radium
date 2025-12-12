---
id: "advanced"
title: "Advanced Features"
sidebar_label: "Advanced Features"
---

# Advanced Features

Commands for advanced execution and workflow management.

## `rad autonomous`

Autonomous execution from high-level goals.

### Usage

```bash
rad autonomous <goal>
```

### Arguments

- `goal` - High-level goal description

### Examples

```bash
# Execute autonomous goal
rad autonomous "Build a REST API with authentication"

# Complex goal
rad autonomous "Create a full-stack application with React frontend and Node.js backend"
```

## `rad hooks`

Manage execution hooks.

### Subcommands

#### `list`

List all registered hooks.

```bash
rad hooks list [--type <type>] [--json] [--verbose]
```

Options:
- `--type <type>` - Filter by hook type
- `--json` - Output as JSON
- `--verbose` - Show detailed information

#### `info <name>`

Show detailed information about a hook.

```bash
rad hooks info `<name>` [--json]
```

#### `enable <name>`

Enable a hook.

```bash
rad hooks enable `<name>`
```

#### `disable <name>`

Disable a hook.

```bash
rad hooks disable `<name>`
```

### Examples

```bash
# List all hooks
rad hooks list

# List hooks by type
rad hooks list --type before_model

# Get hook info
rad hooks info my-hook

# Enable hook
rad hooks enable my-hook

# Disable hook
rad hooks disable my-hook
```

## `rad context`

Manage context files (GEMINI.md).

### Subcommands

#### `list`

List context files.

```bash
rad context list
```

#### `show <file>`

Show context file content.

```bash
rad context show `<file>`
```

#### `validate`

Validate context files.

```bash
rad context validate
```

### Examples

```bash
# List context files
rad context list

# Show context file
rad context show GEMINI.md

# Validate context files
rad context validate
```

## `rad custom`

Manage custom commands.

### Subcommands

#### `list`

List custom commands.

```bash
rad custom list
```

#### `run <command>`

Execute a custom command.

```bash
rad custom run <command>
```

#### `create <name>`

Create a custom command.

```bash
rad custom create `<name>`
```

#### `validate <command>`

Validate a custom command.

```bash
rad custom validate <command>
```

### Examples

```bash
# List custom commands
rad custom list

# Run custom command
rad custom run my-command

# Create custom command
rad custom create my-command

# Validate command
rad custom validate my-command
```

## `rad sandbox`

Manage sandbox environments.

### Subcommands

#### `list`

List available sandbox types.

```bash
rad sandbox list [--json]
```

#### `test [sandbox-type]`

Test a sandbox type.

```bash
rad sandbox test [sandbox-type] [--json]
```

#### `config`

Show current sandbox configuration.

```bash
rad sandbox config [--json]
```

#### `doctor`

Check sandbox prerequisites.

```bash
rad sandbox doctor [--json]
```

### Examples

```bash
# List sandboxes
rad sandbox list

# Test sandbox
rad sandbox test docker

# Show config
rad sandbox config

# Check prerequisites
rad sandbox doctor
```


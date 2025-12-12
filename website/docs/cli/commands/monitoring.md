---
id: "monitoring"
title: "Monitoring and Analytics Commands"
sidebar_label: "Monitoring and Analytics Commands"
---

# Monitoring and Analytics Commands

Commands for monitoring agent execution, tracking costs, and viewing analytics.

## `rad monitor`

Monitor agent execution and telemetry.

### Subcommands

#### `list`

List all agents.

```bash
rad monitor list [--status <status>] [--json]
```

Options:
- `--status <status>` - Filter by status (running, completed, failed, etc.)
- `--json` - Output as JSON

#### `status [agent-id]`

Show agent status.

```bash
rad monitor status [agent-id] [--json]
```

#### `telemetry [agent-id]`

Show telemetry and cost information.

```bash
rad monitor telemetry [agent-id] [--json]
```

### Examples

```bash
# List all agents
rad monitor list

# List running agents
rad monitor list --status running

# Get agent status
rad monitor status agent-123

# Get telemetry for agent
rad monitor telemetry agent-123

# Get all telemetry
rad monitor telemetry
```

## `rad stats`

View session statistics and analytics.

### Subcommands

#### `session [session-id]`

Show session statistics.

```bash
rad stats session [session-id] [--json]
```

#### `costs`

Show cost tracking information.

```bash
rad stats costs [--json]
```

#### `usage`

Show token usage analytics.

```bash
rad stats usage [--json]
```

### Examples

```bash
# Show session stats
rad stats session

# Show specific session
rad stats session session-123

# Show costs
rad stats costs

# Show usage
rad stats usage
```

## `rad engines`

Manage engine providers.

### Subcommands

#### `list`

List available engines.

```bash
rad engines list [--json]
```

#### `show <engine-id>`

Show engine details.

```bash
rad engines show <engine-id> [--json]
```

#### `status`

Show engine status.

```bash
rad engines status [--json]
```

#### `set-default <engine-id>`

Set default engine.

```bash
rad engines set-default <engine-id>
```

### Examples

```bash
# List engines
rad engines list

# Show engine details
rad engines show claude

# Check engine status
rad engines status

# Set default engine
rad engines set-default claude
```

## `rad checkpoint`

Manage checkpoints for rollback.

### Subcommands

#### `list`

List all checkpoints.

```bash
rad checkpoint list [--json]
```

#### `restore <checkpoint-id>`

Restore a checkpoint.

```bash
rad checkpoint restore <checkpoint-id>
```

### Examples

```bash
# List checkpoints
rad checkpoint list

# Restore checkpoint
rad checkpoint restore checkpoint-123
```


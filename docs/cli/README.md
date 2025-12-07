# Radium CLI Documentation

The Radium CLI (`rad`) is a comprehensive command-line interface for managing workspaces, generating and executing plans, managing agents, and orchestrating autonomous workflows.

## Quick Start

### Installation

```bash
# Build from source
cargo build --release -p radium-cli

# Or install via package manager (when available)
```

### Basic Usage

```bash
# Initialize a workspace
rad init

# Generate a plan from a specification
rad plan spec.md

# Execute a plan
rad craft REQ-001

# Complete workflow from source to execution
rad complete spec.md
```

## Command Categories

- **[Workspace Management](commands/workspace.md)** - Initialize, status, clean, doctor
- **[Plan Execution](commands/plan-execution.md)** - Plan generation and execution
- **[Agent Management](commands/agents.md)** - List, search, validate agents
- **[Execution Commands](commands/execution.md)** - Step, run, chat
- **[MCP Integration](commands/mcp.md)** - Model Context Protocol management
- **[Extensions](commands/extensions.md)** - Extension package management
- **[Monitoring](commands/monitoring.md)** - Agent monitoring and analytics
- **[Advanced Features](commands/advanced.md)** - Autonomous execution, checkpoints

## Common Workflows

See [Workflows Guide](workflows.md) for common patterns and use cases.

## Troubleshooting

See [Troubleshooting Guide](troubleshooting.md) for solutions to common issues.

## Getting Help

```bash
# General help
rad --help

# Command-specific help
rad <command> --help

# Subcommand help
rad <command> <subcommand> --help
```


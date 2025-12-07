# Complete Toolkit Extension

A comprehensive extension demonstrating all component types and advanced patterns available in the Radium extension system.

## Installation

```bash
rad extension install ./examples/extensions/complete-toolkit
```

**Note:** This extension depends on `hello-world`, which will be automatically installed if you use `--install-deps`.

## Components

This extension demonstrates all component types:

### Prompts

- **Agents**: `prompts/agents/developer-agent.md` - General development agent
- **Frameworks**: `prompts/frameworks/react-agent.md` - React-specific agent

### MCP Servers

- **database-tools** - Database management MCP server configuration

### Commands

- **build** - Project build command
- **deploy-production** - Production deployment command

### Hooks

- **metrics-hook** - Native hook for metrics collection

### Workflows

- **full-stack-workflow** - Complete development workflow template

## Structure

```
complete-toolkit/
├── radium-extension.json
├── prompts/
│   ├── agents/
│   │   └── developer-agent.md
│   └── frameworks/
│       └── react-agent.md
├── mcp/
│   └── database-tools.json
├── commands/
│   ├── build.toml
│   └── deploy/
│       └── production.toml
├── hooks/
│   └── metrics-hook.toml
├── templates/
│   └── full-stack-workflow.json
└── README.md
```

## Usage

After installation, all components will be available:

```bash
# List agents
rad agents list

# List MCP servers
rad mcp list

# List commands
rad commands list

# List hooks
rad hooks list

# List workflows
rad workflows list
```

## Dependencies

This extension depends on:
- **hello-world** - Minimal extension (automatically installed with `--install-deps`)

## Advanced Patterns

This extension demonstrates:
- **Categorized prompts** - Organizing prompts in subdirectories
- **Nested commands** - Commands in subdirectories
- **Dependency management** - Declaring and resolving dependencies
- **Multiple component types** - Using all available component types
- **Workflow templates** - Creating reusable workflow templates

## See Also

- [Extension System Guide](../../../docs/extensions/README.md)
- [Creating Extensions](../../../docs/extensions/creating-extensions.md)
- [Architecture Documentation](../../../docs/extensions/architecture.md)


# Hello World Extension

A minimal Radium extension demonstrating the basic structure and components.

## Installation

```bash
rad extension install ./examples/extensions/hello-world
```

## Structure

This extension demonstrates the minimal structure required for a Radium extension:

```
hello-world/
├── radium-extension.json    # Extension manifest (required)
├── prompts/                  # Agent prompt templates (optional)
│   └── greeter-agent.md
└── README.md                 # Documentation (recommended)
```

## Components

### Prompts

- **greeter-agent** - A simple greeting agent

## Usage

After installation, the greeter agent will be discoverable:

```bash
# List available agents
rad agents list

# The greeter-agent will appear in the list
```

## Purpose

This extension serves as a minimal example for:
- Understanding extension structure
- Learning how to create extensions
- Testing extension installation
- Demonstrating basic prompt components

## Next Steps

After understanding this minimal example, explore:
- [Code Review Agents](../code-review-agents/) - Multi-agent extension
- [GitHub Integration](../github-integration/) - MCP server extension
- [Custom Workflows](../custom-workflows/) - Workflow templates
- [Complete Toolkit](../complete-toolkit/) - Full-featured example

## See Also

- [Extension System Guide](../../../docs/extensions/README.md)
- [Creating Extensions](../../../docs/extensions/creating-extensions.md)


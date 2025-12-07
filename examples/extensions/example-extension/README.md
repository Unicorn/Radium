# Example Extension

This is an example Radium extension that demonstrates how to package prompts, MCP servers, and custom commands.

## Structure

```
example-extension/
├── radium-extension.json    # Extension manifest
├── prompts/                  # Agent prompt templates
│   └── example-agent.md
├── mcp/                      # MCP server configurations
│   └── example-server.json
├── commands/                 # Custom commands
│   └── example-command.toml
└── README.md                 # This file
```

## Installation

Install this example extension:

```bash
rad extension install ./examples/extensions/example-extension
```

## Components

### Prompts

The extension includes example prompt templates in the `prompts/` directory.

### MCP Servers

Example MCP server configuration in `mcp/example-server.json`.

### Commands

Example custom command in `commands/example-command.toml`.

## Usage

After installation, the extension components will be available to Radium:

- Prompts will be discoverable by the agent system
- MCP servers can be configured and used
- Commands can be executed via the CLI

## See Also

- [Extension System Guide](../../../docs/guides/extension-system.md)
- [REQ-018: Extension System](../../../docs/plan/03-later/REQ-018-extension-system.md)


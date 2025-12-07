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
├── hooks/                    # Hook configurations
│   └── example-hook.toml
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

### Hooks

Example hook configuration in `hooks/example-hook.toml`. Hooks allow you to customize agent behavior at various points in the execution flow.

## Usage

After installation, the extension components will be available to Radium:

- Prompts will be discoverable by the agent system
- MCP servers can be configured and used
- Commands can be executed via the CLI
- Hooks will be automatically discovered and can be managed via `rad hooks list`

## See Also

- [Extension System Guide](../../../docs/guides/extension-system.md)
- [REQ-018: Extension System](../../../docs/plan/03-later/REQ-018-extension-system.md)


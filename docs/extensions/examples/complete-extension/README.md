# Complete Extension Example

This is a complete example extension demonstrating all component types available in Radium extensions.

## Components

This extension includes:

1. **Prompts** (`prompts/example-agent.md`)
   - A sample agent prompt template

2. **MCP Servers** (`mcp/example-server.json`)
   - A sample MCP server configuration

3. **Commands** (`commands/example-command.toml`)
   - A sample custom command definition

4. **Hooks** (`hooks/example-hook.toml`)
   - A sample hook configuration

## Installation

Install this example extension:

```bash
rad extension install ./complete-extension
```

## Usage

After installation, all components are automatically available:

- The prompt can be used in agent configurations
- The MCP server will be loaded
- The command can be executed
- The hook will be registered

## Structure

```
complete-extension/
├── radium-extension.json    # Manifest
├── prompts/
│   └── example-agent.md    # Agent prompt
├── mcp/
│   └── example-server.json # MCP server config
├── commands/
│   └── example-command.toml # Custom command
├── hooks/
│   └── example-hook.toml   # Hook config
└── README.md               # This file
```

## Customization

To customize this example:

1. Edit the manifest (`radium-extension.json`) with your details
2. Replace example components with your own
3. Update version numbers
4. Test installation locally
5. Publish to marketplace (optional)

## See Also

- [Quickstart Guide](../../quickstart.md)
- [Creating Extensions](../../creating-extensions.md)
- [Publishing Guide](../../publishing-guide.md)


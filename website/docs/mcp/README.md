---
id: "README"
title: "MCP Integration"
sidebar_label: "MCP Integration"
---

# MCP Integration

Radium supports the Model Context Protocol (MCP) for connecting to external MCP servers and using their tools and prompts. This enables Radium to extend its capabilities through external services.

## Quick Start

1. **Configure an MCP server** in `.radium/mcp-servers.toml`:

```toml
[[servers]]
name = "my-server"
transport = "stdio"
command = "mcp-server"
args = ["--config", "config.json"]
```

2. **List available tools**:

```bash
rad mcp tools
```

3. **Use MCP tools in agents** - MCP tools are automatically available to agents during execution.

## Features

- **Tool Discovery**: Automatically discover tools from MCP servers
- **Slash Commands**: MCP prompts are available as slash commands in chat
- **Rich Content**: Support for text, images, and audio content
- **OAuth Authentication**: Secure authentication for remote servers
- **Multiple Transports**: Support for stdio, SSE, and HTTP transports

## Documentation

- [User Guide](user-guide.md) - **Start here!** Complete setup and usage guide
- [Configuration Guide](configuration.md) - How to configure MCP servers
- [Authentication](authentication.md) - OAuth setup and token management
- [Using MCP Tools](tools.md) - How agents use MCP tools
- [Slash Commands](prompts.md) - Using MCP prompts as slash commands
- [Troubleshooting](troubleshooting.md) - Common issues and solutions

## Example Guides

- [Stdio Server Example](examples/stdio-server.md) - Local server setup
- [Remote Server Example](examples/remote-server.md) - HTTP/SSE server setup
- [OAuth Server Example](examples/oauth-server.md) - OAuth authentication setup

## Examples

See the [examples directory](../../examples/mcp/) for working configuration examples.

## References

- [MCP Protocol Specification](https://modelcontextprotocol.io)
- [MCP Server Examples](https://github.com/modelcontextprotocol/servers)


---
id: "README"
title: "MCP Tools Extension"
sidebar_label: "MCP Tools Extension"
---

# MCP Tools Extension

This extension demonstrates how to package MCP (Model Context Protocol) server configurations.

## Structure

```
mcp-tools/
├── radium-extension.json
├── mcp/
│   └── example-server.json
└── README.md
```

## Installation

```bash
rad extension install ./docs/extensions/examples/mcp-tools
```

## Components

### MCP Servers

- `example-server.json` - Example MCP server configuration

**Note**: MCP server configurations in extensions are loaded and merged with the workspace MCP configuration. The actual MCP server configuration format uses TOML in the workspace, but extensions can provide JSON configurations that are converted.

## Usage

After installation, the MCP server configurations will be available when initializing the MCP system. The servers will be loaded alongside workspace-configured servers.

To use the MCP servers:

```bash
rad mcp list
```

## See Also

- [MCP Configuration Documentation](../../../mcp/configuration.md)
- [MCP Tools Documentation](../../../mcp/tools.md)


---
id: "extension-mcp-integration"
title: "Extension MCP Integration"
sidebar_label: "Extension MCP Integration"
---

# Extension MCP Integration

This guide explains how extensions can provide MCP server configurations and how they integrate with workspace MCP configurations.

## Overview

Extensions can include MCP server configurations that are automatically loaded and merged with workspace MCP configurations. This allows extensions to provide pre-configured MCP servers that users can use immediately.

## Configuration Precedence

MCP server configurations are loaded in the following order (precedence from highest to lowest):

1. **Workspace MCP Config** (`.radium/mcp-servers.toml`) - **Highest Precedence**
2. **Extension MCP Configs** (from installed extensions) - **Lower Precedence**

### Precedence Rules

- **Workspace configs take precedence**: If a server with the same name exists in both workspace and extension configs, the workspace configuration is used.
- **Extension configs are additive**: Extension servers that don't conflict with workspace servers are added to the server list.
- **No overwriting**: Extension configs cannot overwrite workspace configs, even if the extension is installed after workspace configuration.

## Extension MCP Config Format

Extension MCP configurations can be provided in two formats:

### Format 1: JSON (Extension Format)

Place JSON files in the extension's `mcp/` directory:

```
my-extension/
├── manifest.toml
└── mcp/
    └── database-server.json
```

Example JSON format:

```json
{
  "name": "database-server",
  "transport": "stdio",
  "command": "mcp-postgres",
  "args": ["postgresql://localhost/mydb"]
}
```

### Format 2: TOML (Workspace Format)

Extensions can also use TOML format (same as workspace configs):

```
my-extension/
├── manifest.toml
└── mcp/
    └── servers.toml
```

Example TOML format:

```toml
[[servers]]
name = "database-server"
transport = "stdio"
command = "mcp-postgres"
args = ["postgresql://localhost/mydb"]
```

## Configuration Fields

### Required Fields

- `name`: Unique server identifier (string)
- `transport`: Transport type - `"stdio"`, `"sse"`, or `"http"` (string)

### Transport-Specific Fields

**For stdio transport:**
- `command`: Command to execute (required)
- `args`: Optional array of command arguments

**For SSE/HTTP transport:**
- `url`: Server URL (required)

### Optional Fields

- `auth`: Authentication configuration (see [Authentication Guide](../mcp/authentication.md))

## Examples

### Example 1: Simple Stdio Server

**Extension structure:**
```
my-extension/
├── manifest.toml
└── mcp/
    └── filesystem-server.json
```

**mcp/filesystem-server.json:**
```json
{
  "name": "extension-filesystem",
  "transport": "stdio",
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
}
```

### Example 2: Remote HTTP Server

**mcp/api-server.json:**
```json
{
  "name": "extension-api",
  "transport": "http",
  "url": "https://api.example.com/mcp"
}
```

### Example 3: Multiple Servers (TOML)

**mcp/servers.toml:**
```toml
[[servers]]
name = "extension-db"
transport = "stdio"
command = "mcp-postgres"
args = ["postgresql://localhost/extdb"]

[[servers]]
name = "extension-api"
transport = "http"
url = "https://api.example.com/mcp"
```

## Precedence Examples

### Example 1: No Conflict

**Workspace config:**
```toml
[[servers]]
name = "workspace-server"
transport = "stdio"
command = "mcp-workspace"
```

**Extension config:**
```json
{
  "name": "extension-server",
  "transport": "stdio",
  "command": "mcp-extension"
}
```

**Result**: Both servers are loaded (no conflict).

### Example 2: Name Conflict

**Workspace config:**
```toml
[[servers]]
name = "database-server"
transport = "stdio"
command = "mcp-postgres-workspace"
```

**Extension config:**
```json
{
  "name": "database-server",
  "transport": "stdio",
  "command": "mcp-postgres-extension"
}
```

**Result**: Only workspace server is loaded (workspace takes precedence).

### Example 3: Multiple Extensions

**Extension A config:**
```json
{ "name": "server-a", "transport": "stdio", "command": "mcp-a" }
```

**Extension B config:**
```json
{ "name": "server-b", "transport": "stdio", "command": "mcp-b" }
```

**Result**: Both servers are loaded (no conflicts).

## User Override

Users can override extension MCP configs by adding a server with the same name to their workspace config:

**Extension provides:**
```json
{
  "name": "extension-server",
  "transport": "stdio",
  "command": "mcp-default"
}
```

**User workspace config:**
```toml
[[servers]]
name = "extension-server"
transport = "stdio"
command = "mcp-custom"
args = ["--custom", "config"]
```

**Result**: User's custom configuration is used instead of extension default.

## Best Practices for Extension Developers

### 1. Use Descriptive Names

Use namespaced server names to avoid conflicts:

```json
{
  "name": "my-extension-database",
  "transport": "stdio",
  "command": "mcp-postgres"
}
```

### 2. Provide Sensible Defaults

Configure servers with reasonable defaults that work out of the box:

```json
{
  "name": "my-extension-files",
  "transport": "stdio",
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-filesystem", "${HOME}/Documents"]
}
```

### 3. Document Requirements

Document any requirements in your extension's README:

- Required system dependencies
- Required environment variables
- Required permissions
- Network requirements

### 4. Handle Missing Dependencies

Design your MCP server configs to fail gracefully if dependencies are missing:

- Use `npx` for npm-based servers (auto-installs)
- Provide clear error messages
- Document installation steps

### 5. Avoid Authentication in Extensions

**Don't include OAuth credentials in extension configs:**
- Users should configure authentication themselves
- Credentials are sensitive and user-specific
- Use placeholder or documentation instead

**Instead, document authentication:**
```json
{
  "name": "my-extension-api",
  "transport": "http",
  "url": "https://api.example.com/mcp"
  // Note: Users must configure OAuth authentication
  // See extension README for setup instructions
}
```

## Troubleshooting Extension MCP Configs

### Server Not Loading

**Check:**
1. Extension is installed and discovered
2. MCP config file is in `mcp/` directory
3. Config file format is valid (JSON or TOML)
4. Server name doesn't conflict with workspace config
5. Required fields are present

### Server Overridden by Workspace

**Problem**: Extension server not appearing

**Solution**: This is expected behavior - workspace configs take precedence. Users can:
1. Remove conflicting workspace server
2. Rename extension server in workspace config
3. Use different server name in extension

### Invalid Config Format

**Problem**: Extension MCP config not parsed

**Solutions:**
1. Verify JSON/TOML syntax is valid
2. Check required fields are present
3. Verify transport-specific fields (command for stdio, url for HTTP/SSE)
4. Check file encoding (should be UTF-8)

## Integration Details

### Loading Process

1. **Workspace configs loaded first**: `.radium/mcp-servers.toml` is loaded
2. **Extension configs discovered**: All extensions are scanned for `mcp/` directory
3. **Extension configs parsed**: Each config file is parsed (JSON or TOML)
4. **Conflict resolution**: Extension servers with conflicting names are skipped
5. **Servers connected**: All non-conflicting servers are connected

### File Discovery

Extension MCP configs are discovered from:
- Project-level extensions: `.radium/extensions/{extension-name}/mcp/`
- User-level extensions: `~/.radium/extensions/{extension-name}/mcp/`

All `.json` and `.toml` files in the `mcp/` directory are loaded.

### Error Handling

- **Invalid configs are skipped**: If an extension config is invalid, it's skipped with a warning
- **Partial failures don't block**: If some extension servers fail to connect, others still work
- **Errors are logged**: Check logs for detailed error messages

## Related Documentation

- [MCP User Guide](../mcp/user-guide.md) - User-facing MCP documentation
- [MCP Configuration](../mcp/configuration.md) - Configuration reference
- [Creating Extensions](creating-extensions.md) - How to create extensions
- [Extension Architecture](../extensions/architecture.md) - Extension system overview

## Examples

See the [examples directory](../../examples/extensions/) for complete extension examples with MCP integration.


---
id: "user-guide"
title: "MCP Integration User Guide"
sidebar_label: "MCP Integration User Guide"
---

# MCP Integration User Guide

This guide provides step-by-step instructions for setting up and using MCP (Model Context Protocol) servers with Radium.

## Table of Contents

1. [Introduction](#introduction)
2. [Quick Start](#quick-start)
3. [Setting Up Your First MCP Server](#setting-up-your-first-mcp-server)
4. [Configuration Examples](#configuration-examples)
5. [Using MCP Tools](#using-mcp-tools)
6. [Using Slash Commands](#using-slash-commands)
7. [Authentication Setup](#authentication-setup)
8. [Troubleshooting](#troubleshooting)
9. [Best Practices](#best-practices)

## Introduction

MCP (Model Context Protocol) allows Radium to connect to external servers that provide tools and prompts. This extends Radium's capabilities by enabling:

- **External Tool Integration**: Use tools from external services (databases, APIs, file systems)
- **Slash Commands**: Access MCP server prompts through slash commands in chat
- **Rich Content**: Handle text, images, and audio content from MCP servers

### Supported Transports

Radium supports three transport types for connecting to MCP servers:

1. **Stdio**: For local MCP servers running as processes
2. **SSE (Server-Sent Events)**: For remote servers using HTTP streaming
3. **HTTP**: For remote servers using standard HTTP requests

## Quick Start

### 1. Create Configuration File

Create `.radium/mcp-servers.toml` in your workspace root:

```toml
[[servers]]
name = "my-server"
transport = "stdio"
command = "mcp-server"
args = ["--config", "config.json"]
```

### 2. Verify Configuration

```bash
# List configured servers
rad mcp list

# Test connection
rad mcp test
```

### 3. Use MCP Tools

MCP tools are automatically available to agents during execution. You can also list them:

```bash
rad mcp tools
```

## Setting Up Your First MCP Server

### Step 1: Choose a Transport Type

**For Local Servers (Stdio):**
- Server runs as a local process
- Best for development and testing
- Requires server executable to be in PATH

**For Remote Servers (SSE/HTTP):**
- Server runs on a remote host
- Best for production deployments
- Requires network access and authentication

### Step 2: Create Configuration

Create `.radium/mcp-servers.toml` in your workspace root:

```toml
[[servers]]
name = "example-server"
transport = "stdio"  # or "sse" or "http"
command = "mcp-server"
args = ["--config", "config.json"]
```

### Step 3: Test Connection

```bash
rad mcp test --server example-server
```

### Step 4: Verify Tools Are Available

```bash
rad mcp tools
```

You should see tools from your server listed.

## Configuration Examples

### Example 1: Local Database Server (Stdio)

```toml
[[servers]]
name = "postgres-mcp"
transport = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-postgres", "postgresql://localhost/mydb"]
```

### Example 2: Remote API Server (HTTP)

```toml
[[servers]]
name = "api-server"
transport = "http"
url = "https://api.example.com/mcp"
auth = { auth_type = "oauth", params = { token_url = "https://api.example.com/oauth/token", client_id = "your-client-id", client_secret = "your-client-secret" } }
```

### Example 3: Remote Server with SSE

```toml
[[servers]]
name = "streaming-server"
transport = "sse"
url = "https://stream.example.com/mcp/sse"
auth = { auth_type = "oauth", params = { token_url = "https://stream.example.com/oauth/token", client_id = "your-client-id" } }
```

### Example 4: Multiple Servers

```toml
[[servers]]
name = "database-server"
transport = "stdio"
command = "mcp-postgres"
args = ["postgresql://localhost/mydb"]

[[servers]]
name = "file-server"
transport = "stdio"
command = "mcp-filesystem"
args = ["--root", "/path/to/files"]

[[servers]]
name = "api-server"
transport = "http"
url = "https://api.example.com/mcp"
```

## Using MCP Tools

### Automatic Discovery

MCP tools are automatically discovered when Radium starts and are available to agents during execution. Tools from different servers are prefixed with the server name to avoid conflicts.

### Tool Naming

Tools are registered with their server name as a prefix:

- Server: `database-server`
- Tool: `query`
- Registered as: `database-server/query`

### Using Tools in Agents

Agents can use MCP tools just like built-in tools. The tool name includes the server prefix:

```json
{
  "tool": "database-server/query",
  "arguments": {
    "sql": "SELECT * FROM users LIMIT 10"
  }
}
```

### Listing Available Tools

```bash
# List all tools
rad mcp tools

# List tools from specific server
rad mcp tools --server database-server
```

## Using Slash Commands

MCP prompts are available as slash commands in Radium's chat interface.

### Available Commands

```bash
# List all available slash commands
rad mcp prompts
```

### Using Slash Commands

In the chat interface, type `/` followed by the prompt name:

```
/prompt-name argument1 argument2
```

### Example

If an MCP server provides a prompt called `generate-code`, you can use it as:

```
/generate-code python function to sort a list
```

## Authentication Setup

### OAuth Authentication

For servers requiring OAuth authentication:

1. **Get OAuth Credentials**:
   - Register your application with the OAuth provider
   - Obtain `client_id` and `client_secret`
   - Get the `token_url` endpoint

2. **Configure Authentication**:

```toml
[[servers]]
name = "oauth-server"
transport = "http"
url = "https://api.example.com/mcp"
auth = {
    auth_type = "oauth",
    params = {
        token_url = "https://api.example.com/oauth/token",
        client_id = "your-client-id",
        client_secret = "your-client-secret"
    }
}
```

3. **Initial Token Acquisition**:
   - First-time setup may require manual token acquisition
   - Tokens are stored securely in `~/.radium/mcp_tokens/`
   - Tokens are automatically refreshed when expired

4. **Check Token Status**:

```bash
rad mcp auth status
```

For detailed OAuth setup instructions, see [OAuth Setup Guide](oauth-setup.md).

## Troubleshooting

### Server Not Connecting

**Check Configuration:**
```bash
rad mcp list
```

**Test Connection:**
```bash
rad mcp test --server server-name
```

**Common Issues:**
- Server executable not in PATH (stdio)
- Incorrect URL (HTTP/SSE)
- Network connectivity issues
- Authentication required but not configured

### Tools Not Available

**Verify Server Connection:**
```bash
rad mcp test
```

**Check Tool Discovery:**
```bash
rad mcp tools
```

**Common Issues:**
- Server not connected
- Server doesn't provide tools
- Tool name conflicts (check prefixes)

### Authentication Errors

**Check Token Status:**
```bash
rad mcp auth status
```

**Common Issues:**
- Token expired (should auto-refresh)
- Invalid credentials
- Missing `token_url` in config
- Token storage directory permissions

For more troubleshooting help, see [Troubleshooting Guide](troubleshooting.md).

## Best Practices

### 1. Server Naming

Use descriptive, unique names for servers:

```toml
# Good
name = "postgres-production-db"
name = "github-api-server"

# Avoid
name = "server1"
name = "test"
```

### 2. Configuration Organization

Group related servers together:

```toml
# Database servers
[[servers]]
name = "postgres-main"
# ...

[[servers]]
name = "postgres-analytics"
# ...

# API servers
[[servers]]
name = "github-api"
# ...
```

### 3. Security

- Store sensitive credentials securely
- Use OAuth for remote servers
- Keep tokens in `~/.radium/mcp_tokens/` (restricted permissions)
- Don't commit tokens to version control

### 4. Error Handling

- Test connections before relying on tools
- Monitor server health
- Use descriptive error messages
- Implement retry logic for transient failures

### 5. Performance

- Limit number of servers (each adds overhead)
- Use appropriate transport (stdio for local, HTTP/SSE for remote)
- Monitor connection health
- Cache tool discovery results

## Extension MCP Integration

Extensions can provide MCP server configurations that are automatically loaded. Extension configs have lower precedence than workspace configs.

**Example**: An extension might provide a database MCP server that users can use immediately.

For details on extension MCP integration, see [Extension MCP Integration](../developer-guide/extension-mcp-integration.md).

## Next Steps

- [Configuration Reference](configuration.md) - Detailed configuration options
- [OAuth Setup Guide](oauth-setup.md) - Step-by-step OAuth configuration
- [Using MCP Tools](tools.md) - Advanced tool usage
- [Slash Commands](prompts.md) - Creating and using prompts
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
- [Architecture](architecture.md) - How MCP integration works
- [Extension MCP Integration](../developer-guide/extension-mcp-integration.md) - How extensions provide MCP servers

## Examples

See the [examples directory](../../examples/mcp/) for complete working examples:

- `stdio-server.toml` - Local server configuration
- `remote-server.toml` - Remote HTTP server
- `oauth-server.toml` - OAuth-authenticated server

## References

- [MCP Protocol Specification](https://modelcontextprotocol.io)
- [MCP Server Examples](https://github.com/modelcontextprotocol/servers)
- [Radium MCP Architecture](architecture.md)


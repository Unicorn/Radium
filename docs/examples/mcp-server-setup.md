# MCP Server Setup Examples

This document provides examples for setting up and configuring MCP servers with Radium.

## Basic Stdio Server

Example configuration for a local MCP server using stdio transport:

```toml
# .radium/mcp-servers.toml
[[servers]]
name = "local-server"
transport = "stdio"
command = "mcp-server"
args = ["--config", "server-config.json"]
```

## SSE Server with Authentication

Example configuration for an SSE server with OAuth authentication:

```toml
# .radium/mcp-servers.toml
[[servers]]
name = "remote-api"
transport = "sse"
url = "https://api.example.com/mcp/sse"
auth = { auth_type = "oauth", client_id = "your-client-id", client_secret = "your-secret" }
```

## HTTP Streaming Server

Example configuration for an HTTP streaming server:

```toml
# .radium/mcp-servers.toml
[[servers]]
name = "http-server"
transport = "http"
url = "https://mcp.example.com/api"
```

## Multiple Servers

You can configure multiple MCP servers:

```toml
# .radium/mcp-servers.toml
[[servers]]
name = "database-server"
transport = "stdio"
command = "mcp-database"
args = ["--db", "production.db"]

[[servers]]
name = "file-server"
transport = "stdio"
command = "mcp-files"
args = ["--root", "/data"]

[[servers]]
name = "api-server"
transport = "sse"
url = "http://localhost:8080/mcp"
```

## Testing Configuration

After configuring servers, test the connection:

```bash
# Test all servers
rad mcp test

# Test a specific server
rad mcp test database-server
```

## Listing Tools

Discover available tools from configured servers:

```bash
# List all tools
rad mcp tools

# List tools from a specific server
rad mcp tools database-server
```

## Tool Conflict Resolution

When multiple servers provide tools with the same name, Radium automatically prefixes them:

```
Tools from database-server:
  query
  insert
  update

Tools from api-server:
  query          # Conflicts with database-server:query
  api-server:query  # Automatically prefixed
  call_endpoint
```

## OAuth Token Management

OAuth tokens are automatically stored in `~/.radium/mcp_tokens/`. The token manager handles:

- Token storage and retrieval
- Token expiration checking
- Token refresh (when implemented)

Tokens are stored per server in JSON format:

```json
{
  "access_token": "...",
  "token_type": "Bearer",
  "expires_at": 1234567890,
  "refresh_token": "..."
}
```

## Troubleshooting

### Server Not Connecting

1. Verify the server command exists and is executable
2. Check server logs for errors
3. Test the server manually:
   ```bash
   mcp-server --config config.json
   ```

### Tools Not Discovered

1. Ensure the server is connected: `rad mcp test <server-name>`
2. Check server capabilities support tools
3. Verify tool discovery response format

### Authentication Errors

1. Verify OAuth credentials in configuration
2. Check token expiration: tokens are automatically checked
3. Re-authenticate if needed (refresh flow to be implemented)

## Next Steps

- See [MCP Integration](../features/mcp-integration.md) for API usage
- Check [REQ-009](../plan/02-next/REQ-009-mcp-integration.md) for implementation details


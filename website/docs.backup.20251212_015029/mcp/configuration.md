# MCP Server Configuration

MCP servers are configured in `.radium/mcp-servers.toml` in your workspace root.

## Configuration File Location

The configuration file is located at:
- Workspace root: `.radium/mcp-servers.toml`
- Default location: `~/.radium/mcp-servers.toml` (if no workspace)

## Transport Types

### Stdio Transport

For local MCP servers that run as processes:

```toml
[[servers]]
name = "local-server"
transport = "stdio"
command = "mcp-server"
args = ["--config", "config.json"]
```

**Fields:**
- `name`: Unique identifier for the server
- `transport`: Must be `"stdio"`
- `command`: Command to execute the MCP server
- `args`: Optional array of command arguments

### SSE Transport

For remote servers using Server-Sent Events:

```toml
[[servers]]
name = "remote-server"
transport = "sse"
url = "https://api.example.com/mcp/sse"
```

**Fields:**
- `name`: Unique identifier for the server
- `transport`: Must be `"sse"`
- `url`: SSE endpoint URL

### HTTP Transport

For remote servers using HTTP streaming:

```toml
[[servers]]
name = "http-server"
transport = "http"
url = "https://api.example.com/mcp"
```

**Fields:**
- `name`: Unique identifier for the server
- `transport`: Must be `"http"`
- `url`: HTTP endpoint URL

## Multiple Servers

You can configure multiple MCP servers:

```toml
[[servers]]
name = "database-server"
transport = "stdio"
command = "mcp-database"
args = ["--db", "postgresql://localhost/mydb"]

[[servers]]
name = "api-server"
transport = "http"
url = "https://api.example.com/mcp"
```

## Authentication

For servers requiring authentication, see [Authentication Guide](authentication.md).

## Verifying Configuration

Test your configuration:

```bash
# List configured servers
rad mcp list

# Test connection
rad mcp test

# Test specific server
rad mcp test --server database-server
```

## Examples

### Example 1: Local Filesystem Server

```toml
[[servers]]
name = "filesystem"
transport = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/directory"]
```

### Example 2: PostgreSQL Database Server

```toml
[[servers]]
name = "postgres"
transport = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-postgres", "postgresql://user:pass@localhost/dbname"]
```

### Example 3: Remote API with OAuth

```toml
[[servers]]
name = "api-server"
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

### Example 4: Multiple Servers

```toml
# Database server
[[servers]]
name = "postgres-main"
transport = "stdio"
command = "mcp-postgres"
args = ["postgresql://localhost/maindb"]

# File server
[[servers]]
name = "filesystem"
transport = "stdio"
command = "mcp-filesystem"
args = ["--root", "/home/user/documents"]

# Remote API
[[servers]]
name = "github-api"
transport = "http"
url = "https://api.github.com/mcp"
auth = {
    auth_type = "oauth",
    params = {
        token_url = "https://github.com/login/oauth/access_token",
        client_id = "github-client-id",
        client_secret = "github-client-secret"
    }
}
```

## More Examples

See the [examples directory](../../examples/mcp/) for complete configuration examples:
- [Stdio Server Example](examples/stdio-server.md)
- [Remote Server Example](examples/remote-server.md)
- [OAuth Server Example](examples/oauth-server.md)


# MCP Integration

> **Status**: ✅ Implemented  
> **Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "mcp"`

Radium integrates with the Model Context Protocol (MCP) to enable external tool discovery and execution from MCP servers. This extends Radium's capabilities through an extensible tool ecosystem.

## Overview

MCP (Model Context Protocol) is a protocol that enables AI applications to connect to external data sources and tools. Radium's MCP integration provides:

- **Tool Discovery**: Automatically discover and register tools from MCP servers
- **Multiple Transports**: Support for stdio, SSE, and HTTP streaming transports
- **OAuth Authentication**: Secure authentication for remote MCP servers
- **Rich Content**: Support for text, images, and audio in tool responses
- **Slash Commands**: MCP prompts exposed as slash commands
- **Conflict Resolution**: Automatic prefixing for tool name conflicts

## Configuration

MCP servers are configured in `.radium/mcp-servers.toml`:

```toml
[[servers]]
name = "database-server"
transport = "stdio"
command = "mcp-server-db"
args = ["--config", "db.json"]

[[servers]]
name = "api-server"
transport = "sse"
url = "http://localhost:8080/sse"
auth = { auth_type = "oauth", client_id = "..." }

[[servers]]
name = "remote-server"
transport = "http"
url = "https://api.example.com/mcp"
```

### Transport Types

- **stdio**: Standard input/output for local servers (requires `command` and optional `args`)
- **sse**: Server-Sent Events for HTTP streaming (requires `url`)
- **http**: HTTP streaming for remote servers (requires `url`)

## CLI Commands

### List Servers

```bash
rad mcp list
```

Lists all configured MCP servers.

### List Tools

```bash
# List all tools from all servers
rad mcp tools

# List tools from a specific server
rad mcp tools database-server
```

### Test Connection

```bash
# Test all servers
rad mcp test

# Test a specific server
rad mcp test database-server
```

## Usage

### Tool Discovery

Tools from MCP servers are automatically discovered when Radium initializes. Tools are registered with automatic prefixing to resolve conflicts:

```
Built-in tools:
  file_read, file_write, shell_exec

MCP tools:
  database-server:query
  database-server:insert
  api-server:call_endpoint
```

### Tool Execution

Tools can be executed through the MCP client:

```rust
use radium_core::mcp::McpClient;

let client = McpClient::connect(&server_config).await?;
let result = client.execute_tool("query", &json!({"sql": "SELECT * FROM users"})).await?;
```

### Rich Content

MCP tools can return rich content types:

- **Text**: Plain text responses
- **Image**: Base64-encoded images or URLs
- **Audio**: Audio data or URLs

### OAuth Authentication

OAuth tokens are stored securely in `~/.radium/mcp_tokens/`:

```rust
use radium_core::mcp::OAuthTokenManager;

let mut token_manager = OAuthTokenManager::new(token_dir);
token_manager.load_tokens()?;
```

## Architecture

### Components

- **McpClient**: Main client for connecting to MCP servers
- **McpTransport**: Trait for transport implementations (stdio, SSE, HTTP)
- **McpToolRegistry**: Registry for managing discovered tools with conflict resolution
- **McpConfigManager**: Configuration loading and management
- **OAuthTokenManager**: Secure token storage and management
- **ContentHandler**: Rich content type detection and serialization
- **SlashCommandRegistry**: MCP prompts as slash commands

### Integration

MCP integration is automatically initialized when the agent system starts:

```rust
use radium_core::mcp::McpIntegration;

let integration = McpIntegration::new();
integration.initialize(&workspace).await?;
```

## Examples

See [MCP Server Setup Examples](../examples/mcp-server-setup.md) for detailed examples.

## References

- **Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "mcp"`
- [MCP Protocol Specification](https://modelcontextprotocol.io)
- [Gemini CLI MCP Documentation](https://geminicli.com/docs/tools/mcp-server)


# MCP (Model Context Protocol) Commands

Commands for managing MCP servers and tools.

## `rad mcp`

Manage MCP server configurations and tools.

### Subcommands

#### `list`

List configured MCP servers.

```bash
rad mcp list
```

#### `tools [server]`

List tools from MCP servers.

```bash
rad mcp tools [server]
```

#### `test [server]`

Test connection to MCP servers.

```bash
rad mcp test [server]
```

#### `prompts`

List available MCP prompts (slash commands).

```bash
rad mcp prompts
```

#### `auth`

OAuth authentication commands.

```bash
rad mcp auth status [server]
```

### Examples

```bash
# List all MCP servers
rad mcp list

# List tools from all servers
rad mcp tools

# List tools from specific server
rad mcp tools my-server

# Test all servers
rad mcp test

# Test specific server
rad mcp test my-server

# List MCP prompts
rad mcp prompts

# Check auth status
rad mcp auth status

# Check auth for specific server
rad mcp auth status my-server
```

### Configuration

MCP servers are configured in `.radium/mcp-servers.toml`:

```toml
[[servers]]
name = "my-server"
transport = "stdio"
command = "mcp-server"
args = ["--config", "config.json"]
```


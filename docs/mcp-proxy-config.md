# MCP Proxy Configuration Reference

Complete reference for the MCP proxy server configuration file (`.radium/mcp-proxy.toml`).

## Configuration File Location

The proxy configuration file is located at:
```
.radium/mcp-proxy.toml
```

## Configuration Structure

```toml
[mcp.proxy]
# Proxy server settings

[mcp.proxy.security]
# Security and logging settings

[[mcp.proxy.upstreams]]
# Upstream server configurations (array)
```

## Proxy Settings

### `[mcp.proxy]` Section

#### `enable` (boolean, default: `false`)

Whether the proxy server is enabled. Set to `true` to start the proxy.

```toml
[mcp.proxy]
enable = true
```

#### `port` (integer, default: `3000`)

Port number to listen on for agent connections. Must be in range 1-65535.

```toml
[mcp.proxy]
port = 3000
```

#### `transport` (string, default: `"sse"`)

Transport type for agent connections. Valid values:
- `"sse"`: Server-Sent Events transport
- `"http"`: HTTP transport

```toml
[mcp.proxy]
transport = "http"
```

#### `max_connections` (integer, default: `100`)

Maximum number of concurrent agent connections. Must be greater than 0.

```toml
[mcp.proxy]
max_connections = 100
```

## Security Settings

### `[mcp.proxy.security]` Section

#### `log_requests` (boolean, default: `true`)

Whether to log incoming tool execution requests.

```toml
[mcp.proxy.security]
log_requests = true
```

#### `log_responses` (boolean, default: `true`)

Whether to log tool execution responses.

```toml
[mcp.proxy.security]
log_responses = true
```

#### `redact_patterns` (array of strings, default: see below)

Regex patterns for sensitive data redaction in logs. Default patterns:
- `"api[_-]?key"`
- `"password"`
- `"token"`

```toml
[mcp.proxy.security]
redact_patterns = [
    "api[_-]?key",
    "password",
    "token",
    "secret"
]
```

#### `rate_limit_per_minute` (integer, default: `60`)

Rate limit per minute per agent/tool combination. Set to 0 to disable rate limiting (not recommended).

```toml
[mcp.proxy.security]
rate_limit_per_minute = 60
```

## Upstream Server Configuration

### `[[mcp.proxy.upstreams]]` Section

Each upstream server is configured as a table in the `upstreams` array.

#### Required Fields

- `name` (string): Unique identifier for this upstream server
- `transport` (string): Transport type (`"stdio"`, `"sse"`, or `"http"`)

#### Transport-Specific Fields

**For `stdio` transport:**
- `command` (string, required): Executable command to run
- `args` (array of strings, optional): Command-line arguments

**For `sse` or `http` transport:**
- `url` (string, required): Server endpoint URL

#### Optional Fields

- `priority` (integer, default: `1`): Upstream priority (lower number = higher priority)
- `health_check_interval` (integer, default: `30`): Health check interval in seconds
- `tools` (array of strings, optional): List of tool names this upstream provides (if specified, only these tools are used)

#### Authentication

- `auth` (table, optional): OAuth authentication configuration

## Example Configurations

### Single Upstream

```toml
[mcp.proxy]
enable = true
port = 3000
transport = "http"

[[mcp.proxy.upstreams]]
name = "my-server"
transport = "http"
url = "http://localhost:8080/mcp"
priority = 1
health_check_interval = 30
```

### Multiple Upstreams with Failover

```toml
[mcp.proxy]
enable = true
port = 3000
transport = "http"

[[mcp.proxy.upstreams]]
name = "primary-server"
transport = "http"
url = "http://server1.example.com/mcp"
priority = 1
health_check_interval = 30

[[mcp.proxy.upstreams]]
name = "backup-server"
transport = "http"
url = "http://server2.example.com/mcp"
priority = 2
health_check_interval = 30
```

### High Availability with Load Balancing

```toml
[mcp.proxy]
enable = true
port = 3000
transport = "http"

# Multiple servers with same priority = load balanced
[[mcp.proxy.upstreams]]
name = "server1"
transport = "http"
url = "http://server1.example.com/mcp"
priority = 1
health_check_interval = 30

[[mcp.proxy.upstreams]]
name = "server2"
transport = "http"
url = "http://server2.example.com/mcp"
priority = 1  # Same priority = load balanced
health_check_interval = 30
```

### Security-Focused Configuration

```toml
[mcp.proxy]
enable = true
port = 3000
transport = "http"
max_connections = 50

[mcp.proxy.security]
log_requests = true
log_responses = true
rate_limit_per_minute = 30
redact_patterns = [
    "api[_-]?key",
    "api[_-]?secret",
    "password",
    "token",
    "auth",
    "credential"
]

[[mcp.proxy.upstreams]]
name = "secure-server"
transport = "http"
url = "https://secure.example.com/mcp"
priority = 1
health_check_interval = 60
```

### Local Stdio Server

```toml
[mcp.proxy]
enable = true
port = 3000
transport = "http"

[[mcp.proxy.upstreams]]
name = "local-server"
transport = "stdio"
command = "/usr/local/bin/mcp-server"
args = ["--config", "server-config.json"]
priority = 1
health_check_interval = 30
```

### OAuth Authentication

```toml
[[mcp.proxy.upstreams]]
name = "oauth-server"
transport = "http"
url = "https://api.example.com/mcp"
priority = 1

[mcp.proxy.upstreams.auth]
auth_type = "oauth"
token_url = "https://api.example.com/oauth/token"
client_id = "your-client-id"
client_secret = "your-client-secret"
```

## Validation

The proxy validates configuration on startup and reports errors for:

- Invalid port numbers (must be 1-65535)
- Missing required fields for transport types
- Duplicate upstream names
- Invalid priority values (must be > 0)
- Invalid health check intervals (must be > 0)
- Invalid regex patterns in redact_patterns

## Configuration Updates

Configuration changes require restarting the proxy server:

```bash
rad mcp proxy stop
# Edit .radium/mcp-proxy.toml
rad mcp proxy start
```

Runtime configuration reloading is not currently supported but may be added in future versions.


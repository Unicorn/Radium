# MCP Proxy Server

## Introduction

The MCP Proxy Server provides a centralized gateway for accessing multiple MCP (Model Context Protocol) servers through a single endpoint. Instead of connecting directly to individual MCP servers, agents connect to the proxy, which handles routing, load balancing, failover, security, and tool catalog aggregation.

## Why Use the MCP Proxy?

- **Centralized Management**: Single configuration point for all upstream MCP servers
- **High Availability**: Automatic failover when upstream servers become unavailable
- **Load Balancing**: Distribute requests across multiple upstream servers
- **Security**: Centralized rate limiting, logging, and sensitive data redaction
- **Tool Aggregation**: Unified tool catalog from all upstream servers with conflict resolution

## Architecture Overview

```
Agent -> [MCP Proxy Server] -> Upstream MCP Servers
        (MCP Server)          (MCP Clients)
```

The proxy acts as:
- **MCP Server** to agents (accepts connections, handles tools/list, tools/call, etc.)
- **MCP Client** to upstream servers (connects to and forwards requests)

## Quick Start

### 1. Initialize Configuration

```bash
rad mcp proxy init
```

This creates a default configuration file at `.radium/mcp-proxy.toml`.

### 2. Configure Upstream Servers

Edit `.radium/mcp-proxy.toml` to add your upstream MCP servers:

```toml
[mcp.proxy]
enable = true
port = 3000
transport = "http"

[[mcp.proxy.upstreams]]
name = "server1"
transport = "http"
url = "http://localhost:8080/mcp"
priority = 1
health_check_interval = 30

[[mcp.proxy.upstreams]]
name = "server2"
transport = "http"
url = "http://localhost:8081/mcp"
priority = 2
health_check_interval = 30
```

### 3. Start the Proxy

```bash
rad mcp proxy start
```

The proxy will:
- Connect to all configured upstream servers
- Discover tools from each upstream
- Start health checking for all upstreams
- Begin accepting agent connections on the configured port

### 4. Connect Agents

Agents should connect to the proxy instead of individual upstream servers:

```
http://localhost:3000  (or your configured port)
```

### 5. Stop the Proxy

```bash
rad mcp proxy stop
```

Or press `Ctrl+C` if running in the foreground.

## Configuration

See [MCP Proxy Configuration Reference](./mcp-proxy-config.md) for complete configuration options.

## Use Cases

### Centralized Control

Manage all MCP server connections from a single location, making it easier to update endpoints, add new servers, or modify routing logic.

### High Availability

Configure multiple upstream servers providing the same tools. The proxy automatically fails over to backup servers if the primary becomes unavailable.

### Load Balancing

Distribute tool execution requests across multiple upstream servers to improve performance and reduce load on individual servers.

### Security and Compliance

Implement centralized security policies:
- Rate limiting to prevent abuse
- Request/response logging for auditing
- Sensitive data redaction in logs

## Components

### Upstream Pool

Manages connections to multiple upstream MCP servers, tracking connection state and health.

### Tool Router

Routes tool execution requests to appropriate upstream servers with:
- Explicit routing via `upstream:tool` syntax
- Round-robin load balancing
- Priority-based failover

### Tool Catalog

Aggregates tools from all upstream servers and handles name conflicts using configurable strategies:
- **AutoPrefix**: Automatically prefix conflicting tools with upstream name
- **Reject**: Keep first tool, reject duplicates
- **PriorityOverride**: Keep tool from highest priority upstream

### Security Layer

Enforces security policies:
- Rate limiting per agent/tool combination
- Request/response logging
- Sensitive data redaction using regex patterns

### Health Checker

Monitors upstream server health and automatically reconnects failed servers with exponential backoff.

## Troubleshooting

### Proxy Won't Start

**Error: Port already in use**
- Check if another proxy instance is running: `rad mcp proxy status`
- Stop existing instance: `rad mcp proxy stop`
- Or change the port in your configuration

### Upstream Connection Failures

**Error: Failed to connect to upstream**
- Verify the upstream server is running and accessible
- Check network connectivity
- Verify URL/command configuration is correct
- Check authentication credentials if using OAuth

### Rate Limit Errors

**Error: Rate limit exceeded**
- Adjust `rate_limit_per_minute` in configuration
- Check if multiple agents are using the same rate limit key
- Review security configuration

### Tools Not Appearing

**Tools from upstream not in catalog**
- Wait for tool discovery to complete (runs on startup)
- Check upstream connection status: `rad mcp proxy status`
- Verify upstream server is providing tools correctly
- Check conflict resolution strategy if tool names overlap

## CLI Commands

### `rad mcp proxy init`

Initialize a new proxy configuration file with defaults.

### `rad mcp proxy start`

Start the proxy server. Connects to all configured upstreams and begins accepting agent connections.

### `rad mcp proxy stop`

Stop a running proxy server gracefully.

### `rad mcp proxy status`

Check if the proxy server is running and display status information.

## Next Steps

- [Configuration Reference](./mcp-proxy-config.md)
- [MCP Client Documentation](../cli/mcp.md)
- [MCP Integration Guide](../guides/mcp-integration.md)


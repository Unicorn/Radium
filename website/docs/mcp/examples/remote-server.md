---
id: "remote-server"
title: "Remote Server Example"
sidebar_label: "Remote Server Example"
---

# Remote Server Example

This example shows how to configure a remote MCP server using HTTP or SSE transport.

## Overview

Remote servers run on a different host and communicate over HTTP. Radium supports two transport types for remote servers:

- **HTTP**: Standard HTTP requests/responses
- **SSE (Server-Sent Events)**: HTTP streaming with server-sent events

## HTTP Transport

### Basic Configuration

```toml
[[servers]]
name = "remote-api"
transport = "http"
url = "https://api.example.com/mcp"
```

### With Authentication

```toml
[[servers]]
name = "authenticated-api"
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

## SSE Transport

### Basic Configuration

```toml
[[servers]]
name = "streaming-api"
transport = "sse"
url = "https://api.example.com/mcp/sse"
```

### With Authentication

```toml
[[servers]]
name = "authenticated-streaming"
transport = "sse"
url = "https://api.example.com/mcp/sse"
auth = {
    auth_type = "oauth",
    params = {
        token_url = "https://api.example.com/oauth/token",
        client_id = "your-client-id"
    }
}
```

## Common Examples

### Example 1: Public API

```toml
[[servers]]
name = "public-api"
transport = "http"
url = "https://api.publicservice.com/mcp"
```

### Example 2: Internal API with OAuth

```toml
[[servers]]
name = "internal-api"
transport = "http"
url = "https://internal.company.com/mcp"
auth = {
    auth_type = "oauth",
    params = {
        token_url = "https://internal.company.com/oauth/token",
        client_id = "radium-client",
        client_secret = "secret-key"
    }
}
```

### Example 3: Streaming Service

```toml
[[servers]]
name = "streaming-service"
transport = "sse"
url = "https://stream.example.com/mcp/events"
auth = {
    auth_type = "oauth",
    params = {
        token_url = "https://stream.example.com/oauth/token",
        client_id = "stream-client"
    }
}
```

## Requirements

1. **Network Access**: Server must be accessible from your network
2. **HTTPS Recommended**: Use HTTPS for secure communication
3. **Authentication**: Configure OAuth for protected endpoints
4. **CORS**: Server must allow requests from Radium (if applicable)

## Testing

```bash
# Test connection
rad mcp test --server remote-api

# Verify tools
rad mcp tools --server remote-api
```

## Troubleshooting

### Connection Refused

**Problem**: Cannot connect to remote server

**Solution**:
1. Verify URL is correct:
   ```bash
   curl https://api.example.com/mcp
   ```
2. Check network connectivity
3. Verify firewall settings
4. Check if server requires authentication

### Authentication Errors

**Problem**: 401 Unauthorized responses

**Solution**:
1. Verify OAuth credentials are correct
2. Check token status:
   ```bash
   rad mcp auth status
   ```
3. Ensure `token_url` is correct
4. Verify client_id and client_secret
5. Check token storage: `~/.radium/mcp_tokens/`

### Timeout Errors

**Problem**: Connection times out

**Solution**:
1. Check network latency
2. Verify server is responding:
   ```bash
   curl -v https://api.example.com/mcp
   ```
3. Check firewall/proxy settings
4. Verify server is not overloaded

### SSL/TLS Errors

**Problem**: Certificate validation errors

**Solution**:
1. Verify server certificate is valid
2. Check certificate chain
3. Ensure system time is correct
4. For development, server may need to accept self-signed certificates

## Best Practices

1. **Use HTTPS**: Always use HTTPS for remote servers
2. **Secure Credentials**: Store OAuth credentials securely
3. **Monitor Health**: Regularly check server connectivity
4. **Handle Errors**: Implement retry logic for transient failures
5. **Test First**: Test connection before relying on tools

## Security Considerations

1. **Token Storage**: Tokens are stored in `~/.radium/mcp_tokens/` with restricted permissions (0600)
2. **Credential Management**: Don't commit credentials to version control
3. **Network Security**: Use HTTPS to encrypt communication
4. **Token Refresh**: Tokens are automatically refreshed when expired

## Related Documentation

- [Configuration Guide](../configuration.md)
- [OAuth Setup Guide](../oauth-setup.md)
- [Troubleshooting](../troubleshooting.md)
- [User Guide](../user-guide.md)


---
id: "oauth-server"
title: "OAuth Server Example"
sidebar_label: "OAuth Server Example"
---

# OAuth Server Example

This example shows how to configure an MCP server with OAuth authentication.

## Overview

OAuth authentication is used for remote servers that require secure access. Radium handles token acquisition, storage, and refresh automatically.

## Basic OAuth Configuration

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

## OAuth Flow

1. **Initial Setup**: Configure OAuth parameters in server config
2. **Token Acquisition**: First token may need to be obtained manually (see [OAuth Setup Guide](../oauth-setup.md))
3. **Token Storage**: Tokens stored in `~/.radium/mcp_tokens/{server-name}.json`
4. **Automatic Refresh**: Tokens are refreshed automatically when expired

## Provider Examples

### GitHub OAuth

```toml
[[servers]]
name = "github-mcp"
transport = "http"
url = "https://api.github.com/mcp"
auth = {
    auth_type = "oauth",
    params = {
        token_url = "https://github.com/login/oauth/access_token",
        client_id = "your-github-client-id",
        client_secret = "your-github-client-secret"
    }
}
```

### Google OAuth

```toml
[[servers]]
name = "google-mcp"
transport = "http"
url = "https://api.google.com/mcp"
auth = {
    auth_type = "oauth",
    params = {
        token_url = "https://oauth2.googleapis.com/token",
        client_id = "your-google-client-id",
        client_secret = "your-google-client-secret"
    }
}
```

### Custom OAuth Provider

```toml
[[servers]]
name = "custom-api"
transport = "http"
url = "https://api.custom.com/mcp"
auth = {
    auth_type = "oauth",
    params = {
        token_url = "https://api.custom.com/oauth/token",
        client_id = "your-client-id",
        client_secret = "your-client-secret",
        scope = "read write"  # Optional: specify scopes
    }
}
```

## Token Management

### Check Token Status

```bash
rad mcp auth status
```

### Token Storage Location

Tokens are stored in: `~/.radium/mcp_tokens/{server-name}.json`

### Token File Format

```json
{
  "access_token": "eyJhbGc...",
  "token_type": "Bearer",
  "refresh_token": "def502...",
  "expires_at": 1234567890,
  "scope": "read write"
}
```

### Token Permissions

Token files have restricted permissions (0600) on Unix systems:
- Owner: read/write
- Group: no access
- Others: no access

## Troubleshooting

### Token Not Found

**Problem**: "No token found for server"

**Solution**:
1. Check token file exists: `~/.radium/mcp_tokens/{server-name}.json`
2. Verify server name matches config
3. Initial token may need manual acquisition
4. Check token file permissions

### Token Expired

**Problem**: "OAuth token expired"

**Solution**:
1. Tokens should auto-refresh, but check:
   ```bash
   rad mcp auth status
   ```
2. Verify `refresh_token` is present in token file
3. Check `token_url` is correct
4. Verify client credentials are valid

### Refresh Token Missing

**Problem**: "No refresh token available"

**Solution**:
1. Some providers don't return refresh tokens
2. May need to re-authenticate manually
3. Check provider documentation for refresh token requirements
4. Verify OAuth flow includes refresh token grant

### Invalid Credentials

**Problem**: "Authentication error: invalid client"

**Solution**:
1. Verify `client_id` is correct
2. Verify `client_secret` is correct
3. Check credentials haven't been revoked
4. Ensure OAuth app is properly configured with provider

## Security Best Practices

1. **Secure Storage**: Tokens stored with restricted permissions (0600)
2. **No Version Control**: Never commit tokens or credentials
3. **Credential Rotation**: Rotate credentials periodically
4. **HTTPS Only**: Always use HTTPS for token endpoints
5. **Scope Limitation**: Request only necessary OAuth scopes

## Advanced Configuration

### Custom Token Endpoint

Some providers use different token endpoint formats:

```toml
auth = {
    auth_type = "oauth",
    params = {
        token_url = "https://api.example.com/v2/oauth/token",
        client_id = "client-id",
        client_secret = "client-secret",
        grant_type = "refresh_token"  # Some providers require explicit grant type
    }
}
```

### Token Refresh Behavior

Tokens are automatically refreshed when:
- Token is expired (based on `expires_at`)
- Server returns 401 Unauthorized
- Before connection if token is already expired

## Related Documentation

- [OAuth Setup Guide](../oauth-setup.md) - Detailed OAuth setup instructions
- [Configuration Guide](../configuration.md) - General configuration reference
- [Troubleshooting](../troubleshooting.md) - Common OAuth issues
- [User Guide](../user-guide.md) - Getting started guide


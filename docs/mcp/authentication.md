# MCP Authentication

Radium supports OAuth 2.0 authentication for remote MCP servers.

## OAuth Configuration

Add authentication configuration to your server entry:

```toml
[[servers]]
name = "oauth-server"
transport = "http"
url = "https://api.example.com/mcp"

[auth]
auth_type = "oauth"
token_url = "https://api.example.com/oauth/token"
client_id = "your-client-id"
client_secret = "your-client-secret"
```

**OAuth Parameters:**
- `auth_type`: Must be `"oauth"`
- `token_url`: OAuth token endpoint URL
- `client_id`: OAuth client ID (optional for refresh token flow)
- `client_secret`: OAuth client secret (optional for refresh token flow)

## Token Storage

OAuth tokens are stored in `~/.radium/mcp_tokens/` as JSON files, one per server.

**Security Note**: Token files contain sensitive credentials. Ensure proper file permissions.

## Token Refresh

Radium automatically refreshes OAuth tokens when they expire:

1. Tokens are checked before each request
2. If expired, refresh token is used to obtain a new access token
3. New token is saved automatically

## Checking Token Status

View token status for configured servers:

```bash
# Show status for all servers
rad mcp auth status

# Show status for specific server
rad mcp auth status --server oauth-server
```

## Token Lifecycle

1. **Initial Connection**: If no token exists, connection may fail. You may need to obtain an initial token manually.
2. **Automatic Refresh**: Tokens are automatically refreshed when expired (if refresh token is available).
3. **Token Persistence**: Tokens persist across application restarts.

## Troubleshooting

**Token expired errors:**
- Check token status: `rad mcp auth status`
- Verify refresh token is available
- Check token_url is correct

**Authentication failures:**
- Verify client_id and client_secret (if required)
- Check token_url endpoint is accessible
- Ensure OAuth server supports refresh token flow

## Example Configuration

See [oauth-server.toml](../../examples/mcp/oauth-server.toml) for a complete OAuth configuration example.


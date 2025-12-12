---
id: "oauth-setup"
title: "OAuth Setup Guide"
sidebar_label: "OAuth Setup Guide"
---

# OAuth Setup Guide

This guide provides step-by-step instructions for setting up OAuth authentication with MCP servers.

## Table of Contents

1. [Introduction](#introduction)
2. [OAuth Concepts](#oauth-concepts)
3. [Prerequisites](#prerequisites)
4. [Step-by-Step Setup](#step-by-step-setup)
5. [Provider Examples](#provider-examples)
6. [Token Management](#token-management)
7. [Security Best Practices](#security-best-practices)
8. [Troubleshooting](#troubleshooting)

## Introduction

OAuth 2.0 is an industry-standard protocol for authorization. Radium uses OAuth to securely authenticate with remote MCP servers that require authentication.

### Why OAuth?

- **Security**: Tokens are stored securely and automatically refreshed
- **Standard**: Works with any OAuth 2.0 compliant provider
- **Automatic**: Token refresh happens automatically
- **Flexible**: Supports various OAuth providers (GitHub, Google, custom)

## OAuth Concepts

### Key Terms

- **Client ID**: Public identifier for your application
- **Client Secret**: Secret key for your application (keep secure!)
- **Access Token**: Token used to authenticate API requests
- **Refresh Token**: Token used to obtain new access tokens
- **Token URL**: Endpoint for obtaining and refreshing tokens
- **Authorization URL**: Endpoint for user authorization (if required)

### OAuth Flow

1. **Registration**: Register your application with the OAuth provider
2. **Configuration**: Add OAuth credentials to MCP server config
3. **Initial Token**: Obtain initial access token (may require manual step)
4. **Automatic Refresh**: Radium automatically refreshes tokens when expired

## Prerequisites

Before setting up OAuth, you need:

1. **OAuth Provider Account**: Account with the OAuth provider (GitHub, Google, etc.)
2. **OAuth Application**: Registered OAuth application with the provider
3. **Credentials**: Client ID and Client Secret from the provider
4. **Token Endpoint**: URL for token acquisition and refresh

## Step-by-Step Setup

### Step 1: Register OAuth Application

Register your application with the OAuth provider:

1. Log in to the provider's developer portal
2. Create a new OAuth application
3. Configure redirect URI (if required by provider)
4. Note the **Client ID** and **Client Secret**

### Step 2: Configure MCP Server

Add OAuth configuration to your MCP server config:

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

### Step 3: Obtain Initial Token

For the first connection, you may need to obtain an initial token manually:

1. **Option A: Provider-specific flow**
   - Some providers provide a web interface for token generation
   - Follow provider's documentation for initial token acquisition

2. **Option B: OAuth Authorization Flow**
   - Use OAuth authorization URL to get authorization code
   - Exchange authorization code for access token
   - Save token to `~/.radium/mcp_tokens/{server-name}.json`

3. **Option C: Personal Access Token**
   - Some providers (like GitHub) allow personal access tokens
   - Create token in provider settings
   - Use as initial access token

### Step 4: Verify Configuration

Test your OAuth setup:

```bash
# Test connection
rad mcp test --server oauth-server

# Check token status
rad mcp auth status
```

### Step 5: Automatic Refresh

Once configured, Radium automatically:
- Detects when tokens are expired
- Refreshes tokens using the refresh token
- Updates stored tokens
- Retries failed requests after refresh

## Provider Examples

### GitHub OAuth

#### Step 1: Create GitHub OAuth App

1. Go to GitHub Settings → Developer settings → OAuth Apps
2. Click "New OAuth App"
3. Fill in:
   - Application name: "Radium MCP"
   - Homepage URL: `https://radium.dev`
   - Authorization callback URL: `http://localhost:8080/callback` (or your callback URL)
4. Click "Register application"
5. Note the **Client ID**
6. Generate a **Client Secret**

#### Step 2: Configure Radium

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

#### Step 3: Initial Token (GitHub Personal Access Token)

For GitHub, you can use a Personal Access Token:

1. Go to GitHub Settings → Developer settings → Personal access tokens
2. Generate new token (classic)
3. Select required scopes
4. Copy the token
5. Create token file manually:

```bash
mkdir -p ~/.radium/mcp_tokens
cat > ~/.radium/mcp_tokens/github-mcp.json <<EOF
{
  "access_token": "ghp_your_token_here",
  "token_type": "Bearer",
  "expires_at": null
}
EOF
chmod 600 ~/.radium/mcp_tokens/github-mcp.json
```

### Google OAuth

#### Step 1: Create Google OAuth Credentials

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select existing
3. Enable the API you need
4. Go to "Credentials" → "Create Credentials" → "OAuth client ID"
5. Configure OAuth consent screen
6. Create OAuth client ID (Web application)
7. Note the **Client ID** and **Client Secret**

#### Step 2: Configure Radium

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

#### Step 3: Initial Token

Google OAuth typically requires an authorization flow:

1. Use Google's OAuth playground or a tool to get initial token
2. Save token with refresh token to `~/.radium/mcp_tokens/google-mcp.json`

### Custom OAuth Provider

#### Step 1: Get Provider Information

Obtain from your OAuth provider:
- Token endpoint URL
- Client ID
- Client Secret
- Required scopes (if any)

#### Step 2: Configure Radium

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
        scope = "read write"  # Optional: provider-specific scopes
    }
}
```

#### Step 3: Initial Token

Follow your provider's documentation for initial token acquisition.

## Token Management

### Token Storage

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

### Checking Token Status

```bash
# Check token status for all servers
rad mcp auth status

# Check specific server
rad mcp auth status --server github-mcp
```

### Manual Token Refresh

Tokens refresh automatically, but you can trigger a refresh:

```bash
# Refresh token for a server
rad mcp auth refresh --server github-mcp
```

### Token Expiration

Tokens are automatically refreshed when:
- Token is expired (based on `expires_at`)
- Server returns 401 Unauthorized
- Before connection if token is already expired

### Token Permissions

Token files have restricted permissions (0600) on Unix:
- Owner: read/write
- Group: no access
- Others: no access

## Security Best Practices

### 1. Secure Credential Storage

- **Never commit credentials** to version control
- Store credentials in environment variables or secure vault
- Use `.gitignore` to exclude config files with secrets

### 2. Token Security

- Tokens stored with restricted permissions (0600)
- Tokens automatically refreshed to minimize exposure
- Old tokens invalidated when refreshed

### 3. Credential Rotation

- Rotate client secrets periodically
- Revoke old tokens when rotating
- Update configuration with new credentials

### 4. Scope Limitation

- Request only necessary OAuth scopes
- Use least privilege principle
- Review and minimize granted permissions

### 5. HTTPS Only

- Always use HTTPS for token endpoints
- Verify SSL certificates
- Don't use HTTP for production

### 6. Environment Variables

For sensitive credentials, consider using environment variables:

```toml
[[servers]]
name = "api-server"
transport = "http"
url = "https://api.example.com/mcp"
auth = {
    auth_type = "oauth",
    params = {
        token_url = "https://api.example.com/oauth/token",
        client_id = "${OAUTH_CLIENT_ID}",
        client_secret = "${OAUTH_CLIENT_SECRET}"
    }
}
```

Note: Environment variable substitution may need to be implemented or done manually.

## Troubleshooting

### Token Not Found

**Problem**: "No token found for server"

**Solutions:**
1. Check token file exists: `~/.radium/mcp_tokens/{server-name}.json`
2. Verify server name matches configuration
3. Initial token may need manual acquisition
4. Check token file permissions

### Token Expired

**Problem**: "OAuth token expired"

**Solutions:**
1. Tokens should auto-refresh - check refresh token is present
2. Verify `token_url` is correct
3. Check client credentials are valid
4. Ensure refresh token hasn't been revoked

### Refresh Token Missing

**Problem**: "No refresh token available"

**Solutions:**
1. Some providers don't return refresh tokens
2. May need to re-authenticate manually
3. Check provider documentation for refresh token requirements
4. Verify OAuth flow includes refresh token grant

### Invalid Credentials

**Problem**: "Authentication error: invalid client"

**Solutions:**
1. Verify `client_id` is correct
2. Verify `client_secret` is correct
3. Check credentials haven't been revoked
4. Ensure OAuth app is properly configured with provider

### Token Refresh Fails

**Problem**: Token refresh returns error

**Solutions:**
1. Verify `token_url` is correct
2. Check refresh token is valid
3. Verify client credentials are correct
4. Check provider's token refresh requirements
5. Review provider logs for detailed error

### 401 Unauthorized After Refresh

**Problem**: Still getting 401 after token refresh

**Solutions:**
1. Verify new token is being used
2. Check token has required scopes
3. Verify server accepts the token format
4. Check token hasn't been revoked on provider side

## Common OAuth Providers

### GitHub

- **Token URL**: `https://github.com/login/oauth/access_token`
- **Authorization URL**: `https://github.com/login/oauth/authorize`
- **Documentation**: https://docs.github.com/en/apps/oauth-apps

### Google

- **Token URL**: `https://oauth2.googleapis.com/token`
- **Authorization URL**: `https://accounts.google.com/o/oauth2/v2/auth`
- **Documentation**: https://developers.google.com/identity/protocols/oauth2

### Microsoft/Azure AD

- **Token URL**: `https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token`
- **Authorization URL**: `https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize`
- **Documentation**: https://docs.microsoft.com/en-us/azure/active-directory/develop/

### Custom Provider

For custom OAuth providers, you'll need:
- Token endpoint URL
- Authorization endpoint URL (if required)
- Client ID and Secret
- Required scopes
- Provider-specific configuration

## Advanced Topics

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

Refresh uses the refresh token to obtain a new access token without user interaction.

### Multiple OAuth Servers

You can configure multiple OAuth-authenticated servers:

```toml
[[servers]]
name = "github-api"
transport = "http"
url = "https://api.github.com/mcp"
auth = { auth_type = "oauth", params = { ... } }

[[servers]]
name = "google-api"
transport = "http"
url = "https://api.google.com/mcp"
auth = { auth_type = "oauth", params = { ... } }
```

Each server maintains its own token independently.

## Related Documentation

- [User Guide](user-guide.md) - Getting started with MCP
- [Configuration Guide](configuration.md) - General configuration reference
- [OAuth Server Example](examples/oauth-server.md) - OAuth configuration examples
- [Troubleshooting](troubleshooting.md) - Common OAuth issues
- [Authentication](authentication.md) - Authentication overview

## References

- [OAuth 2.0 Specification](https://oauth.net/2/)
- [OAuth 2.0 Best Practices](https://oauth.net/2/oauth-best-practices/)
- [Provider Documentation](#common-oauth-providers) - Links to provider-specific docs


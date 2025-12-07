# MCP Troubleshooting

Common issues and solutions for MCP integration.

## Connection Issues

### Server Not Connecting

**Symptoms:**
- `rad mcp test` shows server as not connected
- Tools not available
- Slash commands not working

**Solutions:**
1. Verify server configuration:
   ```bash
   rad mcp list
   ```

2. Check server is running (for stdio transport):
   ```bash
   # Test the command manually
   mcp-server --config config.json
   ```

3. Verify URL is accessible (for HTTP/SSE):
   ```bash
   curl https://api.example.com/mcp
   ```

4. Check logs for detailed error messages

### Connection Timeout

**Symptoms:**
- Connection hangs
- Timeout errors

**Solutions:**
- Check network connectivity
- Verify firewall settings
- Increase timeout in configuration (if supported)

## Authentication Issues

### OAuth Token Expired

**Symptoms:**
- Authentication errors
- 401 Unauthorized responses

**Solutions:**
1. Check token status:
   ```bash
   rad mcp auth status
   ```

2. Verify refresh token is available
3. Check token_url is correct
4. Re-authenticate if needed

### Token Not Found

**Symptoms:**
- "No token found" errors

**Solutions:**
- Initial token may need to be obtained manually
- Check token storage directory exists: `~/.radium/mcp_tokens/`
- Verify server has auth configured

## Tool Issues

### Tools Not Available

**Symptoms:**
- `rad mcp tools` shows no tools
- Agents can't find MCP tools

**Solutions:**
1. Verify server is connected:
   ```bash
   rad mcp test
   ```

2. Check server has tools configured
3. Review server logs for tool discovery errors
4. Ensure server implements MCP protocol correctly

### Tool Execution Fails

**Symptoms:**
- Tool calls return errors
- "Tool not found" errors

**Solutions:**
1. Verify tool name is correct (check for server prefix)
2. Check tool arguments match schema
3. Review server logs for execution errors
4. Test tool directly on server if possible

## Slash Command Issues

### Commands Not Found

**Symptoms:**
- `/command` not recognized
- "Unknown command" errors

**Solutions:**
1. List available commands:
   ```bash
   rad mcp prompts
   ```

2. Ensure MCP server is initialized
3. Check prompt names are correctly normalized
4. Verify server has prompts configured

### Command Execution Fails

**Symptoms:**
- Slash command returns error
- No response from command

**Solutions:**
1. Check server connection
2. Verify prompt arguments
3. Review server logs
4. Test prompt directly on server

## Content Issues

### Rich Content Not Displaying

**Symptoms:**
- Images/audio not showing
- Content appears as text

**Solutions:**
1. Check content type is supported
2. Verify content format matches MCP spec
3. Check temp directory permissions
4. Review content handler logs

## General Debugging

### Enable Debug Logging

Set environment variable for detailed logs:

```bash
RUST_LOG=radium_core::mcp=debug rad mcp test
```

### Check Configuration

Validate configuration file:

```bash
# List servers
rad mcp list

# Test connections
rad mcp test
```

### Review Logs

Check application logs for MCP-related messages:

- Connection attempts
- Tool discovery
- Execution errors
- Authentication issues

## Getting Help

If issues persist:

1. Check [MCP Protocol Specification](https://modelcontextprotocol.io)
2. Review server documentation
3. Check server logs
4. Verify server implements MCP correctly
5. Test with minimal configuration

## Common Error Messages

### "Failed to connect to MCP server"

**Possible Causes:**
- Server not running (stdio)
- URL incorrect (HTTP/SSE)
- Network issues
- Authentication required

**Solutions:**
1. For stdio: Verify server executable is in PATH and running
2. For HTTP/SSE: Test URL with `curl` to verify accessibility
3. Check network connectivity and firewall settings
4. Verify authentication is configured if required

### "Tool not found"

**Possible Causes:**
- Tool name incorrect
- Server prefix missing
- Tool not discovered
- Server connection issue

**Solutions:**
1. List available tools: `rad mcp tools`
2. Check tool name includes server prefix (e.g., `server-name/tool-name`)
3. Verify server is connected: `rad mcp test`
4. Check server provides the tool you're looking for

### "Authentication error"

**Possible Causes:**
- Token expired
- Invalid credentials
- Token URL incorrect
- Refresh token missing

**Solutions:**
1. Check token status: `rad mcp auth status`
2. Verify OAuth credentials in configuration
3. Check `token_url` is correct
4. Ensure refresh token is available for auto-refresh
5. Re-authenticate if needed

### "MCP configuration error: Stdio transport requires 'command' field"

**Problem:** Missing required field in configuration

**Solution:**
```toml
# Add the missing field
[[servers]]
name = "my-server"
transport = "stdio"
command = "mcp-server"  # Required for stdio transport
```

### "MCP configuration error: HTTP transport requires 'url' field"

**Problem:** Missing URL for HTTP/SSE transport

**Solution:**
```toml
# Add the missing field
[[servers]]
name = "my-server"
transport = "http"
url = "https://api.example.com/mcp"  # Required for HTTP/SSE transport
```

### "MCP transport error: Failed to spawn process"

**Problem:** Cannot execute the server command

**Solutions:**
1. Verify command is in PATH: `which mcp-server`
2. Use full path: `command = "/usr/local/bin/mcp-server"`
3. Check file permissions: `chmod +x /path/to/mcp-server`
4. For npm packages, use `npx`: `command = "npx"`

### "MCP connection error: Connection closed"

**Problem:** Server disconnected unexpectedly

**Solutions:**
1. Check server logs for errors
2. Verify server is still running
3. Check network stability (for remote servers)
4. Review server resource usage (memory, CPU)


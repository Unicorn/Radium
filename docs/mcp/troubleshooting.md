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

- Server not running (stdio)
- URL incorrect (HTTP/SSE)
- Network issues
- Authentication required

### "Tool not found"

- Tool name incorrect
- Server prefix missing
- Tool not discovered
- Server connection issue

### "Authentication error"

- Token expired
- Invalid credentials
- Token URL incorrect
- Refresh token missing


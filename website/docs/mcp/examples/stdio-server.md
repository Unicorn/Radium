---
id: "stdio-server"
title: "Stdio Server Example"
sidebar_label: "Stdio Server Example"
---

# Stdio Server Example

This example shows how to configure a local MCP server that runs as a process using stdio transport.

## Overview

Stdio transport is used for local MCP servers that run as processes. Communication happens through standard input/output streams.

## Basic Configuration

```toml
[[servers]]
name = "local-server"
transport = "stdio"
command = "mcp-server"
args = ["--config", "config.json"]
```

## Common Examples

### Example 1: Simple Command

```toml
[[servers]]
name = "filesystem-server"
transport = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/directory"]
```

### Example 2: Command with Multiple Arguments

```toml
[[servers]]
name = "database-server"
transport = "stdio"
command = "mcp-postgres"
args = [
    "--host", "localhost",
    "--port", "5432",
    "--database", "mydb",
    "--user", "postgres"
]
```

### Example 3: Using Environment Variables

```toml
[[servers]]
name = "env-server"
transport = "stdio"
command = "mcp-server"
args = ["--env-file", ".env"]
```

## Requirements

1. **Executable in PATH**: The command must be available in your system PATH
2. **Proper Permissions**: The executable must have execute permissions
3. **Working Directory**: Some servers may require a specific working directory

## Testing

```bash
# Test the server connection
rad mcp test --server local-server

# Verify tools are available
rad mcp tools --server local-server
```

## Troubleshooting

### Command Not Found

**Problem**: `rad mcp test` fails with "command not found"

**Solution**:
1. Verify the command is in your PATH:
   ```bash
   which mcp-server
   ```
2. Use full path if not in PATH:
   ```toml
   command = "/usr/local/bin/mcp-server"
   ```
3. For npm packages, use `npx`:
   ```toml
   command = "npx"
   args = ["-y", "@modelcontextprotocol/server-name"]
   ```

### Server Exits Immediately

**Problem**: Server connects but immediately disconnects

**Solution**:
1. Check server logs for errors
2. Verify server configuration is correct
3. Test server manually:
   ```bash
   mcp-server --config config.json
   ```
4. Ensure server implements MCP protocol correctly

### Permission Denied

**Problem**: Cannot execute the command

**Solution**:
1. Check file permissions:
   ```bash
   ls -l /path/to/mcp-server
   ```
2. Make executable if needed:
   ```bash
   chmod +x /path/to/mcp-server
   ```

## Best Practices

1. **Use Absolute Paths**: For production, use absolute paths to avoid PATH issues
2. **Test Manually First**: Test the server command manually before configuring
3. **Check Logs**: Monitor server output for errors
4. **Handle Errors**: Implement proper error handling in your server

## Related Documentation

- [Configuration Guide](../configuration.md)
- [Troubleshooting](../troubleshooting.md)
- [User Guide](../user-guide.md)


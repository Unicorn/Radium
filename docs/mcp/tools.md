# Using MCP Tools

MCP tools from configured servers are automatically available to agents during execution.

## Tool Discovery

Tools are automatically discovered when MCP servers are initialized:

```bash
# List available tools
rad mcp tools

# List tools from specific server
rad mcp tools --server database-server
```

## Tool Naming

Tools use the format `server-name:tool-name` to avoid conflicts:

- If a tool name is unique, it's available as `tool-name`
- If there's a conflict, tools are prefixed: `server-name:tool-name`

**Example:**
```
database-server:query
api-server:query
```

Both servers have a `query` tool, so they're prefixed with server names.

## Using Tools in Agents

MCP tools are automatically available to agents. No special configuration is needed.

**Example agent execution:**
```json
{
  "id": "data-agent",
  "name": "Data Agent",
  "description": "Agent that uses database tools",
  "prompt": "prompts/data-agent.md"
}
```

When this agent executes, it can use MCP tools like `database-server:query` just like built-in tools.

## Tool Execution

Tools are executed through the MCP protocol:

1. Agent requests tool execution
2. Radium routes to appropriate MCP server
3. Tool executes on remote server
4. Results returned to agent

## Rich Content Support

MCP tools can return rich content:

- **Text**: Displayed inline
- **Images**: Saved to temp files, paths shown
- **Audio**: Saved to temp files, paths shown

**Example output:**
```
Tool executed successfully
[Image: image/png] Saved to: /tmp/radium_mcp_abc123.png
```

## Error Handling

MCP tool errors are handled gracefully:

- Connection errors are logged
- Tool execution errors are returned to agent
- Failed servers don't block other servers

## Tool Schemas

Tool input schemas are automatically converted from MCP JSON Schema format to agent-compatible format. Agents receive proper schema information for validation.

## Examples

See [agent-with-mcp-tools.toml](../../examples/mcp/agent-with-mcp-tools.toml) for an example agent configuration.


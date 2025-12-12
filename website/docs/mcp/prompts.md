---
id: "prompts"
title: "MCP Slash Commands"
sidebar_label: "MCP Slash Commands"
---

# MCP Slash Commands

MCP server prompts are automatically registered as slash commands in CLI and TUI chat interfaces.

## Command Discovery

Prompts are automatically discovered and registered when MCP servers are initialized:

```bash
# List available slash commands
rad mcp prompts
```

## Command Format

Slash commands use the format `/prompt-name`:

- Prompt names are normalized (spaces → underscores, lowercase)
- Example: `"Search Database"` becomes `/search_database`

## Using Slash Commands

### In CLI Chat

```bash
rad chat assistant
> /search_database query="SELECT * FROM users"
```

### In TUI

Type slash commands directly in the chat interface:

```
> /search_database query="SELECT * FROM users"
```

## Command Arguments

Arguments are passed as space-separated values:

```bash
/search query text here
```

Arguments are converted to JSON based on the prompt's argument schema.

## Help Command

View available slash commands:

```bash
# In chat
/help

# Or list MCP commands specifically
/mcp-commands
```

## Command Execution

1. Command is parsed from input
2. Lookup in slash command registry
3. Server identified from registry
4. Prompt executed via MCP
5. Result displayed

## Example Prompts

Common MCP prompts might include:

- `/search` - Search functionality
- `/translate` - Translation service
- `/analyze` - Data analysis
- `/generate` - Content generation

## Troubleshooting

**Command not found:**
- Ensure MCP server is connected: `rad mcp test`
- Check prompts are available: `rad mcp prompts`
- Verify server has prompts configured

**Execution errors:**
- Check server connection status
- Verify prompt arguments match schema
- Review server logs for errors

## Examples

See [basic-server.toml](../../examples/mcp/basic-server.toml) for a server configuration with prompts.


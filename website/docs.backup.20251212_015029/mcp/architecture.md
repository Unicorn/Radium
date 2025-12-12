# MCP Integration Architecture

This document describes the architecture of MCP (Model Context Protocol) integration in Radium.

## Overview

MCP integration allows Radium to connect to external MCP servers, discover their tools and prompts, and make them available to agents and users through the orchestration system and CLI/TUI interfaces.

## Component Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│  - CLI Commands (rad mcp *)                                 │
│  - TUI Chat Interface                                        │
│  - Agent Execution                                          │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                  McpIntegration                             │
│  - Server connection management                              │
│  - Tool discovery and registry                               │
│  - Prompt discovery and slash command registration          │
│  - Configuration loading (workspace + extensions)           │
└──────────┬───────────────────────────────┬──────────────────┘
           │                               │
           ▼                               ▼
┌──────────────────────┐      ┌──────────────────────────────┐
│    McpClient         │      │   SlashCommandRegistry        │
│  - Transport layer   │      │   - Prompt-to-command mapping│
│  - JSON-RPC protocol │      │   - Server association        │
│  - OAuth integration │      └──────────────────────────────┘
└──────────┬───────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────┐
│                    Transport Layer                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                  │
│  │  Stdio   │  │   SSE    │  │   HTTP   │                  │
│  │Transport │  │Transport │  │Transport │                  │
│  └──────────┘  └──────────┘  └──────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

## Data Flow

### Tool Discovery Flow

```
1. McpIntegration::initialize()
   ↓
2. Load configs (workspace + extensions)
   ↓
3. For each server:
   - McpClient::connect() → Transport::connect()
   - McpClient::discover_tools() → JSON-RPC "tools/list"
   - McpToolRegistry::register_tool() → Conflict resolution
   ↓
4. Tools available via McpIntegration::get_all_tools()
```

### Tool Execution Flow

```
1. Agent requests tool execution
   ↓
2. Orchestrator → McpIntegration::execute_tool()
   ↓
3. McpClient::execute_tool() → JSON-RPC "tools/call"
   ↓
4. ContentHandler::parse_content() → McpContent enum
   ↓
5. Result returned to agent
```

### Prompt/Slash Command Flow

```
1. McpIntegration::initialize()
   ↓
2. McpClient::list_prompts() → JSON-RPC "prompts/list"
   ↓
3. SlashCommandRegistry::register_prompt_with_server()
   ↓
4. Commands available in chat: /prompt-name
   ↓
5. User types /command → SlashCommandRegistry lookup
   ↓
6. McpClient::execute_prompt() → JSON-RPC "prompts/get"
   ↓
7. Result displayed
```

## Key Components

### McpIntegration

Central manager for MCP server connections:
- Manages multiple server connections
- Coordinates tool and prompt discovery
- Handles configuration precedence (workspace > extension)
- Thread-safe with Arc<Mutex<>>

### McpClient

Per-server connection handler:
- Manages transport layer
- Handles JSON-RPC protocol
- Integrates OAuth token management
- Discovers tools and prompts

### McpToolRegistry

Per-server tool storage:
- Stores discovered tools
- Handles name conflict resolution
- Supports dual-lookup (original + prefixed names)

### SlashCommandRegistry

Prompt-to-command mapping:
- Maps prompts to slash commands
- Tracks server associations
- Used by chat interfaces

### Transport Layer

Three transport implementations:
- **StdioTransport**: Local process communication
- **SseTransport**: Server-Sent Events streaming
- **HttpTransport**: HTTP request/response

## Configuration Precedence

1. **Workspace config** (`.radium/mcp-servers.toml`) - Highest precedence
2. **Extension configs** (from installed extensions) - Lower precedence

If a server name exists in both, workspace config takes precedence.

## Thread Safety

All shared state uses Arc<Mutex<>>:
- `McpIntegration::clients` - Thread-safe client storage
- `McpIntegration::tool_registries` - Thread-safe registry access
- `McpIntegration::slash_registry` - Thread-safe command registry

## Error Handling

- Connection failures are logged but don't block other servers
- Tool execution errors are propagated to agents
- OAuth token refresh happens automatically before requests
- Failed servers are skipped during initialization

## Integration Points

### Orchestrator Bridge

`crates/radium-orchestrator/src/orchestration/mcp_tools.rs`:
- Converts MCP tools to orchestration Tool objects
- Handles rich content (saves images/audio to temp files)
- Provides agent-facing API

### CLI Integration

`apps/cli/src/commands/mcp.rs`:
- `rad mcp list` - List configured servers
- `rad mcp tools` - List available tools
- `rad mcp prompts` - List slash commands
- `rad mcp test` - Test connections
- `rad mcp auth status` - Check OAuth tokens

### Chat Integration

`apps/cli/src/commands/chat.rs`:
- Loads MCP prompts into SlashCommandRegistry
- Executes slash commands via MCP
- Displays MCP command help

## OAuth Flow

```
1. Server config includes OAuth auth block
   ↓
2. OAuthTokenManager loads tokens from ~/.radium/mcp_tokens/
   ↓
3. Before connection: Check token expiration
   ↓
4. If expired: Refresh token using refresh_token
   ↓
5. Bearer token injected into transport headers (SSE/HTTP)
   ↓
6. Token saved for future use
```

## Content Handling

MCP tools can return rich content:
- **Text**: Direct string content
- **Image**: Base64 or URL, saved to temp file
- **Audio**: Base64 or URL, saved to temp file

ContentHandler parses and serializes content types for API compatibility.

## Extension Support

Extensions can provide MCP configs in JSON format:
- Loaded from extension directories
- Lower precedence than workspace configs
- Supports all transport types

## Future Enhancements

- Performance optimization (connection pooling)
- Advanced OAuth flows (PKCE)
- Binary content types
- Tool schema validation
- Prompt templates


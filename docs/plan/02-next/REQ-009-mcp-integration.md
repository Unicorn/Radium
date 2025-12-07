---
req_id: REQ-009
title: MCP Integration
phase: NEXT
status: Not Started
priority: High
estimated_effort: 4-5 hours
dependencies: [REQ-002]
related_docs:
  - docs/features/gemini-cli-enhancements.md#mcp-model-context-protocol-integration
  - docs/project/03-implementation-plan.md#step-1-agent-configuration-system
---

# MCP Integration

## Problem Statement

Agents need access to external tools and services beyond what's built into Radium. Without MCP (Model Context Protocol) integration, agents cannot:
- Discover and use tools from external MCP servers
- Integrate with databases, APIs, and custom workflows
- Leverage community-contributed MCP servers
- Access specialized tools for specific domains
- Extend Radium's capabilities dynamically

The Model Context Protocol enables external tool discovery and execution from MCP servers. Radium needs MCP integration to create an extensible tool ecosystem similar to gemini-cli.

## Solution Overview

Implement MCP client integration that provides:
- Tool discovery from MCP servers
- Multiple transport support (stdio, SSE, HTTP streaming)
- OAuth authentication for remote servers
- Tool conflict resolution with automatic prefixing
- Rich content support (text, images, audio) in tool responses
- MCP prompts as slash commands
- Schema validation and sanitization for API compatibility

The MCP integration extends Radium's capabilities through external tools, enabling integration with databases, APIs, and custom workflows.

## Functional Requirements

### FR-1: MCP Client Implementation

**Description**: MCP client for connecting to and communicating with MCP servers.

**Acceptance Criteria**:
- [ ] MCP client implementation
- [ ] Support for stdio transport
- [ ] Support for SSE (Server-Sent Events) transport
- [ ] Support for HTTP streaming transport
- [ ] Connection management and error handling
- [ ] Protocol message parsing and serialization

**Implementation**: `crates/radium-core/src/mcp/mod.rs`

### FR-2: Tool Discovery

**Description**: Discover and register tools from MCP servers.

**Acceptance Criteria**:
- [ ] Tool discovery from MCP servers
- [ ] Tool registry integration
- [ ] Tool conflict resolution with automatic prefixing
- [ ] Tool metadata and schema storage
- [ ] Tool availability checking

**Implementation**: `crates/radium-core/src/mcp/tools.rs`

### FR-3: Authentication

**Description**: OAuth authentication for remote MCP servers.

**Acceptance Criteria**:
- [ ] OAuth flow for authenticated servers
- [ ] Token storage and management
- [ ] Token refresh logic
- [ ] Multiple authentication methods support

**Implementation**: `crates/radium-core/src/mcp/auth.rs`

### FR-4: Rich Content Support

**Description**: Support for rich content types in tool responses.

**Acceptance Criteria**:
- [ ] Text content support
- [ ] Image content support
- [ ] Audio content support
- [ ] Content type detection and handling
- [ ] Content serialization for API compatibility

**Implementation**: `crates/radium-core/src/mcp/content.rs`

### FR-5: MCP Prompts as Slash Commands

**Description**: Expose MCP server prompts as slash commands.

**Acceptance Criteria**:
- [ ] MCP prompt discovery
- [ ] Slash command registration
- [ ] Prompt execution through MCP
- [ ] Command help and documentation

**Implementation**: `crates/radium-core/src/mcp/prompts.rs`

## Technical Requirements

### TR-1: MCP Protocol

**Description**: Model Context Protocol specification and implementation.

**Protocol**: MCP protocol as defined by Model Context Protocol specification

**Transport Methods**:
- stdio: Standard input/output for local servers
- SSE: Server-Sent Events for HTTP streaming
- HTTP: HTTP streaming for remote servers

### TR-2: MCP Client API

**Description**: APIs for MCP client operations.

**APIs**:
```rust
pub struct McpClient {
    transport: Box<dyn McpTransport>,
    server_info: McpServerInfo,
}

impl McpClient {
    pub fn connect(server_config: &McpServerConfig) -> Result<Self>;
    pub fn discover_tools(&self) -> Result<Vec<McpTool>>;
    pub fn execute_tool(&self, tool_name: &str, arguments: &Value) -> Result<McpToolResult>;
    pub fn list_prompts(&self) -> Result<Vec<McpPrompt>>;
}
```

### TR-3: Tool Conflict Resolution

**Description**: Automatic prefixing for tool name conflicts.

**Strategy**: When tool names conflict, automatically prefix with server identifier (e.g., `server-name:tool-name`)

## User Experience

### UX-1: MCP Server Configuration

**Description**: Users configure MCP servers in workspace settings.

**Example**:
```toml
# .radium/mcp-servers.toml
[[servers]]
name = "database-server"
transport = "stdio"
command = "mcp-server-db"
args = ["--config", "db.json"]
```

### UX-2: Tool Discovery

**Description**: Tools from MCP servers are automatically discovered.

**Example**:
```bash
$ rad tools list
Built-in tools:
  file_read, file_write, shell_exec

MCP tools:
  database-server:query
  database-server:insert
  api-server:call_endpoint
```

## Data Requirements

### DR-1: MCP Server Configuration

**Description**: TOML configuration for MCP servers.

**Location**: `.radium/mcp-servers.toml` or workspace configuration

**Schema**: Server name, transport type, command, arguments, authentication

## Dependencies

- **REQ-002**: Agent Configuration - Required for agent system and tool integration

## Success Criteria

1. [ ] MCP client can connect to MCP servers
2. [ ] Tools can be discovered from MCP servers
3. [ ] Tool conflicts are resolved with automatic prefixing
4. [ ] OAuth authentication works for remote servers
5. [ ] Rich content types are supported in tool responses
6. [ ] MCP prompts are available as slash commands
7. [ ] All MCP operations have comprehensive test coverage

**Completion Metrics**:
- **Status**: Not Started
- **Estimated Effort**: 4-5 hours
- **Priority**: High

## Out of Scope

- MCP server implementation (external)
- Advanced MCP protocol features (future enhancement)
- MCP server marketplace (future enhancement)

## References

- [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#mcp-model-context-protocol-integration)
- [Implementation Plan](../project/03-implementation-plan.md#step-1-agent-configuration-system)
- [MCP Protocol Specification](https://modelcontextprotocol.io)


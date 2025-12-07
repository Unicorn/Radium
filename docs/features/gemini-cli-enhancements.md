# Gemini CLI Enhancements for Radium

> **Note**: Detailed feature requirements have been extracted to [/docs/plan](../plan/README.md) for structured implementation planning.

> **Source**: Features and patterns learned from [gemini-cli](https://github.com/google-gemini/gemini-cli)  
> **Last Updated**: 2025-12-02

This document catalogs valuable features and architectural patterns from gemini-cli that can enhance Radium's capabilities. These enhancements are integrated into Radium's roadmap and implementation plan.

## Overview

Gemini CLI is an open-source AI agent that brings the power of Gemini directly into the terminal. It provides lightweight access to Gemini with built-in tools, extensibility through MCP, and a terminal-first design. Many of its features align well with Radium's goals and can significantly enhance the platform.

## High Priority Features (NEXT Phase)

### MCP (Model Context Protocol) Integration

**Priority**: 🟡 High  
**Status**: Planned for Step 1  
**Est. Time**: 4-5 hours

**Description**: Integration with the Model Context Protocol to enable external tool discovery and execution from MCP servers.

**Key Features**:
- Tool discovery from MCP servers
- Multiple transport support (stdio, SSE, HTTP streaming)
- OAuth authentication for remote servers
- Tool conflict resolution with automatic prefixing
- Rich content support (text, images, audio) in tool responses
- MCP prompts as slash commands

**Implementation Details**:
- MCP client implementation in `crates/radium-core/src/mcp/mod.rs`
- Support for stdio, SSE, and HTTP transports
- OAuth flow for authenticated servers
- Tool registry integration with conflict resolution
- Schema validation and sanitization for Gemini API compatibility

**Benefits**:
- Extends Radium's capabilities through external tools
- Enables integration with databases, APIs, and custom workflows
- Supports community-contributed MCP servers
- Foundation for extensible tool ecosystem

**Requirements**: [REQ-009: MCP Integration](../plan/02-next/REQ-009-mcp-integration.md)

**Reference**: [Gemini CLI MCP Documentation](https://geminicli.com/docs/tools/mcp-server)

---

### Policy Engine for Tool Execution

**Priority**: 🟡 High  
**Status**: Planned for Step 3  
**Est. Time**: 6-7 hours

**Description**: Fine-grained control over tool execution through rule-based policies.

**Key Features**:
- TOML-based policy rule system
- Tool execution control (allow/deny/ask_user)
- Priority-based rule matching with tiered policies (Default/User/Admin)
- Approval modes (yolo, autoEdit)
- Pattern matching for tool names and arguments
- Special syntax for shell commands and MCP tools

**Implementation Details**:
- Policy engine in `crates/radium-core/src/policy/mod.rs`
- TOML policy file parsing
- Rule evaluation with priority system
- Integration with tool execution flow
- Approval mode support

**Benefits**:
- Enhanced security through controlled tool execution
- Flexible policy configuration per workspace
- Enterprise-ready with admin policy support
- User-friendly approval workflows

**Requirements**: [REQ-010: Policy Engine](../plan/02-next/REQ-010-policy-engine.md)

**Reference**: [Gemini CLI Policy Engine Documentation](https://geminicli.com/docs/core/policy-engine)

---

### Context Files (GEMINI.md)

**Priority**: 🟡 High  
**Status**: Planned for Step 1  
**Est. Time**: 3-4 hours

**Description**: Hierarchical context file system for providing persistent instructions to agents.

**Key Features**:
- Hierarchical loading (global → project root → subdirectory)
- Automatic context file discovery
- Context imports with `@file.md` syntax
- Customizable context file names
- Memory management commands

**Implementation Details**:
- Context file loader in `crates/radium-core/src/context/files.rs`
- Hierarchical scanning and loading
- Import resolution and processing
- Configuration for custom file names
- Integration with prompt system

**Benefits**:
- Project-specific agent behavior customization
- Persistent instructions without repeating in prompts
- Modular context organization
- Team-shared context files via version control

**Requirements**: [REQ-011: Context Files](../plan/02-next/REQ-011-context-files.md)

**Reference**: [Gemini CLI Context Files Documentation](https://geminicli.com/docs/cli/gemini-md)

---

### Custom Commands (TOML-based)

**Priority**: 🟡 High  
**Status**: Planned for Step 5  
**Est. Time**: 5-6 hours

**Description**: TOML-based system for defining reusable agent commands and prompts.

**Key Features**:
- TOML-based command definitions
- Command discovery (user vs project precedence)
- Shell command injection (`!{command}`)
- File content injection (`@{file}`)
- Argument handling (`{{args}}`)
- Namespaced commands via directory structure

**Implementation Details**:
- Custom command system in `crates/radium-core/src/commands/custom.rs`
- TOML parsing and validation
- Command discovery and precedence
- Syntax parsing for injections
- Integration with CLI command system

**Benefits**:
- Reusable command shortcuts
- Dynamic command execution with shell/file injection
- Project-specific and user-specific commands
- Improved developer productivity

**Requirements**: [REQ-012: Custom Commands](../plan/02-next/REQ-012-custom-commands.md)

**Reference**: [Gemini CLI Custom Commands Documentation](https://geminicli.com/docs/cli/custom-commands)

---

### Checkpointing System

**Priority**: 🟡 High  
**Status**: Planned for Step 6  
**Est. Time**: 6-7 hours

**Description**: Automatic Git snapshots and conversation history preservation for safe experimentation.

**Key Features**:
- Git snapshot creation before file modifications
- Shadow Git repository management
- Conversation history preservation
- `/restore` command functionality
- Tool call re-proposal after restore

**Implementation Details**:
- Checkpointing system in `crates/radium-core/src/checkpoint/mod.rs`
- Shadow Git repository in `~/.radium/history/`
- Checkpoint storage and retrieval
- Restore command implementation
- Integration with workflow execution

**Benefits**:
- Safe experimentation with code changes
- Easy rollback to previous states
- Conversation context preservation
- Confidence in trying new approaches

**Requirements**: [REQ-013: Checkpointing](../plan/02-next/REQ-013-checkpointing.md)

**Reference**: [Gemini CLI Checkpointing Documentation](https://geminicli.com/docs/cli/checkpointing)

---

### Sandboxing

**Priority**: 🟡 High  
**Status**: Planned for Step 6.5  
**Est. Time**: 12-15 hours

**Description**: Isolated execution environments for safe agent operations.

**Key Features**:
- Multiple sandbox methods (Docker/Podman/macOS Seatbelt)
- Configurable sandbox profiles (permissive/restrictive)
- Network control (open/closed/proxied)
- Custom sandbox flags
- Linux UID/GID handling

**Implementation Details**:
- Sandbox abstraction in `crates/radium-core/src/sandbox/mod.rs`
- Docker/Podman implementation
- macOS Seatbelt integration
- Sandbox configuration system
- Profile management

**Benefits**:
- Enhanced security for agent execution
- Isolation from host system
- Reproducible environments
- Reduced risk of accidental damage

**Requirements**: [REQ-008: Sandboxing](../plan/02-next/REQ-008-sandboxing.md)

**Reference**: [Gemini CLI Sandboxing Documentation](https://geminicli.com/docs/cli/sandbox)

---

## Medium Priority Features (LATER Phase)

### Extension System

**Priority**: 🟢 Medium  
**Status**: Planned for Step 10  
**Est. Time**: 8-10 hours

**Description**: Installable extensions that package prompts, MCP servers, and custom commands.

**Key Features**:
- Extension discovery and loading
- `gemini-extension.json` configuration
- Extension registry
- MCP server integration via extensions
- Custom commands from extensions
- Extension settings management
- User/workspace scoping

**Implementation Details**:
- Extension system in `crates/radium-core/src/extensions/mod.rs`
- Extension discovery from `~/.radium/extensions/`
- Configuration parsing and validation
- Integration with MCP and command systems
- Settings prompt system

**Benefits**:
- Community-contributed extensions
- Easy sharing of agent configurations
- Extensible platform ecosystem
- Workspace-specific extensions

**Reference**: [Gemini CLI Extensions Documentation](https://geminicli.com/docs/extensions/index)

---

### Hooks System

**Priority**: 🟢 Medium  
**Status**: Planned for Step 10  
**Est. Time**: 4-5 hours

**Description**: Intercept and customize behavior at various points in the execution flow.

**Key Features**:
- Hook registration system
- Before/after model call hooks
- Tool selection and execution hooks
- Error handling hooks
- Telemetry hooks
- Hook configuration in settings

**Implementation Details**:
- Hooks system in `crates/radium-core/src/hooks/mod.rs`
- Hook registration and execution
- Integration points throughout the system
- Configuration and settings

**Benefits**:
- Advanced customization without core code changes
- Behavior modification at runtime
- Telemetry and monitoring integration
- Plugin-like functionality

**Reference**: Gemini CLI hooks system (internal implementation)

---

## Architecture Patterns

### Tool Architecture

**Pattern**: Base tool interface with clear contracts, tool registry, and result separation.

**Key Components**:
- Base tool trait with required methods
- Tool registry with discovery mechanisms
- Tool result separation (llmContent vs returnDisplay)
- Rich content support (multimodal tool responses)
- Confirmation system with trust levels

**Benefits**:
- Consistent tool interface
- Easy tool extension
- Clear separation of concerns
- Flexible result handling

---

### CLI/Core Separation

**Pattern**: Clear separation between CLI (frontend) and Core (backend).

**Key Components**:
- CLI package for user-facing interactions
- Core package for orchestration and execution
- Well-defined API between layers
- Independent development paths

**Benefits**:
- Modular architecture
- Potential for different frontends
- Clean separation of concerns
- Easier testing and maintenance

---

## Integration Points

### Step 1: Agent Configuration System
- MCP Integration (1.4)
- Context Files System (1.5)

### Step 3: Workflow Behaviors
- Policy Engine (3.5)

### Step 5: Memory & Context System
- Custom Commands System (5.4)

### Step 6: Monitoring & Telemetry
- Checkpointing System (6.5)

### Step 6.5: Sandboxing
- Complete sandboxing implementation

### Step 10: Advanced Features
- Extension System (10.6)
- Hooks System (10.7)

---

## References

- [Gemini CLI GitHub Repository](https://github.com/google-gemini/gemini-cli)
- [Gemini CLI Documentation](https://geminicli.com/docs/)
- [Model Context Protocol Specification](https://modelcontextprotocol.io/)

---

## Notes

- All features are adapted to fit Radium's Rust architecture
- Time estimates are based on gemini-cli implementation complexity
- Features are prioritized based on value and integration complexity
- Some features may be simplified or combined during implementation


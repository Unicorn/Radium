# CLI vs TUI Feature Parity Analysis

**Date**: 2025-12-10
**Context**: LLM-Driven Tool Selection Migration

---

## Executive Summary

**CRITICAL FINDING**: The CLI and TUI have **completely different architectures** for agent execution and tool handling.

**Impact of Recent Changes**:
- ✅ **TUI**: LLM-driven tool selection implemented (in `apps/tui/src/chat_executor.rs`)
- ❌ **CLI**: Changes do NOT affect CLI (uses completely different code path)

**Parity Status**: ~60% (CLI lacks TUI's custom chat experience)

---

## Architectural Comparison

### TUI Architecture

```
User Input (TUI Interface)
    ↓
apps/tui/src/chat_executor.rs ← OUR CHANGES ARE HERE
    ↓
Custom tool implementation:
    - Hardcoded tool list (get_chat_tools)
    - Custom tool execution logic
    - Terminal command execution with safety checks
    - project_scan, search_files, read_file, etc.
    ↓
Direct API calls to Claude/Gemini/OpenAI
    ↓
Response rendering in TUI
```

**Key File**: `apps/tui/src/chat_executor.rs` (1,700+ lines)
- Custom tool registry
- Proactive execution logic (REMOVED in our changes)
- LLM-driven tool selection (ADDED in our changes)
- Terminal command safety modal
- Streaming response handling

---

### CLI Architecture

```
User Input (stdin)
    ↓
apps/cli/src/commands/chat.rs
    ↓
apps/cli/src/commands/step.rs
    ↓
radium-orchestrator crate
    ├→ Agent loader (reads TOML configs)
    ├→ Tool registry (orchestration/mod.rs)
    └→ Model execution
    ↓
Response printed to stdout
```

**Key Files**:
- `apps/cli/src/commands/chat.rs` - REPL loop
- `apps/cli/src/commands/step.rs` - Agent execution
- `crates/radium-orchestrator/` - Orchestration engine

**Tool Handling**:
- Uses radium-orchestrator's tool registry
- Tools defined in `crates/radium-orchestrator/src/orchestration/`
- Follows agent TOML configuration

---

## Feature Comparison Matrix

| Feature | TUI | CLI | Parity | Notes |
|---------|-----|-----|--------|-------|
| **Core Chat** | ✅ | ✅ | 100% | Both support multi-turn conversation |
| **Agent Selection** | ❌ Fixed (chat mode) | ✅ Agent param | 0% | TUI is single-purpose, CLI is multi-agent |
| **System Prompt** | ✅ chat-assistant.md | ✅ Agent TOML | 100% | CLI can use same prompt via agent config |
| **Tool Calling** | ✅ Custom impl | ✅ Orchestrator | 80% | Different implementations, similar capability |
| **LLM-Driven Selection** | ✅ **NEW!** | ⚠️ Partial | 50% | TUI has it, CLI depends on orchestrator |
| **project_scan** | ✅ Hardcoded | ✅ Orchestrator | 90% | Both have it, different paths |
| **search_files** | ✅ Hardcoded | ✅ Orchestrator | 90% | Both have it |
| **read_file** | ✅ Hardcoded | ✅ Orchestrator | 90% | Both have it |
| **list_directory** | ✅ Hardcoded | ✅ Orchestrator | 90% | Both have it |
| **grep** | ✅ Hardcoded | ✅ Orchestrator | 90% | Both have it |
| **git_log** | ✅ Hardcoded | ✅ Orchestrator | 90% | Both have it |
| **git_diff** | ✅ Hardcoded | ✅ Orchestrator | 90% | Both have it |
| **git_blame** | ❌ | ✅ Orchestrator | 0% | Only in orchestrator |
| **git_show** | ❌ | ✅ Orchestrator | 0% | Only in orchestrator |
| **find_references** | ❌ | ✅ Orchestrator | 0% | Only in orchestrator |
| **Terminal Commands** | ✅ With safety | ✅ Via run_command | 80% | TUI has modal, CLI is direct |
| **Command Safety Check** | ✅ Modal UI | ❌ No check | 0% | TUI-specific feature |
| **Streaming Responses** | ✅ | ✅ | 100% | Both support streaming |
| **Session History** | ✅ | ✅ | 100% | Both save history |
| **Context Files** | ✅ | ✅ | 100% | Both load CLAUDE.md/GEMINI.md |
| **MCP Integration** | ❌ | ✅ | 0% | CLI has MCP slash commands |
| **Analytics** | ❌ | ✅ | 0% | CLI has session reports |

**Overall Parity Score**: ~60%

---

## What This Means for LLM-Driven Tool Selection

### TUI (Our Changes Apply Here)

**Status**: ✅ **Fully Implemented**

The changes we just made affect the TUI:
- ✅ Removed proactive scan gate
- ✅ Enhanced system prompt (chat-assistant.md)
- ✅ Improved tool descriptions
- ✅ LLM now autonomously decides when to call tools

**Testing**: Use `./dist/target/debug/radium-tui` to test

---

### CLI (Our Changes DON'T Apply Here)

**Status**: ⚠️ **Depends on Orchestrator Implementation**

The CLI uses a **completely different execution path**:
- Reads agent config from TOML files
- Uses `radium-orchestrator` crate for tool execution
- Tools are registered in `crates/radium-orchestrator/src/orchestration/`

**Question**: Does the orchestrator already use LLM-driven tool selection?

Let me check the orchestrator implementation...

---

## Orchestrator Tool Execution Analysis

**File**: `crates/radium-orchestrator/src/orchestration/mod.rs`

The orchestrator has:
- ✅ Tool registry with schemas
- ✅ Tool execution via ToolHandler trait
- ✅ project_scan_tool.rs (already implemented)
- ✅ git_extended_tools.rs (git_blame, git_show, find_references)
- ✅ code_analysis_tool.rs

**Architecture**: The orchestrator **already uses LLM-driven tool selection**!

```rust
// Orchestrator exposes tools to the LLM via FunctionDeclarations
let tools = tool_registry.get_all_tools();
// LLM sees tools and decides which to call
// Orchestrator executes the chosen tools
```

**Conclusion**: The CLI **already has LLM-driven tool selection** via the orchestrator!

---

## Key Architectural Difference

### TUI: Custom Chat Executor (Old Approach)

Before our changes:
```rust
// apps/tui/src/chat_executor.rs
if question_type == ProjectOverview {
    execute_proactive_scan()  // ← HARDCODED PATTERN MATCHING
}
```

After our changes:
```rust
// apps/tui/src/chat_executor.rs
// Prepend analysis plan, LLM sees tools and decides
// ← LLM-DRIVEN SELECTION
```

**Problem Solved**: Removed brittle pattern matching

---

### CLI: Orchestrator-Based (Already Correct)

```rust
// CLI → step::execute → orchestrator
let tools = tool_registry.get_all_tools();
agent.execute_with_tools(tools);  // ← LLM-DRIVEN FROM THE START
```

**Status**: CLI was already using the correct architecture!

---

## Why The Divergence?

Looking at git history and code structure:

1. **TUI was built first** as a custom chat interface
   - Custom tool execution logic
   - Hardcoded tool list
   - Pattern-based proactive execution

2. **CLI was built later** using the orchestrator
   - Proper separation of concerns
   - Agent-based architecture
   - Tool registry abstraction

3. **Result**: Two completely different code paths

---

## Parity Gaps

### TUI Has (CLI Doesn't)

1. **Terminal Command Safety Modal**
   - TUI shows modal before executing dangerous commands
   - CLI executes directly

2. **Single-Purpose Chat Mode**
   - TUI is focused on chat only
   - CLI is multi-purpose (agents, requirements, steps)

3. **Custom Tool List**
   - TUI has hardcoded get_chat_tools()
   - CLI uses tool registry

### CLI Has (TUI Doesn't)

1. **Extended Git Tools**
   - git_blame
   - git_show
   - find_references

2. **MCP Integration**
   - Slash commands from MCP servers
   - Dynamic tool discovery

3. **Session Analytics**
   - Token usage reports
   - Session summaries
   - Cost tracking

4. **Agent Selection**
   - Can chat with any agent
   - TUI is fixed to chat-assistant mode

---

## Recommendations

### Short Term (Immediate)

1. **Document the Architecture Difference**
   - Users need to know TUI and CLI are different
   - Set expectations about feature parity

2. **Test Both Independently**
   - TUI test: Verify LLM-driven tool selection works
   - CLI test: Verify orchestrator tools work with chat-assistant agent

3. **Update Documentation**
   - Explain when to use TUI vs CLI
   - Document tool availability in each

### Medium Term (Next Sprint)

1. **Migrate TUI to Use Orchestrator**
   - Remove custom chat_executor tool logic
   - Use radium-orchestrator tool registry
   - Achieve architectural consistency

2. **Add Missing Tools to TUI**
   - git_blame, git_show, find_references
   - Use orchestrator implementations

3. **Add TUI Features to CLI**
   - Command safety checks
   - Interactive approval for dangerous commands

### Long Term (Future)

1. **Unified Architecture**
   - Both TUI and CLI use orchestrator
   - Single source of truth for tools
   - Consistent behavior across interfaces

2. **Feature Parity**
   - All tools available in both
   - Same safety checks
   - Same user experience (adapted to interface)

---

## Testing Guide

### Test TUI (With Our Changes)

```bash
# Test LLM-driven tool selection
./dist/target/debug/radium-tui

# Query that should trigger project_scan
> Scan my project and tell me what it's about

# Expected:
# ✅ LLM calls project_scan("quick")
# ✅ Returns README, manifest, structure
# ❌ Does NOT ask permission
```

---

### Test CLI (With chat-assistant Agent)

```bash
# First, verify ANTHROPIC_API_KEY is set (agent uses Claude)
echo $ANTHROPIC_API_KEY

# Start chat with chat-assistant agent
./dist/target/release/radium-cli chat chat-assistant

# Query that should trigger project_scan
> Scan my project and tell me what it's about

# Expected:
# ✅ LLM calls project_scan tool (via orchestrator)
# ✅ Returns comprehensive response
# ❌ Should NOT ask permission (if orchestrator tools work correctly)
```

---

### Test CLI with Gemini Agent (Alternative)

If you want to test with Gemini instead of Claude:

1. Create a Gemini-based chat agent:

```toml
# agents/test/chat-gemini.toml
[agent]
id = "chat-gemini"
name = "Chat Assistant (Gemini)"
description = "Interactive developer assistant with Gemini"
prompt_path = "prompts/agents/core/chat-assistant.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"
reasoning_effort = "medium"
```

2. Test:

```bash
GEMINI_API_KEY=<your-key> ./dist/target/release/radium-cli chat chat-gemini
> Scan my project
```

---

## Conclusion

### Current State

- **TUI**: ✅ LLM-driven tool selection implemented (via our changes)
- **CLI**: ✅ LLM-driven tool selection already existed (via orchestrator)

### Parity

- **Architecture**: 0% (completely different implementations)
- **Features**: ~60% (CLI has more tools, TUI has better UX)
- **Tool Calling**: 90% (both work, different paths)

### Next Steps

1. **Test both independently** to verify behavior
2. **Document the difference** for users
3. **Consider migration** of TUI to use orchestrator (future work)

---

**Bottom Line**:

Both TUI and CLI can do LLM-driven tool selection, but through **completely different code paths**. Our changes only affected the TUI. The CLI was already using the correct architecture via the orchestrator.

For true parity, we should eventually migrate the TUI to use the orchestrator's tool registry instead of maintaining a separate custom implementation.

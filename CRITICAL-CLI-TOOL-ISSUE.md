# 🚨 CRITICAL: CLI Has No Tools Registered

**Date**: 2025-12-10
**Severity**: BLOCKER
**Impact**: CLI agents cannot execute any tools

---

## The Problem

When testing the CLI with the `chat-gemini` agent:

```bash
$ radium-cli chat chat-gemini
> Scan this project and summarize it

Response:
"Okay, I understand. I will use tools immediately..."

Tool Calls: 0
```

**The agent acknowledged the system prompt but DIDN'T CALL ANY TOOLS.**

---

## Root Cause

**The CLI orchestrator is NOT registering tools with agents.**

### Evidence

```
Tool Calls:                 0 ( ✓ 0 x 0 )
Success Rate:               0.0%
```

The agent has **ZERO tools available** to call.

### Architecture Gap

**What EXISTS**:
- ✅ Tool modules (project_scan_tool.rs, git_extended_tools.rs, etc.)
- ✅ Tool definitions with schemas
- ✅ ToolRegistry struct

**What's MISSING**:
- ❌ Tools not being added to registry in agent execution
- ❌ No call to `registry.register_tool()` for analysis tools
- ❌ Agents receive empty tool list

---

## Comparison: TUI vs CLI

### TUI (WORKS) ✅

```rust
// apps/tui/src/chat_executor.rs:get_chat_tools()
fn get_chat_tools() -> Vec<Tool> {
    vec![
        Tool { name: "project_scan", ...},      // ← Hardcoded list
        Tool { name: "search_files", ...},
        Tool { name: "read_file", ...},
        Tool { name: "grep", ...},
        // ... 8+ tools total
    ]
}
```

**Result**: TUI agents have tools available ✅

---

### CLI (BROKEN) ❌

```rust
// apps/cli/src/commands/step.rs
// → Orchestrator execution
// → ??? Where are tools registered? ???
```

**Result**: CLI agents have NO tools ❌

---

## Expected vs Actual

### Expected Behavior

```
User: "Scan this project"
  ↓
Agent sees tools: [project_scan, read_file, grep, ...]
  ↓
Agent calls: project_scan(depth: "quick")
  ↓
Tool executes → Returns README, manifest, structure
  ↓
Agent synthesizes response
```

### Actual Behavior

```
User: "Scan this project"
  ↓
Agent sees tools: []  ← EMPTY!
  ↓
Agent acknowledges system prompt (no tools to call)
  ↓
Returns generic response
```

---

## Where Tools Should Be Registered

### Option 1: In Orchestrator Service

**File**: `crates/radium-orchestrator/src/orchestration/service.rs`

```rust
// When creating orchestration service for an agent
let mut tool_registry = ToolRegistry::new();

// Register file tools
tool_registry.register_tools(file_tools::create_file_tools(workspace_root.clone()));

// Register project analysis tools  ← MISSING!
tool_registry.register_tools(project_scan_tool::create_project_analysis_tools(workspace_root.clone()));

// Register git tools  ← MISSING!
tool_registry.register_tools(git_extended_tools::create_git_extended_tools(workspace_root.clone()));

// Register code analysis tools  ← MISSING!
tool_registry.register_tool(code_analysis_tool::create_code_analysis_tool(workspace_root.clone()));
```

---

### Option 2: In Step Execution

**File**: `apps/cli/src/commands/step.rs`

```rust
// Before calling agent.execute()
let tools = build_tool_list_for_agent(&workspace, &agent_config);
agent.execute_with_tools(input, tools).await?;
```

---

## Quick Fix

Add tool registration in orchestrator service initialization:

```rust
// crates/radium-orchestrator/src/orchestration/service.rs
use crate::orchestration::{
    file_tools,
    project_scan_tool,        // ← ADD
    git_extended_tools,       // ← ADD
    code_analysis_tool,       // ← ADD
    terminal_tool,
};

impl OrchestrationService {
    pub fn new(workspace_root: PathBuf) -> Self {
        let workspace_root_arc = Arc::new(SimpleWorkspaceRoot { root: workspace_root.clone() });

        let mut tool_registry = ToolRegistry::new();

        // File tools
        for tool in file_tools::create_file_tools(workspace_root_arc.clone()) {
            tool_registry.register_tool(tool);
        }

        // PROJECT ANALYSIS TOOLS  ← ADD THIS!
        for tool in project_scan_tool::create_project_analysis_tools(workspace_root_arc.clone()) {
            tool_registry.register_tool(tool);
        }

        // GIT EXTENDED TOOLS  ← ADD THIS!
        for tool in git_extended_tools::create_git_extended_tools(workspace_root_arc.clone()) {
            tool_registry.register_tool(tool);
        }

        // CODE ANALYSIS TOOL  ← ADD THIS!
        tool_registry.register_tool(
            code_analysis_tool::create_code_analysis_tool(workspace_root_arc.clone())
        );

        // Terminal tool
        tool_registry.register_tool(terminal_tool::create_terminal_tool(workspace_root_arc));

        Self { tool_registry, workspace_root }
    }
}
```

---

## Testing After Fix

```bash
# Test CLI with tools registered
radium-cli chat chat-gemini

> Scan this project

# Expected output:
# 🔧 Tool Call: project_scan
#    Arguments: { "depth": "quick" }
#
# 📊 Tool Result:
#    # Project Scan Results
#    ## README
#    ...
#
# Response:
# "This is Radium, a Rust-based AI orchestration system..."
```

---

## Impact on Parity Analysis

### Previous Analysis (INCOMPLETE)

"CLI already has LLM-driven tool selection via orchestrator" ← **TRUE BUT IRRELEVANT**

**The architecture is correct, but NO TOOLS ARE REGISTERED!**

### Corrected Analysis

| Feature | TUI | CLI | Status |
|---------|-----|-----|--------|
| LLM-driven selection | ✅ | ✅ | Architecture correct |
| Tools available | ✅ | ❌ | CLI has ZERO tools! |
| **Can scan project** | ✅ | ❌ | **BROKEN** |

**Parity Score**: Was ~60%, actually **0%** for tool usage

---

## Action Items

### Immediate (BLOCKER)

- [ ] **Add tool registration in orchestrator service** (Quick fix above)
- [ ] **Rebuild CLI**: `cargo build --release -p radium-cli`
- [ ] **Test**: Verify tools appear in agent execution
- [ ] **Validate**: "Scan my project" should trigger project_scan

### Short Term

- [ ] **Document tool registration**: Add comments explaining which tools are available
- [ ] **Add tool discovery command**: `radium-cli tools list`
- [ ] **Update parity analysis**: CLI was broken, not just different

### Long Term

- [ ] **Unify tool registration**: Single source of truth for available tools
- [ ] **Tool configuration**: Let agents specify which tools they need
- [ ] **Dynamic tool loading**: Load tools based on agent requirements

---

## Lessons Learned

1. **Test End-to-End**: Architectural analysis isn't enough - need live tests
2. **Verify Assumptions**: "CLI uses orchestrator" ≠ "CLI has tools"
3. **Check Integration**: Modules may exist but not be wired up
4. **Tool Availability**: LLM-driven selection is useless without tools!

---

## Related Files

- `crates/radium-orchestrator/src/orchestration/service.rs` - Where to fix
- `crates/radium-orchestrator/src/orchestration/project_scan_tool.rs` - Tool exists
- `crates/radium-orchestrator/src/orchestration/git_extended_tools.rs` - Tool exists
- `crates/radium-orchestrator/src/orchestration/code_analysis_tool.rs` - Tool exists
- `apps/cli/src/commands/step.rs` - Calls orchestrator
- `apps/tui/src/chat_executor.rs` - Working example with hardcoded tools

---

**Status**: 🚨 BLOCKER
**Next Step**: Implement quick fix and test
**Priority**: P0 - CLI is currently non-functional for tool-based tasks

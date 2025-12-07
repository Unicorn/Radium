---
req_id: REQ-010
title: Policy Engine
phase: NEXT
status: Completed
priority: High
estimated_effort: 6-7 hours
dependencies: [REQ-004]
related_docs:
  - docs/features/gemini-cli-enhancements.md#policy-engine-for-tool-execution
  - docs/project/03-implementation-plan.md#step-3-workflow-behaviors
---

# Policy Engine

## Problem Statement

Users need fine-grained control over tool execution to ensure security and prevent unwanted operations. Without a policy engine, users cannot:
- Control which tools agents can execute
- Set approval requirements for sensitive operations
- Configure workspace-specific security policies
- Enforce enterprise security requirements
- Prevent accidental destructive operations

The legacy system and modern AI tools (like gemini-cli) provide policy engines for tool execution control. Radium needs an equivalent system that supports rule-based policies with priority tiers and approval modes.

## Solution Overview

Implement a comprehensive policy engine that provides:
- TOML-based policy rule system
- Tool execution control (allow/deny/ask_user)
- Priority-based rule matching with tiered policies (Admin/User/Default)
- Approval modes (yolo, autoEdit, ask)
- Pattern matching for tool names and arguments
- Special syntax for shell commands and MCP tools
- Integration with workflow execution for policy enforcement

The policy engine enables enhanced security through controlled tool execution, with flexible policy configuration per workspace and enterprise-ready admin policy support.

## Functional Requirements

### FR-1: Policy Rule System

**Description**: TOML-based policy rules with priority tiers.

**Acceptance Criteria**:
- [x] TOML policy file parsing
- [x] Rule definition structure (name, priority, action, pattern)
- [x] Priority tiers (Admin, User, Default)
- [x] Rule action types (allow, deny, ask_user)
- [x] Pattern matching for tool names and arguments
- [x] Rule evaluation with priority system

**Implementation**: `crates/radium-core/src/policy/mod.rs`

### FR-2: Tool Execution Control

**Description**: Control tool execution based on policy rules.

**Acceptance Criteria**:
- [x] Tool execution interception
- [x] Rule matching for tool names
- [x] Rule matching for tool arguments
- [x] Special syntax for shell commands
- [x] Special syntax for MCP tools
- [x] Policy enforcement in workflow execution

**Implementation**: `crates/radium-core/src/policy/enforcement.rs`

### FR-3: Approval Modes

**Description**: Different approval modes for tool execution.

**Acceptance Criteria**:
- [x] Approval mode configuration (yolo, autoEdit, ask)
- [x] Yolo mode (auto-approve all)
- [x] AutoEdit mode (auto-approve with logging)
- [x] Ask mode (require user approval)
- [x] Approval workflow integration

**Implementation**: `crates/radium-core/src/policy/approval.rs`

### FR-4: Session Constitution System

**Description**: Per-session rules and constraints.

**Acceptance Criteria**:
- [x] ConstitutionManager for session-scoped rules
- [x] TTL-based cleanup (1 hour) for stale sessions
- [x] Max 50 rules per session limit
- [x] Constitution tools (update_constitution, reset_constitution, get_constitution)
- [x] Integration with workflow execution context

**Implementation**: `crates/radium-core/src/policy/constitution.rs`

## Technical Requirements

### TR-1: Policy Rule Format

**Description**: TOML format for policy rules.

**TOML Format**:
```toml
[[rules]]
name = "Allow file operations"
priority = "user"  # admin, user, default
action = "allow"   # allow, deny, ask_user
pattern = "file_*"
approval_mode = "autoEdit"  # optional: yolo, autoEdit, ask

[[rules]]
name = "Deny dangerous commands"
priority = "admin"
action = "deny"
pattern = "rm -rf *"
```

**Data Models**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub name: String,
    pub priority: RulePriority,  // Admin, User, Default
    pub action: RuleAction,        // Allow, Deny, AskUser
    pub pattern: String,
    pub approval_mode: Option<ApprovalMode>,  // yolo, autoEdit, ask
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulePriority {
    Admin,
    User,
    Default,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleAction {
    Allow,
    Deny,
    AskUser,
}
```

### TR-2: Policy Engine API

**Description**: APIs for policy evaluation and enforcement.

**APIs**:
```rust
pub struct PolicyEngine {
    rules: Vec<PolicyRule>,
}

impl PolicyEngine {
    pub fn load_from_file(path: &Path) -> Result<Self>;
    pub fn evaluate(&self, tool_name: &str, args: &Value) -> PolicyDecision;
    pub fn should_allow(&self, tool_name: &str, args: &Value) -> bool;
}
```

### TR-3: Pattern Matching

**Description**: Pattern matching for tool names and arguments.

**Pattern Syntax**:
- `file_*` - Matches tools starting with "file_"
- `rm -rf *` - Matches shell commands
- `mcp:server:tool` - Matches MCP tools
- Glob patterns supported

## User Experience

### UX-1: Policy Configuration

**Description**: Users configure policies in TOML files.

**Example**:
```toml
# .radium/policy.toml
[[rules]]
name = "Allow safe file operations"
priority = "user"
action = "allow"
pattern = "read_file|write_file"

[[rules]]
name = "Require approval for destructive operations"
priority = "admin"
action = "ask_user"
pattern = "rm *|delete_*"
```

### UX-2: Policy Enforcement

**Description**: Policies are automatically enforced during tool execution.

**Example**:
```bash
Agent attempts to execute: rm -rf /tmp/test
Policy: Deny dangerous commands (admin priority)
Result: Tool execution blocked
```

## Data Requirements

### DR-1: Policy Configuration Files

**Description**: TOML files containing policy rules.

**Location**: `.radium/policy.toml` or user/project-specific locations

**Schema**: See TR-1 Policy Rule Format

## Dependencies

- **REQ-004**: Workflow Behaviors - Required for workflow execution and tool interception

## Success Criteria

1. [x] Policy rules can be defined in TOML format
2. [x] Tool execution is controlled based on rules
3. [x] Priority-based rule matching works correctly
4. [x] Approval modes function as expected
5. [x] Pattern matching works for tool names and arguments
6. [x] Session constitution system enforces per-session rules
7. [x] All policy operations have comprehensive test coverage (21+ tests)

**Completion Metrics**:
- **Status**: ✅ Complete
- **Test Coverage**: 21+ passing tests
- **Lines of Code**: ~450 lines
- **Implementation**: Full policy engine with constitution system
- **Files**: 
  - `crates/radium-core/src/policy/mod.rs`
  - `crates/radium-core/src/policy/constitution.rs`

## Out of Scope

- Advanced policy analytics (future enhancement)
- Policy versioning (future enhancement)
- Policy templates (future enhancement)

## References

- [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#policy-engine-for-tool-execution)
- [Implementation Plan](../project/03-implementation-plan.md#step-3-workflow-behaviors)
- [Policy Engine Implementation](../../crates/radium-core/src/policy/)


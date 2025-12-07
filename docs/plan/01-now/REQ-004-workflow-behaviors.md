---
req_id: REQ-004
title: Workflow Behaviors
phase: NOW
status: Completed
priority: High
estimated_effort: 21-26 hours
dependencies: [REQ-001, REQ-002, REQ-003]
related_docs:
  - docs/project/02-now-next-later.md#step-3-workflow-behaviors
  - docs/project/03-implementation-plan.md#step-3-workflow-behaviors
  - docs/legacy/legacy-system-feature-backlog.md#22-workflow-behaviors
---

# Workflow Behaviors

## Problem Statement

Workflows need dynamic execution control to handle complex scenarios where agents need to:
- Repeat previous steps when results are unsatisfactory (loop behavior)
- Dynamically trigger other agents based on execution context (trigger behavior)
- Pause execution for manual intervention or review (checkpoint behavior)
- Request metacognitive oversight to prevent reasoning lock-in (vibe check behavior)
- Control tool execution based on rules and policies (policy engine)
- Apply session-specific rules and constraints (constitution system)

Without workflow behaviors, agents cannot adapt to changing conditions, handle errors gracefully, or coordinate with other agents dynamically. The legacy system provided behavior.json files that agents could write to control workflow execution.

## Solution Overview

Implement a comprehensive workflow behavior system that provides:
- Loop behavior for repeating previous steps with max iterations
- Trigger behavior for dynamically inserting agent execution
- Checkpoint behavior for pausing workflow execution
- VibeCheck behavior for metacognitive oversight requests
- Policy Engine for fine-grained tool execution control
- Session Constitution System for per-session rules
- Behavior.json control file support
- Workflow template system for reusable workflows

The workflow behavior system enables dynamic, adaptive workflows that can respond to execution context and coordinate multiple agents effectively.

## Functional Requirements

### FR-1: Loop Behavior

**Description**: Allow agents to request repeating previous steps with configurable limits.

**Acceptance Criteria**:
- [x] Loop behavior configuration (max_iterations, steps_back, skip_list)
- [x] Behavior.json file support for loop requests
- [x] Step back functionality with configurable number of steps
- [x] Maximum iteration limit enforcement
- [x] Skip list for steps to exclude from loop
- [x] Loop counter tracking
- [x] Loop decision evaluation

**Implementation**: `crates/radium-core/src/workflow/behaviors/loop_behavior.rs`

### FR-2: Trigger Behavior

**Description**: Allow agents to dynamically trigger other agents during workflow execution.

**Acceptance Criteria**:
- [x] Trigger behavior configuration (trigger_agent_id)
- [x] Behavior.json file support for trigger requests
- [x] Dynamic agent insertion into workflow
- [x] Agent discovery and validation
- [x] Trigger decision evaluation
- [x] Workflow state management during triggers

**Implementation**: `crates/radium-core/src/workflow/behaviors/trigger.rs`

### FR-3: Checkpoint Behavior

**Description**: Allow agents to pause workflow execution for manual intervention.

**Acceptance Criteria**:
- [x] Checkpoint behavior evaluation
- [x] Behavior.json file support for checkpoint requests
- [x] Workflow state persistence at checkpoints
- [x] Resume from checkpoint functionality
- [x] Checkpoint decision evaluation
- [x] Checkpoint state management

**Implementation**: `crates/radium-core/src/workflow/behaviors/checkpoint.rs`

### FR-4: VibeCheck Behavior

**Description**: Allow agents to request metacognitive oversight to prevent reasoning lock-in.

**Acceptance Criteria**:
- [x] VibeCheck behavior evaluation
- [x] Behavior.json file support for vibe check requests
- [x] Phase-aware interrupt integration (planning/implementation/review)
- [x] Risk score calculation
- [x] VibeCheck decision evaluation
- [x] Integration with oversight service

**Implementation**: `crates/radium-core/src/workflow/behaviors/vibe_check.rs`

### FR-5: Policy Engine

**Description**: Fine-grained tool execution control based on TOML rules.

**Acceptance Criteria**:
- [x] TOML-based policy rule system
- [x] Tool execution control (allow/deny/ask_user)
- [x] Priority-based rule matching (Admin/User/Default)
- [x] Approval modes (yolo, autoEdit, ask)
- [x] Pattern matching for tool names and arguments
- [x] Special syntax for shell commands and MCP tools
- [x] Rule evaluation and enforcement

**Implementation**: `crates/radium-core/src/policy/mod.rs`

### FR-6: Session Constitution System

**Description**: Per-session rules and constraints for workflow execution.

**Acceptance Criteria**:
- [x] ConstitutionManager for session-scoped rules
- [x] TTL-based cleanup for stale sessions
- [x] Constitution tools (update_constitution, reset_constitution, get_constitution)
- [x] Integration with workflow execution context
- [x] Per-session rule limits (max 50 rules)
- [x] Automatic cleanup (1 hour TTL)

**Implementation**: `crates/radium-core/src/policy/constitution.rs`

### FR-7: Behavior.json Control File

**Description**: JSON file that agents write to control workflow execution.

**Acceptance Criteria**:
- [x] Behavior.json file format and parsing
- [x] Support for all behavior action types (loop, trigger, checkpoint, vibecheck, continue, stop)
- [x] File location: `radium/.radium/memory/behavior.json`
- [x] Behavior action reading and validation
- [x] Error handling for invalid behavior files

**Implementation**: `crates/radium-core/src/workflow/behaviors/types.rs`

### FR-8: Workflow Template System

**Description**: Reusable workflow templates with module behaviors.

**Acceptance Criteria**:
- [x] Workflow template discovery
- [x] Template structure (steps, modules, behaviors)
- [x] Module behavior configuration
- [x] Template loading and validation
- [x] Template execution support

**Implementation**: 
- `crates/radium-core/src/workflow/templates.rs`
- `crates/radium-core/src/workflow/template_discovery.rs`

## Technical Requirements

### TR-1: Behavior Action Format

**Description**: JSON format for behavior.json control file.

**JSON Format**:
```json
{
  "action": "loop" | "trigger" | "checkpoint" | "continue" | "stop" | "vibecheck",
  "reason": "Why this action was chosen",
  "triggerAgentId": "agent-to-trigger"  // Required for trigger action
}
```

**Data Models**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BehaviorActionType {
    Loop,
    Trigger,
    Checkpoint,
    Continue,
    Stop,
    VibeCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorAction {
    pub action: BehaviorActionType,
    pub reason: Option<String>,
    pub trigger_agent_id: Option<String>,
}
```

**Location**: `radium/.radium/memory/behavior.json`

### TR-2: Loop Behavior Configuration

**Description**: Configuration for loop behavior.

**Data Models**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopBehaviorConfig {
    pub max_iterations: Option<u32>,
    pub steps: u32,  // Number of steps to go back
    pub skip: Vec<String>,  // Steps to skip
}

#[derive(Debug, Clone)]
pub struct LoopDecision {
    pub should_repeat: bool,
    pub steps_back: u32,
    pub skip_list: Vec<String>,
    pub reason: Option<String>,
}
```

### TR-3: Trigger Behavior Configuration

**Description**: Configuration for trigger behavior.

**Data Models**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerBehaviorConfig {
    pub trigger_agent_id: String,
}

#[derive(Debug, Clone)]
pub struct TriggerDecision {
    pub should_trigger: bool,
    pub agent_id: String,
    pub reason: Option<String>,
}
```

### TR-4: Policy Engine Rules

**Description**: TOML-based policy rules for tool execution control.

**TOML Format**:
```toml
[[rules]]
name = "Allow file operations"
priority = "user"  # admin, user, default
action = "allow"   # allow, deny, ask_user
pattern = "file_*"

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
    pub action: RuleAction,       // Allow, Deny, AskUser
    pub pattern: String,
    pub approval_mode: Option<ApprovalMode>,  // yolo, autoEdit, ask
}
```

### TR-5: Behavior Evaluator Trait

**Description**: Trait for evaluating workflow behaviors.

**APIs**:
```rust
pub trait BehaviorEvaluator {
    type Context;
    type Decision;

    fn evaluate(
        &self,
        behavior_file: &Path,
        output: &str,
        context: &Self::Context,
    ) -> Result<Option<Self::Decision>, BehaviorError>;
}
```

## User Experience

### UX-1: Loop Behavior

**Description**: Agents request looping back to previous steps.

**Example**:
```json
// behavior.json
{
  "action": "loop",
  "reason": "Tests are failing, need to fix implementation"
}
```

### UX-2: Trigger Behavior

**Description**: Agents dynamically trigger other agents.

**Example**:
```json
// behavior.json
{
  "action": "trigger",
  "triggerAgentId": "review-agent",
  "reason": "Need code review before proceeding"
}
```

### UX-3: Checkpoint Behavior

**Description**: Agents pause workflow for manual intervention.

**Example**:
```json
// behavior.json
{
  "action": "checkpoint",
  "reason": "Need user approval for database migration"
}
```

### UX-4: VibeCheck Behavior

**Description**: Agents request metacognitive oversight.

**Example**:
```json
// behavior.json
{
  "action": "vibecheck",
  "reason": "Uncertain about approach, need oversight"
}
```

### UX-5: Policy Engine

**Description**: Users configure tool execution policies.

**Example**:
```toml
# .radium/policy.toml
[[rules]]
name = "Allow safe file operations"
priority = "user"
action = "allow"
pattern = "read_file|write_file"
```

## Data Requirements

### DR-1: Behavior.json File

**Description**: JSON file written by agents to control workflow.

**Location**: `radium/.radium/memory/behavior.json`

**Schema**: See TR-1 Behavior Action Format

### DR-2: Policy Rules File

**Description**: TOML file containing policy rules.

**Location**: `.radium/policy.toml` or user/project-specific locations

**Schema**: See TR-4 Policy Engine Rules

### DR-3: Workflow Templates

**Description**: JSON files containing workflow templates.

**Location**: `templates/workflows/*.json`

**Format**: Workflow template JSON structure

## Dependencies

- **REQ-001**: Workspace System - Required for workspace structure and behavior.json location
- **REQ-002**: Agent Configuration - Required for agent discovery and execution
- **REQ-003**: Core CLI Commands - Required for workflow execution commands

## Success Criteria

1. [x] Loop behavior steps back correctly with max iterations
2. [x] Trigger behavior executes agents dynamically
3. [x] Checkpoint behavior saves and resumes state
4. [x] VibeCheck behavior triggers oversight at checkpoints
5. [x] Policy engine controls tool execution based on rules
6. [x] Session constitution system enforces per-session rules
7. [x] Behavior.json control file works for all behavior types
8. [x] Workflow template system functional
9. [x] All behaviors have comprehensive test coverage (50+ tests)

**Completion Metrics**:
- **Status**: ✅ Complete
- **Test Coverage**: 50+ passing tests
- **Lines of Code**: ~1,400 lines for behaviors + ~450 lines for policy engine
- **Implementation**: Full workflow behavior system implemented
- **Files**: 
  - `crates/radium-core/src/workflow/behaviors/` (loop, trigger, checkpoint, vibe_check, types)
  - `crates/radium-core/src/policy/` (policy engine, constitution)
  - `crates/radium-core/src/workflow/templates.rs`
  - `crates/radium-core/src/workflow/template_discovery.rs`

## Out of Scope

- Advanced workflow orchestration (covered in plan execution)
- Agent execution engine (covered in orchestrator)
- Memory and context system (covered in REQ-006)
- Monitoring and telemetry (covered in REQ-007)

## References

- [Now/Next/Later Roadmap](../project/02-now-next-later.md#step-3-workflow-behaviors)
- [Implementation Plan](../project/03-implementation-plan.md#step-3-workflow-behaviors)
- [Feature Backlog](../legacy/legacy-system-feature-backlog.md#22-workflow-behaviors)
- [Workflow Behaviors Implementation](../../crates/radium-core/src/workflow/behaviors/)
- [Policy Engine Implementation](../../crates/radium-core/src/policy/)


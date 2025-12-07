# Implementation Plan: 0-10 Steps to Feature Parity

> **Goal**: Achieve complete legacy system feature parity in 10 steps  
> **Reference**: See [legacy-system-feature-backlog.md](./legacy-system-feature-backlog.md) for complete feature catalog  
> **Last Updated**: 2025-01-XX (includes vibe-check integration and gap analysis)
> 
> **📋 For current REQ status and tasks, query Braingrid:**  
> `braingrid requirement list -p PROJ-14`  
> See [BRAINGRID_WORKFLOW.md](./BRAINGRID_WORKFLOW.md) for details.

## Overview

This plan breaks down legacy system feature parity into 10 actionable steps, integrating the comprehensive feature backlog into a structured implementation roadmap.

**Total Estimated Time**: 232-298 hours (5-8 weeks)

---

## Step 0: Workspace System

**Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "workspace"`

**Priority**: 🔴 Critical  
**Est. Time**: 10-14 hours  
**Dependencies**: None

### Objectives

Establish the workspace structure that all legacy system features depend on.

### Tasks

#### 0.1: Workspace Module Structure (3-4h)
**File**: `crates/radium-core/src/workspace/mod.rs`

- Create workspace module with stage directories:
  - `radium/backlog/` - Generated plans
  - `radium/development/` - Active development
  - `radium/review/` - Review stage
  - `radium/testing/` - Testing stage
  - `radium/docs/` - Documentation
- Internal `.radium/` directory structure:
  - `agents/`, `artifacts/`, `memory/`, `logs/`, `prompts/`, `inputs/`
- Workspace initialization and validation

**Reference**: [Feature Backlog Section 5.1](./legacy-system-feature-backlog.md#51-workspace-structure)

#### 0.2: Requirement ID System (2-3h)
**File**: `crates/radium-core/src/workspace/requirement_id.rs`

- RequirementId type (REQ-XXX format for local plans)
- Auto-incrementing from counter file (`.radium/requirement-counter.json`)
- Validation and parsing
- Thread-safe counter management

**Reference**: [Feature Backlog Section 5.2](./legacy-system-feature-backlog.md#52-plan-management)

#### 0.3: Plan Structure Types (2-3h)
**File**: `crates/radium-core/src/models/plan.rs`

- Plan struct (metadata, iterations, tasks)
- PlanManifest struct
- PlanStatus enum
- Iteration and Task structs
- Serde serialization for plan.json and manifest

**Reference**: [Feature Backlog Section 5.3](./legacy-system-feature-backlog.md#53-plan-generation)

#### 0.4: Plan Discovery (3-4h)
**File**: `crates/radium-core/src/workspace/plan_discovery.rs`

- Scan all workspace stages for plans
- Find plan by requirement ID or folder name
- List all plans with metadata
- Calculate progress percentages
- Sort by date or ID

**Reference**: [Feature Backlog Section 5.2](./legacy-system-feature-backlog.md#52-plan-management)

### Deliverables

- ✅ Workspace manager crate
- ✅ Plan discovery and listing working
- ✅ Tests for workspace operations

### Success Criteria

- Can create complete workspace structure
- Can generate requirement IDs (REQ-001, REQ-002, etc. for local plans)
- Can discover plans in all stages
- Can calculate plan progress

---

## Step 1: Agent Configuration System

**Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "agent\|mcp\|context"`

**Priority**: 🔴 Critical  
**Est. Time**: 15-18 hours  
**Dependencies**: Step 0

### Objectives

Implement agent configuration and prompt system matching legacy system's structure, with MCP integration and context files support.

### Tasks

#### 1.1: Agent Configuration Format (2-3h)
**File**: `crates/radium-core/src/config/agents.rs`

- AgentConfig struct (TOML format)
- Agent definition structure:
  - `id`, `name`, `description`
  - `prompt_path` or `mirror_path`
  - `engine`, `model`, `reasoning_effort` (optional)
- TOML parsing and validation

**Reference**: [Feature Backlog Section 9.1](./legacy-system-feature-backlog.md#91-agent-configuration-configmainagentsjs)

#### 1.2: Agent Discovery (3-4h)
**File**: `crates/radium-core/src/agents/discovery.rs`

- Scan agent directories for `.toml` files
- Load and parse agent configs
- Build agent registry
- Resolve prompt file paths
- Filter by sub-agent IDs (for templates)

**Reference**: [Feature Backlog Section 3.4](./legacy-system-feature-backlog.md#34-agent-configuration)

#### 1.3: Prompt Template System (4-5h)
**Files**: `crates/radium-core/src/prompts/mod.rs`, `templates.rs`, `processing.rs`

- Load prompt template files (.md)
- Basic placeholder replacement (`{{VAR}}`)
- Prompt validation
- Prompt caching
- File content injection

**Reference**: [Feature Backlog Section 11](./legacy-system-feature-backlog.md#11-prompt-system)

#### 1.4: MCP Integration (4-5h)
**File**: `crates/radium-core/src/mcp/mod.rs`

- MCP client implementation
- Tool discovery from MCP servers
- Multiple transport support (stdio, SSE, HTTP)
- OAuth authentication for remote servers
- Tool conflict resolution with automatic prefixing
- Rich content support (text, images, audio) in tool responses

**Reference**: [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#mcp-model-context-protocol-integration)

#### 1.5: Context Files System (3-4h)
**File**: `crates/radium-core/src/context/files.rs`

- Hierarchical GEMINI.md loading (global → project → subdirectory)
- Context file discovery and scanning
- Context import syntax (`@file.md`)
- Custom context file name configuration
- Context file precedence and merging

**Reference**: [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#context-files-geminimd)

### Deliverables

- ✅ Agent configuration system
- ✅ Prompt template system
- ✅ Agent discovery working
- ✅ Tests for agent loading

### Success Criteria

- Can parse agent TOML configs
- Can discover agents from directories
- Can load prompt templates
- Can replace placeholders
- Can discover and use tools from MCP servers
- Can load context files hierarchically

---

### Step 2: Core CLI Commands

**Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "cli\|command"`

**Status**: ✅ Complete (Implementation) | ❌ 0% Test Coverage  
**Priority**: 🔴 Critical  
**Est. Time**: 8-10 hours (Completed) | **Test Est.**: 15-20 hours

- [x] `rad init` - Intelligent workspace initialization
- [x] `rad status` - Show workspace and engine status
- [x] `rad clean` - Clean workspace artifacts
- [x] `rad plan` - Generate plans from specifications
- [x] `rad craft` - Execute plans
- [x] `rad agents` - Agent management (list, search, info, validate)
- [x] `rad templates` - Template management (list, info, validate)
- [x] `rad auth` - Authentication management
- [x] `rad step` - Single agent execution
- [x] `rad run` - Agent script execution
- [x] CLI structure matching legacy system
- [ ] **TESTING**: Integration tests for all CLI commands (0% coverage - CRITICAL GAP)

**Why Now**: Primary user interface. Must match Radium's `rad` command structure.

**Test Coverage Gap**: ~1,200 lines of CLI command code have 0% test coverage. This is a critical gap that must be addressed.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-2-core-cli-commands) for detailed tasks.  
**Test Requirements**: See [TEST_COVERAGE_REPORT.md](./TEST_COVERAGE_REPORT.md#step-2-core-cli-commands) for detailed test requirements.

### Tasks

#### 2.0: `rad init` Command (2-3h)
**File**: `radium/apps/cli/src/commands/init.rs`

- Interactive initialization wizard
- Detect Git/VCS root and warn if initializing in subdirectory
- Prompt for workspace location (default: `.radium` in CWD)
- Create workspace structure (`.radium/`, `backlog/`, etc.)
- Generate default configuration

**Reference**: [Feature Backlog Section 1.1](./legacy-system-feature-backlog.md#11-core-commands)

#### 2.1: CLI Structure Refactor (30m)
**File**: `radium/apps/cli/src/main.rs`

- Update CLI to match legacy system:
  - `rad` as main command
  - Subcommands: `plan`, `craft`, `run`, `step`, `status`, `clean`, `auth`, `agents`
- Proper help text

**Reference**: [Feature Backlog Section 1.1](./legacy-system-feature-backlog.md#11-core-commands)

#### 2.2: `rad status` Command (1-2h)
**File**: `radium/apps/cli/src/commands/status.rs`

- Show workspace status
- Show available engines/models
- Show authentication status (stub for now)
- JSON output option

**Reference**: [Feature Backlog Section 1.1 - rad status](./legacy-system-feature-backlog.md#rad-status)

#### 2.3: `rad clean` Command (1-2h)
**File**: `radium/apps/cli/src/commands/clean.rs`

- Clean `.radium/artifacts/`
- Clean `.radium/memory/`
- Clean `.radium/logs/`
- Clean `.radium/prompts/`
- Clean `.radium/inputs/`
- Verbose mode

**Reference**: [Feature Backlog Section 1.1 - rad clean](./legacy-system-feature-backlog.md#rad-clean)

#### 2.4: Stub Remaining Commands (30m)
**Files**: `radium/apps/cli/src/commands/{plan,craft,step,run}.rs`

- Create stub implementations
- Print "Coming soon" messages
- Show planned usage

**Reference**: [Feature Backlog Section 1.1](./legacy-system-feature-backlog.md#11-core-commands)

#### 2.5: CLI Diagnostics Command (3-4h) ✅
**Files**: 
- `apps/cli/src/commands/doctor.rs` (new)

- ✅ Added `rad doctor` command for environment validation
- ✅ Check workspace structure and validity
- ✅ Validate environment files (.env detection in CWD and home)
- ✅ Check port availability (for future HTTP server)
- ✅ Validate workspace directory structure
- ✅ Provide actionable error messages
- ✅ JSON output support

**Reference**: Vibe-check-mcp-server `src/cli/doctor.ts`

### Deliverables

- ✅ `rad status` command working
- ✅ `rad clean` command working
- ✅ `rad doctor` command working
- ✅ All commands registered and implemented
- ❌ **Tests for CLI commands (0% coverage - CRITICAL GAP)**
  - [ ] Integration tests for all commands using `assert_cmd`
  - [ ] Test all command variants (JSON, verbose, interactive)
  - [ ] Test error handling and edge cases
  - [ ] Test command argument parsing

### Success Criteria

- CLI structure matches legacy system
- `rad status` shows workspace and engine status
- `rad clean` removes artifacts safely
- `rad doctor` validates environment and provides diagnostics
- All commands registered without panics
- **TESTING**: All CLI commands have integration tests with >90% coverage

**Test Coverage Status**: 0% (Critical)  
**Test Requirements**: See [TEST_COVERAGE_REPORT.md](./TEST_COVERAGE_REPORT.md#step-2-core-cli-commands)

---

## Step 3: Workflow Behaviors

**Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "workflow\|policy"`

**Status**: ✅ Complete (Implementation) | ⚠️ ~70% Test Coverage  
**Priority**: 🟡 High  
**Est. Time**: 18-22 hours (Completed) | **Test Est.**: 8-12 hours  
**Dependencies**: Step 1, Step 2

### Objectives

Implement workflow behaviors (loop, trigger, checkpoint) matching legacy system's workflow system, with policy engine for tool execution control.

### Tasks

#### 3.1: Refactor Workflow Engine (4-5h)
**File**: `crates/radium-core/src/workflow/engine.rs`

- Update workflow engine for legacy system semantics
- Step status tracking and persistence
- Resume from checkpoint functionality
- Step skipping based on completion status

**Reference**: [Feature Backlog Section 2.1](./legacy-system-feature-backlog.md#21-workflow-execution)

#### 3.2: Loop Behavior (3-4h)
**File**: `crates/radium-core/src/workflow/behaviors/loop.rs`

- Step back to previous steps
- Configurable loop steps (loopSteps)
- Maximum iterations (loopMaxIterations)
- Skip list for steps to exclude
- Behavior file (behavior.json) support

**Reference**: [Feature Backlog Section 2.2 - Loop Behavior](./legacy-system-feature-backlog.md#loop-behavior)

#### 3.3: Trigger Behavior (3-4h)
**File**: `crates/radium-core/src/workflow/behaviors/trigger.rs`

- Dynamically trigger other agents
- Main agent call triggers
- Configurable trigger agent ID
- Behavior file override support

**Reference**: [Feature Backlog Section 2.2 - Trigger Behavior](./legacy-system-feature-backlog.md#trigger-behavior)

#### 3.4: Checkpoint Behavior (2-3h)
**File**: `crates/radium-core/src/workflow/behaviors/checkpoint.rs`

- Save and resume workflow state
- Automatic checkpoint creation
- Resume from last checkpoint
- State persistence

**Reference**: [Feature Backlog Section 2.2 - Checkpoint Behavior](./legacy-system-feature-backlog.md#checkpoint-behavior)

#### 3.5: Policy Engine (6-7h)
**File**: `crates/radium-core/src/policy/mod.rs`

- TOML-based policy rule system
- Tool execution control (allow/deny/ask_user)
- Priority-based rule matching with tiered policies (Default/User/Admin)
- Approval modes (yolo, autoEdit)
- Pattern matching for tool names and arguments
- Special syntax for shell commands and MCP tools

**Reference**: [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#policy-engine-for-tool-execution)

#### 3.6: Phase-Aware Interrupt Integration (3-4h) ✅
**Files**: 
- `crates/radium-core/src/workflow/behaviors/vibe_check.rs` (extended)
- `crates/radium-core/src/workflow/engine.rs` (extended)

- ✅ Detect current workflow phase (planning/implementation/review)
- ✅ Customize vibe_check prompts based on phase
- ✅ Integrate with existing `PlanStatus` and `Iteration` tracking
- ✅ Phase-specific oversight strategies

**Reference**: Vibe-check-mcp-server `docs/agent-prompting.md`

### Deliverables

- ✅ Workflow behaviors working
- ✅ Template system functional
- ✅ Resume from checkpoint working
- ✅ VibeCheck behavior integrated
- ✅ Phase-aware oversight support
- ✅ Tests for all behaviors (27 behavior tests + 21 policy tests + vibe_check tests)
- ⚠️ **Workflow service tests (partial - 5 tests added, need more)**
  - [x] Basic workflow service tests (5 tests)
  - [ ] Workflow execution path tests
  - [ ] Error handling tests
  - [ ] Edge case tests

### Success Criteria

- Loop behavior steps back correctly
- Trigger behavior executes agents dynamically
- Checkpoint behavior saves and resumes state
- VibeCheck behavior triggers oversight at checkpoints
- Behavior.json control file works
- **TESTING**: Workflow service has >90% test coverage (currently ~70%)

**Test Coverage Status**: ~70% (Good, but needs improvement)  
**Test Requirements**: See [TEST_COVERAGE_REPORT.md](./TEST_COVERAGE_REPORT.md#step-3-workflow-behaviors)
- Policy engine controls tool execution based on rules
- Approval modes work correctly
- Session constitution rules enforced

---

## Step 4: Plan Generation & Execution

**Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "plan\|generation"`

**Priority**: 🟡 High  
**Est. Time**: 15-20 hours  
**Dependencies**: Step 0, Step 1, Step 3

### Objectives

Implement full `rad plan` and `rad craft` commands for plan generation and execution.

### Tasks

#### 4.1: `rad plan` Implementation (8-10h)
**File**: `radium/apps/cli/src/commands/plan.rs`

- Specification file parsing
- AI-powered plan generation (using plan-agent)
- Iteration structure creation
- Task extraction
- Plan file generation:
  - `01_Plan_Overview_and_Setup.md`
  - `02_Iteration_I*.md`
  - `03_Verification_and_Glossary.md`
  - `plan_manifest.json`
  - `coordinator-prompt.md`
- Interactive mode with question generation
- Tech stack detection

**Reference**: [Feature Backlog Section 1.1 - rad plan](./legacy-system-feature-backlog.md#rad-plan-spec-path)

#### 4.2: `rad craft` Implementation (7-10h)
**File**: `radium/apps/cli/src/commands/craft.rs`

- Plan selection menu
- Plan discovery by requirement ID or folder name
- Iteration-by-iteration execution
- Task-by-task execution
- Resume from checkpoint
- Progress tracking
- JSON output for CI/CD

**Reference**: [Feature Backlog Section 1.1 - rad craft](./legacy-system-feature-backlog.md#rad-craft-plan-identifier)

### Deliverables

- ✅ `rad plan` generates complete plans
- ✅ `rad craft` executes plans
- ✅ Progress tracking working
- ✅ Tests for plan operations

### Success Criteria

- Can generate plans from specifications
- Can execute plans iteration-by-iteration
- Can resume from checkpoints
- Progress tracking accurate

---

## Step 5: Memory & Context System

**Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "memory\|context\|command"`

**Priority**: 🟡 High  
**Est. Time**: 15-18 hours  
**Dependencies**: Step 0, Step 1

### Objectives

Implement plan-scoped memory and context management for agent execution.

### Tasks

#### 5.1: Plan-Scoped Memory Store (3-4h)
**Files**: `crates/radium-core/src/memory/mod.rs`, `store.rs`

- Memory directory per plan: `radium/backlog/<requirement-id>/memory/`
- Memory store interface
- Agent output storage (last 2000 chars)
- Timestamp tracking

**Reference**: [Feature Backlog Section 6.1](./legacy-system-feature-backlog.md#61-plan-scoped-memory)

#### 5.2: File-Based Memory Adapter (2-3h)
**File**: `crates/radium-core/src/memory/adapter.rs`

- File system-based storage
- Directory creation
- File writing and reading
- Content appending

**Reference**: [Feature Backlog Section 6.1](./legacy-system-feature-backlog.md#61-plan-scoped-memory)

#### 5.3: Context Manager (4-5h)
**Files**: `crates/radium-core/src/context/mod.rs`, `manager.rs`, `injection.rs`

- Context gathering (architecture, plan, codebase)
- File input injection syntax: `agent[input:file1.md,file2.md]`
- Tail context support: `agent[tail:50]`
- Context manager agent integration

**Reference**: [Feature Backlog Section 6.2](./legacy-system-feature-backlog.md#62-context-management)

#### 5.4: Custom Commands System (5-6h)
**File**: `crates/radium-core/src/commands/custom.rs`

- TOML-based command definitions
- Command discovery (user vs project precedence)
- Shell command injection (`!{command}`)
- File content injection (`@{file}`)
- Argument handling (`{{args}}`)
- Namespaced commands via directory structure

**Reference**: [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#custom-commands-toml-based)

#### 5.5: History Continuity and Summarization (4-5h) ✅
**Files**: 
- `crates/radium-core/src/context/manager.rs` (extended)
- `crates/radium-core/src/context/history.rs` (new)

- ✅ Added session-based conversation history tracking
- ✅ Implemented history summarization (last 5 interactions)
- ✅ Prevent context window bloat with smart truncation
- ✅ Integrated with existing `ContextManager`
- ✅ Support history retrieval by session ID
- ✅ Automatic cleanup of old interactions (max 10 per session)

**Reference**: Vibe-check-mcp-server `src/utils/state.ts`

### Deliverables

- ✅ Memory system working
- ✅ Context gathering functional
- ✅ Input injection working
- ✅ History continuity functional
- ✅ Tests for memory operations

### Success Criteria

- Can store agent output in plan-scoped memory
- Can inject file contents into prompts
- Can use tail context from previous runs
- Memory persists across agent executions
- Can define and use custom TOML commands
- Shell and file injection syntax works correctly
- Can track conversation history per session
- History summaries prevent context window bloat

---

## Step 6: Monitoring & Telemetry

**Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "monitoring\|telemetry\|checkpoint"`

**Priority**: 🟡 High  
**Est. Time**: 18-22 hours  
**Dependencies**: Step 1

### Objectives

Implement agent monitoring and telemetry tracking matching legacy system's system.

### Tasks

#### 6.1: Monitoring Database Schema (2-3h)
**File**: `crates/radium-core/src/monitoring/schema.rs`

- SQLite schema for agent monitoring
- Agents table (id, status, parent_id, process_id, etc.)
- Telemetry table (tokens, cost, cache stats)
- Parent-child relationships

**Reference**: [Feature Backlog Section 7.1](./legacy-system-feature-backlog.md#71-agent-monitoring)

#### 6.2: Agent Monitoring Service (4-5h)
**File**: `crates/radium-core/src/monitoring/service.rs`

- Agent lifecycle tracking (start, complete, fail)
- Parent-child relationship tracking
- Process ID tracking
- Agent status queries
- Graceful cleanup on termination

**Reference**: [Feature Backlog Section 7.1](./legacy-system-feature-backlog.md#71-agent-monitoring)

#### 6.3: Telemetry Parsing (3-4h)
**File**: `crates/radium-core/src/monitoring/telemetry.rs`

- Token counting (input, output, cached)
- Cost calculation
- Cache statistics (creation, read tokens)
- Engine-specific telemetry parsers
- Telemetry storage in database

**Reference**: [Feature Backlog Section 7.2](./legacy-system-feature-backlog.md#72-telemetry)

#### 6.4: Log File Management (2-3h)
**File**: `crates/radium-core/src/monitoring/logs.rs`

- Agent-specific log files
- Log file path tracking
- Color marker transformation for log files
- Dual-stream logging (UI + file)

**Reference**: [Feature Backlog Section 7.3](./legacy-system-feature-backlog.md#73-logging)

#### 6.5: Checkpointing System (6-7h)
**File**: `crates/radium-core/src/checkpoint/mod.rs`

- Git snapshot creation before file modifications
- Shadow Git repository management
- Conversation history preservation
- `/restore` command implementation
- Tool call re-proposal after restore
- Checkpoint listing and selection

**Reference**: [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#checkpointing-system)

### Deliverables

- ✅ Monitoring database operational
- ✅ Telemetry tracking working
- ✅ Log management functional
- ✅ Tests for monitoring

### Success Criteria

- Can track agent lifecycle
- Can parse telemetry from engine output
- Can query agent status
- Log files created and managed
- Can create checkpoints before file modifications
- Can restore from checkpoints with conversation history

---

## Step 6.5: Sandboxing

**Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "sandbox"`

**Priority**: 🟡 High  
**Est. Time**: 12-15 hours  
**Dependencies**: Step 1

### Objectives

Implement sandboxing support for safe agent execution, especially for shell commands and file operations.

### Tasks

#### 6.5.1: Sandbox Abstraction (3-4h)
**File**: `crates/radium-core/src/sandbox/mod.rs`

- Sandbox trait definition
- Sandbox factory for different types
- Sandbox configuration structure

**Reference**: [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#sandboxing)

#### 6.5.2: Docker/Podman Sandbox (4-5h)
**File**: `crates/radium-core/src/sandbox/docker.rs`

- Container-based sandboxing
- Volume mounting
- Network configuration
- Custom sandbox flags support

**Reference**: [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#sandboxing)

#### 6.5.3: macOS Seatbelt Sandbox (3-4h)
**File**: `crates/radium-core/src/sandbox/seatbelt.rs`

- macOS sandbox-exec integration
- Sandbox profiles (permissive/restrictive)
- Network control (open/closed/proxied)
- Profile configuration

**Reference**: [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#sandboxing)

#### 6.5.4: Sandbox Configuration (2-3h)
**File**: `crates/radium-core/src/sandbox/config.rs`

- Sandbox settings in workspace config
- Profile selection
- Custom sandbox flags
- Environment variable configuration

**Reference**: [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#sandboxing)

### Deliverables

- ✅ Sandbox abstraction working
- ✅ Docker/Podman sandboxing functional
- ✅ macOS Seatbelt sandboxing functional
- ✅ Sandbox configuration system
- ✅ Tests for sandbox operations

### Success Criteria

- Can execute agents in Docker/Podman containers
- Can execute agents with macOS Seatbelt restrictions
- Sandbox profiles work correctly
- Network and file system access properly controlled

---

## Step 6.6: Metacognitive Oversight System

**Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "vibe\|oversight\|metacognitive"`

**Status**: ✅ Complete  
**Priority**: 🟡 High  
**Est. Time**: 20-25 hours (Completed)  
**Dependencies**: Step 3 (Workflow Behaviors), Step 5 (Memory & Context)

### Objectives

Implement Chain-Pattern Interrupt (CPI) system for agent oversight, preventing reasoning lock-in and improving alignment. Research shows +27% success rate and -41% harmful actions.

### Tasks

#### 6.6.1: VibeCheck Behavior Implementation (6-8h) ✅
**Files**: 
- `crates/radium-core/src/workflow/behaviors/vibe_check.rs`
- `crates/radium-core/src/workflow/behaviors/types.rs` (extended)

- ✅ Added `VibeCheck` to `BehaviorActionType` enum
- ✅ Implemented `VibeCheckEvaluator` trait
- ✅ Created `VibeCheckDecision` struct with risk scores and advice
- ✅ Integrated with existing behavior.json system
- ✅ Support for automatic triggers at workflow checkpoints
- ✅ Phase-aware context support (planning/implementation/review)

**Reference**: Vibe-check-mcp-server `src/tools/vibeCheck.ts`

#### 6.6.2: Metacognitive LLM Service (5-6h) ✅
**Files**: 
- `crates/radium-core/src/oversight/mod.rs`
- `crates/radium-core/src/oversight/metacognitive.rs`

- ✅ Created oversight service that uses second LLM for meta-feedback
- ✅ Implemented phase-aware prompts (planning/implementation/review)
- ✅ Support for multiple LLM providers via Model trait
- ✅ Generate structured output: `{ advice, risk_score, traits, uncertainties }`
- ✅ Fallback to basic questions on API failure
- ✅ Risk score estimation from advice content
- ✅ Trait and uncertainty extraction

**Reference**: Vibe-check-mcp-server `src/utils/llm.ts`, `src/tools/vibeCheck.ts`

#### 6.6.3: Session Constitution System (4-5h) ✅
**Files**: 
- `crates/radium-core/src/policy/constitution.rs`
- `crates/radium-core/src/policy/mod.rs` (extended)

- ✅ Extended `PolicyEngine` module with session-scoped rules
- ✅ Implemented `ConstitutionManager` with TTL-based cleanup
- ✅ Added constitution tools: `update_constitution`, `reset_constitution`, `get_constitution`
- ✅ Integration with workflow execution context
- ✅ Support per-session rule limits (max 50 rules)
- ✅ Automatic cleanup of stale sessions (1 hour TTL)

**Reference**: Vibe-check-mcp-server `src/tools/constitution.ts`

#### 6.6.4: Learning from Mistakes System + ACE Skillbook (20-27h) ✅ Complete
**Files**: 
- `crates/radium-core/src/learning/mod.rs` ✅
- `crates/radium-core/src/learning/store.rs` ✅
- `crates/radium-core/src/learning/updates.rs` ✅ (new)
- `crates/radium-core/src/learning/skill_manager.rs` ✅ (new)

**Phase 1: Basic Integration (4-5h)** ✅
- ✅ Exported learning module from `lib.rs`
- ✅ Integrated `LearningStore` with `ContextManager` for gathering learning context
- ✅ Integrated learning context into `MetacognitiveService` oversight prompts
- ✅ Added `gather_learning_context()` method to `ContextManager`

**Phase 2: ACE Skillbook Features (6-8h)** ✅
- ✅ Added `Skill` struct with helpful/harmful/neutral counts
- ✅ Added skill sections: task_guidance, tool_usage, error_handling, code_patterns, communication, general
- ✅ Extended `LearningStore` with skillbook methods:
  - `add_skill()` - Add new strategies
  - `tag_skill()` - Increment helpful/harmful/neutral counts
  - `get_skills_by_section()` - Retrieve skills by category
  - `as_context()` - Format skills for prompt injection
- ✅ Created `UpdateOperation` enum and `UpdateBatch` struct for incremental updates
- ✅ Implemented `apply_update()` method to prevent context collapse

**Phase 3: SkillManager Component (4-5h)** ✅
- ✅ Created `SkillManager` module that generates updates from `OversightResponse`
- ✅ Analyzes helpful/harmful patterns from oversight feedback
- ✅ Generates structured `UpdateBatch` with ADD/UPDATE/TAG/REMOVE operations
- ✅ JSON parsing for skill curation responses

**Phase 4: Enhanced Reflector Integration (2-3h)** ✅
- ✅ Extended `OversightResponse` with `helpful_patterns` and `harmful_patterns` fields
- ✅ Added pattern extraction methods to `MetacognitiveService`
- ✅ Connected reflector output to SkillManager for skill extraction

**Phase 5: Context Injection (2-3h)** ✅
- ✅ Enhanced `ContextManager::build_context()` to inject skillbook strategies
- ✅ Added `gather_skillbook_context()` method
- ✅ Skills formatted with helpful/harmful counts for agent prompts

**Phase 6: Workflow Integration Helper (2-3h)** ✅
- ✅ Created `LearningIntegration` helper for workflow execution
- ✅ `process_task_learning()` method automates the learning loop
- ✅ Integrates MetacognitiveService, SkillManager, and LearningStore
- ✅ Configurable learning (can be enabled/disabled)

**Original Features (Still Supported)**:
- ✅ `LearningStore` with categorized mistake tracking
- ✅ `LearningEntry` with categories: "Complex Solution Bias", "Feature Creep", "Premature Implementation", "Misalignment", "Overtooling"
- ✅ Similarity detection to prevent duplicate entries
- ✅ File-based storage pattern
- ✅ Learning types: mistake, preference, success
- ✅ Category normalization and summaries

**Status**: ✅ Complete - ACE learning integrated with existing mistake tracking system.

**Reference**: 
- Vibe-check-mcp-server `src/tools/vibeLearn.ts`, `src/utils/storage.ts`
- ACE paper (arXiv:2510.04618) and implementation in `old/agentic-context-engine/`

### Deliverables

- ✅ VibeCheck workflow behavior working
- ✅ Metacognitive oversight service operational
- ✅ Session constitution system integrated with policy engine
- ✅ Learning system exported, integrated, and enhanced with ACE skillbook
- ✅ Tests for all oversight components

### Success Criteria

- ✅ Agents can trigger vibe_check at workflow checkpoints
- ✅ Oversight LLM provides phase-aware feedback
- ✅ Session rules enforced during workflow execution
- ✅ Mistakes and skills are logged and fed into oversight prompts
- ✅ Skillbook strategies injected into agent context
- ✅ Risk scores can trigger automatic workflow behaviors
- ✅ Learning loop complete: oversight → patterns → skillbook updates → context injection

**Why Next**: Critical for agent safety and alignment. Research shows +27% success rate and -41% harmful actions. Fits naturally after workflow behaviors and memory systems are complete.

---

## Step 7: Engine Abstraction Layer

**Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "engine\|abstraction"`

**Priority**: 🟢 Medium  
**Est. Time**: 15-20 hours  
**Dependencies**: Step 1

### Objectives

Support multiple AI engines/providers matching legacy system's engine system.

### Tasks

#### 7.1: Engine Abstraction Trait (3-4h)
**Files**: `crates/radium-core/src/engines/mod.rs`, `trait.rs`

- Engine trait definition
- Engine interface abstraction
- Engine metadata (name, description, CLI command)

**Reference**: [Feature Backlog Section 4](./legacy-system-feature-backlog.md#4-engineprovider-system)

#### 7.2: Engine Registry (2-3h)
**File**: `crates/radium-core/src/engines/registry.rs`

- Dynamic engine registration
- Engine metadata storage
- Default engine selection
- Engine lookup by ID

**Reference**: [Feature Backlog Section 4.3](./legacy-system-feature-backlog.md#43-engine-registry)

#### 7.3: CLI Binary Detection (2-3h)
**File**: `crates/radium-core/src/engines/detection.rs`

- Path checking for CLI binaries
- Version command execution
- Timeout handling
- Error detection

**Reference**: [Feature Backlog Section 4.2](./legacy-system-feature-backlog.md#42-engine-features)

#### 7.4: Authentication System (3-4h)
**File**: `crates/radium-core/src/engines/auth.rs`

- Authentication status checking per engine
- Authentication methods per engine
- Auth state persistence
- Multi-provider authentication (CCR, OpenCode)

**Reference**: [Feature Backlog Section 4.2](./legacy-system-feature-backlog.md#42-engine-features)

#### 7.5: Engine Providers (5-6h)
**Files**: `crates/radium-core/src/engines/providers/*.rs`

- Codex provider
- Claude provider
- Cursor provider
- CCR provider
- OpenCode provider
- Auggie provider

**Reference**: [Feature Backlog Section 4.1](./legacy-system-feature-backlog.md#41-supported-engines)

### Deliverables

- ✅ Engine abstraction working
- ✅ Multiple engines supported
- ✅ Authentication functional
- ✅ Tests for engine system

### Success Criteria

- Can detect CLI binaries
- Can check authentication status
- Can execute agents with different engines
- Engine registry functional

---

## Step 8: Enhanced TUI

**Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "tui\|improvement"`

**Priority**: 🟢 Medium  
**Est. Time**: 15-20 hours  
**Dependencies**: Step 3, Step 6

### Objectives

Enhance TUI to match legacy system's workflow dashboard functionality.

### Tasks

#### 8.1: TUI State Management (3-4h)
**File**: `radium/apps/tui/src/state/mod.rs`

- WorkflowUIState class
- Agent state tracking
- Sub-agent state tracking
- Output buffer management
- Telemetry state
- Checkpoint state

**Reference**: [Feature Backlog Section 8.1](./legacy-system-feature-backlog.md#81-tui-framework)

#### 8.2: WorkflowDashboard (4-5h)
**File**: `radium/apps/tui/src/views/workflow_dashboard.rs`

- Main dashboard view
- Overall workflow status
- Agent overview

**Reference**: [Feature Backlog Section 8.2](./legacy-system-feature-backlog.md#82-ui-components)

#### 8.3: AgentTimeline & Status Indicators (3-4h)
**File**: `radium/apps/tui/src/components/agent_timeline.rs`

- Agent execution timeline
- Status indicators
- Progress visualization

**Reference**: [Feature Backlog Section 8.2](./legacy-system-feature-backlog.md#82-ui-components)

#### 8.4: OutputWindow & LogViewer (4-5h)
**Files**: `radium/apps/tui/src/components/output.rs`, `logs.rs`

- Agent output display
- Streaming output
- Log file reading
- Log navigation

**Reference**: [Feature Backlog Section 8.2](./legacy-system-feature-backlog.md#82-ui-components)

#### 8.5: TelemetryBar & StatusFooter (3-4h)
**Files**: `radium/apps/tui/src/components/telemetry.rs`, `status.rs`

- Telemetry display
- Cost information
- Token usage
- Overall status

**Reference**: [Feature Backlog Section 8.2](./legacy-system-feature-backlog.md#82-ui-components)

#### 8.6: CheckpointModal & LoopIndicator (2-3h)
**Files**: `radium/apps/tui/src/components/checkpoint.rs`, `loop.rs`

- Checkpoint dialog
- Loop status display
- Iteration count

**Reference**: [Feature Backlog Section 8.2](./legacy-system-feature-backlog.md#82-ui-components)

### Deliverables

- ✅ Enhanced TUI matching legacy system
- ✅ Real-time updates working
- ✅ All UI components functional
- ✅ Tests for TUI components

### Success Criteria

- WorkflowDashboard displays correctly
- AgentTimeline shows execution status
- OutputWindow streams agent output
- TelemetryBar shows cost and tokens

---

## Step 9: Agent Library (70+ Agents)

**Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "agent.*library"`

**Priority**: 🟢 Medium  
**Est. Time**: 30-40 hours  
**Dependencies**: Step 1

### Objectives

Port legacy system's 70+ specialized agents to Radium.

### Tasks

#### 9.1: Agent Template Generator (3-4h)
**File**: `radium/tools/agent-generator/src/main.rs`

- Generate agent TOML configs
- Generate prompt file structure
- Agent metadata extraction

**Reference**: [Feature Backlog Section 3](./legacy-system-feature-backlog.md#3-agent-system)

#### 9.2: Port Core Legacy System Agents (8-10h)
**Files**: `radium/agents/legacy-system/*.toml` + prompts

- arch-agent
- plan-agent
- task-breakdown
- context-manager
- code-generation
- task-sanity-check
- runtime-prep
- git-commit
- plan-fallback
- task-fallback
- init
- principal-analyst
- specifications-indexer
- blueprint-orchestrator

**Reference**: [Feature Backlog Section 3.1](./legacy-system-feature-backlog.md#31-main-agents-configmainagentsjs)

#### 9.3: Port RAD-Agents Library (20-30h)
**Files**: `radium/agents/rad-agents/**/*.toml` + prompts

- Design agents (6 agents)
- Engineering agents (9 agents)
- Marketing agents (8 agents)
- Product agents (3 agents)
- Project Management agents (5 agents)
- Security agents (7 agents)
- Testing agents (10 agents)
- Support agents (7 agents)
- Specialized agents (4 agents)
- Spatial Computing agents (6 agents)

**Reference**: [Feature Backlog Section 3.1](./legacy-system-feature-backlog.md#31-main-agents-configmainagentsjs)

#### 9.4: Agent Registry & Discovery (3-4h)
**File**: `crates/radium-core/src/agents/registry.rs`

- Agent registry system
- Agent discovery from all directories
- Agent filtering and lookup

**Reference**: [Feature Backlog Section 3.4](./legacy-system-feature-backlog.md#34-agent-configuration)

### Deliverables

- ✅ 70+ agents available
- ✅ Agent discovery working
- ✅ All prompts ported
- ✅ Tests for agent loading

### Success Criteria

- Can discover all 70+ agents
- Can load agent configs
- Can execute agents
- Agent prompts match legacy system

---

## Step 10: Advanced Features

**Priority**: 🟢 Low  
**Est. Time**: 30-35 hours  
**Dependencies**: Step 1, Step 2, Step 4

### Objectives

Implement remaining legacy system features for complete parity.

### Tasks

#### 10.1: Project Introspection (4-5h)
**File**: `crates/radium-core/src/introspection/mod.rs`

- Tech stack detection (frontend, backend, database)
- Automatic detection from project files
- Package.json analysis
- Configuration file detection

**Reference**: [Feature Backlog Section 10.1](./legacy-system-feature-backlog.md#101-tech-stack-detection)

#### 10.2: Question Generation (3-4h)
**File**: `crates/radium-core/src/introspection/questions.rs`

- AI-powered question generation
- Multiple-choice questions
- Free-text questions
- Required vs optional questions
- Context-aware question generation

**Reference**: [Feature Backlog Section 10.2](./legacy-system-feature-backlog.md#102-question-generation)

#### 10.3: Git Integration (3-4h)
**File**: `crates/radium-core/src/git/mod.rs`

- Git commit agent
- Commit message generation
- Branch creation
- .gitignore management

**Reference**: [Feature Backlog Section 13](./legacy-system-feature-backlog.md#13-git-integration)

#### 10.4: Coordinator Service (5-6h)
**File**: `crates/radium-core/src/coordinator/mod.rs`

- Multi-agent coordination
- Script parsing (`rad run` syntax)
- Parallel execution coordination
- Sequential execution coordination

**Reference**: [Feature Backlog Section 17.1](./legacy-system-feature-backlog.md#171-coordinator-service)

#### 10.5: Additional CLI Commands (3-4h)
**Files**: `radium/apps/cli/src/commands/{run,templates,auth}.rs`

- `rad run` command with advanced syntax
- `rad templates` command
- `rad auth` subcommands (login, logout)
- Non-interactive mode and JSON output

**Reference**: [Feature Backlog Section 1](./legacy-system-feature-backlog.md#1-cli-commands)

#### 10.6: Extension System (8-10h)
**File**: `crates/radium-core/src/extensions/mod.rs`

- Extension discovery and loading
- `gemini-extension.json` parsing
- Extension registry
- MCP server integration via extensions
- Custom commands from extensions
- Extension settings management
- User/workspace scoping for extensions

**Reference**: [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#extension-system)

#### 10.7: Hooks System (4-5h)
**File**: `crates/radium-core/src/hooks/mod.rs`

- Hook registration system
- Before/after model call hooks
- Tool selection and execution hooks
- Error handling hooks
- Telemetry hooks
- Hook configuration in settings

**Reference**: [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#hooks-system)

#### 10.8: Agent Prompting Best Practices Documentation (2-3h)
**Files**: 
- `docs/guides/agent-oversight-guide.md` (new)
- `docs/guides/agent-creation-guide.md` (extend)

- Document system prompt patterns for effective oversight integration
- Guidance on treating oversight as pattern interrupts
- Dosage recommendations (10-20% of steps)
- Examples of phase-aware prompting
- Integration patterns for vibe_check tool

**Reference**: Vibe-check-mcp-server `docs/agent-prompting.md`

#### 10.9: Structured Output for Oversight (2-3h)
**Files**: 
- `crates/radium-core/src/oversight/metacognitive.rs` (extend)

- Enhance vibe_check output to include structured JSON
- Return `{ advice, risk_score, traits, uncertainties }` envelope
- Enable programmatic decision-making in workflows
- Preserve human-readable feedback

**Reference**: Vibe-check-mcp-server roadmap (Priority 1)

### Deliverables

- ✅ All remaining CLI commands
- ✅ Coordinator service working
- ✅ Git integration functional
- ✅ Tests for all features

### Success Criteria

- Can detect tech stack from project
- Can generate questions interactively
- Can create git commits
- Can coordinate multiple agents
- All CLI commands functional
- Can install and use extensions
- Hooks system allows behavior customization
- Documentation available for oversight integration
- Structured oversight output enables programmatic decisions

---

## Summary

| Step | Focus | Est. Time | Priority |
|------|-------|-----------|----------|
| 0 | Workspace System | 10-14h | 🔴 Critical |
| 1 | Agent Configuration | 15-18h | 🔴 Critical |
| 2 | Core CLI Commands | 11-14h | 🔴 Critical |
| 3 | Workflow Behaviors | 21-26h | 🟡 High |
| 4 | Plan Generation & Execution | 15-20h | 🟡 High |
| 5 | Memory & Context | 19-23h | 🟡 High |
| 6 | Monitoring & Telemetry | 18-22h | 🟡 High |
| 6.5 | Sandboxing | 12-15h | 🟡 High |
| 6.6 | Metacognitive Oversight | 20-25h | 🟡 High |
| 7 | Engine Abstraction | 15-20h | 🟢 Medium |
| 8 | Enhanced TUI | 15-20h | 🟢 Medium |
| 9 | Agent Library | 30-40h | 🟢 Medium |
| 10 | Advanced Features | 30-35h | 🟢 Low |
| **Total** | | **262-332h** | |

**Timeline**: 6-9 weeks for complete feature parity (includes gemini-cli enhancements and oversight system)

---

## Reference

- **Now/Next/Later**: [02-now-next-later.md](./02-now-next-later.md)
- **Feature Backlog**: [legacy-system-feature-backlog.md](./legacy-system-feature-backlog.md)
- **Completed Work**: [01-completed.md](./01-completed.md)
- **Feature Gaps**: [FEATURE_GAPS.md](../archive/status-reports/FEATURE_GAPS.md) - Track implemented but not integrated features (archived - all gaps resolved)
- **Vibe-Check Integration**: See BrainGrid REQ-119: `braingrid requirement show REQ-119 -p PROJ-14`
- **Gemini CLI Enhancements**: [gemini-cli-enhancements.md](../features/gemini-cli-enhancements.md)


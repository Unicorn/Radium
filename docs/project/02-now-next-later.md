# Now / Next / Later

> **Prioritized feature roadmap for Radium**  
> **Goal**: Achieve legacy system feature parity while leveraging Radium's Rust architecture  
> **Last Updated**: 2025-12-02 (includes gemini-cli enhancements)

## 🎯 NOW: Immediate Priorities (Steps 0-3)

**Focus**: Foundation for legacy system feature parity

### Step 0: Workspace System
**Status**: ✅ Complete
**Priority**: 🔴 Critical
**Est. Time**: 10-14 hours (Completed)

- [x] Workspace directory structure (`.radium/_internals`, `.radium/plan`)
- [x] `.radium/` internal workspace management
- [x] Requirement ID system (REQ-XXX format)
- [x] Plan discovery and listing
- [x] Plan structure types and validation

**Completed**: All workspace features fully implemented with 22+ passing tests. RequirementId auto-incrementing, Plan/Iteration/Task structures, and PlanDiscovery all working.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-0-workspace-system) for detailed tasks.

### Step 1: Agent Configuration System
**Status**: ✅ Complete
**Priority**: 🔴 Critical
**Est. Time**: 15-18 hours (Completed)

- [x] Agent configuration format (TOML-based)
- [x] Agent discovery from directories
- [x] Prompt template loading and organization
- [x] Basic placeholder replacement
- [x] Module configuration with behaviors
- [ ] **MCP (Model Context Protocol) integration** for external tool discovery (Future)
- [ ] **Context Files (GEMINI.md)** hierarchical loading system (Future)

**Completed**: Core agent configuration system fully implemented with ~1,070 lines of code across agents/config.rs (337 lines), agents/discovery.rs (377 lines), and prompts/templates.rs (356 lines). TOML-based configuration, agent discovery, and template system all working. MCP and Context Files deferred to future enhancement.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-1-agent-configuration-system) for detailed tasks.

### Step 2: Core CLI Commands
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
- [x] CLI structure matching legacy system
- [ ] **TESTING**: Integration tests for all CLI commands (0% coverage)
  - [ ] `rad init` - All initialization paths
  - [ ] `rad status` - Human and JSON output modes
  - [ ] `rad clean` - Verbose and non-verbose modes
  - [ ] `rad plan` - All input methods and error cases
  - [ ] `rad craft` - Execution modes and error handling
  - [ ] `rad agents` - All subcommands (list, search, info, validate)
  - [ ] `rad templates` - All subcommands
  - [ ] `rad auth` - Login, logout, status
  - [ ] `rad step` - Single agent execution
  - [ ] `rad run` - Agent script execution

**Why Now**: Primary user interface. Must match Radium's `rad` command structure.

**Test Coverage Gap**: ~1,200 lines of CLI command code have 0% test coverage. Critical for reliability.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-2-core-cli-commands) for detailed tasks.  
**Test Requirements**: See [TEST_COVERAGE_REPORT.md](./TEST_COVERAGE_REPORT.md#step-2-core-cli-commands) for test details.

### Step 3: Workflow Behaviors
**Status**: ✅ Complete
**Priority**: 🟡 High
**Est. Time**: 18-22 hours (Completed)

- [x] Loop behavior (step back with max iterations)
- [x] Trigger behavior (dynamic agent triggering)
- [x] Checkpoint behavior (save and resume)
- [x] Behavior.json control file support
- [x] Workflow template system
- [x] **Policy Engine** for fine-grained tool execution control
- [x] Approval modes and rule-based tool filtering

**Completed**: Full workflow behavior system with 48 passing tests (27 behavior tests + 21 policy tests). Includes workflow behaviors (~1,100 lines) with types.rs (281 lines), loop_behavior.rs (344 lines), trigger.rs (241 lines), checkpoint.rs (227 lines), template discovery, and Policy Engine (~450 lines) with TOML-based rules, priority-based matching (Admin/User/Default), approval modes (yolo/autoEdit/ask), and glob pattern matching.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-3-workflow-behaviors) for detailed tasks.

---

## 🔜 NEXT: High-Value Features (Steps 4-6)

**Focus**: Essential legacy system functionality

### Step 4: Plan Generation & Execution
**Status**: ✅ Complete
**Priority**: 🟢 Completed
**Est. Time**: 15-20 hours (completed)

**Completed:**
- [x] `rad plan` basic implementation (~259 lines)
  - [x] Specification parsing (file or direct input)
  - [x] RequirementId generation and validation
  - [x] Plan directory structure creation
  - [x] Basic plan generation from specs
  - [x] Plan manifest generation with iterations/tasks
  - [x] plan.json and plan_manifest.json output
- [x] `rad craft` with PlanExecutor (~305 lines)
  - [x] Plan discovery by REQ-ID or folder name
  - [x] Iteration-by-iteration execution
  - [x] Task-by-task execution with state persistence
  - [x] Resume from checkpoint (full implementation)
  - [x] Agent discovery and execution
  - [x] Model execution with mock fallback
  - [x] Dry-run mode
  - [x] Dependency validation
  - [x] Progress tracking with percentage display
  - [x] Checkpoint persistence after each task
- [x] **Planning module** (~1,110 lines, 10 tests)
  - [x] AI-powered plan generation using LLM abstraction
  - [x] PlanParser for parsing LLM markdown responses
  - [x] PlanGenerator with configurable model parameters
  - [x] Detailed markdown file generation (4 files)
  - [x] Project overview, iteration details, verification docs
  - [x] Coordinator prompt generation
  - [x] PlanExecutor with state persistence (~410 lines, 5 tests)
  - [x] Task execution with error handling
  - [x] Progress calculation and tracking
  - [x] Dependency validation
  - [x] Manifest save/load for checkpoints

**Summary**: Full plan generation and execution system with AI-powered planning, state persistence, dependency management, and progress tracking. Users can generate plans from specifications and execute them with automatic checkpointing and resume support.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-4-plan-generation--execution) for detailed tasks.

### Step 5: Memory & Context System
**Status**: Not Started  
**Priority**: 🟡 High  
**Est. Time**: 15-18 hours

- Plan-scoped memory storage
- File-based memory adapter
- Context gathering (architecture, plan, codebase)
- File input injection syntax (`agent[input:file1.md]`)
- Tail context support (`agent[tail:50]`)
- **Custom Commands (TOML-based)** system for reusable prompts
- Enhanced context system with file injection syntax (`@{file}`)
- Shell command injection syntax (`!{command}`)

**Why Next**: Essential for agent execution quality. Agents need context from previous runs.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-5-memory--context-system) for detailed tasks.

### Step 6: Monitoring & Telemetry
**Status**: Not Started  
**Priority**: 🟡 High  
**Est. Time**: 18-22 hours

- Agent monitoring database (SQLite)
- Agent lifecycle tracking
- Telemetry parsing (tokens, cost, cache stats)
- Log file management
- Parent-child agent relationships
- **Checkpointing system** (Git snapshots + conversation history)
- `/restore` command functionality

**Why Next**: Needed for debugging, cost tracking, and agent coordination.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-6-monitoring--telemetry) for detailed tasks.

### Step 6.5: Sandboxing
**Status**: Not Started  
**Priority**: 🟡 High  
**Est. Time**: 12-15 hours

- Sandboxing support (Docker/Podman/macOS Seatbelt)
- Sandbox profiles and configuration
- Network control and isolation
- Custom sandbox flags

**Why Next**: Security and safety for agent execution, especially for shell commands and file operations.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-65-sandboxing) for detailed tasks.

---

## ⏰ LATER: Advanced Features (Steps 7-10)

**Focus**: Complete feature parity and enhancements

### Step 7: Engine Abstraction Layer
**Status**: Not Started  
**Priority**: 🟢 Medium  
**Est. Time**: 15-20 hours

- Engine registry and factory
- CLI binary detection
- Authentication system per engine
- Support for: Codex, Claude, Cursor, CCR, OpenCode, Auggie
- Model selection and reasoning effort

**Why Later**: Current Gemini/OpenAI support is sufficient. Multi-engine support can come after core features.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-7-engine-abstraction-layer) for detailed tasks.

### Step 8: Enhanced TUI
**Status**: Not Started  
**Priority**: 🟢 Medium  
**Est. Time**: 15-20 hours

- WorkflowDashboard component
- AgentTimeline with status indicators
- OutputWindow with streaming
- TelemetryBar and StatusFooter
- CheckpointModal and LoopIndicator
- Real-time state updates

**Why Later**: Current TUI is functional. Enhanced UI can come after core functionality.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-8-enhanced-tui) for detailed tasks.

### Step 9: Enhanced Agent Library (72+ Agents)
**Status**: Planning Complete
**Priority**: 🟡 High (Upgraded)
**Est. Time**: 40-50 hours

**NEW: Comprehensive Agent Persona Enhancement**

#### Phase 1: YAML Schema & Parser (Week 1)
- Enhanced YAML frontmatter with model recommendations
- Model selection engine (speed/balanced/thinking/expert)
- Cost estimation and budget tracking
- Fallback chain logic (primary → fallback → mock)

#### Phase 2: Agent Library Enhancement (Weeks 2-3)
- Update all 72 existing agents with enhanced metadata
- Add recommended_models for each agent (primary, fallback, premium)
- Add capabilities, performance_profile, quality_gates
- Category-specific model recommendation guidelines

#### Phase 3: CLI Integration (Week 4)
- `rad step --auto-model` - Use agent's recommended model
- `rad craft` - Per-task model optimization
- `rad agents list` - Browse agents with metadata
- `rad agents search` - Capability-based agent search
- Cost estimation in execution output

#### Phase 4: Advanced Features (Weeks 5-6)
- Agent recommendation engine
- Interactive TUI agent selector
- Cost optimization strategies
- Performance profiling

**Why Elevated to High Priority**:
- Enables intelligent model selection (30-50% cost reduction)
- Improves agent discovery and usability
- Foundation for multi-model orchestration
- 72 existing agents ready to enhance

**Detailed Plan**: See [radium/roadmap/AGENT_LIBRARY_ENHANCEMENT_PLAN.md](../radium/roadmap/AGENT_LIBRARY_ENHANCEMENT_PLAN.md)

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-9-agent-library) for detailed tasks.

### Step 10: Advanced Features
**Status**: Not Started
**Priority**: 🟢 Low
**Est. Time**: 30-35 hours

- Project introspection (tech stack detection)
- AI-powered question generation
- Git integration (git-commit agent)
- Coordinator service (`rad run` command)
- ✅ `rad templates` commands (Complete)
- ✅ `rad auth` commands - API key auth (Complete)
- Non-interactive mode and JSON output
- **Extension System** (installable extensions with gemini-extension.json)
- **Hooks System** for behavior customization

**Why Later**: Advanced features that enhance usability but aren't core to functionality.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-10-advanced-features) for detailed tasks.

### Step 11: Multi-Method Authentication (OAuth/Session)
**Status**: Not Started
**Priority**: 🟢 Low (Future Enhancement)
**Est. Time**: 8-12 hours

- OAuth flow support (browser redirect, callback server, PKCE)
- Token refresh logic and expiration checking
- Cloud provider credentials (AWS, Azure, GCP)
- Multiple auth methods per provider
- Token validation and auto-refresh
- Migration from v1.0 (API key) to v2.0 (multi-method) format

**Why Later**: API key authentication covers 90% of LLM providers. OAuth needed for enterprise SSO and cloud platform integrations.

**Reference**: See [authentication-system-plan.md](./authentication-system-plan.md#future-enhancements-option-b-multi-method-auth) for detailed design.

---

## 📊 Summary

| Phase | Steps | Est. Time | Priority |
|-------|-------|-----------|----------|
| **NOW** | 0-3 | 45-61 hours | 🔴 Critical |
| **NEXT** | 4-6, 6.5 | 67-87 hours | 🟡 High |
| **LATER** | 7-8, 10 | 80-100 hours | 🟢 Medium/Low |
| **HIGH** | 9 | 40-50 hours | 🟡 High (Agent Library) |
| **TESTING** | All | 60-80 hours | 🔴 Critical |
| **Total** | 0-10 | 292-378 hours | |

**Timeline Estimate**: 
- **NOW**: 1-2 weeks
- **NEXT**: 2-3 weeks (includes gemini-cli enhancements)
- **LATER**: 2-3 weeks
- **TESTING**: 1.5-2 weeks (parallel with feature work)
- **Total**: 6-10 weeks for complete feature parity + 100% test coverage

## 🧪 Test Coverage Status

**Current Coverage**: ~37.61% (2080/5531 lines)  
**Target Coverage**: 100%  
**Coverage Gap**: 62.39% (3,451 lines)

### Critical Test Gaps

1. **CLI Commands** (0% coverage) - ~1,200 lines
   - **Priority**: 🔴 Critical
   - **Est. Time**: 15-20 hours
   - **Impact**: All user-facing functionality untested

2. **Server/gRPC** (0% coverage) - ~167 lines
   - **Priority**: 🔴 Critical
   - **Est. Time**: 10-15 hours
   - **Impact**: API layer completely untested

3. **Workflow Service** (0% → Partial coverage) - 56 lines
   - **Priority**: 🔴 Critical
   - **Est. Time**: 3-5 hours
   - **Impact**: Core workflow execution untested

**See [TEST_COVERAGE_REPORT.md](./TEST_COVERAGE_REPORT.md) for detailed coverage analysis and test requirements.**

---

## 🎯 Success Criteria

Feature parity is achieved when:

1. ✅ All CLI commands from legacy system work in Radium
2. ✅ Workflow execution with all behaviors (loop, trigger, checkpoint)
3. ✅ Plan system fully functional (generate, discover, execute)
4. ✅ Memory and context system working
5. ✅ Monitoring and telemetry operational
6. ✅ Workspace structure compatible with legacy system
7. ✅ Test coverage >80% for all new features

---

## 📚 Reference

- **Detailed Implementation Plan**: [03-implementation-plan.md](./03-implementation-plan.md)
- **Feature Backlog**: [legacy-system-feature-backlog.md](./legacy-system-feature-backlog.md)
- **Completed Work**: [01-completed.md](./01-completed.md)
- **Gemini CLI Enhancements**: [gemini-cli-enhancements.md](../features/gemini-cli-enhancements.md)


# Now / Next / Later

> **Prioritized feature roadmap for Radium**  
> **Goal**: Achieve legacy system feature parity while leveraging Radium's Rust architecture  
> **Last Updated**: 2025-12-02

## 🎯 NOW: Immediate Priorities (Steps 0-3)

**Focus**: Foundation for legacy system feature parity

### Step 0: Workspace System
**Status**: 🔄 In Progress  
**Priority**: 🔴 Critical  
**Est. Time**: 10-14 hours

- [x] Workspace directory structure (`.radium/_internals`, `.radium/plan`)
- [x] `.radium/` internal workspace management
- Requirement ID system (REQ-XXX format)
- [x] Plan discovery and listing
- Plan structure types and validation

**Why Now**: Every legacy system feature depends on workspace structure. This is the foundation.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-0-workspace-system) for detailed tasks.

### Step 1: Agent Configuration System
**Status**: Not Started  
**Priority**: 🔴 Critical  
**Est. Time**: 9-12 hours

- Agent configuration format (TOML-based)
- Agent discovery from directories
- Prompt template loading and organization
- Basic placeholder replacement
- Module configuration with behaviors

**Why Now**: All CLI commands and workflows need agent configuration to function.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-1-agent-configuration-system) for detailed tasks.

### Step 2: Core CLI Commands
**Status**: ✅ Complete
**Priority**: 🔴 Critical
**Est. Time**: 8-10 hours (Completed)

- [x] `rad init` - Intelligent workspace initialization
- [x] `rad status` - Show workspace and engine status
- [x] `rad clean` - Clean workspace artifacts
- [x] `rad plan` - Generate plans from specifications
- [x] `rad craft` - Execute plans
- [x] `rad agents` - Agent management (list, search, info, validate)
- [x] `rad templates` - Template management (list, info, validate)
- [x] CLI structure matching legacy system

**Why Now**: Primary user interface. Must match Radium's `rad` command structure.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-2-core-cli-commands) for detailed tasks.

### Step 3: Workflow Behaviors
**Status**: Not Started  
**Priority**: 🟡 High  
**Est. Time**: 12-15 hours

- Loop behavior (step back with max iterations)
- Trigger behavior (dynamic agent triggering)
- Checkpoint behavior (save and resume)
- Behavior.json control file support
- Workflow template system

**Why Now**: Core workflow execution features. Needed for `rad craft` to work properly.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-3-workflow-behaviors) for detailed tasks.

---

## 🔜 NEXT: High-Value Features (Steps 4-6)

**Focus**: Essential legacy system functionality

### Step 4: Plan Generation & Execution
**Status**: Not Started  
**Priority**: 🟡 High  
**Est. Time**: 15-20 hours

- `rad plan` full implementation
  - Specification parsing
  - AI-powered plan generation
  - Iteration and task extraction
  - Plan file generation
- `rad craft` full implementation
  - Plan execution (iteration-by-iteration, task-by-task)
  - Resume from checkpoint
  - Progress tracking

**Why Next**: Core Radium workflow. Users need to generate and execute plans.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-4-plan-generation--execution) for detailed tasks.

### Step 5: Memory & Context System
**Status**: Not Started  
**Priority**: 🟡 High  
**Est. Time**: 10-12 hours

- Plan-scoped memory storage
- File-based memory adapter
- Context gathering (architecture, plan, codebase)
- File input injection syntax (`agent[input:file1.md]`)
- Tail context support (`agent[tail:50]`)

**Why Next**: Essential for agent execution quality. Agents need context from previous runs.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-5-memory--context-system) for detailed tasks.

### Step 6: Monitoring & Telemetry
**Status**: Not Started  
**Priority**: 🟡 High  
**Est. Time**: 12-15 hours

- Agent monitoring database (SQLite)
- Agent lifecycle tracking
- Telemetry parsing (tokens, cost, cache stats)
- Log file management
- Parent-child agent relationships

**Why Next**: Needed for debugging, cost tracking, and agent coordination.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-6-monitoring--telemetry) for detailed tasks.

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
**Est. Time**: 20-25 hours

- Project introspection (tech stack detection)
- AI-powered question generation
- Git integration (git-commit agent)
- Coordinator service (`rad run` command)
- `rad templates` and `rad auth` commands
- Non-interactive mode and JSON output

**Why Later**: Advanced features that enhance usability but aren't core to functionality.

**Reference**: See [03-implementation-plan.md](./03-implementation-plan.md#step-10-advanced-features) for detailed tasks.

---

## 📊 Summary

| Phase | Steps | Est. Time | Priority |
|-------|-------|-----------|----------|
| **NOW** | 0-3 | 39-51 hours | 🔴 Critical |
| **NEXT** | 4-6 | 37-47 hours | 🟡 High |
| **LATER** | 7-8, 10 | 50-65 hours | 🟢 Medium/Low |
| **HIGH** | 9 | 40-50 hours | 🟡 High (Agent Library) |
| **Total** | 0-10 | 166-213 hours | |

**Timeline Estimate**: 
- **NOW**: 1-2 weeks
- **NEXT**: 1-2 weeks  
- **LATER**: 2-3 weeks
- **Total**: 4-7 weeks for complete feature parity

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


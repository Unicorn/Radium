---
req_id: REQ-006
title: Memory & Context System
phase: NEXT
status: Completed
priority: High
estimated_effort: 19-23 hours
dependencies: [REQ-001, REQ-002]
related_docs:
  - docs/project/02-now-next-later.md#step-5-memory--context-system
  - docs/project/03-implementation-plan.md#step-5-memory--context-system
  - docs/legacy/legacy-system-feature-backlog.md#61-plan-scoped-memory
---

# Memory & Context System

## Problem Statement

Agents need access to context from previous executions and the ability to inject relevant information into prompts. Without a memory and context system, agents cannot:
- Remember outputs from previous runs
- Access plan information and metadata
- Inject file contents into prompts dynamically
- Maintain conversation history across sessions
- Use custom commands for common operations
- Learn from past executions

The legacy system provided plan-scoped memory storage and context injection capabilities. Radium needs an equivalent system that supports memory persistence, context gathering, and file injection.

## Solution Overview

Implement a comprehensive memory and context system that provides:
- Plan-scoped memory storage for agent outputs
- Context gathering from multiple sources (plan, architecture, codebase)
- File injection syntax for dynamic content inclusion
- Conversation history tracking with summarization
- Custom commands system for reusable operations
- Integration with learning system for past mistakes and strategies

The memory and context system enables agents to maintain continuity across executions and access relevant information for their tasks.

## Functional Requirements

### FR-1: Plan-Scoped Memory Store

**Description**: Store agent outputs in plan-scoped memory with persistence.

**Acceptance Criteria**:
- [x] Memory directory per plan: `radium/<stage>/REQ-XXX/memory/`
- [x] Memory store interface with async trait
- [x] Agent output storage (last 2000 chars per agent)
- [x] Timestamp tracking for memory entries
- [x] File-based memory adapter
- [x] Memory entry retrieval by agent ID
- [x] Memory persistence and caching

**Implementation**: 
- `crates/radium-core/src/memory/store.rs` (~670 lines)
- `crates/radium-core/src/memory/adapter.rs`

### FR-2: Context Manager

**Description**: Gather context from multiple sources and build comprehensive context strings.

**Acceptance Criteria**:
- [x] Context gathering (architecture, plan, codebase)
- [x] File input injection syntax: `agent[input:file1.md,file2.md]`
- [x] Tail context support: `agent[tail:50]`
- [x] InjectionDirective parsing and execution
- [x] Multi-source context building
- [x] Integration with memory store
- [x] Integration with learning store

**Implementation**: 
- `crates/radium-core/src/context/manager.rs` (~590 lines)
- `crates/radium-core/src/context/injection.rs`

### FR-3: Conversation History

**Description**: Track conversation history per session with summarization.

**Acceptance Criteria**:
- [x] Session-based conversation history tracking
- [x] History summarization (last 5 interactions)
- [x] Context window management to prevent bloat
- [x] Integration with ContextManager
- [x] Support history retrieval by session ID
- [x] Automatic cleanup of old interactions (max 10 per session)

**Implementation**: `crates/radium-core/src/context/history.rs`

### FR-4: Custom Commands System

**Description**: TOML-based command definitions for reusable operations.

**Acceptance Criteria**:
- [x] TOML-based command definitions
- [x] Command discovery (user vs project precedence)
- [x] Shell command injection: `!{command}`
- [x] File content injection: `@{file}`
- [x] Argument handling: `{{args}}`, `{{arg1}}`
- [x] Namespaced commands via directory structure
- [x] User vs project command precedence

**Implementation**: `crates/radium-core/src/commands/custom.rs` (~430 lines)

## Technical Requirements

### TR-1: Memory Store API

**Description**: APIs for storing and retrieving agent outputs.

**APIs**:
```rust
pub trait MemoryAdapter: Send + Sync {
    async fn store(&self, entry: MemoryEntry) -> Result<()>;
    async fn get(&self, agent_id: &str) -> Result<Option<MemoryEntry>>;
    async fn list_agents(&self) -> Result<Vec<String>>;
}

pub struct MemoryStore {
    adapter: Box<dyn MemoryAdapter>,
}

impl MemoryStore {
    pub fn store(&self, entry: MemoryEntry) -> Result<()>;
    pub fn get(&self, agent_id: &str) -> Result<Option<MemoryEntry>>;
    pub fn get_mut(&mut self, agent_id: &str) -> Option<&mut MemoryEntry>;
}
```

**Data Models**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub agent_id: String,
    pub content: String,  // Truncated to 2000 chars
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}
```

**Storage**: `radium/<stage>/REQ-XXX/memory/<agent-id>.json`

### TR-2: Context Manager API

**Description**: APIs for gathering and building context.

**APIs**:
```rust
pub struct ContextManager {
    workspace_root: PathBuf,
    injector: ContextInjector,
    memory_store: Option<MemoryStore>,
    learning_store: Option<LearningStore>,
}

impl ContextManager {
    pub fn for_plan(workspace: &Workspace, req_id: RequirementId) -> Result<Self>;
    pub fn build_context(&self, invocation: &str, req_id: Option<RequirementId>) -> Result<String>;
    pub fn gather_plan_context(&self, req_id: RequirementId) -> Result<String>;
    pub fn gather_memory_context(&self, agent_id: &str) -> Result<Option<String>>;
    pub fn gather_architecture_context(&self) -> Option<String>;
}
```

### TR-3: File Injection Syntax

**Description**: Syntax for injecting file contents into prompts.

**Syntax**:
```
agent[input:file1.md,file2.md]  # Inject file contents
agent[tail:50]                  # Inject last 50 lines from previous output
```

**Data Models**:
```rust
#[derive(Debug, Clone)]
pub enum InjectionDirective {
    Input { files: Vec<PathBuf> },
    Tail { lines: usize },
}

pub struct ContextInjector {
    workspace_root: PathBuf,
}

impl ContextInjector {
    pub fn process_directives(&self, directives: &[InjectionDirective]) -> Result<String>;
}
```

### TR-4: Custom Commands Format

**Description**: TOML format for custom command definitions.

**TOML Format**:
```toml
[command]
name = "command-name"
description = "Command description"
shell = "!{command} {{args}}"
file = "@{file}"
```

**Location**: `.radium/commands/*.toml` or `~/.radium/commands/*.toml`

## User Experience

### UX-1: Memory Storage

**Description**: Agent outputs are automatically stored in plan-scoped memory.

**Example**:
```rust
let memory_store = MemoryStore::new(&workspace_root, req_id)?;
let entry = MemoryEntry::new("code-agent".to_string(), output);
memory_store.store(entry)?;
```

### UX-2: File Injection

**Description**: Users inject file contents into agent prompts.

**Example**:
```bash
$ rad step code-agent[input:spec.md,requirements.md]
# Agent receives spec.md and requirements.md contents in context
```

### UX-3: Context Gathering

**Description**: Context is automatically gathered from multiple sources.

**Example**:
```rust
let manager = ContextManager::for_plan(&workspace, req_id)?;
let context = manager.build_context("arch-agent[input:spec.md]", Some(req_id))?;
// Context includes: plan info, architecture docs, memory, spec.md contents
```

## Data Requirements

### DR-1: Memory Entries

**Description**: JSON files storing agent outputs per plan.

**Location**: `radium/<stage>/REQ-XXX/memory/<agent-id>.json`

**Schema**:
```json
{
  "agent_id": "code-agent",
  "content": "Last 2000 chars of output...",
  "timestamp": 1234567890,
  "metadata": {}
}
```

### DR-2: Conversation History

**Description**: Session-based conversation history storage.

**Location**: `.radium/_internals/sessions/<session-id>/history.json`

**Format**: Array of interaction objects with timestamps

### DR-3: Custom Commands

**Description**: TOML files defining custom commands.

**Location**: `.radium/commands/*.toml` or `~/.radium/commands/*.toml`

**Schema**: See TR-4 Custom Commands Format

## Dependencies

- **REQ-001**: Workspace System - Required for workspace structure and plan discovery
- **REQ-002**: Agent Configuration - Required for agent identification

## Success Criteria

1. [x] Agent outputs can be stored in plan-scoped memory
2. [x] Memory persists across agent executions
3. [x] File contents can be injected into prompts
4. [x] Context can be gathered from multiple sources
5. [x] Conversation history tracks sessions correctly
6. [x] History summarization prevents context window bloat
7. [x] Custom commands can be defined and executed
8. [x] All memory and context operations have comprehensive test coverage (50+ tests)

**Completion Metrics**:
- **Status**: ✅ Complete
- **Lines of Code**: ~2,000+ lines
  - Memory module: ~670 lines (18 tests)
  - Context manager: ~590 lines (24 tests)
  - Custom commands: ~430 lines (8 tests)
  - History: ~310 lines
- **Test Coverage**: 50+ passing tests
- **Implementation**: Complete memory and context system
- **Files**: 
  - `crates/radium-core/src/memory/` (store, adapter)
  - `crates/radium-core/src/context/` (manager, injection, history)
  - `crates/radium-core/src/commands/custom.rs`

## Out of Scope

- Advanced context retrieval (semantic search, future enhancement)
- Context compression (future enhancement)
- Multi-plan context sharing (future enhancement)
- Context versioning (future enhancement)

## References

- [Now/Next/Later Roadmap](../project/02-now-next-later.md#step-5-memory--context-system)
- [Implementation Plan](../project/03-implementation-plan.md#step-5-memory--context-system)
- [Feature Backlog](../legacy/legacy-system-feature-backlog.md#61-plan-scoped-memory)
- [Memory Module Implementation](../../crates/radium-core/src/memory/)
- [Context Manager Implementation](../../crates/radium-core/src/context/)


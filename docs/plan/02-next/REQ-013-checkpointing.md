---
req_id: REQ-013
title: Checkpointing
phase: NEXT
status: Completed
priority: High
estimated_effort: 6-7 hours
dependencies: [REQ-001, REQ-007]
related_docs:
  - docs/features/gemini-cli-enhancements.md#checkpointing-system
  - docs/project/03-implementation-plan.md#step-6-monitoring--telemetry
---

# Checkpointing

## Problem Statement

Users need a way to safely experiment with code changes and easily rollback if needed. Without checkpointing, users cannot:
- Safely try different approaches to problems
- Rollback to previous working states
- Preserve conversation history across restores
- Create snapshots before risky operations
- Restore specific checkpoints with full context

The legacy system and modern AI tools (like gemini-cli) use Git-based checkpointing for safe experimentation. Radium needs an equivalent system that creates Git snapshots and preserves conversation history.

## Solution Overview

Implement a Git-based checkpointing system that provides:
- Git snapshot creation before file modifications
- Shadow Git repository management
- Conversation history preservation
- `/restore` command functionality
- Tool call re-proposal after restore
- Checkpoint listing and selection
- Automatic checkpoint creation before workflow steps

The checkpointing system enables safe experimentation with code changes, easy rollback to previous states, and conversation context preservation.

## Functional Requirements

### FR-1: Git Snapshot Creation

**Description**: Create Git snapshots before file modifications.

**Acceptance Criteria**:
- [x] Git snapshot creation before workflow steps
- [x] Shadow Git repository management
- [x] Checkpoint metadata (id, commit_hash, timestamp, description)
- [x] Checkpoint tagging in shadow repo
- [x] Automatic checkpoint creation before file modifications

**Implementation**: `crates/radium-core/src/checkpoint/snapshot.rs`

### FR-2: Checkpoint Restoration

**Description**: Restore checkpoints with conversation history preservation.

**Acceptance Criteria**:
- [x] Checkpoint restoration from commit hash
- [x] Conversation history preservation
- [x] `/restore` command handler in workflow executor
- [x] Tool call re-proposal after restore
- [x] Restore request detection in agent output

**Implementation**: 
- `crates/radium-core/src/checkpoint/snapshot.rs`
- `crates/radium-core/src/workflow/executor.rs` (restore handler)

### FR-3: Checkpoint Management

**Description**: List, view, and manage checkpoints.

**Acceptance Criteria**:
- [x] Checkpoint listing
- [x] Checkpoint retrieval by ID
- [x] Checkpoint deletion
- [x] Checkpoint metadata display
- [x] CLI commands for checkpoint management

**Implementation**: 
- `crates/radium-core/src/checkpoint/snapshot.rs`
- `apps/cli/src/commands/checkpoint.rs`

### FR-4: Workflow Integration

**Description**: Automatic checkpoint creation during workflow execution.

**Acceptance Criteria**:
- [x] Automatic checkpoint creation before workflow steps
- [x] Checkpoint persistence after each task
- [x] Integration with workflow executor
- [x] Checkpoint state tracking

**Implementation**: 
- `crates/radium-core/src/workflow/executor.rs`
- `crates/radium-core/src/monitoring/service.rs`

## Technical Requirements

### TR-1: Checkpoint Data Structure

**Description**: Checkpoint metadata structure.

**Data Models**:
```rust
#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub id: String,
    pub commit_hash: String,
    pub agent_id: Option<String>,
    pub timestamp: u64,
    pub description: Option<String>,
}
```

### TR-2: Checkpoint Manager API

**Description**: APIs for checkpoint management.

**APIs**:
```rust
pub struct CheckpointManager {
    workspace_root: PathBuf,
    shadow_repo: PathBuf,
}

impl CheckpointManager {
    pub fn new(workspace_root: &Path) -> Result<Self>;
    pub fn create_checkpoint(&self, description: Option<String>) -> Result<Checkpoint>;
    pub fn restore_checkpoint(&self, checkpoint_id: &str) -> Result<()>;
    pub fn list_checkpoints(&self) -> Result<Vec<Checkpoint>>;
    pub fn get_checkpoint(&self, checkpoint_id: &str) -> Result<Checkpoint>;
    pub fn delete_checkpoint(&self, checkpoint_id: &str) -> Result<()>;
}
```

### TR-3: Shadow Git Repository

**Description**: Shadow Git repository for checkpoint storage.

**Location**: `~/.radium/history/<workspace-hash>/`

**Structure**: Git repository with tags for each checkpoint

## User Experience

### UX-1: Automatic Checkpointing

**Description**: Checkpoints are automatically created before file modifications.

**Example**:
```bash
$ rad craft REQ-001
Executing plan...
  Creating checkpoint before task execution...
  ✓ Checkpoint created: checkpoint-abc123
  → Executing task...
```

### UX-2: Checkpoint Restoration

**Description**: Users restore checkpoints via `/restore` command or CLI.

**Example**:
```bash
$ rad checkpoint restore checkpoint-abc123
Restoring checkpoint...
  ✓ Restored to commit: abc123def456
  ✓ Conversation history preserved
```

### UX-3: Checkpoint Listing

**Description**: Users list and view checkpoints.

**Example**:
```bash
$ rad checkpoint list
Checkpoints:
  checkpoint-abc123 (2025-12-06 11:30:00) - Before refactoring
  checkpoint-def456 (2025-12-06 11:35:00) - After API changes
```

## Data Requirements

### DR-1: Shadow Git Repository

**Description**: Git repository for checkpoint storage.

**Location**: `~/.radium/history/<workspace-hash>/`

**Structure**: Standard Git repository with tags

### DR-2: Checkpoint Metadata

**Description**: Checkpoint information stored in Git tags.

**Format**: Git tags with checkpoint IDs and metadata

## Dependencies

- **REQ-001**: Workspace System - Required for workspace structure
- **REQ-007**: Monitoring & Telemetry - Required for workflow integration

## Success Criteria

1. [x] Git snapshots can be created before file modifications
2. [x] Checkpoints can be restored with conversation history
3. [x] `/restore` command is detected and processed
4. [x] Tool calls are re-proposed after restore
5. [x] Checkpoints can be listed and managed
6. [x] Automatic checkpoint creation works during workflow execution
7. [x] All checkpoint operations have comprehensive test coverage (15+ tests)

**Completion Metrics**:
- **Status**: ✅ Complete
- **Test Coverage**: 15+ passing tests
- **Implementation**: Full checkpointing system integrated with workflows
- **Files**: 
  - `crates/radium-core/src/checkpoint/` (snapshot, error)
  - `apps/cli/src/commands/checkpoint.rs`

## Out of Scope

- Advanced checkpoint merging (future enhancement)
- Checkpoint diff visualization (future enhancement)
- Checkpoint scheduling (future enhancement)

## References

- [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#checkpointing-system)
- [Implementation Plan](../project/03-implementation-plan.md#step-6-monitoring--telemetry)
- [Checkpoint Implementation](../../crates/radium-core/src/checkpoint/)


---
id: "checkpoint-system"
title: "Checkpoint System Architecture"
sidebar_label: "Checkpoint System Architecture"
---

# Checkpoint System Architecture

This document describes the technical architecture of Radium's checkpointing system for developers.

## Overview

The checkpointing system provides Git-based snapshots of workspace state, enabling safe experimentation and easy rollback. It uses a shadow Git repository to store checkpoints separately from the main workspace repository.

## System Components

### Core Components

```
┌─────────────────────────────────────────────────────────┐
│                   CheckpointManager                      │
│  - Shadow repository management                           │
│  - Checkpoint CRUD operations                             │
│  - Git snapshot creation                                  │
│  - Workspace restoration                                  │
└──────────────┬────────────────────────────────────────────┘
               │
               ├─────────────────┬──────────────────┐
               ▼                 ▼                  ▼
    ┌──────────────────┐  ┌──────────────┐  ┌──────────────┐
    │  Shadow Git Repo  │  │   Workspace  │  │   Workflow   │
    │  (bare repo)      │  │   (main repo)│  │   Executor   │
    └──────────────────┘  └──────────────┘  └──────────────┘
```

## Architecture Details

### CheckpointManager

**Location**: `crates/radium-core/src/checkpoint/snapshot.rs`

The `CheckpointManager` is the central component that manages all checkpoint operations.

#### Key Methods

```rust
pub struct CheckpointManager {
    workspace_root: PathBuf,
    shadow_repo: PathBuf,
}

impl CheckpointManager {
    pub fn new(workspace_root: impl AsRef<Path>) -> Result<Self>;
    pub fn initialize_shadow_repo(&self) -> Result<()>;
    pub fn create_checkpoint(&self, description: Option<String>) -> Result<Checkpoint>;
    pub fn restore_checkpoint(&self, checkpoint_id: &str) -> Result<()>;
    pub fn list_checkpoints(&self) -> Result<Vec<Checkpoint>>;
    pub fn get_checkpoint(&self, checkpoint_id: &str) -> Result<Checkpoint>;
    pub fn delete_checkpoint(&self, checkpoint_id: &str) -> Result<()>;
    pub fn diff_checkpoint(&self, checkpoint_id: &str) -> Result<String>;
    pub fn find_checkpoint_for_step(&self, step_id: &str) -> Option<Checkpoint>;
}
```

#### Shadow Repository Location

The shadow repository is stored at:
```
<workspace-root>/.radium/_internals/checkpoints/
```

This is a **bare Git repository** (initialized with `git init --bare`), which means:
- No working directory
- All data stored in `.git` structure
- Optimized for storing snapshots
- Can be safely backed up or moved

### Checkpoint Data Structure

```rust
#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub id: String,                    // Format: "checkpoint-<uuid>"
    pub commit_hash: String,           // Git commit hash
    pub agent_id: Option<String>,      // Agent that created checkpoint
    pub timestamp: u64,                // Unix epoch seconds
    pub description: Option<String>,   // User-provided description
    pub task_id: Option<String>,        // Associated task ID
    pub workflow_id: Option<String>,   // Associated workflow ID
}
```

### Checkpoint Creation Flow

```
1. Workflow Executor detects file modification step
   │
   ▼
2. CheckpointManager::create_checkpoint() called
   │
   ▼
3. Ensure shadow repo is initialized (bare repo)
   │
   ▼
4. Get current HEAD commit hash from workspace
   │
   ▼
5. Create Git tag in shadow repo: "checkpoint-<uuid>"
   │
   ▼
6. Store checkpoint metadata (id, commit_hash, timestamp, etc.)
   │
   ▼
7. Return Checkpoint struct
```

### Checkpoint Restoration Flow

```
1. User calls: rad checkpoint restore <checkpoint-id>
   │
   ▼
2. CheckpointManager::restore_checkpoint() called
   │
   ▼
3. Lookup checkpoint by ID (via Git tag)
   │
   ▼
4. Get commit hash from checkpoint
   │
   ▼
5. Use git checkout to restore workspace files
   │
   ▼
6. Workspace restored to checkpoint state
   │
   ▼
7. Conversation history preserved (separate from workspace)
```

### Shadow Repository Structure

```
.radium/_internals/checkpoints/
├── HEAD                    # Points to default branch
├── config                  # Git configuration
├── description             # Repository description
├── hooks/                  # Git hooks (empty by default)
├── info/                   # Repository info
│   └── refs                # Additional refs
├── objects/                # Git object database
│   ├── [0-9a-f][0-9a-f]/  # Packed objects
│   └── pack/               # Pack files
├── refs/
│   ├── heads/              # Branch references
│   └── tags/               # Checkpoint tags
│       ├── checkpoint-abc123def456
│       ├── checkpoint-def456ghi789
│       └── ...
└── packed-refs             # Packed references
```

### Checkpoint Tag Format

Checkpoints are stored as Git tags with the format:
```
checkpoint-<uuid>
```

Example: `checkpoint-8803f83d-807b-4e3d-b88b-e24ec1c08242`

The tag points to a commit in the shadow repository that represents the workspace state at that point in time.

## Integration Points

### Workflow Executor Integration

**Location**: `crates/radium-core/src/workflow/executor.rs`

The workflow executor automatically creates checkpoints before file modification steps:

```rust
// Pseudo-code
impl WorkflowExecutor {
    async fn execute_step(&self, step: &WorkflowStep) -> Result<()> {
        // Create checkpoint before file modifications
        if step.modifies_files {
            let checkpoint = self.checkpoint_manager
                .create_checkpoint(Some(format!("Before step: {}", step.id)))?;
        }
        
        // Execute step
        step.execute().await?;
    }
}
```

### Agent Behavior Integration

**Location**: `crates/radium-core/src/workflow/behaviors/checkpoint.rs`

Agents can trigger checkpoints via `behavior.json`:

```rust
pub struct CheckpointEvaluator;

impl CheckpointEvaluator {
    pub fn evaluate_checkpoint(
        &self,
        behavior_file: &Path,
        _output: &str,
    ) -> Result<Option<CheckpointDecision>, BehaviorError> {
        let Some(action) = BehaviorAction::read_from_file(behavior_file)? else {
            return Ok(None);
        };
        
        if action.action != BehaviorActionType::Checkpoint {
            return Ok(None);
        }
        
        Ok(Some(CheckpointDecision {
            should_stop_workflow: true,
            reason: action.reason,
        }))
    }
}
```

### CLI Integration

**Location**: `apps/cli/src/commands/checkpoint.rs`

The CLI provides user-facing commands:

```rust
pub enum CheckpointCommand {
    List { json: bool },
    Restore { checkpoint_id: String },
}
```

### TUI Integration

**Location**: `apps/tui/src/components/checkpoint_modal.rs`

The TUI provides a visual interface for checkpoint management:

- Modal dialog for checkpoint selection
- Visual indicators for restorable checkpoints
- Keyboard navigation
- Checkpoint details display

## Data Flow

### Checkpoint Creation

```
Workspace (main repo)
    │
    ├─> Get current commit hash
    │
    └─> Create tag in shadow repo
        │
        └─> Store metadata
            │
            └─> Return Checkpoint struct
```

### Checkpoint Restoration

```
Shadow Repo (checkpoint tag)
    │
    ├─> Lookup checkpoint by ID
    │
    ├─> Get commit hash
    │
    └─> Restore files to workspace
        │
        └─> Workspace state restored
```

## Error Handling

### CheckpointError

**Location**: `crates/radium-core/src/checkpoint/error.rs`

```rust
pub enum CheckpointError {
    RepositoryNotFound(String),
    ShadowRepoInitFailed(String),
    GitCommandFailed(String),
    CheckpointNotFound(String),
    // ... other variants
}
```

Common error scenarios:
- Workspace is not a Git repository
- Shadow repository initialization fails
- Git commands fail (Git not in PATH, permissions, etc.)
- Checkpoint ID doesn't exist
- File system errors

## Testing

### Unit Tests

**Location**: `crates/radium-core/src/checkpoint/snapshot.rs` (test module)

Unit tests cover:
- CheckpointManager creation and initialization
- Checkpoint creation and metadata
- Checkpoint listing and retrieval
- Checkpoint deletion
- Checkpoint restoration
- Error handling

### Integration Tests

Integration tests (to be added) will cover:
- Automatic checkpoint creation during workflow execution
- Checkpoint restoration with workspace state verification
- Agent-initiated checkpoints via behavior.json
- Error scenarios (missing Git, corrupted checkpoint, etc.)

## Performance Considerations

1. **Shadow Repository Size**: The shadow repository can grow over time. Consider implementing cleanup/garbage collection.

2. **Checkpoint Creation**: Creating a checkpoint involves:
   - Git tag creation (fast)
   - Commit hash lookup (fast)
   - No file copying (uses Git references)

3. **Checkpoint Restoration**: Restoring a checkpoint involves:
   - Git checkout operation (depends on workspace size)
   - File system operations (can be slow for large workspaces)

4. **Listing Checkpoints**: Uses `git tag -l` which is efficient even with many checkpoints.

## Security Considerations

1. **Shadow Repository**: Stored locally, not synced to remote repositories
2. **File Permissions**: Respects workspace file permissions
3. **Git Security**: Relies on Git's security model for repository integrity

## Future Enhancements

Potential improvements:
- Checkpoint cleanup and garbage collection
- Diff visualization between checkpoints
- Checkpoint compression
- Remote checkpoint synchronization
- Checkpoint scheduling
- Advanced filtering and search

## Related Documentation

- [User Guide](../features/checkpointing.md) - End-user documentation
- [Workflow Architecture](./tui-architecture.md) - Workflow execution details
- [Agent System](../developer-guide/agent-system-architecture.md) - Agent behavior system


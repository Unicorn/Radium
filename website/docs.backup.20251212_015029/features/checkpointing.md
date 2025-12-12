# Checkpointing System

The checkpointing system provides a safe way to experiment with code changes and easily rollback to previous working states. It creates Git snapshots of your workspace before modifications, allowing you to restore to any checkpoint with full conversation history preservation.

## Overview

Checkpoints are automatically created during workflow execution before file modifications, and can also be manually triggered by agents. Each checkpoint captures the complete state of your workspace at a specific point in time, stored in a shadow Git repository.

### Key Features

- **Automatic Checkpointing**: Checkpoints are created automatically before workflow steps
- **Manual Checkpointing**: Agents can trigger checkpoints via behavior.json
- **Easy Restoration**: Restore your workspace to any checkpoint with a single command
- **Conversation History**: Full context is preserved across checkpoint restores
- **Shadow Repository**: Checkpoints are stored separately from your main Git repository

## CLI Usage

### Listing Checkpoints

View all available checkpoints:

```bash
rad checkpoint list
```

Output example:
```
ID                                       Commit         Description                    Timestamp          
---------------------------------------------------------------------------------------------------------
checkpoint-abc123def456                  1a2b3c4d5e6f   Before refactoring            2025-12-06 11:30:00
checkpoint-def456ghi789                  7g8h9i0j1k2l   After API changes             2025-12-06 11:35:00
```

Get JSON output for scripting:

```bash
rad checkpoint list --json
```

### Restoring Checkpoints

Restore your workspace to a specific checkpoint:

```bash
rad checkpoint restore checkpoint-abc123def456
```

This will:
1. Restore all files to their state at the checkpoint
2. Preserve conversation history
3. Display a confirmation message

**Note**: After restoration, you may need to re-propose tool calls if you were in the middle of agent execution.

### Example Workflow

```bash
# Start working on a feature
rad craft REQ-001

# Checkpoints are automatically created before each step
# List checkpoints to see what's available
rad checkpoint list

# If something goes wrong, restore to a previous checkpoint
rad checkpoint restore checkpoint-abc123def456

# Continue working from the restored state
```

## TUI Usage

The TUI provides a visual interface for managing checkpoints.

### Opening the Checkpoint Modal

The checkpoint modal can be accessed during workflow execution or from the main TUI interface. It displays:

- **Checkpoint List**: All available checkpoints with visual indicators
- **Details Panel**: Information about the selected checkpoint
- **Help Text**: Available keyboard shortcuts

### Keyboard Shortcuts

- **↑/↓**: Navigate through checkpoint list
- **Enter**: Restore selected checkpoint
- **Esc**: Close the modal

### Visual Indicators

- **✓**: Checkpoint is restorable
- **✗**: Checkpoint cannot be restored (grayed out)

### Checkpoint Details

The details panel shows:
- Checkpoint ID
- Checkpoint name/description
- Step number (if created during workflow)
- Restorable status

## Automatic Checkpointing

Checkpoints are automatically created during workflow execution:

1. **Before Workflow Steps**: A checkpoint is created before each workflow step that modifies files
2. **Workflow Integration**: The checkpoint manager is initialized from the workspace
3. **State Tracking**: Checkpoint metadata includes workflow ID, task ID, and agent ID

### Example: Automatic Checkpoint Flow

```
User: rad craft REQ-001
  → Workflow executor detects file modification step
  → CheckpointManager creates Git snapshot
  → Checkpoint metadata stored with unique ID
  → Workflow step executes
  → User can restore if needed
```

## Manual Checkpointing

Agents can trigger checkpoints manually by writing a checkpoint action to `behavior.json`:

```json
{
  "action": "checkpoint",
  "reason": "Manual review required before proceeding"
}
```

When an agent writes this action:
1. The workflow executor detects the checkpoint behavior
2. A checkpoint is created with the provided reason
3. The workflow pauses for user review
4. The user can restore or continue

### Example: Agent-Initiated Checkpoint

```json
// behavior.json
{
  "action": "checkpoint",
  "reason": "Need to verify database schema changes before migration"
}
```

## Shadow Repository

Checkpoints are stored in a shadow Git repository located at:

```
<workspace-root>/.radium/_internals/checkpoints/
```

This is a bare Git repository that:
- Stores all checkpoint snapshots
- Uses Git tags to identify checkpoints (format: `checkpoint-<uuid>`)
- Is separate from your main Git repository
- Automatically initialized on first checkpoint creation

### Repository Structure

```
.radium/_internals/checkpoints/
├── HEAD              # Current branch reference
├── config            # Git configuration
├── objects/          # Git objects (commits, trees, blobs)
├── refs/
│   ├── heads/        # Branch references
│   └── tags/          # Checkpoint tags (checkpoint-*)
└── ...
```

## Checkpoint Metadata

Each checkpoint includes:

- **ID**: Unique identifier (format: `checkpoint-<uuid>`)
- **Commit Hash**: Git commit hash of the snapshot
- **Agent ID**: ID of the agent that created the checkpoint (if applicable)
- **Timestamp**: Unix timestamp of creation
- **Description**: Optional user-provided description
- **Task ID**: Associated task ID (for recovery)
- **Workflow ID**: Associated workflow ID (for recovery)

## Troubleshooting

### "Workspace is not a git repository"

Checkpoints require your workspace to be a Git repository. Initialize Git first:

```bash
git init
git add .
git commit -m "Initial commit"
```

### "No checkpoints found"

This is normal if:
- No workflows have been executed yet
- No checkpoints have been manually created
- The shadow repository hasn't been initialized

Checkpoints are created automatically during workflow execution.

### "Failed to restore checkpoint"

Possible causes:
- Checkpoint ID doesn't exist (verify with `rad checkpoint list`)
- Shadow repository is corrupted
- Git is not available in PATH

Try:
1. Verify checkpoint exists: `rad checkpoint list`
2. Check Git availability: `git --version`
3. Re-initialize shadow repo (will lose existing checkpoints)

### Shadow Repository Issues

If the shadow repository becomes corrupted:

1. **Backup existing checkpoints** (if needed):
   ```bash
   cp -r .radium/_internals/checkpoints .radium/_internals/checkpoints.backup
   ```

2. **Remove corrupted repository**:
   ```bash
   rm -rf .radium/_internals/checkpoints
   ```

3. **New checkpoints will be created automatically** on next workflow execution

## Best Practices

1. **Regular Checkpoints**: Checkpoints are created automatically, but you can verify they exist with `rad checkpoint list`

2. **Descriptive Checkpoints**: When manually creating checkpoints, use descriptive reasons in behavior.json

3. **Checkpoint Before Major Changes**: Agents automatically create checkpoints, but you can manually trigger them before risky operations

4. **Monitor Shadow Repository**: The shadow repository can grow over time. Consider cleanup if disk space is a concern (cleanup functionality coming soon)

5. **Version Control**: The shadow repository is local-only and not tracked by Git. Don't commit `.radium/_internals/` to your repository

## Related Features

- [Workflow Execution](../user-guide/orchestration.md) - How workflows create checkpoints
- [Agent Behaviors](../developer-guide/agent-system-architecture.md) - How agents trigger checkpoints
- [Checkpoint Architecture](../architecture/checkpoint-system.md) - Technical implementation details

## See Also

- [Architecture Documentation](../architecture/checkpoint-system.md) - For developers
- [CLI Commands](../cli/commands/) - Complete CLI reference
- [TUI Guide](../user-guide/) - Complete TUI documentation


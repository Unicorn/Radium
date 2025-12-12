---
id: "file-editing"
title: "Patch-First File Editing"
sidebar_label: "File Editing"
---

# Patch-First File Editing

Radium provides safe, reviewable, atomic file editing capabilities with workspace boundary validation, transaction support, and policy integration.

## Overview

The file editing system enables Claude-Code-level file editing with:

- **Safe operations**: All file operations are constrained to workspace boundaries
- **Reviewable changes**: Dry-run previews and diff display before applying
- **Atomic operations**: Transaction support for multi-file changes
- **Policy integration**: Approval flow for all file-mutating operations

## Core Concepts

### Workspace Boundary Validation

All file operations are validated to ensure they cannot escape the workspace root. This prevents:
- Path traversal attacks (`../` attempts)
- Symlink escapes
- Access to files outside the workspace

### Patch-First Philosophy

Changes are represented as patches (unified diff format) which:
- Show exactly what will change
- Enable review before application
- Support dry-run previews
- Provide clear conflict detection

### Transaction Support

Multiple file operations can be grouped into transactions:
- **Atomic commits**: All operations succeed or all fail
- **Rollback capability**: Restore previous state on failure
- **Backup management**: Automatic backups for undo

## Tools

### apply_patch

Apply unified diff patches to files with context validation and conflict detection.

**Input Format:**
- Unified diff (standard git diff format)
- Structured hunks (explicit format)

**Features:**
- Context validation
- Conflict detection
- Dry-run preview
- Multi-file support
- Multi-hunk patches

**Example:**
```rust
use radium_core::workspace::{PatchApplicator, PatchInput, PatchContent};

let applicator = PatchApplicator::new(workspace_root)?;
let patch = PatchInput {
    patch: PatchContent::UnifiedDiff {
        content: "--- a/file.txt\n+++ b/file.txt\n@@ -1,1 +1,1 @@\n-old\n+new".to_string(),
    },
    dry_run: false,
    allow_create: true,
    expected_hash: None,
    options: Default::default(),
};

let result = applicator.apply(&patch);
```

### File Operations

#### create_file

Create a new file with content.

```rust
use radium_core::workspace::FileOperations;

let ops = FileOperations::new(workspace_root)?;
let path = ops.create_file("new.txt", "content")?;
```

#### delete_file

Delete a file (with safety checks).

```rust
let path = ops.delete_file("file.txt")?;
```

#### rename_path

Rename or move a file/directory.

```rust
let (old_path, new_path) = ops.rename_path("old.txt", "new.txt")?;
```

#### create_dir

Create a directory (with parent creation).

```rust
let path = ops.create_dir("subdir/nested")?;
```

### Enhanced read_file

Read files with optional line ranges for precise patch targeting.

**Parameters:**
- `file_path`: Path to file (required)
- `start_line`: Optional start line (1-indexed)
- `end_line`: Optional end line (1-indexed, inclusive)

**Example:**
```rust
// Read entire file
read_file(file_path: "file.txt")

// Read line range
read_file(file_path: "file.txt", start_line: 10, end_line: 20)
```

## Transactions

Group multiple operations for atomic execution:

```rust
use radium_core::workspace::FileTransaction;

let mut tx = FileTransaction::new(workspace_root)?;

tx.create_file("file1.txt", "content1")?;
tx.create_file("file2.txt", "content2")?;
tx.write_file("existing.txt", "new content")?;

// Commit all operations atomically
let changed = tx.commit()?;

// Or rollback if needed
tx.rollback()?;
```

## Tool Integration Layer

The `ToolIntegration` layer orchestrates all file operations with:
- Boundary validation
- Policy approval checks
- Transaction management
- Structured results

```rust
use radium_core::workspace::{ToolIntegration, FileOperationRequest};
use radium_core::policy::{PolicyEngine, ApprovalMode};

let engine = PolicyEngine::new(ApprovalMode::Ask)?;
let integration = ToolIntegration::new(workspace_root, engine)?;

let request = FileOperationRequest::CreateFile {
    path: PathBuf::from("new.txt"),
    content: "content".to_string(),
};

let result = integration.request_operation(&request, false).await;
```

## Error Handling

All file operations return structured errors with:
- **Actionable messages**: Clear descriptions of what went wrong
- **Recovery suggestions**: Suggested fixes for common issues
- **Context information**: Affected paths and operation details

**Error Types:**
- `PathNotFound`: File doesn't exist
- `PermissionDenied`: Insufficient permissions
- `AlreadyExists`: File already exists
- `WorkspaceBoundaryViolation`: Path outside workspace
- `PatchConflict`: Context mismatch in patch
- `InvalidInput`: Invalid parameters
- `IoError`: I/O operation failed
- `TransactionFailed`: Transaction rollback

**Example:**
```rust
match ops.create_file("file.txt", "content") {
    Ok(path) => println!("Created: {}", path.display()),
    Err(FileOperationError::AlreadyExists { path, .. }) => {
        println!("File already exists: {}", path);
    }
    Err(e) => {
        if let Some(suggestion) = e.suggest_fix() {
            println!("Error: {}\nSuggestion: {}", e, suggestion);
        }
    }
}
```

## Policy Integration

All file-mutating operations go through policy approval:

- **Allow**: Operation proceeds automatically
- **Deny**: Operation is blocked
- **AskUser**: User approval required (default for destructive operations)
- **DryRunFirst**: Preview changes before approval

Policy rules can be configured in `.radium/policy.toml`:

```toml
[[rules]]
name = "Require approval for file deletions"
priority = "user"
action = "ask_user"
tool_pattern = "delete_file"
reason = "File deletions are destructive"
```

## Diff Display

### CLI

Use `format_patch_result_for_cli` to display patch results:

```rust
use radium_core::workspace::format_patch_result_for_cli;

let formatted = format_patch_result_for_cli(&result);
println!("{}", formatted);
```

### TUI

Use `format_patch_result_for_tui` for TUI display:

```rust
use radium_core::workspace::format_patch_result_for_tui;

let lines = format_patch_result_for_tui(&result);
// Render in TUI component
```

## Best Practices

1. **Always use dry-run first**: Preview changes before applying
2. **Use transactions for multi-file changes**: Ensure atomicity
3. **Check policy before operations**: Understand approval requirements
4. **Handle errors gracefully**: Use error suggestions for recovery
5. **Validate paths early**: Check boundary validation before complex operations

## Examples

### Simple File Edit

```rust
let ops = FileOperations::new(workspace_root)?;
ops.create_file("config.toml", "[settings]\nkey = \"value\"")?;
```

### Multi-File Refactoring

```rust
let mut tx = FileTransaction::new(workspace_root)?;

// Rename file
tx.rename_file("old.rs", "new.rs")?;

// Update references in other files
tx.write_file("main.rs", updated_content)?;

// Commit atomically
tx.commit()?;
```

### Safe Patch Application

```rust
let applicator = PatchApplicator::new(workspace_root)?;

// First, dry-run to preview
let mut patch = PatchInput { /* ... */ };
patch.dry_run = true;
let preview = applicator.apply(&patch);

// Review the preview
println!("{}", format_patch_result_for_cli(&preview));

// If approved, apply for real
patch.dry_run = false;
let result = applicator.apply(&patch);
```

## Security Considerations

- All paths are validated against workspace boundaries
- Symlink escapes are detected and prevented
- Policy engine gates all file-mutating operations
- Transactions ensure atomicity (no partial state)

## Performance

- Boundary validation: < 1ms per path
- Patch parsing: O(n) where n is patch size
- Transaction commit: O(m) where m is number of operations
- Dry-run: Same performance as apply (no I/O overhead)

## See Also

- [Policy Engine](./policy-engine.md) - Policy configuration
- [Workspace Management](../cli/commands/workspace.md) - Workspace operations
- [Error Handling](../cli/error-handling.md) - Error handling patterns

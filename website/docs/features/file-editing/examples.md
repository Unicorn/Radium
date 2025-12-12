---
id: "file-editing-examples"
title: "File Editing Examples"
sidebar_label: "Examples"
parent_id: "file-editing"
---

# File Editing Examples

Practical examples for common file editing workflows.

## Basic File Operations

### Create a New File

```rust
use radium_core::workspace::FileOperations;

let ops = FileOperations::new("/workspace")?;
let path = ops.create_file("README.md", "# My Project\n\nDescription")?;
println!("Created: {}", path.display());
```

### Delete a File

```rust
let path = ops.delete_file("temp.txt")?;
println!("Deleted: {}", path.display());
```

### Rename a File

```rust
let (old_path, new_path) = ops.rename_path("old_name.rs", "new_name.rs")?;
println!("Renamed: {} -> {}", old_path.display(), new_path.display());
```

### Create Directory Structure

```rust
let path = ops.create_dir("src/components/ui")?;
println!("Created directory: {}", path.display());
```

## Patch Application

### Apply a Simple Patch

```rust
use radium_core::workspace::{PatchApplicator, PatchInput, PatchContent};

let applicator = PatchApplicator::new("/workspace")?;

let patch = PatchInput {
    patch: PatchContent::UnifiedDiff {
        content: r#"--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,3 @@
 fn main() {
-    println!("Hello");
+    println!("Hello, World!");
 }
"#.to_string(),
    },
    dry_run: false,
    allow_create: true,
    expected_hash: None,
    options: Default::default(),
};

let result = applicator.apply(&patch);
if result.success {
    println!("Patch applied successfully");
    for file in result.changed_files {
        println!("  Modified: {}", file.path.display());
    }
}
```

### Dry-Run Preview

```rust
let mut patch = PatchInput {
    patch: PatchContent::UnifiedDiff {
        content: diff_content,
    },
    dry_run: true,  // Preview only
    allow_create: true,
    expected_hash: None,
    options: Default::default(),
};

let preview = applicator.apply(&patch);
println!("{}", format_patch_result_for_cli(&preview));

// User reviews preview, then applies if approved
patch.dry_run = false;
let result = applicator.apply(&patch);
```

### Multi-File Patch

```rust
let multi_file_patch = PatchInput {
    patch: PatchContent::UnifiedDiff {
        content: r#"--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,1 +1,1 @@
-pub fn old_name() {}
+pub fn new_name() {}
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,1 +1,1 @@
-    old_name();
+    new_name();
"#.to_string(),
    },
    dry_run: false,
    allow_create: true,
    expected_hash: None,
    options: Default::default(),
};

let result = applicator.apply(&multi_file_patch);
assert_eq!(result.changed_files.len(), 2);
```

## Transactions

### Atomic Multi-File Operation

```rust
use radium_core::workspace::FileTransaction;

let mut tx = FileTransaction::new("/workspace")?;

// Add multiple operations
tx.create_file("src/lib.rs", "pub fn hello() {}")?;
tx.create_file("src/main.rs", "fn main() { hello(); }")?;
tx.create_file("Cargo.toml", "[package]\nname = \"my-project\"")?;

// Commit all atomically
match tx.commit() {
    Ok(changed) => {
        println!("Created {} files", changed.len());
    }
    Err(e) => {
        println!("Transaction failed: {}", e);
        // All changes are automatically rolled back
    }
}
```

### Rollback on Error

```rust
let mut tx = FileTransaction::new("/workspace")?;

// Create a file
tx.create_file("file1.txt", "content1")?;

// Try to create existing file (will fail)
if tx.create_file("file1.txt", "content2").is_err() {
    // Rollback the entire transaction
    tx.rollback()?;
    // file1.txt will not exist
}
```

## Error Handling

### Handle Common Errors

```rust
use radium_core::workspace::{FileOperations, FileOperationError};

let ops = FileOperations::new("/workspace")?;

match ops.create_file("file.txt", "content") {
    Ok(path) => println!("Created: {}", path.display()),
    
    Err(FileOperationError::AlreadyExists { path, .. }) => {
        println!("File already exists: {}", path);
        // Optionally overwrite or use different name
    }
    
    Err(FileOperationError::WorkspaceBoundaryViolation { path, reason, .. }) => {
        println!("Path outside workspace: {} ({})", path, reason);
    }
    
    Err(FileOperationError::PermissionDenied { path, .. }) => {
        println!("Permission denied: {}", path);
        if let Some(suggestion) = error.suggest_fix() {
            println!("Suggestion: {}", suggestion);
        }
    }
    
    Err(e) => {
        println!("Error: {}", e);
        if let Some(suggestion) = e.suggest_fix() {
            println!("Suggestion: {}", suggestion);
        }
    }
}
```

### Recovery Strategies

```rust
use radium_core::workspace::{FileOperationError, ErrorRecovery};

let error = FileOperationError::PathNotFound {
    path: "/workspace/missing.txt".to_string(),
    operation: "read_file".to_string(),
};

match error.recovery_strategy() {
    RecoveryStrategy::Retry => {
        // Retry the operation
    }
    RecoveryStrategy::UserInput(prompt) => {
        println!("{}", prompt);
        // Get user input
    }
    RecoveryStrategy::Abort => {
        // Abort the operation
    }
    RecoveryStrategy::Skip => {
        // Skip this operation
    }
}
```

## Policy Integration

### Check Approval Before Operation

```rust
use radium_core::workspace::{ToolIntegration, FileOperationRequest};
use radium_core::policy::{PolicyEngine, ApprovalMode};

let engine = PolicyEngine::new(ApprovalMode::Ask)?;
let integration = ToolIntegration::new("/workspace", engine)?;

let request = FileOperationRequest::DeleteFile {
    path: PathBuf::from("important.txt"),
};

// Check if approval is required
let result = integration.request_operation(&request, true).await; // dry-run
if result.success {
    // Show preview and get user approval
    println!("Preview: {}", format_integration_result_for_cli(&result));
    
    // If approved, execute
    let final_result = integration.request_operation(&request, false).await;
}
```

## Reading Files with Line Ranges

### Read Specific Lines

```rust
// Read lines 10-20
let content = read_file(
    file_path: "src/main.rs",
    start_line: 10,
    end_line: 20
)?;

// Use for precise patch targeting
let patch = create_patch_for_lines(content, 10, 20);
```

## Common Workflows

### Refactoring: Rename Function Across Files

```rust
let mut tx = FileTransaction::new("/workspace")?;

// 1. Read files to find references
let main_content = read_file("src/main.rs")?;
let lib_content = read_file("src/lib.rs")?;

// 2. Create patches for each file
let main_patch = create_rename_patch(&main_content, "old_func", "new_func");
let lib_patch = create_rename_patch(&lib_content, "old_func", "new_func");

// 3. Apply patches in transaction
let applicator = PatchApplicator::new("/workspace")?;
let main_result = applicator.apply(&main_patch);
let lib_result = applicator.apply(&lib_patch);

// 4. Commit all changes atomically
if main_result.success && lib_result.success {
    tx.commit()?;
} else {
    tx.rollback()?;
}
```

### Safe File Deletion with Backup

```rust
use std::fs;

// Create backup before deletion
let file_content = fs::read_to_string("config.toml")?;
fs::write("config.toml.backup", &file_content)?;

// Delete original
let ops = FileOperations::new("/workspace")?;
ops.delete_file("config.toml")?;

// Can restore from backup if needed
```

### Batch File Creation

```rust
let mut tx = FileTransaction::new("/workspace")?;

let files = vec![
    ("src/main.rs", "fn main() {}"),
    ("src/lib.rs", "pub fn hello() {}"),
    ("Cargo.toml", "[package]\nname = \"project\""),
];

for (path, content) in files {
    tx.create_file(path, content)?;
}

tx.commit()?;
```

## Advanced: Custom Patch Generation

### Generate Patch from Changes

```rust
use radium_core::workspace::patch::{FilePatch, Hunk};

let old_content = "line 1\nline 2\nline 3";
let new_content = "line 1\nline 2 modified\nline 3";

// Create structured hunk
let hunk = Hunk {
    old_start: 2,
    old_count: 1,
    new_start: 2,
    new_count: 1,
    context_before: vec!["line 1".to_string()],
    removed_lines: vec!["line 2".to_string()],
    added_lines: vec!["line 2 modified".to_string()],
    context_after: vec!["line 3".to_string()],
};

let file_patch = FilePatch {
    path: "file.txt".to_string(),
    hunks: vec![hunk],
};

let patch = PatchInput {
    patch: PatchContent::StructuredHunks {
        files: vec![file_patch],
    },
    dry_run: false,
    allow_create: true,
    expected_hash: None,
    options: Default::default(),
};
```

## Integration with CLI

### CLI Command Example

```rust
// In CLI command handler
use radium_core::workspace::{format_patch_result_for_cli, PatchApplicator, PatchInput};

let applicator = PatchApplicator::new(workspace_root)?;
let patch = parse_patch_from_args(args)?;

let result = applicator.apply(&patch);
println!("{}", format_patch_result_for_cli(&result));

if !result.success {
    std::process::exit(1);
}
```

## See Also

- [File Editing Overview](./file-editing.md) - Complete feature documentation
- [Policy Engine](../policy-engine.md) - Policy configuration
- [Error Handling](../../cli/error-handling.md) - Error handling patterns

---
id: "file-editing-api"
title: "File Editing API Reference"
sidebar_label: "API Reference"
parent_id: "file-editing"
---

# File Editing API Reference

Complete API reference for file editing tools.

## Modules

### `radium_core::workspace::boundary`

Workspace boundary validation.

#### `BoundaryValidator`

```rust
pub struct BoundaryValidator {
    workspace_root: PathBuf,
}

impl BoundaryValidator {
    pub fn new(workspace_root: impl AsRef<Path>) -> Result<Self>;
    pub fn validate_path(&self, path: impl AsRef<Path>, allow_absolute: bool) -> Result<PathBuf>;
    pub fn validate_paths<I, P>(&self, paths: I, allow_absolute: bool) -> Result<Vec<PathBuf>>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>;
    pub fn workspace_root(&self) -> &Path;
    pub fn is_unsafe_path(path: &str) -> bool;
}
```

### `radium_core::workspace::file_ops`

File mutation operations.

#### `FileOperations`

```rust
pub struct FileOperations {
    workspace_root: PathBuf,
    boundary_validator: BoundaryValidator,
}

impl FileOperations {
    pub fn new(workspace_root: impl AsRef<Path>) -> FileOperationResult<Self>;
    pub fn create_file(&self, path: impl AsRef<Path>, content: &str) -> FileOperationResult<PathBuf>;
    pub fn delete_file(&self, path: impl AsRef<Path>) -> FileOperationResult<PathBuf>;
    pub fn rename_path(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> FileOperationResult<(PathBuf, PathBuf)>;
    pub fn create_dir(&self, path: impl AsRef<Path>) -> FileOperationResult<PathBuf>;
    pub fn workspace_root(&self) -> &Path;
}
```

### `radium_core::workspace::patch`

Patch application.

#### `PatchApplicator`

```rust
pub struct PatchApplicator {
    workspace_root: PathBuf,
    boundary_validator: BoundaryValidator,
}

impl PatchApplicator {
    pub fn new(workspace_root: impl AsRef<Path>) -> FileOperationResult<Self>;
    pub fn apply(&self, input: &PatchInput) -> PatchResult;
}
```

#### `PatchInput`

```rust
pub struct PatchInput {
    pub patch: PatchContent,
    pub dry_run: bool,
    pub allow_create: bool,
    pub expected_hash: Option<String>,
    pub options: PatchOptions,
}
```

#### `PatchContent`

```rust
pub enum PatchContent {
    UnifiedDiff { content: String },
    StructuredHunks { files: Vec<FilePatch> },
}
```

#### `PatchResult`

```rust
pub struct PatchResult {
    pub success: bool,
    pub changed_files: Vec<ChangedFile>,
    pub errors: Vec<FileOperationError>,
    pub summary: PatchSummary,
}
```

### `radium_core::workspace::transaction`

Transaction support.

#### `FileTransaction`

```rust
pub struct FileTransaction {
    file_ops: FileOperations,
    operations: Vec<FileOperation>,
    backups: HashMap<PathBuf, String>,
}

impl FileTransaction {
    pub fn new(workspace_root: impl AsRef<Path>) -> FileOperationResult<Self>;
    pub fn create_file(&mut self, path: impl AsRef<Path>, content: &str) -> FileOperationResult<()>;
    pub fn delete_file(&mut self, path: impl AsRef<Path>) -> FileOperationResult<()>;
    pub fn rename_file(&mut self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> FileOperationResult<()>;
    pub fn write_file(&mut self, path: impl AsRef<Path>, content: &str) -> FileOperationResult<()>;
    pub fn commit(self) -> FileOperationResult<Vec<PathBuf>>;
    pub fn rollback(&mut self) -> FileOperationResult<()>;
    pub fn operation_count(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

### `radium_core::workspace::tool_integration`

Tool integration layer.

#### `ToolIntegration`

```rust
pub struct ToolIntegration {
    workspace_root: PathBuf,
    boundary_validator: BoundaryValidator,
    file_ops: FileOperations,
    patch_applicator: PatchApplicator,
    approval_flow: ApprovalFlow,
}

impl ToolIntegration {
    pub fn new(workspace_root: impl AsRef<Path>, policy_engine: PolicyEngine) -> FileOperationResult<Self>;
    pub async fn request_operation(&self, request: &FileOperationRequest, dry_run: bool) -> IntegrationResult;
    pub fn begin_transaction(&self) -> FileOperationResult<FileTransaction>;
}
```

#### `FileOperationRequest`

```rust
pub enum FileOperationRequest {
    ApplyPatch { input: PatchInput },
    CreateFile { path: PathBuf, content: String },
    DeleteFile { path: PathBuf },
    RenameFile { from: PathBuf, to: PathBuf },
    CreateDir { path: PathBuf },
}
```

### `radium_core::workspace::errors`

Error handling.

#### `FileOperationError`

```rust
pub enum FileOperationError {
    PathNotFound { path: String, operation: String },
    PermissionDenied { path: String, operation: String, required_permission: String },
    AlreadyExists { path: String, operation: String },
    WorkspaceBoundaryViolation { path: String, workspace_root: String, reason: String },
    PatchConflict { file: String, line_number: usize, expected: String, actual: String },
    InvalidInput { operation: String, field: String, reason: String },
    IoError { path: String, operation: String, source: std::io::Error },
    TransactionFailed { operations_attempted: usize, failed_at: String, reason: String },
}

impl FileOperationError {
    pub fn with_context(self, context: ErrorContext) -> Self;
    pub fn suggest_fix(&self) -> Option<String>;
    pub fn is_recoverable(&self) -> bool;
    pub fn recovery_strategy(&self) -> RecoveryStrategy;
    pub fn affected_paths(&self) -> Vec<PathBuf>;
}
```

#### `RecoveryStrategy`

```rust
pub enum RecoveryStrategy {
    Retry,
    Skip,
    Abort,
    UserInput(String),
}
```

### `radium_core::workspace::diff_display`

Diff display utilities.

#### Functions

```rust
pub fn format_patch_result_for_cli(result: &PatchResult) -> String;
pub fn format_integration_result_for_cli(result: &IntegrationResult) -> String;
pub fn format_patch_result_for_tui(result: &PatchResult) -> Vec<String>;
```

## Error Types

### `BoundaryError`

```rust
pub enum BoundaryError {
    OutsideBoundary { path: String, root: String },
    PathTraversal(String),
    AbsolutePath(String),
    SymlinkEscape { path: String, resolved: String },
    Io(#[from] std::io::Error),
    CanonicalizationFailed(String),
}
```

## Type Aliases

```rust
pub type FileOperationResult<T> = std::result::Result<T, FileOperationError>;
pub type Result<T> = std::result::Result<T, BoundaryError>; // in boundary module
```

## See Also

- [File Editing Overview](./file-editing.md) - Feature overview
- [File Editing Examples](./examples.md) - Usage examples

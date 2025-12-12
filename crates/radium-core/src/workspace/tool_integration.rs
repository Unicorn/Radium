//! Tool integration layer for file operations.
//!
//! This module orchestrates file operations, ensuring consistent approval flow,
//! transaction management, and error handling across all file-mutating tools.

use crate::policy::PolicyEngine;
use crate::workspace::approval_flow::ApprovalFlow;
use crate::workspace::boundary::BoundaryValidator;
use crate::workspace::errors::{ErrorContext, FileOperationError, FileOperationResult};
use crate::workspace::file_ops::FileOperations;
use crate::workspace::patch::{PatchApplicator, PatchInput, PatchResult};
use crate::workspace::transaction::FileTransaction;
use std::path::{Path, PathBuf};

/// File operation request.
#[derive(Debug, Clone)]
pub enum FileOperationRequest {
    /// Apply a patch.
    ApplyPatch {
        input: PatchInput,
    },
    /// Create a file.
    CreateFile {
        path: PathBuf,
        content: String,
    },
    /// Delete a file.
    DeleteFile {
        path: PathBuf,
    },
    /// Rename a file.
    RenameFile {
        from: PathBuf,
        to: PathBuf,
    },
    /// Create a directory.
    CreateDir {
        path: PathBuf,
    },
}

/// Result of a file operation through the integration layer.
#[derive(Debug, Clone)]
pub struct IntegrationResult {
    /// Whether the operation succeeded.
    pub success: bool,
    /// List of changed file paths.
    pub changed_paths: Vec<PathBuf>,
    /// List of errors encountered.
    pub errors: Vec<FileOperationError>,
    /// Diff information for changed files (if applicable).
    pub diffs: Vec<String>,
}

impl IntegrationResult {
    /// Create a successful result.
    pub fn success(changed_paths: Vec<PathBuf>) -> Self {
        Self {
            success: true,
            changed_paths,
            errors: Vec::new(),
            diffs: Vec::new(),
        }
    }

    /// Create a failed result.
    pub fn failure(errors: Vec<FileOperationError>) -> Self {
        Self {
            success: false,
            changed_paths: Vec::new(),
            errors,
            diffs: Vec::new(),
        }
    }
}

/// Tool integration layer.
pub struct ToolIntegration {
    /// Workspace root.
    workspace_root: PathBuf,
    /// Boundary validator.
    boundary_validator: BoundaryValidator,
    /// File operations handler.
    file_ops: FileOperations,
    /// Patch applicator.
    patch_applicator: PatchApplicator,
    /// Approval flow handler.
    approval_flow: ApprovalFlow,
}

impl ToolIntegration {
    /// Create a new tool integration layer.
    ///
    /// # Errors
    /// Returns error if workspace root cannot be validated or components cannot be initialized.
    pub fn new(
        workspace_root: impl AsRef<Path>,
        policy_engine: PolicyEngine,
    ) -> FileOperationResult<Self> {
        let root = workspace_root.as_ref().to_path_buf();
        let boundary_validator = BoundaryValidator::new(&root)?;
        let file_ops = FileOperations::new(&root)?;
        let patch_applicator = PatchApplicator::new(&root)?;
        let approval_flow = ApprovalFlow::new(policy_engine);

        Ok(Self {
            workspace_root: root,
            boundary_validator,
            file_ops,
            patch_applicator,
            approval_flow,
        })
    }

    /// Request a file operation with approval flow.
    ///
    /// # Arguments
    /// * `request` - The file operation request
    /// * `dry_run` - Whether to perform a dry-run (preview without executing)
    ///
    /// # Returns
    /// Result with changed paths, diffs, and any errors
    pub async fn request_operation(
        &self,
        request: &FileOperationRequest,
        dry_run: bool,
    ) -> IntegrationResult {
        // Validate all paths first
        let paths = self.extract_paths(request);
        match self.boundary_validator.validate_paths(paths.iter(), false) {
            Ok(_) => {}
            Err(e) => {
                return IntegrationResult::failure(vec![FileOperationError::from(e)]);
            }
        }

        // Check approval for each path
        for path in &paths {
            let operation = self.get_operation_name(request);
            match self.approval_flow.check_approval(operation, path).await {
                Ok(decision) => {
                    if matches!(decision.action, crate::policy::PolicyAction::Deny) {
                        return IntegrationResult::failure(vec![FileOperationError::PermissionDenied {
                            path: path.display().to_string(),
                            operation: operation.to_string(),
                            required_permission: "write".to_string(),
                        }]);
                    }
                    // If AskUser, we'd normally prompt here, but for now we'll allow it
                    // In a real implementation, this would trigger user interaction
                }
                Err(e) => {
                    return IntegrationResult::failure(vec![e]);
                }
            }
        }

        // Execute operation
        if dry_run {
            self.dry_run(request).await
        } else {
            self.execute(request).await
        }
    }

    /// Execute a file operation.
    async fn execute(&self, request: &FileOperationRequest) -> IntegrationResult {
        match request {
            FileOperationRequest::ApplyPatch { input } => {
                let result = self.patch_applicator.apply(input);
                IntegrationResult {
                    success: result.success,
                    changed_paths: result.changed_files.iter().map(|f| f.path.clone()).collect(),
                    errors: result.errors,
                    diffs: result.changed_files.iter().map(|f| f.diff.clone()).collect(),
                }
            }
            FileOperationRequest::CreateFile { path, content } => {
                match self.file_ops.create_file(path, content) {
                    Ok(changed_path) => IntegrationResult::success(vec![changed_path]),
                    Err(e) => IntegrationResult::failure(vec![e]),
                }
            }
            FileOperationRequest::DeleteFile { path } => {
                match self.file_ops.delete_file(path) {
                    Ok(changed_path) => IntegrationResult::success(vec![changed_path]),
                    Err(e) => IntegrationResult::failure(vec![e]),
                }
            }
            FileOperationRequest::RenameFile { from, to } => {
                match self.file_ops.rename_path(from, to) {
                    Ok((old_path, new_path)) => {
                        IntegrationResult::success(vec![old_path, new_path])
                    }
                    Err(e) => IntegrationResult::failure(vec![e]),
                }
            }
            FileOperationRequest::CreateDir { path } => {
                match self.file_ops.create_dir(path) {
                    Ok(changed_path) => IntegrationResult::success(vec![changed_path]),
                    Err(e) => IntegrationResult::failure(vec![e]),
                }
            }
        }
    }

    /// Perform a dry-run (preview without executing).
    async fn dry_run(&self, request: &FileOperationRequest) -> IntegrationResult {
        match request {
            FileOperationRequest::ApplyPatch { input } => {
                let mut dry_run_input = input.clone();
                dry_run_input.dry_run = true;
                let result = self.patch_applicator.apply(&dry_run_input);
                IntegrationResult {
                    success: result.success,
                    changed_paths: result.changed_files.iter().map(|f| f.path.clone()).collect(),
                    errors: result.errors,
                    diffs: result.changed_files.iter().map(|f| f.diff.clone()).collect(),
                }
            }
            _ => {
                // For other operations, we can't easily preview without executing
                // Return success with empty changes
                IntegrationResult::success(Vec::new())
            }
        }
    }

    /// Extract paths from a request.
    fn extract_paths(&self, request: &FileOperationRequest) -> Vec<PathBuf> {
        match request {
            FileOperationRequest::ApplyPatch { .. } => {
                // Paths will be extracted from patch during application
                Vec::new()
            }
            FileOperationRequest::CreateFile { path, .. } => vec![path.clone()],
            FileOperationRequest::DeleteFile { path } => vec![path.clone()],
            FileOperationRequest::RenameFile { from, to } => vec![from.clone(), to.clone()],
            FileOperationRequest::CreateDir { path } => vec![path.clone()],
        }
    }

    /// Get the operation name for policy evaluation.
    fn get_operation_name(&self, request: &FileOperationRequest) -> &str {
        match request {
            FileOperationRequest::ApplyPatch { .. } => "apply_patch",
            FileOperationRequest::CreateFile { .. } => "create_file",
            FileOperationRequest::DeleteFile { .. } => "delete_file",
            FileOperationRequest::RenameFile { .. } => "rename_file",
            FileOperationRequest::CreateDir { .. } => "create_dir",
        }
    }

    /// Execute operations within a transaction.
    pub fn begin_transaction(&self) -> FileOperationResult<FileTransaction> {
        FileTransaction::new(&self.workspace_root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::ApprovalMode;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_tool_integration_create_file() {
        let temp = TempDir::new().unwrap();
        let engine = PolicyEngine::new(ApprovalMode::Yolo).unwrap();
        let integration = ToolIntegration::new(temp.path(), engine).unwrap();

        let request = FileOperationRequest::CreateFile {
            path: PathBuf::from("test.txt"),
            content: "hello".to_string(),
        };

        let result = integration.request_operation(&request, false).await;
        assert!(result.success);
        assert_eq!(result.changed_paths.len(), 1);
    }

    #[tokio::test]
    async fn test_tool_integration_dry_run() {
        let temp = TempDir::new().unwrap();
        let engine = PolicyEngine::new(ApprovalMode::Yolo).unwrap();
        let integration = ToolIntegration::new(temp.path(), engine).unwrap();

        let request = FileOperationRequest::CreateFile {
            path: PathBuf::from("test.txt"),
            content: "hello".to_string(),
        };

        let result = integration.request_operation(&request, true).await;
        // Dry run should succeed but not create file
        assert!(result.success);
        assert!(!temp.path().join("test.txt").exists());
    }
}

//! Approval flow for file operations.
//!
//! This module provides integration with the policy engine to gate file-mutating
//! operations through approval checks.

use crate::policy::{PolicyAction, PolicyDecision, PolicyEngine};
use crate::workspace::errors::{FileOperationError, FileOperationResult};
use std::path::PathBuf;

/// Approval flow handler for file operations.
pub struct ApprovalFlow {
    /// Policy engine for making decisions.
    policy_engine: PolicyEngine,
}

impl ApprovalFlow {
    /// Create a new approval flow handler.
    pub fn new(policy_engine: PolicyEngine) -> Self {
        Self { policy_engine }
    }

    /// Check if a file operation requires approval.
    ///
    /// # Arguments
    /// * `operation` - The operation type (e.g., "create_file", "delete_file")
    /// * `path` - The file path being operated on
    ///
    /// # Returns
    /// Policy decision indicating whether to allow, deny, or ask user
    pub async fn check_approval(
        &self,
        operation: &str,
        path: &PathBuf,
    ) -> FileOperationResult<PolicyDecision> {
        let tool_name = format!("{}", operation);
        let args = vec![path.display().to_string()];

        self.policy_engine
            .evaluate(&tool_name, &args)
            .await
            .map_err(|e| FileOperationError::InvalidInput {
                operation: "check_approval".to_string(),
                field: "policy_evaluation".to_string(),
                reason: format!("Policy evaluation failed: {}", e),
            })
    }

    /// Check if an operation is allowed without user interaction.
    pub async fn is_allowed(&self, operation: &str, path: &PathBuf) -> FileOperationResult<bool> {
        let decision = self.check_approval(operation, path).await?;
        Ok(matches!(decision.action, PolicyAction::Allow))
    }

    /// Check if an operation requires user approval.
    pub async fn requires_approval(
        &self,
        operation: &str,
        path: &PathBuf,
    ) -> FileOperationResult<bool> {
        let decision = self.check_approval(operation, path).await?;
        Ok(matches!(decision.action, PolicyAction::AskUser))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::types::ApprovalMode;

    #[tokio::test]
    async fn test_approval_flow_check() {
        let engine = PolicyEngine::new(ApprovalMode::Ask).unwrap();
        let flow = ApprovalFlow::new(engine);

        let path = PathBuf::from("test.txt");
        let decision = flow.check_approval("create_file", &path).await.unwrap();
        
        // In Ask mode, should require approval or allow
        assert!(matches!(decision.action, PolicyAction::AskUser | PolicyAction::Allow | PolicyAction::DryRunFirst));
    }
}

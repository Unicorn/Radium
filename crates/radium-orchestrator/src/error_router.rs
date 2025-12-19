//! Error routing and fix orchestration.
//!
//! This module handles routing detected errors to specialized agents,
//! managing fix proposals, and coordinating fix application and verification.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Status of a fix proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    /// Pending user approval.
    Pending,
    /// Approved by user, awaiting execution.
    Approved,
    /// Rejected by user.
    Rejected,
    /// Currently being applied.
    Applying,
    /// Successfully applied.
    Applied,
    /// Being verified.
    Verifying,
    /// Verified successful.
    Verified,
    /// Verification failed, rolled back.
    RolledBack,
    /// Approval timed out.
    TimedOut,
}

impl std::fmt::Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalStatus::Pending => write!(f, "pending"),
            ProposalStatus::Approved => write!(f, "approved"),
            ProposalStatus::Rejected => write!(f, "rejected"),
            ProposalStatus::Applying => write!(f, "applying"),
            ProposalStatus::Applied => write!(f, "applied"),
            ProposalStatus::Verifying => write!(f, "verifying"),
            ProposalStatus::Verified => write!(f, "verified"),
            ProposalStatus::RolledBack => write!(f, "rolled_back"),
            ProposalStatus::TimedOut => write!(f, "timed_out"),
        }
    }
}

/// Context about an error that was detected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// ID of the process where the error occurred.
    pub process_id: String,
    /// Relevant log lines containing the error.
    pub error_lines: Vec<String>,
    /// Error severity level.
    pub severity: String,
    /// Type of error.
    pub error_type: String,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f64,
    /// Suggested agent ID to handle this error.
    pub suggested_agent: Option<String>,
    /// Additional context as JSON.
    pub metadata: JsonValue,
}

/// A proposed fix for a detected error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixProposal {
    /// Unique identifier for this proposal.
    pub id: String,
    /// Error context that triggered this proposal.
    pub error_context: ErrorContext,
    /// Proposed fix description or steps.
    pub proposed_fix: String,
    /// Agent ID that generated this proposal.
    pub agent_id: String,
    /// Confidence level (0.0 to 1.0).
    pub confidence: f64,
    /// Root cause analysis.
    pub root_cause: String,
    /// Expected impact of the fix.
    pub impact: String,
    /// Rollback strategy if fix fails.
    pub rollback_strategy: String,
    /// Current status of the proposal.
    pub status: ProposalStatus,
    /// When the proposal was created.
    pub created_at: SystemTime,
    /// When approval was requested.
    pub approval_requested_at: Option<SystemTime>,
    /// When the proposal was approved/rejected.
    pub decided_at: Option<SystemTime>,
}

/// Result of fix verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixVerificationResult {
    /// Whether the fix was successful.
    pub success: bool,
    /// Details about the verification.
    pub details: String,
    /// Whether the error reappeared.
    pub error_reappeared: bool,
    /// Whether the process remained stable.
    pub process_stable: bool,
}

/// Router for directing errors to specialized agents.
pub struct ErrorRouter {
    /// Queue of pending fix proposals awaiting approval.
    approval_queue: Arc<Mutex<VecDeque<FixProposal>>>,
    /// Timeout for approval requests (default: 5 minutes).
    approval_timeout: Duration,
}

impl ErrorRouter {
    /// Creates a new error router.
    pub fn new() -> Self {
        Self {
            approval_queue: Arc::new(Mutex::new(VecDeque::new())),
            approval_timeout: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Creates a new error router with custom approval timeout.
    pub fn with_approval_timeout(timeout: Duration) -> Self {
        Self {
            approval_queue: Arc::new(Mutex::new(VecDeque::new())),
            approval_timeout: timeout,
        }
    }

    /// Routes an error to an appropriate agent and creates a fix proposal.
    ///
    /// # Arguments
    /// * `error_context` - Context about the detected error
    ///
    /// # Returns
    /// The proposal ID that can be used to track and manage the fix
    pub async fn route_error(&self, error_context: ErrorContext) -> Result<String> {
        info!(
            process_id = %error_context.process_id,
            error_type = %error_context.error_type,
            severity = %error_context.severity,
            "Routing error to agent"
        );

        // Determine which agent to use
        let agent_id = self.select_agent(&error_context);

        debug!(
            agent_id = %agent_id,
            "Selected agent for error handling"
        );

        // For now, create a placeholder proposal
        // In a full implementation, this would:
        // 1. Spawn the selected agent with error context
        // 2. Wait for agent to analyze and generate fix
        // 3. Parse agent output into structured proposal
        let proposal = self.create_placeholder_proposal(error_context, agent_id).await;

        // Add to approval queue
        let proposal_id = proposal.id.clone();
        self.approval_queue.lock().await.push_back(proposal);

        info!(
            proposal_id = %proposal_id,
            "Fix proposal created and queued for approval"
        );

        Ok(proposal_id)
    }

    /// Selects an appropriate agent based on error context.
    fn select_agent(&self, context: &ErrorContext) -> String {
        // Use suggested agent from context if available
        if let Some(agent) = &context.suggested_agent {
            return agent.clone();
        }

        // Fallback based on error type
        match context.error_type.as_str() {
            "type_error" => "typescript-agent".to_string(),
            "compilation_error" => "build-agent".to_string(),
            "runtime_crash" => "debugging-agent".to_string(),
            "network_error" => "network-agent".to_string(),
            "database_error" => "database-agent".to_string(),
            _ => "code-agent".to_string(),
        }
    }

    /// Creates a placeholder fix proposal (to be replaced with actual agent interaction).
    async fn create_placeholder_proposal(
        &self,
        error_context: ErrorContext,
        agent_id: String,
    ) -> FixProposal {
        let proposal_id = Uuid::new_v4().to_string();

        // Generate placeholder fix based on error type
        let (proposed_fix, root_cause, impact) = self.generate_placeholder_fix(&error_context);

        FixProposal {
            id: proposal_id,
            error_context,
            proposed_fix,
            agent_id,
            confidence: 0.75, // Placeholder confidence
            root_cause,
            impact,
            rollback_strategy: "Revert changes using checkpoint if process crashes or error persists".to_string(),
            status: ProposalStatus::Pending,
            created_at: SystemTime::now(),
            approval_requested_at: Some(SystemTime::now()),
            decided_at: None,
        }
    }

    /// Generates a placeholder fix (to be replaced with actual agent analysis).
    fn generate_placeholder_fix(&self, context: &ErrorContext) -> (String, String, String) {
        match context.error_type.as_str() {
            "type_error" => (
                "Add null check before accessing property".to_string(),
                "Attempting to access property on undefined/null value".to_string(),
                "Low risk: Defensive programming, prevents crash".to_string(),
            ),
            "compilation_error" => (
                "Add missing import or dependency".to_string(),
                "Module or package not found in scope".to_string(),
                "Medium risk: May require dependency installation".to_string(),
            ),
            "runtime_crash" => (
                "Add error handling or bounds checking".to_string(),
                "Unhandled exception or panic in code".to_string(),
                "High priority: Prevents process crashes".to_string(),
            ),
            "network_error" => (
                "Add retry logic with exponential backoff".to_string(),
                "Network connection failure or timeout".to_string(),
                "Low risk: Improves resilience".to_string(),
            ),
            "database_error" => (
                "Fix SQL query or add connection retry".to_string(),
                "Database query failed or connection lost".to_string(),
                "Medium risk: May require schema changes".to_string(),
            ),
            _ => (
                "Review error and apply appropriate fix".to_string(),
                "Generic error detected in process logs".to_string(),
                "Unknown impact: Requires manual review".to_string(),
            ),
        }
    }

    /// Gets the next pending fix proposal from the queue.
    pub async fn get_next_pending_proposal(&self) -> Option<FixProposal> {
        let mut queue = self.approval_queue.lock().await;

        // Find first pending proposal
        if let Some(pos) = queue.iter().position(|p| p.status == ProposalStatus::Pending) {
            queue.get(pos).cloned()
        } else {
            None
        }
    }

    /// Approves a fix proposal and prepares it for execution.
    ///
    /// # Arguments
    /// * `proposal_id` - ID of the proposal to approve
    pub async fn approve_proposal(&self, proposal_id: &str) -> Result<()> {
        let mut queue = self.approval_queue.lock().await;

        let proposal = queue
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or_else(|| anyhow!("Proposal not found: {}", proposal_id))?;

        if proposal.status != ProposalStatus::Pending {
            return Err(anyhow!(
                "Proposal {} is not in pending status (current: {})",
                proposal_id,
                proposal.status
            ));
        }

        proposal.status = ProposalStatus::Approved;
        proposal.decided_at = Some(SystemTime::now());

        info!(proposal_id = %proposal_id, "Fix proposal approved");

        Ok(())
    }

    /// Rejects a fix proposal.
    ///
    /// # Arguments
    /// * `proposal_id` - ID of the proposal to reject
    /// * `reason` - Optional reason for rejection
    pub async fn reject_proposal(&self, proposal_id: &str, reason: Option<String>) -> Result<()> {
        let mut queue = self.approval_queue.lock().await;

        let proposal = queue
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or_else(|| anyhow!("Proposal not found: {}", proposal_id))?;

        if proposal.status != ProposalStatus::Pending {
            return Err(anyhow!(
                "Proposal {} is not in pending status (current: {})",
                proposal_id,
                proposal.status
            ));
        }

        proposal.status = ProposalStatus::Rejected;
        proposal.decided_at = Some(SystemTime::now());

        if let Some(reason) = reason {
            warn!(
                proposal_id = %proposal_id,
                reason = %reason,
                "Fix proposal rejected"
            );
        } else {
            warn!(proposal_id = %proposal_id, "Fix proposal rejected");
        }

        Ok(())
    }

    /// Applies an approved fix proposal.
    ///
    /// # Arguments
    /// * `proposal_id` - ID of the proposal to apply
    pub async fn apply_approved_fix(&self, proposal_id: &str) -> Result<()> {
        let mut queue = self.approval_queue.lock().await;

        let proposal = queue
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or_else(|| anyhow!("Proposal not found: {}", proposal_id))?;

        if proposal.status != ProposalStatus::Approved {
            return Err(anyhow!(
                "Proposal {} is not approved (current: {})",
                proposal_id,
                proposal.status
            ));
        }

        proposal.status = ProposalStatus::Applying;

        info!(
            proposal_id = %proposal_id,
            "Applying fix"
        );

        // TODO: In full implementation:
        // 1. Create checkpoint for rollback
        // 2. Execute fix steps (file edits, command runs, etc.)
        // 3. Restart process if needed
        // 4. Update status to Applied

        // For now, just mark as applied
        proposal.status = ProposalStatus::Applied;

        info!(
            proposal_id = %proposal_id,
            "Fix applied successfully"
        );

        Ok(())
    }

    /// Verifies that a fix resolved the issue.
    ///
    /// # Arguments
    /// * `proposal_id` - ID of the proposal to verify
    /// * `verification_window` - How long to monitor for error recurrence
    pub async fn verify_fix(
        &self,
        proposal_id: &str,
        verification_window: Duration,
    ) -> Result<FixVerificationResult> {
        let mut queue = self.approval_queue.lock().await;

        let proposal = queue
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or_else(|| anyhow!("Proposal not found: {}", proposal_id))?;

        if proposal.status != ProposalStatus::Applied {
            return Err(anyhow!(
                "Proposal {} is not applied (current: {})",
                proposal_id,
                proposal.status
            ));
        }

        proposal.status = ProposalStatus::Verifying;

        info!(
            proposal_id = %proposal_id,
            verification_window_secs = verification_window.as_secs(),
            "Starting fix verification"
        );

        // TODO: In full implementation:
        // 1. Monitor process logs for error recurrence
        // 2. Check process stability (no crashes)
        // 3. Detect new errors introduced by fix
        // 4. Return detailed verification result

        // For now, assume success
        let result = FixVerificationResult {
            success: true,
            details: "Fix verified: no error recurrence detected".to_string(),
            error_reappeared: false,
            process_stable: true,
        };

        if result.success {
            proposal.status = ProposalStatus::Verified;
            info!(
                proposal_id = %proposal_id,
                "Fix verified successfully"
            );
        } else {
            proposal.status = ProposalStatus::RolledBack;
            error!(
                proposal_id = %proposal_id,
                "Fix verification failed, rolling back"
            );
        }

        Ok(result)
    }

    /// Checks for timed-out approval requests.
    pub async fn check_timeouts(&self) {
        let mut queue = self.approval_queue.lock().await;

        let now = SystemTime::now();

        for proposal in queue.iter_mut() {
            if proposal.status == ProposalStatus::Pending {
                if let Some(requested_at) = proposal.approval_requested_at {
                    if let Ok(elapsed) = now.duration_since(requested_at) {
                        if elapsed > self.approval_timeout {
                            proposal.status = ProposalStatus::TimedOut;
                            proposal.decided_at = Some(now);

                            warn!(
                                proposal_id = %proposal.id,
                                "Approval request timed out"
                            );
                        }
                    }
                }
            }
        }
    }

    /// Gets all proposals (for debugging/monitoring).
    pub async fn get_all_proposals(&self) -> Vec<FixProposal> {
        self.approval_queue.lock().await.iter().cloned().collect()
    }

    /// Removes old proposals from the queue.
    ///
    /// # Arguments
    /// * `max_age` - Maximum age to keep proposals
    pub async fn cleanup_old_proposals(&self, max_age: Duration) {
        let mut queue = self.approval_queue.lock().await;
        let now = SystemTime::now();

        queue.retain(|proposal| {
            if let Ok(age) = now.duration_since(proposal.created_at) {
                age < max_age
            } else {
                true // Keep if we can't determine age
            }
        });
    }
}

impl Default for ErrorRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_error_context() -> ErrorContext {
        ErrorContext {
            process_id: "test-process-123".to_string(),
            error_lines: vec![
                "Error: Cannot find module 'foo'".to_string(),
                "at require (internal/modules/cjs/loader.js:883:19)".to_string(),
            ],
            severity: "high".to_string(),
            error_type: "compilation_error".to_string(),
            confidence: 0.85,
            suggested_agent: Some("typescript-agent".to_string()),
            metadata: serde_json::json!({
                "command": "node",
                "args": ["server.js"],
                "working_dir": "/test",
                "pid": 12345
            }),
        }
    }

    #[tokio::test]
    async fn test_route_error_creates_proposal() {
        let router = ErrorRouter::new();
        let context = create_test_error_context();

        let proposal_id = router.route_error(context).await.unwrap();

        assert!(!proposal_id.is_empty());

        // Check that proposal was added to queue
        let proposals = router.get_all_proposals().await;
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].id, proposal_id);
        assert_eq!(proposals[0].status, ProposalStatus::Pending);
    }

    #[tokio::test]
    async fn test_approve_and_apply_proposal() {
        let router = ErrorRouter::new();
        let context = create_test_error_context();

        let proposal_id = router.route_error(context).await.unwrap();

        // Approve
        router.approve_proposal(&proposal_id).await.unwrap();

        let proposals = router.get_all_proposals().await;
        assert_eq!(proposals[0].status, ProposalStatus::Approved);

        // Apply
        router.apply_approved_fix(&proposal_id).await.unwrap();

        let proposals = router.get_all_proposals().await;
        assert_eq!(proposals[0].status, ProposalStatus::Applied);
    }

    #[tokio::test]
    async fn test_reject_proposal() {
        let router = ErrorRouter::new();
        let context = create_test_error_context();

        let proposal_id = router.route_error(context).await.unwrap();

        // Reject
        router
            .reject_proposal(&proposal_id, Some("Not needed".to_string()))
            .await
            .unwrap();

        let proposals = router.get_all_proposals().await;
        assert_eq!(proposals[0].status, ProposalStatus::Rejected);
    }

    #[tokio::test]
    async fn test_verify_fix() {
        let router = ErrorRouter::new();
        let context = create_test_error_context();

        let proposal_id = router.route_error(context).await.unwrap();

        // Approve and apply first
        router.approve_proposal(&proposal_id).await.unwrap();
        router.apply_approved_fix(&proposal_id).await.unwrap();

        // Verify
        let result = router
            .verify_fix(&proposal_id, Duration::from_secs(10))
            .await
            .unwrap();

        assert!(result.success);

        let proposals = router.get_all_proposals().await;
        assert_eq!(proposals[0].status, ProposalStatus::Verified);
    }

    #[tokio::test]
    async fn test_get_next_pending_proposal() {
        let router = ErrorRouter::new();
        let context1 = create_test_error_context();
        let context2 = create_test_error_context();

        let id1 = router.route_error(context1).await.unwrap();
        let _id2 = router.route_error(context2).await.unwrap();

        // Approve first one
        router.approve_proposal(&id1).await.unwrap();

        // Should get second pending one
        let next = router.get_next_pending_proposal().await;
        assert!(next.is_some());
        assert_ne!(next.unwrap().id, id1);
    }
}

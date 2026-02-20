//! Child Workflow component schema
//!
//! The Child Workflow component invokes another workflow as a child.
//! Supports parent close policies and various execution options.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use super::activity::RetryConfig;

/// Parent close policy options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ParentClosePolicy {
    /// Terminate child when parent closes
    #[default]
    Terminate,
    /// Abandon child (let it continue)
    Abandon,
    /// Request cancellation of child
    RequestCancel,
}

impl ParentClosePolicy {
    /// Convert to TypeScript representation
    pub fn to_typescript(&self) -> &'static str {
        match self {
            ParentClosePolicy::Terminate => "'terminate'",
            ParentClosePolicy::Abandon => "'abandon'",
            ParentClosePolicy::RequestCancel => "'request-cancel'",
        }
    }
}

/// Child Workflow component input
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ChildWorkflowInput {
    /// Workflow name/type to invoke
    #[validate(length(min = 1, message = "Workflow name is required"))]
    pub workflow_name: String,

    /// Custom workflow ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,

    /// Task queue for the child workflow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_queue: Option<String>,

    /// Input parameters for child workflow
    #[serde(default)]
    pub input: HashMap<String, serde_json::Value>,

    /// Parent close policy
    #[serde(default)]
    pub parent_close_policy: ParentClosePolicy,

    /// Workflow execution timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_timeout_ms: Option<u64>,

    /// Workflow run timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_timeout_ms: Option<u64>,

    /// Whether to wait for result
    #[serde(default = "default_true")]
    pub await_result: bool,

    /// Retry configuration
    #[serde(default)]
    pub retry: RetryConfig,

    /// Memo fields
    #[serde(default)]
    pub memo: HashMap<String, serde_json::Value>,

    /// Search attributes
    #[serde(default)]
    pub search_attributes: HashMap<String, serde_json::Value>,
}

fn default_true() -> bool {
    true
}

impl ChildWorkflowInput {
    /// Create a new child workflow input
    pub fn new(workflow_name: impl Into<String>) -> Self {
        Self {
            workflow_name: workflow_name.into(),
            workflow_id: None,
            task_queue: None,
            input: HashMap::new(),
            parent_close_policy: ParentClosePolicy::default(),
            execution_timeout_ms: None,
            run_timeout_ms: None,
            await_result: true,
            retry: RetryConfig::default(),
            memo: HashMap::new(),
            search_attributes: HashMap::new(),
        }
    }

    /// Set custom workflow ID
    pub fn with_workflow_id(mut self, id: impl Into<String>) -> Self {
        self.workflow_id = Some(id.into());
        self
    }

    /// Set task queue
    pub fn with_task_queue(mut self, queue: impl Into<String>) -> Self {
        self.task_queue = Some(queue.into());
        self
    }

    /// Add input parameter
    pub fn with_input(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.input.insert(key.into(), value);
        self
    }

    /// Set parent close policy
    pub fn with_parent_close_policy(mut self, policy: ParentClosePolicy) -> Self {
        self.parent_close_policy = policy;
        self
    }

    /// Set execution timeout
    pub fn with_execution_timeout(mut self, ms: u64) -> Self {
        self.execution_timeout_ms = Some(ms);
        self
    }

    /// Set to fire-and-forget (don't await result)
    pub fn fire_and_forget(mut self) -> Self {
        self.await_result = false;
        self
    }

    /// Set retry configuration
    pub fn with_retry(mut self, retry: RetryConfig) -> Self {
        self.retry = retry;
        self
    }
}

impl Default for ChildWorkflowInput {
    fn default() -> Self {
        Self::new("childWorkflow")
    }
}

/// Workflow execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowStatus {
    /// Workflow is running
    #[default]
    Running,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed
    Failed,
    /// Workflow was cancelled
    Cancelled,
    /// Workflow was terminated
    Terminated,
    /// Workflow timed out
    TimedOut,
}

impl WorkflowStatus {
    /// Check if the workflow is finished
    pub fn is_finished(&self) -> bool {
        !matches!(self, WorkflowStatus::Running)
    }

    /// Check if the workflow completed successfully
    pub fn is_success(&self) -> bool {
        matches!(self, WorkflowStatus::Completed)
    }
}

/// Child Workflow component output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChildWorkflowOutput {
    /// Child workflow ID
    pub workflow_id: String,

    /// Child workflow run ID
    pub run_id: String,

    /// Result of the child workflow (if completed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    /// Current status
    pub status: WorkflowStatus,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl ChildWorkflowOutput {
    /// Create a completed output
    pub fn completed(
        workflow_id: impl Into<String>,
        run_id: impl Into<String>,
        result: serde_json::Value,
        duration_ms: u64,
    ) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            run_id: run_id.into(),
            result: Some(result),
            status: WorkflowStatus::Completed,
            error: None,
            duration_ms,
        }
    }

    /// Create a started (fire-and-forget) output
    pub fn started(workflow_id: impl Into<String>, run_id: impl Into<String>) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            run_id: run_id.into(),
            result: None,
            status: WorkflowStatus::Running,
            error: None,
            duration_ms: 0,
        }
    }

    /// Create a failed output
    pub fn failed(
        workflow_id: impl Into<String>,
        run_id: impl Into<String>,
        error: impl Into<String>,
        duration_ms: u64,
    ) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            run_id: run_id.into(),
            result: None,
            status: WorkflowStatus::Failed,
            error: Some(error.into()),
            duration_ms,
        }
    }
}

impl Default for ChildWorkflowOutput {
    fn default() -> Self {
        Self::started("workflow-id", "run-id")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parent_close_policy_serialization() {
        assert_eq!(
            serde_json::to_string(&ParentClosePolicy::Terminate).unwrap(),
            "\"terminate\""
        );
        assert_eq!(
            serde_json::to_string(&ParentClosePolicy::Abandon).unwrap(),
            "\"abandon\""
        );
    }

    #[test]
    fn test_child_workflow_input() {
        let input = ChildWorkflowInput::new("processOrder")
            .with_workflow_id("order-123")
            .with_input("orderId", serde_json::json!("123"))
            .with_parent_close_policy(ParentClosePolicy::Abandon);

        assert_eq!(input.workflow_name, "processOrder");
        assert!(input.workflow_id.is_some());
        assert_eq!(input.parent_close_policy, ParentClosePolicy::Abandon);
    }

    #[test]
    fn test_child_workflow_fire_and_forget() {
        let input = ChildWorkflowInput::new("backgroundTask").fire_and_forget();
        assert!(!input.await_result);
    }

    #[test]
    fn test_workflow_status() {
        assert!(!WorkflowStatus::Running.is_finished());
        assert!(WorkflowStatus::Completed.is_finished());
        assert!(WorkflowStatus::Completed.is_success());
        assert!(!WorkflowStatus::Failed.is_success());
    }

    #[test]
    fn test_child_workflow_output_completed() {
        let output = ChildWorkflowOutput::completed(
            "wf-123",
            "run-456",
            serde_json::json!({"result": "success"}),
            5000,
        );

        assert_eq!(output.status, WorkflowStatus::Completed);
        assert!(output.result.is_some());
        assert!(output.error.is_none());
    }

    #[test]
    fn test_child_workflow_output_failed() {
        let output = ChildWorkflowOutput::failed("wf-123", "run-456", "Timeout", 30000);

        assert_eq!(output.status, WorkflowStatus::Failed);
        assert!(output.error.is_some());
    }

    #[test]
    fn test_serialization() {
        let input = ChildWorkflowInput::new("myWorkflow")
            .with_task_queue("my-queue")
            .with_execution_timeout(60000);

        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("workflowName"));
        assert!(json.contains("taskQueue"));
    }
}

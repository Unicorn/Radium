//! Child Workflow Orchestration
//!
//! Enhanced child workflow support with:
//! - Workflow ID generation strategies
//! - Parent-child relationship tracking
//! - Cancellation type control
//! - Comprehensive execution options

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

// Import from public re-exports in components module
use crate::schema::components::RetryConfig;

// Re-export base types from components
pub use crate::schema::components::{ParentClosePolicy, WorkflowStatus};

/// Strategy for generating child workflow IDs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowIdStrategy {
    /// Use explicitly provided workflow ID
    Explicit,
    /// Generate a UUID for the workflow ID
    #[default]
    Uuid,
    /// Use parent workflow ID with a suffix
    ParentSuffix,
    /// Use a custom pattern with variable substitution
    Pattern,
}

impl WorkflowIdStrategy {
    /// Convert to TypeScript representation
    pub fn to_typescript(&self) -> &'static str {
        match self {
            WorkflowIdStrategy::Explicit => "'explicit'",
            WorkflowIdStrategy::Uuid => "'uuid'",
            WorkflowIdStrategy::ParentSuffix => "'parent-suffix'",
            WorkflowIdStrategy::Pattern => "'pattern'",
        }
    }
}

/// Policy for handling workflow ID conflicts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowIdReusePolicy {
    /// Allow starting if previous workflow completed
    #[default]
    AllowDuplicate,
    /// Allow only if previous workflow failed
    AllowDuplicateFailedOnly,
    /// Never reuse workflow ID
    RejectDuplicate,
    /// Terminate running workflow and start new
    TerminateIfRunning,
}

impl WorkflowIdReusePolicy {
    /// Convert to TypeScript Temporal SDK enum
    pub fn to_typescript(&self) -> &'static str {
        match self {
            WorkflowIdReusePolicy::AllowDuplicate => "WorkflowIdReusePolicy.ALLOW_DUPLICATE",
            WorkflowIdReusePolicy::AllowDuplicateFailedOnly => {
                "WorkflowIdReusePolicy.ALLOW_DUPLICATE_FAILED_ONLY"
            }
            WorkflowIdReusePolicy::RejectDuplicate => "WorkflowIdReusePolicy.REJECT_DUPLICATE",
            WorkflowIdReusePolicy::TerminateIfRunning => {
                "WorkflowIdReusePolicy.TERMINATE_IF_RUNNING"
            }
        }
    }
}

/// How cancellation is handled for child workflows
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CancellationType {
    /// Wait for child to acknowledge and complete cancellation
    #[default]
    WaitCancellationCompleted,
    /// Try to cancel without waiting for completion
    TryCancel,
    /// Don't send cancellation (abandon child)
    Abandon,
}

impl CancellationType {
    /// Convert to TypeScript Temporal SDK enum
    pub fn to_typescript(&self) -> &'static str {
        match self {
            CancellationType::WaitCancellationCompleted => {
                "ChildWorkflowCancellationType.WAIT_CANCELLATION_COMPLETED"
            }
            CancellationType::TryCancel => "ChildWorkflowCancellationType.TRY_CANCEL",
            CancellationType::Abandon => "ChildWorkflowCancellationType.ABANDON",
        }
    }
}

/// Search attribute value types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SearchAttributeValue {
    /// String value
    String(String),
    /// Integer value
    Int(i64),
    /// Floating point value
    Double(f64),
    /// Boolean value
    Bool(bool),
    /// DateTime value
    Datetime(DateTime<Utc>),
    /// Array of strings
    StringArray(Vec<String>),
}

impl SearchAttributeValue {
    /// Convert to TypeScript representation
    pub fn to_typescript(&self) -> String {
        match self {
            SearchAttributeValue::String(s) => format!("'{}'", s),
            SearchAttributeValue::Int(n) => n.to_string(),
            SearchAttributeValue::Double(n) => n.to_string(),
            SearchAttributeValue::Bool(b) => b.to_string(),
            SearchAttributeValue::Datetime(dt) => format!("new Date('{}')", dt.to_rfc3339()),
            SearchAttributeValue::StringArray(arr) => {
                let items: Vec<_> = arr.iter().map(|s| format!("'{}'", s)).collect();
                format!("[{}]", items.join(", "))
            }
        }
    }
}

/// Enhanced child workflow configuration for orchestration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ChildWorkflowOrchestration {
    /// Workflow type/name to invoke
    #[validate(length(min = 1, message = "Workflow type is required"))]
    pub workflow_type: String,

    /// Task queue (defaults to parent's if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_queue: Option<String>,

    /// Workflow ID generation strategy
    #[serde(default)]
    pub id_strategy: WorkflowIdStrategy,

    /// Explicit workflow ID (required when strategy is Explicit)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,

    /// ID pattern for Pattern strategy
    /// Supports: {parent_id}, {index}, {timestamp}, {uuid}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_pattern: Option<String>,

    /// ID reuse policy for conflicts
    #[serde(default)]
    pub id_reuse_policy: WorkflowIdReusePolicy,

    /// Input to pass to child workflow
    #[serde(default)]
    pub input: HashMap<String, serde_json::Value>,

    /// Parent close policy
    #[serde(default)]
    pub parent_close_policy: ParentClosePolicy,

    /// Execution timeout (total time for all runs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_timeout_ms: Option<u64>,

    /// Run timeout (time for single run)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_timeout_ms: Option<u64>,

    /// Task timeout (time to start execution)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_timeout_ms: Option<u64>,

    /// Retry policy for the child workflow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<RetryConfig>,

    /// Cancellation type
    #[serde(default)]
    pub cancellation_type: CancellationType,

    /// Whether to wait for result (false = fire and forget)
    #[serde(default = "default_true")]
    pub await_result: bool,

    /// Search attributes to set on child
    #[serde(default)]
    pub search_attributes: HashMap<String, SearchAttributeValue>,

    /// Memo fields for storing arbitrary data
    #[serde(default)]
    pub memo: HashMap<String, String>,

    /// Cron schedule for recurring child workflow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron_schedule: Option<String>,
}

fn default_true() -> bool {
    true
}

impl ChildWorkflowOrchestration {
    /// Create new orchestration config for a workflow type
    pub fn new(workflow_type: impl Into<String>) -> Self {
        Self {
            workflow_type: workflow_type.into(),
            task_queue: None,
            id_strategy: WorkflowIdStrategy::default(),
            workflow_id: None,
            id_pattern: None,
            id_reuse_policy: WorkflowIdReusePolicy::default(),
            input: HashMap::new(),
            parent_close_policy: ParentClosePolicy::default(),
            execution_timeout_ms: None,
            run_timeout_ms: None,
            task_timeout_ms: None,
            retry_policy: None,
            cancellation_type: CancellationType::default(),
            await_result: true,
            search_attributes: HashMap::new(),
            memo: HashMap::new(),
            cron_schedule: None,
        }
    }

    /// Set explicit workflow ID
    pub fn with_workflow_id(mut self, id: impl Into<String>) -> Self {
        self.workflow_id = Some(id.into());
        self.id_strategy = WorkflowIdStrategy::Explicit;
        self
    }

    /// Set workflow ID pattern
    pub fn with_id_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.id_pattern = Some(pattern.into());
        self.id_strategy = WorkflowIdStrategy::Pattern;
        self
    }

    /// Use parent workflow ID with suffix
    pub fn with_parent_suffix(mut self) -> Self {
        self.id_strategy = WorkflowIdStrategy::ParentSuffix;
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

    /// Set ID reuse policy
    pub fn with_id_reuse_policy(mut self, policy: WorkflowIdReusePolicy) -> Self {
        self.id_reuse_policy = policy;
        self
    }

    /// Set cancellation type
    pub fn with_cancellation_type(mut self, cancellation_type: CancellationType) -> Self {
        self.cancellation_type = cancellation_type;
        self
    }

    /// Configure as fire-and-forget
    pub fn fire_and_forget(mut self) -> Self {
        self.await_result = false;
        self
    }

    /// Set execution timeout
    pub fn with_execution_timeout(mut self, ms: u64) -> Self {
        self.execution_timeout_ms = Some(ms);
        self
    }

    /// Set run timeout
    pub fn with_run_timeout(mut self, ms: u64) -> Self {
        self.run_timeout_ms = Some(ms);
        self
    }

    /// Set retry policy
    pub fn with_retry_policy(mut self, policy: RetryConfig) -> Self {
        self.retry_policy = Some(policy);
        self
    }

    /// Add search attribute
    pub fn with_search_attribute(
        mut self,
        name: impl Into<String>,
        value: SearchAttributeValue,
    ) -> Self {
        self.search_attributes.insert(name.into(), value);
        self
    }

    /// Add memo field
    pub fn with_memo(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.memo.insert(key.into(), value.into());
        self
    }

    /// Set cron schedule
    pub fn with_cron_schedule(mut self, schedule: impl Into<String>) -> Self {
        self.cron_schedule = Some(schedule.into());
        self
    }

    /// Validate the configuration
    pub fn validate_config(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate ID strategy requirements
        match self.id_strategy {
            WorkflowIdStrategy::Explicit if self.workflow_id.is_none() => {
                errors.push("Explicit ID strategy requires workflow_id".to_string());
            }
            WorkflowIdStrategy::Pattern if self.id_pattern.is_none() => {
                errors.push("Pattern ID strategy requires id_pattern".to_string());
            }
            _ => {}
        }

        // Validate timeouts
        if let (Some(exec), Some(run)) = (self.execution_timeout_ms, self.run_timeout_ms) {
            if run > exec {
                errors.push("run_timeout_ms cannot exceed execution_timeout_ms".to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Generate the workflow ID based on strategy
    pub fn generate_workflow_id(&self, parent_id: &str, index: usize) -> String {
        match self.id_strategy {
            WorkflowIdStrategy::Explicit => self.workflow_id.clone().unwrap_or_default(),
            WorkflowIdStrategy::Uuid => uuid::Uuid::new_v4().to_string(),
            WorkflowIdStrategy::ParentSuffix => {
                format!("{}-child-{}", parent_id, index)
            }
            WorkflowIdStrategy::Pattern => {
                let pattern = self.id_pattern.clone().unwrap_or_default();
                pattern
                    .replace("{parent_id}", parent_id)
                    .replace("{index}", &index.to_string())
                    .replace("{timestamp}", &Utc::now().timestamp().to_string())
                    .replace("{uuid}", &uuid::Uuid::new_v4().to_string())
            }
        }
    }

    /// Generate TypeScript code for executing this child workflow
    pub fn to_typescript(&self) -> String {
        let mut code = String::new();

        // Generate the executeChild call
        code.push_str(&format!(
            r#"// Execute child workflow: {}
const childHandle = await executeChild('{}', {{
  args: [{}],
"#,
            self.workflow_type,
            self.workflow_type,
            self.input_to_typescript()
        ));

        // Add workflow ID
        match self.id_strategy {
            WorkflowIdStrategy::Explicit => {
                if let Some(id) = &self.workflow_id {
                    code.push_str(&format!("  workflowId: '{}',\n", id));
                }
            }
            WorkflowIdStrategy::Uuid => {
                code.push_str("  workflowId: uuid4(),\n");
            }
            WorkflowIdStrategy::ParentSuffix => {
                code.push_str(
                    "  workflowId: `${workflowInfo().workflowId}-child-${childIndex++}`,\n",
                );
            }
            WorkflowIdStrategy::Pattern => {
                if let Some(pattern) = &self.id_pattern {
                    let ts_pattern = pattern
                        .replace("{parent_id}", "${workflowInfo().workflowId}")
                        .replace("{index}", "${childIndex++}")
                        .replace("{timestamp}", "${Date.now()}")
                        .replace("{uuid}", "${uuid4()}");
                    code.push_str(&format!("  workflowId: `{}`,\n", ts_pattern));
                }
            }
        }

        // Add task queue
        if let Some(queue) = &self.task_queue {
            code.push_str(&format!("  taskQueue: '{}',\n", queue));
        }

        // Add parent close policy
        code.push_str(&format!(
            "  parentClosePolicy: ParentClosePolicy.{},\n",
            match self.parent_close_policy {
                ParentClosePolicy::Terminate => "PARENT_CLOSE_POLICY_TERMINATE",
                ParentClosePolicy::Abandon => "PARENT_CLOSE_POLICY_ABANDON",
                ParentClosePolicy::RequestCancel => "PARENT_CLOSE_POLICY_REQUEST_CANCEL",
            }
        ));

        // Add ID reuse policy
        code.push_str(&format!(
            "  workflowIdReusePolicy: {},\n",
            self.id_reuse_policy.to_typescript()
        ));

        // Add cancellation type
        code.push_str(&format!(
            "  cancellationType: {},\n",
            self.cancellation_type.to_typescript()
        ));

        // Add timeouts
        if let Some(timeout) = self.execution_timeout_ms {
            code.push_str(&format!("  workflowExecutionTimeout: '{}ms',\n", timeout));
        }
        if let Some(timeout) = self.run_timeout_ms {
            code.push_str(&format!("  workflowRunTimeout: '{}ms',\n", timeout));
        }
        if let Some(timeout) = self.task_timeout_ms {
            code.push_str(&format!("  workflowTaskTimeout: '{}ms',\n", timeout));
        }

        // Add retry policy
        if let Some(retry) = &self.retry_policy {
            code.push_str(&format!(
                r#"  retry: {{
    maximumAttempts: {},
    initialInterval: '{}ms',
    maximumInterval: '{}ms',
    backoffCoefficient: {},
  }},
"#,
                retry.max_attempts,
                retry.initial_interval_ms,
                retry.max_interval_ms,
                retry.backoff_coefficient
            ));
        }

        // Add search attributes
        if !self.search_attributes.is_empty() {
            code.push_str("  searchAttributes: {\n");
            for (key, value) in &self.search_attributes {
                code.push_str(&format!("    {}: [{}],\n", key, value.to_typescript()));
            }
            code.push_str("  },\n");
        }

        // Add memo
        if !self.memo.is_empty() {
            code.push_str("  memo: {\n");
            for (key, value) in &self.memo {
                code.push_str(&format!("    {}: '{}',\n", key, value));
            }
            code.push_str("  },\n");
        }

        // Add cron schedule
        if let Some(cron) = &self.cron_schedule {
            code.push_str(&format!("  cronSchedule: '{}',\n", cron));
        }

        code.push_str("});\n\n");

        // Handle result
        if self.await_result {
            code.push_str("const childResult = await childHandle.result();\n");
        } else {
            code.push_str("// Fire and forget - not awaiting result\n");
        }

        code
    }

    /// Convert input to TypeScript object
    fn input_to_typescript(&self) -> String {
        if self.input.is_empty() {
            return "{}".to_string();
        }

        let mut parts: Vec<String> = Vec::new();
        for (key, value) in &self.input {
            let ts_value = match value {
                serde_json::Value::String(s) => format!("'{}'", s),
                serde_json::Value::Null => "null".to_string(),
                _ => value.to_string(),
            };
            parts.push(format!("{}: {}", key, ts_value));
        }

        format!("{{ {} }}", parts.join(", "))
    }
}

impl Default for ChildWorkflowOrchestration {
    fn default() -> Self {
        Self::new("ChildWorkflow")
    }
}

/// Handle for tracking a child workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChildWorkflowHandle {
    /// Child workflow ID
    pub workflow_id: String,
    /// Child workflow run ID
    pub run_id: String,
    /// Workflow type that was started
    pub workflow_type: String,
    /// Parent workflow ID
    pub parent_workflow_id: String,
    /// Parent workflow run ID
    pub parent_run_id: String,
    /// First execution run ID (same across continue-as-new)
    pub first_execution_run_id: Option<String>,
}

impl ChildWorkflowHandle {
    /// Create a new handle
    pub fn new(
        workflow_id: impl Into<String>,
        run_id: impl Into<String>,
        workflow_type: impl Into<String>,
        parent_workflow_id: impl Into<String>,
        parent_run_id: impl Into<String>,
    ) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            run_id: run_id.into(),
            workflow_type: workflow_type.into(),
            parent_workflow_id: parent_workflow_id.into(),
            parent_run_id: parent_run_id.into(),
            first_execution_run_id: None,
        }
    }
}

/// Result of a child workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChildWorkflowExecutionResult {
    /// Handle to the child workflow
    pub handle: ChildWorkflowHandle,
    /// Execution status
    pub status: WorkflowStatus,
    /// Result value if completed successfully
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error details if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<WorkflowExecutionError>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Error from a workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowExecutionError {
    /// Error type/name
    pub error_type: String,
    /// Error message
    pub message: String,
    /// Nested cause (for chained errors)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<Box<WorkflowExecutionError>>,
    /// Stack trace if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<String>,
    /// Whether this error is retryable
    #[serde(default)]
    pub retryable: bool,
}

impl WorkflowExecutionError {
    /// Create a new error
    pub fn new(error_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error_type: error_type.into(),
            message: message.into(),
            cause: None,
            stack_trace: None,
            retryable: false,
        }
    }

    /// Add a cause to this error
    pub fn with_cause(mut self, cause: WorkflowExecutionError) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    /// Mark as retryable
    pub fn retryable(mut self) -> Self {
        self.retryable = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_id_strategy_serialization() {
        assert_eq!(
            serde_json::to_string(&WorkflowIdStrategy::Uuid).unwrap(),
            "\"uuid\""
        );
        assert_eq!(
            serde_json::to_string(&WorkflowIdStrategy::ParentSuffix).unwrap(),
            "\"parent-suffix\""
        );
    }

    #[test]
    fn test_id_reuse_policy_typescript() {
        assert_eq!(
            WorkflowIdReusePolicy::AllowDuplicate.to_typescript(),
            "WorkflowIdReusePolicy.ALLOW_DUPLICATE"
        );
        assert_eq!(
            WorkflowIdReusePolicy::TerminateIfRunning.to_typescript(),
            "WorkflowIdReusePolicy.TERMINATE_IF_RUNNING"
        );
    }

    #[test]
    fn test_cancellation_type_typescript() {
        assert_eq!(
            CancellationType::WaitCancellationCompleted.to_typescript(),
            "ChildWorkflowCancellationType.WAIT_CANCELLATION_COMPLETED"
        );
    }

    #[test]
    fn test_search_attribute_value_typescript() {
        assert_eq!(
            SearchAttributeValue::String("test".to_string()).to_typescript(),
            "'test'"
        );
        assert_eq!(SearchAttributeValue::Int(42).to_typescript(), "42");
        assert_eq!(SearchAttributeValue::Bool(true).to_typescript(), "true");
    }

    #[test]
    fn test_child_workflow_orchestration_builder() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_workflow_id("order-123")
            .with_task_queue("orders")
            .with_parent_close_policy(ParentClosePolicy::Abandon)
            .with_id_reuse_policy(WorkflowIdReusePolicy::RejectDuplicate)
            .with_cancellation_type(CancellationType::TryCancel)
            .with_execution_timeout(300000)
            .with_input("orderId", serde_json::json!("order-123"))
            .with_search_attribute("CustomerId", SearchAttributeValue::String("cust-456".into()))
            .with_memo("reason", "customer request");

        assert_eq!(config.workflow_type, "ProcessOrder");
        assert_eq!(config.id_strategy, WorkflowIdStrategy::Explicit);
        assert_eq!(config.workflow_id, Some("order-123".to_string()));
        assert_eq!(config.task_queue, Some("orders".to_string()));
        assert_eq!(config.parent_close_policy, ParentClosePolicy::Abandon);
        assert!(config.input.contains_key("orderId"));
        assert!(config.search_attributes.contains_key("CustomerId"));
    }

    #[test]
    fn test_parent_suffix_strategy() {
        let config = ChildWorkflowOrchestration::new("SubWorkflow").with_parent_suffix();

        assert_eq!(config.id_strategy, WorkflowIdStrategy::ParentSuffix);

        let generated_id = config.generate_workflow_id("parent-workflow-1", 0);
        assert_eq!(generated_id, "parent-workflow-1-child-0");
    }

    #[test]
    fn test_pattern_strategy() {
        let config = ChildWorkflowOrchestration::new("SubWorkflow")
            .with_id_pattern("child-{parent_id}-{index}");

        assert_eq!(config.id_strategy, WorkflowIdStrategy::Pattern);

        let generated_id = config.generate_workflow_id("main", 5);
        assert_eq!(generated_id, "child-main-5");
    }

    #[test]
    fn test_fire_and_forget() {
        let config = ChildWorkflowOrchestration::new("BackgroundJob").fire_and_forget();

        assert!(!config.await_result);
    }

    #[test]
    fn test_validate_explicit_strategy_requires_id() {
        let config = ChildWorkflowOrchestration {
            id_strategy: WorkflowIdStrategy::Explicit,
            workflow_id: None,
            ..ChildWorkflowOrchestration::new("Test")
        };

        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .iter()
            .any(|e| e.contains("Explicit ID strategy")));
    }

    #[test]
    fn test_validate_pattern_strategy_requires_pattern() {
        let config = ChildWorkflowOrchestration {
            id_strategy: WorkflowIdStrategy::Pattern,
            id_pattern: None,
            ..ChildWorkflowOrchestration::new("Test")
        };

        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .iter()
            .any(|e| e.contains("Pattern ID strategy")));
    }

    #[test]
    fn test_validate_timeout_relationship() {
        let config = ChildWorkflowOrchestration {
            execution_timeout_ms: Some(60000),
            run_timeout_ms: Some(120000), // Greater than execution timeout
            ..ChildWorkflowOrchestration::new("Test")
        };

        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .iter()
            .any(|e| e.contains("run_timeout_ms cannot exceed")));
    }

    #[test]
    fn test_to_typescript_basic() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_task_queue("orders")
            .with_execution_timeout(300000);

        let ts = config.to_typescript();

        assert!(ts.contains("executeChild('ProcessOrder'"));
        assert!(ts.contains("taskQueue: 'orders'"));
        assert!(ts.contains("workflowExecutionTimeout: '300000ms'"));
        assert!(ts.contains("await childHandle.result()"));
    }

    #[test]
    fn test_to_typescript_fire_and_forget() {
        let config = ChildWorkflowOrchestration::new("BackgroundJob").fire_and_forget();

        let ts = config.to_typescript();

        assert!(ts.contains("Fire and forget"));
        assert!(!ts.contains("await childHandle.result()"));
    }

    #[test]
    fn test_child_workflow_handle() {
        let handle = ChildWorkflowHandle::new(
            "child-123",
            "run-456",
            "ProcessOrder",
            "parent-789",
            "parent-run-101",
        );

        assert_eq!(handle.workflow_id, "child-123");
        assert_eq!(handle.parent_workflow_id, "parent-789");
    }

    #[test]
    fn test_workflow_execution_error() {
        let cause = WorkflowExecutionError::new("NetworkError", "Connection refused");
        let error = WorkflowExecutionError::new("ProcessingError", "Failed to process order")
            .with_cause(cause)
            .retryable();

        assert!(error.retryable);
        assert!(error.cause.is_some());
        assert_eq!(
            error.cause.unwrap().error_type,
            "NetworkError"
        );
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = ChildWorkflowOrchestration::new("TestWorkflow")
            .with_task_queue("test-queue")
            .with_input("key", serde_json::json!("value"));

        let json = serde_json::to_string(&config).unwrap();
        let restored: ChildWorkflowOrchestration = serde_json::from_str(&json).unwrap();

        assert_eq!(config.workflow_type, restored.workflow_type);
        assert_eq!(config.task_queue, restored.task_queue);
    }
}

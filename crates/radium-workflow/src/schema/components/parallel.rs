//! Parallel component schema
//!
//! The Parallel component executes multiple branches concurrently.
//! Supports various join strategies and state merging.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// Join strategy for parallel execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum JoinStrategy {
    /// Wait for all branches to complete
    #[default]
    All,
    /// Wait for any one branch to complete
    Any,
    /// Wait for all branches, don't fail on individual errors
    AllSettled,
    /// Return first result, cancel others
    Race,
}

impl JoinStrategy {
    /// Convert to TypeScript representation
    pub fn to_typescript(&self) -> &'static str {
        match self {
            JoinStrategy::All => "'all'",
            JoinStrategy::Any => "'any'",
            JoinStrategy::AllSettled => "'allSettled'",
            JoinStrategy::Race => "'race'",
        }
    }
}

/// A branch in parallel execution
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Branch {
    /// Branch name
    #[validate(length(min = 1, message = "Branch name is required"))]
    pub name: String,

    /// Start node ID for this branch
    pub start_node: String,

    /// Branch-specific timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,

    /// Whether this branch is required (for AllSettled)
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

impl Branch {
    /// Create a new branch
    pub fn new(name: impl Into<String>, start_node: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start_node: start_node.into(),
            timeout_ms: None,
            required: true,
        }
    }

    /// Set timeout
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// Set as optional (won't fail AllSettled)
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

/// Parallel component input
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ParallelInput {
    /// Branch definitions
    #[validate(length(min = 2, message = "At least 2 branches required"))]
    pub branches: Vec<Branch>,

    /// Join strategy
    #[serde(default)]
    pub join_strategy: JoinStrategy,

    /// Maximum concurrent branches (0 = unlimited)
    #[serde(default)]
    pub max_concurrent: usize,

    /// Timeout for entire parallel block in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,

    /// Cancel remaining branches on error
    #[serde(default = "default_true")]
    pub cancel_on_error: bool,
}

impl ParallelInput {
    /// Create a new parallel input with branches
    pub fn new(branches: Vec<Branch>) -> Self {
        Self {
            branches,
            join_strategy: JoinStrategy::default(),
            max_concurrent: 0,
            timeout_ms: None,
            cancel_on_error: true,
        }
    }

    /// Add a branch
    pub fn add_branch(mut self, branch: Branch) -> Self {
        self.branches.push(branch);
        self
    }

    /// Set join strategy
    pub fn with_join_strategy(mut self, strategy: JoinStrategy) -> Self {
        self.join_strategy = strategy;
        self
    }

    /// Set max concurrent
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// Disable cancel on error
    pub fn continue_on_error(mut self) -> Self {
        self.cancel_on_error = false;
        self
    }
}

impl Default for ParallelInput {
    fn default() -> Self {
        Self::new(vec![
            Branch::new("branch1", "node1"),
            Branch::new("branch2", "node2"),
        ])
    }
}

/// Result from a single branch
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchResult {
    /// Branch name
    pub branch_name: String,

    /// Whether the branch succeeded
    pub success: bool,

    /// Branch result (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Whether the branch was cancelled
    #[serde(default)]
    pub cancelled: bool,
}

impl BranchResult {
    /// Create a successful branch result
    pub fn success(
        branch_name: impl Into<String>,
        result: serde_json::Value,
        duration_ms: u64,
    ) -> Self {
        Self {
            branch_name: branch_name.into(),
            success: true,
            result: Some(result),
            error: None,
            duration_ms,
            cancelled: false,
        }
    }

    /// Create a failed branch result
    pub fn failure(
        branch_name: impl Into<String>,
        error: impl Into<String>,
        duration_ms: u64,
    ) -> Self {
        Self {
            branch_name: branch_name.into(),
            success: false,
            result: None,
            error: Some(error.into()),
            duration_ms,
            cancelled: false,
        }
    }

    /// Create a cancelled branch result
    pub fn cancelled(branch_name: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            branch_name: branch_name.into(),
            success: false,
            result: None,
            error: None,
            duration_ms,
            cancelled: true,
        }
    }
}

/// Parallel component output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParallelOutput {
    /// Whether the parallel block completed successfully
    pub completed: bool,

    /// Results from each branch
    pub results: HashMap<String, BranchResult>,

    /// Total duration in milliseconds
    pub duration_ms: u64,

    /// Whether any branches were cancelled
    #[serde(default)]
    pub had_cancellations: bool,

    /// Whether any branches failed
    #[serde(default)]
    pub had_failures: bool,
}

impl ParallelOutput {
    /// Create a new parallel output
    pub fn new(results: HashMap<String, BranchResult>, duration_ms: u64) -> Self {
        let had_cancellations = results.values().any(|r| r.cancelled);
        let had_failures = results.values().any(|r| !r.success && !r.cancelled);
        let completed = !had_failures && !had_cancellations;

        Self {
            completed,
            results,
            duration_ms,
            had_cancellations,
            had_failures,
        }
    }

    /// Get all successful results
    pub fn successful_results(&self) -> HashMap<String, &serde_json::Value> {
        self.results
            .iter()
            .filter(|(_, r)| r.success)
            .filter_map(|(k, r)| r.result.as_ref().map(|v| (k.clone(), v)))
            .collect()
    }

    /// Get all failed results
    pub fn failed_branches(&self) -> Vec<&str> {
        self.results
            .iter()
            .filter(|(_, r)| !r.success && !r.cancelled)
            .map(|(k, _)| k.as_str())
            .collect()
    }
}

impl Default for ParallelOutput {
    fn default() -> Self {
        Self::new(HashMap::new(), 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_strategy_serialization() {
        assert_eq!(
            serde_json::to_string(&JoinStrategy::All).unwrap(),
            "\"all\""
        );
        assert_eq!(
            serde_json::to_string(&JoinStrategy::Race).unwrap(),
            "\"race\""
        );
    }

    #[test]
    fn test_branch_creation() {
        let branch = Branch::new("processA", "node-a")
            .with_timeout(30000)
            .optional();

        assert_eq!(branch.name, "processA");
        assert_eq!(branch.timeout_ms, Some(30000));
        assert!(!branch.required);
    }

    #[test]
    fn test_parallel_input() {
        let input = ParallelInput::new(vec![
            Branch::new("fetch", "fetch-node"),
            Branch::new("process", "process-node"),
        ])
        .with_join_strategy(JoinStrategy::AllSettled)
        .with_max_concurrent(2);

        assert_eq!(input.branches.len(), 2);
        assert_eq!(input.join_strategy, JoinStrategy::AllSettled);
        assert_eq!(input.max_concurrent, 2);
    }

    #[test]
    fn test_parallel_input_validation() {
        use validator::Validate;

        let valid = ParallelInput::new(vec![
            Branch::new("a", "node-a"),
            Branch::new("b", "node-b"),
        ]);
        assert!(valid.validate().is_ok());

        let invalid = ParallelInput::new(vec![Branch::new("only", "node-only")]);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_branch_result_success() {
        let result = BranchResult::success("branch1", serde_json::json!({"data": "result"}), 1000);
        assert!(result.success);
        assert!(!result.cancelled);
    }

    #[test]
    fn test_branch_result_failure() {
        let result = BranchResult::failure("branch1", "Timeout", 5000);
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_branch_result_cancelled() {
        let result = BranchResult::cancelled("branch1", 500);
        assert!(!result.success);
        assert!(result.cancelled);
    }

    #[test]
    fn test_parallel_output() {
        let mut results = HashMap::new();
        results.insert(
            "branch1".to_string(),
            BranchResult::success("branch1", serde_json::json!(1), 100),
        );
        results.insert(
            "branch2".to_string(),
            BranchResult::success("branch2", serde_json::json!(2), 200),
        );

        let output = ParallelOutput::new(results, 200);

        assert!(output.completed);
        assert!(!output.had_failures);
        assert!(!output.had_cancellations);
        assert_eq!(output.successful_results().len(), 2);
    }

    #[test]
    fn test_parallel_output_with_failure() {
        let mut results = HashMap::new();
        results.insert(
            "branch1".to_string(),
            BranchResult::success("branch1", serde_json::json!(1), 100),
        );
        results.insert(
            "branch2".to_string(),
            BranchResult::failure("branch2", "Error", 50),
        );

        let output = ParallelOutput::new(results, 100);

        assert!(!output.completed);
        assert!(output.had_failures);
        assert_eq!(output.failed_branches().len(), 1);
    }

    #[test]
    fn test_serialization() {
        let input = ParallelInput::new(vec![
            Branch::new("a", "node-a"),
            Branch::new("b", "node-b"),
        ])
        .with_timeout(60000);

        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("branches"));
        assert!(json.contains("joinStrategy"));
    }
}

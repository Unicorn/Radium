//! Batch processing component schema
//!
//! The Batch component processes items in configurable batches with concurrency
//! control. It invokes a named activity once per batch and aggregates per-item
//! results, supporting three failure strategies: stop on first error, continue
//! through all errors, or silently skip failed items.

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Strategy applied when one or more items in a batch fail.
///
/// Controls whether the component halts immediately, accumulates all errors,
/// or silently drops failed items and continues.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum BatchFailStrategy {
    /// Halt the entire batch run as soon as the first item fails (default).
    #[default]
    StopOnFirst,
    /// Continue processing remaining batches even when some items fail;
    /// all errors are collected in the output.
    ContinueAll,
    /// Skip failed items without recording them as errors; only successes
    /// are included in `results`.
    SkipFailed,
}

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

/// Batch processing component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct BatchInput {
    /// The items to process. Each element is forwarded verbatim to the
    /// activity as part of the batch payload.
    pub items: Vec<serde_json::Value>,

    /// Number of items to include in each batch sent to the activity.
    /// Defaults to 10.
    #[serde(default = "default_batch_size")]
    pub batch_size: u32,

    /// Maximum number of batches to process simultaneously. A value of 1
    /// (default) means purely sequential processing.
    #[serde(default = "default_concurrency")]
    pub concurrency: u32,

    /// Strategy applied when a batch invocation fails.
    #[serde(default)]
    pub fail_strategy: BatchFailStrategy,

    /// Name of the activity to invoke for each batch. Must be a non-empty
    /// string matching the registered activity name in the task queue.
    #[validate(length(min = 1, message = "activity_name must not be empty"))]
    pub activity_name: String,

    /// Optional task queue override. When absent the workflow's default task
    /// queue is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_queue: Option<String>,

    /// Shared component behaviors (retry, timeout, rate limit, etc.).
    #[serde(default = "batch_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn default_batch_size() -> u32 {
    10
}

fn default_concurrency() -> u32 {
    1
}

fn batch_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 300_000,
        heartbeat_interval_ms: Some(10_000),
        rate_limit: RateLimitConfig {
            requests_per_second: 10,
            burst: 20,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl Default for BatchInput {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            batch_size: default_batch_size(),
            concurrency: default_concurrency(),
            fail_strategy: BatchFailStrategy::default(),
            activity_name: String::new(),
            task_queue: None,
            behaviors: batch_default_behaviors(),
        }
    }
}

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

/// Result for a single item processed by the batch component.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BatchItemResult {
    /// Zero-based index of the item in the original `items` array.
    pub index: u32,

    /// Whether the activity call for this item succeeded.
    pub success: bool,

    /// Activity return value when `success` is `true`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    /// Error message when `success` is `false`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Batch processing component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BatchOutput {
    /// Per-item results in index order.
    pub results: Vec<BatchItemResult>,

    /// Total number of items submitted for processing.
    pub total_items: u32,

    /// Number of items that completed successfully.
    pub successful: u32,

    /// Number of items that failed.
    pub failed: u32,

    /// Number of items that were skipped (only non-zero when
    /// `fail_strategy` is `SkipFailed`).
    pub skipped: u32,
}

impl Default for BatchOutput {
    fn default() -> Self {
        Self {
            results: Vec::new(),
            total_items: 0,
            successful: 0,
            failed: 0,
            skipped: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_with_defaults() {
        let input = BatchInput::default();

        assert!(input.items.is_empty());
        assert_eq!(input.batch_size, 10);
        assert_eq!(input.concurrency, 1);
        assert_eq!(input.fail_strategy, BatchFailStrategy::StopOnFirst);
        assert!(input.activity_name.is_empty());
        assert!(input.task_queue.is_none());

        // Behavior tier: I/O — 5-minute timeout with heartbeat
        assert_eq!(input.behaviors.timeout_ms, 300_000);
        assert_eq!(input.behaviors.heartbeat_interval_ms, Some(10_000));
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 10);
        assert_eq!(input.behaviors.rate_limit.burst, 20);
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
items:
  - id: 1
    name: "alpha"
  - id: 2
    name: "beta"
  - id: 3
    name: "gamma"
batch_size: 2
concurrency: 3
fail_strategy: continue_all
activity_name: "process_record"
task_queue: "high-priority"
behaviors:
  timeout_ms: 120000
  heartbeat_interval_ms: 5000
  rate_limit:
    requests_per_second: 5
    burst: 10
"#;
        let input: BatchInput = serde_yaml::from_str(yaml).expect("deserialize");

        assert_eq!(input.items.len(), 3);
        assert_eq!(input.items[0]["id"], 1);
        assert_eq!(input.items[1]["name"], "beta");
        assert_eq!(input.batch_size, 2);
        assert_eq!(input.concurrency, 3);
        assert_eq!(input.fail_strategy, BatchFailStrategy::ContinueAll);
        assert_eq!(input.activity_name, "process_record");
        assert_eq!(input.task_queue, Some("high-priority".to_string()));
        assert_eq!(input.behaviors.timeout_ms, 120_000);
        assert_eq!(input.behaviors.heartbeat_interval_ms, Some(5_000));
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 5);
        assert_eq!(input.behaviors.rate_limit.burst, 10);
    }

    #[test]
    fn test_output_serialize_deserialize() {
        let output = BatchOutput {
            results: vec![
                BatchItemResult {
                    index: 0,
                    success: true,
                    result: Some(serde_json::json!({"processed": true})),
                    error: None,
                },
                BatchItemResult {
                    index: 1,
                    success: false,
                    result: None,
                    error: Some("activity timeout".to_string()),
                },
            ],
            total_items: 2,
            successful: 1,
            failed: 1,
            skipped: 0,
        };

        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: BatchOutput = serde_yaml::from_str(&yaml).expect("deserialize");

        assert_eq!(restored.results.len(), 2);
        assert_eq!(restored.total_items, 2);
        assert_eq!(restored.successful, 1);
        assert_eq!(restored.failed, 1);
        assert_eq!(restored.skipped, 0);

        let first = &restored.results[0];
        assert!(first.success);
        assert!(first.result.is_some());
        assert!(first.error.is_none());

        let second = &restored.results[1];
        assert!(!second.success);
        assert!(second.result.is_none());
        assert_eq!(second.error.as_deref(), Some("activity timeout"));

        // Verify optional fields are omitted from JSON when None.
        let json = serde_json::to_string(&output.results[0]).expect("serialize to JSON");
        assert!(!json.contains("\"error\""));
        assert!(json.contains("\"result\""));

        let json_failed = serde_json::to_string(&output.results[1]).expect("serialize to JSON");
        assert!(!json_failed.contains("\"result\""));
        assert!(json_failed.contains("\"error\""));
    }

    #[test]
    fn test_fail_strategy_default() {
        let strategy = BatchFailStrategy::default();
        assert_eq!(strategy, BatchFailStrategy::StopOnFirst);

        // Verify all variants round-trip through JSON.
        let variants = [
            BatchFailStrategy::StopOnFirst,
            BatchFailStrategy::ContinueAll,
            BatchFailStrategy::SkipFailed,
        ];
        for variant in &variants {
            let json = serde_json::to_string(variant).expect("serialize");
            let restored: BatchFailStrategy =
                serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&restored, variant);
        }

        // Spot-check wire format.
        assert_eq!(
            serde_json::to_string(&BatchFailStrategy::StopOnFirst).unwrap(),
            "\"stop_on_first\""
        );
        assert_eq!(
            serde_json::to_string(&BatchFailStrategy::ContinueAll).unwrap(),
            "\"continue_all\""
        );
        assert_eq!(
            serde_json::to_string(&BatchFailStrategy::SkipFailed).unwrap(),
            "\"skip_failed\""
        );
    }
}

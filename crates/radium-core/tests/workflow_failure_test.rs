//! Tests for failure detection and classification system.

use radium_core::workflow::failure::{
    FailureClassifier, FailureHistory, FailurePolicy, FailureType,
};
use std::time::Duration;

#[test]
fn test_transient_failure_classification() {
    let classifier = FailureClassifier::new();
    let failure_type = classifier.classify_from_string("Connection timeout occurred");

    assert!(matches!(failure_type, FailureType::Transient { .. }));
    assert!(failure_type.is_recoverable());
}

#[test]
fn test_permanent_failure_classification() {
    let classifier = FailureClassifier::new();
    let failure_type = classifier.classify_from_string("Validation failed: invalid input");

    assert!(matches!(failure_type, FailureType::Permanent { .. }));
    assert!(!failure_type.is_recoverable());
}

#[test]
fn test_agent_failure_classification() {
    let classifier = FailureClassifier::new();
    let failure_type = classifier.classify_from_string("Agent not found: code-agent");

    match failure_type {
        FailureType::AgentFailure { agent_id, .. } => {
            assert_eq!(agent_id, "code-agent");
        }
        _ => panic!("Expected AgentFailure"),
    }
    assert!(failure_type.is_recoverable());
}

#[test]
fn test_retry_threshold_enforcement() {
    let policy = FailurePolicy::with_config(3, Duration::from_secs(5), false);
    let mut history = FailureHistory::new("task-1".to_string());

    // Add 3 failures
    for _ in 0..3 {
        history.add_failure(
            FailureType::Transient { reason: "timeout".to_string() },
            "Timeout".to_string(),
        );
    }

    assert!(!history.should_retry(&policy));
    assert_eq!(history.get_retry_count(), 3);
}

#[test]
fn test_failure_history_tracking() {
    let mut history = FailureHistory::new("task-1".to_string());
    assert_eq!(history.get_retry_count(), 0);

    history.add_failure(
        FailureType::Transient { reason: "timeout".to_string() },
        "Connection timeout".to_string(),
    );
    assert_eq!(history.get_retry_count(), 1);

    history.add_failure(
        FailureType::Permanent { reason: "validation".to_string() },
        "Validation failed".to_string(),
    );
    assert_eq!(history.get_retry_count(), 2);
    assert_eq!(history.failures.len(), 2);
}

#[test]
fn test_failure_policy_should_retry() {
    let policy = FailurePolicy::default();
    let history = FailureHistory::new("task-1".to_string());

    assert!(policy.should_retry(
        &history,
        &FailureType::Transient { reason: "timeout".to_string() }
    ));
    assert!(!policy.should_retry(
        &history,
        &FailureType::Permanent { reason: "validation".to_string() }
    ));
}

#[test]
fn test_failure_type_description() {
    let failure = FailureType::Transient { reason: "timeout".to_string() };
    let desc = failure.description();
    assert!(desc.contains("Transient"));
    assert!(desc.contains("timeout"));
}

#[test]
fn test_network_error_classification() {
    let classifier = FailureClassifier::new();
    let failure_type = classifier.classify_from_string("Network connection refused");

    assert!(matches!(failure_type, FailureType::Transient { .. }));
}

#[test]
fn test_validation_error_classification() {
    let classifier = FailureClassifier::new();
    let failure_type = classifier.classify_from_string("Invalid input: missing required field");

    assert!(matches!(failure_type, FailureType::Permanent { .. }));
}

#[test]
fn test_unknown_error_classification() {
    let classifier = FailureClassifier::new();
    let failure_type = classifier.classify_from_string("Something unexpected happened");

    assert!(matches!(failure_type, FailureType::Unknown { .. }));
    assert!(!failure_type.is_recoverable());
}


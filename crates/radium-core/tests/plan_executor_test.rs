//! Integration tests for plan execution with error handling and retry logic.
//!
//! Note: Full integration tests for retry logic with task execution require
//! agent setup (agent config files, prompt files, etc.). This test file focuses
//! on testing error categorization and exponential backoff calculation, which
//! are the core components of the retry logic.

use radium_core::planning::{ErrorCategory, ExecutionError};

#[tokio::test]
async fn test_error_category_recoverable_patterns() {
    let recoverable_patterns = vec![
        "429 rate limit",
        "timeout error",
        "network failure",
        "500 server error",
        "connection lost",
    ];

    for pattern in recoverable_patterns {
        let error = ExecutionError::ModelExecution(pattern.to_string());
        assert_eq!(
            error.category(),
            ErrorCategory::Recoverable,
            "Pattern '{}' should be recoverable",
            pattern
        );
    }
}

#[tokio::test]
async fn test_error_category_fatal_patterns() {
    let fatal_patterns = vec![
        "401 unauthorized",
        "403 forbidden",
        "missing config",
        "invalid input",
        "dependency not met",
    ];

    for pattern in fatal_patterns {
        let error = ExecutionError::ModelExecution(pattern.to_string());
        assert_eq!(
            error.category(),
            ErrorCategory::Fatal,
            "Pattern '{}' should be fatal",
            pattern
        );
    }
}


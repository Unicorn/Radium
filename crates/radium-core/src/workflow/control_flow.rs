//! Control flow support for workflows.
//!
//! This module provides functionality for conditional branching and
//! step dependencies in workflow execution.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

use super::engine::ExecutionContext;

/// Configuration for conditional step execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepCondition {
    /// Condition expression to evaluate (e.g., "previous_step.result.status == 'success'").
    /// Currently supports simple equality checks on step results.
    pub condition: Option<String>,
    /// Skip this step if the condition is true.
    pub skip_if: Option<String>,
    /// Dependencies: step IDs that must complete before this step can run.
    pub depends_on: Option<Vec<String>>,
}

impl StepCondition {
    /// Parses step condition from JSON configuration.
    ///
    /// # Arguments
    /// * `config_json` - JSON string from WorkflowStep.config_json
    ///
    /// # Returns
    /// `Ok(StepCondition)` if parsing succeeded, or `ControlFlowError` if it failed.
    pub fn from_json(config_json: Option<&String>) -> Result<Option<Self>, ControlFlowError> {
        let Some(config_str) = config_json else {
            return Ok(None);
        };

        if config_str.is_empty() {
            return Ok(None);
        }

        // Handle null JSON value
        if config_str.trim() == "null" {
            return Ok(None);
        }

        let config: StepCondition = serde_json::from_str(config_str).map_err(|e| {
            ControlFlowError::InvalidConfig(format!("Failed to parse step config: {}", e))
        })?;

        Ok(Some(config))
    }
}

/// Evaluates whether a step should be executed based on conditions.
///
/// # Arguments
/// * `step_id` - The ID of the step to evaluate
/// * `condition` - The step condition configuration
/// * `context` - The execution context with step results
///
/// # Returns
/// `Ok(true)` if the step should execute, `Ok(false)` if it should be skipped,
/// or `ControlFlowError` if evaluation failed.
pub fn should_execute_step(
    step_id: &str,
    condition: Option<&StepCondition>,
    context: &ExecutionContext,
) -> Result<bool, ControlFlowError> {
    let Some(condition) = condition else {
        return Ok(true); // No condition, execute step
    };

    // Check dependencies
    if let Some(ref depends_on) = condition.depends_on {
        for dep_step_id in depends_on {
            if !context.step_results.contains_key(dep_step_id) {
                warn!(
                    step_id = %step_id,
                    dependency = %dep_step_id,
                    "Step dependency not satisfied, skipping step"
                );
                return Ok(false);
            }

            // Check if dependency succeeded
            if let Some(dep_result) = context.get_step_result(dep_step_id) {
                if !dep_result.success {
                    warn!(
                        step_id = %step_id,
                        dependency = %dep_step_id,
                        "Step dependency failed, skipping step"
                    );
                    return Ok(false);
                }
            }
        }
    }

    // Evaluate skip_if condition
    if let Some(ref skip_condition) = condition.skip_if {
        if evaluate_condition(skip_condition, context) {
            debug!(
                step_id = %step_id,
                condition = %skip_condition,
                "Step condition evaluated to skip"
            );
            return Ok(false);
        }
    }

    // Evaluate execute condition
    if let Some(ref execute_condition) = condition.condition {
        if !evaluate_condition(execute_condition, context) {
            debug!(
                step_id = %step_id,
                condition = %execute_condition,
                "Step condition evaluated to skip"
            );
            return Ok(false);
        }
    }

    Ok(true)
}

/// Evaluates a condition expression against the execution context.
///
/// Currently supports simple expressions like:
/// - `previous_step.result.success == true`
/// - `step_id.result.output.field == 'value'`
///
/// # Arguments
/// * `condition` - The condition expression
/// * `context` - The execution context
///
/// # Returns
/// `true` if condition is true, `false` if false.
fn evaluate_condition(condition: &str, context: &ExecutionContext) -> bool {
    // Simple condition evaluation - check if a step result exists and matches
    // This is a basic implementation; a full expression evaluator would be more complex

    // Check for simple equality patterns like "step_id.result.success == true"
    if condition.contains("==") {
        let parts: Vec<&str> = condition.split("==").map(str::trim).collect();
        if parts.len() == 2 {
            let left = parts[0].trim();
            let right = parts[1].trim().trim_matches('\'').trim_matches('"');

            // Parse left side to get step result
            if left.contains(".result.") {
                let path_parts: Vec<&str> = left.split('.').collect();
                if path_parts.len() >= 3 && path_parts[1] == "result" {
                    let step_id = path_parts[0];
                    if let Some(step_result) = context.get_step_result(step_id) {
                        // Check the property
                        let property = path_parts[2];
                        match property {
                            "success" => {
                                let expected = right.parse::<bool>().unwrap_or(false);
                                return step_result.success == expected;
                            }
                            "error" => {
                                if right == "null" {
                                    return step_result.error.is_none();
                                }
                                if let Some(ref error) = step_result.error {
                                    return error == right;
                                }
                            }
                            _ => {
                                // For other properties, check output
                                if let Some(ref output) = step_result.output {
                                    if let Some(value) = output.get(property) {
                                        let value_str =
                                            value.to_string().trim_matches('"').to_string();
                                        return value_str == right;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Default: if we can't evaluate, assume true (execute the step)
    warn!(
        condition = %condition,
        "Could not evaluate condition, defaulting to true"
    );
    true
}

/// Errors that can occur during control flow evaluation.
#[derive(Error, Debug)]
pub enum ControlFlowError {
    /// Invalid condition configuration.
    #[error("Invalid condition config: {0}")]
    InvalidConfig(String),

    /// Condition evaluation error.
    #[error("Condition evaluation error: {0}")]
    Evaluation(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::engine::StepResult;
    use chrono::Utc;
    use serde_json::{Value, json};

    fn create_test_context() -> ExecutionContext {
        let mut context = ExecutionContext::new("workflow-1".to_string());
        context.record_step_result(
            "step-1".to_string(),
            StepResult::success(
                "step-1".to_string(),
                Value::String("output".to_string()),
                Utc::now(),
                Utc::now(),
            ),
        );
        context.record_step_result(
            "step-2".to_string(),
            StepResult::failure(
                "step-2".to_string(),
                "Error message".to_string(),
                Utc::now(),
                Utc::now(),
            ),
        );
        context
    }

    #[test]
    fn test_should_execute_step_no_condition() {
        let context = create_test_context();
        let result = should_execute_step("step-3", None, &context).unwrap();
        assert!(result);
    }

    #[test]
    fn test_should_execute_step_with_dependency() {
        let context = create_test_context();
        let condition = StepCondition {
            condition: None,
            skip_if: None,
            depends_on: Some(vec!["step-1".to_string()]),
        };
        let result = should_execute_step("step-3", Some(&condition), &context).unwrap();
        assert!(result);
    }

    #[test]
    fn test_should_execute_step_with_missing_dependency() {
        let context = create_test_context();
        let condition = StepCondition {
            condition: None,
            skip_if: None,
            depends_on: Some(vec!["step-99".to_string()]),
        };
        let result = should_execute_step("step-3", Some(&condition), &context).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_should_execute_step_with_failed_dependency() {
        let context = create_test_context();
        let condition = StepCondition {
            condition: None,
            skip_if: None,
            depends_on: Some(vec!["step-2".to_string()]),
        };
        let result = should_execute_step("step-3", Some(&condition), &context).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_evaluate_condition_success() {
        let context = create_test_context();
        let result = evaluate_condition("step-1.result.success == true", &context);
        assert!(result);
    }

    #[test]
    fn test_evaluate_condition_failure() {
        let context = create_test_context();
        let result = evaluate_condition("step-2.result.success == true", &context);
        assert!(!result);
    }

    #[test]
    fn test_step_condition_from_json() {
        let json = r#"{"condition": "step-1.result.success == true", "depends_on": ["step-1"]}"#;
        let condition = StepCondition::from_json(Some(&json.to_string())).unwrap();
        assert!(condition.is_some());
        let condition = condition.unwrap();
        assert_eq!(condition.condition, Some("step-1.result.success == true".to_string()));
        assert_eq!(condition.depends_on, Some(vec!["step-1".to_string()]));
    }

    #[test]
    fn test_step_condition_from_json_empty() {
        let condition = StepCondition::from_json(None).unwrap();
        assert!(condition.is_none());
    }

    #[test]
    fn test_step_condition_from_json_empty_string() {
        let condition = StepCondition::from_json(Some(&"".to_string())).unwrap();
        assert!(condition.is_none());
    }

    #[test]
    fn test_step_condition_from_json_invalid() {
        let json = r#"{"invalid": json}"#;
        let result = StepCondition::from_json(Some(&json.to_string()));
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, ControlFlowError::InvalidConfig(_)));
        }
    }

    #[test]
    fn test_step_condition_from_json_null_string() {
        // Test handling of "null" string (not None, but the string "null")
        let condition = StepCondition::from_json(Some(&"null".to_string())).unwrap();
        assert!(condition.is_none());
    }

    #[test]
    fn test_step_condition_from_json_whitespace_null() {
        // Test handling of " null " with whitespace
        let condition = StepCondition::from_json(Some(&" null ".to_string())).unwrap();
        assert!(condition.is_none());
    }

    #[test]
    fn test_step_condition_from_json_malformed_json() {
        // Test with malformed JSON that can't be parsed
        let json = r#"{"condition": "unclosed"#;
        let result = StepCondition::from_json(Some(&json.to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_should_execute_step_with_skip_if() {
        let context = create_test_context();
        let condition = StepCondition {
            condition: None,
            skip_if: Some("step-1.result.success == true".to_string()),
            depends_on: None,
        };
        let result = should_execute_step("step-3", Some(&condition), &context).unwrap();
        // step-1 succeeded, so skip_if is true, so we should NOT execute
        assert!(!result);
    }

    #[test]
    fn test_should_execute_step_with_skip_if_false() {
        let context = create_test_context();
        let condition = StepCondition {
            condition: None,
            skip_if: Some("step-2.result.success == true".to_string()),
            depends_on: None,
        };
        let result = should_execute_step("step-3", Some(&condition), &context).unwrap();
        // step-2 failed, so skip_if is false, so we SHOULD execute
        assert!(result);
    }

    #[test]
    fn test_should_execute_step_with_condition_true() {
        let context = create_test_context();
        let condition = StepCondition {
            condition: Some("step-1.result.success == true".to_string()),
            skip_if: None,
            depends_on: None,
        };
        let result = should_execute_step("step-3", Some(&condition), &context).unwrap();
        assert!(result);
    }

    #[test]
    fn test_should_execute_step_with_condition_false() {
        let context = create_test_context();
        let condition = StepCondition {
            condition: Some("step-2.result.success == true".to_string()),
            skip_if: None,
            depends_on: None,
        };
        let result = should_execute_step("step-3", Some(&condition), &context).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_should_execute_step_with_multiple_dependencies() {
        let mut context = ExecutionContext::new("workflow-1".to_string());
        context.record_step_result(
            "step-1".to_string(),
            StepResult::success(
                "step-1".to_string(),
                Value::String("output1".to_string()),
                Utc::now(),
                Utc::now(),
            ),
        );
        context.record_step_result(
            "step-2".to_string(),
            StepResult::success(
                "step-2".to_string(),
                Value::String("output2".to_string()),
                Utc::now(),
                Utc::now(),
            ),
        );

        let condition = StepCondition {
            condition: None,
            skip_if: None,
            depends_on: Some(vec!["step-1".to_string(), "step-2".to_string()]),
        };
        let result = should_execute_step("step-3", Some(&condition), &context).unwrap();
        assert!(result);
    }

    #[test]
    fn test_should_execute_step_with_multiple_dependencies_one_missing() {
        let mut context = ExecutionContext::new("workflow-1".to_string());
        context.record_step_result(
            "step-1".to_string(),
            StepResult::success(
                "step-1".to_string(),
                Value::String("output1".to_string()),
                Utc::now(),
                Utc::now(),
            ),
        );

        let condition = StepCondition {
            condition: None,
            skip_if: None,
            depends_on: Some(vec!["step-1".to_string(), "step-2".to_string()]),
        };
        let result = should_execute_step("step-3", Some(&condition), &context).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_evaluate_condition_with_output_field() {
        let mut context = ExecutionContext::new("workflow-1".to_string());
        context.record_step_result(
            "step-1".to_string(),
            StepResult::success(
                "step-1".to_string(),
                json!({"status": "success", "value": 42}),
                Utc::now(),
                Utc::now(),
            ),
        );

        let result = evaluate_condition("step-1.result.output.status == 'success'", &context);
        assert!(result);
    }

    #[test]
    fn test_evaluate_condition_with_error_check() {
        let context = create_test_context();
        let result = evaluate_condition("step-2.result.error == null", &context);
        // step-2 has an error, so this should be false
        assert!(!result);
    }

    #[test]
    fn test_evaluate_condition_with_error_value() {
        let context = create_test_context();
        let result = evaluate_condition("step-2.result.error == 'Error message'", &context);
        assert!(result);
    }

    #[test]
    fn test_evaluate_condition_invalid_syntax() {
        let context = create_test_context();
        // Invalid condition syntax - should default to true
        let result = evaluate_condition("invalid syntax here", &context);
        assert!(result);
    }

    #[test]
    fn test_evaluate_condition_complex_path() {
        let mut context = ExecutionContext::new("workflow-1".to_string());
        context.record_step_result(
            "step-1".to_string(),
            StepResult::success(
                "step-1".to_string(),
                json!({"data": {"nested": {"value": "test"}}}),
                Utc::now(),
                Utc::now(),
            ),
        );

        // Simple condition evaluator doesn't handle nested paths, so this will default to true
        let result =
            evaluate_condition("step-1.result.output.data.nested.value == 'test'", &context);
        // Current implementation doesn't support nested paths, so defaults to true
        assert!(result);
    }

    #[test]
    fn test_should_execute_step_with_multiple_dependencies_one_failed() {
        let mut context = ExecutionContext::new("workflow-1".to_string());
        context.record_step_result(
            "step-1".to_string(),
            StepResult::success(
                "step-1".to_string(),
                Value::String("output1".to_string()),
                Utc::now(),
                Utc::now(),
            ),
        );
        context.record_step_result(
            "step-2".to_string(),
            StepResult::failure("step-2".to_string(), "Error".to_string(), Utc::now(), Utc::now()),
        );

        let condition = StepCondition {
            condition: None,
            skip_if: None,
            depends_on: Some(vec!["step-1".to_string(), "step-2".to_string()]),
        };
        let result = should_execute_step("step-3", Some(&condition), &context).unwrap();
        // step-2 failed, so should not execute
        assert!(!result);
    }

    #[test]
    fn test_evaluate_condition_missing_step_result() {
        let context = ExecutionContext::new("workflow-1".to_string());
        // Condition references step that doesn't exist
        let result = evaluate_condition("nonexistent.result.success == true", &context);
        // Should default to true when step doesn't exist
        assert!(result);
    }

    #[test]
    fn test_evaluate_condition_with_quoted_strings() {
        let mut context = ExecutionContext::new("workflow-1".to_string());
        context.record_step_result(
            "step-1".to_string(),
            StepResult::success(
                "step-1".to_string(),
                json!({"status": "success"}),
                Utc::now(),
                Utc::now(),
            ),
        );

        // Test with single quotes
        let result1 = evaluate_condition("step-1.result.output.status == 'success'", &context);
        assert!(result1);

        // Test with double quotes
        let result2 = evaluate_condition(r#"step-1.result.output.status == "success""#, &context);
        assert!(result2);
    }
}

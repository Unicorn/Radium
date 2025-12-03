//! Loop behavior implementation.
//!
//! Allows agents to request repeating previous workflow steps with:
//! - Maximum iteration limits
//! - Step-back count
//! - Skip lists for specific steps

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::types::{BehaviorAction, BehaviorActionType, BehaviorError, BehaviorEvaluator};

/// Configuration for loop behavior in a module.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoopBehaviorConfig {
    /// Number of steps to go back when looping.
    pub steps: usize,
    /// Maximum number of iterations before stopping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_iterations: Option<usize>,
    /// List of step IDs to skip during loop.
    #[serde(default)]
    pub skip: Vec<String>,
}

/// Decision result from loop evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopDecision {
    /// Whether to repeat the loop.
    pub should_repeat: bool,
    /// Number of steps to go back.
    pub steps_back: usize,
    /// List of step IDs to skip.
    pub skip_list: Vec<String>,
    /// Human-readable reason for the decision.
    pub reason: Option<String>,
}

/// Context for loop evaluation.
#[derive(Debug, Clone)]
pub struct LoopEvaluationContext {
    /// Current iteration count for this loop.
    pub iteration_count: usize,
    /// Loop behavior configuration.
    pub config: Option<LoopBehaviorConfig>,
}

impl LoopEvaluationContext {
    /// Creates a new loop evaluation context.
    pub fn new(iteration_count: usize, config: Option<LoopBehaviorConfig>) -> Self {
        Self { iteration_count, config }
    }
}

/// Evaluates loop behavior based on behavior.json and configuration.
pub struct LoopEvaluator;

impl LoopEvaluator {
    /// Creates a new loop evaluator.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates loop behavior with typed context.
    ///
    /// # Arguments
    /// * `behavior_file` - Path to behavior.json
    /// * `output` - Output from agent execution
    /// * `context` - Loop evaluation context
    ///
    /// # Returns
    /// `Ok(Some(LoopDecision))` if loop should be triggered,
    /// `Ok(None)` if no loop behavior,
    /// `Err(BehaviorError)` on evaluation error.
    pub fn evaluate_loop(
        &self,
        behavior_file: &Path,
        _output: &str,
        context: &LoopEvaluationContext,
    ) -> Result<Option<LoopDecision>, BehaviorError> {
        // Must have loop configuration
        let Some(config) = &context.config else {
            return Ok(None);
        };

        // Check for behavior action
        let Some(action) = BehaviorAction::read_from_file(behavior_file)? else {
            return Ok(None);
        };

        // Check max iterations
        if let Some(max_iter) = config.max_iterations {
            if context.iteration_count + 1 > max_iter {
                return Ok(Some(LoopDecision {
                    should_repeat: false,
                    steps_back: config.steps,
                    skip_list: config.skip.clone(),
                    reason: Some(format!("loop limit reached ({})", max_iter)),
                }));
            }
        }

        // Handle behavior action
        match action.action {
            BehaviorActionType::Loop => Ok(Some(LoopDecision {
                should_repeat: true,
                steps_back: config.steps,
                skip_list: config.skip.clone(),
                reason: action.reason,
            })),
            BehaviorActionType::Stop => Ok(Some(LoopDecision {
                should_repeat: false,
                steps_back: config.steps,
                skip_list: config.skip.clone(),
                reason: action.reason,
            })),
            _ => {
                // Continue, Checkpoint, Trigger = no loop behavior
                Ok(None)
            }
        }
    }
}

impl Default for LoopEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl BehaviorEvaluator for LoopEvaluator {
    type Decision = LoopDecision;

    fn evaluate(
        &self,
        behavior_file: &Path,
        output: &str,
        context: &dyn std::any::Any,
    ) -> Result<Option<Self::Decision>, BehaviorError> {
        let loop_context = context
            .downcast_ref::<LoopEvaluationContext>()
            .ok_or_else(|| BehaviorError::InvalidConfig("Invalid context type".to_string()))?;

        self.evaluate_loop(behavior_file, output, loop_context)
    }
}

/// Tracks loop iteration counts for workflow steps.
#[derive(Debug, Clone, Default)]
pub struct LoopCounters {
    counters: HashMap<String, usize>,
}

impl LoopCounters {
    /// Creates a new loop counter tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the current iteration count for a loop key.
    ///
    /// # Arguments
    /// * `key` - Loop key (typically "module-id:step-index")
    ///
    /// # Returns
    /// The current iteration count (0 if not yet tracked).
    pub fn get(&self, key: &str) -> usize {
        self.counters.get(key).copied().unwrap_or(0)
    }

    /// Increments the iteration count for a loop key.
    ///
    /// # Arguments
    /// * `key` - Loop key (typically "module-id:step-index")
    ///
    /// # Returns
    /// The new iteration count.
    pub fn increment(&mut self, key: &str) -> usize {
        let count = self.counters.entry(key.to_string()).or_insert(0);
        *count += 1;
        *count
    }

    /// Resets the iteration count for a loop key.
    ///
    /// # Arguments
    /// * `key` - Loop key (typically "module-id:step-index")
    pub fn reset(&mut self, key: &str) {
        self.counters.insert(key.to_string(), 0);
    }

    /// Clears all loop counters.
    pub fn clear(&mut self) {
        self.counters.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_loop_evaluator_no_config() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        let evaluator = LoopEvaluator::new();
        let context = LoopEvaluationContext::new(0, None);

        let result = evaluator.evaluate_loop(&behavior_file, "", &context);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_loop_evaluator_no_behavior_file() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        let evaluator = LoopEvaluator::new();
        let config = LoopBehaviorConfig { steps: 2, max_iterations: Some(5), skip: vec![] };
        let context = LoopEvaluationContext::new(0, Some(config));

        let result = evaluator.evaluate_loop(&behavior_file, "", &context);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_loop_evaluator_loop_action() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        // Write loop action
        let action = BehaviorAction::new(BehaviorActionType::Loop).with_reason("Tests failing");
        action.write_to_file(&behavior_file).unwrap();

        let evaluator = LoopEvaluator::new();
        let config = LoopBehaviorConfig {
            steps: 3,
            max_iterations: Some(10),
            skip: vec!["step-a".to_string()],
        };
        let context = LoopEvaluationContext::new(2, Some(config));

        let result = evaluator.evaluate_loop(&behavior_file, "", &context).unwrap();
        assert!(result.is_some());

        let decision = result.unwrap();
        assert!(decision.should_repeat);
        assert_eq!(decision.steps_back, 3);
        assert_eq!(decision.skip_list, vec!["step-a"]);
        assert_eq!(decision.reason.as_deref(), Some("Tests failing"));
    }

    #[test]
    fn test_loop_evaluator_max_iterations_reached() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        // Write loop action
        let action = BehaviorAction::new(BehaviorActionType::Loop);
        action.write_to_file(&behavior_file).unwrap();

        let evaluator = LoopEvaluator::new();
        let config = LoopBehaviorConfig { steps: 2, max_iterations: Some(5), skip: vec![] };
        let context = LoopEvaluationContext::new(5, Some(config)); // Iteration 5, max is 5

        let result = evaluator.evaluate_loop(&behavior_file, "", &context).unwrap();
        assert!(result.is_some());

        let decision = result.unwrap();
        assert!(!decision.should_repeat);
        assert_eq!(decision.reason.as_deref(), Some("loop limit reached (5)"));
    }

    #[test]
    fn test_loop_evaluator_stop_action() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        // Write stop action
        let action = BehaviorAction::new(BehaviorActionType::Stop).with_reason("All tests pass");
        action.write_to_file(&behavior_file).unwrap();

        let evaluator = LoopEvaluator::new();
        let config = LoopBehaviorConfig { steps: 2, max_iterations: Some(10), skip: vec![] };
        let context = LoopEvaluationContext::new(2, Some(config));

        let result = evaluator.evaluate_loop(&behavior_file, "", &context).unwrap();
        assert!(result.is_some());

        let decision = result.unwrap();
        assert!(!decision.should_repeat);
        assert_eq!(decision.reason.as_deref(), Some("All tests pass"));
    }

    #[test]
    fn test_loop_evaluator_continue_action() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        // Write continue action (should not trigger loop)
        let action = BehaviorAction::new(BehaviorActionType::Continue);
        action.write_to_file(&behavior_file).unwrap();

        let evaluator = LoopEvaluator::new();
        let config = LoopBehaviorConfig { steps: 2, max_iterations: Some(10), skip: vec![] };
        let context = LoopEvaluationContext::new(2, Some(config));

        let result = evaluator.evaluate_loop(&behavior_file, "", &context).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_loop_counters() {
        let mut counters = LoopCounters::new();

        assert_eq!(counters.get("loop-1"), 0);

        let count1 = counters.increment("loop-1");
        assert_eq!(count1, 1);
        assert_eq!(counters.get("loop-1"), 1);

        let count2 = counters.increment("loop-1");
        assert_eq!(count2, 2);

        counters.reset("loop-1");
        assert_eq!(counters.get("loop-1"), 0);

        counters.increment("loop-2");
        counters.increment("loop-3");
        assert_eq!(counters.get("loop-2"), 1);
        assert_eq!(counters.get("loop-3"), 1);

        counters.clear();
        assert_eq!(counters.get("loop-2"), 0);
        assert_eq!(counters.get("loop-3"), 0);
    }

    #[test]
    fn test_loop_evaluator_skip_list_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        let action = BehaviorAction::new(BehaviorActionType::Loop).with_reason("Retry needed");
        action.write_to_file(&behavior_file).unwrap();

        let evaluator = LoopEvaluator::new();
        let config = LoopBehaviorConfig {
            steps: 2,
            max_iterations: Some(10),
            skip: vec!["step-1".to_string(), "step-2".to_string()],
        };
        let context = LoopEvaluationContext::new(1, Some(config));

        let result = evaluator.evaluate_loop(&behavior_file, "", &context).unwrap();
        assert!(result.is_some());

        let decision = result.unwrap();
        assert!(decision.should_repeat);
        assert_eq!(decision.skip_list.len(), 2);
        assert!(decision.skip_list.contains(&"step-1".to_string()));
        assert!(decision.skip_list.contains(&"step-2".to_string()));
    }

    #[test]
    fn test_loop_evaluator_no_max_iterations() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        let action = BehaviorAction::new(BehaviorActionType::Loop);
        action.write_to_file(&behavior_file).unwrap();

        let evaluator = LoopEvaluator::new();
        let config = LoopBehaviorConfig {
            steps: 2,
            max_iterations: None, // No max iterations
            skip: vec![],
        };
        let context = LoopEvaluationContext::new(100, Some(config)); // High iteration count

        let result = evaluator.evaluate_loop(&behavior_file, "", &context).unwrap();
        // Should still allow loop since no max_iterations set
        assert!(result.is_some());
        let decision = result.unwrap();
        assert!(decision.should_repeat);
    }

    #[test]
    fn test_loop_evaluator_empty_skip_list() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        let action = BehaviorAction::new(BehaviorActionType::Loop);
        action.write_to_file(&behavior_file).unwrap();

        let evaluator = LoopEvaluator::new();
        let config = LoopBehaviorConfig {
            steps: 3,
            max_iterations: Some(10),
            skip: vec![], // Empty skip list
        };
        let context = LoopEvaluationContext::new(2, Some(config));

        let result = evaluator.evaluate_loop(&behavior_file, "", &context).unwrap();
        assert!(result.is_some());

        let decision = result.unwrap();
        assert!(decision.should_repeat);
        assert!(decision.skip_list.is_empty());
    }
}

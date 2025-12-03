//! Trigger behavior implementation.
//!
//! Allows agents to dynamically trigger other agents during workflow execution.

use serde::{Deserialize, Serialize};
use std::path::Path;

use super::types::{BehaviorAction, BehaviorActionType, BehaviorError, BehaviorEvaluator};

/// Configuration for trigger behavior in a module.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerBehaviorConfig {
    /// Default agent ID to trigger (can be overridden in behavior.json).
    pub trigger_agent_id: Option<String>,
}

/// Decision result from trigger evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerDecision {
    /// Whether to trigger an agent.
    pub should_trigger: bool,
    /// Agent ID to trigger.
    pub trigger_agent_id: String,
    /// Human-readable reason for the decision.
    pub reason: Option<String>,
}

/// Context for trigger evaluation.
#[derive(Debug, Clone)]
pub struct TriggerEvaluationContext {
    /// Trigger behavior configuration.
    pub config: Option<TriggerBehaviorConfig>,
}

impl TriggerEvaluationContext {
    /// Creates a new trigger evaluation context.
    pub fn new(config: Option<TriggerBehaviorConfig>) -> Self {
        Self { config }
    }
}

/// Evaluates trigger behavior based on behavior.json and configuration.
pub struct TriggerEvaluator;

impl TriggerEvaluator {
    /// Creates a new trigger evaluator.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates trigger behavior with typed context.
    ///
    /// # Arguments
    /// * `behavior_file` - Path to behavior.json
    /// * `output` - Output from agent execution
    /// * `context` - Trigger evaluation context
    ///
    /// # Returns
    /// `Ok(Some(TriggerDecision))` if trigger should be activated,
    /// `Ok(None)` if no trigger behavior,
    /// `Err(BehaviorError)` on evaluation error.
    pub fn evaluate_trigger(
        &self,
        behavior_file: &Path,
        _output: &str,
        context: &TriggerEvaluationContext,
    ) -> Result<Option<TriggerDecision>, BehaviorError> {
        // Must have trigger configuration
        let config = match &context.config {
            Some(cfg) => cfg,
            None => return Ok(None),
        };

        // Check for behavior action
        let action = match BehaviorAction::read_from_file(behavior_file)? {
            Some(a) => a,
            None => return Ok(None),
        };

        // Only handle trigger actions
        if action.action != BehaviorActionType::Trigger {
            return Ok(None);
        }

        // Get target agent ID (from behavior.json or config)
        let target_agent_id = action
            .trigger_agent_id
            .or_else(|| config.trigger_agent_id.clone())
            .ok_or_else(|| {
            BehaviorError::MissingField(
                "triggerAgentId required in behavior.json or module configuration".to_string(),
            )
        })?;

        Ok(Some(TriggerDecision {
            should_trigger: true,
            trigger_agent_id: target_agent_id,
            reason: action.reason,
        }))
    }
}

impl Default for TriggerEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl BehaviorEvaluator for TriggerEvaluator {
    type Decision = TriggerDecision;

    fn evaluate(
        &self,
        behavior_file: &Path,
        output: &str,
        context: &dyn std::any::Any,
    ) -> Result<Option<Self::Decision>, BehaviorError> {
        let trigger_context = context
            .downcast_ref::<TriggerEvaluationContext>()
            .ok_or_else(|| BehaviorError::InvalidConfig("Invalid context type".to_string()))?;

        self.evaluate_trigger(behavior_file, output, trigger_context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_trigger_evaluator_no_config() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        let evaluator = TriggerEvaluator::new();
        let context = TriggerEvaluationContext::new(None);

        let result = evaluator.evaluate_trigger(&behavior_file, "", &context);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_trigger_evaluator_no_behavior_file() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        let evaluator = TriggerEvaluator::new();
        let config = TriggerBehaviorConfig { trigger_agent_id: Some("fallback-agent".to_string()) };
        let context = TriggerEvaluationContext::new(Some(config));

        let result = evaluator.evaluate_trigger(&behavior_file, "", &context);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_trigger_evaluator_trigger_action_with_config() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        // Write trigger action (agent ID from config)
        let action = BehaviorAction::new(BehaviorActionType::Trigger).with_reason("Error detected");
        action.write_to_file(&behavior_file).unwrap();

        let evaluator = TriggerEvaluator::new();
        let config = TriggerBehaviorConfig { trigger_agent_id: Some("error-handler".to_string()) };
        let context = TriggerEvaluationContext::new(Some(config));

        let result = evaluator.evaluate_trigger(&behavior_file, "", &context).unwrap();
        assert!(result.is_some());

        let decision = result.unwrap();
        assert!(decision.should_trigger);
        assert_eq!(decision.trigger_agent_id, "error-handler");
        assert_eq!(decision.reason.as_deref(), Some("Error detected"));
    }

    #[test]
    fn test_trigger_evaluator_trigger_action_override_config() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        // Write trigger action with agent ID override
        let action = BehaviorAction::new(BehaviorActionType::Trigger)
            .with_trigger_agent("specific-handler")
            .with_reason("Need special handling");
        action.write_to_file(&behavior_file).unwrap();

        let evaluator = TriggerEvaluator::new();
        let config =
            TriggerBehaviorConfig { trigger_agent_id: Some("default-handler".to_string()) };
        let context = TriggerEvaluationContext::new(Some(config));

        let result = evaluator.evaluate_trigger(&behavior_file, "", &context).unwrap();
        assert!(result.is_some());

        let decision = result.unwrap();
        assert!(decision.should_trigger);
        assert_eq!(decision.trigger_agent_id, "specific-handler");
        assert_eq!(decision.reason.as_deref(), Some("Need special handling"));
    }

    #[test]
    fn test_trigger_evaluator_trigger_action_no_agent_id() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        // Write trigger action without agent ID
        let action = BehaviorAction::new(BehaviorActionType::Trigger);
        action.write_to_file(&behavior_file).unwrap();

        let evaluator = TriggerEvaluator::new();
        let config = TriggerBehaviorConfig {
            trigger_agent_id: None, // No default either
        };
        let context = TriggerEvaluationContext::new(Some(config));

        let result = evaluator.evaluate_trigger(&behavior_file, "", &context);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BehaviorError::MissingField(_)));
    }

    #[test]
    fn test_trigger_evaluator_non_trigger_action() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        // Write loop action (should not trigger)
        let action = BehaviorAction::new(BehaviorActionType::Loop);
        action.write_to_file(&behavior_file).unwrap();

        let evaluator = TriggerEvaluator::new();
        let config = TriggerBehaviorConfig { trigger_agent_id: Some("handler".to_string()) };
        let context = TriggerEvaluationContext::new(Some(config));

        let result = evaluator.evaluate_trigger(&behavior_file, "", &context).unwrap();
        assert!(result.is_none());
    }
}

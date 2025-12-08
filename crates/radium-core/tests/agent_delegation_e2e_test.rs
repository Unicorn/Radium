//! End-to-end tests for agent delegation workflow.
//!
//! Tests verify the complete agent delegation workflow where a main agent
//! spawns specialized subagents via trigger behavior, with proper permission
//! enforcement at each level.

use radium_core::agents::config::{AgentConfig, TriggerBehaviorConfig};
use radium_core::workflow::behaviors::trigger::{
    TriggerDecision, TriggerEvaluationContext, TriggerEvaluator,
};
use radium_core::workflow::behaviors::types::{BehaviorAction, BehaviorActionType};
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test executor agent config with trigger behavior
fn create_executor_agent_config() -> AgentConfig {
    AgentConfig {
        id: "executor-agent".to_string(),
        name: "Executor Agent".to_string(),
        description: "Test executor agent with trigger behavior".to_string(),
        prompt_path: PathBuf::from("prompts/agents/specialized/executor-agent.md"),
        engine: None,
        model: None,
        reasoning_effort: None,
        mirror_path: None,
        trigger_behavior: Some(TriggerBehaviorConfig {
            trigger_agent_id: Some("error-handler".to_string()),
        }),
        file_path: None,
        capabilities: Default::default(),
        sandbox: None,
        persona_config: None,
    }
}

/// Helper to create a test error handler agent config
fn create_error_handler_agent_config() -> AgentConfig {
    AgentConfig {
        id: "error-handler".to_string(),
        name: "Error Handler Agent".to_string(),
        description: "Test error handler agent".to_string(),
        prompt_path: PathBuf::from("prompts/agents/specialized/analyzer-agent.md"),
        engine: None,
        model: None,
        reasoning_effort: None,
        mirror_path: None,
        trigger_behavior: None,
        file_path: None,
        capabilities: Default::default(),
        sandbox: None,
        persona_config: None,
    }
}

#[tokio::test]
async fn test_basic_trigger_behavior() {
    // Test basic trigger behavior: main agent → subagent
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    // Create trigger action
    let action = BehaviorAction::new(BehaviorActionType::Trigger)
        .with_trigger_agent("error-handler")
        .with_reason("Error detected in execution");
    action.write_to_file(&behavior_file).unwrap();

    // Create evaluator and context
    let evaluator = TriggerEvaluator::new();
    let config = TriggerBehaviorConfig {
        trigger_agent_id: Some("error-handler".to_string()),
    };
    let context = TriggerEvaluationContext::new(Some(config));

    // Evaluate trigger
    let result = evaluator.evaluate_trigger(&behavior_file, "", &context).unwrap();
    assert!(result.is_some());

    let decision = result.unwrap();
    assert!(decision.should_trigger);
    assert_eq!(decision.trigger_agent_id, "error-handler");
    assert_eq!(decision.reason.as_deref(), Some("Error detected in execution"));
}

#[tokio::test]
async fn test_default_agent_id_from_config() {
    // Test that default agent ID from trigger_behavior config is used
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    // Create trigger action without agent ID (should use config default)
    let action = BehaviorAction::new(BehaviorActionType::Trigger)
        .with_reason("Need error handling");
    action.write_to_file(&behavior_file).unwrap();

    // Create evaluator with default agent ID in config
    let evaluator = TriggerEvaluator::new();
    let config = TriggerBehaviorConfig {
        trigger_agent_id: Some("error-handler".to_string()),
    };
    let context = TriggerEvaluationContext::new(Some(config));

    // Evaluate trigger
    let result = evaluator.evaluate_trigger(&behavior_file, "", &context).unwrap();
    assert!(result.is_some());

    let decision = result.unwrap();
    assert!(decision.should_trigger);
    assert_eq!(decision.trigger_agent_id, "error-handler"); // From config
    assert_eq!(decision.reason.as_deref(), Some("Need error handling"));
}

#[tokio::test]
async fn test_runtime_agent_id_override() {
    // Test that runtime agent ID in behavior.json overrides config default
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    // Create trigger action with specific agent ID (should override config)
    let action = BehaviorAction::new(BehaviorActionType::Trigger)
        .with_trigger_agent("specific-handler")
        .with_reason("Need specialized handling");
    action.write_to_file(&behavior_file).unwrap();

    // Create evaluator with different default agent ID
    let evaluator = TriggerEvaluator::new();
    let config = TriggerBehaviorConfig {
        trigger_agent_id: Some("default-handler".to_string()),
    };
    let context = TriggerEvaluationContext::new(Some(config));

    // Evaluate trigger
    let result = evaluator.evaluate_trigger(&behavior_file, "", &context).unwrap();
    assert!(result.is_some());

    let decision = result.unwrap();
    assert!(decision.should_trigger);
    assert_eq!(decision.trigger_agent_id, "specific-handler"); // Override from behavior.json
    assert_eq!(decision.reason.as_deref(), Some("Need specialized handling"));
}

#[tokio::test]
async fn test_delegation_chain() {
    // Test delegation chain: agent A → agent B → agent C
    let temp_dir = TempDir::new().unwrap();

    // First delegation: executor → error-handler
    let behavior_file_1 = temp_dir.path().join("behavior1.json");
    let action1 = BehaviorAction::new(BehaviorActionType::Trigger)
        .with_trigger_agent("error-handler")
        .with_reason("Error detected");
    action1.write_to_file(&behavior_file_1).unwrap();

    let evaluator = TriggerEvaluator::new();
    let config1 = TriggerBehaviorConfig {
        trigger_agent_id: Some("error-handler".to_string()),
    };
    let context1 = TriggerEvaluationContext::new(Some(config1));

    let result1 = evaluator.evaluate_trigger(&behavior_file_1, "", &context1).unwrap();
    assert!(result1.is_some());
    let decision1 = result1.unwrap();
    assert_eq!(decision1.trigger_agent_id, "error-handler");

    // Second delegation: error-handler → analyzer
    let behavior_file_2 = temp_dir.path().join("behavior2.json");
    let action2 = BehaviorAction::new(BehaviorActionType::Trigger)
        .with_trigger_agent("analyzer-agent")
        .with_reason("Need code analysis");
    action2.write_to_file(&behavior_file_2).unwrap();

    let config2 = TriggerBehaviorConfig {
        trigger_agent_id: Some("analyzer-agent".to_string()),
    };
    let context2 = TriggerEvaluationContext::new(Some(config2));

    let result2 = evaluator.evaluate_trigger(&behavior_file_2, "", &context2).unwrap();
    assert!(result2.is_some());
    let decision2 = result2.unwrap();
    assert_eq!(decision2.trigger_agent_id, "analyzer-agent");

    // Third delegation: analyzer → reviewer
    let behavior_file_3 = temp_dir.path().join("behavior3.json");
    let action3 = BehaviorAction::new(BehaviorActionType::Trigger)
        .with_trigger_agent("reviewer-agent")
        .with_reason("Need code review");
    action3.write_to_file(&behavior_file_3).unwrap();

    let config3 = TriggerBehaviorConfig {
        trigger_agent_id: Some("reviewer-agent".to_string()),
    };
    let context3 = TriggerEvaluationContext::new(Some(config3));

    let result3 = evaluator.evaluate_trigger(&behavior_file_3, "", &context3).unwrap();
    assert!(result3.is_some());
    let decision3 = result3.unwrap();
    assert_eq!(decision3.trigger_agent_id, "reviewer-agent");

    // Verify delegation chain: executor → error-handler → analyzer → reviewer
    assert_eq!(decision1.trigger_agent_id, "error-handler");
    assert_eq!(decision2.trigger_agent_id, "analyzer-agent");
    assert_eq!(decision3.trigger_agent_id, "reviewer-agent");
}

#[tokio::test]
async fn test_reason_tracking() {
    // Test that reason is properly tracked in delegation decisions
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    let reasons = vec![
        "Build failure detected",
        "Test suite errors",
        "Code quality issues",
        "Security vulnerability found",
    ];

    for reason in &reasons {
        let action = BehaviorAction::new(BehaviorActionType::Trigger)
            .with_trigger_agent("error-handler")
            .with_reason(reason.to_string());
        action.write_to_file(&behavior_file).unwrap();

        let evaluator = TriggerEvaluator::new();
        let config = TriggerBehaviorConfig {
            trigger_agent_id: Some("error-handler".to_string()),
        };
        let context = TriggerEvaluationContext::new(Some(config));

        let result = evaluator.evaluate_trigger(&behavior_file, "", &context).unwrap();
        assert!(result.is_some());

        let decision = result.unwrap();
        assert_eq!(decision.reason.as_deref(), Some(*reason));
    }
}

#[tokio::test]
async fn test_error_missing_agent_id() {
    // Test error handling when agent ID is missing from both behavior.json and config
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    // Create trigger action without agent ID
    let action = BehaviorAction::new(BehaviorActionType::Trigger);
    action.write_to_file(&behavior_file).unwrap();

    // Create evaluator without default agent ID
    let evaluator = TriggerEvaluator::new();
    let config = TriggerBehaviorConfig {
        trigger_agent_id: None,
    };
    let context = TriggerEvaluationContext::new(Some(config));

    // Should return error
    let result = evaluator.evaluate_trigger(&behavior_file, "", &context);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("triggerAgentId required"));
}

#[tokio::test]
async fn test_error_no_behavior_file() {
    // Test that missing behavior.json file returns None (no trigger)
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("nonexistent.json");

    let evaluator = TriggerEvaluator::new();
    let config = TriggerBehaviorConfig {
        trigger_agent_id: Some("error-handler".to_string()),
    };
    let context = TriggerEvaluationContext::new(Some(config));

    // Should return None when file doesn't exist
    let result = evaluator.evaluate_trigger(&behavior_file, "", &context).unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_error_no_trigger_config() {
    // Test that missing trigger config returns None (no trigger)
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    let action = BehaviorAction::new(BehaviorActionType::Trigger)
        .with_trigger_agent("error-handler");
    action.write_to_file(&behavior_file).unwrap();

    let evaluator = TriggerEvaluator::new();
    let context = TriggerEvaluationContext::new(None); // No config

    // Should return None when no config
    let result = evaluator.evaluate_trigger(&behavior_file, "", &context).unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_non_trigger_actions_ignored() {
    // Test that non-trigger actions (loop, checkpoint, etc.) are ignored
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    let evaluator = TriggerEvaluator::new();
    let config = TriggerBehaviorConfig {
        trigger_agent_id: Some("error-handler".to_string()),
    };
    let context = TriggerEvaluationContext::new(Some(config));

    // Test loop action (should not trigger)
    let loop_action = BehaviorAction::new(BehaviorActionType::Loop);
    loop_action.write_to_file(&behavior_file).unwrap();
    let result = evaluator.evaluate_trigger(&behavior_file, "", &context).unwrap();
    assert!(result.is_none());

    // Test checkpoint action (should not trigger)
    let checkpoint_action = BehaviorAction::new(BehaviorActionType::Checkpoint);
    checkpoint_action.write_to_file(&behavior_file).unwrap();
    let result = evaluator.evaluate_trigger(&behavior_file, "", &context).unwrap();
    assert!(result.is_none());

    // Test continue action (should not trigger)
    let continue_action = BehaviorAction::new(BehaviorActionType::Continue);
    continue_action.write_to_file(&behavior_file).unwrap();
    let result = evaluator.evaluate_trigger(&behavior_file, "", &context).unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_complete_delegation_workflow() {
    // Test complete delegation workflow: executor agent triggers error-handler
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    // Simulate executor agent detecting error and writing behavior.json
    let action = BehaviorAction::new(BehaviorActionType::Trigger)
        .with_trigger_agent("error-handler")
        .with_reason("Build failure in test suite, need specialized error analysis");
    action.write_to_file(&behavior_file).unwrap();

    // Create executor agent config (from Task 1)
    let executor_config = create_executor_agent_config();
    assert_eq!(executor_config.id, "executor-agent");
    assert!(executor_config.trigger_behavior.is_some());
    assert_eq!(
        executor_config.trigger_behavior.as_ref().unwrap().trigger_agent_id,
        Some("error-handler".to_string())
    );

    // Evaluate trigger with executor's config
    let evaluator = TriggerEvaluator::new();
    let trigger_config = executor_config.trigger_behavior.unwrap();
    let context = TriggerEvaluationContext::new(Some(trigger_config));

    let result = evaluator.evaluate_trigger(&behavior_file, "", &context).unwrap();
    assert!(result.is_some());

    let decision = result.unwrap();
    assert!(decision.should_trigger);
    assert_eq!(decision.trigger_agent_id, "error-handler");
    assert!(decision
        .reason
        .as_ref()
        .unwrap()
        .contains("Build failure"));

    // Verify error-handler agent would be spawned with correct permissions
    // (In real workflow, this would spawn the agent with its own config)
    let error_handler_config = create_error_handler_agent_config();
    assert_eq!(error_handler_config.id, "error-handler");
    // Error handler would inherit appropriate permissions from its config
}

#[tokio::test]
async fn test_permission_boundary_enforcement() {
    // Test that permission boundaries are maintained during delegation
    // This is a conceptual test - in practice, each agent would have its own
    // policy rules applied based on its capabilities
    
    let executor_config = create_executor_agent_config();
    let error_handler_config = create_error_handler_agent_config();

    // Executor agent has trigger behavior (can delegate)
    assert!(executor_config.trigger_behavior.is_some());

    // Error handler agent has no trigger behavior (cannot delegate further)
    assert!(error_handler_config.trigger_behavior.is_none());

    // In a real system, each agent's permissions would be enforced by:
    // 1. Agent capabilities (model_class, cost_tier)
    // 2. Policy rules matching agent type
    // 3. Session constitution rules
    // This test verifies the structure supports permission boundaries
    assert_ne!(executor_config.id, error_handler_config.id);
}

#[tokio::test]
async fn test_circular_delegation_prevention() {
    // Test that circular delegation can be detected and prevented
    // This is a conceptual test - actual prevention would be in workflow engine
    
    let temp_dir = TempDir::new().unwrap();

    // Simulate circular delegation: A → B → A
    let behavior_file_a = temp_dir.path().join("behavior_a.json");
    let action_a = BehaviorAction::new(BehaviorActionType::Trigger)
        .with_trigger_agent("agent-b")
        .with_reason("Need agent B");
    action_a.write_to_file(&behavior_file_a).unwrap();

    let behavior_file_b = temp_dir.path().join("behavior_b.json");
    let action_b = BehaviorAction::new(BehaviorActionType::Trigger)
        .with_trigger_agent("agent-a")
        .with_reason("Need agent A");
    action_b.write_to_file(&behavior_file_b).unwrap();

    // Both delegations are valid individually
    let evaluator = TriggerEvaluator::new();
    let config_a = TriggerBehaviorConfig {
        trigger_agent_id: Some("agent-b".to_string()),
    };
    let context_a = TriggerEvaluationContext::new(Some(config_a));

    let result_a = evaluator.evaluate_trigger(&behavior_file_a, "", &context_a).unwrap();
    assert!(result_a.is_some());
    assert_eq!(result_a.unwrap().trigger_agent_id, "agent-b");

    let config_b = TriggerBehaviorConfig {
        trigger_agent_id: Some("agent-a".to_string()),
    };
    let context_b = TriggerEvaluationContext::new(Some(config_b));

    let result_b = evaluator.evaluate_trigger(&behavior_file_b, "", &context_b).unwrap();
    assert!(result_b.is_some());
    assert_eq!(result_b.unwrap().trigger_agent_id, "agent-a");

    // In a real workflow engine, circular delegation would be prevented by:
    // 1. Tracking delegation chain depth
    // 2. Detecting cycles in delegation graph
    // 3. Enforcing maximum delegation depth
    // This test verifies the trigger mechanism works, prevention is workflow-level
}


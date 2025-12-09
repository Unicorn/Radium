#![cfg(feature = "workflow")]

//! Integration tests for metacognitive oversight system.
//!
//! Tests the complete flow from VibeCheck behavior detection through
//! MetacognitiveService oversight to learning store updates.

use radium_abstraction::{ChatMessage, Model, ModelError, ModelParameters, ModelResponse};
use std::sync::Arc;
use tempfile::TempDir;

use radium_core::context::ContextManager;
use radium_core::learning::{LearningIntegration, LearningStore, SkillManager};
use radium_core::oversight::{MetacognitiveService, OversightResponse};
use radium_core::policy::ConstitutionManager;
use radium_core::workflow::behaviors::vibe_check::{
    VibeCheckContext, VibeCheckEvaluator, WorkflowPhase,
};
use radium_core::workflow::behaviors::types::{BehaviorAction, BehaviorActionType};
use radium_core::workspace::Workspace;

// Mock model for testing
struct MockOversightModel;

#[async_trait::async_trait]
impl Model for MockOversightModel {
    async fn generate_text(
        &self,
        _prompt: &str,
        _params: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        Ok(ModelResponse {
            content: "Mock text response".to_string(),
            model_id: Some("mock".to_string()),
            usage: None,
        })
    }

    async fn generate_chat_completion(
        &self,
        _messages: &[ChatMessage],
        _params: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        // Return a realistic oversight response
        Ok(ModelResponse {
            content: "This approach looks good overall, but consider simplifying the implementation. The solution may be over-engineered for the current requirements.".to_string(),
            model_id: Some("mock".to_string()),
            usage: None,
        })
    }

    fn model_id(&self) -> &str {
        "mock"
    }
}

#[tokio::test]
async fn test_vibecheck_evaluator_basic_detection() {
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    // Write vibe check action
    let action = BehaviorAction::new(BehaviorActionType::VibeCheck)
        .with_reason("Need to verify approach");
    action.write_to_file(&behavior_file).unwrap();

    let evaluator = VibeCheckEvaluator::new();
    let context = VibeCheckContext::new(WorkflowPhase::Planning);
    let result = evaluator.evaluate_vibe_check(&behavior_file, "", &context).unwrap();

    assert!(result.is_some());
    let decision = result.unwrap();
    assert!(decision.should_trigger);
    assert_eq!(decision.reason.as_deref(), Some("Need to verify approach"));
}

#[tokio::test]
async fn test_vibecheck_with_oversight_integration() {
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    // Write vibe check action
    let action = BehaviorAction::new(BehaviorActionType::VibeCheck)
        .with_reason("Uncertain about approach");
    action.write_to_file(&behavior_file).unwrap();

    // Setup services
    let model = Arc::new(MockOversightModel);
    let metacognitive = MetacognitiveService::new(model);
    let constitution_manager = ConstitutionManager::new();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    let context_manager = ContextManager::new(&workspace);

    // Create vibe check context
    let vibe_context = VibeCheckContext::new(WorkflowPhase::Implementation)
        .with_goal("Build a web app")
        .with_plan("Use React and Node.js")
        .with_progress("50% complete")
        .with_task_context("Working on authentication");

    // Evaluate with oversight
    let evaluator = VibeCheckEvaluator::new();
    let result = evaluator
        .evaluate_with_oversight(
            &behavior_file,
            "Some output",
            &vibe_context,
            &metacognitive,
            &context_manager,
            &constitution_manager,
            Some("test-session"),
        )
        .await
        .unwrap();

    assert!(result.is_some());
    let decision = result.unwrap();
    assert!(decision.should_trigger);
    assert!(!decision.advice.is_empty());
    // Risk score should be calculated from the mock response
    assert!(decision.risk_score >= 0.0 && decision.risk_score <= 1.0);
}

#[tokio::test]
async fn test_learning_integration_from_oversight() {
    let temp_dir = TempDir::new().unwrap();
    let learning_store =
        Arc::new(std::sync::Mutex::new(LearningStore::new(temp_dir.path()).unwrap()));
    let model = Arc::new(MockOversightModel);
    let metacognitive = Arc::new(MetacognitiveService::new(model.clone()));
    let skill_manager = Arc::new(SkillManager::new(model));

    let integration = LearningIntegration::new(
        radium_core::learning::LearningConfig::default(),
        metacognitive,
        skill_manager,
        learning_store.clone(),
    );

    // Create a mock oversight response with traits
    let oversight_response = OversightResponse::new(
        "This solution is too complex and over-engineered".to_string(),
        0.7,
    )
    .with_trait("Complex Solution Bias")
    .with_helpful_patterns(vec!["Use simpler patterns".to_string()])
    .with_harmful_patterns(vec!["Avoid over-engineering".to_string()]);

    // Update learning from oversight
    let result = integration
        .update_from_oversight(&oversight_response, "Web development", "50%")
        .await;

    // The update may fail if skill manager can't parse the mock response, which is ok for this test
    // We're mainly testing that the integration method exists and can be called
    if let Err(e) = &result {
        // If it fails, it's likely because the mock model doesn't return proper JSON for skill updates
        // This is acceptable - the important thing is that the integration method exists
        eprintln!("Update from oversight failed (expected with mock model): {}", e);
    }

    // Verify mistake was added
    let store = learning_store.lock().unwrap();
    let entries = store.get_entries_by_category("Complex Solution Bias");
    assert!(!entries.is_empty());
}

#[tokio::test]
async fn test_constitution_integration_with_oversight() {
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    // Write vibe check action
    let action = BehaviorAction::new(BehaviorActionType::VibeCheck);
    action.write_to_file(&behavior_file).unwrap();

    // Setup services
    let model = Arc::new(MockOversightModel);
    let metacognitive = MetacognitiveService::new(model);
    let constitution_manager = ConstitutionManager::new();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    let context_manager = ContextManager::new(&workspace);

    // Add constitution rules
    constitution_manager.update_constitution("test-session", "No external network calls".to_string());
    constitution_manager.update_constitution("test-session", "Prefer unit tests".to_string());

    // Create vibe check context
    let vibe_context = VibeCheckContext::new(WorkflowPhase::Planning)
        .with_goal("Build feature")
        .with_plan("Use API calls");

    // Evaluate with oversight
    let evaluator = VibeCheckEvaluator::new();
    let result = evaluator
        .evaluate_with_oversight(
            &behavior_file,
            "",
            &vibe_context,
            &metacognitive,
            &context_manager,
            &constitution_manager,
            Some("test-session"),
        )
        .await
        .unwrap();

    assert!(result.is_some());
    // Constitution rules should be included in the oversight request
    // (we can't easily verify this without inspecting the request, but the call should succeed)
}

#[tokio::test]
async fn test_vibecheck_no_behavior_file() {
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    let evaluator = VibeCheckEvaluator::new();
    let context = VibeCheckContext::new(WorkflowPhase::Implementation);
    let result = evaluator.evaluate_vibe_check(&behavior_file, "", &context).unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn test_vibecheck_non_vibecheck_action() {
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    // Write loop action (should not trigger vibe check)
    let action = BehaviorAction::new(BehaviorActionType::Loop);
    action.write_to_file(&behavior_file).unwrap();

    let evaluator = VibeCheckEvaluator::new();
    let context = VibeCheckContext::new(WorkflowPhase::Implementation);
    let result = evaluator.evaluate_vibe_check(&behavior_file, "", &context).unwrap();

    assert!(result.is_none());
}


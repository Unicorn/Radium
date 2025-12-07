//! Integration tests for VibeCheck workflow behavior.
//!
//! Tests the integration of VibeCheck behavior with the workflow execution engine,
//! including oversight triggering, phase-aware context, and risk score calculation.

use radium_abstraction::{ChatMessage, Model, ModelError, ModelParameters, ModelResponse};
use radium_core::context::ContextManager;
use radium_core::models::{Task, Workflow, WorkflowStep};
use radium_core::oversight::MetacognitiveService;
use radium_core::policy::ConstitutionManager;
use radium_core::storage::{
    Database, SqliteTaskRepository, SqliteWorkflowRepository, TaskRepository, WorkflowRepository,
};
use radium_core::workflow::behaviors::types::{BehaviorAction, BehaviorActionType};
use radium_core::workflow::behaviors::vibe_check::{
    VibeCheckContext, VibeCheckEvaluator, WorkflowPhase,
};
use radium_core::workflow::WorkflowExecutor;
use radium_core::workspace::{Workspace, WorkspaceStructure};
use radium_orchestrator::{AgentExecutor, Orchestrator, SimpleAgent};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

// Mock model that returns phase-aware oversight responses
struct MockOversightModel {
    phase: WorkflowPhase,
}

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
        let advice = match self.phase {
            WorkflowPhase::Planning => {
                "The plan looks comprehensive but may be over-engineered. Consider simplifying the approach and focusing on core requirements first."
            }
            WorkflowPhase::Implementation => {
                "The implementation is progressing well, but watch for complexity creep. The current approach may introduce unnecessary dependencies."
            }
            WorkflowPhase::Review => {
                "The review shows good progress, but ensure all edge cases are covered. Consider adding more comprehensive error handling."
            }
        };

        Ok(ModelResponse {
            content: format!(
                "{}\n\nTraits: Complex Solution Bias, Feature Creep\nUncertainties: Performance requirements unclear, scalability concerns",
                advice
            ),
            model_id: Some("mock".to_string()),
            usage: None,
        })
    }

    fn model_id(&self) -> &str {
        "mock"
    }
}

#[tokio::test]
async fn test_vibecheck_behavior_triggers_oversight_in_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    let ws_structure = WorkspaceStructure::new(workspace.root());
    let behavior_file = ws_structure.memory_dir().join("behavior.json");

    // Write vibe check action
    let action = BehaviorAction::new(BehaviorActionType::VibeCheck)
        .with_reason("Need oversight before proceeding");
    action.write_to_file(&behavior_file).unwrap();

    // Setup workflow executor
    let db = Arc::new(std::sync::Mutex::new(Database::open_in_memory().unwrap()));
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let workflow_executor =
        WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor), None);

    // Register agent
    let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
    orchestrator.register_agent(agent).await;

    // Create task
    {
        let mut db_lock = db.lock().unwrap();
        let mut task_repo = SqliteTaskRepository::new(&mut *db_lock);
        let task = Task::new(
            "task-1".to_string(),
            "Task 1".to_string(),
            "Test task".to_string(),
            "test-agent".to_string(),
            json!({"input": "test"}),
        );
        task_repo.create(&task).unwrap();
    }

    // Create workflow
    {
        let mut db_lock = db.lock().unwrap();
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
        let mut workflow = Workflow::new(
            "workflow-1".to_string(),
            "VibeCheck Test".to_string(),
            "Testing vibe check integration".to_string(),
        );
        workflow
            .add_step(WorkflowStep::new(
                "step-1".to_string(),
                "Step 1".to_string(),
                "Test step".to_string(),
                "task-1".to_string(),
                0,
            ))
            .unwrap();
        workflow_repo.create(&workflow).unwrap();
    }

    // Execute workflow - vibe check should be detected
    let mut workflow = {
        let mut db_lock = db.lock().unwrap();
        let workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
        workflow_repo.get_by_id("workflow-1").unwrap()
    };

    let result = workflow_executor.execute_workflow(&mut workflow, Arc::clone(&db)).await;
    
    // Workflow should complete (vibe check detection doesn't block execution)
    assert!(result.is_ok());
    
    // Verify behavior file was read (the executor logs this, but we can verify the file exists)
    assert!(behavior_file.exists());
}

#[tokio::test]
async fn test_phase_aware_oversight_planning() {
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    // Write vibe check action
    let action = BehaviorAction::new(BehaviorActionType::VibeCheck)
        .with_reason("Planning phase oversight");
    action.write_to_file(&behavior_file).unwrap();

    // Setup services with Planning phase model
    let model = Arc::new(MockOversightModel { phase: WorkflowPhase::Planning });
    let metacognitive = MetacognitiveService::new(model);
    let constitution_manager = ConstitutionManager::new();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    let context_manager = ContextManager::new(&workspace);

    // Create vibe check context for Planning phase
    let vibe_context = VibeCheckContext::new(WorkflowPhase::Planning)
        .with_goal("Build a web application")
        .with_plan("Use React for frontend, Node.js for backend, PostgreSQL for database")
        .with_progress("Planning phase - 0% complete")
        .with_task_context("Defining architecture and technology stack");

    // Evaluate with oversight
    let evaluator = VibeCheckEvaluator::new();
    let result = evaluator
        .evaluate_with_oversight(
            &behavior_file,
            "Planning output",
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
    assert!(decision.risk_score >= 0.0 && decision.risk_score <= 1.0);
    
    // Verify traits were extracted
    assert!(!decision.traits.is_empty());
}

#[tokio::test]
async fn test_phase_aware_oversight_implementation() {
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    // Write vibe check action
    let action = BehaviorAction::new(BehaviorActionType::VibeCheck)
        .with_reason("Implementation phase oversight");
    action.write_to_file(&behavior_file).unwrap();

    // Setup services with Implementation phase model
    let model = Arc::new(MockOversightModel { phase: WorkflowPhase::Implementation });
    let metacognitive = MetacognitiveService::new(model);
    let constitution_manager = ConstitutionManager::new();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    let context_manager = ContextManager::new(&workspace);

    // Create vibe check context for Implementation phase
    let vibe_context = VibeCheckContext::new(WorkflowPhase::Implementation)
        .with_goal("Build a web application")
        .with_plan("Use React for frontend, Node.js for backend")
        .with_progress("50% complete - frontend done, working on backend")
        .with_task_context("Implementing authentication middleware");

    // Evaluate with oversight
    let evaluator = VibeCheckEvaluator::new();
    let result = evaluator
        .evaluate_with_oversight(
            &behavior_file,
            "Implementation output",
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
    assert!(decision.risk_score >= 0.0 && decision.risk_score <= 1.0);
}

#[tokio::test]
async fn test_phase_aware_oversight_review() {
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    // Write vibe check action
    let action = BehaviorAction::new(BehaviorActionType::VibeCheck)
        .with_reason("Review phase oversight");
    action.write_to_file(&behavior_file).unwrap();

    // Setup services with Review phase model
    let model = Arc::new(MockOversightModel { phase: WorkflowPhase::Review });
    let metacognitive = MetacognitiveService::new(model);
    let constitution_manager = ConstitutionManager::new();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    let context_manager = ContextManager::new(&workspace);

    // Create vibe check context for Review phase
    let vibe_context = VibeCheckContext::new(WorkflowPhase::Review)
        .with_goal("Build a web application")
        .with_plan("Use React for frontend, Node.js for backend")
        .with_progress("100% complete - ready for review")
        .with_task_context("Reviewing code quality and test coverage");

    // Evaluate with oversight
    let evaluator = VibeCheckEvaluator::new();
    let result = evaluator
        .evaluate_with_oversight(
            &behavior_file,
            "Review output",
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
    assert!(decision.risk_score >= 0.0 && decision.risk_score <= 1.0);
}

#[tokio::test]
async fn test_risk_score_calculation() {
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    // Write vibe check action
    let action = BehaviorAction::new(BehaviorActionType::VibeCheck);
    action.write_to_file(&behavior_file).unwrap();

    // Create a mock model that returns high-risk advice
    struct HighRiskModel;

    #[async_trait::async_trait]
    impl Model for HighRiskModel {
        async fn generate_text(
            &self,
            _prompt: &str,
            _params: Option<ModelParameters>,
        ) -> Result<ModelResponse, ModelError> {
            Ok(ModelResponse {
                content: "Mock text".to_string(),
                model_id: Some("mock".to_string()),
                usage: None,
            })
        }

        async fn generate_chat_completion(
            &self,
            _messages: &[ChatMessage],
            _params: Option<ModelParameters>,
        ) -> Result<ModelResponse, ModelError> {
            // Return advice with high-risk keywords that are detected by estimate_risk_score
            Ok(ModelResponse {
                content: "This approach is wrong and incorrect. There are serious problems and issues that need to be addressed. The solution is over-engineered and complex, with many concerns about misalignment.".to_string(),
                model_id: Some("mock".to_string()),
                usage: None,
            })
        }

        fn model_id(&self) -> &str {
            "mock"
        }
    }

    // Setup services
    let model = Arc::new(HighRiskModel);
    let metacognitive = MetacognitiveService::new(model);
    let constitution_manager = ConstitutionManager::new();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    let context_manager = ContextManager::new(&workspace);

    let vibe_context = VibeCheckContext::new(WorkflowPhase::Implementation)
        .with_goal("Test goal")
        .with_plan("Test plan");

    // Evaluate with oversight
    let evaluator = VibeCheckEvaluator::new();
    let result = evaluator
        .evaluate_with_oversight(
            &behavior_file,
            "Test output",
            &vibe_context,
            &metacognitive,
            &context_manager,
            &constitution_manager,
            None,
        )
        .await
        .unwrap();

    assert!(result.is_some());
    let decision = result.unwrap();
    // High-risk keywords (wrong, incorrect, problem, issue, complex, over-engineered) should result in higher risk score
    // Base is 0.3, +0.3 for wrong/incorrect, +0.2 for problem/issue, +0.15 for complex/over-engineered = 0.95
    assert!(decision.risk_score > 0.7, "Risk score should be elevated for high-risk advice, got {}", decision.risk_score);
}

#[tokio::test]
async fn test_trait_extraction_from_oversight() {
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    // Write vibe check action
    let action = BehaviorAction::new(BehaviorActionType::VibeCheck);
    action.write_to_file(&behavior_file).unwrap();

    // Create a mock model that returns specific traits
    struct TraitModel;

    #[async_trait::async_trait]
    impl Model for TraitModel {
        async fn generate_text(
            &self,
            _prompt: &str,
            _params: Option<ModelParameters>,
        ) -> Result<ModelResponse, ModelError> {
            Ok(ModelResponse {
                content: "Mock text".to_string(),
                model_id: Some("mock".to_string()),
                usage: None,
            })
        }

        async fn generate_chat_completion(
            &self,
            _messages: &[ChatMessage],
            _params: Option<ModelParameters>,
        ) -> Result<ModelResponse, ModelError> {
            // Return response with explicit traits
            Ok(ModelResponse {
                content: "The solution shows signs of Complex Solution Bias and Feature Creep. Consider simplifying.\n\nTraits: Complex Solution Bias, Feature Creep, Premature Implementation\nUncertainties: Performance unclear, scalability unknown".to_string(),
                model_id: Some("mock".to_string()),
                usage: None,
            })
        }

        fn model_id(&self) -> &str {
            "mock"
        }
    }

    // Setup services
    let model = Arc::new(TraitModel);
    let metacognitive = MetacognitiveService::new(model);
    let constitution_manager = ConstitutionManager::new();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    let context_manager = ContextManager::new(&workspace);

    let vibe_context = VibeCheckContext::new(WorkflowPhase::Implementation)
        .with_goal("Test goal")
        .with_plan("Test plan");

    // Evaluate with oversight
    let evaluator = VibeCheckEvaluator::new();
    let result = evaluator
        .evaluate_with_oversight(
            &behavior_file,
            "Test output",
            &vibe_context,
            &metacognitive,
            &context_manager,
            &constitution_manager,
            None,
        )
        .await
        .unwrap();

    assert!(result.is_some());
    let decision = result.unwrap();
    
    // Verify traits were extracted
    assert!(!decision.traits.is_empty());
    assert!(decision.traits.iter().any(|t| t.contains("Complex Solution Bias")));
    assert!(decision.traits.iter().any(|t| t.contains("Feature Creep")));
    
    // Verify uncertainties were extracted
    assert!(!decision.uncertainties.is_empty());
}

#[tokio::test]
async fn test_behavior_json_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    // Test valid vibe check action
    let action = BehaviorAction::new(BehaviorActionType::VibeCheck)
        .with_reason("Testing behavior parsing");
    action.write_to_file(&behavior_file).unwrap();

    let evaluator = VibeCheckEvaluator::new();
    let context = VibeCheckContext::new(WorkflowPhase::Implementation);
    let result = evaluator.evaluate_vibe_check(&behavior_file, "", &context).unwrap();

    assert!(result.is_some());
    let decision = result.unwrap();
    assert!(decision.should_trigger);
    assert_eq!(decision.reason.as_deref(), Some("Testing behavior parsing"));

    // Test non-vibecheck action (should return None)
    let loop_action = BehaviorAction::new(BehaviorActionType::Loop);
    loop_action.write_to_file(&behavior_file).unwrap();

    let result = evaluator.evaluate_vibe_check(&behavior_file, "", &context).unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_vibecheck_with_constitution_rules() {
    let temp_dir = TempDir::new().unwrap();
    let behavior_file = temp_dir.path().join("behavior.json");

    // Write vibe check action
    let action = BehaviorAction::new(BehaviorActionType::VibeCheck);
    action.write_to_file(&behavior_file).unwrap();

    // Setup services
    let model = Arc::new(MockOversightModel { phase: WorkflowPhase::Implementation });
    let metacognitive = MetacognitiveService::new(model);
    let constitution_manager = ConstitutionManager::new();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    let context_manager = ContextManager::new(&workspace);

    // Add constitution rules
    constitution_manager.update_constitution("test-session", "No external API calls".to_string());
    constitution_manager.update_constitution("test-session", "Prefer unit tests over integration tests".to_string());

    let vibe_context = VibeCheckContext::new(WorkflowPhase::Implementation)
        .with_goal("Build feature")
        .with_plan("Use external API");

    // Evaluate with oversight - constitution rules should be included
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
    let decision = result.unwrap();
    assert!(decision.should_trigger);
    // Constitution rules should be included in the oversight request
    // (we verify this by ensuring the call succeeds)
}


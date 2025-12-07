//! Integration tests for autonomous planning system.
//!
//! Tests the complete autonomous planning pipeline including plan generation,
//! validation with retry logic, workflow generation, and error handling.

use radium_abstraction::{ChatMessage, Model, ModelError, ModelParameters, ModelResponse};
use radium_core::agents::registry::AgentRegistry;
#[cfg(feature = "workflow")]
use radium_core::planning::{
    AutonomousPlanner, DependencyGraph, ParsedIteration, ParsedPlan, ParsedTask,
    PlanValidator, PlanningError, ValidationReport, WorkflowGenerator,
};
#[cfg(not(feature = "workflow"))]
use radium_core::planning::{DependencyGraph, ParsedIteration, ParsedPlan, ParsedTask};
use std::sync::Arc;

// Mock model that can return different responses based on call count
struct MockPlanModel {
    responses: Vec<String>,
    call_count: Arc<std::sync::Mutex<usize>>,
}

impl MockPlanModel {
    fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            call_count: Arc::new(std::sync::Mutex::new(0)),
        }
    }
}

#[async_trait::async_trait]
impl Model for MockPlanModel {
    async fn generate_text(
        &self,
        _prompt: &str,
        _params: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        let mut count = self.call_count.lock().unwrap();
        let index = (*count).min(self.responses.len() - 1);
        *count += 1;
        let response = self.responses[index].clone();

        Ok(ModelResponse {
            content: response,
            model_id: Some("mock".to_string()),
            usage: None,
        })
    }

    async fn generate_chat_completion(
        &self,
        _messages: &[ChatMessage],
        _params: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        self.generate_text("", _params).await
    }

    fn model_id(&self) -> &str {
        "mock"
    }
}

// Helper to create a valid plan structure
fn create_valid_plan() -> ParsedPlan {
    ParsedPlan {
        project_name: "Test Project".to_string(),
        description: Some("A test project".to_string()),
        tech_stack: vec!["Rust".to_string(), "TypeScript".to_string()],
        iterations: vec![
            ParsedIteration {
                number: 1,
                name: "Iteration 1".to_string(),
                description: Some("First iteration".to_string()),
                goal: Some("Setup project".to_string()),
                tasks: vec![
                    ParsedTask {
                        number: 1,
                        title: "Task 1".to_string(),
                        description: Some("Setup workspace".to_string()),
                        agent_id: Some("code-agent".to_string()),
                        dependencies: vec![],
                        acceptance_criteria: vec!["Workspace created".to_string()],
                    },
                    ParsedTask {
                        number: 2,
                        title: "Task 2".to_string(),
                        description: Some("Create models".to_string()),
                        agent_id: Some("code-agent".to_string()),
                        dependencies: vec!["I1.T1".to_string()],
                        acceptance_criteria: vec!["Models created".to_string()],
                    },
                ],
            },
        ],
    }
}

// Helper to create a plan with circular dependencies
fn create_cyclic_plan() -> ParsedPlan {
    ParsedPlan {
        project_name: "Cyclic Project".to_string(),
        description: None,
        tech_stack: vec![],
        iterations: vec![ParsedIteration {
            number: 1,
            name: "Iteration 1".to_string(),
            description: None,
            goal: None,
            tasks: vec![
                ParsedTask {
                    number: 1,
                    title: "Task 1".to_string(),
                    description: None,
                    agent_id: Some("code-agent".to_string()),
                    dependencies: vec!["I1.T3".to_string()], // Depends on T3
                    acceptance_criteria: vec![],
                },
                ParsedTask {
                    number: 2,
                    title: "Task 2".to_string(),
                    description: None,
                    agent_id: Some("code-agent".to_string()),
                    dependencies: vec!["I1.T1".to_string()], // Depends on T1
                    acceptance_criteria: vec![],
                },
                ParsedTask {
                    number: 3,
                    title: "Task 3".to_string(),
                    description: None,
                    agent_id: Some("code-agent".to_string()),
                    dependencies: vec!["I1.T2".to_string()], // Depends on T2 -> cycle!
                    acceptance_criteria: vec![],
                },
            ],
        }],
    }
}

// Helper to create a plan with missing dependency
fn create_plan_with_missing_dependency() -> ParsedPlan {
    ParsedPlan {
        project_name: "Missing Dep Project".to_string(),
        description: None,
        tech_stack: vec![],
        iterations: vec![ParsedIteration {
            number: 1,
            name: "Iteration 1".to_string(),
            description: None,
            goal: None,
            tasks: vec![ParsedTask {
                number: 1,
                title: "Task 1".to_string(),
                description: None,
                agent_id: Some("code-agent".to_string()),
                dependencies: vec!["I5.T1".to_string()], // Non-existent dependency
                acceptance_criteria: vec![],
            }],
        }],
    }
}

// Helper to create a plan with unknown agent
fn create_plan_with_unknown_agent() -> ParsedPlan {
    ParsedPlan {
        project_name: "Unknown Agent Project".to_string(),
        description: None,
        tech_stack: vec![],
        iterations: vec![ParsedIteration {
            number: 1,
            name: "Iteration 1".to_string(),
            description: None,
            goal: None,
            tasks: vec![ParsedTask {
                number: 1,
                title: "Task 1".to_string(),
                description: None,
                agent_id: Some("unknown-agent".to_string()), // Unknown agent
                dependencies: vec![],
                acceptance_criteria: vec![],
            }],
        }],
    }
}

// Helper to create a valid plan response string
fn create_valid_plan_response() -> String {
    r#"# Test Project

Brief project description.

## Tech Stack
- Rust
- TypeScript

## Iteration 1: Setup

Goal: Setup project structure

1. **Task 1** - Setup workspace
   - Agent: code-agent
   - Dependencies: 
   - Acceptance Criteria:
     - Workspace created

2. **Task 2** - Create models
   - Agent: code-agent
   - Dependencies: I1.T1
   - Acceptance Criteria:
     - Models created
"#
    .to_string()
}

#[cfg(feature = "workflow")]
#[tokio::test]
async fn test_plan_validator_valid_plan() {
    let registry = Arc::new(AgentRegistry::new());
    let validator = PlanValidator::new(registry);

    let plan = create_valid_plan();
    let report = validator.validate_plan(&plan);

    assert!(report.is_valid);
    assert!(report.errors.is_empty());
}

#[cfg(feature = "workflow")]
#[tokio::test]
async fn test_plan_validator_cyclic_dependencies() {
    let registry = Arc::new(AgentRegistry::new());
    let validator = PlanValidator::new(registry);

    let plan = create_cyclic_plan();
    let report = validator.validate_plan(&plan);

    assert!(!report.is_valid);
    assert!(report.errors.iter().any(|e| e.contains("circular dependency")));
}

#[cfg(feature = "workflow")]
#[tokio::test]
async fn test_plan_validator_missing_dependency() {
    let registry = Arc::new(AgentRegistry::new());
    let validator = PlanValidator::new(registry);

    let plan = create_plan_with_missing_dependency();
    let report = validator.validate_plan(&plan);

    assert!(!report.is_valid);
    assert!(report
        .errors
        .iter()
        .any(|e| e.contains("non-existent dependency")));
}

#[cfg(feature = "workflow")]
#[tokio::test]
async fn test_plan_validator_unknown_agent() {
    let registry = Arc::new(AgentRegistry::new());
    let validator = PlanValidator::new(registry);

    let plan = create_plan_with_unknown_agent();
    let report = validator.validate_plan(&plan);

    // Unknown agents are warnings, not errors
    assert!(report.is_valid); // Still valid, just a warning
    assert!(report
        .warnings
        .iter()
        .any(|w| w.contains("unknown agent")));
}

#[cfg(feature = "workflow")]
#[tokio::test]
async fn test_workflow_generator_valid_plan() {
    let generator = WorkflowGenerator::new();
    let plan = create_valid_plan();

    // Convert plan to manifest for DAG
    let manifest = {
        use radium_core::models::{Iteration, PlanManifest, PlanTask};
        use radium_core::workspace::RequirementId;
        use std::str::FromStr;

        let req_id = RequirementId::from_str("TEST").unwrap();
        let mut manifest = PlanManifest::new(req_id, plan.project_name.clone());

        for parsed_iter in &plan.iterations {
            let mut iteration = Iteration::new(parsed_iter.number, parsed_iter.name.clone());
            for parsed_task in &parsed_iter.tasks {
                let mut task = PlanTask::new(
                    &format!("I{}", parsed_iter.number),
                    parsed_task.number,
                    parsed_task.title.clone(),
                );
                task.dependencies = parsed_task.dependencies.clone();
                task.agent_id = parsed_task.agent_id.clone();
                iteration.add_task(task);
            }
            manifest.add_iteration(iteration);
        }

        manifest
    };

    let dag = DependencyGraph::from_manifest(&manifest).unwrap();
    let workflow = generator.generate_workflow(&plan, &dag).unwrap();

    // Workflow should have steps in dependency order
    assert_eq!(workflow.steps.len(), 2);
    // Steps should be in topological order (T1 before T2)
}

#[cfg(feature = "workflow")]
#[tokio::test]
async fn test_autonomous_planner_successful_generation() {
    let registry = Arc::new(AgentRegistry::new());
    let planner = AutonomousPlanner::new(registry);

    let valid_response = create_valid_plan_response();
    let model = Arc::new(MockPlanModel::new(vec![valid_response]));

    let goal = "Build a test project";
    let result = planner.plan_from_goal(goal, model).await;

    assert!(result.is_ok());
    let autonomous_plan = result.unwrap();

    assert_eq!(autonomous_plan.plan.project_name, "Test Project");
    assert!(!autonomous_plan.plan.iterations.is_empty());
    assert!(!autonomous_plan.workflow.steps.is_empty());
}

#[cfg(feature = "workflow")]
#[tokio::test]
async fn test_autonomous_planner_validation_retry() {
    let registry = Arc::new(AgentRegistry::new());
    let planner = AutonomousPlanner::new(registry);

    // First response has cycle, second is valid
    let invalid_response = r#"# Test Project

## Iteration 1: Setup

1. **Task 1** - Setup
   - Dependencies: I1.T2
2. **Task 2** - Build
   - Dependencies: I1.T1
"#
    .to_string();

    let valid_response = create_valid_plan_response();

    let model = Arc::new(MockPlanModel::new(vec![
        invalid_response,
        valid_response,
    ]));

    let goal = "Build a test project";
    let result = planner.plan_from_goal(goal, model).await;

    // Should succeed after retry
    assert!(result.is_ok());
}

#[cfg(feature = "workflow")]
#[tokio::test]
async fn test_autonomous_planner_validation_failure_after_max_retries() {
    let registry = Arc::new(AgentRegistry::new());
    let planner = AutonomousPlanner::new(registry);

    // Always return invalid plan with cycle
    let invalid_response = r#"# Test Project

## Iteration 1: Setup

1. **Task 1** - Setup
   - Dependencies: I1.T2
2. **Task 2** - Build
   - Dependencies: I1.T1
"#
    .to_string();

    let model = Arc::new(MockPlanModel::new(vec![
        invalid_response.clone(),
        invalid_response.clone(),
        invalid_response,
    ]));

    let goal = "Build a test project";
    let result = planner.plan_from_goal(goal, model).await;

    // Should fail after max retries
    assert!(result.is_err());
    match result.unwrap_err() {
        PlanningError::ValidationFailed(_) => {}
        e => panic!("Expected ValidationFailed, got {:?}", e),
    }
}

#[cfg(feature = "workflow")]
#[tokio::test]
async fn test_autonomous_planner_cycle_detection_integration() {
    let registry = Arc::new(AgentRegistry::new());
    let planner = AutonomousPlanner::new(registry);

    // Response with cycle
    let cyclic_response = r#"# Test Project

## Iteration 1: Setup

1. **Task 1** - Setup
   - Dependencies: I1.T3
2. **Task 2** - Build
   - Dependencies: I1.T1
3. **Task 3** - Test
   - Dependencies: I1.T2
"#
    .to_string();

    let model = Arc::new(MockPlanModel::new(vec![
        cyclic_response.clone(),
        cyclic_response.clone(),
        cyclic_response,
    ]));

    let goal = "Build a test project";
    let result = planner.plan_from_goal(goal, model).await;

    // Should fail with cycle detection
    assert!(result.is_err());
    match result.unwrap_err() {
        PlanningError::Dag(_) => {} // DAG error indicates cycle
        e => panic!("Expected DAG error (cycle), got {:?}", e),
    }
}

#[cfg(feature = "workflow")]
#[tokio::test]
async fn test_plan_validator_multi_stage_validation() {
    let registry = Arc::new(AgentRegistry::new());
    let validator = PlanValidator::new(registry);

    // Create plan with multiple validation issues
    let plan = ParsedPlan {
        project_name: "Multi Issue Project".to_string(),
        description: None,
        tech_stack: vec![],
        iterations: vec![
            ParsedIteration {
                number: 1,
                name: "Iteration 1".to_string(),
                description: None,
                goal: None,
                tasks: vec![
                    ParsedTask {
                        number: 1,
                        title: "Task 1".to_string(),
                        description: None,
                        agent_id: Some("unknown-agent".to_string()), // Unknown agent (warning)
                        dependencies: vec!["I5.T1".to_string()], // Missing dependency (error)
                        acceptance_criteria: vec![],
                    },
                    ParsedTask {
                        number: 2,
                        title: "Task 2".to_string(),
                        description: None,
                        agent_id: Some("code-agent".to_string()),
                        dependencies: vec!["I1.T1".to_string()], // Valid dependency
                        acceptance_criteria: vec![],
                    },
                ],
            },
        ],
    };

    let report = validator.validate_plan(&plan);

    // Should have both errors and warnings
    assert!(!report.is_valid);
    assert!(report.errors.iter().any(|e| e.contains("non-existent dependency")));
    assert!(report.warnings.iter().any(|w| w.contains("unknown agent")));
}

#[cfg(feature = "workflow")]
#[tokio::test]
async fn test_plan_validator_agent_registry_integration() {
    let registry = Arc::new(AgentRegistry::new());
    let validator = PlanValidator::new(registry.clone());

    // Create plan with known and unknown agents
    let plan = ParsedPlan {
        project_name: "Agent Test Project".to_string(),
        description: None,
        tech_stack: vec![],
        iterations: vec![ParsedIteration {
            number: 1,
            name: "Iteration 1".to_string(),
            description: None,
            goal: None,
            tasks: vec![
                ParsedTask {
                    number: 1,
                    title: "Task 1".to_string(),
                    description: None,
                    agent_id: Some("code-agent".to_string()), // Known agent
                    dependencies: vec![],
                    acceptance_criteria: vec![],
                },
                ParsedTask {
                    number: 2,
                    title: "Task 2".to_string(),
                    description: None,
                    agent_id: Some("nonexistent-agent".to_string()), // Unknown agent
                    dependencies: vec![],
                    acceptance_criteria: vec![],
                },
                ParsedTask {
                    number: 3,
                    title: "Task 3".to_string(),
                    description: None,
                    agent_id: Some("auto".to_string()), // Auto agent (should not warn)
                    dependencies: vec![],
                    acceptance_criteria: vec![],
                },
            ],
        }],
    };

    let report = validator.validate_plan(&plan);

    // Should be valid (unknown agents are warnings, not errors)
    assert!(report.is_valid);
    // Should have warning for unknown agent
    assert!(report.warnings.iter().any(|w| w.contains("nonexistent-agent")));
    // Should not warn about "auto" agent
    assert!(!report.warnings.iter().any(|w| w.contains("auto")));
}

#[cfg(feature = "workflow")]
#[tokio::test]
async fn test_autonomous_planner_validation_retry_with_feedback() {
    let registry = Arc::new(AgentRegistry::new());
    let planner = AutonomousPlanner::new(registry);

    // First response has missing dependency, second has valid plan
    let invalid_response = r#"# Test Project

## Iteration 1: Setup

1. **Task 1** - Setup
   - Dependencies: I5.T1
2. **Task 2** - Build
   - Dependencies: I1.T1
"#
    .to_string();

    let valid_response = create_valid_plan_response();

    let model = Arc::new(MockPlanModel::new(vec![
        invalid_response,
        valid_response,
    ]));

    let goal = "Build a test project";
    let result = planner.plan_from_goal(goal, model).await;

    // Should succeed after retry with feedback
    assert!(result.is_ok());
    let autonomous_plan = result.unwrap();
    assert_eq!(autonomous_plan.plan.project_name, "Test Project");
}

#[cfg(feature = "workflow")]
#[tokio::test]
async fn test_autonomous_planner_validation_retry_max_attempts() {
    let registry = Arc::new(AgentRegistry::new());
    let planner = AutonomousPlanner::new(registry);

    // Always return invalid plan with missing dependency
    let invalid_response = r#"# Test Project

## Iteration 1: Setup

1. **Task 1** - Setup
   - Dependencies: I5.T1
2. **Task 2** - Build
   - Dependencies: I1.T1
"#
    .to_string();

    // Should try 3 times total (initial + 2 retries)
    let model = Arc::new(MockPlanModel::new(vec![
        invalid_response.clone(),
        invalid_response.clone(),
        invalid_response,
    ]));

    let goal = "Build a test project";
    let result = planner.plan_from_goal(goal, model).await;

    // Should fail after max retries
    assert!(result.is_err());
    match result.unwrap_err() {
        PlanningError::ValidationFailed(_) => {}
        e => panic!("Expected ValidationFailed, got {:?}", e),
    }
}


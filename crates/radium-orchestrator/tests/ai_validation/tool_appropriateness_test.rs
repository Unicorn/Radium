//! Tool appropriateness tests.
//!
//! Tests whether the orchestrator selects and uses tools appropriately.

use crate::ai_validation::{
    evaluator::{AiEvaluator, EvaluationAspect, EvaluationCriteria},
    scenarios::{tool_avoidance_when_unnecessary, tool_selection_simple, tool_selection_multi_step},
    skip_if_no_evaluator_key, skip_if_no_orchestrator_key,
};
use radium_orchestrator::orchestration::{
    context::OrchestrationContext,
    engine::OrchestrationEngine,
    file_tools::create_file_operation_tools,
    providers::GeminiOrchestrator,
};
use std::{env, path::PathBuf, sync::Arc};

/// Create a real orchestrator for testing.
fn create_test_orchestrator() -> Arc<dyn radium_orchestrator::orchestration::OrchestrationProvider> {
    let api_key = env::var("GEMINI_API_KEY")
        .or_else(|_| env::var("ANTHROPIC_API_KEY"))
        .expect("Either GEMINI_API_KEY or ANTHROPIC_API_KEY must be set");

    // For now, use Gemini. Could be extended to support Claude based on env var.
    Arc::new(
        GeminiOrchestrator::new("gemini-2.0-flash-exp", api_key)
            .with_temperature(0.7)
    )
}

/// Create workspace root provider for file tools.
struct TestWorkspaceRoot {
    root: PathBuf,
}

impl radium_orchestrator::orchestration::file_tools::WorkspaceRootProvider for TestWorkspaceRoot {
    fn workspace_root(&self) -> Option<PathBuf> {
        Some(self.root.clone())
    }
}

#[tokio::test]
#[ignore = "AI validation test - requires API keys and costs money"]
async fn test_tool_selection_for_simple_task() {
    if skip_if_no_orchestrator_key() || skip_if_no_evaluator_key() {
        return;
    }

    println!("\n🧪 Testing: Simple Tool Selection");

    let scenario = tool_selection_simple();
    let evaluator = AiEvaluator::from_env()
        .expect("Failed to create evaluator");

    // Create orchestrator with real file tools
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: env::current_dir().unwrap(),
    });
    let tools = create_file_operation_tools(workspace_root);
    let provider = create_test_orchestrator();
    let engine = OrchestrationEngine::with_defaults(provider, tools);

    // Execute the scenario
    let mut context = OrchestrationContext::new("test-session");
    let result = engine.execute(&scenario.user_input, &mut context).await;

    let response = match result {
        Ok(r) => {
            println!("✓ Orchestrator response received");
            r.response
        }
        Err(e) => {
            println!("✗ Orchestration failed: {}", e);
            panic!("Orchestration failed: {}", e);
        }
    };

    // Evaluate with AI
    let criteria = EvaluationCriteria {
        aspect: EvaluationAspect::ToolAppropriateness,
        required_elements: vec![
            "Should call read_file tool".to_string(),
            "Should not ask user for clarification when file path is explicit".to_string(),
            "Should not use unnecessary search tools".to_string(),
        ],
        min_score: 70,
    };

    let evaluation = evaluator.evaluate(
        &scenario.description,
        &scenario.user_input,
        &scenario.expected_behavior,
        &response,
        &criteria,
    ).await.expect("Evaluation failed");

    // Print results
    println!("\n📊 AI Evaluation Results:");
    println!("   Score: {}/100", evaluation.score.unwrap_or(0));
    println!("   Passed: {}", if evaluation.passed { "✓" } else { "✗" });
    println!("   Feedback: {}", evaluation.feedback);
    println!("   Reasoning:\n{}", evaluation.reasoning);

    assert!(evaluation.passed, "AI evaluation failed: {}", evaluation.feedback);
    assert!(
        evaluation.score.unwrap_or(0) >= criteria.min_score,
        "Score {} below threshold {}",
        evaluation.score.unwrap_or(0),
        criteria.min_score
    );
}

#[tokio::test]
#[ignore = "AI validation test - requires API keys and costs money"]
async fn test_tool_selection_for_multi_step_task() {
    if skip_if_no_orchestrator_key() || skip_if_no_evaluator_key() {
        return;
    }

    println!("\n🧪 Testing: Multi-Step Tool Usage");

    let scenario = tool_selection_multi_step();
    let evaluator = AiEvaluator::from_env()
        .expect("Failed to create evaluator");

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: env::current_dir().unwrap(),
    });
    let tools = create_file_operation_tools(workspace_root);
    let provider = create_test_orchestrator();
    let engine = OrchestrationEngine::with_defaults(provider, tools);

    let mut context = OrchestrationContext::new("test-session");
    let result = engine.execute(&scenario.user_input, &mut context).await;

    let response = match result {
        Ok(r) => {
            println!("✓ Orchestrator response received");
            r.response
        }
        Err(e) => {
            println!("✗ Orchestration failed: {}", e);
            panic!("Orchestration failed: {}", e);
        }
    };

    let criteria = EvaluationCriteria {
        aspect: EvaluationAspect::ToolAppropriateness,
        required_elements: vec![
            "Should use glob_file_search with appropriate pattern".to_string(),
            "Should provide a count or summary of results".to_string(),
            "Should not read individual files unnecessarily".to_string(),
        ],
        min_score: 70,
    };

    let evaluation = evaluator.evaluate(
        &scenario.description,
        &scenario.user_input,
        &scenario.expected_behavior,
        &response,
        &criteria,
    ).await.expect("Evaluation failed");

    println!("\n📊 AI Evaluation Results:");
    println!("   Score: {}/100", evaluation.score.unwrap_or(0));
    println!("   Passed: {}", if evaluation.passed { "✓" } else { "✗" });
    println!("   Feedback: {}", evaluation.feedback);
    println!("   Reasoning:\n{}", evaluation.reasoning);

    assert!(evaluation.passed, "AI evaluation failed: {}", evaluation.feedback);
    assert!(evaluation.score.unwrap_or(0) >= criteria.min_score);
}

#[tokio::test]
#[ignore = "AI validation test - requires API keys and costs money"]
async fn test_avoids_tools_when_unnecessary() {
    if skip_if_no_orchestrator_key() || skip_if_no_evaluator_key() {
        return;
    }

    println!("\n🧪 Testing: Avoid Unnecessary Tool Usage");

    let scenario = tool_avoidance_when_unnecessary();
    let evaluator = AiEvaluator::from_env()
        .expect("Failed to create evaluator");

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: env::current_dir().unwrap(),
    });
    let tools = create_file_operation_tools(workspace_root);
    let provider = create_test_orchestrator();
    let engine = OrchestrationEngine::with_defaults(provider, tools);

    let mut context = OrchestrationContext::new("test-session");
    let result = engine.execute(&scenario.user_input, &mut context).await;

    let response = match result {
        Ok(r) => {
            println!("✓ Orchestrator response received");
            r.response
        }
        Err(e) => {
            println!("✗ Orchestration failed: {}", e);
            panic!("Orchestration failed: {}", e);
        }
    };

    let criteria = EvaluationCriteria {
        aspect: EvaluationAspect::ToolAppropriateness,
        required_elements: vec![
            "Should answer from knowledge without tools".to_string(),
            "Should provide accurate programming information".to_string(),
            "Should not use file search or read tools".to_string(),
        ],
        min_score: 70,
    };

    let evaluation = evaluator.evaluate(
        &scenario.description,
        &scenario.user_input,
        &scenario.expected_behavior,
        &response,
        &criteria,
    ).await.expect("Evaluation failed");

    println!("\n📊 AI Evaluation Results:");
    println!("   Score: {}/100", evaluation.score.unwrap_or(0));
    println!("   Passed: {}", if evaluation.passed { "✓" } else { "✗" });
    println!("   Feedback: {}", evaluation.feedback);
    println!("   Reasoning:\n{}", evaluation.reasoning);

    assert!(evaluation.passed, "AI evaluation failed: {}", evaluation.feedback);
    assert!(evaluation.score.unwrap_or(0) >= criteria.min_score);
}

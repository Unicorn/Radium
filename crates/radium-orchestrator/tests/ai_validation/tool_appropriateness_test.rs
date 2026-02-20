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
    // Using gemini-2.0-flash (stable and compatible with orchestrator)
    Arc::new(
        GeminiOrchestrator::new("gemini-2.0-flash", api_key)
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
    // Use the RAD project root (go up from crates/radium-orchestrator/tests)
    let mut workspace_root_path = env::current_dir().unwrap();
    // Navigate to project root (typically /Users/clay/Development/RAD)
    while workspace_root_path.file_name().map(|n| n.to_str()) != Some(Some("RAD")) {
        if !workspace_root_path.pop() {
            // Fallback to current dir if we can't find RAD
            workspace_root_path = env::current_dir().unwrap();
            break;
        }
    }
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: workspace_root_path,
    });
    let tools = create_file_operation_tools(workspace_root);
    let provider = create_test_orchestrator();
    let engine = OrchestrationEngine::with_defaults(provider, tools);

    // Execute the scenario
    let mut context = OrchestrationContext::new("test-session");
    let result = engine.execute(&scenario.user_input, &mut context).await;

    let (response, full_trace) = match result {
        Ok(r) => {
            println!("✓ Orchestrator response received");

            // Build a complete trace showing the full conversation
            let mut trace = String::new();
            trace.push_str("=== Full Orchestration Trace ===\n\n");

            trace.push_str("User Request: ");
            trace.push_str(&scenario.user_input);
            trace.push_str("\n\n");

            trace.push_str("Conversation History:\n");
            for (i, msg) in context.conversation_history.iter().enumerate() {
                let content_preview = if msg.content.len() > 300 {
                    format!("{}...(truncated)", &msg.content[..300])
                } else {
                    msg.content.clone()
                };
                trace.push_str(&format!("{}. {}: {}\n\n", i + 1, msg.role, content_preview));
            }

            trace.push_str("Final Response: ");
            trace.push_str(&r.response);
            trace.push_str("\n\nNote: The conversation history above shows all interactions, including any tool calls made by the orchestrator.");

            (r.response, trace)
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
        &full_trace,  // Use full trace instead of just final response
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

    // Use the RAD project root
    let mut workspace_root_path = env::current_dir().unwrap();
    while workspace_root_path.file_name().map(|n| n.to_str()) != Some(Some("RAD")) {
        if !workspace_root_path.pop() {
            workspace_root_path = env::current_dir().unwrap();
            break;
        }
    }
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: workspace_root_path,
    });
    let tools = create_file_operation_tools(workspace_root);
    let provider = create_test_orchestrator();
    let engine = OrchestrationEngine::with_defaults(provider, tools);

    let mut context = OrchestrationContext::new("test-session");
    let result = engine.execute(&scenario.user_input, &mut context).await;

    let (response, full_trace) = match result {
        Ok(r) => {
            println!("✓ Orchestrator response received");

            // Build a complete trace showing the full conversation
            let mut trace = String::new();
            trace.push_str("=== Full Orchestration Trace ===\n\n");

            trace.push_str("User Request: ");
            trace.push_str(&scenario.user_input);
            trace.push_str("\n\n");

            trace.push_str("Conversation History:\n");
            for (i, msg) in context.conversation_history.iter().enumerate() {
                let content_preview = if msg.content.len() > 300 {
                    format!("{}...(truncated)", &msg.content[..300])
                } else {
                    msg.content.clone()
                };
                trace.push_str(&format!("{}. {}: {}\n\n", i + 1, msg.role, content_preview));
            }

            trace.push_str("Final Response: ");
            trace.push_str(&r.response);
            trace.push_str("\n\nNote: The conversation history above shows all interactions, including any tool calls made by the orchestrator.");

            (r.response, trace)
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
        &full_trace,  // Use full trace instead of just final response
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

    // Use the RAD project root
    let mut workspace_root_path = env::current_dir().unwrap();
    while workspace_root_path.file_name().map(|n| n.to_str()) != Some(Some("RAD")) {
        if !workspace_root_path.pop() {
            workspace_root_path = env::current_dir().unwrap();
            break;
        }
    }
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: workspace_root_path,
    });
    let tools = create_file_operation_tools(workspace_root);
    let provider = create_test_orchestrator();
    let engine = OrchestrationEngine::with_defaults(provider, tools);

    let mut context = OrchestrationContext::new("test-session");
    let result = engine.execute(&scenario.user_input, &mut context).await;

    let (response, full_trace) = match result {
        Ok(r) => {
            println!("✓ Orchestrator response received");

            // Build a complete trace showing the full conversation
            let mut trace = String::new();
            trace.push_str("=== Full Orchestration Trace ===\n\n");

            trace.push_str("User Request: ");
            trace.push_str(&scenario.user_input);
            trace.push_str("\n\n");

            trace.push_str("Conversation History:\n");
            for (i, msg) in context.conversation_history.iter().enumerate() {
                let content_preview = if msg.content.len() > 300 {
                    format!("{}...(truncated)", &msg.content[..300])
                } else {
                    msg.content.clone()
                };
                trace.push_str(&format!("{}. {}: {}\n\n", i + 1, msg.role, content_preview));
            }

            trace.push_str("Final Response: ");
            trace.push_str(&r.response);
            trace.push_str("\n\nNote: The conversation history above shows all interactions, including any tool calls made by the orchestrator.");

            (r.response, trace)
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
        &full_trace,  // Use full trace instead of just final response
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

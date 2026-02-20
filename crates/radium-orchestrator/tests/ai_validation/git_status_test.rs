//! Git status reporting accuracy tests.
//!
//! Tests whether the orchestrator properly checks git status and reports accurately.

use crate::ai_validation::{
    evaluator::{AiEvaluator, EvaluationAspect, EvaluationCriteria},
    scenarios::{git_status_basic, git_status_clean_workspace, git_status_unstaged_changes},
    skip_if_no_evaluator_key, skip_if_no_orchestrator_key,
};
use radium_orchestrator::orchestration::{
    context::OrchestrationContext,
    engine::OrchestrationEngine,
    file_tools::create_file_operation_tools,
    git_extended_tools::create_git_extended_tools,
    providers::GeminiOrchestrator,
};
use std::{env, path::PathBuf, sync::Arc};

/// Create a real orchestrator for testing.
fn create_test_orchestrator() -> Arc<dyn radium_orchestrator::orchestration::OrchestrationProvider> {
    let api_key = env::var("GEMINI_API_KEY")
        .or_else(|_| env::var("ANTHROPIC_API_KEY"))
        .expect("Either GEMINI_API_KEY or ANTHROPIC_API_KEY must be set");

    Arc::new(GeminiOrchestrator::new("gemini-2.0-flash", api_key).with_temperature(0.7))
}

/// Workspace root provider for testing.
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
async fn test_git_status_basic_check() {
    if skip_if_no_orchestrator_key() || skip_if_no_evaluator_key() {
        return;
    }

    println!("\n🧪 Testing: Basic Git Status Check");

    let scenario = git_status_basic();
    let evaluator = AiEvaluator::from_env().expect("Failed to create evaluator");

    // Get workspace root (RAD project directory)
    let mut workspace_root_path = env::current_dir().unwrap();
    while workspace_root_path.file_name().and_then(|n| n.to_str()) != Some("RAD") {
        if !workspace_root_path.pop() {
            workspace_root_path = env::current_dir().unwrap();
            break;
        }
    }

    // Create orchestrator with file AND git tools
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: workspace_root_path.clone(),
    });
    let mut tools = create_file_operation_tools(workspace_root.clone());
    tools.extend(create_git_extended_tools(workspace_root));

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
                let content_preview = if msg.content.len() > 500 {
                    format!("{}...(truncated)", &msg.content[..500])
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
            "Should call git_status tool".to_string(),
            "Should not assume workspace is clean without checking".to_string(),
            "Should report actual file statuses from git".to_string(),
            "Should show branch information".to_string(),
        ],
        min_score: 75,
    };

    let evaluation = evaluator
        .evaluate(
            &scenario.description,
            &scenario.user_input,
            &scenario.expected_behavior,
            &full_trace,
            &criteria,
        )
        .await
        .expect("Evaluation failed");

    // Print results
    println!("\n📊 AI Evaluation Results:");
    println!("   Score: {}/100", evaluation.score.unwrap_or(0));
    println!(
        "   Passed: {}",
        if evaluation.passed { "✓" } else { "✗" }
    );
    println!("   Feedback: {}", evaluation.feedback);
    println!("   Reasoning:\n{}", evaluation.reasoning);

    assert!(
        evaluation.passed,
        "AI evaluation failed: {}",
        evaluation.feedback
    );
    assert!(
        evaluation.score.unwrap_or(0) >= criteria.min_score,
        "Score {} below threshold {}",
        evaluation.score.unwrap_or(0),
        criteria.min_score
    );
}

#[tokio::test]
#[ignore = "AI validation test - requires API keys and costs money"]
async fn test_git_status_unstaged_changes() {
    if skip_if_no_orchestrator_key() || skip_if_no_evaluator_key() {
        return;
    }

    println!("\n🧪 Testing: Unstaged Changes Detection");

    let scenario = git_status_unstaged_changes();
    let evaluator = AiEvaluator::from_env().expect("Failed to create evaluator");

    // Get workspace root
    let mut workspace_root_path = env::current_dir().unwrap();
    while workspace_root_path.file_name().and_then(|n| n.to_str()) != Some("RAD") {
        if !workspace_root_path.pop() {
            workspace_root_path = env::current_dir().unwrap();
            break;
        }
    }

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: workspace_root_path.clone(),
    });
    let mut tools = create_file_operation_tools(workspace_root.clone());
    tools.extend(create_git_extended_tools(workspace_root));

    let provider = create_test_orchestrator();
    let engine = OrchestrationEngine::with_defaults(provider, tools);

    let mut context = OrchestrationContext::new("test-session");
    let result = engine.execute(&scenario.user_input, &mut context).await;

    let (response, full_trace) = match result {
        Ok(r) => {
            println!("✓ Orchestrator response received");

            let mut trace = String::new();
            trace.push_str("=== Full Orchestration Trace ===\n\n");
            trace.push_str("User Request: ");
            trace.push_str(&scenario.user_input);
            trace.push_str("\n\n");

            trace.push_str("Conversation History:\n");
            for (i, msg) in context.conversation_history.iter().enumerate() {
                let content_preview = if msg.content.len() > 500 {
                    format!("{}...(truncated)", &msg.content[..500])
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
            "Should call git_status tool to check working directory".to_string(),
            "Should distinguish between staged and unstaged changes".to_string(),
            "Should not report clean workspace without verification".to_string(),
            "Should list specific files with unstaged modifications".to_string(),
        ],
        min_score: 75,
    };

    let evaluation = evaluator
        .evaluate(
            &scenario.description,
            &scenario.user_input,
            &scenario.expected_behavior,
            &full_trace,
            &criteria,
        )
        .await
        .expect("Evaluation failed");

    println!("\n📊 AI Evaluation Results:");
    println!("   Score: {}/100", evaluation.score.unwrap_or(0));
    println!(
        "   Passed: {}",
        if evaluation.passed { "✓" } else { "✗" }
    );
    println!("   Feedback: {}", evaluation.feedback);
    println!("   Reasoning:\n{}", evaluation.reasoning);

    assert!(
        evaluation.passed,
        "AI evaluation failed: {}",
        evaluation.feedback
    );
    assert!(evaluation.score.unwrap_or(0) >= criteria.min_score);
}

#[tokio::test]
#[ignore = "AI validation test - requires API keys and costs money"]
async fn test_git_status_clean_workspace_verification() {
    if skip_if_no_orchestrator_key() || skip_if_no_evaluator_key() {
        return;
    }

    println!("\n🧪 Testing: Clean Workspace Verification");

    let scenario = git_status_clean_workspace();
    let evaluator = AiEvaluator::from_env().expect("Failed to create evaluator");

    let mut workspace_root_path = env::current_dir().unwrap();
    while workspace_root_path.file_name().and_then(|n| n.to_str()) != Some("RAD") {
        if !workspace_root_path.pop() {
            workspace_root_path = env::current_dir().unwrap();
            break;
        }
    }

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: workspace_root_path.clone(),
    });
    let mut tools = create_file_operation_tools(workspace_root.clone());
    tools.extend(create_git_extended_tools(workspace_root));

    let provider = create_test_orchestrator();
    let engine = OrchestrationEngine::with_defaults(provider, tools);

    let mut context = OrchestrationContext::new("test-session");
    let result = engine.execute(&scenario.user_input, &mut context).await;

    let (response, full_trace) = match result {
        Ok(r) => {
            println!("✓ Orchestrator response received");

            let mut trace = String::new();
            trace.push_str("=== Full Orchestration Trace ===\n\n");
            trace.push_str("User Request: ");
            trace.push_str(&scenario.user_input);
            trace.push_str("\n\n");

            trace.push_str("Conversation History:\n");
            for (i, msg) in context.conversation_history.iter().enumerate() {
                let content_preview = if msg.content.len() > 500 {
                    format!("{}...(truncated)", &msg.content[..500])
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
            "Should call git_status tool to verify clean state".to_string(),
            "Should not assume clean state based on user's phrasing".to_string(),
            "Should report actual status from git, not assumptions".to_string(),
        ],
        min_score: 75,
    };

    let evaluation = evaluator
        .evaluate(
            &scenario.description,
            &scenario.user_input,
            &scenario.expected_behavior,
            &full_trace,
            &criteria,
        )
        .await
        .expect("Evaluation failed");

    println!("\n📊 AI Evaluation Results:");
    println!("   Score: {}/100", evaluation.score.unwrap_or(0));
    println!(
        "   Passed: {}",
        if evaluation.passed { "✓" } else { "✗" }
    );
    println!("   Feedback: {}", evaluation.feedback);
    println!("   Reasoning:\n{}", evaluation.reasoning);

    assert!(
        evaluation.passed,
        "AI evaluation failed: {}",
        evaluation.feedback
    );
    assert!(evaluation.score.unwrap_or(0) >= criteria.min_score);
}

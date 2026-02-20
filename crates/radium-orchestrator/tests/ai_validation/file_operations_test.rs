//! File operations AI validation tests.
//!
//! Tests whether the orchestrator correctly uses file operation tools,
//! especially the new create_dir, delete_file, and rename_file tools,
//! and properly handles leading slash path auto-correction.

use crate::ai_validation::{
    evaluator::{AiEvaluator, EvaluationAspect, EvaluationCriteria},
    scenarios::{
        file_ops_create_dir_first, file_ops_delete_safely, file_ops_leading_slash_handling,
        file_ops_rename_file_usage,
    },
    skip_if_no_evaluator_key, skip_if_no_orchestrator_key,
};
use radium_orchestrator::orchestration::{
    context::OrchestrationContext,
    engine::OrchestrationEngine,
    file_tools::create_file_operation_tools,
    providers::GeminiOrchestrator,
};
use std::{env, path::PathBuf, sync::Arc};
use tempfile::TempDir;

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
async fn test_file_ops_leading_slash_handling() {
    if skip_if_no_orchestrator_key() || skip_if_no_evaluator_key() {
        return;
    }

    println!("\n🧪 Testing: Leading Slash Auto-Correction");

    let scenario = file_ops_leading_slash_handling();
    let evaluator = AiEvaluator::from_env().expect("Failed to create evaluator");

    // Use temp directory as workspace
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);
    let provider = create_test_orchestrator();
    let engine = OrchestrationEngine::with_defaults(provider, tools);

    // Execute the scenario
    let mut context = OrchestrationContext::new("test-session");
    let result = engine.execute(&scenario.user_input, &mut context).await;

    let (_response, full_trace) = match result {
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

            // Verify the file was created at workspace root, not system root
            let expected_path = temp_dir.path().join("config/settings.json");
            trace.push_str(&format!(
                "\n\nFile created at: {} (exists: {})",
                expected_path.display(),
                expected_path.exists()
            ));

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
            "Should use write_file tool with path containing leading slash".to_string(),
            "File should be created at workspace_root/config/settings.json".to_string(),
            "Should NOT create file at system root /config/".to_string(),
            "Should not show confusion about leading slash".to_string(),
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
    println!("   Passed: {}", if evaluation.passed { "✓" } else { "✗" });
    println!("   Feedback: {}", evaluation.feedback);
    println!("   Reasoning:\n{}", evaluation.reasoning);

    // Verify file was actually created at workspace root
    let expected_path = temp_dir.path().join("config/settings.json");
    assert!(
        expected_path.exists(),
        "File should be created at workspace root, not system root"
    );

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
async fn test_file_ops_rename_file_usage() {
    if skip_if_no_orchestrator_key() || skip_if_no_evaluator_key() {
        return;
    }

    println!("\n🧪 Testing: Rename File Tool Usage");

    let scenario = file_ops_rename_file_usage();
    let evaluator = AiEvaluator::from_env().expect("Failed to create evaluator");

    // Use temp directory and create initial file
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("old_name.txt"), "test content").unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);
    let provider = create_test_orchestrator();
    let engine = OrchestrationEngine::with_defaults(provider, tools);

    let mut context = OrchestrationContext::new("test-session");
    let result = engine.execute(&scenario.user_input, &mut context).await;

    let (_response, full_trace) = match result {
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
            "Should use rename_file tool".to_string(),
            "Should NOT use read+write+delete combination".to_string(),
            "Should specify old_path and new_path parameters".to_string(),
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
    println!("   Passed: {}", if evaluation.passed { "✓" } else { "✗" });
    println!("   Feedback: {}", evaluation.feedback);
    println!("   Reasoning:\n{}", evaluation.reasoning);

    assert!(
        evaluation.passed,
        "AI evaluation failed: {}",
        evaluation.feedback
    );
    assert!(
        evaluation.score.unwrap_or(0) >= criteria.min_score
    );
}

#[tokio::test]
#[ignore = "AI validation test - requires API keys and costs money"]
async fn test_file_ops_delete_safely() {
    if skip_if_no_orchestrator_key() || skip_if_no_evaluator_key() {
        return;
    }

    println!("\n🧪 Testing: Safe File Deletion");

    let scenario = file_ops_delete_safely();
    let evaluator = AiEvaluator::from_env().expect("Failed to create evaluator");

    // Use temp directory and create file to delete
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("temporary_file.txt"), "temp").unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);
    let provider = create_test_orchestrator();
    let engine = OrchestrationEngine::with_defaults(provider, tools);

    let mut context = OrchestrationContext::new("test-session");
    let result = engine.execute(&scenario.user_input, &mut context).await;

    let (_response, full_trace) = match result {
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
            "Should use delete_file tool".to_string(),
            "Should specify correct file path".to_string(),
            "Should execute deletion without unnecessary confirmation".to_string(),
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
    println!("   Passed: {}", if evaluation.passed { "✓" } else { "✗" });
    println!("   Feedback: {}", evaluation.feedback);
    println!("   Reasoning:\n{}", evaluation.reasoning);

    assert!(
        evaluation.passed,
        "AI evaluation failed: {}",
        evaluation.feedback
    );
    assert!(
        evaluation.score.unwrap_or(0) >= criteria.min_score
    );
}

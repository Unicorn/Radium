//! Vibe check command implementation.
//!
//! Allows users to manually trigger a vibe check for metacognitive oversight.

use anyhow::{Context, Result};
use colored::Colorize;
use radium_core::{
    context::ContextManager,
    oversight::{MetacognitiveService, OversightRequest, WorkflowPhase as OversightWorkflowPhase},
    policy::ConstitutionManager,
    workspace::Workspace,
    workflow::behaviors::vibe_check::{VibeCheckContext, WorkflowPhase as VibeCheckWorkflowPhase},
};
use radium_models::ModelFactory;

/// Execute the vibe check command.
///
/// Triggers a manual vibe check with metacognitive oversight.
pub async fn execute(
    phase: Option<String>,
    goal: Option<String>,
    plan: Option<String>,
    progress: Option<String>,
    task_context: Option<String>,
    json: bool,
) -> Result<()> {
    if !json {
        println!("{}", "rad vibecheck".bold().cyan());
        println!();
    }

    // Discover workspace
    let workspace = Workspace::discover()
        .context("No Radium workspace found. Run 'rad init' to create one.")?;

    // Initialize context manager
    let context_manager = ContextManager::new(&workspace);

    // Determine workflow phase
    let vibe_check_phase = match phase.as_deref() {
        Some("planning") => VibeCheckWorkflowPhase::Planning,
        Some("implementation") => VibeCheckWorkflowPhase::Implementation,
        Some("review") => VibeCheckWorkflowPhase::Review,
        _ => VibeCheckWorkflowPhase::Implementation, // Default
    };

    let oversight_phase = match phase.as_deref() {
        Some("planning") => OversightWorkflowPhase::Planning,
        Some("implementation") => OversightWorkflowPhase::Implementation,
        Some("review") => OversightWorkflowPhase::Review,
        _ => OversightWorkflowPhase::Implementation, // Default
    };

    // Get goal and plan from args or workspace
    let goal = goal.unwrap_or_else(|| {
        // Try to load from plan if available
        // For now, use a default
        "No goal specified".to_string()
    });

    let plan = plan.unwrap_or_else(|| {
        // Try to load from plan if available
        "No plan specified".to_string()
    });

    // Build vibe check context
    let mut vibe_context = VibeCheckContext::new(vibe_check_phase)
        .with_goal(goal.clone())
        .with_plan(plan.clone());

    if let Some(progress) = progress {
        vibe_context = vibe_context.with_progress(progress);
    }

    if let Some(task_context) = task_context {
        vibe_context = vibe_context.with_task_context(task_context);
    }

    // Initialize model for metacognitive service
    // Use default model from environment or fallback to mock
    let engine = std::env::var("RADIUM_ENGINE").unwrap_or_else(|_| "mock".to_string());
    let model_id = std::env::var("RADIUM_MODEL").unwrap_or_else(|_| String::new());
    let model = ModelFactory::create_from_str(&engine, model_id)
        .context("Failed to create model for oversight")?;

    let metacognitive = MetacognitiveService::new(model);
    let _constitution_manager = ConstitutionManager::new();

    // Build oversight request
    let mut oversight_request = OversightRequest::new(oversight_phase, goal, plan);

    if let Some(progress) = vibe_context.progress.clone() {
        oversight_request = oversight_request.with_progress(progress);
    }

    if let Some(task_context) = vibe_context.task_context.clone() {
        oversight_request = oversight_request.with_task_context(task_context);
    }

    // Add learning context if available
    if let Some(learning_context) = context_manager.gather_learning_context(3) {
        let context = learning_context
            .strip_prefix("# Learning Context\n\n")
            .and_then(|s| s.strip_suffix("\n"))
            .unwrap_or(&learning_context)
            .to_string();
        oversight_request = oversight_request.with_learning_context(context);
    }

    // Add history summary if available
    if let Ok(Some(history)) = context_manager.gather_memory_context("agent") {
        oversight_request = oversight_request.with_history_summary(history);
    }

    // Generate oversight
    if !json {
        println!("  {}", "Generating oversight feedback...".dimmed());
    }

    let oversight_response = metacognitive
        .generate_oversight(&oversight_request)
        .await
        .context("Failed to generate oversight feedback")?;

    // Display results
    if json {
        // JSON output
        let json_output = serde_json::json!({
            "risk_score": oversight_response.risk_score,
            "advice": oversight_response.advice,
            "traits": oversight_response.traits,
            "uncertainties": oversight_response.uncertainties,
            "helpful_patterns": oversight_response.helpful_patterns,
            "harmful_patterns": oversight_response.harmful_patterns,
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Formatted output
        println!();
        println!("{}", "Oversight Feedback".bold());
        println!();

        // Risk score with color coding
        let risk_display = if oversight_response.risk_score < 0.3 {
            format!("{:.2} (Low)", oversight_response.risk_score).green()
        } else if oversight_response.risk_score < 0.7 {
            format!("{:.2} (Medium)", oversight_response.risk_score).yellow()
        } else {
            format!("{:.2} (High)", oversight_response.risk_score).red()
        };

        println!("  {} Risk Score: {}", "•".cyan(), risk_display);
        println!();

        // Advice
        println!("  {}", "Advice:".bold());
        for line in oversight_response.advice.lines() {
            println!("    {}", line);
        }
        println!();

        // Traits
        if !oversight_response.traits.is_empty() {
            println!("  {} Traits:", "•".cyan());
            for trait_ in &oversight_response.traits {
                println!("    - {}", trait_);
            }
            println!();
        }

        // Uncertainties
        if !oversight_response.uncertainties.is_empty() {
            println!("  {} Uncertainties:", "•".cyan());
            for uncertainty in &oversight_response.uncertainties {
                println!("    - {}", uncertainty);
            }
            println!();
        }

        // Patterns (if available)
        if !oversight_response.helpful_patterns.is_empty() {
            println!("  {} Helpful Patterns:", "•".green());
            for pattern in &oversight_response.helpful_patterns {
                println!("    + {}", pattern);
            }
            println!();
        }

        if !oversight_response.harmful_patterns.is_empty() {
            println!("  {} Harmful Patterns:", "•".red());
            for pattern in &oversight_response.harmful_patterns {
                println!("    - {}", pattern);
            }
            println!();
        }
    }

    Ok(())
}


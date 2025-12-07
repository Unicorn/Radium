//! Autonomous execution command.
//!
//! Provides CLI command for autonomous execution from high-level goals.

use anyhow::{Context, Result};
use colored::Colorize;
use radium_core::autonomous::{AutonomousConfig, AutonomousOrchestrator};
use radium_core::agents::registry::AgentRegistry;
use radium_core::models::selector::ModelSelector;
use radium_core::storage::Database;
use radium_abstraction::Model;
use radium_orchestrator::{AgentExecutor, Orchestrator};
use std::sync::Arc;

/// Execute autonomous command.
pub async fn execute(goal: String) -> Result<()> {
    println!("{}", format!("rad autonomous: {}", goal).bold().cyan());
    println!();

    // Discover workspace
    let workspace = radium_core::workspace::Workspace::discover()
        .context("No Radium workspace found. Run 'rad init' first.")?;

    // Initialize database
    let db_path = workspace.radium_dir().join("radium.db");
    let db_path_str = db_path.to_string_lossy();
    let db = Arc::new(std::sync::Mutex::new(
        Database::open(&db_path_str).context("Failed to open database")?,
    ));

    // Initialize orchestrator and executor
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());

    // Initialize agent registry
    let agent_registry = Arc::new(AgentRegistry::with_discovery()
        .context("Failed to discover agents")?);

    // Initialize model selector and get model
    let mut model_selector = ModelSelector::new();
    let default_metadata = radium_core::agents::AgentMetadata::default();
    let model = model_selector
        .select_model(&radium_core::models::selector::SelectionOptions {
            agent_metadata: &default_metadata,
            estimated_prompt_tokens: None,
            estimated_completion_tokens: None,
            allow_premium_without_approval: false,
        })
        .context("Failed to select model")?
        .model;

    // Create autonomous orchestrator
    let config = AutonomousConfig::default();
    let orchestrator_autonomous = AutonomousOrchestrator::new(
        &orchestrator,
        &executor,
        &db,
        agent_registry,
        config,
    )
    .context("Failed to create autonomous orchestrator")?;

    // Execute autonomously
    println!("  {} Decomposing goal into workflow...", "•".dimmed());
    let result = orchestrator_autonomous
        .execute_autonomous(&goal, model)
        .await
        .context("Autonomous execution failed")?;

    println!();
    if result.success {
        println!("  {} Execution completed successfully", "✓".green());
        println!("  {} Workflow ID: {}", "•".dimmed(), result.workflow_id.cyan());
        println!("  {} Steps completed: {}", "•".dimmed(), result.steps_completed.to_string().green());
        if result.recoveries_performed > 0 {
            println!("  {} Recoveries performed: {}", "•".dimmed(), result.recoveries_performed);
        }
        if result.reassignments_performed > 0 {
            println!("  {} Reassignments performed: {}", "•".dimmed(), result.reassignments_performed);
        }
    } else {
        println!("  {} Execution failed", "✗".red());
        if let Some(error) = &result.error {
            println!("  {} Error: {}", "•".dimmed(), error.red());
        }
    }

    Ok(())
}


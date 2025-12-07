//! Complete command implementation.
//!
//! Executes a unified workflow that detects source, fetches content,
//! generates a plan, and executes it automatically.

use anyhow::Context;
use colored::Colorize;
use radium_core::{
    workflow::{CompletionEvent, CompletionOptions, CompletionService},
    Workspace,
};

/// Execute the complete command.
///
/// Automatically detects source type, fetches content, generates a plan,
/// and executes it without user intervention (YOLO mode).
pub async fn execute(source: String) -> anyhow::Result<()> {
    println!("{}", "rad complete".bold().cyan());
    println!();

    // Discover workspace
    let workspace = Workspace::discover().context("Failed to discover workspace")?;
    workspace.ensure_structure().context("Failed to ensure workspace structure")?;

    // Create completion service
    let service = CompletionService::new();

    // Create options
    let options = CompletionOptions {
        workspace_path: workspace.root().to_path_buf(),
        engine: std::env::var("RADIUM_ENGINE").unwrap_or_else(|_| "mock".to_string()),
        model_id: std::env::var("RADIUM_MODEL").ok(),
        requirement_id: None,
    };

    // Execute workflow
    let mut event_rx = service
        .execute(source.clone(), options)
        .await
        .context("Failed to start completion workflow")?;

    // Process events
    while let Some(event) = event_rx.recv().await {
        match event {
            CompletionEvent::Detected { source_type } => {
                println!("  {} Detected source: {}", "ℹ️".cyan(), source_type.green());
            }
            CompletionEvent::Fetching => {
                println!("  {} Fetching requirements...", "⬇️".cyan());
            }
            CompletionEvent::Planning => {
                print!("  {} Generating plan...", "🧠".cyan());
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
            CompletionEvent::PlanGenerated { iterations, tasks } => {
                println!();
                println!(
                    "  {} Generated plan with {} iterations, {} tasks",
                    "✓".green(),
                    iterations.to_string().green(),
                    tasks.to_string().green()
                );
            }
            CompletionEvent::PlanPersisted { path } => {
                println!("  {} Plan saved to: {}", "✓".green(), path.display().to_string().cyan());
            }
            CompletionEvent::ExecutionStarted { total_tasks } => {
                println!();
                println!("  {} Executing {} tasks...", "🚀".cyan(), total_tasks.to_string().cyan());
                println!();
            }
            CompletionEvent::TaskProgress {
                current,
                total,
                task_name,
            } => {
                println!(
                    "    {} Executing Task {}/{}: {}",
                    "→".cyan(),
                    current.to_string().cyan(),
                    total.to_string().dimmed(),
                    task_name
                );
            }
            CompletionEvent::TaskCompleted { task_name } => {
                println!("      {} Task completed: {}", "✓".green(), task_name.green());
            }
            CompletionEvent::Completed => {
                println!();
                println!("{}", "✓ Completion workflow finished successfully!".green().bold());
                println!();
                break;
            }
            CompletionEvent::Error { message } => {
                println!();
                println!("  {} Error: {}", "✗".red(), message.red());
                return Err(anyhow::anyhow!("Completion workflow failed: {}", message));
            }
        }
    }

    Ok(())
}


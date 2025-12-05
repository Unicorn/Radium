//! Craft command implementation.
//!
//! Executes a generated plan through its iterations and tasks.

use anyhow::{Context, bail};
use colored::Colorize;
use radium_core::{
    ExecutionConfig, PlanDiscovery, PlanExecutor, PlanManifest, PlanStatus, RequirementId,
    Workspace,
};
use radium_models::ModelFactory;

/// Execute the craft command.
///
/// Executes a generated plan through its iterations and tasks.
pub async fn execute(
    plan_identifier: Option<String>,
    iteration: Option<String>,
    task: Option<String>,
    resume: bool,
    dry_run: bool,
    json: bool,
) -> anyhow::Result<()> {
    println!("{}", "rad craft".bold().cyan());
    println!();

    // Discover workspace
    let workspace = Workspace::discover().context("Failed to discover workspace")?;

    // Get plan identifier
    let plan_id = plan_identifier.ok_or_else(|| {
        anyhow::anyhow!("Plan identifier is required. Usage: rad craft <plan-id>")
    })?;

    // Check if plan_identifier is a file path
    let plan_path = std::path::PathBuf::from(&plan_id);
    let (project_name, mut manifest, manifest_path) = if plan_path.exists() && plan_path.is_file() {
        println!("  Loading plan from file: {}", plan_id.green());
        let content = std::fs::read_to_string(&plan_path).context("Failed to read plan file")?;

        // Try to deserialize as PlanManifest
        let manifest: PlanManifest =
            serde_json::from_str(&content).context("Failed to parse plan manifest")?;

        (manifest.project_name.clone(), manifest, plan_path)
    } else {
        // Find the plan in workspace
        println!("  Looking for plan: {}", plan_id.green());

        let discovery = PlanDiscovery::new(&workspace);
        let discovered_plan = if plan_id.starts_with("REQ-") {
            // Try to parse as requirement ID
            let req_id: RequirementId = plan_id.parse().context("Invalid requirement ID format")?;
            discovery.find_by_requirement_id(req_id).context("Failed to search for plan")?
        } else {
            // Try to find by folder name
            discovery.find_by_folder_name(&plan_id).context("Failed to search for plan")?
        };

        let discovered_plan =
            discovered_plan.ok_or_else(|| anyhow::anyhow!("Plan not found: {}", plan_id))?;

        println!("  ✓ Found plan at: {}", discovered_plan.path.display().to_string().dimmed());
        println!();

        // Load plan manifest
        if !discovered_plan.has_manifest {
            bail!(
                "Plan manifest not found at {}/plan/plan_manifest.json",
                discovered_plan.path.display()
            );
        }

        let manifest = discovered_plan.load_manifest().context("Failed to load plan manifest")?;
        let manifest_path = discovered_plan.path.join("plan/plan_manifest.json");

        (discovered_plan.plan.project_name, manifest, manifest_path)
    };

    // Display plan information
    display_plan_info(&project_name, &manifest)?;

    // Check for dry run mode
    if dry_run {
        println!();
        println!("{}", "Dry run mode - no execution".yellow());
        return Ok(());
    }

    // Execute the plan
    println!();
    println!("{}", "Executing plan...".bold());
    println!();

    execute_plan(
        &mut manifest,
        &manifest_path,
        iteration.as_deref(),
        task.as_deref(),
        resume,
        json,
    )
    .await?;

    println!();
    println!("{}", "Plan execution completed!".green().bold());
    println!();

    Ok(())
}

/// Display plan information.
fn display_plan_info(project_name: &str, manifest: &PlanManifest) -> anyhow::Result<()> {
    println!("{}", "Plan Information:".bold());
    println!("  Project: {}", project_name.green());
    println!("  Requirement ID: {}", manifest.requirement_id.to_string().green());
    println!("  Iterations: {}", manifest.iterations.len().to_string().cyan());

    // Count total tasks
    let total_tasks: usize = manifest.iterations.iter().map(|i| i.tasks.len()).sum();
    println!("  Total Tasks: {}", total_tasks.to_string().cyan());

    // Display iteration summary
    println!();
    println!("{}", "Iterations:".bold());
    for iteration in &manifest.iterations {
        let status_str = match iteration.status {
            PlanStatus::NotStarted => "Not Started".dimmed(),
            PlanStatus::InProgress => "In Progress".yellow(),
            PlanStatus::Completed => "Completed".green(),
            PlanStatus::Failed => "Failed".red(),
            PlanStatus::Paused => "Paused".yellow(),
            PlanStatus::Blocked => "Blocked".red(),
        };

        let completed_tasks = iteration.tasks.iter().filter(|t| t.completed).count();
        let total = iteration.tasks.len();

        println!(
            "  {} - {} ({}/{} tasks) [{}]",
            iteration.id.cyan(),
            iteration.name,
            completed_tasks,
            total,
            status_str
        );
    }

    Ok(())
}

/// Execute the plan with state persistence.
async fn execute_plan(
    manifest: &mut PlanManifest,
    manifest_path: &std::path::Path,
    iteration_filter: Option<&str>,
    task_filter: Option<&str>,
    resume: bool,
    _json: bool,
) -> anyhow::Result<()> {
    // Create executor with configuration
    let config = ExecutionConfig {
        resume,
        skip_completed: !resume,
        check_dependencies: true,
        state_path: manifest_path.to_path_buf(),
    };
    let executor = PlanExecutor::with_config(config);

    // Determine which iterations to execute
    let iteration_ids: Vec<String> = if let Some(iter_id) = iteration_filter {
        vec![iter_id.to_string()]
    } else {
        manifest.iterations.iter().map(|i| i.id.clone()).collect()
    };

    if iteration_ids.is_empty() {
        bail!("No iterations found to execute");
    }

    // Execute each iteration
    for iter_id in iteration_ids {
        let iteration = manifest
            .get_iteration(&iter_id)
            .ok_or_else(|| anyhow::anyhow!("Iteration not found: {}", iter_id))?;

        // Skip completed iterations if not resuming
        if !resume && iteration.status == PlanStatus::Completed {
            println!("  {} Skipping completed iteration: {}", "→".dimmed(), iter_id.dimmed());
            continue;
        }

        println!("{}", format!("Iteration {}", iter_id).bold().cyan());
        println!("  Goal: {}", iteration.goal.as_ref().unwrap_or(&"No goal specified".to_string()));
        println!();

        // Determine which tasks to execute
        let task_ids: Vec<String> = if let Some(task_id) = task_filter {
            vec![task_id.to_string()]
        } else {
            iteration.tasks.iter().map(|t| t.id.clone()).collect()
        };

        if task_ids.is_empty() {
            println!("  {} No tasks to execute", "!".yellow());
            continue;
        }

        // Execute each task
        for task_id in task_ids {
            let iteration = manifest.get_iteration(&iter_id).unwrap();
            let task = iteration
                .get_task(&task_id)
                .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;

            // Skip completed tasks if not resuming
            if !resume && task.completed {
                println!("    {} {}", "✓".green(), task.title.dimmed());
                continue;
            }

            println!("    {} {}", "→".cyan(), task.title);

            // Check dependencies
            if let Err(e) = executor.check_dependencies(manifest, task) {
                println!("      {} Dependency not met: {}", "✗".red(), e.to_string().red());
                println!("      {} Skipping task", "→".yellow());
                continue;
            }

            // Get agent and model
            let agent_id = if let Some(id) = &task.agent_id {
                id
            } else {
                println!("      {} No agent assigned, skipping", "!".yellow());
                continue;
            };

            println!("      {} Agent: {}", "•".dimmed(), agent_id.cyan());
            println!("      {} Executing...", "•".cyan());

            // Create model instance
            let engine = "mock"; // Default to mock for now
            let model_id = String::new();

            let model = match ModelFactory::create_from_str(engine, model_id) {
                Ok(m) => m,
                Err(e) => {
                    println!("      {} Failed to create model: {}", "✗".red(), e.to_string().red());
                    continue;
                }
            };

            // Execute task
            match executor.execute_task(task, model).await {
                Ok(result) => {
                    if result.success {
                        println!("      {} Execution complete", "✓".green());

                        if let Some(response) = &result.response {
                            println!();
                            println!("      {}", "Response:".bold().green());
                            println!("      {}", "─".repeat(60).dimmed());

                            for line in response.lines().take(5) {
                                println!("      {}", line);
                            }

                            if response.lines().count() > 5 {
                                println!(
                                    "      {} ... ({} more lines)",
                                    "".dimmed(),
                                    response.lines().count() - 5
                                );
                            }

                            println!("      {}", "─".repeat(60).dimmed());
                        }

                        if let Some((prompt, completion)) = result.tokens_used {
                            println!(
                                "      {} Tokens: {} prompt, {} completion",
                                "•".dimmed(),
                                prompt.to_string().dimmed(),
                                completion.to_string().dimmed()
                            );
                        }

                        // Mark task as complete
                        executor.mark_task_complete(manifest, &iter_id, &task_id)?;

                        // Save checkpoint
                        executor.save_manifest(manifest, manifest_path)?;

                        // Show progress
                        let progress = executor.calculate_progress(manifest);
                        println!(
                            "      {} Progress: {}%",
                            "•".dimmed(),
                            progress.to_string().cyan()
                        );
                    } else {
                        println!(
                            "      {} Execution failed: {}",
                            "✗".red(),
                            result.error.unwrap_or_default().red()
                        );
                        bail!("Task execution failed");
                    }
                }
                Err(e) => {
                    println!("      {} Execution error: {}", "✗".red(), e.to_string().red());
                    bail!("Task execution error: {}", e);
                }
            }

            println!();
        }

        println!();
    }

    Ok(())
}

//! Craft command implementation.
//!
//! Executes a generated plan through its iterations and tasks.

use anyhow::{Context, bail};
use colored::Colorize;
use radium_core::{
    AgentDiscovery, PlanDiscovery, PlanManifest, PlanStatus, PromptContext, PromptTemplate,
    RequirementId, Workspace,
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
    let (project_name, manifest) = if plan_path.exists() && plan_path.is_file() {
        println!("  Loading plan from file: {}", plan_id.green());
        let content = std::fs::read_to_string(&plan_path).context("Failed to read plan file")?;

        // Try to deserialize as PlanManifest
        let manifest: PlanManifest =
            serde_json::from_str(&content).context("Failed to parse plan manifest")?;

        (manifest.project_name.clone(), manifest)
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

        (discovered_plan.plan.project_name, manifest)
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

    execute_plan(&manifest, iteration.as_deref(), task.as_deref(), resume, json).await?;

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

/// Execute the plan.
async fn execute_plan(
    manifest: &PlanManifest,
    iteration_filter: Option<&str>,
    task_filter: Option<&str>,
    resume: bool,
    json: bool,
) -> anyhow::Result<()> {
    // Determine which iterations to execute
    let iterations_to_execute: Vec<_> = if let Some(iter_id) = iteration_filter {
        manifest.iterations.iter().filter(|i| i.id == iter_id).collect()
    } else {
        manifest.iterations.iter().collect()
    };

    if iterations_to_execute.is_empty() {
        bail!("No iterations found to execute");
    }

    // Execute each iteration
    for iteration in iterations_to_execute {
        // Skip completed iterations if not resuming
        if !resume && iteration.status == PlanStatus::Completed {
            println!("  {} Skipping completed iteration: {}", "→".dimmed(), iteration.id.dimmed());
            continue;
        }

        println!("{}", format!("Iteration {}", iteration.id).bold().cyan());
        println!("  Goal: {}", iteration.goal.as_ref().unwrap_or(&"No goal specified".to_string()));
        println!();

        // Determine which tasks to execute
        let tasks_to_execute: Vec<_> = if let Some(task_id) = task_filter {
            iteration.tasks.iter().filter(|t| t.id == task_id).collect()
        } else {
            iteration.tasks.iter().collect()
        };

        if tasks_to_execute.is_empty() {
            println!("  {} No tasks to execute", "!".yellow());
            continue;
        }

        // Execute each task
        for task in tasks_to_execute {
            // Skip completed tasks if not resuming
            if !resume && task.completed {
                println!("    {} {}", "✓".green(), task.title.dimmed());
                continue;
            }

            println!("    {} {}", "→".cyan(), task.title);

            // TODO: Execute actual task with agent
            // For now, we simulate execution
            execute_task_stub(task, json).await?;
        }

        println!();
    }

    Ok(())
}

/// Execute a task with its assigned agent.
async fn execute_task_stub(task: &radium_core::PlanTask, _json: bool) -> anyhow::Result<()> {
    println!("      {} Executing task...", "•".cyan());
    println!("      {} Task ID: {}", "•".dimmed(), task.id.dimmed());

    if let Some(desc) = &task.description {
        println!("      {} Description: {}", "•".dimmed(), desc.dimmed());
    }

    // Check if agent is assigned
    let agent_id = match &task.agent_id {
        Some(id) => id,
        None => {
            println!("      {} No agent assigned", "!".yellow());
            return Ok(());
        }
    };

    println!("      {} Agent: {}", "•".dimmed(), agent_id.cyan());

    // Discover and load agent configuration
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all().context("Failed to discover agents")?;

    let agent =
        agents.get(agent_id).ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

    // Load and render prompt
    let prompt_content = std::fs::read_to_string(&agent.prompt_path)
        .context(format!("Failed to load prompt from {:?}", agent.prompt_path))?;

    let template = PromptTemplate::from_str(prompt_content);

    // Create context with task information
    let mut context = PromptContext::new();
    context.set("task_id", task.id.clone());
    context.set("task_title", task.title.clone());
    if let Some(desc) = &task.description {
        context.set("task_description", desc.clone());
    }

    let rendered = template.render(&context)?;

    // Execute with model
    let engine = agent.engine.as_deref().unwrap_or("mock");
    let model_id = agent.model.clone().unwrap_or_default();

    println!("      {} Executing with {}...", "•".cyan(), engine);

    // Try to create model instance
    match ModelFactory::create_from_str(engine, model_id.clone()) {
        Ok(model_instance) => {
            // Execute real model
            match model_instance.generate_text(&rendered, None).await {
                Ok(response) => {
                    println!("      {} Execution complete", "✓".green());
                    println!();
                    println!("      {}", "Response:".bold().green());
                    println!("      {}", "─".repeat(60).dimmed());

                    // Indent response content
                    for line in response.content.lines() {
                        println!("      {}", line);
                    }

                    println!("      {}", "─".repeat(60).dimmed());

                    if let Some(usage) = response.usage {
                        println!(
                            "      {} Tokens: {} prompt, {} completion",
                            "•".dimmed(),
                            usage.prompt_tokens.to_string().dimmed(),
                            usage.completion_tokens.to_string().dimmed()
                        );
                    }

                    Ok(())
                }
                Err(e) => {
                    println!("      {} Model execution failed: {}", "✗".red(), e.to_string().red());
                    Err(anyhow::anyhow!("Model execution failed: {}", e))
                }
            }
        }
        Err(e) => {
            println!("      {} Could not initialize model: {}", "!".yellow(), e);
            println!(
                "      {} Set API key: export {}_API_KEY=your-key",
                "i".cyan(),
                engine.to_uppercase()
            );
            println!("      {} Falling back to mock model...", "→".dimmed());

            // Fall back to mock model
            let mock_model = ModelFactory::create_from_str("mock", model_id)?;
            let response = mock_model.generate_text(&rendered, None).await?;

            println!("      {} Mock execution complete", "✓".yellow());
            println!("      {}", "Mock Response:".bold().yellow());
            println!("      {}", "─".repeat(60).dimmed());

            for line in response.content.lines().take(3) {
                println!("      {}", line.dimmed());
            }

            println!("      {}", "─".repeat(60).dimmed());

            Ok(())
        }
    }
}

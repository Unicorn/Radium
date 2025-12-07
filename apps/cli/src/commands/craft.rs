//! Craft command implementation.
//!
//! Executes a generated plan through its iterations and tasks.

use anyhow::{Context, bail};
use chrono::Utc;
use colored::Colorize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tokio::signal;
use radium_core::{
    analytics::{ReportFormatter, SessionAnalytics, SessionReport, SessionStorage},
    context::{ContextFileLoader, ContextManager}, AgentDiscovery, ExecutionConfig, monitoring::MonitoringService, PlanDiscovery,
    PlanExecutor, PlanManifest, PlanStatus, RequirementId, RunMode, Workspace,
    memory::MemoryStore,
};
use std::sync::{Arc, Mutex};
use radium_models::ModelFactory;
use uuid::Uuid;

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
    yolo: bool,
    engine: Option<String>,
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
    let (project_name, mut manifest, manifest_path, plan_dir) = if plan_path.exists() && plan_path.is_file() {
        println!("  Loading plan from file: {}", plan_id.green());
        let content = std::fs::read_to_string(&plan_path).context("Failed to read plan file")?;

        // Try to deserialize as PlanManifest
        let manifest: PlanManifest =
            serde_json::from_str(&content).context("Failed to parse plan manifest")?;

        let plan_dir = plan_path.parent().unwrap_or(&plan_path).to_path_buf();
        (manifest.project_name.clone(), manifest, plan_path, plan_dir)
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
        let plan_dir = discovered_plan.path.clone();

        (discovered_plan.plan.project_name, manifest, manifest_path, plan_dir)
    };

    // Display plan information
    display_plan_info(&project_name, &manifest)?;

    // Check for dry run mode
    if dry_run {
        println!();
        println!("{}", "Dry run mode - no execution".yellow());
        return Ok(());
    }

    // Load context files from plan directory
    let workspace_root = workspace.root().to_path_buf();
    let loader = ContextFileLoader::new(&workspace_root);
    let context_files = loader.load_hierarchical(&plan_dir).unwrap_or_default();

    // Generate session ID for tracking
    let session_id = Uuid::new_v4().to_string();
    let session_start_time = Utc::now();
    let mut executed_agent_ids = Vec::new();
    
    // Open monitoring service for agent tracking
    let monitoring_path = workspace.radium_dir().join("monitoring.db");
    let monitoring = MonitoringService::open(&monitoring_path).ok();

    // Discover agents for engine/model resolution
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all().context("Failed to discover agents")?;

    // Execute the plan
    println!();
    println!("{}", "Executing plan...".bold());
    if !context_files.is_empty() {
        let context_file_paths = loader.get_context_file_paths(&plan_dir);
        println!("  {} Loaded context from {} file(s)", "✓".green(), context_file_paths.len());
    }
    println!();

    execute_plan(
        &mut manifest,
        &manifest_path,
        iteration.as_deref(),
        task.as_deref(),
        resume,
        json,
        yolo,
        engine.as_deref(),
        if context_files.is_empty() { None } else { Some(context_files) },
        &mut Some(&mut executed_agent_ids),
        &session_id,
        monitoring.as_ref(),
        &agents,
        &workspace,
    )
    .await?;

    // Generate and display session report
    let session_end_time = Some(Utc::now());
    
    // Generate session metrics if we have monitoring and agent IDs
    if let Some(monitoring) = monitoring {
        if !executed_agent_ids.is_empty() {
            let analytics = SessionAnalytics::new(monitoring);
            
            // Generate session metrics
            if let Ok(metrics) = analytics.generate_session_metrics_with_workspace(
                &session_id,
                &executed_agent_ids,
                session_start_time,
                session_end_time,
                Some(workspace.root()),
            ) {
                let report = SessionReport::new(metrics);
                
                // Save report
                let storage = SessionStorage::new(workspace.root())?;
                if let Ok(_) = storage.save_report(&report) {
                    // Display session summary
                    display_session_summary(&report);
                }
            }
        }
    }

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

/// Display session summary at end of execution.
fn display_session_summary(report: &SessionReport) {
    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!("{}", "Session Summary".bold().cyan());
    println!("{}", "─".repeat(60).dimmed());
    
    let formatter = ReportFormatter;
    let summary = formatter.format(report);
    
    // Print a condensed version (first few lines)
    for line in summary.lines().take(15) {
        println!("{}", line);
    }
    
    println!();
    println!("  {} Full report: {}", "💡".cyan(), format!("rad stats session {}", report.metrics.session_id).dimmed());
    println!("{}", "─".repeat(60).dimmed());
    println!();
}

/// Execute the plan with state persistence.
async fn execute_plan(
    manifest: &mut PlanManifest,
    manifest_path: &std::path::Path,
    iteration_filter: Option<&str>,
    task_filter: Option<&str>,
    resume: bool,
    _json: bool,
    yolo: bool,
    engine: Option<&str>,
    context_files: Option<String>,
    executed_agent_ids: &mut Option<&mut Vec<String>>,
    session_id: &str,
    monitoring: Option<&MonitoringService>,
    agents: &std::collections::HashMap<String, radium_core::agents::config::AgentConfig>,
    workspace: &Workspace,
) -> anyhow::Result<()> {
    // Determine run mode based on yolo flag
    let run_mode = if yolo {
        RunMode::Continuous
    } else {
        RunMode::Bounded(5) // Default bounded limit
    };

    // Initialize MemoryStore and ContextManager for the plan
    let requirement_id = manifest.requirement_id;
    
    // Create memory store for persisting agent outputs
    let memory_store = Arc::new(Mutex::new(
        MemoryStore::new(workspace.root(), requirement_id)
            .context("Failed to create memory store")?
    ));

    // Create context manager for comprehensive context gathering
    let context_manager = Arc::new(Mutex::new(
        ContextManager::for_plan(&workspace, requirement_id)
            .context("Failed to create context manager")?
    ));

    // Create executor with configuration
    let config = ExecutionConfig {
        resume,
        skip_completed: !resume,
        check_dependencies: true,
        state_path: manifest_path.to_path_buf(),
        context_files,
        run_mode,
        context_manager: Some(context_manager),
        memory_store: Some(memory_store),
        requirement_id: Some(requirement_id),
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

    // Execution loop with iteration tracking
    let mut execution_iteration = 0;
    const CONTINUOUS_SANITY_LIMIT: usize = 1000;
    let abort_requested = Arc::new(AtomicBool::new(false));
    let start_time = Instant::now();

    // Register SIGINT handler for graceful shutdown
    let abort_flag = abort_requested.clone();
    tokio::spawn(async move {
        if let Ok(()) = signal::ctrl_c().await {
            abort_flag.store(true, Ordering::Relaxed);
        }
    });

    loop {
        execution_iteration += 1;
        
        // Print progress at start of each execution iteration
        let elapsed = start_time.elapsed();
        executor.print_progress(manifest, execution_iteration, elapsed, None);

        // Check if we should continue based on run mode
        let should_continue = match run_mode {
            RunMode::Bounded(max) => {
                if execution_iteration > max {
                    println!("  {} Reached maximum iterations ({}). Stopping execution.", "→".yellow(), max);
                    break;
                }
                execution_iteration <= max
            }
            RunMode::Continuous => {
                if execution_iteration > CONTINUOUS_SANITY_LIMIT {
                    println!("  {} Reached sanity limit ({}). Stopping execution.", "→".yellow(), CONTINUOUS_SANITY_LIMIT);
                    break;
                }
                true
            }
        };

        // Check if all tasks are complete
        if !executor.has_incomplete_tasks(manifest) {
            println!("  {} All tasks completed. Execution finished.", "✓".green());
            break;
        }

        // Check abort flag (for SIGINT handling)
        if abort_requested.load(Ordering::Relaxed) {
            println!("\n{}", "Execution aborted by user. Progress saved to plan_manifest.json".yellow());
            // Save manifest before exiting
            executor.save_manifest(manifest, manifest_path)?;
            std::process::exit(130); // Standard exit code for SIGINT
        }

        // Execute each iteration
        for iter_id in &iteration_ids {
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
            
            // Update progress with current task name
            let elapsed = start_time.elapsed();
            executor.print_progress(manifest, execution_iteration, elapsed, Some(&task.title));

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

            // Get agent config
            let agent = agents.get(agent_id).ok_or_else(|| {
                anyhow::anyhow!("Agent not found: {}", agent_id)
            })?;

            println!("      {} Agent: {}", "•".dimmed(), agent_id.cyan());
            println!("      {} Executing...", "•".cyan());

            // Register agent in monitoring for session tracking
            // Create agent ID that includes session context for lookup
            let tracked_agent_id = format!("{}-{}", session_id, agent_id);
            if let Some(monitoring) = monitoring {
                use radium_core::monitoring::{AgentRecord, AgentStatus};
                let mut agent_record = AgentRecord::new(tracked_agent_id.clone(), agent_id.clone());
                agent_record.plan_id = Some(session_id.to_string());
                if let Err(e) = monitoring.register_agent(&agent_record) {
                    eprintln!("      {} Warning: Failed to register agent: {}", "⚠".yellow(), e);
                } else {
                    let _ = monitoring.update_status(&tracked_agent_id, AgentStatus::Running);
                }
            }

            // Track agent ID for session analytics
            if let Some(agent_ids) = executed_agent_ids.as_mut() {
                agent_ids.push(tracked_agent_id.clone());
            }

            // Create model instance
            // Engine resolution: CLI flag → Agent config → Default "mock"
            let selected_engine = engine
                .or_else(|| agent.engine.as_deref())
                .unwrap_or("mock");
            let model_id = agent.model.as_deref().unwrap_or("default").to_string();

            // Display engine info if different from agent config
            if engine.is_some() && engine != agent.engine.as_deref() {
                println!("      {} Using engine override: {}", "→".cyan(), selected_engine.cyan());
            }

            let model = match ModelFactory::create_from_str(selected_engine, model_id) {
                Ok(m) => m,
                Err(e) => {
                    println!("      {} Failed to create model: {}", "✗".red(), e.to_string().red());
                    continue;
                }
            };

            // Execute task with retry logic for recoverable errors
            let task_result = executor.execute_task_with_retry(
                task,
                model,
                3, // max_retries
                1000, // base_delay_ms
            ).await;

            match task_result {
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
                            
                            // Record telemetry if monitoring is available
                            if let Some(monitoring) = monitoring {
                                use radium_core::monitoring::{TelemetryRecord, TelemetryTracking};
                                let mut telemetry = TelemetryRecord::new(tracked_agent_id.clone())
                                    .with_tokens(prompt as u64, completion as u64)
                                    .with_engine_id(selected_engine.to_string());
                                if let Some(model) = agent.model.as_deref() {
                                    telemetry = telemetry.with_model(model.to_string(), selected_engine.to_string());
                                }
                                telemetry.calculate_cost();
                                if let Err(e) = monitoring.record_telemetry(&telemetry).await {
                                    eprintln!("      {} Warning: Failed to record telemetry: {}", "⚠".yellow(), e);
                                }
                            }
                        }

                        // Complete agent in monitoring
                        if let Some(monitoring) = monitoring {
                            use radium_core::monitoring::AgentStatus;
                            let _ = monitoring.complete_agent(&tracked_agent_id, 0);
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
        }

        println!();

        // Re-evaluate manifest state after each execution cycle
        // Save manifest to persist progress
        executor.save_manifest(manifest, manifest_path)?;

        // Check abort flag again (user might have pressed Ctrl+C during execution)
        if abort_requested.load(Ordering::Relaxed) {
            println!("\n{}", "Execution aborted by user. Progress saved to plan_manifest.json".yellow());
            executor.save_manifest(manifest, manifest_path)?;
            std::process::exit(130); // Standard exit code for SIGINT
        }

        // Check again if we should continue (tasks might have been completed in this cycle)
        // The loop will check conditions at the start of the next iteration
    }

    Ok(())
}

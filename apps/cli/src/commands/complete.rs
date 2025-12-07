//! Complete command implementation.
//!
//! Executes a unified workflow that detects source, fetches content,
//! generates a plan, and executes it automatically.

use anyhow::{Context, bail};
use colored::Colorize;
use radium_core::{
    generate_plan_files, Iteration, Plan, PlanGenerator, PlanManifest, PlanParser, PlanStatus,
    PlanTask, RequirementId, Workspace,
    workflow::{detect_source, fetch_source_content, SourceDetectionError, SourceFetchError, SourceType},
    context::ContextFileLoader, ExecutionConfig, monitoring::MonitoringService, PlanDiscovery,
    PlanExecutor, RunMode,
};
use radium_models::ModelFactory;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::signal;
use uuid::Uuid;
use chrono::Utc;

/// Execute the complete command.
///
/// Automatically detects source type, fetches content, generates a plan,
/// and executes it without user intervention (YOLO mode).
pub async fn execute(source: String) -> anyhow::Result<()> {
    println!("{}", "rad complete".bold().cyan());
    println!();

    // Step 1: Detect source type
    println!("{}", "Step 1: Detecting source type...".bold());
    let source_type = detect_source(&source)
        .map_err(|e| anyhow::anyhow!("Source detection failed: {}", e))?;
    
    let source_desc = match &source_type {
        SourceType::LocalFile(path) => {
            format!("File: {}", path.display())
        }
        SourceType::JiraTicket(id) => {
            format!("Jira ticket: {}", id)
        }
        SourceType::BraingridReq(id) => {
            format!("Braingrid REQ: {}", id)
        }
        SourceType::Invalid => {
            bail!("Invalid source type detected");
        }
    };
    println!("  ✓ {}", source_desc.green());
    println!();

    // Step 2: Fetch content
    println!("{}", "Step 2: Fetching content...".bold());
    let spec_content = fetch_source_content(source_type)
        .await
        .map_err(|e| match e {
            SourceFetchError::MissingCredentials(ref _provider, ref cmd) => {
                anyhow::anyhow!("{} Please run `rad auth login {}` first.", e, cmd)
            }
            _ => anyhow::anyhow!("Failed to fetch content: {}", e)
        })?;
    
    println!("  ✓ Fetched {} bytes", spec_content.len().to_string().green());
    println!();

    // Step 3: Discover workspace
    let workspace = Workspace::discover().context("Failed to discover workspace")?;
    workspace.ensure_structure().context("Failed to ensure workspace structure")?;

    // Step 4: Generate plan
    println!("{}", "Step 3: Generating plan...".bold());
    
    // Generate requirement ID
    let requirement_id = RequirementId::next(workspace.root().join(".radium"))
        .context("Failed to generate requirement ID")?;
    
    println!("  Requirement ID: {}", requirement_id.to_string().green());

    // Extract project name from spec
    let project_name = extract_project_name(&spec_content)
        .unwrap_or_else(|| "project".to_string());
    let folder_name = format!("{}-{}", requirement_id, slugify(&project_name));
    
    println!("  Folder name: {}", folder_name.green());
    println!();

    // Create plan directory
    let plan_dir = workspace.structure().backlog_dir().join(&folder_name);
    
    if plan_dir.exists() {
        bail!("Plan directory already exists: {}\nUse a different ID or name.", plan_dir.display());
    }

    create_plan_structure(&plan_dir).context("Failed to create plan structure")?;
    
    // Copy specification file
    let spec_dest = plan_dir.join("specifications.md");
    fs::write(&spec_dest, spec_content.as_bytes()).context("Failed to write specifications.md")?;

    // Generate AI-powered plan
    let engine = std::env::var("RADIUM_ENGINE").unwrap_or_else(|_| "mock".to_string());
    let model_id = std::env::var("RADIUM_MODEL").unwrap_or_else(|_| String::new());
    let model = ModelFactory::create_from_str(&engine, model_id)
        .context("Failed to create model for plan generation")?;

    let generator = PlanGenerator::new();
    let parsed_plan = generator
        .generate(&spec_content, model)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to generate plan: {}", e))?;

    println!("  ✓ Generated plan with {} iterations", parsed_plan.iterations.len());

    // Convert ParsedPlan to Plan and PlanManifest
    let project_name = parsed_plan.project_name.clone();
    let total_iterations = parsed_plan.iterations.len() as u32;
    let total_tasks: usize = parsed_plan.iterations.iter().map(|i| i.tasks.len()).sum();

    let mut plan = Plan::new(
        requirement_id,
        project_name.clone(),
        folder_name.clone(),
        "backlog".to_string(),
    );
    plan.total_iterations = total_iterations;
    plan.total_tasks = total_tasks as u32;

    let mut manifest = PlanManifest::new(requirement_id, project_name.clone());
    for parsed_iter in &parsed_plan.iterations {
        let mut iteration = Iteration::new(parsed_iter.number, parsed_iter.name.clone());
        iteration.description = parsed_iter.description.clone();
        iteration.goal = parsed_iter.goal.clone();

        for parsed_task in &parsed_iter.tasks {
            let task = PlanTask {
                id: format!("I{}.T{}", parsed_iter.number, parsed_task.number),
                number: parsed_task.number,
                title: parsed_task.title.clone(),
                description: parsed_task.description.clone(),
                completed: false,
                agent_id: parsed_task.agent_id.clone(),
                dependencies: parsed_task.dependencies.clone(),
                acceptance_criteria: parsed_task.acceptance_criteria.clone(),
                metadata: std::collections::HashMap::new(),
            };
            iteration.add_task(task);
        }

        manifest.add_iteration(iteration);
    }

    println!("  ✓ Generated {} tasks", total_tasks);

    // Save plan files
    let plan_json_path = plan_dir.join("plan.json");
    let plan_json = serde_json::to_string_pretty(&plan).context("Failed to serialize plan")?;
    fs::write(&plan_json_path, plan_json).context("Failed to write plan.json")?;

    let manifest_path = plan_dir.join("plan").join("plan_manifest.json");
    let manifest_json =
        serde_json::to_string_pretty(&manifest).context("Failed to serialize manifest")?;
    fs::write(&manifest_path, manifest_json).context("Failed to write plan_manifest.json")?;

    generate_plan_files(&plan_dir, &parsed_plan).context("Failed to generate markdown files")?;

    println!("  ✓ Plan saved to {}", plan_dir.display().to_string().cyan());
    println!();

    // Step 5: Execute plan with YOLO mode
    println!("{}", "Step 4: Executing plan (YOLO mode)...".bold());
    println!();

    // Load context files
    let workspace_root = workspace.root().to_path_buf();
    let loader = ContextFileLoader::new(&workspace_root);
    let context_files = loader.load_hierarchical(&plan_dir).unwrap_or_default();

    // Generate session ID
    let session_id = Uuid::new_v4().to_string();
    let session_start_time = Utc::now();
    let mut executed_agent_ids = Vec::new();
    
    // Open monitoring service
    let monitoring_path = workspace.radium_dir().join("monitoring.db");
    let monitoring = MonitoringService::open(&monitoring_path).ok();

    // Execute plan with YOLO mode (continuous execution)
    execute_plan_yolo(
        &mut manifest,
        &manifest_path,
        if context_files.is_empty() { None } else { Some(context_files) },
        &mut executed_agent_ids,
        &session_id,
        monitoring.as_ref(),
    )
    .await?;

    println!();
    println!("{}", "Complete workflow finished!".green().bold());
    println!();

    Ok(())
}

/// Creates the plan directory structure.
fn create_plan_structure(plan_dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(plan_dir)?;
    fs::create_dir_all(plan_dir.join("plan"))?;
    fs::create_dir_all(plan_dir.join("artifacts").join("architecture"))?;
    fs::create_dir_all(plan_dir.join("artifacts").join("tasks"))?;
    fs::create_dir_all(plan_dir.join("memory"))?;
    fs::create_dir_all(plan_dir.join("prompts"))?;
    Ok(())
}

/// Extracts project name from specification content.
fn extract_project_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return Some(trimmed[2..].trim().to_string());
        }
    }
    None
}

/// Converts a string to a URL-friendly slug.
fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

/// Execute plan in YOLO mode (continuous execution until all tasks complete).
async fn execute_plan_yolo(
    manifest: &mut PlanManifest,
    manifest_path: &std::path::Path,
    context_files: Option<String>,
    executed_agent_ids: &mut Vec<String>,
    session_id: &str,
    monitoring: Option<&MonitoringService>,
) -> anyhow::Result<()> {
    // Create executor with YOLO mode (continuous execution)
    let config = ExecutionConfig {
        resume: false,
        skip_completed: true,
        check_dependencies: true,
        state_path: manifest_path.to_path_buf(),
        context_files,
        run_mode: RunMode::Continuous,
        context_manager: None,
        memory_store: None,
        requirement_id: None,
    };
    let executor = PlanExecutor::with_config(config);

    // Get all iteration IDs
    let iteration_ids: Vec<String> = manifest.iterations.iter().map(|i| i.id.clone()).collect();

    if iteration_ids.is_empty() {
        bail!("No iterations found to execute");
    }

    // Execution loop
    let mut execution_iteration = 0;
    const CONTINUOUS_SANITY_LIMIT: usize = 1000;
    let abort_requested = Arc::new(AtomicBool::new(false));
    let start_time = Instant::now();

    // Register SIGINT handler
    let abort_flag = abort_requested.clone();
    tokio::spawn(async move {
        if let Ok(()) = signal::ctrl_c().await {
            abort_flag.store(true, Ordering::Relaxed);
        }
    });

    loop {
        execution_iteration += 1;
        
        // Print progress
        let elapsed = start_time.elapsed();
        executor.print_progress(manifest, execution_iteration, elapsed, None);

        // Check sanity limit
        if execution_iteration > CONTINUOUS_SANITY_LIMIT {
            println!("  {} Reached sanity limit ({}). Stopping execution.", "→".yellow(), CONTINUOUS_SANITY_LIMIT);
            break;
        }

        // Check if all tasks are complete
        if !executor.has_incomplete_tasks(manifest) {
            println!("  {} All tasks completed. Execution finished.", "✓".green());
            break;
        }

        // Check abort flag
        if abort_requested.load(Ordering::Relaxed) {
            println!("\n{}", "Execution aborted by user. Progress saved to plan_manifest.json".yellow());
            executor.save_manifest(manifest, manifest_path)?;
            std::process::exit(130);
        }

        // Execute each iteration
        for iter_id in &iteration_ids {
            let iteration = manifest
                .get_iteration(&iter_id)
                .ok_or_else(|| anyhow::anyhow!("Iteration not found: {}", iter_id))?;

            // Skip completed iterations
            if iteration.status == PlanStatus::Completed {
                continue;
            }

            println!("{}", format!("Iteration {}", iter_id).bold().cyan());
            println!("  Goal: {}", iteration.goal.as_ref().unwrap_or(&"No goal specified".to_string()));
            println!();

            // Get all task IDs
            let task_ids: Vec<String> = iteration.tasks.iter().map(|t| t.id.clone()).collect();

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

                // Skip completed tasks
                if task.completed {
                    continue;
                }

                println!("    {} {}", "→".cyan(), task.title);
                
                let elapsed = start_time.elapsed();
                executor.print_progress(manifest, execution_iteration, elapsed, Some(&task.title));

                // Check dependencies
                if let Err(e) = executor.check_dependencies(manifest, task) {
                    println!("      {} Dependency not met: {}", "✗".red(), e.to_string().red());
                    println!("      {} Skipping task", "→".yellow());
                    continue;
                }

                // Get agent
                let agent_id = if let Some(id) = &task.agent_id {
                    id
                } else {
                    println!("      {} No agent assigned, skipping", "!".yellow());
                    continue;
                };

                println!("      {} Agent: {}", "•".dimmed(), agent_id.cyan());
                println!("      {} Executing...", "•".cyan());

                // Register agent in monitoring
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

                executed_agent_ids.push(tracked_agent_id.clone());

                // Create model instance (use mock for now, can be configured)
                let engine = std::env::var("RADIUM_ENGINE").unwrap_or_else(|_| "mock".to_string());
                let model_id = std::env::var("RADIUM_MODEL").unwrap_or_else(|_| "default".to_string());
                
                let model = match ModelFactory::create_from_str(&engine, model_id) {
                    Ok(m) => m,
                    Err(e) => {
                        println!("      {} Failed to create model: {}", "✗".red(), e.to_string().red());
                        continue;
                    }
                };

                // Execute task
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
                                if !response.is_empty() {
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
                            }

                            if let Some((prompt, completion)) = result.tokens_used {
                                println!(
                                    "      {} Tokens: {} prompt, {} completion",
                                    "•".dimmed(),
                                    prompt.to_string().dimmed(),
                                    completion.to_string().dimmed()
                                );
                                
                                // Record telemetry
                                if let Some(monitoring) = monitoring {
                                    use radium_core::monitoring::{TelemetryRecord, TelemetryTracking};
                                    let mut telemetry = TelemetryRecord::new(tracked_agent_id.clone())
                                        .with_tokens(prompt as u64, completion as u64);
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
                            executor.save_manifest(manifest, manifest_path)?;

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
                            // In YOLO mode, continue with other tasks even if one fails
                            println!("      {} Continuing with other tasks...", "→".yellow());
                        }
                    }
                    Err(e) => {
                        println!("      {} Execution error: {}", "✗".red(), e.to_string().red());
                        // In YOLO mode, continue with other tasks
                        println!("      {} Continuing with other tasks...", "→".yellow());
                    }
                }

                println!();
            }

            println!();
        }

        // Save manifest after each cycle
        executor.save_manifest(manifest, manifest_path)?;

        // Check abort flag again
        if abort_requested.load(Ordering::Relaxed) {
            println!("\n{}", "Execution aborted by user. Progress saved to plan_manifest.json".yellow());
            executor.save_manifest(manifest, manifest_path)?;
            std::process::exit(130);
        }
    }

    Ok(())
}


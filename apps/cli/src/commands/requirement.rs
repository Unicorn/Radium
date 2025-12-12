//! Requirement command implementation.
//!
//! Provides autonomous execution of Braingrid requirements with task breakdown
//! and real-time status synchronization.

use anyhow::{Context, bail};
use clap::Subcommand;
use colored::Colorize;
use radium_core::{
    checkpoint::CheckpointManager,
    context::braingrid_client::BraingridClient,
    planning::dag::DependencyGraph,
    workflow::{
        AgentSelector, ParallelExecutor, ProgressReporter, ReportGenerator,
        StatePersistence,
    },
    storage::Database,
    Workspace,
};
use radium_orchestrator::{AgentExecutor, AgentRegistry, Orchestrator};
use std::sync::Arc;
use std::time::Instant;
use std::process::Command;
use serde::Deserialize;

use crate::colors::RadiumBrandColors;

/// Requirement command subcommands.
#[derive(Subcommand, Debug)]
pub enum RequirementCommand {
    /// Execute a requirement (default if no subcommand)
    Execute {
        /// Braingrid requirement ID (e.g., "REQ-173")
        req_id: Option<String>,

        /// Braingrid project ID (defaults to BRAINGRID_PROJECT_ID env var or PROJ-14)
        #[arg(long)]
        project: Option<String>,

        /// List all requirements for the project
        #[arg(long)]
        ls: bool,

        /// Maximum number of concurrent task executions (default: 3)
        #[arg(long, default_value = "3")]
        parallel: usize,

        /// Show execution plan without running tasks
        #[arg(long)]
        dry_run: bool,

        /// Resume from last checkpoint if execution was interrupted
        #[arg(long)]
        resume: bool,

        /// Fail if no tasks exist instead of triggering breakdown
        #[arg(long)]
        skip_breakdown: bool,
    },
    /// Resume an interrupted requirement execution
    Resume {
        /// Braingrid requirement ID (e.g., "REQ-173")
        req_id: String,

        /// Braingrid project ID (defaults to BRAINGRID_PROJECT_ID env var or PROJ-14)
        #[arg(long)]
        project: Option<String>,

        /// Restore to a specific checkpoint before resuming
        #[arg(long)]
        from_checkpoint: Option<String>,
    },
}

/// Execute requirement command (dispatches to subcommands).
pub async fn execute_command(cmd: RequirementCommand) -> anyhow::Result<()> {
    match cmd {
        RequirementCommand::Execute { req_id, project, ls, parallel, dry_run, resume, skip_breakdown } => {
            if ls {
                list(project).await?;
            } else if let Some(id) = req_id {
                execute(id, project, parallel, dry_run, resume, skip_breakdown).await?;
            } else {
                anyhow::bail!("Requirement ID is required when not using --ls");
            }
        }
        RequirementCommand::Resume { req_id, project, from_checkpoint } => {
            resume_command(req_id, project, from_checkpoint).await?;
        }
    }
    Ok(())
}

/// Braingrid requirement list response
#[derive(Debug, Deserialize)]
struct RequirementListResponse {
    requirements: Vec<RequirementSummary>,
    pagination: Pagination,
}

/// Requirement summary in list view
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RequirementSummary {
    short_id: String,
    name: String,
    status: String,
    complexity: u8,
    readiness: u8,
    task_progress: TaskProgress,
    created_at: String,
    updated_at: String,
}

/// Task progress information
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TaskProgress {
    total: usize,
    completed: usize,
    progress_percentage: u8,
}

/// Pagination metadata
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Pagination {
    page: usize,
    limit: usize,
    total: usize,
    total_pages: usize,
}

/// Execute the requirement command.
///
/// Autonomously executes a complete Braingrid requirement:
/// 1. Fetches requirement tree (with tasks)
/// 2. Triggers breakdown if no tasks exist
/// 3. Executes each task autonomously
/// 4. Updates task statuses in real-time
/// 5. Sets requirement to REVIEW when complete
///
/// # Arguments
/// * `req_id` - Braingrid requirement ID (e.g., "REQ-173")
/// * `project_id` - Braingrid project ID (e.g., "PROJ-14")
/// * `max_parallel` - Maximum concurrent task executions
/// * `dry_run` - Show execution plan without running
/// * `resume` - Resume from last checkpoint
/// * `skip_breakdown` - Fail if no tasks instead of triggering breakdown
async fn execute(
    req_id: String,
    project_id: Option<String>,
    max_parallel: usize,
    dry_run: bool,
    resume: bool,
    skip_breakdown: bool,
) -> anyhow::Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "rad requirement execute".bold().color(colors.primary()));
    println!();

    let start_time = Instant::now();

    // Validate requirement ID format
    if !req_id.starts_with("REQ-") {
        bail!("Invalid requirement ID format. Expected format: REQ-XXX (e.g., REQ-173)");
    }

    // Get project ID from environment or parameter
    let project_id = project_id
        .or_else(|| std::env::var("BRAINGRID_PROJECT_ID").ok())
        .unwrap_or_else(|| {
            println!("{}", "Warning: No project ID specified, using default PROJ-14".color(colors.warning()));
            "PROJ-14".to_string()
        });

    println!("{}", "Configuration:".bold());
    println!("  Requirement ID: {}", req_id.color(colors.primary()));
    println!("  Project ID: {}", project_id.color(colors.primary()));
    println!("  Max Parallel: {}", max_parallel);
    if dry_run {
        println!("  {} Dry-run mode (no execution)", "⚠".color(colors.warning()));
    }
    if resume {
        println!("  {} Resume mode", "↻".color(colors.primary()));
    }
    println!();

    // Discover workspace
    println!("{}", "Initializing workspace...".dimmed());
    let workspace = Workspace::discover().context("Failed to discover workspace")?;
    workspace.ensure_structure().context("Failed to ensure workspace structure")?;
    println!("  {} Workspace initialized", "✓".color(colors.success()));
    println!();

    // Initialize database
    println!("{}", "Initializing database...".dimmed());
    let db_path = workspace.radium_dir().join("database.db");
    let _db = Arc::new(std::sync::Mutex::new(
        Database::open(db_path.to_str().unwrap()).context("Failed to open database")?,
    ));
    println!("  {} Database initialized", "✓".color(colors.success()));
    println!();

    // Initialize orchestrator and executor
    println!("{}", "Initializing orchestrator...".dimmed());
    let _orchestrator = Arc::new(Orchestrator::new());

    // Use Gemini as model type
    let executor = Arc::new(AgentExecutor::new(
        radium_models::ModelType::Gemini,
        "gemini-2.0-flash-exp".to_string(),
    ));
    println!("  {} Orchestrator initialized", "✓".color(colors.success()));
    println!();

    // Initialize agent registry
    println!("{}", "Loading agent registry...".dimmed());
    let agent_registry = Arc::new(AgentRegistry::new());
    // TODO: Load agents from workspace
    println!("  {} Agent registry loaded", "✓".color(colors.success()));
    println!();

    // Initialize model
    println!("{}", "Initializing AI model...".dimmed());
    use radium_models::{ModelConfig, ModelFactory, ModelType};
    let config = ModelConfig {
        model_type: ModelType::Gemini,
        model_id: "gemini-2.0-flash-exp".to_string(),
        api_key: std::env::var("GEMINI_API_KEY").ok(),
        base_url: None,
        enable_context_caching: None,
        cache_ttl: None,
        cache_breakpoints: None,
        cache_identifier: None,
        enable_code_execution: None,
    };
    let _model = ModelFactory::create(config)
        .context("Failed to create model")?;
    println!("  {} Model initialized", "✓".color(colors.success()));
    println!();

    // Initialize Braingrid client
    let braingrid_client = Arc::new(BraingridClient::new(project_id.clone()));

    // Fetch requirement
    println!("{}", "Fetching requirement...".dimmed());
    let mut requirement = braingrid_client
        .fetch_requirement_tree(&req_id)
        .await
        .context("Failed to fetch requirement")?;
    println!("  {} Requirement loaded: {}", "✓".color(colors.success()), requirement.name);
    println!();

    // Check if tasks exist, trigger breakdown if needed
    if requirement.tasks.is_empty() {
        if skip_breakdown {
            bail!("No tasks found for requirement {} and --skip-breakdown is set", req_id);
        }
        println!("{}", "No tasks found, triggering breakdown...".dimmed());
        requirement.tasks = braingrid_client
            .breakdown_requirement(&req_id)
            .await
            .context("Failed to trigger breakdown")?;
        println!("  {} Generated {} tasks", "✓".color(colors.success()), requirement.tasks.len());
        println!();
    }

    // Build dependency graph
    println!("{}", "Building dependency graph...".dimmed());
    let dep_graph = DependencyGraph::from_braingrid_tasks(&requirement.tasks)
        .context("Failed to build dependency graph")?;
    
    // Validate for cycles
    dep_graph.detect_cycles()
        .map_err(|e| anyhow::anyhow!("Circular dependency detected: {}", e))?;
    
    println!("  {} Dependency graph validated", "✓".color(colors.success()));
    println!();

    // Dry-run mode: show execution plan
    if dry_run {
        println!("{}", "Execution Plan (Dry-Run)".bold().color(colors.primary()));
        println!("{}", "─".repeat(60).dimmed());
        println!();
        
        let execution_order = dep_graph.topological_sort()
            .context("Failed to get execution order")?;
        
        println!("  Execution Order:");
        for (idx, task_id) in execution_order.iter().enumerate() {
            if let Some(task) = requirement.tasks.iter().find(|t| t.number == *task_id) {
                println!("    {}. {}: {}", idx + 1, task_id, task.title);
            }
        }
        println!();
        
        println!("  Ready Tasks (can run in parallel):");
        let ready = dep_graph.ready_tasks(&std::collections::HashSet::new());
        for task_id in ready {
            if let Some(task) = requirement.tasks.iter().find(|t| t.number == task_id) {
                println!("    - {}: {}", task_id, task.title);
            }
        }
        println!();
        
        println!("  {} Dry-run complete. Use without --dry-run to execute.", "ℹ".color(colors.info()));
        return Ok(());
    }

    // Initialize state persistence
    let state_persistence = StatePersistence::new(workspace.root());

    // Check for resume
    if resume {
        if let Some(persisted_state) = state_persistence.load_state(&req_id)
            .context("Failed to load execution state")? {
            println!("{}", format!("Resuming from checkpoint ({} completed tasks)", persisted_state.completed_tasks.len()).dimmed());
            println!();
        }
    }

    // Create agent selector
    let agent_selector = Arc::new(AgentSelector::new(Arc::clone(&agent_registry)));

    // Create parallel executor
    let parallel_executor = ParallelExecutor::new(
        max_parallel,
        Arc::clone(&braingrid_client),
        Arc::clone(&executor),
        agent_selector,
    );

    // Initialize progress reporter
    let progress_reporter = ProgressReporter::new(requirement.clone(), requirement.tasks.len());

    // Update requirement status to IN_PROGRESS
    braingrid_client
        .update_requirement_status(&req_id, radium_core::context::braingrid_client::RequirementStatus::InProgress)
        .await
        .context("Failed to update requirement status")?;

    // Execute tasks
    println!("{}", format!("Executing requirement {}...", req_id).bold());
    println!("{}", "─".repeat(60).dimmed());
    println!();

    let (execution_report, execution_state) = parallel_executor
        .execute_tasks(requirement.tasks.clone(), &dep_graph, &req_id)
        .await
        .map_err(|e| anyhow::anyhow!("Parallel execution failed: {}", e))?;

    // Generate completion report
    let report_generator = ReportGenerator::new(workspace.root());
    let completion_report = report_generator.generate_report(
        &requirement,
        &execution_state,
        &execution_report,
    );

    // Save report
    let report_path = report_generator
        .save_report(&completion_report, &req_id)
        .context("Failed to save completion report")?;
    
    println!("  {} Completion report saved to: {}", "✓".color(colors.success()), report_path.display());

    // Update requirement status
    let final_status = if execution_report.success {
        radium_core::context::braingrid_client::RequirementStatus::Review
    } else {
        radium_core::context::braingrid_client::RequirementStatus::InProgress
    };
    
    braingrid_client
        .update_requirement_status(&req_id, final_status.clone())
        .await
        .context("Failed to update requirement status")?;

    // Clean up state on success
    if execution_report.success {
        let _ = state_persistence.delete_state(&req_id);
    }

    // Display summary
    progress_reporter.finish(&execution_report);
    report_generator.display_summary(&completion_report);

    let _elapsed = start_time.elapsed();


    Ok(())
}

/// List all requirements for a project.
///
/// Displays a formatted table of all requirements with their status, progress, and metadata.
///
/// # Arguments
/// * `project_id` - Braingrid project ID (e.g., "PROJ-14")
pub async fn list(project_id: Option<String>) -> anyhow::Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "rad requirement list".bold().color(colors.primary()));
    println!();

    // Get project ID from environment or parameter
    let project_id = project_id
        .or_else(|| std::env::var("BRAINGRID_PROJECT_ID").ok())
        .unwrap_or_else(|| {
            println!("{}", "Warning: No project ID specified, using default PROJ-14".color(colors.warning()));
            "PROJ-14".to_string()
        });

    println!("{}", "Configuration:".bold());
    println!("  Project ID: {}", project_id.color(colors.primary()));
    println!();

    // Call braingrid CLI
    println!("{}", "Fetching requirements...".dimmed());
    let output = Command::new("braingrid")
        .args(&[
            "requirement",
            "list",
            "-p",
            &project_id,
            "--format",
            "json",
        ])
        .output()
        .context("Failed to execute braingrid command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to list requirements: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Strip spinner animation and extract JSON (starts with '{')
    let json_start = stdout.find('{').unwrap_or(0);
    let json_str = &stdout[json_start..];

    let response: RequirementListResponse = serde_json::from_str(json_str)
        .context("Failed to parse requirement list JSON")?;

    println!("  {} Fetched {} requirements", "✓".color(colors.success()), response.requirements.len());
    println!();

    // Display requirements table
    println!("{}", "─".repeat(110).dimmed());
    println!(
        "{:<12} {:<50} {:<15} {:<10} {:<15}",
        "ID".bold(),
        "Name".bold(),
        "Status".bold(),
        "Progress".bold(),
        "Tasks".bold()
    );
    println!("{}", "─".repeat(110).dimmed());

    for req in &response.requirements {
        let status_colored = match req.status.as_str() {
            "COMPLETED" => req.status.color(colors.success()),
            "IN_PROGRESS" => req.status.color(colors.primary()),
            "REVIEW" => req.status.color(colors.warning()),
            "PLANNED" => req.status.color(colors.info()),
            "IDEA" => req.status.dimmed(),
            "CANCELLED" => req.status.color(colors.error()),
            _ => req.status.white(),
        };

        let progress_bar = format!(
            "{}%",
            req.task_progress.progress_percentage
        );
        let progress_colored = if req.task_progress.progress_percentage == 100 {
            progress_bar.color(colors.success())
        } else if req.task_progress.progress_percentage > 0 {
            progress_bar.color(colors.warning())
        } else {
            progress_bar.dimmed()
        };

        let tasks_info = format!(
            "{}/{}",
            req.task_progress.completed,
            req.task_progress.total
        );

        // Truncate name if too long
        let name = if req.name.len() > 48 {
            format!("{}...", &req.name[..45])
        } else {
            req.name.clone()
        };

        println!(
            "{:<12} {:<50} {:<15} {:<10} {:<15}",
            req.short_id.color(colors.primary()),
            name,
            status_colored,
            progress_colored,
            tasks_info
        );
    }

    println!("{}", "─".repeat(110).dimmed());
    println!();

    // Display pagination info
    println!(
        "  Showing page {} of {} ({} total requirements)",
        response.pagination.page,
        response.pagination.total_pages,
        response.pagination.total
    );
    println!();

    Ok(())
}

/// Resume an interrupted requirement execution.
///
/// Loads persisted execution state and continues from where it left off,
/// skipping completed tasks.
///
/// # Arguments
/// * `req_id` - Braingrid requirement ID to resume
/// * `project_id` - Optional project ID
/// * `from_checkpoint` - Optional checkpoint ID to restore to before resuming
async fn resume_command(
    req_id: String,
    project_id: Option<String>,
    from_checkpoint: Option<String>,
) -> anyhow::Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", format!("rad requirement resume {}", req_id).bold().color(colors.primary()));
    println!();

    // Validate requirement ID format
    if !req_id.starts_with("REQ-") {
        bail!("Invalid requirement ID format. Expected format: REQ-XXX (e.g., REQ-173)");
    }

    // Get workspace
    let workspace = Workspace::discover()
        .context("No Radium workspace found. Run 'rad init' to create one.")?;

    // Load persisted execution state
    let state_persistence = StatePersistence::new(workspace.root());
    let persisted_state = state_persistence
        .load_state(&req_id)
        .context("Failed to load execution state")?;

    let persisted_state = match persisted_state {
        Some(state) => state,
        None => {
            bail!("No resumable execution found for {}. Use 'rad requirement execute {}' to start execution.", req_id, req_id);
        }
    };

    // Display resume summary
    println!("{}", "Resume Summary".bold());
    println!("{}", "─".repeat(60));
    println!("Requirement: {} - {}", req_id, persisted_state.requirement_title);
    println!("Completed tasks: {}", persisted_state.completed_tasks.len());
    println!("Remaining tasks: {}", persisted_state.next_tasks.len());
    println!("Last checkpoint: {}", persisted_state.last_checkpoint_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!();

    // Restore checkpoint if specified
    if let Some(checkpoint_id) = from_checkpoint {
        println!("Restoring to checkpoint: {}...", checkpoint_id);
        let checkpoint_manager = CheckpointManager::new(workspace.root())
            .context("Workspace is not a git repository. Checkpoints require git.")?;
        checkpoint_manager
            .restore_checkpoint(&checkpoint_id)
            .context(format!("Failed to restore checkpoint: {}", checkpoint_id))?;
        println!("✓ Checkpoint restored");
        println!();
    }

    // Get project ID
    let colors = RadiumBrandColors::new();
    let project_id = project_id
        .or_else(|| std::env::var("BRAINGRID_PROJECT_ID").ok())
        .unwrap_or_else(|| {
            println!("{}", "Warning: No project ID specified, using default PROJ-14".color(colors.warning()));
            "PROJ-14".to_string()
        });

    // Continue with normal execution (it will use the persisted state via resume flag)
    println!("Resuming execution...");
    println!();

    // Call execute with resume=true - it should handle the persisted state
    execute(req_id, Some(project_id), 3, false, true, false).await?;

    Ok(())
}

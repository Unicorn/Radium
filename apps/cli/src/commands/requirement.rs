//! Requirement command implementation.
//!
//! Provides autonomous execution of Braingrid requirements with task breakdown
//! and real-time status synchronization.

use anyhow::{Context, bail};
use colored::Colorize;
use radium_core::{
    workflow::{RequirementExecutor, RequirementExecutionError},
    agents::registry::AgentRegistry,
    storage::Database,
    Workspace,
};
use radium_orchestrator::{AgentExecutor, Orchestrator};
use std::sync::Arc;
use std::time::Instant;
use std::process::Command;
use serde::Deserialize;

/// Braingrid requirement list response
#[derive(Debug, Deserialize)]
struct RequirementListResponse {
    requirements: Vec<RequirementSummary>,
    pagination: Pagination,
}

/// Requirement summary in list view
#[derive(Debug, Deserialize)]
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
struct TaskProgress {
    total: usize,
    completed: usize,
    progress_percentage: u8,
}

/// Pagination metadata
#[derive(Debug, Deserialize)]
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
pub async fn execute(req_id: String, project_id: Option<String>) -> anyhow::Result<()> {
    println!("{}", "rad requirement execute".bold().cyan());
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
            println!("{}", "Warning: No project ID specified, using default PROJ-14".yellow());
            "PROJ-14".to_string()
        });

    println!("{}", "Configuration:".bold());
    println!("  Requirement ID: {}", req_id.cyan());
    println!("  Project ID: {}", project_id.cyan());
    println!();

    // Discover workspace
    println!("{}", "Initializing workspace...".dimmed());
    let workspace = Workspace::discover().context("Failed to discover workspace")?;
    workspace.ensure_structure().context("Failed to ensure workspace structure")?;
    println!("  {} Workspace initialized", "✓".green());
    println!();

    // Initialize database
    println!("{}", "Initializing database...".dimmed());
    let db_path = workspace.radium_dir().join("database.db");
    let db = Arc::new(std::sync::Mutex::new(
        Database::open(db_path.to_str().unwrap()).context("Failed to open database")?,
    ));
    println!("  {} Database initialized", "✓".green());
    println!();

    // Initialize orchestrator and executor
    println!("{}", "Initializing orchestrator...".dimmed());
    let orchestrator = Arc::new(Orchestrator::new());

    // Use Gemini as model type
    let executor = Arc::new(AgentExecutor::new(
        radium_models::ModelType::Gemini,
        "gemini-2.0-flash-exp".to_string(),
    ));
    println!("  {} Orchestrator initialized", "✓".green());
    println!();

    // Initialize agent registry
    println!("{}", "Loading agent registry...".dimmed());
    let agent_registry = Arc::new(AgentRegistry::new());
    // TODO: Load agents from workspace
    println!("  {} Agent registry loaded", "✓".green());
    println!();

    // Initialize model
    println!("{}", "Initializing AI model...".dimmed());
    use radium_models::{ModelConfig, ModelFactory, ModelType};
    let config = ModelConfig {
        model_type: ModelType::Gemini,
        model_id: "gemini-2.0-flash-exp".to_string(),
        api_key: std::env::var("GEMINI_API_KEY").ok(),
    };
    let model = ModelFactory::create(config)
        .context("Failed to create model")?;
    println!("  {} Model initialized", "✓".green());
    println!();

    // Create requirement executor
    println!("{}", "Creating requirement executor...".dimmed());
    let executor_instance = RequirementExecutor::new(
        project_id.clone(),
        &orchestrator,
        &executor,
        &db,
        agent_registry,
        model,
    )
    .context("Failed to create requirement executor")?;
    println!("  {} Executor created", "✓".green());
    println!();

    // Execute requirement
    println!("{}", format!("Executing requirement {}...", req_id).bold());
    println!("{}", "─".repeat(60).dimmed());
    println!();

    let result = executor_instance
        .execute_requirement(&req_id)
        .await
        .map_err(|e| match e {
            RequirementExecutionError::Braingrid(err) => {
                if err.to_string().contains("not found") {
                    anyhow::anyhow!("Requirement {} not found in project {}", req_id, project_id)
                } else {
                    anyhow::anyhow!("Braingrid error: {}", err)
                }
            }
            RequirementExecutionError::NoTasks(req) => {
                anyhow::anyhow!("No tasks available for requirement {} after breakdown", req)
            }
            RequirementExecutionError::Autonomous(err) => {
                anyhow::anyhow!("Autonomous execution error: {}", err)
            }
            RequirementExecutionError::TaskFailed(task_id, err) => {
                anyhow::anyhow!("Task {} failed: {}", task_id, err)
            }
            RequirementExecutionError::Configuration(err) => {
                anyhow::anyhow!("Configuration error: {}", err)
            }
        })?;

    let elapsed = start_time.elapsed();

    // Display results
    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!("{}", "Execution Summary".bold().cyan());
    println!("{}", "─".repeat(60).dimmed());
    println!();

    println!("  Requirement: {}", result.requirement_id.cyan());
    println!("  Tasks Completed: {}", result.tasks_completed.to_string().green());
    println!("  Tasks Failed: {}", result.tasks_failed.to_string().red());
    println!("  Execution Time: {}s", elapsed.as_secs().to_string().cyan());
    println!("  Final Status: {:?}", result.final_status);
    println!();

    if result.success {
        println!(
            "  {} Requirement execution completed successfully!",
            "✓".green()
        );
        println!(
            "  {} Requirement status set to REVIEW",
            "→".cyan()
        );
    } else {
        println!(
            "  {} Requirement execution completed with {} failed tasks",
            "⚠".yellow(),
            result.tasks_failed
        );
        println!(
            "  {} Review failed tasks in Braingrid",
            "→".yellow()
        );
    }

    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!();

    Ok(())
}

/// List all requirements for a project.
///
/// Displays a formatted table of all requirements with their status, progress, and metadata.
///
/// # Arguments
/// * `project_id` - Braingrid project ID (e.g., "PROJ-14")
pub async fn list(project_id: Option<String>) -> anyhow::Result<()> {
    println!("{}", "rad requirement list".bold().cyan());
    println!();

    // Get project ID from environment or parameter
    let project_id = project_id
        .or_else(|| std::env::var("BRAINGRID_PROJECT_ID").ok())
        .unwrap_or_else(|| {
            println!("{}", "Warning: No project ID specified, using default PROJ-14".yellow());
            "PROJ-14".to_string()
        });

    println!("{}", "Configuration:".bold());
    println!("  Project ID: {}", project_id.cyan());
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

    println!("  {} Fetched {} requirements", "✓".green(), response.requirements.len());
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
            "COMPLETED" => req.status.green(),
            "IN_PROGRESS" => req.status.cyan(),
            "REVIEW" => req.status.yellow(),
            "PLANNED" => req.status.blue(),
            "IDEA" => req.status.dimmed(),
            "CANCELLED" => req.status.red(),
            _ => req.status.white(),
        };

        let progress_bar = format!(
            "{}%",
            req.task_progress.progress_percentage
        );
        let progress_colored = if req.task_progress.progress_percentage == 100 {
            progress_bar.green()
        } else if req.task_progress.progress_percentage > 0 {
            progress_bar.yellow()
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
            req.short_id.cyan(),
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

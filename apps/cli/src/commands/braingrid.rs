//! Braingrid command implementation.
//!
//! Provides CLI commands for interacting with Braingrid requirements and tasks.

use anyhow::{Context, Result};
use colored::Colorize;
use radium_core::context::braingrid_client::{
    BraingridClient, BraingridError, RequirementStatus, TaskStatus,
};
use std::path::PathBuf;

use crate::colors::RadiumBrandColors;

/// Create a new requirement (Braingrid `specify`)
pub async fn specify(text: Vec<String>, file: Option<PathBuf>, project_id: Option<String>) -> Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "rad braingrid specify".bold().color(colors.primary()));
    println!();

    let project_id = get_project_id(project_id)?;

    let spec_text = if let Some(path) = file {
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        contents
    } else {
        let joined = text.join(" ").trim().to_string();
        if joined.is_empty() {
            anyhow::bail!("No specification text provided. Pass text args or use --file <path>.");
        }
        joined
    };

    println!("{}", "Configuration:".bold());
    println!("  Project ID: {}", project_id.color(colors.primary()));
    println!("  Input: {}", if spec_text.len() > 80 { "text (truncated)".dimmed() } else { spec_text.clone().color(colors.primary()) });
    println!();

    println!("{}", "Creating requirement via Braingrid...".dimmed());
    let client = BraingridClient::new(project_id);
    let created = client
        .specify_requirement(&spec_text)
        .await
        .map_err(|e| map_braingrid_error(e, "specify"))?;

    println!("  {} Created requirement {}", "✓".color(colors.success()), created.color(colors.primary()));
    println!();
    println!("Next steps:");
    println!("  - View:     {}", format!("rad braingrid read {}", created).dimmed());
    println!("  - Breakdown:{}", format!("rad braingrid breakdown {}", created).dimmed());
    println!("  - Execute:  {}", format!("rad requirement execute {}", created).dimmed());
    println!();

    Ok(())
}

/// Read a requirement with all tasks
pub async fn read(req_id: String, project_id: Option<String>) -> Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "rad braingrid read".bold().color(colors.primary()));
    println!();

    let project_id = get_project_id(project_id)?;

    println!("{}", "Configuration:".bold());
    println!("  Requirement ID: {}", req_id.color(colors.primary()));
    println!("  Project ID: {}", project_id.color(colors.primary()));
    println!();

    println!("{}", "Fetching requirement...".dimmed());
    let client = BraingridClient::new(project_id);
    let requirement = client
        .fetch_requirement_tree(&req_id)
        .await
        .map_err(|e| map_braingrid_error(e, &req_id))?;

    println!("  {} Requirement loaded", "✓".color(colors.success()));
    println!();

    // Display requirement details
    println!("{}", "─".repeat(80).dimmed());
    println!("{}", format!("Requirement: {}", requirement.id).bold().color(colors.primary()));
    println!("{}", "─".repeat(80).dimmed());
    println!();
    println!("  Name: {}", requirement.name);
    println!("  Status: {:?}", requirement.status);
    if let Some(assigned_to) = &requirement.assigned_to {
        println!("  Assigned to: {}", assigned_to);
    }
    println!("  Tasks: {}", requirement.tasks.len());
    println!();

    if !requirement.tasks.is_empty() {
        println!("{}", "Tasks:".bold());
        println!("{}", "─".repeat(80).dimmed());
        for (idx, task) in requirement.tasks.iter().enumerate() {
            let status_color = match task.status {
                TaskStatus::Completed => "✓".color(colors.success()),
                TaskStatus::InProgress => "→".color(colors.primary()),
                TaskStatus::Planned => "○".color(colors.warning()),
                TaskStatus::Cancelled => "✗".color(colors.error()),
            };
            println!(
                "  {} {} {} - {}",
                status_color,
                task.task_id().color(colors.primary()),
                format!("({})", task.number).dimmed(),
                task.title
            );
            if let Some(description) = &task.description {
                let desc = if description.len() > 60 {
                    format!("{}...", &description[..57])
                } else {
                    description.clone()
                };
                println!("      {}", desc.dimmed());
            }
        }
        println!("{}", "─".repeat(80).dimmed());
    } else {
        println!("  {} No tasks found", "○".dimmed());
    }

    println!();
    Ok(())
}

/// List all tasks for a requirement
pub async fn tasks(req_id: String, project_id: Option<String>) -> Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "rad braingrid tasks".bold().color(colors.primary()));
    println!();

    let project_id = get_project_id(project_id)?;

    println!("{}", "Configuration:".bold());
    println!("  Requirement ID: {}", req_id.color(colors.primary()));
    println!("  Project ID: {}", project_id.color(colors.primary()));
    println!();

    println!("{}", "Fetching tasks...".dimmed());
    let client = BraingridClient::new(project_id);
    let tasks = client
        .list_tasks(&req_id)
        .await
        .map_err(|e| map_braingrid_error(e, &req_id))?;

    println!("  {} Found {} tasks", "✓".color(colors.success()), tasks.len());
    println!();

    if tasks.is_empty() {
        println!("  {} No tasks found for requirement {}", "○".dimmed(), req_id.color(colors.primary()));
        println!();
        return Ok(());
    }

    // Display tasks table
    println!("{}", "─".repeat(100).dimmed());
    println!(
        "{:<15} {:<10} {:<50} {:<15}",
        "Task ID".bold(),
        "Number".bold(),
        "Title".bold(),
        "Status".bold()
    );
    println!("{}", "─".repeat(100).dimmed());

    for task in &tasks {
        let status_str = format!("{:?}", task.status);
        let status_colored = match task.status {
            TaskStatus::Completed => status_str.color(colors.success()),
            TaskStatus::InProgress => status_str.color(colors.primary()),
            TaskStatus::Planned => status_str.color(colors.warning()),
            TaskStatus::Cancelled => status_str.color(colors.error()),
        };

        let title = if task.title.len() > 48 {
            format!("{}...", &task.title[..45])
        } else {
            task.title.clone()
        };

        println!(
            "{:<15} {:<10} {:<50} {:<15}",
            task.task_id().color(colors.primary()),
            task.number.dimmed(),
            title,
            status_colored
        );
    }

    println!("{}", "─".repeat(100).dimmed());
    println!();
    Ok(())
}

/// Update task status
pub async fn update_task(
    task_id: String,
    req_id: String,
    status_str: String,
    project_id: Option<String>,
) -> Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "rad braingrid update-task".bold().color(colors.primary()));
    println!();

    let project_id = get_project_id(project_id)?;

    let status = parse_task_status(&status_str)?;

    println!("{}", "Configuration:".bold());
    println!("  Task ID: {}", task_id.color(colors.primary()));
    println!("  Requirement ID: {}", req_id.color(colors.primary()));
    println!("  Status: {:?}", status);
    println!("  Project ID: {}", project_id.color(colors.primary()));
    println!();

    println!("{}", "Updating task status...".dimmed());
    let client = BraingridClient::new(project_id);
    client
        .update_task_status(&task_id, &req_id, status, None)
        .await
        .map_err(|e| map_braingrid_error(e, &task_id))?;

    println!("  {} Task status updated successfully", "✓".color(colors.success()));
    println!();
    Ok(())
}

/// Update requirement status
pub async fn update_req(
    req_id: String,
    status_str: String,
    project_id: Option<String>,
) -> Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "rad braingrid update-req".bold().color(colors.primary()));
    println!();

    let project_id = get_project_id(project_id)?;

    let status = parse_requirement_status(&status_str)?;

    println!("{}", "Configuration:".bold());
    println!("  Requirement ID: {}", req_id.color(colors.primary()));
    println!("  Status: {:?}", status);
    println!("  Project ID: {}", project_id.color(colors.primary()));
    println!();

    println!("{}", "Updating requirement status...".dimmed());
    let client = BraingridClient::new(project_id);
    client
        .update_requirement_status(&req_id, status)
        .await
        .map_err(|e| map_braingrid_error(e, &req_id))?;

    println!("  {} Requirement status updated successfully", "✓".color(colors.success()));
    println!();
    Ok(())
}

/// Trigger requirement breakdown (create tasks)
pub async fn breakdown(req_id: String, project_id: Option<String>) -> Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "rad braingrid breakdown".bold().color(colors.primary()));
    println!();

    let project_id = get_project_id(project_id)?;

    println!("{}", "Configuration:".bold());
    println!("  Requirement ID: {}", req_id.color(colors.primary()));
    println!("  Project ID: {}", project_id.color(colors.primary()));
    println!();

    println!("{}", "Triggering requirement breakdown...".dimmed());
    let client = BraingridClient::new(project_id);
    let tasks = client
        .breakdown_requirement(&req_id)
        .await
        .map_err(|e| map_braingrid_error(e, &req_id))?;

    println!("  {} Breakdown completed", "✓".color(colors.success()));
    println!("  {} Created {} tasks", "→".color(colors.primary()), tasks.len());
    println!();

    if !tasks.is_empty() {
        println!("{}", "Created Tasks:".bold());
        for task in &tasks {
            println!("  {} {}", "•".color(colors.primary()), task.task_id().color(colors.primary()));
            println!("      {}", task.title);
        }
        println!();
    }

    Ok(())
}

/// Update requirement with an action (Braingrid `requirement update --action ...`)
pub async fn action(req_id: String, action: Vec<String>, project_id: Option<String>) -> Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "rad braingrid action".bold().color(colors.primary()));
    println!();

    let project_id = get_project_id(project_id)?;
    let action_text = action.join(" ").trim().to_string();
    if action_text.is_empty() {
        anyhow::bail!("Action text is required.");
    }

    println!("{}", "Configuration:".bold());
    println!("  Requirement ID: {}", req_id.color(colors.primary()));
    println!("  Project ID: {}", project_id.color(colors.primary()));
    println!("  Action: {}", action_text.color(colors.primary()));
    println!();

    println!("{}", "Updating requirement...".dimmed());
    let client = BraingridClient::new(project_id);
    client
        .update_requirement_action(&req_id, &action_text)
        .await
        .map_err(|e| map_braingrid_error(e, &req_id))?;

    println!("  {} Requirement updated successfully", "✓".color(colors.success()));
    println!();
    Ok(())
}

/// Ensure tasks exist for a requirement (trigger breakdown only if empty)
pub async fn ensure_tasks(req_id: String, project_id: Option<String>) -> Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "rad braingrid ensure-tasks".bold().color(colors.primary()));
    println!();

    let project_id = get_project_id(project_id)?;

    println!("{}", "Configuration:".bold());
    println!("  Requirement ID: {}", req_id.color(colors.primary()));
    println!("  Project ID: {}", project_id.color(colors.primary()));
    println!();

    println!("{}", "Fetching requirement...".dimmed());
    let client = BraingridClient::new(project_id);
    let requirement = client
        .fetch_requirement_tree(&req_id)
        .await
        .map_err(|e| map_braingrid_error(e, &req_id))?;

    if !requirement.tasks.is_empty() {
        println!("  {} Tasks already exist ({} tasks). No action needed.", "✓".color(colors.success()), requirement.tasks.len());
        println!();
        return Ok(());
    }

    println!("{}", "No tasks found; triggering breakdown...".dimmed());
    let tasks = client
        .breakdown_requirement(&req_id)
        .await
        .map_err(|e| map_braingrid_error(e, &req_id))?;

    println!("  {} Created {} tasks", "✓".color(colors.success()), tasks.len());
    println!();
    Ok(())
}

/// Clear Braingrid cache
pub async fn cache_clear(project_id: Option<String>) -> Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "rad braingrid cache clear".bold().color(colors.primary()));
    println!();

    let project_id = get_project_id(project_id)?;

    println!("{}", "Configuration:".bold());
    println!("  Project ID: {}", project_id.color(colors.primary()));
    println!();

    println!("{}", "Clearing cache...".dimmed());
    let client = BraingridClient::new(project_id);
    client.clear_cache().await;

    println!("  {} Cache cleared", "✓".color(colors.success()));
    println!();
    Ok(())
}

/// Show cache statistics
pub async fn cache_stats(project_id: Option<String>) -> Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "rad braingrid cache stats".bold().color(colors.primary()));
    println!();

    let project_id = get_project_id(project_id)?;

    println!("{}", "Configuration:".bold());
    println!("  Project ID: {}", project_id.color(colors.primary()));
    println!();

    println!("{}", "Fetching cache statistics...".dimmed());
    let client = BraingridClient::new(project_id);
    let stats = client.get_cache_stats().await;

    println!("  {} Cache statistics loaded", "✓".color(colors.success()));
    println!();

    println!("{}", "─".repeat(60).dimmed());
    println!("{}", "Cache Statistics".bold().color(colors.primary()));
    println!("{}", "─".repeat(60).dimmed());
    println!();
    println!("  Hits: {}", stats.hits.to_string().color(colors.success()));
    println!("  Misses: {}", stats.misses.to_string().color(colors.warning()));
    println!("  Size: {} entries", stats.size.to_string().color(colors.primary()));
    println!("  Hit Rate: {:.2}%", stats.hit_rate());
    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!();

    Ok(())
}

// Helper functions

fn get_project_id(project_id: Option<String>) -> Result<String> {
    Ok(project_id
        .or_else(|| std::env::var("BRAINGRID_PROJECT_ID").ok())
        .unwrap_or_else(|| {
            let colors = RadiumBrandColors::new();
            println!("{}", "Warning: No project ID specified, using default PROJ-14".color(colors.warning()));
            "PROJ-14".to_string()
        }))
}

fn parse_task_status(status_str: &str) -> Result<TaskStatus> {
    match status_str.to_uppercase().as_str() {
        "PLANNED" => Ok(TaskStatus::Planned),
        "IN_PROGRESS" | "INPROGRESS" => Ok(TaskStatus::InProgress),
        "COMPLETED" => Ok(TaskStatus::Completed),
        "CANCELLED" | "CANCELED" => Ok(TaskStatus::Cancelled),
        _ => anyhow::bail!(
            "Invalid task status: {}. Valid values: PLANNED, IN_PROGRESS, COMPLETED, CANCELLED",
            status_str
        ),
    }
}

fn parse_requirement_status(status_str: &str) -> Result<RequirementStatus> {
    match status_str.to_uppercase().as_str() {
        "IDEA" => Ok(RequirementStatus::Idea),
        "PLANNED" => Ok(RequirementStatus::Planned),
        "IN_PROGRESS" | "INPROGRESS" => Ok(RequirementStatus::InProgress),
        "REVIEW" => Ok(RequirementStatus::Review),
        "COMPLETED" => Ok(RequirementStatus::Completed),
        "CANCELLED" | "CANCELED" => Ok(RequirementStatus::Cancelled),
        _ => anyhow::bail!(
            "Invalid requirement status: {}. Valid values: IDEA, PLANNED, IN_PROGRESS, REVIEW, COMPLETED, CANCELLED",
            status_str
        ),
    }
}

fn map_braingrid_error(e: BraingridError, resource_id: &str) -> anyhow::Error {
    match e {
        BraingridError::CliNotFound(path) => anyhow::anyhow!(
            "Braingrid CLI not found at '{}'. Please ensure braingrid is installed and in PATH, or set BRAINGRID_CLI_PATH environment variable.",
            path
        ),
        BraingridError::AuthenticationFailed(msg) => {
            anyhow::anyhow!("Braingrid authentication failed: {}. Please check your credentials.", msg)
        }
        BraingridError::NotFound(msg) => {
            anyhow::anyhow!("Resource not found: {} ({})", resource_id, msg)
        }
        BraingridError::InvalidStatus(msg) => {
            anyhow::anyhow!("Invalid status transition: {}", msg)
        }
        BraingridError::BreakdownFailed(msg) => {
            anyhow::anyhow!("Failed to breakdown requirement: {}", msg)
        }
        BraingridError::NetworkError(msg) => {
            anyhow::anyhow!("Network error connecting to Braingrid: {}", msg)
        }
        BraingridError::ParseError(msg) => {
            anyhow::anyhow!("Failed to parse Braingrid response: {}", msg)
        }
        BraingridError::Timeout(duration) => {
            anyhow::anyhow!("Braingrid command timed out after {:?}", duration)
        }
        BraingridError::CommandFailed(msg) => {
            anyhow::anyhow!("Braingrid command failed: {}", msg)
        }
        BraingridError::IoError(e) => {
            anyhow::anyhow!("IO error: {}", e)
        }
    }
}


//! Braingrid command implementation.
//!
//! Provides CLI commands for interacting with Braingrid requirements and tasks.

use anyhow::{Context, Result};
use colored::Colorize;
use radium_core::context::braingrid_client::{
    BraingridClient, BraingridError, CacheStats, RequirementStatus, TaskStatus,
};
use std::process::Command;

/// Read a requirement with all tasks
pub async fn read(req_id: String, project_id: Option<String>) -> Result<()> {
    println!("{}", "rad braingrid read".bold().cyan());
    println!();

    let project_id = get_project_id(project_id)?;

    println!("{}", "Configuration:".bold());
    println!("  Requirement ID: {}", req_id.cyan());
    println!("  Project ID: {}", project_id.cyan());
    println!();

    println!("{}", "Fetching requirement...".dimmed());
    let client = BraingridClient::new(project_id);
    let requirement = client
        .fetch_requirement_tree(&req_id)
        .await
        .map_err(|e| map_braingrid_error(e, &req_id))?;

    println!("  {} Requirement loaded", "✓".green());
    println!();

    // Display requirement details
    println!("{}", "─".repeat(80).dimmed());
    println!("{}", format!("Requirement: {}", requirement.id).bold().cyan());
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
                TaskStatus::Completed => "✓".green(),
                TaskStatus::InProgress => "→".cyan(),
                TaskStatus::Planned => "○".yellow(),
                TaskStatus::Cancelled => "✗".red(),
            };
            println!(
                "  {} {} {} - {}",
                status_color,
                task.task_id().cyan(),
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
    println!("{}", "rad braingrid tasks".bold().cyan());
    println!();

    let project_id = get_project_id(project_id)?;

    println!("{}", "Configuration:".bold());
    println!("  Requirement ID: {}", req_id.cyan());
    println!("  Project ID: {}", project_id.cyan());
    println!();

    println!("{}", "Fetching tasks...".dimmed());
    let client = BraingridClient::new(project_id);
    let tasks = client
        .list_tasks(&req_id)
        .await
        .map_err(|e| map_braingrid_error(e, &req_id))?;

    println!("  {} Found {} tasks", "✓".green(), tasks.len());
    println!();

    if tasks.is_empty() {
        println!("  {} No tasks found for requirement {}", "○".dimmed(), req_id.cyan());
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
            TaskStatus::Completed => status_str.green(),
            TaskStatus::InProgress => status_str.cyan(),
            TaskStatus::Planned => status_str.yellow(),
            TaskStatus::Cancelled => status_str.red(),
        };

        let title = if task.title.len() > 48 {
            format!("{}...", &task.title[..45])
        } else {
            task.title.clone()
        };

        println!(
            "{:<15} {:<10} {:<50} {:<15}",
            task.task_id().cyan(),
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
    println!("{}", "rad braingrid update-task".bold().cyan());
    println!();

    let project_id = get_project_id(project_id)?;

    let status = parse_task_status(&status_str)?;

    println!("{}", "Configuration:".bold());
    println!("  Task ID: {}", task_id.cyan());
    println!("  Requirement ID: {}", req_id.cyan());
    println!("  Status: {:?}", status);
    println!("  Project ID: {}", project_id.cyan());
    println!();

    println!("{}", "Updating task status...".dimmed());
    let client = BraingridClient::new(project_id);
    client
        .update_task_status(&task_id, &req_id, status, None)
        .await
        .map_err(|e| map_braingrid_error(e, &task_id))?;

    println!("  {} Task status updated successfully", "✓".green());
    println!();
    Ok(())
}

/// Update requirement status
pub async fn update_req(
    req_id: String,
    status_str: String,
    project_id: Option<String>,
) -> Result<()> {
    println!("{}", "rad braingrid update-req".bold().cyan());
    println!();

    let project_id = get_project_id(project_id)?;

    let status = parse_requirement_status(&status_str)?;

    println!("{}", "Configuration:".bold());
    println!("  Requirement ID: {}", req_id.cyan());
    println!("  Status: {:?}", status);
    println!("  Project ID: {}", project_id.cyan());
    println!();

    println!("{}", "Updating requirement status...".dimmed());
    let client = BraingridClient::new(project_id);
    client
        .update_requirement_status(&req_id, status)
        .await
        .map_err(|e| map_braingrid_error(e, &req_id))?;

    println!("  {} Requirement status updated successfully", "✓".green());
    println!();
    Ok(())
}

/// Trigger requirement breakdown (create tasks)
pub async fn breakdown(req_id: String, project_id: Option<String>) -> Result<()> {
    println!("{}", "rad braingrid breakdown".bold().cyan());
    println!();

    let project_id = get_project_id(project_id)?;

    println!("{}", "Configuration:".bold());
    println!("  Requirement ID: {}", req_id.cyan());
    println!("  Project ID: {}", project_id.cyan());
    println!();

    println!("{}", "Triggering requirement breakdown...".dimmed());
    let client = BraingridClient::new(project_id);
    let tasks = client
        .breakdown_requirement(&req_id)
        .await
        .map_err(|e| map_braingrid_error(e, &req_id))?;

    println!("  {} Breakdown completed", "✓".green());
    println!("  {} Created {} tasks", "→".cyan(), tasks.len());
    println!();

    if !tasks.is_empty() {
        println!("{}", "Created Tasks:".bold());
        for task in &tasks {
            println!("  {} {}", "•".cyan(), task.task_id().cyan());
            println!("      {}", task.title);
        }
        println!();
    }

    Ok(())
}

/// Clear Braingrid cache
pub async fn cache_clear(project_id: Option<String>) -> Result<()> {
    println!("{}", "rad braingrid cache clear".bold().cyan());
    println!();

    let project_id = get_project_id(project_id)?;

    println!("{}", "Configuration:".bold());
    println!("  Project ID: {}", project_id.cyan());
    println!();

    println!("{}", "Clearing cache...".dimmed());
    let client = BraingridClient::new(project_id);
    client.clear_cache().await;

    println!("  {} Cache cleared", "✓".green());
    println!();
    Ok(())
}

/// Show cache statistics
pub async fn cache_stats(project_id: Option<String>) -> Result<()> {
    println!("{}", "rad braingrid cache stats".bold().cyan());
    println!();

    let project_id = get_project_id(project_id)?;

    println!("{}", "Configuration:".bold());
    println!("  Project ID: {}", project_id.cyan());
    println!();

    println!("{}", "Fetching cache statistics...".dimmed());
    let client = BraingridClient::new(project_id);
    let stats = client.get_cache_stats().await;

    println!("  {} Cache statistics loaded", "✓".green());
    println!();

    println!("{}", "─".repeat(60).dimmed());
    println!("{}", "Cache Statistics".bold().cyan());
    println!("{}", "─".repeat(60).dimmed());
    println!();
    println!("  Hits: {}", stats.hits.to_string().green());
    println!("  Misses: {}", stats.misses.to_string().yellow());
    println!("  Size: {} entries", stats.size.to_string().cyan());
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
            println!("{}", "Warning: No project ID specified, using default PROJ-14".yellow());
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


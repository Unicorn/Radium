//! Monitoring commands for viewing agent status, telemetry, and logs.

use anyhow::{Context, Result};
use clap::Subcommand;
use radium_core::monitoring::{AgentStatus, MonitoringService, TelemetryTracking};
use radium_core::workspace::Workspace;

/// Monitoring subcommands
#[derive(Subcommand, Debug)]
pub enum MonitorCommand {
    /// Show agent status
    Status {
        /// Agent ID (optional, shows all if not specified)
        agent_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List all agents
    List {
        /// Filter by status (starting, running, completed, failed, terminated)
        #[arg(long)]
        status: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show telemetry and cost information
    Telemetry {
        /// Agent ID (optional, shows all if not specified)
        agent_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// Execute monitor command
pub async fn execute(cmd: MonitorCommand) -> Result<()> {
    let workspace = Workspace::discover()
        .context("No Radium workspace found. Run 'rad init' to create one.")?;

    let monitoring_path = workspace.radium_dir().join("monitoring.db");
    let monitoring = MonitoringService::open(monitoring_path)
        .context("Failed to open monitoring database. No agents have been tracked yet.")?;

    match cmd {
        MonitorCommand::Status { agent_id, json } => {
            status_command(&monitoring, agent_id.as_deref(), json).await
        }
        MonitorCommand::List { status, json } => {
            list_command(&monitoring, status.as_deref(), json).await
        }
        MonitorCommand::Telemetry { agent_id, json } => {
            telemetry_command(&monitoring, agent_id.as_deref(), json).await
        }
    }
}

async fn status_command(
    monitoring: &MonitoringService,
    agent_id: Option<&str>,
    json: bool,
) -> Result<()> {
    if let Some(id) = agent_id {
        let record = monitoring.get_agent(id).context(format!("Agent {} not found", id))?;

        if json {
            println!("{}", serde_json::to_string_pretty(&record)?);
        } else {
            println!("Agent: {}", record.id);
            println!("Type: {}", record.agent_type);
            println!("Status: {:?}", record.status);
            if let Some(ref parent_id) = record.parent_id {
                println!("Parent: {}", parent_id);
            }
            if let Some(ref plan_id) = record.plan_id {
                println!("Plan: {}", plan_id);
            }
            if let Some(process_id) = record.process_id {
                println!("Process ID: {}", process_id);
            }
            if let Some(end_time) = record.end_time {
                let duration = end_time - record.start_time;
                println!("Duration: {}s", duration);
            }
            if let Some(exit_code) = record.exit_code {
                println!("Exit Code: {}", exit_code);
            }
            if let Some(ref error) = record.error_message {
                println!("Error: {}", error);
            }
            if let Some(ref log_file) = record.log_file {
                println!("Log File: {}", log_file);
            }
        }
    } else {
        let agents = monitoring.list_agents()?;
        if json {
            println!("{}", serde_json::to_string_pretty(&agents)?);
        } else {
            if agents.is_empty() {
                println!("No agents found.");
            } else {
                println!("{:<30} {:<15} {:<20} {:<10}", "ID", "Type", "Status", "Plan");
                println!("{}", "-".repeat(75));
                for record in agents {
                    let plan = record.plan_id.as_deref().unwrap_or("-");
                    println!(
                        "{:<30} {:<15} {:<20} {:<10}",
                        record.id,
                        record.agent_type,
                        format!("{:?}", record.status),
                        plan
                    );
                }
            }
        }
    }
    Ok(())
}

async fn list_command(
    monitoring: &MonitoringService,
    status_filter: Option<&str>,
    json: bool,
) -> Result<()> {
    let mut agents = monitoring.list_agents()?;

    // Filter by status if specified
    if let Some(status_str) = status_filter {
        let status = AgentStatus::from_str(status_str).context(format!(
            "Invalid status: {}. Valid values: starting, running, completed, failed, terminated",
            status_str
        ))?;
        agents.retain(|a| a.status == status);
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&agents)?);
    } else {
        if agents.is_empty() {
            println!("No agents found.");
        } else {
            println!(
                "{:<30} {:<15} {:<20} {:<10} {:<15}",
                "ID", "Type", "Status", "Plan", "Duration"
            );
            println!("{}", "-".repeat(90));
            for record in agents {
                let plan = record.plan_id.as_deref().unwrap_or("-");
                let duration = if let Some(end_time) = record.end_time {
                    format!("{}s", end_time - record.start_time)
                } else {
                    "running".to_string()
                };
                println!(
                    "{:<30} {:<15} {:<20} {:<10} {:<15}",
                    record.id,
                    record.agent_type,
                    format!("{:?}", record.status),
                    plan,
                    duration
                );
            }
        }
    }
    Ok(())
}

async fn telemetry_command(
    monitoring: &MonitoringService,
    agent_id: Option<&str>,
    json: bool,
) -> Result<()> {
    if let Some(id) = agent_id {
        let telemetry = monitoring.get_agent_telemetry(id)?;
        let total_cost = monitoring.get_total_cost(id)?;

        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "agent_id": id,
                    "telemetry": telemetry,
                    "total_cost": total_cost
                }))?
            );
        } else {
            println!("Telemetry for agent: {}", id);
            println!("Total Cost: ${:.4}", total_cost);
            println!();
            if telemetry.is_empty() {
                println!("No telemetry data found.");
            } else {
                println!(
                    "{:<20} {:<15} {:<15} {:<15} {:<15} {:<10}",
                    "Timestamp", "Input Tokens", "Output Tokens", "Total Tokens", "Cost", "Model"
                );
                println!("{}", "-".repeat(90));
                for record in telemetry {
                    let timestamp = chrono::DateTime::from_timestamp(record.timestamp as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| record.timestamp.to_string());
                    let model = record.model.as_deref().unwrap_or("-");
                    println!(
                        "{:<20} {:<15} {:<15} {:<15} ${:<14.4} {:<10}",
                        timestamp,
                        record.input_tokens,
                        record.output_tokens,
                        record.total_tokens,
                        record.estimated_cost,
                        model
                    );
                }
            }
        }
    } else {
        // Show summary for all agents
        let agents = monitoring.list_agents()?;
        let mut total_cost = 0.0;
        let mut total_tokens = 0u64;

        for agent in &agents {
            let cost = monitoring.get_total_cost(&agent.id).unwrap_or(0.0);
            total_cost += cost;
            let telemetry = monitoring.get_agent_telemetry(&agent.id).unwrap_or_default();
            for t in telemetry {
                total_tokens += t.total_tokens;
            }
        }

        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "total_agents": agents.len(),
                    "total_cost": total_cost,
                    "total_tokens": total_tokens
                }))?
            );
        } else {
            println!("Monitoring Summary");
            println!("Total Agents: {}", agents.len());
            println!("Total Cost: ${:.4}", total_cost);
            println!("Total Tokens: {}", total_tokens);
        }
    }
    Ok(())
}

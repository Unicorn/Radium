//! Monitoring commands for viewing agent status, telemetry, and logs.

use anyhow::{Context, Result};
use clap::Subcommand;
use radium_core::monitoring::{AgentStatus, AgentUsage, MonitoringService, TelemetryTracking, UsageFilter};
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
        /// Show tool execution details
        #[arg(long)]
        show_tools: bool,
        /// Filter by tool name
        #[arg(long)]
        tool: Option<String>,
    },
    /// Show agent usage analytics
    Usage {
        /// Agent ID (optional, shows all if not specified)
        agent_id: Option<String>,
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Minimum execution count
        #[arg(long)]
        min_executions: Option<u64>,
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
        MonitorCommand::Telemetry { agent_id, json, show_tools, tool } => {
            telemetry_command(&monitoring, agent_id.as_deref(), json, show_tools, tool.as_deref()).await
        }
        MonitorCommand::Usage {
            agent_id,
            category,
            min_executions,
            json,
        } => {
            usage_command(&monitoring, agent_id.as_deref(), category.as_deref(), min_executions, json).await
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
    show_tools: bool,
    tool_filter: Option<&str>,
) -> Result<()> {
    if let Some(id) = agent_id {
        let mut telemetry = monitoring.get_agent_telemetry(id)?;
        
        // Filter by tool if specified
        if let Some(tool_name) = tool_filter {
            telemetry.retain(|t| t.tool_name.as_deref() == Some(tool_name));
        }
        
        let total_cost = telemetry.iter().map(|t| t.estimated_cost).sum::<f64>();

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
                if show_tools {
                    // Enhanced display with tool information
                    println!(
                        "{:<20} {:<15} {:<15} {:<15} {:<15} {:<20} {:<15} {:<10}",
                        "Timestamp", "Input Tokens", "Output Tokens", "Total Tokens", "Cost", "Tool", "Approval", "Model"
                    );
                    println!("{}", "-".repeat(130));
                    for record in &telemetry {
                        let timestamp = chrono::DateTime::from_timestamp(record.timestamp as i64, 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_else(|| record.timestamp.to_string());
                        let model = record.model.as_deref().unwrap_or("-");
                        let tool = record.tool_name.as_deref().unwrap_or("-");
                        let approval = if let Some(approved) = record.tool_approved {
                            if approved {
                                record.tool_approval_type.as_deref().unwrap_or("approved")
                            } else {
                                "denied"
                            }
                        } else {
                            "-"
                        };
                        println!(
                            "{:<20} {:<15} {:<15} {:<15} ${:<14.4} {:<20} {:<15} {:<10}",
                            timestamp,
                            record.input_tokens,
                            record.output_tokens,
                            record.total_tokens,
                            record.estimated_cost,
                            tool,
                            approval,
                            model
                        );
                    }
                    
                    // Show tool usage summary
                    let tool_usage: std::collections::HashMap<String, (u64, u64, u64)> = telemetry
                        .iter()
                        .filter_map(|t| {
                            t.tool_name.as_ref().map(|name| {
                                let count = 1;
                                let approved = if t.tool_approved == Some(true) { 1 } else { 0 };
                                let denied = if t.tool_approved == Some(false) { 1 } else { 0 };
                                (name.clone(), (count, approved, denied))
                            })
                        })
                        .fold(std::collections::HashMap::new(), |mut acc, (name, (count, approved, denied))| {
                            let entry = acc.entry(name).or_insert((0, 0, 0));
                            entry.0 += count;
                            entry.1 += approved;
                            entry.2 += denied;
                            acc
                        });
                    
                    if !tool_usage.is_empty() {
                        println!();
                        println!("Tool Usage Summary:");
                        println!("{:<30} {:<15} {:<15} {:<15}", "Tool", "Executions", "Approved", "Denied");
                        println!("{}", "-".repeat(75));
                        let mut tool_vec: Vec<_> = tool_usage.into_iter().collect();
                        tool_vec.sort_by(|a, b| b.1 .0.cmp(&a.1 .0)); // Sort by execution count
                        for (tool, (count, approved, denied)) in tool_vec {
                            println!(
                                "{:<30} {:<15} {:<15} {:<15}",
                                tool,
                                count,
                                approved,
                                denied
                            );
                        }
                    }
                } else {
                    // Standard display without tool details
                    println!(
                        "{:<20} {:<15} {:<15} {:<15} {:<15} {:<10}",
                        "Timestamp", "Input Tokens", "Output Tokens", "Total Tokens", "Cost", "Model"
                    );
                    println!("{}", "-".repeat(90));
                    for record in &telemetry {
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
        }
    } else {
        // Show summary for all agents (optimized query)
        let summary = monitoring.get_telemetry_summary()?;
        let total_cost: f64 = summary.iter().map(|s| s.total_cost).sum();
        let total_tokens: u64 = summary.iter().map(|s| s.total_tokens).sum();

        // Get tool usage summary if requested
        let mut tool_summary: Option<std::collections::HashMap<String, (u64, u64, u64)>> = None;
        if show_tools {
            let mut tool_usage = std::collections::HashMap::new();
            for s in &summary {
                let telemetry = monitoring.get_agent_telemetry(&s.agent_id)?;
                for t in telemetry {
                    if let Some(ref tool_name) = t.tool_name {
                        let entry = tool_usage.entry(tool_name.clone()).or_insert((0, 0, 0));
                        entry.0 += 1;
                        if t.tool_approved == Some(true) {
                            entry.1 += 1;
                        } else if t.tool_approved == Some(false) {
                            entry.2 += 1;
                        }
                    }
                }
            }
            tool_summary = Some(tool_usage);
        }

        if json {
            let mut json_obj = serde_json::json!({
                "total_agents": summary.len(),
                "total_cost": total_cost,
                "total_tokens": total_tokens,
                "summary": summary
            });
            if let Some(ref tool_usage) = tool_summary {
                let tool_json: Vec<_> = tool_usage.iter().map(|(tool, (count, approved, denied))| {
                    serde_json::json!({
                        "tool": tool,
                        "executions": count,
                        "approved": approved,
                        "denied": denied
                    })
                }).collect();
                json_obj["tool_usage"] = serde_json::json!(tool_json);
            }
            println!("{}", serde_json::to_string_pretty(&json_obj)?);
        } else {
            println!("Monitoring Summary");
            println!("Total Agents: {}", summary.len());
            println!("Total Cost: ${:.4}", total_cost);
            println!("Total Tokens: {}", total_tokens);
            if !summary.is_empty() {
                println!();
                println!("{:<30} {:<15} {:<15} {:<10}", "Agent ID", "Total Tokens", "Total Cost", "Records");
                println!("{}", "-".repeat(70));
                for s in &summary {
                    println!(
                        "{:<30} {:<15} ${:<14.4} {:<10}",
                        s.agent_id,
                        s.total_tokens,
                        s.total_cost,
                        s.record_count
                    );
                }
            }
            
            // Show tool usage summary if requested
            if let Some(tool_usage) = tool_summary {
                if !tool_usage.is_empty() {
                    println!();
                    println!("Tool Usage Summary (All Agents):");
                    println!("{:<30} {:<15} {:<15} {:<15}", "Tool", "Executions", "Approved", "Denied");
                    println!("{}", "-".repeat(75));
                    let mut tool_vec: Vec<_> = tool_usage.into_iter().collect();
                    tool_vec.sort_by(|a, b| b.1 .0.cmp(&a.1 .0)); // Sort by execution count
                    for (tool, (count, approved, denied)) in tool_vec {
                        println!(
                            "{:<30} {:<15} {:<15} {:<15}",
                            tool,
                            count,
                            approved,
                            denied
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

async fn usage_command(
    monitoring: &MonitoringService,
    agent_id: Option<&str>,
    category: Option<&str>,
    min_executions: Option<u64>,
    json: bool,
) -> Result<()> {
    if let Some(id) = agent_id {
        let usage = monitoring.get_agent_usage(id)
            .context(format!("Agent usage for {} not found", id))?;

        if let Some(usage) = usage {
            if json {
                println!("{}", serde_json::to_string_pretty(&usage)?);
            } else {
                display_agent_usage(&usage);
            }
        } else {
            println!("No usage data found for agent: {}", id);
        }
    } else {
        // List all usage with filters
        let filter = UsageFilter {
            category: category.map(|s| s.to_string()),
            min_executions,
            since: None,
        };

        let usage_list = monitoring.list_agent_usage(filter)?;

        if json {
            println!("{}", serde_json::to_string_pretty(&usage_list)?);
        } else {
            if usage_list.is_empty() {
                println!("No agent usage data found.");
            } else {
                println!("{:<30} {:<12} {:<15} {:<12} {:<10} {:<10} {:<20}", 
                    "Agent ID", "Executions", "Total Duration", "Total Tokens", "Success", "Failure", "Last Used");
                println!("{}", "-".repeat(120));
                for usage in usage_list {
                    let last_used = if let Some(timestamp) = usage.last_used_at {
                        chrono::DateTime::from_timestamp(timestamp as i64, 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_else(|| timestamp.to_string())
                    } else {
                        "Never".to_string()
                    };
                    let duration_secs = usage.total_duration / 1000;
                    let avg_duration = if usage.execution_count > 0 {
                        duration_secs as f64 / usage.execution_count as f64
                    } else {
                        0.0
                    };
                    let success_rate = if usage.execution_count > 0 {
                        (usage.success_count as f64 / usage.execution_count as f64) * 100.0
                    } else {
                        0.0
                    };

                    println!(
                        "{:<30} {:<12} {:<15} {:<12} {:<10} {:<10} {:<20}",
                        usage.agent_id,
                        usage.execution_count,
                        format!("{}s (avg: {:.1}s)", duration_secs, avg_duration),
                        usage.total_tokens,
                        format!("{} ({:.1}%)", usage.success_count, success_rate),
                        usage.failure_count,
                        last_used
                    );
                }
            }
        }
    }
    Ok(())
}

fn display_agent_usage(usage: &AgentUsage) {
    println!("Agent Usage: {}", usage.agent_id);
    println!("Execution Count: {}", usage.execution_count);
    
    let duration_secs = usage.total_duration / 1000;
    let avg_duration = if usage.execution_count > 0 {
        duration_secs as f64 / usage.execution_count as f64
    } else {
        0.0
    };
    println!("Total Duration: {}s (average: {:.1}s per execution)", duration_secs, avg_duration);
    
    println!("Total Tokens: {}", usage.total_tokens);
    println!("Success Count: {}", usage.success_count);
    println!("Failure Count: {}", usage.failure_count);
    
    let success_rate = if usage.execution_count > 0 {
        (usage.success_count as f64 / usage.execution_count as f64) * 100.0
    } else {
        0.0
    };
    println!("Success Rate: {:.1}%", success_rate);
    
    if let Some(timestamp) = usage.last_used_at {
        let last_used = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| timestamp.to_string());
        println!("Last Used: {}", last_used);
    } else {
        println!("Last Used: Never");
    }
    
    if let Some(ref category) = usage.category {
        println!("Category: {}", category);
    }
}

//! Statistics and analytics commands for session reporting.

use anyhow::{Context, Result};
use clap::Subcommand;
use radium_core::analytics::{ReportFormatter, SessionAnalytics, SessionReport, SessionStorage};
use radium_core::monitoring::MonitoringService;
use radium_core::workspace::Workspace;
use std::fs;

/// Statistics subcommands
#[derive(Subcommand, Debug)]
pub enum StatsCommand {
    /// Show current session statistics
    Session {
        /// Session ID (optional, uses most recent if not specified)
        session_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show detailed model usage breakdown
    Model {
        /// Session ID (optional, shows all sessions if not specified)
        session_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show historical session summaries
    History {
        /// Number of sessions to show (default: 10)
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Export analytics to JSON
    Export {
        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
        /// Session ID (optional, exports all if not specified)
        session_id: Option<String>,
    },
}

/// Execute stats command
pub async fn execute(cmd: StatsCommand) -> Result<()> {
    let workspace = Workspace::discover()
        .context("No Radium workspace found. Run 'rad init' to create one.")?;

    let monitoring_path = workspace.radium_dir().join("monitoring.db");
    let monitoring = MonitoringService::open(monitoring_path)
        .context("Failed to open monitoring database. No agents have been tracked yet.")?;

    let analytics = SessionAnalytics::new(monitoring);

    match cmd {
        StatsCommand::Session { session_id, json } => {
            session_command(&analytics, session_id.as_deref(), json).await
        }
        StatsCommand::Model { session_id, json } => {
            model_command(&analytics, session_id.as_deref(), json).await
        }
        StatsCommand::History { limit, json } => history_command(&analytics, limit, json).await,
        StatsCommand::Export { output, session_id } => {
            export_command(&analytics, output.as_deref(), session_id.as_deref()).await
        }
    }
}

async fn session_command(
    analytics: &SessionAnalytics,
    session_id: Option<&str>,
    json: bool,
) -> Result<()> {
    let session_id = session_id
        .map(|s| s.to_string())
        .or_else(|| get_latest_session_id())
        .context("No session ID provided and no recent sessions found")?;

    let metrics = analytics.get_session_metrics(&session_id)?;
    let report = SessionReport::new(metrics);

    // Save report to storage for future reference
    let workspace = Workspace::discover()?;
    let storage = SessionStorage::new(workspace.root())?;
    let _ = storage.save_report(&report); // Ignore errors

    let formatter = ReportFormatter;

    if json {
        println!("{}", formatter.format_json(&report)?);
    } else {
        println!("{}", formatter.format(&report));
    }

    Ok(())
}

async fn model_command(
    analytics: &SessionAnalytics,
    session_id: Option<&str>,
    json: bool,
) -> Result<()> {
    // For now, show model usage from the session
    // In the future, this could aggregate across all sessions
    if let Some(sid) = session_id {
        let metrics = analytics.get_session_metrics(sid)?;

        if json {
            let model_data: Vec<serde_json::Value> = metrics
                .model_usage
                .iter()
                .map(|(model, stats)| {
                    serde_json::json!({
                        "model": model,
                        "requests": stats.requests,
                        "input_tokens": stats.input_tokens,
                        "output_tokens": stats.output_tokens,
                        "cached_tokens": stats.cached_tokens,
                        "estimated_cost": stats.estimated_cost,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&model_data)?);
        } else {
            println!("Model Usage Breakdown for Session: {}\n", sid);
            println!(
                "{:<30} {:<10} {:<15} {:<15} {:<15} {:<12}",
                "Model", "Requests", "Input Tokens", "Output Tokens", "Cached Tokens", "Cost"
            );
            println!("{}", "-".repeat(100));
            for (model, stats) in &metrics.model_usage {
                println!(
                    "{:<30} {:<10} {:<15} {:<15} {:<15} ${:<11.4}",
                    model,
                    stats.requests,
                    stats.input_tokens,
                    stats.output_tokens,
                    stats.cached_tokens,
                    stats.estimated_cost
                );
            }
        }
    } else {
        // Show aggregated model usage across all sessions
        // TODO: Implement aggregation across all sessions
        println!("Aggregated model usage (all sessions) - Coming soon");
    }

    Ok(())
}

async fn history_command(_analytics: &SessionAnalytics, limit: usize, json: bool) -> Result<()> {
    // Load session reports from storage
    let workspace = Workspace::discover()?;
    let storage = SessionStorage::new(workspace.root())?;
    let mut reports = storage.list_reports()?;

    // Limit results
    reports.truncate(limit);

    if json {
        println!("{}", serde_json::to_string_pretty(&reports)?);
    } else {
        if reports.is_empty() {
            println!("No session history found.");
        } else {
            println!("Recent Session Summaries\n");
            println!(
                "{:<30} {:<20} {:<15} {:<15} {:<12}",
                "Session ID", "Duration", "Tool Calls", "Success Rate", "Cost"
            );
            println!("{}", "-".repeat(92));
            for report in &reports {
                let duration = report.metrics.wall_time;
                let duration_str = if duration.as_secs() > 3600 {
                    format!("{}h {}m", duration.as_secs() / 3600, (duration.as_secs() % 3600) / 60)
                } else if duration.as_secs() > 60 {
                    format!("{}m {}s", duration.as_secs() / 60, duration.as_secs() % 60)
                } else {
                    format!("{}s", duration.as_secs())
                };

                println!(
                    "{:<30} {:<20} {:<15} {:<14.1}% ${:<11.4}",
                    report.metrics.session_id,
                    duration_str,
                    report.metrics.tool_calls,
                    report.metrics.success_rate(),
                    report.metrics.total_cost
                );
            }
        }
    }

    Ok(())
}

async fn export_command(
    analytics: &SessionAnalytics,
    output: Option<&str>,
    session_id: Option<&str>,
) -> Result<()> {
    let workspace = Workspace::discover()?;
    let storage = SessionStorage::new(workspace.root())?;

    if let Some(sid) = session_id {
        // Try to load from storage first, otherwise generate
        let report = if let Ok(stored) = storage.load_report(sid) {
            stored
        } else {
            let metrics = analytics.get_session_metrics(sid)?;
            let report = SessionReport::new(metrics);
            // Save it for future use
            storage.save_report(&report)?;
            report
        };

        let json = serde_json::to_string_pretty(&report)?;

        if let Some(output_path) = output {
            fs::write(output_path, json)?;
            println!("Exported session {} to {}", sid, output_path);
        } else {
            println!("{}", json);
        }
    } else {
        // Export all stored sessions
        let reports = storage.list_reports()?;
        let json = serde_json::to_string_pretty(&reports)?;

        if let Some(output_path) = output {
            fs::write(output_path, json)?;
            println!("Exported {} sessions to {}", reports.len(), output_path);
        } else {
            println!("{}", json);
        }
    }

    Ok(())
}

/// Get the most recent session ID from stored reports.
fn get_latest_session_id() -> Option<String> {
    let workspace = Workspace::discover().ok()?;
    let storage = SessionStorage::new(workspace.root()).ok()?;
    let reports = storage.list_reports().ok()?;

    reports.first().map(|r| r.metrics.session_id.clone())
}

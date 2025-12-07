//! Statistics and analytics commands for session reporting.

use anyhow::{Context, Result};
use clap::Subcommand;
use radium_core::analytics::{ComparisonFormatter, ReportFormatter, SessionAnalytics, SessionComparison, SessionReport, SessionStorage};
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
    /// Show engine usage breakdown
    Engine {
        /// Session ID (optional, shows all sessions if not specified)
        session_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show historical session summaries
    History {
        /// Number of sessions to show (default: 10, max: 100)
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
    /// Compare two sessions
    Compare {
        /// First session ID
        session_a: String,
        /// Second session ID
        session_b: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
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
        StatsCommand::Engine { session_id, json } => {
            engine_command(&analytics, session_id.as_deref(), json).await
        }
        StatsCommand::History { limit, json } => history_command(&analytics, limit, json).await,
        StatsCommand::Export { output, session_id } => {
            export_command(&analytics, output.as_deref(), session_id.as_deref()).await
        }
        StatsCommand::Compare { session_a, session_b, json } => {
            compare_command(&analytics, &session_a, &session_b, json).await
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
        println!("{}", formatter.format_json(&report, false)?);
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
        let workspace = Workspace::discover()?;
        let aggregated = analytics.get_aggregated_model_usage(Some(workspace.root()))?;

        if json {
            let model_data: Vec<serde_json::Value> = aggregated
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
            println!("Aggregated Model Usage (All Sessions)\n");
            if aggregated.is_empty() {
                println!("No session data available.");
            } else {
                println!(
                    "{:<30} {:<10} {:<15} {:<15} {:<15} {:<12}",
                    "Model", "Requests", "Input Tokens", "Output Tokens", "Cached Tokens", "Cost"
                );
                println!("{}", "-".repeat(100));
                
                // Sort by total tokens (descending) for better readability
                let mut sorted_models: Vec<_> = aggregated.iter().collect();
                sorted_models.sort_by(|a, b| {
                    let total_a = a.1.input_tokens + a.1.output_tokens;
                    let total_b = b.1.input_tokens + b.1.output_tokens;
                    total_b.cmp(&total_a)
                });
                
                for (model, stats) in sorted_models {
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
                
                // Print totals
                let total_requests: u64 = aggregated.values().map(|s| s.requests).sum();
                let total_input: u64 = aggregated.values().map(|s| s.input_tokens).sum();
                let total_output: u64 = aggregated.values().map(|s| s.output_tokens).sum();
                let total_cached: u64 = aggregated.values().map(|s| s.cached_tokens).sum();
                let total_cost: f64 = aggregated.values().map(|s| s.estimated_cost).sum();
                
                println!("{}", "-".repeat(100));
                println!(
                    "{:<30} {:<10} {:<15} {:<15} {:<15} ${:<11.4}",
                    "TOTAL",
                    total_requests,
                    total_input,
                    total_output,
                    total_cached,
                    total_cost
                );
            }
        }
    }

    Ok(())
}

async fn engine_command(
    analytics: &SessionAnalytics,
    session_id: Option<&str>,
    json: bool,
) -> Result<()> {
    // Show engine usage from the session
    if let Some(sid) = session_id {
        let metrics = analytics.get_session_metrics(sid)?;

        if json {
            let engine_data: Vec<serde_json::Value> = metrics
                .engine_usage
                .iter()
                .map(|(engine, stats)| {
                    serde_json::json!({
                        "engine": engine,
                        "requests": stats.requests,
                        "input_tokens": stats.input_tokens,
                        "output_tokens": stats.output_tokens,
                        "cached_tokens": stats.cached_tokens,
                        "estimated_cost": stats.estimated_cost,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&engine_data)?);
        } else {
            println!("Engine Usage Breakdown for Session: {}\n", sid);
            if metrics.engine_usage.is_empty() {
                println!("No engine usage data available.");
            } else {
                println!(
                    "{:<20} {:<10} {:<15} {:<15} {:<15} {:<12}",
                    "Engine", "Requests", "Input Tokens", "Output Tokens", "Cached Tokens", "Cost"
                );
                println!("{}", "-".repeat(100));
                
                // Sort by total tokens (descending)
                let mut sorted_engines: Vec<_> = metrics.engine_usage.iter().collect();
                sorted_engines.sort_by(|a, b| {
                    let total_a = a.1.input_tokens + a.1.output_tokens;
                    let total_b = b.1.input_tokens + b.1.output_tokens;
                    total_b.cmp(&total_a)
                });
                
                for (engine, stats) in sorted_engines {
                    println!(
                        "{:<20} {:<10} {:<15} {:<15} {:<15} ${:<11.4}",
                        engine,
                        stats.requests,
                        stats.input_tokens,
                        stats.output_tokens,
                        stats.cached_tokens,
                        stats.estimated_cost
                    );
                }
            }
        }
    } else {
        // Show aggregated engine usage across all sessions
        println!("Aggregated Engine Usage (All Sessions)\n");
        println!("Note: Full aggregation across all sessions requires database query support.");
        println!("For session-specific engine usage, use: rad stats engine <session-id>");
    }

    Ok(())
}

async fn history_command(_analytics: &SessionAnalytics, limit: usize, json: bool) -> Result<()> {
    // Enforce maximum limit
    let limit = limit.min(100);
    
    // Load session reports from storage with pagination
    let workspace = Workspace::discover()?;
    let storage = SessionStorage::new(workspace.root())?;
    let reports = storage.list_reports_paginated(Some(limit), None)?;

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

async fn compare_command(
    analytics: &SessionAnalytics,
    session_a: &str,
    session_b: &str,
    json: bool,
) -> Result<()> {
    let workspace = Workspace::discover()?;
    let storage = SessionStorage::new(workspace.root())?;

    // Try to load from storage first, otherwise generate
    let report_a = if let Ok(stored) = storage.load_report(session_a) {
        stored
    } else {
        let metrics = analytics.get_session_metrics(session_a)?;
        SessionReport::new(metrics)
    };

    let report_b = if let Ok(stored) = storage.load_report(session_b) {
        stored
    } else {
        let metrics = analytics.get_session_metrics(session_b)?;
        SessionReport::new(metrics)
    };

    let comparison = SessionComparison::new(&report_a, &report_b);

    if json {
        // For JSON output, serialize the comparison
        let json_value = serde_json::json!({
            "session_a": comparison.session_a_id,
            "session_b": comparison.session_b_id,
            "token_delta": comparison.token_delta,
            "token_percentage_change": comparison.token_percentage_change(),
            "cost_delta": comparison.cost_delta,
            "cost_percentage_change": comparison.cost_percentage_change(),
            "wall_time_delta_secs": comparison.wall_time_delta.as_secs(),
            "wall_time_percentage_change": comparison.wall_time_percentage_change(),
            "agent_active_time_delta_secs": comparison.agent_active_time_delta.as_secs(),
            "tool_calls_delta": comparison.tool_calls_delta,
            "success_rate_delta": comparison.success_rate_delta,
            "lines_added_delta": comparison.lines_added_delta,
            "lines_removed_delta": comparison.lines_removed_delta,
        });
        println!("{}", serde_json::to_string_pretty(&json_value)?);
    } else {
        let formatter = ComparisonFormatter;
        println!("{}", formatter.format(&comparison));
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

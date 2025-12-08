//! Statistics and analytics commands for session reporting.

use anyhow::{Context, Result};
use clap::Subcommand;
use radium_core::analytics::{
    ComparisonFormatter, CostQueryService, CsvExporter, ExportFormat, ExportOptions, Exporter,
    JsonExporter, MarkdownExporter, ReportFormatter, SessionAnalytics, SessionComparison,
    SessionReport, SessionStorage,
};
use radium_core::monitoring::MonitoringService;
use radium_core::workspace::Workspace;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

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
    /// Export analytics data
    Export {
        /// Output file path (default: stdout for session export, ~/.radium/reports/ for cost export)
        #[arg(short, long)]
        output: Option<String>,
        /// Session ID (optional, exports session report if provided, otherwise exports cost data)
        session_id: Option<String>,
        /// Export format for cost data (csv, json, markdown). Ignored if session_id is provided.
        #[arg(long)]
        format: Option<String>,
        /// Start date for cost export filter (ISO 8601 format, e.g., 2025-12-01)
        #[arg(long)]
        start: Option<String>,
        /// End date for cost export filter (ISO 8601 format, e.g., 2025-12-31)
        #[arg(long)]
        end: Option<String>,
        /// Filter by plan/requirement ID (e.g., REQ-123)
        #[arg(long)]
        plan: Option<String>,
        /// Filter by provider (e.g., anthropic, openai)
        #[arg(long)]
        provider: Option<String>,
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
        StatsCommand::Export {
            output,
            session_id,
            format,
            start,
            end,
            plan,
            provider,
        } => {
            export_command(
                &analytics,
                &monitoring,
                output.as_deref(),
                session_id.as_deref(),
                format.as_deref(),
                start.as_deref(),
                end.as_deref(),
                plan.as_deref(),
                provider.as_deref(),
            )
            .await
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
    monitoring: &MonitoringService,
    output: Option<&str>,
    session_id: Option<&str>,
    format: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
    plan: Option<&str>,
    provider: Option<&str>,
) -> Result<()> {
    // If session_id is provided, export session report (backward compatibility)
    if let Some(sid) = session_id {
        let workspace = Workspace::discover()?;
        let storage = SessionStorage::new(workspace.root())?;

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
        return Ok(());
    }

    // Otherwise, export cost data
    // Parse format (default to csv)
    let export_format = if let Some(fmt_str) = format {
        ExportFormat::from_str(fmt_str).context(format!("Invalid format: {}. Use csv, json, or markdown", fmt_str))?
    } else {
        ExportFormat::Csv
    };

    // Parse dates
    let start_date = if let Some(start_str) = start {
        Some(
            DateTime::parse_from_rfc3339(start_str)
                .or_else(|_| DateTime::parse_from_str(start_str, "%Y-%m-%d"))
                .context(format!("Invalid start date format: {}. Use ISO 8601 or YYYY-MM-DD", start_str))?
                .with_timezone(&Utc),
        )
    } else {
        None
    };

    let end_date = if let Some(end_str) = end {
        Some(
            DateTime::parse_from_rfc3339(end_str)
                .or_else(|_| DateTime::parse_from_str(end_str, "%Y-%m-%d"))
                .context(format!("Invalid end date format: {}. Use ISO 8601 or YYYY-MM-DD", end_str))?
                .with_timezone(&Utc),
        )
    } else {
        None
    };

    // Create export options
    let output_path = if let Some(path) = output {
        Some(PathBuf::from(path))
    } else {
        Some(generate_default_output_path(&export_format)?)
    };

    let options = ExportOptions {
        format: export_format,
        start_date,
        end_date,
        plan_id: plan.map(|s| s.to_string()),
        provider: provider.map(|s| s.to_string()),
        output_path: output_path.clone(),
    };

    // Query cost data
    let cost_service = CostQueryService::new(monitoring);
    let records = cost_service.query_records(&options)
        .context("Failed to query cost data from database")?;

    if records.is_empty() {
        println!("No cost data found matching the specified filters.");
        return Ok(());
    }

    // Generate summary
    let summary = cost_service.generate_summary(&records);

    // Select exporter
    let exporter: Box<dyn Exporter> = match options.format {
        ExportFormat::Csv => Box::new(CsvExporter),
        ExportFormat::Json => Box::new(JsonExporter),
        ExportFormat::Markdown => Box::new(MarkdownExporter),
    };

    // Export (use summary for better readability, but could also use detailed records)
    let export_content = if records.len() > 100 {
        // For large datasets, export summary
        exporter.export_summary(&summary, &options)
            .context("Failed to generate export")?
    } else {
        // For smaller datasets, export detailed records
        exporter.export(&records, &options)
            .context("Failed to generate export")?
    };

    // Write to file
    let output_path_str = output_path.as_ref().unwrap();
    fs::create_dir_all(output_path_str.parent().unwrap())?;
    fs::write(output_path_str, export_content)?;

    println!(
        "Exported {} cost records to {}",
        records.len(),
        output_path_str.display()
    );
    println!("Total cost: ${:.4}", summary.total_cost);
    println!("Total tokens: {}", summary.total_tokens);

    Ok(())
}

/// Generate default output path for cost export.
fn generate_default_output_path(format: &ExportFormat) -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .context("Could not determine home directory")?;
    
    let reports_dir = PathBuf::from(home).join(".radium").join("reports");
    fs::create_dir_all(&reports_dir)?;

    let date_str = Utc::now().format("%Y-%m-%d").to_string();
    let filename = format!("{}-costs-export.{}", date_str, format.extension());
    
    Ok(reports_dir.join(filename))
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

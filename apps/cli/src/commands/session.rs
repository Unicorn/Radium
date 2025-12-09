//! Session management commands for searching, exporting, deleting, and viewing session information.

use anyhow::{Context, Result};
use clap::Subcommand;
use radium_core::analytics::{SessionReport, SessionStorage};
use radium_core::context::HistoryManager;
use radium_core::workspace::Workspace;
use serde::Serialize;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use uuid::Uuid;

/// Session management subcommands
#[derive(Subcommand, Debug)]
pub enum SessionCommand {
    /// Search sessions by content or metadata
    Search {
        /// Search query (searches in conversation content)
        query: String,
        /// Filter by agent ID
        #[arg(long)]
        agent: Option<String>,
        /// Filter by date (ISO 8601 format, e.g., 2025-12-01)
        #[arg(long)]
        date: Option<String>,
        /// Filter by model name
        #[arg(long)]
        model: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Export session transcript to file
    Export {
        /// Session ID to export
        session_id: String,
        /// Export format (json, markdown). Default: json
        #[arg(long)]
        format: Option<String>,
        /// Include analytics data in export
        #[arg(long)]
        include_analytics: bool,
    },
    /// Delete sessions
    Delete {
        /// Session ID to delete (mutually exclusive with --before and --all)
        session_id: Option<String>,
        /// Delete all sessions before this date (ISO 8601 format)
        #[arg(long)]
        before: Option<String>,
        /// Delete all sessions (requires confirmation)
        #[arg(long)]
        all: bool,
    },
    /// Show session storage information
    Info {
        /// Session ID to show details for (optional, shows storage overview if not provided)
        session_id: Option<String>,
    },
}

/// Execute session command
pub async fn execute(cmd: SessionCommand) -> Result<()> {
    match cmd {
        SessionCommand::Search {
            query,
            agent,
            date,
            model,
            json,
        } => search_command(&query, agent.as_deref(), date.as_deref(), model.as_deref(), json).await,
        SessionCommand::Export {
            session_id,
            format,
            include_analytics,
        } => {
            export_command(&session_id, format.as_deref(), include_analytics).await
        }
        SessionCommand::Delete {
            session_id,
            before,
            all,
        } => delete_command(session_id.as_deref(), before.as_deref(), all).await,
        SessionCommand::Info { session_id } => info_command(session_id.as_deref()).await,
    }
}

/// Search sessions by content and metadata
async fn search_command(
    query: &str,
    agent: Option<&str>,
    date: Option<&str>,
    model: Option<&str>,
    json: bool,
) -> Result<()> {
    let workspace = Workspace::discover()
        .context("No Radium workspace found. Run 'rad init' to create one.")?;

    // Initialize history manager
    let history_dir = workspace.root().join(".radium/_internals/history");
    let history = HistoryManager::new(&history_dir)?;

    // Initialize session storage
    let storage = SessionStorage::new(workspace.root())?;

    // Load session metadata for efficient filtering
    let metadata_list = storage.list_report_metadata()?;

    // Filter by date first (if provided)
    let date_filtered: Vec<_> = if let Some(date_str) = date {
        // Parse date (expecting ISO 8601 format like "2025-12-01" or full datetime)
        let filter_date = chrono::DateTime::parse_from_rfc3339(date_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
            })
            .context(format!("Invalid date format: {}. Use ISO 8601 or YYYY-MM-DD", date_str))?;

        metadata_list
            .into_iter()
            .filter(|m| m.generated_at.date_naive() <= filter_date.date_naive())
            .collect()
    } else {
        metadata_list
    };

    // Search results structure
    #[derive(Debug, serde::Serialize)]
    struct SearchResult {
        session_id: String,
        timestamp: String,
        agent: Option<String>,
        model: Option<String>,
        snippet: String,
    }

    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    // For each session, check metadata filters and search content
    for metadata in date_filtered {
        // Load full report to check model and agent filters
        let report = match storage.load_report(&metadata.session_id) {
            Ok(r) => r,
            Err(_) => continue, // Skip if report can't be loaded
        };

        // Filter by model (check if any model in model_usage matches)
        if let Some(model_filter) = model {
            let model_matches = report
                .metrics
                .model_usage
                .keys()
                .any(|m| m.to_lowercase().contains(&model_filter.to_lowercase()));
            if !model_matches {
                continue;
            }
        }

        // Filter by agent (try to extract from session_id pattern: "agent-id_timestamp")
        if let Some(agent_filter) = agent {
            let session_id_lower = metadata.session_id.to_lowercase();
            let agent_filter_lower = agent_filter.to_lowercase();
            if !session_id_lower.contains(&agent_filter_lower) {
                continue;
            }
        }

        // Search content in interactions
        let interactions = history.get_interactions(Some(&metadata.session_id));
        let mut found_match = false;
        let mut matching_snippet = String::new();

        for interaction in &interactions {
            // Search in goal, plan, and output fields
            let searchable_text = format!(
                "{} {} {}",
                interaction.goal, interaction.plan, interaction.output
            )
            .to_lowercase();

            if searchable_text.contains(&query_lower) {
                found_match = true;
                // Extract snippet (first 100 chars of matching text)
                let snippet_source = format!(
                    "{} {} {}",
                    interaction.goal, interaction.plan, interaction.output
                );
                matching_snippet = if snippet_source.len() > 100 {
                    format!("{}...", &snippet_source[..100])
                } else {
                    snippet_source
                };
                break;
            }
        }

        if found_match {
            // Get primary model (most used model)
            let primary_model = report
                .metrics
                .model_usage
                .iter()
                .max_by_key(|(_, stats)| stats.requests)
                .map(|(model, _)| model.clone());

            // Try to extract agent from session_id
            let agent_name = if let Some(underscore_pos) = metadata.session_id.find('_') {
                Some(metadata.session_id[..underscore_pos].to_string())
            } else {
                None
            };

            results.push(SearchResult {
                session_id: metadata.session_id.clone(),
                timestamp: metadata.generated_at.to_rfc3339(),
                agent: agent_name,
                model: primary_model,
                snippet: matching_snippet,
            });
        }
    }

    // Sort by timestamp (most recent first)
    results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Output results
    if json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        if results.is_empty() {
            println!("No sessions found matching query: {}", query);
        } else {
            println!("\nFound {} session(s):\n", results.len());
            println!(
                "{:<30} {:<20} {:<20} {:<25} {}",
                "Session ID", "Timestamp", "Agent", "Model", "Snippet"
            );
            println!("{}", "-".repeat(120));
            for result in &results {
                let agent_str = result.agent.as_deref().unwrap_or("N/A");
                let model_str = result.model.as_deref().unwrap_or("N/A");
                println!(
                    "{:<30} {:<20} {:<20} {:<25} {}",
                    result.session_id,
                    result.timestamp,
                    agent_str,
                    model_str,
                    result.snippet
                );
            }
            println!();
        }
    }

    Ok(())
}

/// Export session transcript
async fn export_command(
    session_id: &str,
    format: Option<&str>,
    include_analytics: bool,
) -> Result<()> {
    let workspace = Workspace::discover()
        .context("No Radium workspace found. Run 'rad init' to create one.")?;

    // Initialize history manager
    let history_dir = workspace.root().join(".radium/_internals/history");
    let history = HistoryManager::new(&history_dir)?;

    // Get interactions for this session
    let interactions = history.get_interactions(Some(session_id));
    if interactions.is_empty() {
        return Err(anyhow::anyhow!("Session '{}' not found", session_id));
    }

    // Load analytics if requested
    let analytics = if include_analytics {
        let storage = SessionStorage::new(workspace.root())?;
        storage.load_report(session_id).ok()
    } else {
        None
    };

    // Determine output format (default: json)
    let output_format = format.unwrap_or("json").to_lowercase();
    if output_format != "json" && output_format != "markdown" {
        return Err(anyhow::anyhow!(
            "Invalid format: {}. Use 'json' or 'markdown'",
            output_format
        ));
    }

    // Generate output content
    let content = if output_format == "json" {
        // JSON format
        #[derive(Serialize)]
        struct ExportData {
            session_id: String,
            interactions: Vec<InteractionExport>,
            analytics: Option<SessionReport>,
        }

        #[derive(Serialize)]
        struct InteractionExport {
            goal: String,
            plan: String,
            output: String,
            timestamp: String,
        }

        let export_data = ExportData {
            session_id: session_id.to_string(),
            interactions: interactions
                .iter()
                .map(|i| InteractionExport {
                    goal: i.goal.clone(),
                    plan: i.plan.clone(),
                    output: i.output.clone(),
                    timestamp: i.timestamp.to_rfc3339(),
                })
                .collect(),
            analytics: analytics,
        };

        serde_json::to_string_pretty(&export_data)?
    } else {
        // Markdown format
        let mut md = String::new();
        md.push_str(&format!("# Session: {}\n\n", session_id));
        md.push_str(&format!("**Generated:** {}\n\n", chrono::Utc::now().to_rfc3339()));
        md.push_str("---\n\n");

        // Interactions
        md.push_str("## Interactions\n\n");
        for (i, interaction) in interactions.iter().enumerate() {
            md.push_str(&format!("### Interaction {}\n\n", i + 1));
            md.push_str(&format!("**Timestamp:** {}\n\n", interaction.timestamp.to_rfc3339()));
            md.push_str(&format!("**Goal:** {}\n\n", interaction.goal));
            md.push_str(&format!("**Plan:** {}\n\n", interaction.plan));
            md.push_str(&format!("**Output:**\n\n{}\n\n", interaction.output));
            md.push_str("---\n\n");
        }

        // Analytics section if included
        if let Some(ref report) = analytics {
            md.push_str("## Analytics\n\n");
            md.push_str(&format!("**Session ID:** {}\n\n", report.metrics.session_id));
            md.push_str(&format!("**Start Time:** {}\n\n", report.metrics.start_time.to_rfc3339()));
            if let Some(end_time) = report.metrics.end_time {
                md.push_str(&format!("**End Time:** {}\n\n", end_time.to_rfc3339()));
            }
            md.push_str(&format!("**Wall Time:** {:.2}s\n\n", report.metrics.wall_time.as_secs_f64()));
            md.push_str(&format!("**Tool Calls:** {}\n\n", report.metrics.tool_calls));
            md.push_str(&format!("**Success Rate:** {:.1}%\n\n", report.metrics.success_rate()));
            md.push_str(&format!("**Total Cost:** ${:.4}\n\n", report.metrics.total_cost));
            
            if !report.metrics.model_usage.is_empty() {
                md.push_str("### Model Usage\n\n");
                md.push_str("| Model | Requests | Input Tokens | Output Tokens | Cost |\n");
                md.push_str("|-------|----------|--------------|---------------|------|\n");
                for (model, stats) in &report.metrics.model_usage {
                    md.push_str(&format!(
                        "| {} | {} | {} | {} | ${:.4} |\n",
                        model, stats.requests, stats.input_tokens, stats.output_tokens, stats.estimated_cost
                    ));
                }
                md.push_str("\n");
            }
        }

        md
    };

    // Write to file using atomic write pattern
    let filename = format!("{}.{}", session_id, output_format);
    let file_path = std::env::current_dir()?.join(&filename);

    // Atomic write: write to temp file, then rename
    let temp_filename = format!("{}.tmp.{}", session_id, uuid::Uuid::new_v4());
    let temp_path = std::env::current_dir()?.join(&temp_filename);
    
    fs::write(&temp_path, content)
        .context("Failed to write export file")?;
    
    fs::rename(&temp_path, &file_path)
        .context("Failed to rename temp file to final destination")?;

    println!("Exported session '{}' to {}", session_id, file_path.display());
    if include_analytics {
        println!("Analytics data included in export.");
    }

    Ok(())
}

/// Delete sessions
async fn delete_command(
    session_id: Option<&str>,
    before: Option<&str>,
    all: bool,
) -> Result<()> {
    let workspace = Workspace::discover()
        .context("No Radium workspace found. Run 'rad init' to create one.")?;

    // Initialize history manager and storage
    let history_dir = workspace.root().join(".radium/_internals/history");
    let mut history = HistoryManager::new(&history_dir)?;
    let storage = SessionStorage::new(workspace.root())?;

    // Validate that exactly one deletion mode is specified
    let mode_count = [session_id.is_some(), before.is_some(), all].iter().filter(|&&x| x).count();
    if mode_count != 1 {
        return Err(anyhow::anyhow!(
            "Exactly one deletion mode must be specified: session_id, --before, or --all"
        ));
    }

    let mut deleted_count = 0;

    if let Some(sid) = session_id {
        // Single session deletion
        // Remove from history
        history.clear_session(Some(sid))?;
        
        // Delete analytics file
        let sessions_dir = storage.sessions_dir();
        let file_path = sessions_dir.join(format!("{}.json", sid));
        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }
        
        deleted_count = 1;
        println!("Deleted session: {}", sid);
    } else if let Some(before_date_str) = before {
        // Batch deletion by date
        let before_date = chrono::DateTime::parse_from_rfc3339(before_date_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(before_date_str, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
            })
            .context(format!("Invalid date format: {}. Use ISO 8601 or YYYY-MM-DD", before_date_str))?;

        // Get all sessions and filter by date
        let metadata_list = storage.list_report_metadata()?;
        let sessions_to_delete: Vec<_> = metadata_list
            .into_iter()
            .filter(|m| m.generated_at.date_naive() < before_date.date_naive())
            .collect();

        let sessions_dir = storage.sessions_dir();
        for metadata in &sessions_to_delete {
            // Remove from history
            history.clear_session(Some(&metadata.session_id))?;
            
            // Delete analytics file
            let file_path = sessions_dir.join(format!("{}.json", metadata.session_id));
            if file_path.exists() {
                let _ = fs::remove_file(&file_path);
            }
        }

        deleted_count = sessions_to_delete.len();
        println!("Deleted {} session(s) before {}", deleted_count, before_date_str);
    } else if all {
        // Delete all sessions with confirmation
        print!("Delete all sessions? This cannot be undone. (y/N): ");
        io::stdout().flush()?;
        
        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;
        let confirmation = confirmation.trim().to_lowercase();
        
        if confirmation == "y" || confirmation == "yes" {
            // Get all sessions
            let metadata_list = storage.list_report_metadata()?;
            let sessions_dir = storage.sessions_dir();
            
            for metadata in &metadata_list {
                // Remove from history
                let _ = history.clear_session(Some(&metadata.session_id));
                
                // Delete analytics file
                let file_path = sessions_dir.join(format!("{}.json", metadata.session_id));
                if file_path.exists() {
                    let _ = fs::remove_file(&file_path);
                }
            }

            deleted_count = metadata_list.len();
            println!("Deleted all {} session(s)", deleted_count);
        } else {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    Ok(())
}

/// Show session info
async fn info_command(session_id: Option<&str>) -> Result<()> {
    let workspace = Workspace::discover()
        .context("No Radium workspace found. Run 'rad init' to create one.")?;

    let storage = SessionStorage::new(workspace.root())?;
    let sessions_dir = storage.sessions_dir();

    if let Some(sid) = session_id {
        // Session detail mode
        let report = storage.load_report(sid)
            .context(format!("Session '{}' not found", sid))?;

        // Get interaction count from history
        let history_dir = workspace.root().join(".radium/_internals/history");
        let history = HistoryManager::new(&history_dir)?;
        let interactions = history.get_interactions(Some(sid));
        let interaction_count = interactions.len();
        let last_interaction = interactions.last().map(|i| i.timestamp);

        // Get primary model (most used)
        let primary_model = report
            .metrics
            .model_usage
            .iter()
            .max_by_key(|(_, stats)| stats.requests)
            .map(|(model, _)| model.clone());

        // Format duration
        let duration_str = format_duration(report.metrics.wall_time);

        // Display session details
        println!("\nSession Information");
        println!("{}", "=".repeat(60));
        println!("Session ID:        {}", report.metrics.session_id);
        println!("Generated:         {}", report.generated_at.to_rfc3339());
        println!("Start Time:        {}", report.metrics.start_time.to_rfc3339());
        if let Some(end_time) = report.metrics.end_time {
            println!("End Time:          {}", end_time.to_rfc3339());
        }
        println!("Duration:          {}", duration_str);
        println!("Interactions:      {}", interaction_count);
        if let Some(last) = last_interaction {
            println!("Last Interaction:  {}", last.to_rfc3339());
        }
        if let Some(model) = primary_model {
            println!("Primary Model:     {}", model);
        }
        println!("Tool Calls:        {}", report.metrics.tool_calls);
        println!("Success Rate:       {:.1}%", report.metrics.success_rate());
        
        // Token and cost info
        let (input_tokens, output_tokens) = report.metrics.total_tokens();
        println!("Input Tokens:      {}", input_tokens);
        println!("Output Tokens:     {}", output_tokens);
        println!("Total Cost:        ${:.4}", report.metrics.total_cost);

        // File paths
        let history_file = history_dir.join("history.json");
        let analytics_file = sessions_dir.join(format!("{}.json", sid));
        println!("\nFile Paths:");
        println!("  History:         {}", history_file.display());
        println!("  Analytics:       {}", analytics_file.display());
        println!();
    } else {
        // Storage overview mode
        let metadata_list = storage.list_report_metadata()?;
        let session_count = metadata_list.len();

        // Calculate total size
        let mut total_size: u64 = 0;
        if sessions_dir.exists() {
            for entry in fs::read_dir(sessions_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = fs::metadata(&path) {
                        total_size += metadata.len();
                    }
                }
            }
        }

        let size_str = format_file_size(total_size);

        println!("\nSession Storage Information");
        println!("{}", "=".repeat(60));
        println!("Storage Directory: {}", sessions_dir.display());
        println!("Total Size:         {}", size_str);
        println!("Session Count:      {}", session_count);
        println!();
    }

    Ok(())
}

/// Format file size in human-readable format
fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Format duration in human-readable format
fn format_duration(d: std::time::Duration) -> String {
    let secs = d.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}


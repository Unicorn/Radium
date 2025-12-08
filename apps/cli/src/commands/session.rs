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
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
            })
            .context(format!("Invalid date format: {}. Use ISO 8601 or YYYY-MM-DD", date_str))?
            .with_timezone(&chrono::Utc);

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
    // Stub implementation - will be implemented in Task 3
    println!("Export command: session_id={}, format={:?}, include_analytics={}", 
        session_id, format, include_analytics);
    Ok(())
}

/// Delete sessions
async fn delete_command(
    session_id: Option<&str>,
    before: Option<&str>,
    all: bool,
) -> Result<()> {
    // Stub implementation - will be implemented in Task 4
    println!("Delete command: session_id={:?}, before={:?}, all={}", 
        session_id, before, all);
    Ok(())
}

/// Show session info
async fn info_command(session_id: Option<&str>) -> Result<()> {
    // Stub implementation - will be implemented in Task 5
    println!("Info command: session_id={:?}", session_id);
    Ok(())
}


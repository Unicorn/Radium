//! Checkpoint management commands for restoring agent work snapshots.

use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use radium_core::checkpoint::CheckpointManager;
use radium_core::workspace::Workspace;

/// Checkpoint subcommands
#[derive(Subcommand, Debug)]
pub enum CheckpointCommand {
    /// List all checkpoints
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Restore a checkpoint
    Restore {
        /// Checkpoint ID to restore
        checkpoint_id: String,
        /// Optional file path to restore (if provided, only this file will be restored)
        #[arg(long)]
        file: Option<String>,
    },
    /// Clean up old checkpoints
    Cleanup {
        /// Number of most recent checkpoints to keep
        #[arg(long, default_value = "10")]
        keep: usize,
    },
    /// Show checkpoint repository information
    Info,
    /// Show diff between two checkpoints
    Diff {
        /// Source checkpoint ID
        from_id: String,
        /// Target checkpoint ID
        to_id: String,
    },
    /// Show changes in a checkpoint (from previous checkpoint)
    Show {
        /// Checkpoint ID to show
        checkpoint_id: String,
    },
    /// View or update checkpoint expiration policy
    Policy {
        /// Show current policy (default if no flags)
        #[arg(long)]
        show: bool,
        /// Set age-based expiration in days (0 to disable)
        #[arg(long)]
        age_days: Option<u32>,
        /// Set maximum size in GB (0 to disable)
        #[arg(long)]
        max_size_gb: Option<f64>,
        /// Set minimum number of checkpoints to keep
        #[arg(long)]
        min_keep: Option<usize>,
    },
}

/// Execute checkpoint command
pub async fn execute(cmd: CheckpointCommand) -> Result<()> {
    let workspace = Workspace::discover()
        .context("No Radium workspace found. Run 'rad init' to create one.")?;

    let checkpoint_manager = CheckpointManager::new(workspace.root())
        .context("Workspace is not a git repository. Checkpoints require git.")?;

    match cmd {
        CheckpointCommand::List { json } => list_command(&checkpoint_manager, json).await,
        CheckpointCommand::Restore { checkpoint_id, file } => {
            restore_command(&checkpoint_manager, &checkpoint_id, file.as_deref()).await
        }
        CheckpointCommand::Cleanup { keep } => cleanup_command(&checkpoint_manager, keep).await,
        CheckpointCommand::Info => info_command(&checkpoint_manager).await,
        CheckpointCommand::Diff { from_id, to_id } => {
            diff_command(&checkpoint_manager, &from_id, &to_id).await
        }
        CheckpointCommand::Show { checkpoint_id } => {
            show_command(&checkpoint_manager, &checkpoint_id).await
        }
        CheckpointCommand::Policy { show, age_days, max_size_gb, min_keep } => {
            policy_command(&checkpoint_manager, show, age_days, max_size_gb, min_keep).await
        }
    }
}

async fn list_command(checkpoint_manager: &CheckpointManager, json: bool) -> Result<()> {
    let checkpoints = checkpoint_manager.list_checkpoints()?;

    if json {
        let json_checkpoints: Vec<serde_json::Value> = checkpoints
            .iter()
            .map(|c| {
                serde_json::json!({
                    "id": c.id,
                    "commit_hash": c.commit_hash,
                    "agent_id": c.agent_id,
                    "timestamp": c.timestamp,
                    "description": c.description,
                    "execution_duration_secs": c.execution_duration_secs,
                    "memory_usage_mb": c.memory_usage_mb,
                    "cpu_time_secs": c.cpu_time_secs,
                    "tokens_used": c.tokens_used,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_checkpoints)?);
    } else {
        if checkpoints.is_empty() {
            println!("No checkpoints found.");
        } else {
            // Check if any checkpoint has resource metadata
            let has_metadata = checkpoints.iter().any(|c| {
                c.execution_duration_secs.is_some()
                    || c.memory_usage_mb.is_some()
                    || c.cpu_time_secs.is_some()
                    || c.tokens_used.is_some()
            });
            
            if has_metadata {
                // Extended format with resource metadata
                println!("{:<40} {:<15} {:<25} {:<12} {:<12} {:<12} {:<15}", 
                    "ID", "Commit", "Description", "Duration", "Memory", "Tokens", "Timestamp");
                println!("{}", "-".repeat(131));
                for checkpoint in checkpoints {
                    let timestamp = chrono::DateTime::from_timestamp(checkpoint.timestamp as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| checkpoint.timestamp.to_string());
                    let description = checkpoint.description.as_deref().unwrap_or("-");
                    let description_short = if description.len() > 23 {
                        &description[..20]
                    } else {
                        description
                    };
                    let commit_short =
                        &checkpoint.commit_hash[..std::cmp::min(12, checkpoint.commit_hash.len())];
                    let duration = checkpoint.execution_duration_secs
                        .map(|s| format_duration(s))
                        .unwrap_or_else(|| "-".to_string());
                    let memory = checkpoint.memory_usage_mb
                        .map(|m| format!("{:.1} MB", m))
                        .unwrap_or_else(|| "-".to_string());
                    let tokens = checkpoint.tokens_used
                        .map(|t| format_number(t))
                        .unwrap_or_else(|| "-".to_string());
                    println!(
                        "{:<40} {:<15} {:<25} {:<12} {:<12} {:<12} {:<15}",
                        checkpoint.id, commit_short, description_short, duration, memory, tokens, timestamp
                    );
                }
            } else {
                // Simple format (no metadata)
                println!("{:<40} {:<15} {:<30} {:<20}", "ID", "Commit", "Description", "Timestamp");
                println!("{}", "-".repeat(105));
                for checkpoint in checkpoints {
                    let timestamp = chrono::DateTime::from_timestamp(checkpoint.timestamp as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| checkpoint.timestamp.to_string());
                    let description = checkpoint.description.as_deref().unwrap_or("-");
                    let commit_short =
                        &checkpoint.commit_hash[..std::cmp::min(12, checkpoint.commit_hash.len())];
                    println!(
                        "{:<40} {:<15} {:<30} {:<20}",
                        checkpoint.id, commit_short, description, timestamp
                    );
                }
            }
        }
    }
    Ok(())
}

/// Formats duration in seconds to human-readable string.
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{}m {}s", mins, secs)
    } else {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;
        format!("{}h {}m {}s", hours, mins, secs)
    }
}

/// Formats number with thousand separators.
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut count = 0;
    for ch in s.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
        count += 1;
    }
    result.chars().rev().collect()
    Ok(())
}

async fn restore_command(
    checkpoint_manager: &CheckpointManager,
    checkpoint_id: &str,
    file_path: Option<&str>,
) -> Result<()> {
    if let Some(file) = file_path {
        // Selective file restore
        println!("Restoring file '{}' from checkpoint: {}", file, checkpoint_id);

        match checkpoint_manager.restore_file_from_checkpoint(checkpoint_id, file) {
            Ok(true) => {
                println!("✓ File restored successfully.");
                println!("  File '{}' has been restored to its state in checkpoint '{}'.", file, checkpoint_id);
            }
            Ok(false) => {
                println!("✓ File already up-to-date.");
                println!("  File '{}' matches the checkpoint version, no changes needed.", file);
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to restore file: {}", e));
            }
        }
    } else {
        // Full workspace restore
        println!("Restoring checkpoint: {}", checkpoint_id);

        checkpoint_manager
            .restore_checkpoint(checkpoint_id)
            .context(format!("Failed to restore checkpoint: {}", checkpoint_id))?;

        println!("✓ Checkpoint restored successfully.");
        println!("  Your workspace has been restored to the state at this checkpoint.");
        println!(
            "  Note: You may need to re-propose tool calls if you were in the middle of agent execution."
        );
    }

    Ok(())
}

async fn cleanup_command(checkpoint_manager: &CheckpointManager, keep: usize) -> Result<()> {
    let all_checkpoints = checkpoint_manager.list_checkpoints()?;
    let total_count = all_checkpoints.len();

    if total_count <= keep {
        println!("No cleanup needed. You have {} checkpoint(s), keeping {}.", total_count, keep);
        return Ok(());
    }

    println!("Cleaning up old checkpoints (keeping {} most recent)...", keep);
    let deleted_count = checkpoint_manager
        .cleanup_old_checkpoints(keep)
        .context("Failed to clean up checkpoints")?;

    if deleted_count > 0 {
        println!("✓ Cleaned up {} checkpoint(s).", deleted_count);
        println!("  Kept {} most recent checkpoint(s).", keep);
        println!("  Git garbage collection completed.");
    } else {
        println!("No checkpoints were deleted.");
    }

    Ok(())
}

async fn info_command(checkpoint_manager: &CheckpointManager) -> Result<()> {
    let checkpoints = checkpoint_manager.list_checkpoints()?;
    let repo_size = checkpoint_manager.get_shadow_repo_size();

    println!("Checkpoint Repository Information");
    println!("{}", "=".repeat(40));
    println!("Total checkpoints: {}", checkpoints.len());
    
    // Format size
    let size_mb = repo_size as f64 / (1024.0 * 1024.0);
    if size_mb < 1.0 {
        println!("Repository size: {} bytes", repo_size);
    } else {
        println!("Repository size: {:.2} MB ({})", size_mb, repo_size);
    }

    if !checkpoints.is_empty() {
        println!("\nMost recent checkpoints:");
        for (i, checkpoint) in checkpoints.iter().take(5).enumerate() {
            let timestamp = chrono::DateTime::from_timestamp(checkpoint.timestamp as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| checkpoint.timestamp.to_string());
            let description = checkpoint.description.as_deref().unwrap_or("-");
            println!("  {}. {} - {}", i + 1, checkpoint.id, description);
            println!("     Created: {}", timestamp);
        }
        if checkpoints.len() > 5 {
            println!("  ... and {} more", checkpoints.len() - 5);
        }
    }

    Ok(())
}

async fn diff_command(
    checkpoint_manager: &CheckpointManager,
    from_id: &str,
    to_id: &str,
) -> Result<()> {
    println!("Diff between checkpoints:");
    println!("  From: {}", from_id);
    println!("  To:   {}", to_id);
    println!();

    let diff = checkpoint_manager
        .diff_checkpoints(from_id, to_id)
        .context(format!("Failed to diff checkpoints: {} -> {}", from_id, to_id))?;

    // Display statistics
    println!("Statistics:");
    println!("  Files changed: {}", diff.files_changed());
    println!("  Files added:    {}", diff.added.len().to_string().green());
    println!("  Files modified: {}", diff.modified.len().to_string().yellow());
    println!("  Files deleted:  {}", diff.deleted.len().to_string().red());
    println!("  Insertions:     {}", diff.insertions.to_string().green());
    println!("  Deletions:      {}", diff.deletions.to_string().red());
    println!();

    // Display file changes
    if !diff.added.is_empty() {
        println!("{}", "Added files:".green().bold());
        for file in &diff.added {
            println!("  + {}", file.green());
        }
        println!();
    }

    if !diff.modified.is_empty() {
        println!("{}", "Modified files:".yellow().bold());
        for file in &diff.modified {
            println!("  ~ {}", file.yellow());
        }
        println!();
    }

    if !diff.deleted.is_empty() {
        println!("{}", "Deleted files:".red().bold());
        for file in &diff.deleted {
            println!("  - {}", file.red());
        }
        println!();
    }

    if diff.files_changed() == 0 {
        println!("No changes between checkpoints.");
    }

    Ok(())
}

async fn show_command(
    checkpoint_manager: &CheckpointManager,
    checkpoint_id: &str,
) -> Result<()> {
    let checkpoints = checkpoint_manager.list_checkpoints()?;

    // Find the checkpoint and its position
    let current_idx = checkpoints
        .iter()
        .position(|cp| cp.id == checkpoint_id)
        .context(format!("Checkpoint not found: {}", checkpoint_id))?;

    if current_idx == 0 {
        println!("This is the most recent checkpoint. No previous checkpoint to compare against.");
        return Ok(());
    }

    // Get the previous checkpoint
    let previous_id = &checkpoints[current_idx - 1].id;

    println!("Showing changes in checkpoint: {}", checkpoint_id);
    println!("Comparing with previous checkpoint: {}", previous_id);
    println!();

    let diff = checkpoint_manager
        .diff_checkpoints(previous_id, checkpoint_id)
        .context(format!(
            "Failed to diff checkpoints: {} -> {}",
            previous_id, checkpoint_id
        ))?;

    // Display statistics
    println!("Statistics:");
    println!("  Files changed: {}", diff.files_changed());
    println!("  Files added:    {}", diff.added.len().to_string().green());
    println!("  Files modified: {}", diff.modified.len().to_string().yellow());
    println!("  Files deleted:  {}", diff.deleted.len().to_string().red());
    println!("  Insertions:     {}", diff.insertions.to_string().green());
    println!("  Deletions:      {}", diff.deletions.to_string().red());
    println!();

    // Display file changes
    if !diff.added.is_empty() {
        println!("{}", "Added files:".green().bold());
        for file in &diff.added {
            println!("  + {}", file.green());
        }
        println!();
    }

    if !diff.modified.is_empty() {
        println!("{}", "Modified files:".yellow().bold());
        for file in &diff.modified {
            println!("  ~ {}", file.yellow());
        }
        println!();
    }

    if !diff.deleted.is_empty() {
        println!("{}", "Deleted files:".red().bold());
        for file in &diff.deleted {
            println!("  - {}", file.red());
        }
        println!();
    }

    if diff.files_changed() == 0 {
        println!("No changes from previous checkpoint.");
    }

    Ok(())
}

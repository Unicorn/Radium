//! Checkpoint management commands for restoring agent work snapshots.

use anyhow::{Context, Result};
use clap::Subcommand;
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
    },
    /// Clean up old checkpoints
    Cleanup {
        /// Number of most recent checkpoints to keep
        #[arg(long, default_value = "10")]
        keep: usize,
    },
    /// Show checkpoint repository information
    Info,
}

/// Execute checkpoint command
pub async fn execute(cmd: CheckpointCommand) -> Result<()> {
    let workspace = Workspace::discover()
        .context("No Radium workspace found. Run 'rad init' to create one.")?;

    let checkpoint_manager = CheckpointManager::new(workspace.root())
        .context("Workspace is not a git repository. Checkpoints require git.")?;

    match cmd {
        CheckpointCommand::List { json } => list_command(&checkpoint_manager, json).await,
        CheckpointCommand::Restore { checkpoint_id } => {
            restore_command(&checkpoint_manager, &checkpoint_id).await
        }
        CheckpointCommand::Cleanup { keep } => cleanup_command(&checkpoint_manager, keep).await,
        CheckpointCommand::Info => info_command(&checkpoint_manager).await,
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
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_checkpoints)?);
    } else {
        if checkpoints.is_empty() {
            println!("No checkpoints found.");
        } else {
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
    Ok(())
}

async fn restore_command(
    checkpoint_manager: &CheckpointManager,
    checkpoint_id: &str,
) -> Result<()> {
    println!("Restoring checkpoint: {}", checkpoint_id);

    checkpoint_manager
        .restore_checkpoint(checkpoint_id)
        .context(format!("Failed to restore checkpoint: {}", checkpoint_id))?;

    println!("✓ Checkpoint restored successfully.");
    println!("  Your workspace has been restored to the state at this checkpoint.");
    println!(
        "  Note: You may need to re-propose tool calls if you were in the middle of agent execution."
    );

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

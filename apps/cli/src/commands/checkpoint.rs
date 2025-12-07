//! Checkpoint management commands for restoring agent work snapshots.

use anyhow::{Context, Result};
use clap::Subcommand;
use radium_core::checkpoint::{Checkpoint, CheckpointManager};
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
    }
}

async fn list_command(checkpoint_manager: &CheckpointManager, json: bool) -> Result<()> {
    let checkpoints = checkpoint_manager.list_checkpoints()?;

    if json {
        let json_checkpoints: Vec<serde_json::Value> = checkpoints.iter().map(|c| {
            serde_json::json!({
                "id": c.id,
                "commit_hash": c.commit_hash,
                "agent_id": c.agent_id,
                "timestamp": c.timestamp,
                "description": c.description,
            })
        }).collect();
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
                let commit_short = &checkpoint.commit_hash[..std::cmp::min(12, checkpoint.commit_hash.len())];
                println!(
                    "{:<40} {:<15} {:<30} {:<20}",
                    checkpoint.id,
                    commit_short,
                    description,
                    timestamp
                );
            }
        }
    }
    Ok(())
}

async fn restore_command(checkpoint_manager: &CheckpointManager, checkpoint_id: &str) -> Result<()> {
    println!("Restoring checkpoint: {}", checkpoint_id);
    
    checkpoint_manager.restore_checkpoint(checkpoint_id)
        .context(format!("Failed to restore checkpoint: {}", checkpoint_id))?;
    
    println!("✓ Checkpoint restored successfully.");
    println!("  Your workspace has been restored to the state at this checkpoint.");
    println!("  Note: You may need to re-propose tool calls if you were in the middle of agent execution.");
    
    Ok(())
}


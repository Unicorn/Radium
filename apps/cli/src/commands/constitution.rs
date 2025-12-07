//! Constitution management commands.

use clap::Subcommand;
use radium_core::policy::ConstitutionManager;
use std::sync::Arc;

/// Constitution command options.
#[derive(Subcommand, Debug)]
pub enum ConstitutionCommand {
    /// Add or update a rule for a session
    Update {
        /// Session ID
        session_id: String,
        /// Rule text
        rule: String,
    },

    /// Reset all rules for a session
    Reset {
        /// Session ID
        session_id: String,
    },

    /// Get rules for a session
    Get {
        /// Session ID
        session_id: String,
    },

    /// List all active sessions
    List,
}

/// Execute constitution command.
pub async fn execute_constitution_command(command: ConstitutionCommand) -> anyhow::Result<()> {
    let manager = Arc::new(ConstitutionManager::new());

    match command {
        ConstitutionCommand::Update { session_id, rule } => {
            update_constitution(manager, session_id, rule).await
        }
        ConstitutionCommand::Reset { session_id } => reset_constitution(manager, session_id).await,
        ConstitutionCommand::Get { session_id } => get_constitution(manager, session_id).await,
        ConstitutionCommand::List => list_constitutions(manager).await,
    }
}

/// Update constitution for a session.
async fn update_constitution(
    manager: Arc<ConstitutionManager>,
    session_id: String,
    rule: String,
) -> anyhow::Result<()> {
    manager.update_constitution(&session_id, rule.clone());
    
    let rules = manager.get_constitution(&session_id);
    println!("✓ Added rule to session '{}'", session_id);
    println!("  Rule: {}", rule);
    println!("  Total rules: {}", rules.len());

    Ok(())
}

/// Reset constitution for a session.
async fn reset_constitution(
    manager: Arc<ConstitutionManager>,
    session_id: String,
) -> anyhow::Result<()> {
    use std::io::{self, Write};
    
    print!("Reset all rules for session '{}'? (y/N): ", session_id);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();
    if input != "y" && input != "yes" {
        println!("Cancelled.");
        return Ok(());
    }

    manager.reset_constitution(&session_id, vec![]);
    println!("✓ Reset all rules for session '{}'", session_id);

    Ok(())
}

/// Get constitution rules for a session.
async fn get_constitution(
    manager: Arc<ConstitutionManager>,
    session_id: String,
) -> anyhow::Result<()> {
    let rules = manager.get_constitution(&session_id);

    if rules.is_empty() {
        println!("No rules found for session '{}'", session_id);
        return Ok(());
    }

    println!("Constitution Rules for Session: {}", session_id);
    println!("==========================================");
    for (i, rule) in rules.iter().enumerate() {
        println!("{}. {}", i + 1, rule);
    }
    println!();
    println!("Total rules: {}", rules.len());

    Ok(())
}

/// List all active sessions.
async fn list_constitutions(manager: Arc<ConstitutionManager>) -> anyhow::Result<()> {
    // Note: ConstitutionManager doesn't currently have a method to list all sessions
    // This is a limitation - sessions are tracked internally but not exposed
    // For now, we'll inform the user about this limitation
    println!("Constitution Sessions");
    println!("====================");
    println!();
    println!("Note: Session listing is not currently available.");
    println!("Sessions are automatically tracked when rules are added.");
    println!("Use 'rad constitution get <session-id>' to check a specific session.");
    println!();
    println!("Sessions are automatically cleaned up after 1 hour of inactivity.");

    Ok(())
}


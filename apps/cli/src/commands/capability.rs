//! Capability management commands.

use clap::Subcommand;
use radium_core::agents::{
    capability_manager::{CapabilityManager, ElevationRequest},
    config::{AgentCapabilities, CostTier, ModelClass},
};
use radium_core::policy::constitution::ConstitutionManager;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Capability command options.
#[derive(Subcommand, Debug)]
pub enum CapabilityCommand {
    /// Request capability elevation for an agent
    Request {
        /// Agent ID
        agent_id: String,
        /// Capability to request (e.g., "write", "network", "admin")
        capability: String,
        /// Reason for the elevation request
        #[arg(long)]
        reason: String,
        /// Duration in seconds (default: 3600 = 1 hour)
        #[arg(long, default_value = "3600")]
        duration: u64,
    },
    /// List active capability elevations
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Revoke capability elevation for an agent
    Revoke {
        /// Agent ID
        agent_id: String,
    },
    /// Show elevation history
    History {
        /// Agent ID (optional, shows all if not specified)
        agent_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// Execute capability command.
pub async fn execute_capability_command(command: CapabilityCommand) -> anyhow::Result<()> {
    // Note: In a real implementation, we'd need to initialize the capability manager
    // with a constitution manager. For now, this is a placeholder structure.
    
    match command {
        CapabilityCommand::Request { agent_id, capability, reason, duration } => {
            request_elevation(agent_id, capability, reason, duration).await
        }
        CapabilityCommand::List { json } => list_elevations(json).await,
        CapabilityCommand::Revoke { agent_id } => revoke_elevation(agent_id).await,
        CapabilityCommand::History { agent_id, json } => show_history(agent_id, json).await,
    }
}

/// Request capability elevation.
async fn request_elevation(
    agent_id: String,
    capability: String,
    reason: String,
    duration: u64,
) -> anyhow::Result<()> {
    // Create a basic capability based on the requested capability type
    let requested_capabilities = match capability.as_str() {
        "write" => AgentCapabilities {
            model_class: ModelClass::Balanced,
            cost_tier: CostTier::Medium,
            max_concurrent_tasks: 5,
        },
        "network" => AgentCapabilities {
            model_class: ModelClass::Fast,
            cost_tier: CostTier::Low,
            max_concurrent_tasks: 10,
        },
        "admin" => AgentCapabilities {
            model_class: ModelClass::Reasoning,
            cost_tier: CostTier::High,
            max_concurrent_tasks: 3,
        },
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown capability: {}. Valid options: write, network, admin",
                capability
            ));
        }
    };

    // Create elevation request
    let constitution_manager = Arc::new(ConstitutionManager::new());
    let capability_manager = CapabilityManager::new(constitution_manager);
    
    let request = capability_manager.create_elevation_request(
        agent_id.clone(),
        requested_capabilities,
        reason.clone(),
        Some(duration),
    );

    println!("Capability elevation request created:");
    println!("  Agent ID: {}", request.agent_id);
    println!("  Capability: {}", capability);
    println!("  Reason: {}", request.justification);
    println!("  Duration: {} seconds", duration);
    println!();
    println!("⚠️  This request requires user approval.");
    println!("   In a real implementation, this would prompt for approval.");
    println!("   For now, use 'rad capability grant <agent-id>' to grant elevation.");

    Ok(())
}

/// List active elevations.
async fn list_elevations(json: bool) -> anyhow::Result<()> {
    let constitution_manager = Arc::new(ConstitutionManager::new());
    let capability_manager = CapabilityManager::new(constitution_manager);
    
    let elevations = capability_manager.list_active_elevations().await;

    if json {
        println!("{}", serde_json::to_string_pretty(&elevations)?);
    } else {
        if elevations.is_empty() {
            println!("No active capability elevations.");
        } else {
            println!("Active Capability Elevations:");
            println!("{}", "=".repeat(60));
            for elevation in elevations {
                println!("Agent ID: {}", elevation.agent_id);
                println!("  Justification: {}", elevation.justification);
                println!("  Duration: {:?}", elevation.duration_secs);
                println!("  Created: {}", elevation.created_at);
                println!();
            }
        }
    }

    Ok(())
}

/// Revoke elevation.
async fn revoke_elevation(agent_id: String) -> anyhow::Result<()> {
    let constitution_manager = Arc::new(ConstitutionManager::new());
    let capability_manager = CapabilityManager::new(constitution_manager);
    
    capability_manager.revoke_elevation(&agent_id).await
        .map_err(|e| anyhow::anyhow!("Failed to revoke elevation: {}", e))?;

    println!("✓ Capability elevation revoked for agent: {}", agent_id);
    Ok(())
}

/// Show elevation history.
async fn show_history(agent_id: Option<String>, json: bool) -> anyhow::Result<()> {
    let constitution_manager = Arc::new(ConstitutionManager::new());
    let capability_manager = CapabilityManager::new(constitution_manager);
    
    let history = if let Some(agent_id) = agent_id {
        capability_manager.get_elevation_history(&agent_id).await
    } else {
        capability_manager.get_all_elevation_history().await
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&history)?);
    } else {
        if history.is_empty() {
            println!("No elevation history found.");
        } else {
            println!("Elevation History:");
            println!("{}", "=".repeat(60));
            for record in history {
                println!("Agent ID: {}", record.agent_id);
                println!("  Granted: {}", record.granted_at);
                if let Some(revoked_at) = record.revoked_at {
                    println!("  Revoked: {} ({})", revoked_at, if record.manually_revoked { "manual" } else { "expired" });
                } else {
                    println!("  Status: Active");
                }
                println!("  Justification: {}", record.justification);
                println!();
            }
        }
    }

    Ok(())
}


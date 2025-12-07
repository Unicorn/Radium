//! Engine management command implementation.

use super::EnginesCommand;
use anyhow::{Context, Result};
use colored::Colorize;
use radium_core::engines::{Engine, EngineRegistry};
use radium_core::engines::providers::{ClaudeEngine, GeminiEngine, MockEngine, OpenAIEngine};
use radium_core::workspace::Workspace;
use serde_json::json;
use std::sync::Arc;

/// Execute the engines command.
pub async fn execute(command: EnginesCommand) -> Result<()> {
    match command {
        EnginesCommand::List { json } => list_engines(json).await,
        EnginesCommand::Show { engine_id, json } => show_engine(&engine_id, json).await,
        EnginesCommand::Status { json } => status_engines(json).await,
        EnginesCommand::SetDefault { engine_id } => set_default_engine(&engine_id).await,
    }
}

/// Initialize engine registry with all available engines.
fn init_registry() -> EngineRegistry {
    let registry = EngineRegistry::new();

    // Register all available engines
    let _ = registry.register(Arc::new(MockEngine::new()));
    let _ = registry.register(Arc::new(ClaudeEngine::new()));
    let _ = registry.register(Arc::new(OpenAIEngine::new()));
    let _ = registry.register(Arc::new(GeminiEngine::new()));

    registry
}

/// List all available engines.
async fn list_engines(json_output: bool) -> Result<()> {
    let registry = init_registry();
    let engines = registry.list().context("Failed to list engines")?;

    if json_output {
        let engine_list: Vec<_> = engines
            .iter()
            .map(|metadata| {
                json!({
                    "id": metadata.id,
                    "name": metadata.name,
                    "description": metadata.description,
                    "models": metadata.models,
                    "requires_auth": metadata.requires_auth,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&engine_list)?);
    } else {
        println!();
        println!("{}", format!("🔧 Available Engines ({})", engines.len()).bold().green());
        println!();

        // Table header
        println!("{:<15} {:<20} {:<40} {:<10}", "ID", "Name", "Models", "Auth");
        println!("{}", "─".repeat(85));

        for metadata in &engines {
            let models_str = if metadata.models.len() > 2 {
                format!("{} (+{} more)", metadata.models[0], metadata.models.len() - 1)
            } else {
                metadata.models.join(", ")
            };
            let auth_str = if metadata.requires_auth {
                "Required".yellow()
            } else {
                "None".dimmed()
            };

            println!(
                "{:<15} {:<20} {:<40} {}",
                metadata.id.cyan(),
                metadata.name,
                models_str.dimmed(),
                auth_str
            );
        }

        println!();
    }

    Ok(())
}

/// Show detailed information about a specific engine.
async fn show_engine(engine_id: &str, json_output: bool) -> Result<()> {
    let registry = init_registry();
    let engine = registry.get(engine_id)
        .with_context(|| format!("Engine not found: {}", engine_id))?;

    let metadata = engine.metadata();

    if json_output {
        let engine_info = json!({
            "id": metadata.id,
            "name": metadata.name,
            "description": metadata.description,
            "cli_command": metadata.cli_command,
            "models": metadata.models,
            "requires_auth": metadata.requires_auth,
            "version": metadata.version,
            "default_model": engine.default_model(),
        });
        println!("{}", serde_json::to_string_pretty(&engine_info)?);
    } else {
        println!();
        println!("{}", format!("Engine: {}", metadata.name).bold().cyan());
        println!();
        println!("  ID:          {}", metadata.id.cyan());
        println!("  Name:        {}", metadata.name);
        println!("  Description: {}", metadata.description.dimmed());
        
        if let Some(ref cli_cmd) = metadata.cli_command {
            println!("  CLI Command: {}", cli_cmd.cyan());
        }
        
        println!("  Models:      {}", metadata.models.join(", ").dimmed());
        println!("  Default:     {}", engine.default_model().cyan());
        println!("  Auth:        {}", if metadata.requires_auth {
            "Required".yellow()
        } else {
            "Not required".dimmed()
        });
        
        if let Some(ref version) = metadata.version {
            println!("  Version:     {}", version.dimmed());
        }
        
        println!();
    }

    Ok(())
}

/// Show authentication status for all engines.
async fn status_engines(json_output: bool) -> Result<()> {
    let registry = init_registry();
    let engines = registry.list().context("Failed to list engines")?;

    if json_output {
        let mut status_list = Vec::new();
        for metadata in &engines {
            let engine = registry.get(&metadata.id)?;
            let available = engine.is_available().await;
            let authenticated = engine.is_authenticated().await.unwrap_or(false);

            status_list.push(json!({
                "id": metadata.id,
                "name": metadata.name,
                "available": available,
                "authenticated": authenticated,
            }));
        }
        println!("{}", serde_json::to_string_pretty(&status_list)?);
    } else {
        println!();
        println!("{}", "Engine Status".bold().green());
        println!();

        // Table header
        println!("{:<15} {:<20} {:<15} {:<15}", "ID", "Name", "Available", "Authenticated");
        println!("{}", "─".repeat(65));

        for metadata in &engines {
            let engine = registry.get(&metadata.id)?;
            let available = engine.is_available().await;
            let authenticated = engine.is_authenticated().await.unwrap_or(false);

            let available_str = if available {
                "✓".green()
            } else {
                "✗".red()
            };

            let auth_str = if authenticated {
                "✓".green()
            } else if metadata.requires_auth {
                "✗".red()
            } else {
                "—".dimmed()
            };

            println!(
                "{:<15} {:<20} {:<15} {}",
                metadata.id.cyan(),
                metadata.name,
                available_str,
                auth_str
            );
        }

        println!();
    }

    Ok(())
}

/// Set the default engine for the workspace.
async fn set_default_engine(engine_id: &str) -> Result<()> {
    let registry = init_registry();

    // Verify engine exists
    let engine = registry.get(engine_id)
        .with_context(|| format!("Engine not found: {}", engine_id))?;

    // Set as default
    registry.set_default(engine_id)
        .with_context(|| format!("Failed to set default engine: {}", engine_id))?;

    // Try to save to workspace config (optional, may not have workspace)
    if let Ok(workspace) = Workspace::discover() {
        let config_path = workspace.radium_dir().join("config.toml");
        // For now, just print a message - full config persistence can be added later
        println!();
        println!("{}", format!("✓ Set default engine to: {}", engine_id).green());
        println!("  Note: Default engine preference will be saved to workspace config in a future update.");
        println!();
    } else {
        println!();
        println!("{}", format!("✓ Set default engine to: {}", engine_id).green());
        println!();
    }

    Ok(())
}


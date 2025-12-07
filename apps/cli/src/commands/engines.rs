//! Engine management command implementation.

use super::{EnginesCommand, EngineConfigCommand};
use anyhow::{Context, Result};
use colored::Colorize;
use radium_core::engines::{Engine, EngineRegistry, HealthStatus, PerEngineConfig};
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
        EnginesCommand::Health { json, timeout } => health_engines(json, timeout).await,
        EnginesCommand::Config(cmd) => execute_config_command(cmd).await,
    }
}

/// Initialize engine registry with all available engines.
fn init_registry() -> EngineRegistry {
    // Try to get workspace config path
    let config_path = Workspace::discover()
        .ok()
        .map(|w| w.radium_dir().join("config.toml"));

    let registry = if let Some(ref path) = config_path {
        EngineRegistry::with_config_path(path)
    } else {
        EngineRegistry::new()
    };

    // Register all available engines
    let _ = registry.register(Arc::new(MockEngine::new()));
    let _ = registry.register(Arc::new(ClaudeEngine::new()));
    let _ = registry.register(Arc::new(OpenAIEngine::new()));
    let _ = registry.register(Arc::new(GeminiEngine::new()));

    // Load config after engines are registered
    let _ = registry.load_config();

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

    // Set as default (this will also persist to config)
    registry.set_default(engine_id)
        .with_context(|| format!("Failed to set default engine: {}", engine_id))?;

    println!();
    println!("{}", format!("✓ Set default engine to: {}", engine_id).green());
    if let Ok(workspace) = Workspace::discover() {
        let config_path = workspace.radium_dir().join("config.toml");
        println!("  Saved to: {}", config_path.display().to_string().dimmed());
    }
    println!();

    Ok(())
}

/// Check health of all engines.
async fn health_engines(json_output: bool, timeout: u64) -> Result<()> {
    let registry = init_registry();
    let health_results = registry.check_health(timeout).await;

    if json_output {
        let health_list: Vec<_> = health_results
            .iter()
            .map(|health| {
                let status_str = match &health.status {
                    HealthStatus::Healthy => "healthy",
                    HealthStatus::Warning(msg) => "warning",
                    HealthStatus::Failed(msg) => "failed",
                };
                let status_msg = match &health.status {
                    HealthStatus::Healthy => None,
                    HealthStatus::Warning(msg) | HealthStatus::Failed(msg) => Some(msg.clone()),
                };
                json!({
                    "id": health.engine_id,
                    "name": health.engine_name,
                    "status": status_str,
                    "status_message": status_msg,
                    "available": health.available,
                    "authenticated": health.authenticated,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&health_list)?);
    } else {
        println!();
        println!("{}", format!("Engine Health Check (timeout: {}s)", timeout).bold().green());
        println!();

        // Table header
        println!("{:<15} {:<20} {:<15} {:<15} {:<30}", "ID", "Name", "Status", "Available", "Issue");
        println!("{}", "─".repeat(95));

        for health in &health_results {
            let status_str = match &health.status {
                HealthStatus::Healthy => "✓ Healthy".green(),
                HealthStatus::Warning(msg) => format!("⚠ Warning: {}", msg).yellow(),
                HealthStatus::Failed(msg) => format!("✗ Failed: {}", msg).red(),
            };

            let available_str = if health.available {
                "✓".green()
            } else {
                "✗".red()
            };

            let authenticated_str = if health.authenticated {
                "✓".green()
            } else if !health.available {
                "—".dimmed()
            } else {
                "✗".red()
            };

            // Generate troubleshooting hint
            let issue_hint = match &health.status {
                HealthStatus::Healthy => "—".dimmed(),
                HealthStatus::Warning(msg) => {
                    if msg.contains("Authentication") {
                        format!("Run: rad auth login {}", health.engine_id).dimmed()
                    } else {
                        msg.clone().dimmed()
                    }
                }
                HealthStatus::Failed(msg) => {
                    if msg.contains("not authenticated") {
                        format!("Run: rad auth login {}", health.engine_id).dimmed()
                    } else if msg.contains("not available") {
                        format!("Check API connectivity").dimmed()
                    } else if msg.contains("timed out") {
                        format!("Increase timeout or check network").dimmed()
                    } else {
                        msg.clone().dimmed()
                    }
                }
            };

            println!(
                "{:<15} {:<20} {:<15} {:<15} {}",
                health.engine_id.cyan(),
                health.engine_name,
                status_str,
                format!("{} / {}", available_str, authenticated_str),
                issue_hint
            );
        }

        println!();

        // Summary
        let healthy_count = health_results.iter().filter(|h| matches!(h.status, HealthStatus::Healthy)).count();
        let warning_count = health_results.iter().filter(|h| matches!(h.status, HealthStatus::Warning(_))).count();
        let failed_count = health_results.iter().filter(|h| matches!(h.status, HealthStatus::Failed(_))).count();

        if healthy_count == health_results.len() {
            println!("{}", format!("✓ All engines are healthy ({})", healthy_count).green());
        } else {
            println!("{}", format!("Summary: {} healthy, {} warnings, {} failed", healthy_count, warning_count, failed_count).yellow());
            if failed_count > 0 {
                println!();
                println!("{}", "Troubleshooting:".bold());
                println!("  • Authentication issues: Run 'rad auth login <engine-id>'");
                println!("  • Connectivity issues: Check your network connection and API endpoints");
                println!("  • Timeout issues: Increase timeout with --timeout flag");
            }
        }

        println!();
    }

    Ok(())
}


//! Models command implementation.

use super::types::ModelsCommand;
use anyhow::{Context, Result};
use colored::Colorize;
use radium_core::engines::{
    CredentialStatus, EngineRegistry, ValidationStatus,
};
use radium_core::engines::providers::{ClaudeEngine, GeminiEngine, MockEngine, OpenAIEngine};
use radium_core::workspace::Workspace;
use serde_json::json;
use std::sync::Arc;

/// Execute the models command.
pub async fn execute(command: ModelsCommand) -> Result<()> {
    match command {
        ModelsCommand::List { json } => list_models(json).await,
        ModelsCommand::Test { model_id } => test_model(&model_id).await,
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

/// List all configured models with their status.
async fn list_models(json_output: bool) -> Result<()> {
    let registry = init_registry();
    let engines = registry
        .list_available()
        .await
        .context("Failed to list available engines")?;

    if json_output {
        let engine_list: Vec<_> = engines
            .iter()
            .map(|info| {
                json!({
                    "id": info.id,
                    "name": info.name,
                    "provider": info.provider,
                    "is_default": info.is_default,
                    "credential_status": match info.credential_status {
                        CredentialStatus::Available => "available",
                        CredentialStatus::Missing => "missing",
                        CredentialStatus::Invalid => "invalid",
                        CredentialStatus::Unknown => "unknown",
                    },
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&engine_list)?);
    } else {
        println!();
        println!(
            "{}",
            format!("📋 Configured Models ({})", engines.len())
                .bold()
                .cyan()
        );
        println!();

        if engines.is_empty() {
            println!("  {}", "No models configured.".dimmed());
            println!();
            println!("  {}", "To configure models, edit .radium/config.toml".dimmed());
            return Ok(());
        }

        // Table header
        println!(
            "{:<15} {:<20} {:<15} {:<10} {}",
            "ID", "Name", "Provider", "Default", "Status"
        );
        println!("{}", "─".repeat(80));

        for info in &engines {
            let default_str = if info.is_default {
                "(default)".green()
            } else {
                "".dimmed()
            };

            let status_str = match info.credential_status {
                CredentialStatus::Available => "✓ OK".green(),
                CredentialStatus::Missing => "✗ MISSING".red(),
                CredentialStatus::Invalid => "✗ INVALID".red(),
                CredentialStatus::Unknown => "? UNKNOWN".yellow(),
            };

            println!(
                "{:<15} {:<20} {:<15} {:<10} {}",
                info.id.cyan(),
                info.name,
                info.provider.dimmed(),
                default_str,
                status_str
            );
        }

        println!();
    }

    Ok(())
}

/// Test a specific model with comprehensive validation.
async fn test_model(model_id: &str) -> Result<()> {
    let registry = init_registry();

    println!();
    println!("{}", format!("Testing model '{}'...", model_id).bold().cyan());
    println!();

    // Check if engine exists
    let engine = registry
        .get(model_id)
        .with_context(|| format!("Model '{}' not found", model_id))?;

    let metadata = engine.metadata();

    // Stage 1: Configuration validation
    println!("  {}", "Stage 1: Configuration validation".dimmed());
    let validation = registry
        .validate_engine(model_id)
        .await
        .with_context(|| format!("Failed to validate model '{}'", model_id))?;

    if validation.config_valid {
        println!("    {} Configuration valid", "✓".green());
    } else {
        println!("    {} Configuration invalid", "✗".red());
        if let Some(ref msg) = validation.error_message {
            println!("      {}", msg.red());
        }
        println!();
        println!("  {}", "Validation failed. Fix configuration issues and try again.".red());
        return Ok(());
    }

    // Stage 2: Credential check
    println!("  {}", "Stage 2: Credential check".dimmed());
    if validation.credentials_available {
        println!("    {} Credentials found", "✓".green());
    } else {
        println!("    {} Credentials missing", "✗".red());
        if let Some(ref msg) = validation.error_message {
            println!("      {}", msg.red());
        }
        println!();
        println!("  {}", "Validation failed. Configure credentials and try again.".red());
        return Ok(());
    }

    // Stage 3: API availability check
    println!("  {}", "Stage 3: API availability check".dimmed());
    let start = std::time::Instant::now();
    let available = engine.is_available().await;
    let api_duration = start.elapsed();

    if available {
        println!("    {} API connection successful ({:?})", "✓".green(), api_duration);
    } else {
        println!("    {} API not reachable", "✗".red());
        println!();
        println!("  {}", "Validation failed. Check network connectivity and try again.".red());
        return Ok(());
    }

    // Stage 4: Test generation
    println!("  {}", "Stage 4: Test generation".dimmed());
    let test_request = radium_core::engines::ExecutionRequest::new(
        engine.default_model(),
        "Hello".to_string(),
    );

    let gen_start = std::time::Instant::now();
    match engine.execute(test_request).await {
        Ok(response) => {
            let gen_duration = gen_start.elapsed();
            let token_count = response.content.split_whitespace().count();
            println!(
                "    {} Test generation completed ({} tokens in {:?})",
                "✓".green(),
                token_count,
                gen_duration
            );
            println!();
            println!("  {}", "All validation stages passed!".green().bold());
        }
        Err(e) => {
            println!("    {} Test generation failed: {}", "✗".red(), e);
            println!();
            println!("  {}", "Validation failed. Check API connectivity and credentials.".red());
            return Ok(());
        }
    }

    Ok(())
}


//! Models command implementation.

use super::types::ModelsCommand;
use anyhow::{Context, Result};
use colored::Colorize;
use radium_core::config::model_cache::load_cache_config;
use radium_core::engines::{
    CredentialStatus, EngineRegistry, ValidationStatus,
};
use radium_core::engines::providers::{ClaudeEngine, GeminiEngine, MockEngine, OpenAIEngine};
use radium_core::workspace::Workspace;
use radium_models::{ModelCache, ModelConfig, ModelType};
use serde_json::json;
use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use toml;

/// Execute the models command.
pub async fn execute(command: ModelsCommand) -> Result<()> {
    match command {
        ModelsCommand::List { json } => list_models(json).await,
        ModelsCommand::Test { model_id } => test_model(&model_id).await,
        ModelsCommand::Warm {
            provider,
            model,
            agents,
            config,
        } => warm_models(provider, model, agents, config).await,
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

/// Warm models in the cache.
async fn warm_models(
    provider: Option<String>,
    model: Option<String>,
    agents: bool,
    config_file: Option<PathBuf>,
) -> Result<()> {
    // Discover workspace
    let workspace = Workspace::discover()
        .map_err(|_| anyhow::anyhow!("No Radium workspace found. Run 'rad init' first."))?;

    // Load cache configuration
    let cache_config = load_cache_config(workspace.root())
        .context("Failed to load cache configuration")?;

    if !cache_config.enabled {
        println!("{}", "Cache is disabled in configuration.".yellow());
        return Ok(());
    }

    // Create cache
    let cache = Arc::new(
        ModelCache::new(cache_config).context("Failed to create model cache")?,
    );

    println!();
    println!("{}", "Warming models...".bold().cyan());
    println!();

    let mut warmed_count = 0;
    let mut failed_count = 0;

    if let Some(config_path) = config_file {
        // Warm from config file
        warm_from_config(&cache, &config_path, &mut warmed_count, &mut failed_count).await?;
    } else if agents {
        // Warm all agent models
        warm_agent_models(&cache, &workspace, &mut warmed_count, &mut failed_count).await?;
    } else if let (Some(prov), Some(mod_name)) = (provider, model) {
        // Warm specific model
        warm_specific_model(&cache, &prov, &mod_name, &mut warmed_count, &mut failed_count).await?;
    } else {
        return Err(anyhow::anyhow!(
            "Must specify --provider and --model, --agents, or --config"
        ));
    }

    println!();
    if warmed_count > 0 {
        println!(
            "{}",
            format!("{} models warmed successfully", warmed_count).green()
        );
    }
    if failed_count > 0 {
        println!(
            "{}",
            format!("{} models failed to warm", failed_count).red()
        );
    }

    // Show cache status
    let stats = cache.get_stats();
    println!(
        "{}",
        format!("Cache status: {}/{} slots used", stats.cache_size, cache.config().max_cache_size)
            .dimmed()
    );

    Ok(())
}

/// Warm a specific model.
async fn warm_specific_model(
    cache: &Arc<ModelCache>,
    provider: &str,
    model: &str,
    warmed_count: &mut usize,
    failed_count: &mut usize,
) -> Result<()> {
    let model_type = ModelType::from_str(provider).map_err(|()| {
        anyhow::anyhow!("Unknown provider: {}. Valid providers: mock, gemini, openai, claude, universal", provider)
    })?;

    let config = ModelConfig::new(model_type, model.to_string());
    let start = Instant::now();

    match cache.get_or_create(config) {
        Ok(_) => {
            let duration = start.elapsed();
            display_warm_progress(provider, model, duration, true);
            *warmed_count += 1;
        }
        Err(e) => {
            display_warm_progress(provider, model, start.elapsed(), false);
            eprintln!("  Error: {}", e);
            *failed_count += 1;
        }
    }

    Ok(())
}

/// Warm all models used by agents.
async fn warm_agent_models(
    cache: &Arc<ModelCache>,
    _workspace: &Workspace,
    warmed_count: &mut usize,
    failed_count: &mut usize,
) -> Result<()> {
    use radium_core::agents::registry::AgentRegistry;

    // Discover agents using AgentRegistry
    let registry = AgentRegistry::with_discovery()
        .context("Failed to initialize agent registry")?;

    let agents = registry.list_all().context("Failed to list agents")?;

    let mut models_to_warm = HashSet::new();

    for agent in &agents {
        // Get metadata from agent config
        if let Some(ref persona) = agent.persona_config {
            if let Some(ref recommended) = persona.recommended_models {
                // Add primary model
                models_to_warm.insert((
                    recommended.primary.engine.clone(),
                    recommended.primary.model.clone(),
                ));

                // Add fallback if present
                if let Some(ref fallback) = recommended.fallback {
                    models_to_warm.insert((fallback.engine.clone(), fallback.model.clone()));
                }

                // Add premium if present
                if let Some(ref premium) = recommended.premium {
                    models_to_warm.insert((premium.engine.clone(), premium.model.clone()));
                }
            }
        }
    }

    for (engine, model) in models_to_warm {
        warm_specific_model(cache, &engine, &model, warmed_count, failed_count).await?;
    }

    Ok(())
}

/// Warm models from configuration file.
async fn warm_from_config(
    cache: &Arc<ModelCache>,
    config_path: &PathBuf,
    warmed_count: &mut usize,
    failed_count: &mut usize,
) -> Result<()> {
    let content = std::fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    #[derive(serde::Deserialize)]
    struct ModelEntry {
        provider: String,
        model: String,
    }

    #[derive(serde::Deserialize)]
    struct ConfigFile {
        models: Vec<ModelEntry>,
    }

    let config: ConfigFile = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

    for entry in config.models {
        warm_specific_model(cache, &entry.provider, &entry.model, warmed_count, failed_count).await?;
    }

    Ok(())
}

/// Display warm progress for a model.
fn display_warm_progress(provider: &str, model: &str, duration: std::time::Duration, success: bool) {
    let status = if success {
        "✓".green()
    } else {
        "✗".red()
    };

    let duration_ms = duration.as_millis();
    println!("  {} {}/{} ({}ms)", status, provider, model, duration_ms);
}


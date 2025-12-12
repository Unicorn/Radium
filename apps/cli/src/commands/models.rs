//! Models command implementation.

use super::types::{FileCommand, ModelsCommand};
use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use radium_core::auth::{CredentialStore, ProviderType};
use radium_core::config::model_cache::load_cache_config;
use radium_core::engines::{
    CredentialStatus, EngineRegistry, ValidationStatus,
};
use radium_core::engines::providers::{BurnEngine, ClaudeEngine, GeminiEngine, MockEngine, OpenAIEngine};
use radium_core::workspace::Workspace;
use radium_models::gemini::file_api::GeminiFileApi;
use radium_models::{CacheKey, ModelCache, ModelConfig, ModelType};
use serde_json::json;
use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use toml;

use crate::colors::RadiumBrandColors;

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
        ModelsCommand::ClearCache { provider, model } => {
            clear_cache(provider, model).await
        }
        ModelsCommand::CacheStatus { json } => cache_status(json).await,
        ModelsCommand::File(command) => handle_file_command(command).await,
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
    let _ = registry.register(Arc::new(BurnEngine::new()));

    // Load config after engines are registered
    let _ = registry.load_config();

    registry
}

/// List all configured models with their status.
async fn list_models(json_output: bool) -> Result<()> {
    let colors = RadiumBrandColors::new();
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
                .color(colors.primary())
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
                "(default)".color(colors.success())
            } else {
                "".dimmed()
            };

            let status_str = match info.credential_status {
                CredentialStatus::Available => "✓ OK".color(colors.success()),
                CredentialStatus::Missing => "✗ MISSING".color(colors.error()),
                CredentialStatus::Invalid => "✗ INVALID".color(colors.error()),
                CredentialStatus::Unknown => "? UNKNOWN".color(colors.warning()),
            };

            println!(
                "{:<15} {:<20} {:<15} {:<10} {}",
                info.id.color(colors.primary()),
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
    let colors = RadiumBrandColors::new();
    println!("{}", format!("Testing model '{}'...", model_id).bold().color(colors.primary()));
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
        println!("    {} Configuration valid", "✓".color(colors.success()));
    } else {
        println!("    {} Configuration invalid", "✗".color(colors.error()));
        if let Some(ref msg) = validation.error_message {
            println!("      {}", msg.color(colors.error()));
        }
        println!();
        println!("  {}", "Validation failed. Fix configuration issues and try again.".color(colors.error()));
        return Ok(());
    }

    // Stage 2: Credential check
    println!("  {}", "Stage 2: Credential check".dimmed());
    if validation.credentials_available {
        println!("    {} Credentials found", "✓".color(colors.success()));
    } else {
        println!("    {} Credentials missing", "✗".color(colors.error()));
        if let Some(ref msg) = validation.error_message {
            println!("      {}", msg.color(colors.error()));
        }
        println!();
        println!("  {}", "Validation failed. Configure credentials and try again.".color(colors.error()));
        return Ok(());
    }

    // Stage 3: API availability check
    println!("  {}", "Stage 3: API availability check".dimmed());
    let start = std::time::Instant::now();
    let available = engine.is_available().await;
    let api_duration = start.elapsed();

    if available {
        println!("    {} API connection successful ({:?})", "✓".color(colors.success()), api_duration);
    } else {
        println!("    {} API not reachable", "✗".color(colors.error()));
        println!();
        println!("  {}", "Validation failed. Check network connectivity and try again.".color(colors.error()));
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
                "✓".color(colors.success()),
                token_count,
                gen_duration
            );
            println!();
            println!("  {}", "All validation stages passed!".color(colors.success()).bold());
        }
        Err(e) => {
            println!("    {} Test generation failed: {}", "✗".color(colors.error()), e);
            println!();
            println!("  {}", "Validation failed. Check API connectivity and credentials.".color(colors.error()));
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
        let colors = RadiumBrandColors::new();
        println!("{}", "Cache is disabled in configuration.".color(colors.warning()));
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

    let mut models_to_warm: HashSet<(String, String)> = HashSet::new();

    for agent in &agents {
        // Get metadata from agent config
        if let Some(ref persona) = agent.persona_config {
            // Add primary model
            models_to_warm.insert((
                persona.models.primary.engine.clone(),
                persona.models.primary.model.clone(),
            ));

            // Add fallback if present
            if let Some(ref fallback) = persona.models.fallback {
                models_to_warm.insert((fallback.engine.clone(), fallback.model.clone()));
            }

            // Add premium if present
            if let Some(ref premium) = persona.models.premium {
                models_to_warm.insert((premium.engine.clone(), premium.model.clone()));
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
    let colors = RadiumBrandColors::new();
    let status = if success {
        "✓".color(colors.success())
    } else {
        "✗".color(colors.error())
    };

    let duration_ms = duration.as_millis();
    println!("  {} {}/{} ({}ms)", status, provider, model, duration_ms);
}

/// Clear models from the cache.
async fn clear_cache(provider: Option<String>, model: Option<String>) -> Result<()> {
    // Discover workspace
    let workspace = Workspace::discover()
        .map_err(|_| anyhow::anyhow!("No Radium workspace found. Run 'rad init' first."))?;

    // Load cache configuration
    let cache_config = load_cache_config(workspace.root())
        .context("Failed to load cache configuration")?;

    if !cache_config.enabled {
        let colors = RadiumBrandColors::new();
        println!("{}", "Cache is disabled in configuration.".color(colors.warning()));
        return Ok(());
    }

    // Create cache
    let cache = Arc::new(
        ModelCache::new(cache_config).context("Failed to create model cache")?,
    );

    let colors = RadiumBrandColors::new();
    let cleared_count = if let (Some(ref prov), Some(ref mod_name)) = (provider.as_ref(), model.as_ref()) {
        // Clear specific model
        let model_type = ModelType::from_str(prov).map_err(|()| {
            anyhow::anyhow!("Unknown provider: {}", prov)
        })?;
        let key = radium_models::CacheKey::new(model_type, mod_name.to_string(), None);
        if cache.remove(&key) {
            println!("{}", format!("Cleared {}/{} from cache", prov, key.model_name).color(colors.success()));
            1
        } else {
            println!("{}", format!("Model {}/{} not found in cache", prov, key.model_name).color(colors.warning()));
            0
        }
    } else if let Some(ref prov) = provider {
        // Clear all models from provider
        use radium_models::CacheKey;
        let model_type = ModelType::from_str(&prov).map_err(|()| {
            anyhow::anyhow!("Unknown provider: {}", prov)
        })?;

        // Get list of cached models
        let models = cache.list_models();
        let keys_to_remove: Vec<CacheKey> = models
            .into_iter()
            .filter(|(k, _)| k.provider == model_type)
            .map(|(k, _)| k)
            .collect();

        let mut cleared = 0;
        for key in keys_to_remove {
            if cache.remove(&key) {
                cleared += 1;
            }
        }

        if cleared > 0 {
            println!("{}", format!("Cleared {} models from {} provider", cleared, prov).color(colors.success()));
        } else {
            println!("{}", format!("No {} models found in cache", prov).color(colors.warning()));
        }
        cleared
    } else {
        // Clear entire cache
        let stats_before = cache.get_stats();
        cache.clear();
        println!("{}", format!("Cleared {} models from cache", stats_before.cache_size).color(colors.success()));
        stats_before.cache_size
    };

    if cleared_count == 0 {
        println!("{}", "Cache is already empty.".dimmed());
    }

    Ok(())
}

/// Display cache status and statistics.
async fn cache_status(json_output: bool) -> Result<()> {
    // Discover workspace
    let workspace = Workspace::discover()
        .map_err(|_| anyhow::anyhow!("No Radium workspace found. Run 'rad init' first."))?;

    // Load cache configuration
    let cache_config = load_cache_config(workspace.root())
        .context("Failed to load cache configuration")?;

    if !cache_config.enabled {
        if json_output {
            println!("{}", serde_json::to_string_pretty(&json!({
                "enabled": false,
                "message": "Cache is disabled in configuration"
            }))?);
        } else {
            let colors = RadiumBrandColors::new();
            println!("{}", "Cache is disabled in configuration.".color(colors.warning()));
        }
        return Ok(());
    }

    // Create cache
    let cache = Arc::new(
        ModelCache::new(cache_config.clone()).context("Failed to create model cache")?,
    );

    let stats = cache.get_stats();
    let models = cache.list_models();

    if json_output {
        let mut models_json = Vec::new();
        let now = Instant::now();

        for (key, cached) in &models {
            let last_accessed_secs = now.duration_since(cached.last_accessed).as_secs();
            let age_secs = now.duration_since(cached.created_at).as_secs();

            models_json.push(json!({
                "provider": format!("{:?}", key.provider).to_lowercase(),
                "model": key.model_name,
                "last_accessed_secs": last_accessed_secs,
                "access_count": cached.access_count,
                "age_secs": age_secs,
            }));
        }

        let output = json!({
            "cache_size": stats.cache_size,
            "max_cache_size": cache_config.max_cache_size,
            "total_hits": stats.total_hits,
            "total_misses": stats.total_misses,
            "total_evictions": stats.total_evictions,
            "models": models_json,
            "config": {
                "enabled": cache_config.enabled,
                "inactivity_timeout_secs": cache_config.inactivity_timeout_secs,
                "max_cache_size": cache_config.max_cache_size,
                "cleanup_interval_secs": cache_config.cleanup_interval_secs,
            }
        });

        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Formatted table output
        println!();
        let colors = RadiumBrandColors::new();
        println!("{}", "Model Cache Status".bold().color(colors.primary()));
        println!("{}", "=".repeat(50));
        println!();

        // Summary
        println!("Cache Size: {}/{} models", stats.cache_size, cache_config.max_cache_size);
        println!("Total Access Count: {}", stats.total_hits + stats.total_misses);
        println!();

        if models.is_empty() {
            println!("{}", "Cache is empty.".dimmed());
            println!();
        } else {
            // Table header
            println!(
                "{:<12} {:<25} {:<15} {:<10} {:<12}",
                "Provider", "Model", "Last Accessed", "Accesses", "Age"
            );
            println!("{}", "─".repeat(80));

            let now = Instant::now();
            for (key, cached) in &models {
                let provider_str = format!("{:?}", key.provider).to_lowercase();
                let last_accessed = format_relative_time(now.duration_since(cached.last_accessed));
                let age = format_duration(now.duration_since(cached.created_at));

                println!(
                    "{:<12} {:<25} {:<15} {:<10} {:<12}",
                    provider_str.color(colors.primary()),
                    key.model_name,
                    last_accessed.dimmed(),
                    cached.access_count,
                    age.dimmed()
                );
            }

            println!();
        }

        // Configuration section
        println!("{}", "Configuration:".bold());
        println!(
            "  Inactivity Timeout: {}",
            format_duration(cache_config.inactivity_timeout())
        );
        println!("  Max Cache Size: {}", cache_config.max_cache_size);
        println!(
            "  Cleanup Interval: {}",
            format_duration(cache_config.cleanup_interval())
        );
        println!();
    }

    Ok(())
}

/// Format a duration as a human-readable string.
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        let mins = secs / 60;
        let remaining_secs = secs % 60;
        if remaining_secs == 0 {
            format!("{}m", mins)
        } else {
            format!("{}m {}s", mins, remaining_secs)
        }
    } else {
        let hours = secs / 3600;
        let remaining_mins = (secs % 3600) / 60;
        if remaining_mins == 0 {
            format!("{}h", hours)
        } else {
            format!("{}h {}m", hours, remaining_mins)
        }
    }
}

/// Format a duration as relative time (e.g., "2m ago").
fn format_relative_time(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s ago", secs)
    } else if secs < 3600 {
        let mins = secs / 60;
        format!("{}m ago", mins)
    } else {
        let hours = secs / 3600;
        format!("{}h ago", hours)
    }
}

/// Handle file management commands.
async fn handle_file_command(command: FileCommand) -> Result<()> {
    match command {
        FileCommand::Upload {
            path,
            mime_type,
            display_name,
        } => file_upload(path, mime_type, display_name).await,
        FileCommand::List => file_list().await,
        FileCommand::Delete { file_id } => file_delete(file_id).await,
    }
}

/// Upload a file to Gemini File API.
async fn file_upload(
    path: PathBuf,
    mime_type: Option<String>,
    display_name: Option<String>,
) -> Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "rad models file upload".bold().color(colors.primary()));
    println!();

    // Get API key
    let credential_store = CredentialStore::new()
        .context("Failed to initialize credential store")?;
    let api_key = credential_store
        .get(ProviderType::Gemini)
        .context("Gemini API key not found. Use 'rad auth set gemini' to set it.")?;

    // Create file API client
    let file_api = GeminiFileApi::with_api_key(api_key);

    println!("{}", "Uploading file...".dimmed());
    println!("  Path: {}", path.display().to_string().color(colors.primary()));

    // Upload file
    let file = file_api
        .upload_file(&path, mime_type, display_name)
        .await
        .context("Failed to upload file")?;

    println!();
    println!("  {} File uploaded successfully", "✓".color(colors.success()));
    println!();
    println!("{}", "File Details:".bold());
    println!("  Name: {}", file.name.color(colors.primary()));
    println!("  URI: {}", file.uri.color(colors.primary()));
    println!("  State: {}", format!("{:?}", file.state).color(colors.success()));
    println!("  Size: {}", format_file_size(file.size_bytes));
    println!("  MIME Type: {}", file.mime_type.color(colors.primary()));
    if let Some(display) = &file.display_name {
        println!("  Display Name: {}", display.color(colors.primary()));
    }
    let expires_in = file.expire_time.signed_duration_since(Utc::now());
    if expires_in.num_seconds() > 0 {
        println!("  Expires In: {}", format_duration(expires_in.to_std().unwrap()));
    } else {
        println!("  Expires In: {}", "Expired".color(colors.error()));
    }
    println!();

    Ok(())
}

/// List all uploaded files.
async fn file_list() -> Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "rad models file list".bold().color(colors.primary()));
    println!();

    // Get API key
    let credential_store = CredentialStore::new()
        .context("Failed to initialize credential store")?;
    let api_key = credential_store
        .get(ProviderType::Gemini)
        .context("Gemini API key not found. Use 'rad auth set gemini' to set it.")?;

    // Create file API client
    let file_api = GeminiFileApi::with_api_key(api_key);

    println!("{}", "Fetching files...".dimmed());

    // List files
    let files = file_api
        .list_files()
        .await
        .context("Failed to list files")?;

    println!();
    if files.is_empty() {
        println!("  {} No files found", "ℹ".color(colors.info()));
        println!();
        return Ok(());
    }

    println!(
        "{}",
        format!("📋 Uploaded Files ({})", files.len())
            .bold()
            .color(colors.primary())
    );
    println!();

    // Print table header
    println!(
        "{:<30} {:<12} {:<12} {:<20}",
        "Name".bold(),
        "State".bold(),
        "Size".bold(),
        "Expires In".bold()
    );
    println!("{}", "-".repeat(80));

    // Print files
    for file in files {
        let name = if file.name.len() > 28 {
            format!("{}..", &file.name[..26])
        } else {
            file.name.clone()
        };

        let state_str = match file.state {
            radium_models::gemini::file_api::FileState::Active => {
                format!("{:?}", file.state).color(colors.success()).to_string()
            }
            radium_models::gemini::file_api::FileState::Processing => {
                format!("{:?}", file.state).color(colors.warning()).to_string()
            }
            radium_models::gemini::file_api::FileState::Failed => {
                format!("{:?}", file.state).color(colors.error()).to_string()
            }
        };

        let expires_in = file.expire_time.signed_duration_since(Utc::now());
        let expires_str = if expires_in.num_seconds() > 0 {
            format_duration(expires_in.to_std().unwrap())
        } else {
            "Expired".color(colors.error()).to_string()
        };

        println!(
            "{:<30} {:<12} {:<12} {:<20}",
            name.color(colors.primary()),
            state_str,
            format_file_size(file.size_bytes),
            expires_str
        );
    }

    println!();
    Ok(())
}

/// Delete a file from Gemini File API.
async fn file_delete(file_id: String) -> Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "rad models file delete".bold().color(colors.primary()));
    println!();

    // Get API key
    let credential_store = CredentialStore::new()
        .context("Failed to initialize credential store")?;
    let api_key = credential_store
        .get(ProviderType::Gemini)
        .context("Gemini API key not found. Use 'rad auth set gemini' to set it.")?;

    // Create file API client
    let file_api = GeminiFileApi::with_api_key(api_key);

    println!("{}", "Deleting file...".dimmed());
    println!("  File ID: {}", file_id.color(colors.primary()));

    // Delete file
    file_api
        .delete_file(&file_id)
        .await
        .context("Failed to delete file")?;

    println!();
    println!("  {} File deleted successfully", "✓".color(colors.success()));
    println!();

    Ok(())
}

/// Format file size in human-readable format.
fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}


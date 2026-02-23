//! Status command implementation.

use anyhow::Context;
use colored::Colorize;
use radium_core::auth::{CredentialStore, ProviderType};
use radium_core::engines::{CredentialStatus, EngineRegistry};
use radium_core::engines::providers::{BurnEngine, ClaudeEngine, GeminiEngine, MockEngine, OpenAIEngine};
use radium_core::{AgentDiscovery, Workspace};
use serde_json::json;
use std::sync::Arc;

use crate::colors::RadiumBrandColors;

/// Initialize engine registry with all available engines.
fn init_registry() -> EngineRegistry {
    let config_path = Workspace::discover()
        .ok()
        .map(|w| w.radium_dir().join("config.toml"));

    let registry = if let Some(ref path) = config_path {
        EngineRegistry::with_config_path(path)
    } else {
        EngineRegistry::new()
    };

    let _ = registry.register(Arc::new(MockEngine::new()));
    let _ = registry.register(Arc::new(ClaudeEngine::new()));
    let _ = registry.register(Arc::new(OpenAIEngine::new()));
    let _ = registry.register(Arc::new(GeminiEngine::new()));
    let _ = registry.register(Arc::new(BurnEngine::new()));
    let _ = registry.load_config();

    registry
}

/// Execute the status command.
///
/// Shows workspace status, available agents, models, and authentication.
pub async fn execute(json_output: bool) -> anyhow::Result<()> {
    if json_output { execute_json().await } else { execute_human().await }
}

async fn execute_human() -> anyhow::Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "Radium Status".bold().color(colors.primary()));
    println!();

    // Workspace status
    println!("{}", "Workspace:".bold());
    match Workspace::discover() {
        Ok(workspace) => {
            println!("  Location: {}", workspace.root().display().to_string().color(colors.success()));
            println!("  Valid: {}", "✓".color(colors.success()));

            // Check if empty
            if workspace.is_empty()? {
                println!("  Plans: {}", "0 (empty)".color(colors.warning()));
            } else {
                let plans = workspace.discover_plans()?;
                println!("  Plans: {}", format!("{}", plans.len()).color(colors.success()));
            }
        }
        Err(e) => {
            println!("  Status: {}", format!("Not found - {}", e).color(colors.error()));
            println!();
            println!("  {}", "Create a workspace with:".color(colors.warning()));
            println!("    rad plan <spec.md>");
            println!();
        }
    }
    println!();

    // Agent discovery
    println!("{}", "Agents:".bold());
    let discovery = AgentDiscovery::new();
    match discovery.discover_all() {
        Ok(agents) => {
            if agents.is_empty() {
                println!("  {}", "No agents found".color(colors.warning()));
                println!("  {}", "Place agent configs in ./agents/ or ~/.radium/agents/".dimmed());
            } else {
                println!("  Total: {}", format!("{}", agents.len()).color(colors.success()));
                println!();
                println!("  {}:", "Categories".dimmed());

                // Group by category
                let mut by_category: std::collections::HashMap<String, Vec<&str>> =
                    std::collections::HashMap::new();
                for (id, config) in &agents {
                    let category = config.category.as_deref().unwrap_or("uncategorized");
                    by_category.entry(category.to_string()).or_default().push(id.as_str());
                }

                for (category, agent_ids) in &by_category {
                    println!(
                        "    {}: {} agents",
                        category.color(colors.primary()),
                        agent_ids.len().to_string().dimmed()
                    );
                }
            }
        }
        Err(e) => {
            println!("  {}", format!("Discovery failed - {}", e).color(colors.error()));
        }
    }
    println!();

    // Models (live from registry)
    println!("{}", "Models:".bold());
    let registry = init_registry();
    match registry.list_available().await {
        Ok(engines) if !engines.is_empty() => {
            for info in &engines {
                let status_icon = match info.credential_status {
                    CredentialStatus::Available => "✓".color(colors.success()),
                    CredentialStatus::Missing | CredentialStatus::Invalid => "✗".color(colors.error()),
                    CredentialStatus::Unknown => "?".color(colors.warning()),
                };
                println!(
                    "  {} {} ({}) [{}]",
                    status_icon,
                    info.name,
                    info.provider.dimmed(),
                    info.id.dimmed()
                );
            }
        }
        Ok(_) => {
            println!("  {}", "No models configured.".color(colors.warning()));
            println!("  {}", "Edit .radium/config.toml to configure models.".dimmed());
        }
        Err(e) => {
            println!("  {}", format!("Failed to list models: {}", e).color(colors.error()));
        }
    }
    println!();

    // Authentication (live from credential store)
    println!("{}", "Authentication:".bold());
    match CredentialStore::new().context("Failed to initialize credential store") {
        Ok(store) => {
            for provider in ProviderType::all() {
                if store.is_configured(provider) {
                    println!(
                        "  • {}: {}",
                        provider.as_str(),
                        "✓ Configured".color(colors.success())
                    );
                } else {
                    let env_var = provider.env_var_names().into_iter().next().unwrap_or("(unknown)");
                    println!(
                        "  • {}: {} (set {})",
                        provider.as_str(),
                        "Not configured".color(colors.warning()),
                        env_var.dimmed()
                    );
                }
            }
        }
        Err(e) => {
            println!("  {}", format!("Failed to load credentials: {}", e).color(colors.error()));
        }
    }

    Ok(())
}

async fn execute_json() -> anyhow::Result<()> {
    let mut status = json!({
        "workspace": null,
        "agents": {},
        "models": [],
        "authentication": {}
    });

    // Workspace status
    if let Ok(workspace) = Workspace::discover() {
        let plans = workspace.discover_plans().unwrap_or_default();
        status["workspace"] = json!({
            "location": workspace.root().display().to_string(),
            "valid": true,
            "plan_count": plans.len(),
        });
    }

    // Agent discovery
    if let Ok(agents) = AgentDiscovery::new().discover_all() {
        let mut by_category: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for (id, config) in agents {
            let category = config.category.unwrap_or_else(|| "uncategorized".to_string());
            by_category.entry(category).or_default().push(id);
        }
        status["agents"] = json!(by_category);
    }

    // Models (live from registry)
    let registry = init_registry();
    let engine_list = match registry.list_available().await {
        Ok(engines) => engines
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
            .collect::<Vec<_>>(),
        Err(_) => vec![],
    };
    status["models"] = json!(engine_list);

    // Authentication (live from credential store)
    let mut auth = serde_json::Map::new();
    if let Ok(store) = CredentialStore::new() {
        for provider in ProviderType::all() {
            let value = if store.is_configured(provider) {
                "configured"
            } else {
                "not_configured"
            };
            auth.insert(provider.as_str().to_string(), json!(value));
        }
    }
    status["authentication"] = json!(auth);

    println!("{}", serde_json::to_string_pretty(&status)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// init_registry() should register all 5 known providers and return a non-empty registry.
    #[tokio::test]
    async fn test_init_registry_registers_all_engines() {
        let registry = init_registry();
        let engines = registry.list_available().await.expect("list_available failed");
        // Must contain at least: mock, claude, openai, gemini, burn
        assert!(
            engines.len() >= 5,
            "expected at least 5 engines, got {}: {:?}",
            engines.len(),
            engines.iter().map(|e| &e.id).collect::<Vec<_>>()
        );
        let ids: Vec<&str> = engines.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"mock"),   "mock engine missing");
        assert!(ids.contains(&"claude"), "claude engine missing");
        assert!(ids.contains(&"openai"), "openai engine missing");
        assert!(ids.contains(&"gemini"), "gemini engine missing");
        assert!(ids.contains(&"burn"),   "burn engine missing");
    }

    /// The mock engine should always report credentials as available.
    #[tokio::test]
    async fn test_init_registry_mock_engine_available() {
        let registry = init_registry();
        let engines = registry.list_available().await.expect("list_available failed");
        let mock = engines.iter().find(|e| e.id == "mock").expect("mock engine not found");
        assert_eq!(
            mock.credential_status,
            CredentialStatus::Available,
            "mock engine should always be available"
        );
    }
}

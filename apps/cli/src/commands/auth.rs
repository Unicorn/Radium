//! Auth command implementation.

use anyhow::{Result, anyhow};
use colored::Colorize;
use radium_core::auth::{CredentialStore, ProviderType};
use serde_json::json;
use std::io::{self, Write};

use super::AuthCommand;

/// Executes the auth command.
pub async fn execute(command: AuthCommand) -> Result<()> {
    match command {
        AuthCommand::Login { all, provider } => {
            if all {
                login_all_providers().await
            } else if let Some(ref p) = provider {
                login_provider(p).await
            } else {
                // Interactive mode: prompt for provider selection
                login_interactive().await
            }
        }
        AuthCommand::Logout { all, provider } => {
            if all {
                logout_all_providers().await
            } else if let Some(ref p) = provider {
                logout_provider(p).await
            } else {
                logout_interactive().await
            }
        }
        AuthCommand::Status { json } => show_status(json).await,
    }
}

async fn login_provider(provider_name: &str) -> Result<()> {
    let provider_type = ProviderType::parse(provider_name).ok_or_else(|| {
        anyhow!("Unknown provider: {}. Supported providers: gemini, openai", provider_name)
    })?;

    println!();
    println!("{}", format!("Login to {}", provider_name).bold().cyan());
    println!();

    // Prompt for API key
    print!("Enter API key: ");
    io::stdout().flush()?;

    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim().to_string();

    if api_key.is_empty() {
        return Err(anyhow!("API key cannot be empty"));
    }

    // Store credential
    let store = CredentialStore::new()?;
    store.store(provider_type, api_key)?;

    println!();
    println!("{}", format!("✓ Successfully authenticated with {}", provider_name).green());
    println!("  Credentials stored in: {}", "~/.radium/auth/credentials.json".yellow());
    println!();

    Ok(())
}

async fn login_all_providers() -> Result<()> {
    println!();
    println!("{}", "Login to all providers".bold().cyan());
    println!();

    for provider_type in ProviderType::all() {
        match login_provider(provider_type.as_str()).await {
            Ok(()) => {}
            Err(e) => {
                eprintln!(
                    "{}",
                    format!("✗ Failed to login to {}: {}", provider_type.as_str(), e).red()
                );
            }
        }
        println!();
    }

    Ok(())
}

async fn login_interactive() -> Result<()> {
    println!();
    println!("{}", "Authentication".bold().cyan());
    println!();
    println!("Select a provider:");

    let providers = ProviderType::all();
    for (i, provider) in providers.iter().enumerate() {
        println!("  {}. {}", i + 1, provider.as_str());
    }
    println!();

    print!("Choice (1-{}): ", providers.len());
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice: usize = choice.trim().parse().map_err(|_| anyhow!("Invalid choice"))?;

    if choice == 0 || choice > providers.len() {
        return Err(anyhow!("Invalid choice"));
    }

    let provider = providers[choice - 1];
    login_provider(provider.as_str()).await
}

async fn logout_provider(provider_name: &str) -> Result<()> {
    let provider_type = ProviderType::parse(provider_name)
        .ok_or_else(|| anyhow!("Unknown provider: {}", provider_name))?;

    let store = CredentialStore::new()?;
    store.remove(provider_type)?;

    println!();
    println!("{}", format!("✓ Logged out from {}", provider_name).green());
    println!();

    Ok(())
}

async fn logout_all_providers() -> Result<()> {
    let store = CredentialStore::new()?;

    println!();
    println!("{}", "Logout from all providers".bold().cyan());
    println!();

    for provider_type in ProviderType::all() {
        match store.remove(provider_type) {
            Ok(()) => {
                println!("{}", format!("✓ Logged out from {}", provider_type.as_str()).green())
            }
            Err(e) => eprintln!(
                "{}",
                format!("✗ Error logging out from {}: {}", provider_type.as_str(), e).red()
            ),
        }
    }
    println!();

    Ok(())
}

async fn logout_interactive() -> Result<()> {
    println!();
    println!("{}", "Logout".bold().cyan());
    println!();

    let store = CredentialStore::new()?;
    let configured = store.list()?;

    if configured.is_empty() {
        println!("No providers are currently logged in.");
        println!();
        return Ok(());
    }

    println!("Select a provider to logout:");
    for (i, provider) in configured.iter().enumerate() {
        println!("  {}. {}", i + 1, provider.as_str());
    }
    println!();

    print!("Choice (1-{}): ", configured.len());
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice: usize = choice.trim().parse().map_err(|_| anyhow!("Invalid choice"))?;

    if choice == 0 || choice > configured.len() {
        return Err(anyhow!("Invalid choice"));
    }

    let provider = configured[choice - 1];
    logout_provider(provider.as_str()).await
}

async fn show_status(json_output: bool) -> Result<()> {
    let store = CredentialStore::new()?;

    if json_output {
        let mut status = serde_json::Map::new();
        for provider in ProviderType::all() {
            let configured = store.is_configured(provider);
            let source = if configured {
                // Try to determine if it's from file or environment
                match store.get(provider) {
                    Ok(_) => {
                        // Check if it exists in the file
                        if store.list().unwrap_or_default().contains(&provider) {
                            "file"
                        } else {
                            "environment"
                        }
                    }
                    Err(_) => "none",
                }
            } else {
                "none"
            };

            status.insert(
                provider.as_str().to_string(),
                json!({
                    "configured": configured,
                    "source": source
                }),
            );
        }
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        println!();
        println!("{}", "Authentication Status".bold().cyan());
        println!();

        for provider in ProviderType::all() {
            let configured = store.is_configured(provider);
            let status_text = if configured {
                let source = if store.list().unwrap_or_default().contains(&provider) {
                    "(from file)"
                } else {
                    "(from environment)"
                };
                format!("{} {}", "✓ Configured".green(), source.dimmed())
            } else {
                format!("{}", "✗ Not configured".yellow())
            };

            println!("  • {}: {}", provider.as_str(), status_text);

            // Show environment variable names
            if !configured {
                let env_vars = provider.env_var_names();
                println!(
                    "    {}",
                    format!("Environment variables: {}", env_vars.join(", ")).dimmed()
                );
            }
        }
        println!();
        println!("Credentials stored in: {}", "~/.radium/auth/credentials.json".yellow());
        println!();
    }

    Ok(())
}

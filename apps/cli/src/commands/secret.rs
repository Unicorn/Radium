//! Secret command implementation.

use anyhow::{Result, anyhow};
use colored::Colorize;
use radium_core::security::{MigrationManager, SecretManager, SecretScanner};
use radium_core::workspace::Workspace;
use rpassword::read_password;
use serde_json::json;
use std::io::{self, Write};
use std::path::PathBuf;

use super::SecretCommand;

/// Default vault path.
fn default_vault_path() -> Result<PathBuf> {
    #[allow(clippy::disallowed_methods)]
    let home = std::env::var("HOME")
        .map_err(|_| anyhow!("HOME environment variable not set"))?;
    Ok(PathBuf::from(home).join(".radium/auth/secrets.vault"))
}

/// Prompts for master password.
fn prompt_master_password() -> Result<String> {
    print!("Enter master password: ");
    io::stdout().flush()?;
    let password = read_password()?;
    Ok(password)
}

/// Prompts for master password with confirmation.
fn prompt_master_password_with_confirmation() -> Result<String> {
    print!("Enter master password: ");
    io::stdout().flush()?;
    let password = read_password()?;

    if password.len() < 12 {
        return Err(anyhow!("Password must be at least 12 characters"));
    }

    print!("Confirm master password: ");
    io::stdout().flush()?;
    let confirm = read_password()?;

    if password != confirm {
        return Err(anyhow!("Passwords do not match"));
    }

    Ok(password)
}

/// Executes the secret command.
pub async fn execute(command: SecretCommand) -> Result<()> {
    match command {
        SecretCommand::Add { name } => add_secret(name).await,
        SecretCommand::List { json } => list_secrets(json).await,
        SecretCommand::Rotate { name } => rotate_secret(name).await,
        SecretCommand::Remove { name, force } => remove_secret(name, force).await,
        SecretCommand::Scan { json } => scan_secrets(json).await,
        SecretCommand::Migrate => migrate_secrets().await,
    }
}

/// Adds a new secret.
async fn add_secret(name: Option<String>) -> Result<()> {
    let secret_name = if let Some(n) = name {
        n
    } else {
        print!("Enter secret name: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    if secret_name.is_empty() {
        return Err(anyhow!("Secret name cannot be empty"));
    }

    println!();
    println!("{}", format!("Add secret: {}", secret_name).bold().cyan());
    println!();

    // Check if vault exists to determine if we need to create or open
    let vault_path = default_vault_path()?;
    let master_password = if vault_path.exists() {
        prompt_master_password()?
    } else {
        println!("Creating new encrypted vault...");
        prompt_master_password_with_confirmation()?
    };

    // Create or open secret manager
    let mut manager = if vault_path.exists() {
        SecretManager::from_existing(vault_path, &master_password)
            .map_err(|e| anyhow!("Failed to open vault: {}", e))?
    } else {
        SecretManager::new(vault_path, &master_password)
            .map_err(|e| anyhow!("Failed to create vault: {}", e))?
    };

    // Prompt for secret value
    print!("Enter secret value: ");
    io::stdout().flush()?;
    let value = read_password()?;

    if value.is_empty() {
        return Err(anyhow!("Secret value cannot be empty"));
    }

    // Confirm value
    print!("Confirm secret value: ");
    io::stdout().flush()?;
    let confirm = read_password()?;

    if value != confirm {
        return Err(anyhow!("Secret values do not match"));
    }

    // Store secret
    manager.store_secret(&secret_name, &value)
        .map_err(|e| anyhow!("Failed to store secret: {}", e))?;

    println!();
    println!("{}", format!("✓ Secret '{}' stored successfully", secret_name).green());
    println!();

    Ok(())
}

/// Lists all secrets.
async fn list_secrets(json_output: bool) -> Result<()> {
    let vault_path = default_vault_path()?;

    if !vault_path.exists() {
        if json_output {
            println!("[]");
        } else {
            println!("{}", "No secrets found. Vault does not exist.".yellow());
        }
        return Ok(());
    }

    let master_password = prompt_master_password()?;
    let manager = SecretManager::from_existing(vault_path, &master_password)
        .map_err(|e| anyhow!("Failed to open vault: {}", e))?;

    let names = manager.list_secrets()
        .map_err(|e| anyhow!("Failed to list secrets: {}", e))?;

    if json_output {
        let json_array = json!(names);
        println!("{}", serde_json::to_string_pretty(&json_array)?);
    } else {
        if names.is_empty() {
            println!("{}", "No secrets found.".yellow());
        } else {
            println!();
            println!("{}", "Stored Secrets".bold().cyan());
            println!();
            for name in &names {
                println!("  • {}", name);
            }
            println!();
            println!("{}", format!("Total: {} secret(s)", names.len()).dimmed());
        }
    }

    Ok(())
}

/// Rotates a secret.
async fn rotate_secret(name: String) -> Result<()> {
    let vault_path = default_vault_path()?;

    if !vault_path.exists() {
        return Err(anyhow!("Vault does not exist. Create it by adding a secret first."));
    }

    let master_password = prompt_master_password()?;
    let mut manager = SecretManager::from_existing(vault_path, &master_password)
        .map_err(|e| anyhow!("Failed to open vault: {}", e))?;

    // Verify secret exists
    manager.get_secret(&name)
        .map_err(|_| anyhow!("Secret '{}' not found", name))?;

    println!();
    println!("{}", format!("Rotate secret: {}", name).bold().cyan());
    println!();

    // Prompt for new value
    print!("Enter new secret value: ");
    io::stdout().flush()?;
    let new_value = read_password()?;

    if new_value.is_empty() {
        return Err(anyhow!("Secret value cannot be empty"));
    }

    // Confirm new value
    print!("Confirm new secret value: ");
    io::stdout().flush()?;
    let confirm = read_password()?;

    if new_value != confirm {
        return Err(anyhow!("Secret values do not match"));
    }

    // Rotate secret
    manager.rotate_secret(&name, &new_value)
        .map_err(|e| anyhow!("Failed to rotate secret: {}", e))?;

    println!();
    println!("{}", format!("✓ Secret '{}' rotated successfully", name).green());
    println!();

    Ok(())
}

/// Removes a secret.
async fn remove_secret(name: String, force: bool) -> Result<()> {
    let vault_path = default_vault_path()?;

    if !vault_path.exists() {
        return Err(anyhow!("Vault does not exist."));
    }

    let master_password = prompt_master_password()?;
    let mut manager = SecretManager::from_existing(vault_path, &master_password)
        .map_err(|e| anyhow!("Failed to open vault: {}", e))?;

    // Verify secret exists
    manager.get_secret(&name)
        .map_err(|_| anyhow!("Secret '{}' not found", name))?;

    if !force {
        print!("Are you sure you want to delete secret '{}'? (yes/no): ", name);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "yes" {
            println!("Cancelled.");
            return Ok(());
        }
    }

    manager.remove_secret(&name)
        .map_err(|e| anyhow!("Failed to remove secret: {}", e))?;

    println!();
    println!("{}", format!("✓ Secret '{}' removed successfully", name).green());
    println!();

    Ok(())
}

/// Scans workspace for hardcoded credentials.
async fn scan_secrets(json_output: bool) -> Result<()> {
    let workspace = Workspace::discover()
        .map_err(|e| anyhow!("Failed to discover workspace: {}", e))?;

    let scanner = SecretScanner::new();
    let report = scanner.scan_workspace(&workspace)
        .map_err(|e| anyhow!("Failed to scan workspace: {}", e))?;

    if json_output {
        let json_report = json!({
            "total_files_scanned": report.total_files_scanned,
            "matches": report.matches.iter().map(|m| json!({
                "file_path": m.file_path.to_string_lossy(),
                "line_number": m.line_number,
                "column": m.column,
                "credential_type": m.credential_type,
                "severity": format!("{:?}", m.severity),
                "matched_text": m.matched_text,
            })).collect::<Vec<_>>(),
            "high_severity_count": report.high_severity_count,
            "medium_severity_count": report.medium_severity_count,
            "low_severity_count": report.low_severity_count,
        });
        println!("{}", serde_json::to_string_pretty(&json_report)?);
    } else {
        println!();
        println!("{}", "Secret Scan Report".bold().cyan());
        println!();

        if report.matches.is_empty() {
            println!("{}", "✓ No hardcoded credentials detected.".green());
        } else {
            println!("Files scanned: {}", report.total_files_scanned);
            println!("Matches found: {}", report.matches.len());
            println!();

            // Group by severity
            if report.high_severity_count > 0 {
                println!("{}", format!("High Severity: {}", report.high_severity_count).red().bold());
            }
            if report.medium_severity_count > 0 {
                println!("{}", format!("Medium Severity: {}", report.medium_severity_count).yellow().bold());
            }
            if report.low_severity_count > 0 {
                println!("{}", format!("Low Severity: {}", report.low_severity_count).blue().bold());
            }
            println!();

            // Show matches
            for m in &report.matches {
                let severity_color = match m.severity {
                    radium_core::security::Severity::High => "red",
                    radium_core::security::Severity::Medium => "yellow",
                    radium_core::security::Severity::Low => "blue",
                };

                println!(
                    "  {} {}:{} - {} ({})",
                    m.file_path.display(),
                    m.line_number,
                    m.column,
                    m.credential_type,
                    format!("{:?}", m.severity).color(severity_color),
                );
            }
        }
        println!();
    }

    Ok(())
}

/// Migrates credentials from plaintext to encrypted vault.
async fn migrate_secrets() -> Result<()> {
    println!();
    println!("{}", "Migrate Credentials to Encrypted Vault".bold().cyan());
    println!();

    // Check if credentials file exists
    let creds_path = MigrationManager::detect_credentials_file();
    if creds_path.is_none() {
        println!("{}", "No credentials.json file found. Nothing to migrate.".yellow());
        return Ok(());
    }

    println!("Found credentials file: {}", creds_path.as_ref().unwrap().display());
    println!();

    // Prompt for master password
    println!("Create a master password for the encrypted vault:");
    let master_password = prompt_master_password_with_confirmation()?;

    println!();
    println!("Migrating credentials...");

    // Run migration
    let report = MigrationManager::migrate_to_vault(&master_password)
        .map_err(|e| anyhow!("Migration failed: {}", e))?;

    println!();
    if report.migrated > 0 {
        println!("{}", format!("✓ Migration completed successfully!").green());
        println!();
        println!("  Total credentials: {}", report.total_credentials);
        println!("  Migrated: {}", report.migrated);
        if report.failed > 0 {
            println!("  {} Failed: {}", "⚠".yellow(), report.failed);
        }
        println!("  Backup created: {}", report.backup_path.display());
        println!();
        println!("{}", "Original credentials.json has been marked as deprecated.".dimmed());
        println!("{}", "Your credentials are now stored in the encrypted vault.".dimmed());
        println!();
        println!("Next steps:");
        println!("  1. Verify your credentials work: radium auth status");
        println!("  2. Test secret access: radium secret list");
        println!("  3. If needed, rollback from: {}", report.backup_path.display());
    } else {
        println!("{}", "⚠ No credentials were migrated.".yellow());
        if report.failed > 0 {
            println!("  All {} credentials failed to migrate.", report.failed);
        }
    }
    println!();

    Ok(())
}


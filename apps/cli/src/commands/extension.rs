//! Extension command implementation.
//!
//! Provides commands for installing, managing, and discovering extensions.

use super::ExtensionCommand;
use colored::Colorize;
use radium_core::extensions::{
    ExtensionDiscovery, ExtensionManager, InstallOptions,
};
use serde_json::json;
use std::path::Path;
use tabled::{Table, Tabled, settings::Style};

/// Execute the extension command.
pub async fn execute(command: ExtensionCommand) -> anyhow::Result<()> {
    match command {
        ExtensionCommand::Install {
            source,
            overwrite,
            install_deps,
        } => install_extension(&source, overwrite, install_deps).await,
        ExtensionCommand::Uninstall { name } => uninstall_extension(&name).await,
        ExtensionCommand::List { json, verbose } => list_extensions(json, verbose).await,
        ExtensionCommand::Info { name, json } => show_extension_info(&name, json).await,
        ExtensionCommand::Search { query, json } => search_extensions(&query, json).await,
    }
}

/// Install an extension.
async fn install_extension(
    source: &str,
    overwrite: bool,
    install_deps: bool,
) -> anyhow::Result<()> {
    let manager = ExtensionManager::new()?;

    // Check if source is a URL or local path
    let is_url = source.starts_with("http://") || source.starts_with("https://");

    if is_url {
        println!("{}", "Installing extension from URL...".yellow());
        println!("{}", "URL installation is not yet implemented.".red());
        return Err(anyhow::anyhow!("URL installation not yet implemented"));
    }

    // Local path installation
    let package_path = Path::new(source);
    if !package_path.exists() {
        return Err(anyhow::anyhow!("Extension package not found: {}", source));
    }

    if !package_path.is_dir() {
        return Err(anyhow::anyhow!("Extension source must be a directory: {}", source));
    }

    println!("{}", format!("Installing extension from: {}", source).yellow());

    let options = InstallOptions {
        overwrite,
        install_dependencies: install_deps,
        validate_after_install: true,
    };

    match manager.install(package_path, options) {
        Ok(extension) => {
            println!("{}", format!("✓ Extension '{}' installed successfully", extension.name).green());
            println!(
                "  Version: {}",
                extension.version.bright_black()
            );
            if !extension.manifest.description.is_empty() {
                println!("  Description: {}", extension.manifest.description.bright_black());
            }
            Ok(())
        }
        Err(e) => {
            println!("{}", format!("✗ Failed to install extension: {}", e).red());
            Err(anyhow::anyhow!("Installation failed: {}", e))
        }
    }
}

/// Uninstall an extension.
async fn uninstall_extension(name: &str) -> anyhow::Result<()> {
    let manager = ExtensionManager::new()?;

    println!("{}", format!("Uninstalling extension: {}", name).yellow());

    match manager.uninstall(name) {
        Ok(()) => {
            println!("{}", format!("✓ Extension '{}' uninstalled successfully", name).green());
            Ok(())
        }
        Err(e) => {
            println!("{}", format!("✗ Failed to uninstall extension: {}", e).red());
            Err(anyhow::anyhow!("Uninstallation failed: {}", e))
        }
    }
}

/// List all installed extensions.
async fn list_extensions(json_output: bool, verbose: bool) -> anyhow::Result<()> {
    let manager = ExtensionManager::new()?;
    let extensions = manager.list()?;

    if extensions.is_empty() {
        if !json_output {
            println!("{}", "No extensions installed.".yellow());
            println!();
            println!("Install extensions from a local directory:");
            println!("  $ rad extension install ./my-extension");
        }
        return Ok(());
    }

    if json_output {
        let extension_list: Vec<_> = extensions
            .iter()
            .map(|ext| {
                json!({
                    "name": ext.name,
                    "version": ext.version,
                    "description": ext.manifest.description,
                    "author": ext.manifest.author,
                    "components": {
                        "prompts": ext.manifest.components.prompts.len(),
                        "mcp_servers": ext.manifest.components.mcp_servers.len(),
                        "commands": ext.manifest.components.commands.len(),
                    },
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&extension_list)?);
    } else {
        println!();
        println!("{}", format!("📦 Installed Extensions ({})", extensions.len()).bold().green());
        println!();

        if verbose {
            display_extensions_detailed(&extensions);
        } else {
            display_extensions_table(&extensions);
        }
    }

    Ok(())
}

/// Show detailed information about a specific extension.
async fn show_extension_info(name: &str, json_output: bool) -> anyhow::Result<()> {
    let manager = ExtensionManager::new()?;
    let extension = manager.get(name)?;

    let extension = extension.ok_or_else(|| anyhow::anyhow!("Extension not found: {}", name))?;

    if json_output {
        let info = json!({
            "name": extension.name,
            "version": extension.version,
            "description": extension.manifest.description,
            "author": extension.manifest.author,
            "install_path": extension.install_path,
            "components": {
                "prompts": extension.manifest.components.prompts,
                "mcp_servers": extension.manifest.components.mcp_servers,
                "commands": extension.manifest.components.commands,
            },
            "dependencies": extension.manifest.dependencies,
        });
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!();
        println!("{}", format!("Extension: {}", extension.name).bold().green());
        println!();
        println!("  Version:      {}", extension.version);
        println!("  Description:  {}", extension.manifest.description);
        println!("  Author:       {}", extension.manifest.author);
        println!("  Install Path: {}", extension.install_path.display());
        println!();

        if !extension.manifest.components.prompts.is_empty() {
            println!("  Prompts:");
            for prompt in &extension.manifest.components.prompts {
                println!("    • {}", prompt);
            }
            println!();
        }

        if !extension.manifest.components.mcp_servers.is_empty() {
            println!("  MCP Servers:");
            for mcp in &extension.manifest.components.mcp_servers {
                println!("    • {}", mcp);
            }
            println!();
        }

        if !extension.manifest.components.commands.is_empty() {
            println!("  Commands:");
            for cmd in &extension.manifest.components.commands {
                println!("    • {}", cmd);
            }
            println!();
        }

        if !extension.manifest.dependencies.is_empty() {
            println!("  Dependencies:");
            for dep in &extension.manifest.dependencies {
                println!("    • {}", dep);
            }
        }
    }

    Ok(())
}

/// Search for extensions.
async fn search_extensions(query: &str, json_output: bool) -> anyhow::Result<()> {
    let discovery = ExtensionDiscovery::new();
    let matches = discovery.search(query)?;

    if matches.is_empty() {
        if !json_output {
            println!("{}", format!("No extensions found matching '{}'", query).yellow());
        }
        return Ok(());
    }

    if json_output {
        let extension_list: Vec<_> = matches
            .iter()
            .map(|ext| {
                json!({
                    "name": ext.name,
                    "version": ext.version,
                    "description": ext.manifest.description,
                    "author": ext.manifest.author,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&extension_list)?);
    } else {
        println!();
        println!("{}", format!("Found {} extension(s) matching '{}'", matches.len(), query).bold().green());
        println!();

        for ext in &matches {
            println!("  {} ({})", ext.name.bold(), ext.version.bright_black());
            if !ext.manifest.description.is_empty() {
                println!("    {}", ext.manifest.description.bright_black());
            }
            println!();
        }
    }

    Ok(())
}

/// Extension table row for display.
#[derive(Tabled)]
struct ExtensionRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Version")]
    version: String,
    #[tabled(rename = "Description")]
    description: String,
}

/// Display extensions in a table format.
fn display_extensions_table(extensions: &[radium_core::extensions::Extension]) {
    let rows: Vec<ExtensionRow> = extensions
        .iter()
        .map(|ext| ExtensionRow {
            name: ext.name.clone(),
            version: ext.version.clone(),
            description: ext.manifest.description.clone(),
        })
        .collect();

    let table = Table::new(rows)
        .with(Style::modern())
        .to_string();
    println!("{}", table);
}

/// Display extensions in detailed format.
fn display_extensions_detailed(extensions: &[radium_core::extensions::Extension]) {
    for ext in extensions {
        println!("  {} ({})", ext.name.bold().green(), ext.version.bright_black());
        if !ext.manifest.description.is_empty() {
            println!("    Description: {}", ext.manifest.description.bright_black());
        }
        if !ext.manifest.author.is_empty() {
            println!("    Author:      {}", ext.manifest.author.bright_black());
        }
        println!("    Components:  {} prompts, {} MCP servers, {} commands",
            ext.manifest.components.prompts.len(),
            ext.manifest.components.mcp_servers.len(),
            ext.manifest.components.commands.len(),
        );
        println!();
    }
}


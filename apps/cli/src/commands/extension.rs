//! Extension command implementation.
//!
//! Provides commands for installing, managing, and discovering extensions.

use super::ExtensionCommand;
use colored::Colorize;
use inquire::Confirm;
use radium_core::extensions::{ExtensionDiscovery, ExtensionManager, InstallOptions};
use serde_json::json;
use std::path::Path;
use tabled::{Table, Tabled, settings::Style};

/// Execute the extension command.
pub async fn execute(command: ExtensionCommand) -> anyhow::Result<()> {
    match command {
        ExtensionCommand::Install { source, overwrite, install_deps } => {
            install_extension(&source, overwrite, install_deps).await
        }
        ExtensionCommand::Uninstall { name } => uninstall_extension(&name).await,
        ExtensionCommand::List { json, verbose } => list_extensions(json, verbose).await,
        ExtensionCommand::Info { name, json } => show_extension_info(&name, json).await,
        ExtensionCommand::Search { query, json } => search_extensions(&query, json).await,
        ExtensionCommand::Create { name, author, description } => {
            create_extension(&name, author.as_deref(), description.as_deref()).await
        }
    }
}

/// Install an extension.
async fn install_extension(
    source: &str,
    overwrite: bool,
    install_deps: bool,
) -> anyhow::Result<()> {
    let manager = ExtensionManager::new()?;

    println!("{}", format!("Installing extension from: {}", source).yellow());

    let options = InstallOptions {
        overwrite,
        install_dependencies: install_deps,
        validate_after_install: true,
    };

    match manager.install_from_source(source, options) {
        Ok(extension) => {
            println!(
                "{}",
                format!("✓ Extension '{}' installed successfully", extension.name).green()
            );
            println!("  Version: {}", extension.version.bright_black());
            if !extension.manifest.description.is_empty() {
                println!("  Description: {}", extension.manifest.description.bright_black());
            }
            Ok(())
        }
        Err(e) => {
            println!("{}", format!("✗ Failed to install extension: {}", e).red());
            println!("{}", "Run 'rad extension install --help' for usage examples.".dimmed());
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
            println!("{}", "Run 'rad extension uninstall --help' for usage examples.".dimmed());
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

    let extension = extension.ok_or_else(|| {
        anyhow::anyhow!("Extension not found: {}. Use 'rad extension list' to see installed extensions.", name)
    })?;

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
        println!(
            "{}",
            format!("Found {} extension(s) matching '{}'", matches.len(), query).bold().green()
        );
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

    let table = Table::new(rows).with(Style::modern()).to_string();
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
        println!(
            "    Components:  {} prompts, {} MCP servers, {} commands",
            ext.manifest.components.prompts.len(),
            ext.manifest.components.mcp_servers.len(),
            ext.manifest.components.commands.len(),
        );
        println!();
    }
}

/// Create a new extension template.
async fn create_extension(
    name: &str,
    author: Option<&str>,
    description: Option<&str>,
) -> anyhow::Result<()> {
    use radium_core::extensions::manifest::{ExtensionManifest, ExtensionComponents};
    use std::fs;
    use std::path::Path;

    // Validate extension name
    if name.is_empty() {
        return Err(anyhow::anyhow!("Extension name cannot be empty"));
    }

    if !name.chars().next().unwrap_or(' ').is_alphabetic() {
        return Err(anyhow::anyhow!(
            "Extension name must start with a letter: '{}'",
            name
        ));
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(anyhow::anyhow!(
            "Extension name can only contain letters, numbers, dashes, and underscores: '{}'",
            name
        ));
    }

    // Check if directory already exists
    let extension_dir = Path::new(name);
    if extension_dir.exists() {
        return Err(anyhow::anyhow!(
            "Directory '{}' already exists. Choose a different name or remove the existing directory.",
            name
        ));
    }

    println!("{}", format!("Creating extension: {}", name).yellow());

    // Create extension directory
    fs::create_dir_all(extension_dir)?;

    // Create component directories
    let prompts_dir = extension_dir.join("prompts");
    let mcp_dir = extension_dir.join("mcp");
    let commands_dir = extension_dir.join("commands");
    let hooks_dir = extension_dir.join("hooks");

    fs::create_dir_all(&prompts_dir)?;
    fs::create_dir_all(&mcp_dir)?;
    fs::create_dir_all(&commands_dir)?;
    fs::create_dir_all(&hooks_dir)?;

    // Create manifest
    let manifest = ExtensionManifest {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: description.unwrap_or("A Radium extension").to_string(),
        author: author.unwrap_or("").to_string(),
        components: ExtensionComponents {
            prompts: vec!["prompts/*.md".to_string()],
            mcp_servers: vec!["mcp/*.json".to_string()],
            commands: vec!["commands/*.toml".to_string()],
            hooks: vec!["hooks/*.toml".to_string()],
        },
        dependencies: Vec::new(),
        metadata: std::collections::HashMap::new(),
    };

    let manifest_path = extension_dir.join("radium-extension.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, manifest_json)?;

    // Create README.md
    let readme_content = format!(
        r#"# {}

{}

## Installation

Install this extension:

```bash
rad extension install ./{}
```

## Components

This extension can contain:

- **Prompts**: Agent prompt templates in `prompts/`
- **MCP Servers**: MCP server configurations in `mcp/`
- **Commands**: Custom commands in `commands/`
- **Hooks**: Hook configurations in `hooks/`

## Documentation

See [Creating Extensions](../../docs/extensions/creating-extensions.md) for detailed information on building extensions.
"#,
        name,
        description.unwrap_or("A Radium extension"),
        name
    );

    let readme_path = extension_dir.join("README.md");
    fs::write(&readme_path, readme_content)?;

    println!(
        "{}",
        format!("✓ Extension '{}' created successfully", name).green()
    );
    println!();
    println!("  Directory: {}", extension_dir.display());
    println!("  Manifest: {}", manifest_path.display());
    println!();
    println!("Next steps:");
    println!("  1. Add your components to the extension directories");
    println!("  2. Test installation: rad extension install ./{}", name);
    println!("  3. See docs/extensions/creating-extensions.md for details");

    Ok(())
}

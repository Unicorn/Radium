//! Extension command implementation.
//!
//! Provides commands for installing, managing, and discovering extensions.

use super::ExtensionCommand;
use colored::Colorize;
use inquire::Confirm;
use radium_core::extensions::{
    ExtensionDiscovery, ExtensionManager, ExtensionPublisher, ExtensionSigner, InstallOptions,
    MarketplaceClient, MarketplaceExtension, PublishingError, SignatureVerifier,
    TrustedKeysManager,
};
use serde_json::json;
use std::fs;
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
        ExtensionCommand::Search { query, json, marketplace_only, local_only } => {
            search_extensions(&query, json, marketplace_only, local_only).await
        }
        ExtensionCommand::Browse { json } => browse_marketplace(json).await,
        ExtensionCommand::Create { name, author, description } => {
            create_extension(&name, author.as_deref(), description.as_deref()).await
        }
        ExtensionCommand::Sign { path, key_file, generate_key } => {
            sign_extension(&path, key_file.as_deref(), generate_key).await
        }
        ExtensionCommand::Verify { name_or_path, key_file } => {
            verify_extension(&name_or_path, key_file.as_deref()).await
        }
        ExtensionCommand::TrustKey { action, name, key_file } => {
            manage_trusted_keys(&action, name.as_deref(), key_file.as_deref()).await
        }
        ExtensionCommand::Publish { path, api_key, sign_with_key } => {
            publish_extension(&path, api_key.as_deref(), sign_with_key.as_deref()).await
        }
        ExtensionCommand::CheckUpdates { json } => {
            check_for_updates(json).await
        }
        ExtensionCommand::Update { name, all, dry_run } => {
            update_extension(name.as_deref(), all, dry_run).await
        }
        ExtensionCommand::Analytics { action, name, json } => {
            manage_analytics(&action, name.as_deref(), json).await
        }
        ExtensionCommand::Graph { name, format, conflicts_only } => {
            show_dependency_graph(name.as_deref(), &format, conflicts_only).await
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

    // Check if source is a marketplace name (not a path or URL)
    let actual_source: String = if !source.starts_with("http://")
        && !source.starts_with("https://")
        && !Path::new(source).exists()
        && !source.contains('/')
        && !source.contains('\\')
        && !source.ends_with(".tar.gz")
        && !source.ends_with(".zip") {
        // Likely a marketplace name, try to fetch from marketplace
        let mut marketplace_client = MarketplaceClient::new().unwrap_or_else(|_| {
            MarketplaceClient::with_url("http://localhost:8080".to_string()).unwrap()
        });

        if let Ok(Some(extension_info)) = marketplace_client.get_extension_info(source) {
            println!("{}", format!("Found extension '{}' in marketplace", source).green());
            println!("{}", format!("Downloading from: {}", extension_info.download_url).bright_black());
            extension_info.download_url
        } else {
            source.to_string()
        }
    } else {
        source.to_string()
    };

    println!("{}", format!("Installing extension from: {}", actual_source).yellow());
    println!("{}", "Validating extension package...".bright_black());

    // Check if extension already exists and prompt for overwrite if needed
    let mut should_overwrite = overwrite;
    if !overwrite {
        // Try to load manifest to get extension name
        use radium_core::extensions::manifest::ExtensionManifest;
        use std::path::Path;
        
        let source_path = if actual_source.starts_with("http://") || actual_source.starts_with("https://") {
            // For URLs, we can't check ahead of time, so skip the check
            None
        } else {
            Some(Path::new(actual_source))
        };
        
        if let Some(path) = source_path {
            if path.exists() {
                let manifest_path = if path.is_dir() {
                    path.join("radium-extension.json")
                } else {
                    // For archives, we can't easily check without extracting
                    // So we'll handle the error during installation
                    path.to_path_buf()
                };
                
                if manifest_path.exists() && manifest_path.is_file() {
                    if let Ok(manifest) = ExtensionManifest::load(&manifest_path) {
                        if let Ok(Some(existing)) = manager.get(&manifest.name) {
                            println!(
                                "{}",
                                format!("⚠ Extension '{}' is already installed", existing.name).yellow()
                            );
                            println!("  Installed version: {}", existing.version.bright_black());
                            println!("  New version: {}", manifest.version.bright_black());
                            
                            if let Ok(true) = Confirm::new("Overwrite existing extension?")
                                .with_default(false)
                                .prompt()
                            {
                                should_overwrite = true;
                            } else {
                                println!("{}", "Installation cancelled.".yellow());
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }

    let options = InstallOptions {
        overwrite: should_overwrite,
        install_dependencies: install_deps,
        validate_after_install: true,
    };

    println!("{}", "Installing extension files...".bright_black());

    match manager.install_from_source(actual_source, options) {
        Ok(extension) => {
            println!(
                "{}",
                format!("✓ Extension '{}' installed successfully", extension.name).green()
            );
            println!("  Version: {}", extension.version.bright_black());
            if !extension.manifest.description.is_empty() {
                println!("  Description: {}", extension.manifest.description.bright_black());
            }
            
            // Show component summary
            let component_count = extension.manifest.components.prompts.len()
                + extension.manifest.components.mcp_servers.len()
                + extension.manifest.components.commands.len()
                + extension.manifest.components.hooks.len();
            
            if component_count > 0 {
                println!("  Components: {} total", component_count);
            }
            
            if !extension.manifest.dependencies.is_empty() {
                println!("  Dependencies: {}", extension.manifest.dependencies.join(", "));
            }
            
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            println!("{}", format!("✗ Failed to install extension: {}", error_msg).red());
            
            // Provide helpful suggestions based on error type
            if error_msg.contains("manifest") || error_msg.contains("JSON") {
                println!();
                println!("{}", "💡 Troubleshooting tips:".yellow());
                println!("  • Ensure radium-extension.json is valid JSON");
                println!("  • Check that all required fields are present (name, version, description, author)");
                println!("  • Verify the manifest file exists in the extension root");
            } else if error_msg.contains("version") {
                println!();
                println!("{}", "💡 Troubleshooting tips:".yellow());
                println!("  • Version must follow semantic versioning (e.g., 1.0.0)");
                println!("  • Check the version field in radium-extension.json");
            } else if error_msg.contains("name") {
                println!();
                println!("{}", "💡 Troubleshooting tips:".yellow());
                println!("  • Extension name must start with a letter");
                println!("  • Name can only contain letters, numbers, dashes, and underscores");
                println!("  • Check the name field in radium-extension.json");
            } else if error_msg.contains("already installed") || error_msg.contains("conflict") {
                println!();
                println!("{}", "💡 Troubleshooting tips:".yellow());
                println!("  • Use --overwrite flag to replace existing extension");
                println!("  • Or uninstall the existing extension first: rad extension uninstall <name>");
            } else if error_msg.contains("dependency") {
                println!();
                println!("{}", "💡 Troubleshooting tips:".yellow());
                println!("  • Use --install-deps flag to automatically install dependencies");
                println!("  • Or install dependencies manually first");
            }
            
            println!();
            println!("{}", "Run 'rad extension install --help' for usage examples.".dimmed());
            Err(anyhow::anyhow!("Installation failed: {}", e))
        }
    }
}

/// Uninstall an extension.
async fn uninstall_extension(name: &str) -> anyhow::Result<()> {
    let manager = ExtensionManager::new()?;

    // Check if extension exists first
    if let Ok(Some(extension)) = manager.get(name) {
        println!("{}", format!("Uninstalling extension: {}", name).yellow());
        println!("  Version: {}", extension.version.bright_black());
        if !extension.manifest.description.is_empty() {
            println!("  Description: {}", extension.manifest.description.bright_black());
        }
        
        // Confirm uninstallation
        if let Ok(true) = Confirm::new(&format!("Are you sure you want to uninstall '{}'?", name))
            .with_default(false)
            .prompt()
        {
            match manager.uninstall(name) {
                Ok(()) => {
                    println!("{}", format!("✓ Extension '{}' uninstalled successfully", name).green());
                    Ok(())
                }
                Err(e) => {
                    println!("{}", format!("✗ Failed to uninstall extension: {}", e).red());
                    println!();
                    println!("{}", "💡 Troubleshooting tips:".yellow());
                    println!("  • Ensure the extension name is correct (case-sensitive)");
                    println!("  • Check that the extension directory exists");
                    println!();
                    println!("{}", "Run 'rad extension list' to see installed extensions.".dimmed());
                    Err(anyhow::anyhow!("Uninstallation failed: {}", e))
                }
            }
        } else {
            println!("{}", "Uninstallation cancelled.".yellow());
            Ok(())
        }
    } else {
        println!("{}", format!("✗ Extension '{}' not found", name).red());
        println!();
        println!("{}", "💡 Troubleshooting tips:".yellow());
        println!("  • Check the extension name is correct (case-sensitive)");
        println!("  • Use 'rad extension list' to see installed extensions");
        println!();
        Err(anyhow::anyhow!("Extension not found: {}", name))
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
            println!("{}", "Get started:".bright_black());
            println!("  Install from local directory:");
            println!("    $ rad extension install ./my-extension");
            println!();
            println!("  Install from URL:");
            println!("    $ rad extension install https://example.com/extension.tar.gz");
            println!();
            println!("  Create a new extension:");
            println!("    $ rad extension create my-extension --author \"Your Name\"");
            println!();
            println!("  See examples:");
            println!("    $ ls examples/extensions/");
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
async fn search_extensions(
    query: &str,
    json_output: bool,
    marketplace_only: bool,
    local_only: bool,
) -> anyhow::Result<()> {
    let mut local_matches = Vec::new();
    let mut marketplace_matches = Vec::new();

    // Search local extensions
    if !marketplace_only {
        let discovery = ExtensionDiscovery::new();
        local_matches = discovery.search(query)?;
    }

    // Search marketplace
    if !local_only {
        let mut marketplace_client = MarketplaceClient::new().unwrap_or_else(|_| {
            // If marketplace client fails to initialize, continue without it
            MarketplaceClient::with_url("http://localhost:8080".to_string()).unwrap()
        });
        
        if let Ok(matches) = marketplace_client.search_extensions(query) {
            marketplace_matches = matches;
        }
    }

    let total_matches = local_matches.len() + marketplace_matches.len();

    if total_matches == 0 {
        if !json_output {
            println!("{}", format!("No extensions found matching '{}'", query).yellow());
        }
        return Ok(());
    }

    if json_output {
        let mut extension_list = Vec::new();
        
        for ext in &local_matches {
            extension_list.push(json!({
                "name": ext.name,
                "version": ext.version,
                "description": ext.manifest.description,
                "author": ext.manifest.author,
                "source": "local",
            }));
        }
        
        for ext in &marketplace_matches {
            extension_list.push(json!({
                "name": ext.name,
                "version": ext.version,
                "description": ext.description,
                "author": ext.author,
                "source": "marketplace",
                "download_url": ext.download_url,
                "download_count": ext.download_count,
                "rating": ext.rating,
            }));
        }
        
        println!("{}", serde_json::to_string_pretty(&extension_list)?);
    } else {
        println!();
        println!(
            "{}",
            format!("Found {} extension(s) matching '{}'", total_matches, query).bold().green()
        );
        println!();

        if !local_matches.is_empty() {
            println!("{}", "Local Extensions:".bold());
            for ext in &local_matches {
                println!("  {} ({})", ext.name.bold(), ext.version.bright_black());
                if !ext.manifest.description.is_empty() {
                    println!("    {}", ext.manifest.description.bright_black());
                }
                println!();
            }
        }

        if !marketplace_matches.is_empty() {
            println!("{}", "Marketplace Extensions:".bold());
            for ext in &marketplace_matches {
                print!("  {} ({})", ext.name.bold(), ext.version.bright_black());
                if let Some(downloads) = ext.download_count {
                    print!(" - {} downloads", downloads);
                }
                if let Some(rating) = ext.rating {
                    print!(" - ⭐ {:.1}", rating);
                }
                println!();
                if !ext.description.is_empty() {
                    println!("    {}", ext.description.bright_black());
                }
                println!();
            }
        }
    }

    Ok(())
}

/// Browse popular extensions from the marketplace.
async fn browse_marketplace(json_output: bool) -> anyhow::Result<()> {
    let mut marketplace_client = MarketplaceClient::new().unwrap_or_else(|_| {
        MarketplaceClient::with_url("http://localhost:8080".to_string()).unwrap()
    });

    // Search for popular extensions (empty query to get all)
    let extensions = marketplace_client.search_extensions("")
        .unwrap_or_else(|_| Vec::new());

    if extensions.is_empty() {
        if !json_output {
            println!("{}", "No extensions available in marketplace.".yellow());
        }
        return Ok(());
    }

    // Sort by download count and rating
    let mut sorted_extensions = extensions;
    sorted_extensions.sort_by(|a, b| {
        let a_score = a.download_count.unwrap_or(0) as f64 + (a.rating.unwrap_or(0.0) * 100.0);
        let b_score = b.download_count.unwrap_or(0) as f64 + (b.rating.unwrap_or(0.0) * 100.0);
        b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
    });

    if json_output {
        let extension_list: Vec<_> = sorted_extensions
            .iter()
            .map(|ext| {
                json!({
                    "name": ext.name,
                    "version": ext.version,
                    "description": ext.description,
                    "author": ext.author,
                    "download_url": ext.download_url,
                    "download_count": ext.download_count,
                    "rating": ext.rating,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&extension_list)?);
    } else {
        println!("{}", format!("Popular Extensions ({})", sorted_extensions.len()).bold().green());
        println!();

        for ext in &sorted_extensions {
            print!("  {} ({})", ext.name.bold(), ext.version.bright_black());
            if let Some(downloads) = ext.download_count {
                print!(" - {} downloads", downloads);
            }
            if let Some(rating) = ext.rating {
                print!(" - ⭐ {:.1}", rating);
            }
            println!();
            if !ext.description.is_empty() {
                println!("    {}", ext.description.bright_black());
            }
            println!("    Install: rad extension install {}", ext.name);
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

/// Sign an extension.
async fn sign_extension(path: &str, key_file: Option<&str>, generate_key: bool) -> anyhow::Result<()> {
    let extension_path = Path::new(path);
    if !extension_path.exists() {
        return Err(anyhow::anyhow!("Extension path does not exist: {}", path));
    }

    let signer = if generate_key {
        println!("{}", "Generating new keypair...".bright_black());
        let (signer, public_key) = ExtensionSigner::generate();
        
        // Save keys
        let private_key_path = extension_path.join("private.key");
        let public_key_path = extension_path.join("public.key");
        
        fs::write(&private_key_path, signer.public_key())?;
        fs::write(&public_key_path, &public_key)?;
        
        println!("{}", format!("✓ Keypair generated and saved").green());
        println!("  Private key: {}", private_key_path.display());
        println!("  Public key: {}", public_key_path.display());
        println!();
        println!("{}", "⚠ Keep your private key secure!".yellow());
        
        signer
    } else if let Some(key_file) = key_file {
        let key_bytes = fs::read(key_file)?;
        ExtensionSigner::from_private_key(&key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to load private key: {}", e))?
    } else {
        return Err(anyhow::anyhow!("Either --key-file or --generate-key must be provided"));
    };

    println!("{}", format!("Signing extension: {}", path).yellow());
    
    let sig_path = signer.sign_extension(extension_path)
        .map_err(|e| anyhow::anyhow!("Failed to sign extension: {}", e))?;
    
    println!("{}", format!("✓ Extension signed successfully").green());
    println!("  Signature: {}", sig_path.display());
    
    Ok(())
}

/// Verify an extension signature.
async fn verify_extension(name_or_path: &str, key_file: Option<&str>) -> anyhow::Result<()> {
    let extension_path = if Path::new(name_or_path).exists() {
        Path::new(name_or_path).to_path_buf()
    } else {
        // Try as installed extension name
        let manager = ExtensionManager::new()?;
        let extension = manager.get(name_or_path)?
            .ok_or_else(|| anyhow::anyhow!("Extension not found: {}", name_or_path))?;
        extension.install_path
    };

    if !SignatureVerifier::has_signature(&extension_path) {
        println!("{}", format!("⚠ Extension has no signature file").yellow());
        return Ok(());
    }

    println!("{}", format!("Verifying extension signature: {}", name_or_path).yellow());

    if let Some(key_file) = key_file {
        // Verify with specific key
        let public_key = fs::read(key_file)?;
        SignatureVerifier::verify(&extension_path, &public_key)
            .map_err(|e| anyhow::anyhow!("Signature verification failed: {}", e))?;
        println!("{}", format!("✓ Signature verified successfully").green());
    } else {
        // Try with trusted keys
        let keys_manager = TrustedKeysManager::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize trusted keys manager: {}", e))?;
        
        let trusted_keys = keys_manager.list_trusted_keys()
            .map_err(|e| anyhow::anyhow!("Failed to list trusted keys: {}", e))?;
        
        if trusted_keys.is_empty() {
            return Err(anyhow::anyhow!("No trusted keys found. Use --key-file or add a trusted key with 'rad extension trust-key add'"));
        }

        let mut verified = false;
        for key_name in trusted_keys {
            if let Ok(public_key) = keys_manager.get_trusted_key(&key_name) {
                if SignatureVerifier::verify(&extension_path, &public_key).is_ok() {
                    println!("{}", format!("✓ Signature verified with trusted key: {}", key_name).green());
                    verified = true;
                    break;
                }
            }
        }

        if !verified {
            return Err(anyhow::anyhow!("Signature verification failed with all trusted keys"));
        }
    }

    Ok(())
}

/// Manage trusted public keys.
async fn manage_trusted_keys(action: &str, name: Option<&str>, key_file: Option<&str>) -> anyhow::Result<()> {
    let keys_manager = TrustedKeysManager::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize trusted keys manager: {}", e))?;

    match action {
        "add" => {
            let name = name.ok_or_else(|| anyhow::anyhow!("Key name required for 'add' action"))?;
            let key_file = key_file.ok_or_else(|| anyhow::anyhow!("--key-file required for 'add' action"))?;
            
            let public_key = fs::read(key_file)?;
            keys_manager.add_trusted_key(name, &public_key)
                .map_err(|e| anyhow::anyhow!("Failed to add trusted key: {}", e))?;
            
            println!("{}", format!("✓ Trusted key '{}' added successfully", name).green());
        }
        "list" => {
            let keys = keys_manager.list_trusted_keys()
                .map_err(|e| anyhow::anyhow!("Failed to list trusted keys: {}", e))?;
            
            if keys.is_empty() {
                println!("{}", "No trusted keys found.".yellow());
            } else {
                println!("{}", format!("Trusted Keys ({})", keys.len()).bold().green());
                for key in keys {
                    println!("  • {}", key);
                }
            }
        }
        "remove" => {
            let name = name.ok_or_else(|| anyhow::anyhow!("Key name required for 'remove' action"))?;
            
            keys_manager.remove_trusted_key(name)
                .map_err(|e| anyhow::anyhow!("Failed to remove trusted key: {}", e))?;
            
            println!("{}", format!("✓ Trusted key '{}' removed successfully", name).green());
        }
        _ => {
            return Err(anyhow::anyhow!("Invalid action: {}. Use 'add', 'list', or 'remove'", action));
        }
    }

    Ok(())
}

/// Publish an extension to the marketplace.
async fn publish_extension(
    path: &str,
    api_key: Option<&str>,
    sign_with_key: Option<&str>,
) -> anyhow::Result<()> {
    let extension_path = Path::new(path);
    if !extension_path.exists() {
        return Err(anyhow::anyhow!("Extension path does not exist: {}", path));
    }

    // Get API key
    let env_api_key = std::env::var("RADIUM_MARKETPLACE_API_KEY").ok();
    let api_key = api_key
        .or_else(|| env_api_key.as_deref())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "API key required. Provide --api-key or set RADIUM_MARKETPLACE_API_KEY environment variable"
            )
        })?;

    // Load signing key if provided
    let sign_key = if let Some(key_file) = sign_with_key {
        Some(fs::read(key_file)?)
    } else {
        None
    };

    println!("{}", format!("Publishing extension: {}", path).yellow());
    println!("{}", "Validating extension...".bright_black());

    let publisher = ExtensionPublisher::new()
        .map_err(|e| anyhow::anyhow!("Failed to create publisher: {}", e))?;

    // Validate
    publisher
        .validate_for_publish(extension_path)
        .map_err(|e| anyhow::anyhow!("Validation failed: {}", e))?;

    println!("{}", "✓ Extension validation passed".green());
    println!("{}", "Publishing to marketplace...".bright_black());

    // Publish
    let published = publisher
        .publish(extension_path, api_key, sign_key.as_deref())
        .map_err(|e| match e {
            PublishingError::Validation(msg) => anyhow::anyhow!("Validation error: {}", msg),
            PublishingError::Marketplace(e) => anyhow::anyhow!("Marketplace error: {}", e),
            PublishingError::Signing(msg) => anyhow::anyhow!("Signing error: {}", msg),
            e => anyhow::anyhow!("Publishing failed: {}", e),
        })?;

    println!("{}", format!("✓ Extension '{}' published successfully", published.name).green());
    println!("  Version: {}", published.version.bright_black());
    println!("  Download URL: {}", published.download_url);

    Ok(())
}

/// Check for available extension updates.
async fn check_for_updates(json: bool) -> anyhow::Result<()> {
    use radium_core::extensions::{ExtensionManager, MarketplaceClient, UpdateChecker};

    let manager = ExtensionManager::new()?;
    let extensions = manager.list()?;

    if extensions.is_empty() {
        if json {
            println!("[]");
        } else {
            println!("{}", "No extensions installed.".bright_black());
        }
        return Ok(());
    }

    let mut marketplace = MarketplaceClient::new().unwrap_or_else(|_| {
        MarketplaceClient::with_url("http://localhost:8080".to_string()).unwrap()
    });

    // Get latest versions from marketplace
    let get_latest = |name: &str| -> Option<(String, Option<String>, Option<String>)> {
        marketplace.get_extension_info(name).ok().flatten().map(|ext| {
            (ext.version, Some(ext.description), Some(ext.download_url))
        })
    };

    let updates = UpdateChecker::check_all_updates(&extensions, get_latest)
        .map_err(|e| anyhow::anyhow!("Failed to check updates: {}", e))?;

    if json {
        let json_updates: Vec<serde_json::Value> = updates.iter().map(|u| {
            json!({
                "name": u.name,
                "current_version": u.current_version,
                "new_version": u.new_version,
                "description": u.description,
                "download_url": u.download_url,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&json_updates)?);
    } else {
        if updates.is_empty() {
            println!("{}", "✓ All extensions are up to date.".green());
        } else {
            println!("{}", format!("Found {} available update(s):", updates.len()).yellow());
            println!();
            for update in &updates {
                println!("  {} {} → {}", 
                    update.name.bright_white().bold(),
                    update.current_version.bright_black(),
                    update.new_version.green().bold()
                );
                if let Some(desc) = &update.description {
                    println!("    {}", desc.bright_black());
                }
            }
            println!();
            println!("{}", "Run 'rad extension update <name>' to update an extension, or 'rad extension update --all' to update all.".bright_black());
        }
    }

    Ok(())
}

/// Update an extension or all extensions.
async fn update_extension(name: Option<&str>, all: bool, dry_run: bool) -> anyhow::Result<()> {
    use radium_core::extensions::{ExtensionManager, MarketplaceClient, InstallOptions, UpdateChecker};

    let manager = ExtensionManager::new()?;
    let mut marketplace = MarketplaceClient::new().unwrap_or_else(|_| {
        MarketplaceClient::with_url("http://localhost:8080".to_string()).unwrap()
    });

    if all {
        // Update all extensions
        let extensions = manager.list()?;
        let get_latest = |name: &str| -> Option<(String, Option<String>, Option<String>)> {
            marketplace.get_extension_info(name).ok().flatten().map(|ext| {
                (ext.version, Some(ext.description), Some(ext.download_url))
            })
        };

        let updates = UpdateChecker::check_all_updates(&extensions, get_latest)
            .map_err(|e| anyhow::anyhow!("Failed to check updates: {}", e))?;

        if updates.is_empty() {
            println!("{}", "✓ All extensions are up to date.".green());
            return Ok(());
        }

        if dry_run {
            println!("{}", format!("Would update {} extension(s):", updates.len()).yellow());
            for update in &updates {
                println!("  {} {} → {}", 
                    update.name.bright_white().bold(),
                    update.current_version.bright_black(),
                    update.new_version.green().bold()
                );
            }
            return Ok(());
        }

        println!("{}", format!("Updating {} extension(s)...", updates.len()).yellow());
        let mut updated = 0;
        let mut failed = 0;

        for update in &updates {
            print!("  Updating {}... ", update.name.bright_white());
            match update_single_extension(&manager, &mut marketplace, &update.name, &update.download_url.as_ref().unwrap()).await {
                Ok(_) => {
                    println!("{}", "✓".green());
                    updated += 1;
                }
                Err(e) => {
                    println!("{} {}", "✗".red(), e.to_string().bright_red());
                    failed += 1;
                }
            }
        }

        println!();
        println!("{}", format!("✓ Updated {} extension(s)", updated).green());
        if failed > 0 {
            println!("{}", format!("✗ Failed to update {} extension(s)", failed).red());
        }
    } else if let Some(ext_name) = name {
        // Update single extension
        if dry_run {
            if let Some(ext_info) = marketplace.get_extension_info(ext_name)? {
                if let Some(installed) = manager.get(ext_name)? {
                    if UpdateChecker::check_for_update(&installed, &ext_info.version)? {
                        println!("{}", format!("Would update {} {} → {}", 
                            ext_name.bright_white().bold(),
                            installed.version.bright_black(),
                            ext_info.version.green().bold()
                        ).yellow());
                    } else {
                        println!("{}", format!("{} is already up to date (v{})", ext_name, installed.version).bright_black());
                    }
                } else {
                    return Err(anyhow::anyhow!("Extension '{}' not found", ext_name));
                }
            } else {
                return Err(anyhow::anyhow!("Extension '{}' not found in marketplace", ext_name));
            }
        } else {
            if let Some(ext_info) = marketplace.get_extension_info(ext_name)? {
                update_single_extension(&manager, &mut marketplace, ext_name, &ext_info.download_url).await?;
                println!("{}", format!("✓ Extension '{}' updated successfully", ext_name).green());
            } else {
                return Err(anyhow::anyhow!("Extension '{}' not found in marketplace", ext_name));
            }
        }
    } else {
        return Err(anyhow::anyhow!("Either specify an extension name or use --all"));
    }

    Ok(())
}

/// Helper function to update a single extension.
async fn update_single_extension(
    manager: &ExtensionManager,
    _marketplace: &mut MarketplaceClient,
    name: &str,
    download_url: &str,
) -> anyhow::Result<()> {
    use radium_core::extensions::InstallOptions;

    let options = InstallOptions {
        overwrite: true,
        install_dependencies: true,
        validate_after_install: true,
    };

    manager.install_from_source(download_url, options)
        .map_err(|e| anyhow::anyhow!("Update failed: {}", e))?;

    Ok(())
}

/// Manage extension analytics.
async fn manage_analytics(action: &str, name: Option<&str>, json: bool) -> anyhow::Result<()> {
    use radium_core::extensions::ExtensionAnalyticsService;
    use std::path::PathBuf;

    // Get analytics data directory
    let data_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?
        .join(".radium")
        .join("analytics");

    let mut service = ExtensionAnalyticsService::new(data_dir);

    match action {
        "status" => {
            if json {
                println!("{}", serde_json::json!({
                    "enabled": service.is_enabled()
                }));
            } else {
                if service.is_enabled() {
                    println!("{}", "Analytics: Enabled".green());
                } else {
                    println!("{}", "Analytics: Disabled".bright_black());
                    println!("{}", "Run 'rad extension analytics opt-in' to enable.".bright_black());
                }
            }
        }
        "opt-in" => {
            service.enable()
                .map_err(|e| anyhow::anyhow!("Failed to enable analytics: {}", e))?;
            if !json {
                println!("{}", "✓ Analytics enabled".green());
                println!("{}", "Extension usage data will be collected locally.".bright_black());
            }
        }
        "opt-out" => {
            service.disable()
                .map_err(|e| anyhow::anyhow!("Failed to disable analytics: {}", e))?;
            if !json {
                println!("{}", "✓ Analytics disabled".green());
                println!("{}", "All analytics data has been cleared.".bright_black());
            }
        }
        "view" => {
            if !service.is_enabled() {
                return Err(anyhow::anyhow!("Analytics is not enabled. Run 'rad extension analytics opt-in' first."));
            }

            if let Some(ext_name) = name {
                let analytics = service.get_analytics(ext_name)
                    .map_err(|e| anyhow::anyhow!("Failed to get analytics: {}", e))?;

                if json {
                    println!("{}", serde_json::to_string_pretty(&analytics)?);
                } else {
                    println!("{}", format!("Analytics for '{}':", ext_name).yellow());
                    println!("  Installs: {}", analytics.install_count);
                    println!("  Uninstalls: {}", analytics.uninstall_count);
                    println!("  Usage events: {}", analytics.usage_count);
                    println!("  Error events: {}", analytics.error_count);
                    if let Some(first) = analytics.first_installed {
                        println!("  First installed: {}", first.format("%Y-%m-%d %H:%M:%S"));
                    }
                    if let Some(last) = analytics.last_used {
                        println!("  Last used: {}", last.format("%Y-%m-%d %H:%M:%S"));
                    }
                }
            } else {
                let all_analytics = service.get_all_analytics()
                    .map_err(|e| anyhow::anyhow!("Failed to get analytics: {}", e))?;

                if json {
                    println!("{}", serde_json::to_string_pretty(&all_analytics)?);
                } else {
                    if all_analytics.is_empty() {
                        println!("{}", "No analytics data available.".bright_black());
                    } else {
                        println!("{}", format!("Analytics for {} extension(s):", all_analytics.len()).yellow());
                        for analytics in all_analytics {
                            println!("  {}: {} installs, {} usage events", 
                                analytics.extension_name.bright_white(),
                                analytics.install_count,
                                analytics.usage_count
                            );
                        }
                    }
                }
            }
        }
        "clear" => {
            service.clear_data()
                .map_err(|e| anyhow::anyhow!("Failed to clear analytics: {}", e))?;
            if !json {
                println!("{}", "✓ Analytics data cleared".green());
            }
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown action: {}. Use: status, opt-in, opt-out, view, clear", action));
        }
    }

    Ok(())
}

/// Show extension dependency graph.
async fn show_dependency_graph(name: Option<&str>, format: &str, conflicts_only: bool) -> anyhow::Result<()> {
    use radium_core::extensions::{ExtensionManager, DependencyGraph};

    let manager = ExtensionManager::new()?;
    let extensions = manager.list()?;

    if extensions.is_empty() {
        println!("{}", "No extensions installed.".bright_black());
        return Ok(());
    }

    let graph = DependencyGraph::from_extensions(&extensions);

    if conflicts_only {
        let cycles = graph.detect_cycles();
        if cycles.is_empty() {
            println!("{}", "✓ No circular dependencies detected".green());
        } else {
            println!("{}", format!("Found {} circular dependency chain(s):", cycles.len()).yellow());
            for cycle in cycles {
                println!("  {}", cycle.join(" -> "));
            }
        }
        return Ok(());
    }

    match format {
        "dot" => {
            println!("{}", graph.to_dot());
        }
        "json" => {
            println!("{}", serde_json::to_string_pretty(&graph.to_json())?);
        }
        "ascii" | _ => {
            if let Some(ext_name) = name {
                println!("{}", format!("Dependency graph for '{}':", ext_name).yellow());
                println!("{}", graph.to_ascii_tree(Some(ext_name)));
            } else {
                println!("{}", "Extension Dependency Graph:".yellow());
                println!("{}", graph.to_ascii_tree(None));
            }
        }
    }

    Ok(())
}

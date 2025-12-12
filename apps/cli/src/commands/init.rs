//! Init command implementation.
//!
//! Initializes a new Radium workspace.

use anyhow::{Context, Result};
use colored::Colorize;
use radium_core::Workspace;
use std::env;
use std::fs;
use std::path::PathBuf;

use crate::colors::RadiumBrandColors;

/// Execute the init command.

pub async fn execute(
    path: Option<String>,
    use_defaults: bool,
    with_context: bool,
    sandbox: Option<String>,
    sandbox_network: Option<String>,
) -> Result<()> {
    let colors = RadiumBrandColors::new();
    println!("{}", "rad init".bold().color(colors.primary()));

    println!();

    // Determine target directory

    let current_dir = env::current_dir().context("Failed to get current directory")?;

    let target_path = if let Some(p) = path {
        PathBuf::from(p)
    } else if use_defaults {
        current_dir.clone()
    } else {
        // Interactive mode

        // Detect VCS

        let is_git_repo = current_dir.join(".git").exists();

        let _is_git_root = is_git_repo; // Simplified check - assumes .git is in current dir

        // TODO: Deeper git check could check parent dirs, but for now we check CWD.

        // If we are inside a git repo but not at root, we might want to warn.

        // For now, prompt for confirmation or path

        println!("This command will initialize a new Radium workspace.");

        let _default_path = ".radium";

        if is_git_repo {
            println!("{} Git repository detected.", "•".color(colors.primary()));
        }

        // We interpret the "path" as where the WORKSPACE ROOT is.

        // Radium workspace creates a .radium folder INSIDE the root.

        // So if user accepts current dir, we create ./radium/ and ./backlog/ etc.

        let confirm = inquire::Confirm::new("Initialize workspace in current directory?")
            .with_default(true)
            .prompt()?;

        if confirm {
            current_dir.clone()
        } else {
            let input = inquire::Text::new("Enter workspace path:").with_default(".").prompt()?;

            PathBuf::from(input)
        }
    };

    // Normalize path
    let target_path =
        if target_path.is_absolute() { target_path } else { current_dir.join(target_path) };

    println!();
    println!("Initializing workspace at: {}", target_path.display().to_string().color(colors.primary()));

    // Check if already initialized
    if target_path.join(".radium").exists() {
        println!("{} Workspace already initialized.", "!".color(colors.warning()));
        return Ok(());
    }

    // Create workspace
    println!("{}", "Creating workspace structure...".bold());

    // Use Workspace::create which ensures structure
    match Workspace::create(&target_path) {
        Ok(_) => {
            println!("  ✓ Created .radium directory");
            println!("  ✓ Created _internals directory");
            println!("  ✓ Created plan directory structure (backlog, development, etc.)");
        }
        Err(e) => {
            anyhow::bail!("Failed to create workspace: {}", e);
        }
    }

    // Create context file if requested
    if with_context {
        let context_file_path = target_path.join("GEMINI.md");
        
        // Try to find template file in various locations
        let template_content = find_template_content()?;
        
        match fs::write(&context_file_path, template_content) {
            Ok(_) => {
                println!("  ✓ Created GEMINI.md context file");
            }
            Err(e) => {
                println!("  {} Failed to create GEMINI.md: {}", "!".color(colors.warning()), e);
            }
        }
    }

    // Configure sandbox if requested
    if let Some(sandbox_type_str) = sandbox {
        use radium_core::sandbox::{NetworkMode, SandboxConfig, SandboxFactory, SandboxType};
        use toml;

        // Parse sandbox type
        let sandbox_type_enum = match sandbox_type_str.to_lowercase().as_str() {
            "none" => SandboxType::None,
            "docker" => SandboxType::Docker,
            "podman" => SandboxType::Podman,
            "seatbelt" => SandboxType::Seatbelt,
            _ => {
                    println!("  {} Invalid sandbox type: {}, using 'none'", "!".color(colors.warning()), sandbox_type_str);
                SandboxType::None
            }
        };

        // Parse network mode
        let network_mode = if let Some(net) = sandbox_network {
            match net.to_lowercase().as_str() {
                "open" => NetworkMode::Open,
                "closed" => NetworkMode::Closed,
                "proxied" => NetworkMode::Proxied,
                _ => {
                    println!("  {} Invalid network mode: {}, using 'open'", "!".color(colors.warning()), net);
                    NetworkMode::Open
                }
            }
        } else {
            NetworkMode::Open // Default
        };

        // Validate sandbox availability
        let test_config = SandboxConfig::new(sandbox_type_enum.clone());
        match SandboxFactory::create(&test_config) {
            Ok(_) => {
                // Sandbox is available
            }
            Err(e) => {
                if matches!(e, radium_core::sandbox::SandboxError::NotAvailable(_)) {
                    println!("  {} Warning: Sandbox type '{}' is not available on this system", "!".color(colors.warning()), sandbox_type_str);
                    println!("  {} Configuration will be saved, but sandbox will not be used until available.", " ".dimmed());
                } else {
                    println!("  {} Failed to validate sandbox: {}", "!".color(colors.warning()), e);
                }
            }
        }

        // Create sandbox config
        let sandbox_config = SandboxConfig::new(sandbox_type_enum)
            .with_network(network_mode);

        // Load or create workspace config
        let config_path = target_path.join(".radium").join("config.toml");
        let mut workspace_config: toml::Value = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            toml::Value::Table(toml::map::Map::new())
        };

        // Update sandbox section
        // Convert SandboxConfig to toml::Value via string serialization
        let sandbox_str = toml::to_string(&sandbox_config)?;
        let sandbox_table: toml::Value = toml::from_str(&sandbox_str)?;
        workspace_config
            .as_table_mut()
            .ok_or_else(|| anyhow::anyhow!("Invalid config format"))?
            .insert("sandbox".to_string(), sandbox_table);

        // Write config back
        let config_str = toml::to_string_pretty(&workspace_config)?;
        fs::write(&config_path, config_str)?;

        println!("  ✓ Configured sandbox: {}", sandbox_type_str.bold());
    }

    println!();
    println!("{}", "Workspace initialized successfully!".color(colors.success()).bold());
    println!();
    
    if with_context {
        println!("{}", "Next steps:".bold());
        println!("  1. Customize {} with your project guidelines", "GEMINI.md".color(colors.primary()));
        println!("  2. Create a plan: {}", "rad plan <spec-file>".color(colors.primary()));
        println!("  3. Execute a plan: {}", "rad craft <plan-id>".color(colors.primary()));
    } else {
        println!("{}", "Next steps:".bold());
        println!("  1. Create a plan: {}", "rad plan <spec-file>".color(colors.primary()));
        println!("  2. Execute a plan: {}", "rad craft <plan-id>".color(colors.primary()));
        println!();
        println!("  {} Tip: Add a {} file for persistent agent instructions", "•".dimmed(), "GEMINI.md".color(colors.primary()));
    }
    println!();

    Ok(())
}

/// Find template content from various possible locations.
fn find_template_content() -> Result<String> {
    // Try to find template file in common locations
    
    // 1. Try relative to current directory (for development/running from repo)
    let current_dir = env::current_dir().ok();
    if let Some(dir) = &current_dir {
        let template_path = dir.join("templates").join("GEMINI.md.template");
        if template_path.exists() {
            return Ok(fs::read_to_string(&template_path)?);
        }
    }
    
    // 2. Try relative to workspace root if we're in a workspace
    if let Ok(workspace) = Workspace::discover() {
        let template_path = workspace.root().join("templates").join("GEMINI.md.template");
        if template_path.exists() {
            return Ok(fs::read_to_string(&template_path)?);
        }
    }
    
    // 3. Fallback to embedded template content (embedded at compile time)
    // Path is relative to this source file: apps/cli/src/commands/init.rs
    // Going up: commands -> src -> cli -> apps -> root, then templates/
    Ok(include_str!("../../../../templates/GEMINI.md.template").to_string())
}

//! Init command implementation.
//!
//! Initializes a new Radium workspace.

use anyhow::{Context, Result};
use colored::Colorize;
use radium_core::Workspace;
use std::env;
use std::path::PathBuf;

/// Execute the init command.

pub async fn execute(path: Option<String>, use_defaults: bool) -> Result<()> {
    println!("{}", "rad init".bold().cyan());

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
            println!("{} Git repository detected.", "•".cyan());
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
    println!("Initializing workspace at: {}", target_path.display().to_string().cyan());

    // Check if already initialized
    if target_path.join(".radium").exists() {
        println!("{} Workspace already initialized.", "!".yellow());
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

    // TODO: Generate default config file if needed
    // let config_path = target_path.join(".radium").join("config.toml");
    // ...

    println!();
    println!("{}", "Workspace initialized successfully!".green().bold());
    println!();
    println!("Next steps:");
    println!("  1. Create a plan: {}", "rad plan <spec-file>".cyan());
    println!("  2. Execute a plan: {}", "rad craft <plan-id>".cyan());
    println!();

    Ok(())
}

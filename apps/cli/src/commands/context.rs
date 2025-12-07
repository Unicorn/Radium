//! Context file management commands.
//!
//! Provides commands for discovering, listing, and validating context files (GEMINI.md).

use anyhow::{Context, Result};
use colored::Colorize;
use radium_core::context::{generate_template, ContextFileLoader, TemplateType};
use radium_core::Workspace;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Execute the context command.
pub async fn execute(command: ContextCommand) -> Result<()> {
    match command {
        ContextCommand::List => list_context_files().await,
        ContextCommand::Show { path } => show_context_for_path(&path).await,
        ContextCommand::Validate => validate_context_files().await,
        ContextCommand::Init {
            template,
            global,
            path,
        } => init_context_file(&template, global, path.as_deref()).await,
    }
}

/// List all context files in the workspace.
async fn list_context_files() -> Result<()> {
    println!("{}", "Context Files".bold().cyan());
    println!();

    let workspace = Workspace::discover().ok();
    let workspace_root = workspace
        .as_ref()
        .map(|w| w.root().to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let loader = ContextFileLoader::new(&workspace_root);
    let files = loader.discover_context_files().context("Failed to discover context files")?;

    if files.is_empty() {
        println!("  {}", "No context files found.".yellow());
        println!();
        println!("  {}", "Context files (GEMINI.md) can be placed in:".dimmed());
        println!("    • Project root");
        println!("    • Subdirectories (for hierarchical loading)");
        println!("    • ~/.radium/ (for global context)");
        return Ok(());
    }

    println!("  {} Found {} context file(s)", "✓".green(), files.len());
    println!();

    for (i, file) in files.iter().enumerate() {
        let metadata = fs::metadata(file).ok();
        let size = metadata
            .and_then(|m| m.len().try_into().ok())
            .unwrap_or(0);
        let size_str = if size > 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else {
            format!("{} B", size)
        };

        // Determine file type
        let file_type = if file.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|n| n == ".radium")
            .unwrap_or(false)
        {
            "global".cyan()
        } else if file.parent()
            .and_then(|p| p.to_str())
            .map(|p| p == workspace_root.to_str().unwrap_or(""))
            .unwrap_or(false)
        {
            "project".green()
        } else {
            "subdirectory".yellow()
        };

        println!(
            "  {} {} ({}) - {}",
            format!("{}.", i + 1).dimmed(),
            file.display().to_string().cyan(),
            file_type,
            size_str.dimmed()
        );
    }

    println!();
    Ok(())
}

/// Show which context files would be loaded for a given path.
async fn show_context_for_path(path_str: &str) -> Result<()> {
    println!("{}", "Context Files for Path".bold().cyan());
    println!();

    let workspace = Workspace::discover().ok();
    let workspace_root = workspace
        .as_ref()
        .map(|w| w.root().to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let loader = ContextFileLoader::new(&workspace_root);

    // Resolve path
    let target_path = if Path::new(path_str).is_absolute() {
        PathBuf::from(path_str)
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| workspace_root.clone())
            .join(path_str)
    };

    if !target_path.exists() {
        println!("  {} Path not found: {}", "✗".red(), path_str);
        return Ok(());
    }

    let context_file_paths = loader.get_context_file_paths(&target_path);

    if context_file_paths.is_empty() {
        println!("  {} No context files would be loaded for: {}", "!".yellow(), path_str);
        println!();
        println!("  {}", "Context files are loaded hierarchically:".dimmed());
        println!("    1. Global: ~/.radium/GEMINI.md");
        println!("    2. Project: <workspace>/GEMINI.md");
        println!("    3. Subdirectory: <path>/GEMINI.md");
        return Ok(());
    }

    println!("  {} Context files for: {}", "✓".green(), path_str.cyan());
    println!();
    println!("  {}", "Loading order (precedence: lowest to highest):".dimmed());
    println!();

    for (i, file_path) in context_file_paths.iter().enumerate() {
        let precedence = match i {
            0 => "1. Global (lowest)".dimmed(),
            1 => "2. Project".green(),
            2 => "3. Subdirectory (highest)".yellow(),
            _ => format!("{}.", i + 1).dimmed(),
        };

        let metadata = fs::metadata(file_path).ok();
        let size = metadata
            .and_then(|m| m.len().try_into().ok())
            .unwrap_or(0);
        let size_str = if size > 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else {
            format!("{} B", size)
        };

        println!("  {} {} ({})", precedence, file_path.display().to_string().cyan(), size_str.dimmed());
    }

    // Show merged content preview
    println!();
    println!("  {}", "Merged content preview:".dimmed());
    let content = loader.load_hierarchical(&target_path).unwrap_or_default();
    if content.is_empty() {
        println!("    {}", "(empty)".dimmed());
    } else {
        let preview = if content.len() > 200 {
            format!("{}...\n    [truncated {} bytes]", &content[..200], content.len() - 200)
        } else {
            content
        };
        for line in preview.lines().take(10) {
            println!("    {}", line.dimmed());
        }
        if preview.lines().count() > 10 {
            println!("    {} ... ({} more lines)", "".dimmed(), preview.lines().count() - 10);
        }
    }

    println!();
    Ok(())
}

/// Validate all context files in the workspace.
async fn validate_context_files() -> Result<()> {
    println!("{}", "Validating Context Files".bold().cyan());
    println!();

    let workspace = Workspace::discover().ok();
    let workspace_root = workspace
        .as_ref()
        .map(|w| w.root().to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let loader = ContextFileLoader::new(&workspace_root);
    let files = loader.discover_context_files().context("Failed to discover context files")?;

    if files.is_empty() {
        println!("  {} No context files found to validate.", "!".yellow());
        return Ok(());
    }

    println!("  {} Validating {} context file(s)...", "•".cyan(), files.len());
    println!();

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for file in &files {
        // Check if file is readable
        let content = match fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                errors.push((file.clone(), format!("Cannot read file: {}", e)));
                continue;
            }
        };

        // Check for imports and validate them
        if content.contains('@') {
            // Try to process imports to detect circular dependencies and missing files
            let base_path = file.parent().unwrap_or(&workspace_root);
            match loader.process_imports(&content, base_path) {
                Ok(_) => {
                    // Imports are valid
                }
                Err(e) => {
                    errors.push((file.clone(), format!("Import error: {}", e)));
                }
            }
        }

        // Basic syntax checks
        if content.trim().is_empty() {
            warnings.push((file.clone(), "File is empty".to_string()));
        }
    }

    // Report results
    if errors.is_empty() && warnings.is_empty() {
        println!("  {} All context files are valid!", "✓".green());
        println!();
    } else {
        if !errors.is_empty() {
            println!("  {} Found {} error(s):", "✗".red(), errors.len());
            println!();
            for (file, error) in &errors {
                println!("    {} {}", file.display().to_string().red(), error.red());
            }
            println!();
        }

        if !warnings.is_empty() {
            println!("  {} Found {} warning(s):", "!".yellow(), warnings.len());
            println!();
            for (file, warning) in &warnings {
                println!("    {} {}", file.display().to_string().yellow(), warning.yellow());
            }
            println!();
        }
    }

    Ok(())
}

/// Initialize a context file from a template.
async fn init_context_file(template_str: &str, global: bool, custom_path: Option<&str>) -> Result<()> {
    println!("{}", "Initialize Context File".bold().cyan());
    println!();

    // Parse template type
    let template_type = TemplateType::from_str(template_str)
        .ok_or_else(|| anyhow::anyhow!("Invalid template type: {}. Available: basic, coding-standards, architecture, team-conventions", template_str))?;

    // Determine target path
    let target_path = if global {
        // Global context file: ~/.radium/GEMINI.md
        let home = std::env::var("HOME")
            .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
        let radium_dir = PathBuf::from(home).join(".radium");
        fs::create_dir_all(&radium_dir).context("Failed to create ~/.radium directory")?;
        radium_dir.join("GEMINI.md")
    } else if let Some(path_str) = custom_path {
        // Custom path provided
        let path = Path::new(path_str);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(path)
        }
    } else {
        // Project root: discover workspace or use current directory
        let workspace = Workspace::discover().ok();
        let workspace_root = workspace
            .as_ref()
            .map(|w| w.root().to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        workspace_root.join("GEMINI.md")
    };

    // Check if file already exists
    if target_path.exists() {
        print!(
            "  {} File already exists: {}\n  Overwrite? (y/N): ",
            "!".yellow(),
            target_path.display().to_string().cyan()
        );
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") && !input.trim().eq_ignore_ascii_case("yes") {
            println!("  {} Cancelled.", "•".dimmed());
            return Ok(());
        }
    }

    // Generate template content
    let content = generate_template(template_type);

    // Write file
    fs::write(&target_path, content).context("Failed to write context file")?;

    // Success message
    println!();
    println!("  {} Created context file: {}", "✓".green(), target_path.display().to_string().cyan());
    println!("  {} Template: {}", "•".dimmed(), template_type.as_str().cyan());
    println!("  {} Description: {}", "•".dimmed(), template_type.description().dimmed());
    println!();
    println!("  {}", "Next steps:".dimmed());
    println!("    • Edit the file to customize it for your project");
    println!("    • Use `rad context validate` to check for issues");
    println!("    • See `docs/features/context-files.md` for more information");
    println!();

    Ok(())
}

/// Context command subcommands.
#[derive(clap::Subcommand, Debug, Clone)]
pub enum ContextCommand {
    /// List all context files in workspace.
    List,
    /// Show which context files would be loaded for a path.
    Show { path: String },
    /// Validate all context files.
    Validate,
    /// Initialize a context file from a template.
    Init {
        /// Template type to use (basic, coding-standards, architecture, team-conventions)
        #[arg(short, long, default_value = "basic")]
        template: String,
        /// Create global context file (~/.radium/GEMINI.md) instead of project file
        #[arg(short, long)]
        global: bool,
        /// Custom path for the context file (defaults to project root)
        #[arg(short, long)]
        path: Option<String>,
    },
}


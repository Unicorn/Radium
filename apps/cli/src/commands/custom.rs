//! Custom commands CLI implementation.
//!
//! Provides commands for listing, executing, creating, and validating custom commands.

use super::CustomCommand;
use anyhow::{Context, bail};
use colored::Colorize;
use radium_core::commands::CommandRegistry;
use radium_core::hooks::loader::HookLoader;
use radium_core::hooks::registry::HookRegistry;
use radium_core::Workspace;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tabled::{Table, Tabled, settings::Style};

/// Execute the custom command.
pub async fn execute(command: CustomCommand) -> anyhow::Result<()> {
    match command {
        CustomCommand::List { namespace, verbose } => list_commands(namespace, verbose).await,
        CustomCommand::Execute { name, args } => execute_command(&name, &args).await,
        CustomCommand::Create {
            name,
            description,
            template,
            user,
            namespace,
        } => create_command(&name, description.as_deref(), template.as_deref(), user, namespace.as_deref()).await,
        CustomCommand::Validate { name, verbose } => validate_commands(name.as_deref(), verbose).await,
    }
}

/// List all available custom commands.
async fn list_commands(namespace_filter: Option<String>, verbose: bool) -> anyhow::Result<()> {
    // Discover workspace root
    let workspace = Workspace::discover().ok();
    let workspace_root = workspace
        .as_ref()
        .map(|w| w.root().to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Get user home directory
    let user_home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get user home directory"))?;
    let user_commands_dir = user_home.join(".radium").join("commands");

    // Project commands directory
    let project_commands_dir = workspace_root.join(".radium").join("commands");

    // Initialize registry
    let mut registry = CommandRegistry::new()
        .with_project_dir(&project_commands_dir)
        .with_user_dir(&user_commands_dir);

    // Discover commands
    registry.discover().context("Failed to discover custom commands")?;

    if registry.is_empty() {
        println!("{}", "No custom commands found.".yellow());
        println!();
        println!("Try creating commands in:");
        println!("  • .radium/commands/ (project-local)");
        println!("  • ~/.radium/commands/ (user-level)");
        return Ok(());
    }

    // Collect commands with metadata
    let mut commands: Vec<CommandInfo> = Vec::new();
    for name in registry.list() {
        let cmd = registry.get(&name).unwrap();
        
        // Apply namespace filter if specified
        if let Some(filter) = &namespace_filter {
            let cmd_namespace = cmd.namespace.as_deref().unwrap_or("");
            if !cmd_namespace.contains(filter) && !name.contains(filter) {
                continue;
            }
        }

        // Determine source
        let source = if name.contains(':') && !name.starts_with("git:") && !name.starts_with("docker:") {
            "extension"
        } else if project_commands_dir.exists() {
            // Check if command exists in project directory
            let cmd_file = project_commands_dir.join(format!("{}.toml", cmd.name));
            if cmd_file.exists() || cmd.namespace.is_some() {
                // Check namespace directory
                if let Some(ns) = &cmd.namespace {
                    let ns_dir = project_commands_dir.join(ns.replace(':', "/"));
                    let ns_cmd_file = ns_dir.join(format!("{}.toml", cmd.name));
                    if ns_cmd_file.exists() {
                        "project"
                    } else {
                        "user"
                    }
                } else if cmd_file.exists() {
                    "project"
                } else {
                    "user"
                }
            } else {
                "user"
            }
        } else {
            "user"
        };

        commands.push(CommandInfo {
            name: name.clone(),
            namespace: cmd.namespace.clone(),
            source: source.to_string(),
            description: cmd.description.clone(),
        });
    }

    // Sort by name
    commands.sort_by(|a, b| a.name.cmp(&b.name));

    println!();
    println!("{}", format!("📦 Found {} custom commands", commands.len()).bold().green());
    println!();

    if verbose {
        display_commands_detailed(&commands);
    } else {
        display_commands_table(&commands);
    }

    Ok(())
}

/// Command information for display.
#[derive(Tabled)]
struct CommandInfo {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Namespace")]
    #[tabled(display_with = "display_option")]
    namespace: Option<String>,
    #[tabled(rename = "Source")]
    source: String,
    #[tabled(rename = "Description")]
    description: String,
}

/// Helper to display Option<String> in table.
fn display_option(opt: &Option<String>) -> String {
    opt.as_deref().unwrap_or("-").to_string()
}

/// Display commands in a table format.
fn display_commands_table(commands: &[CommandInfo]) {
    let table = Table::new(commands)
        .with(Style::rounded())
        .to_string();
    println!("{}", table);
}

/// Display commands in detailed format.
fn display_commands_detailed(commands: &[CommandInfo]) {
    for cmd in commands {
        println!("{}", format!("{}", cmd.name).bold().cyan());
        if let Some(ns) = &cmd.namespace {
            println!("  Namespace: {}", ns.dimmed());
        }
        println!("  Source: {}", cmd.source.dimmed());
        if !cmd.description.is_empty() {
            println!("  Description: {}", cmd.description);
        }
        println!();
    }
}

/// Execute a custom command.
async fn execute_command(name: &str, args: &[String]) -> anyhow::Result<()> {
    println!("{}", format!("rad custom execute {}", name).bold().cyan());
    println!();

    // Discover workspace root
    let workspace = Workspace::discover().ok();
    let workspace_root = workspace
        .as_ref()
        .map(|w| w.root().to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Get user home directory
    let user_home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get user home directory"))?;
    let user_commands_dir = user_home.join(".radium").join("commands");

    // Project commands directory
    let project_commands_dir = workspace_root.join(".radium").join("commands");

    // Initialize registry
    let mut registry = CommandRegistry::new()
        .with_project_dir(&project_commands_dir)
        .with_user_dir(&user_commands_dir);

    // Discover commands
    registry.discover().context("Failed to discover custom commands")?;

    // Look up command
    let command = registry
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("Command '{}' not found", name))?;

    println!("  {} Command: {}", "•".dimmed(), name.cyan());
    if !command.description.is_empty() {
        println!("  {} Description: {}", "•".dimmed(), command.description.dimmed());
    }
    if !args.is_empty() {
        println!("  {} Arguments: {}", "•".dimmed(), args.join(" ").dimmed());
    }
    println!();

    // Load hook registry if workspace exists
    let hook_registry = if let Some(ref ws) = workspace {
        let registry = Arc::new(HookRegistry::new());
        // Load hooks from workspace and extensions (best effort, don't fail if loading fails)
        let _ = HookLoader::load_from_workspace(ws.root(), &registry).await;
        let _ = HookLoader::load_from_extensions(&registry).await;
        Some(registry)
    } else {
        None
    };

    // Execute command with hooks
    let output = command
        .execute_with_hooks(args, &workspace_root, hook_registry)
        .await
        .context("Failed to execute custom command")?;

    // Print output
    print!("{}", output);

    Ok(())
}

/// Create a new custom command.
async fn create_command(
    name: &str,
    description: Option<&str>,
    template: Option<&str>,
    user: bool,
    namespace: Option<&str>,
) -> anyhow::Result<()> {
    println!("{}", "rad custom create".bold().cyan());
    println!();

    // Determine target directory
    let target_dir = if user {
        let user_home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get user home directory"))?;
        user_home.join(".radium").join("commands")
    } else {
        // Discover workspace root
        let workspace = Workspace::discover()
            .map_err(|_| anyhow::anyhow!("Not in a Radium workspace. Run 'rad init' first or use --user flag"))?;
        workspace.root().join(".radium").join("commands")
    };

    // Create namespace subdirectory if specified
    let command_dir = if let Some(ns) = namespace {
        target_dir.join(ns)
    } else {
        target_dir.clone()
    };

    // Create parent directories
    fs::create_dir_all(&command_dir)
        .context(format!("Failed to create directory: {}", command_dir.display()))?;

    // Determine file path
    let file_path = command_dir.join(format!("{}.toml", name));

    // Check if command already exists
    if file_path.exists() {
        bail!("Command '{}' already exists at {}", name, file_path.display());
    }

    // Get description
    let description = if let Some(desc) = description {
        desc.to_string()
    } else {
        inquire::Text::new("Enter command description:")
            .with_help_message("A brief description of what this command does")
            .prompt()
            .context("Failed to get description")?
    };

    // Get template
    let template = if let Some(tmpl) = template {
        tmpl.to_string()
    } else {
        inquire::Text::new("Enter command template:")
            .with_help_message("Use !{command} for shell injection, @{file} for file injection, {{args}} for arguments")
            .with_default("!{echo 'Hello {{args}}'}")
            .prompt()
            .context("Failed to get template")?
    };

    // Generate TOML content
    let toml_content = format!(
        r#"[command]
name = "{}"
description = "{}"
template = "{}"
"#,
        name, description, template
    );

    // Write file
    fs::write(&file_path, toml_content)
        .context(format!("Failed to write command file: {}", file_path.display()))?;

    println!("  {} Created command: {}", "✓".green(), name.cyan());
    println!("  {} Location: {}", "•".dimmed(), file_path.display());
    println!();

    Ok(())
}

/// Validate custom commands.
async fn validate_commands(name: Option<&str>, verbose: bool) -> anyhow::Result<()> {
    println!("{}", "rad custom validate".bold().cyan());
    println!();

    // Discover workspace root
    let workspace = Workspace::discover().ok();
    let workspace_root = workspace
        .as_ref()
        .map(|w| w.root().to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Get user home directory
    let user_home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get user home directory"))?;
    let user_commands_dir = user_home.join(".radium").join("commands");

    // Project commands directory
    let project_commands_dir = workspace_root.join(".radium").join("commands");

    // Initialize registry
    let mut registry = CommandRegistry::new()
        .with_project_dir(&project_commands_dir)
        .with_user_dir(&user_commands_dir);

    // Discover commands
    registry.discover().context("Failed to discover custom commands")?;

    let commands_to_validate: Vec<String> = if let Some(cmd_name) = name {
        if registry.get(cmd_name).is_some() {
            vec![cmd_name.to_string()]
        } else {
            bail!("Command '{}' not found", cmd_name);
        }
    } else {
        registry.list()
    };

    if commands_to_validate.is_empty() {
        println!("{}", "No commands to validate.".yellow());
        return Ok(());
    }

    let mut valid_count = 0;
    let mut invalid_count = 0;
    let mut errors: Vec<(String, Vec<String>)> = Vec::new();

    for cmd_name in &commands_to_validate {
        let command = registry.get(cmd_name).unwrap();
        let mut cmd_errors = Vec::new();

        // Check template syntax
        if let Err(e) = check_template_syntax(&command.template) {
            cmd_errors.push(format!("Template syntax error: {}", e));
        }

        // Check file references
        if let Err(e) = check_file_references(&command.template, &workspace_root) {
            cmd_errors.push(format!("File reference error: {}", e));
        }

        if cmd_errors.is_empty() {
            valid_count += 1;
            if verbose {
                println!("  {} {}", "✓".green(), cmd_name.cyan());
            }
        } else {
            invalid_count += 1;
            errors.push((cmd_name.clone(), cmd_errors));
            println!("  {} {} {}", "✗".red(), cmd_name.cyan(), "(invalid)".red());
            if verbose {
                for err in &errors.last().unwrap().1 {
                    println!("    {}", format!("  • {}", err).dimmed());
                }
            }
        }
    }

    println!();
    println!("  {} Valid: {}", "•".dimmed(), valid_count.to_string().green());
    println!("  {} Invalid: {}", "•".dimmed(), invalid_count.to_string().red());
    println!();

    if invalid_count > 0 {
        anyhow::bail!("Validation failed: {} command(s) have errors", invalid_count);
    }

    Ok(())
}

/// Check template syntax for balanced braces.
fn check_template_syntax(template: &str) -> anyhow::Result<()> {
    let mut shell_depth = 0;
    let mut file_depth = 0;
    let mut arg_depth = 0;

    let chars: Vec<char> = template.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i < chars.len() - 1 {
            // Check for !{ (shell injection)
            if chars[i] == '!' && chars[i + 1] == '{' {
                shell_depth += 1;
                i += 2;
                continue;
            }
            // Check for @{ (file injection)
            if chars[i] == '@' && chars[i + 1] == '{' {
                file_depth += 1;
                i += 2;
                continue;
            }
            // Check for {{ (argument placeholder)
            if chars[i] == '{' && chars[i + 1] == '{' {
                arg_depth += 1;
                i += 2;
                continue;
            }
        }

        // Check for closing braces
        if chars[i] == '}' {
            if shell_depth > 0 {
                shell_depth -= 1;
            } else if file_depth > 0 {
                file_depth -= 1;
            } else if arg_depth > 0 {
                arg_depth -= 1;
            } else {
                return Err(anyhow::anyhow!("Unmatched closing brace at position {}", i));
            }
        }

        i += 1;
    }

    if shell_depth > 0 {
        return Err(anyhow::anyhow!("Unclosed shell command injection (!{{)"));
    }
    if file_depth > 0 {
        return Err(anyhow::anyhow!("Unclosed file injection (@{{)"));
    }
    if arg_depth > 0 {
        return Err(anyhow::anyhow!("Unclosed argument placeholder ({{{{)"));
    }

    Ok(())
}

/// Check that referenced files exist.
fn check_file_references(template: &str, base_dir: &Path) -> anyhow::Result<()> {
    let mut result = template.to_string();

    // Find all @{...} patterns
    while let Some(start) = result.find("@{") {
        let Some(end) = result[start..].find('}') else {
            break;
        };

        let end = start + end;
        let file_path_str = &result[start + 2..end];
        let file_path = base_dir.join(file_path_str);

        if !file_path.exists() {
            return Err(anyhow::anyhow!(
                "Referenced file does not exist: {}",
                file_path.display()
            ));
        }

        // Remove this pattern to continue checking
        result.replace_range(start..=end, "");
    }

    Ok(())
}


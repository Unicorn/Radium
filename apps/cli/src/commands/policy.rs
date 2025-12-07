//! Policy management commands.

use clap::Subcommand;
use radium_core::policy::{ApprovalMode, PolicyAction, PolicyEngine, PolicyError};
use radium_core::workspace::Workspace;
use std::path::PathBuf;

/// Policy command options.
#[derive(Subcommand, Debug)]
pub enum PolicyCommand {
    /// List all loaded policy rules
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Test policy evaluation for a tool
    Check {
        /// Tool name to check
        tool_name: String,

        /// Tool arguments (space-separated)
        #[arg(last = true)]
        args: Vec<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Validate policy.toml syntax
    Validate {
        /// Path to policy file (default: .radium/policy.toml)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Create default policy.toml template
    Init {
        /// Overwrite existing file
        #[arg(short, long)]
        force: bool,
    },
}

/// Execute policy command.
pub async fn execute_policy_command(command: PolicyCommand) -> anyhow::Result<()> {
    match command {
        PolicyCommand::List { json, verbose } => list_policies(json, verbose).await,
        PolicyCommand::Check { tool_name, args, json } => check_policy(tool_name, args, json).await,
        PolicyCommand::Validate { file } => validate_policy(file).await,
        PolicyCommand::Init { force } => init_policy(force).await,
    }
}

/// List all policy rules.
async fn list_policies(json: bool, verbose: bool) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let policy_file = workspace.root().join(".radium").join("policy.toml");

    let engine = if policy_file.exists() {
        PolicyEngine::from_file(&policy_file).map_err(|e| {
            anyhow::anyhow!("Failed to load policy file {}: {}", policy_file.display(), e)
        })?
    } else {
        // Create default engine with Ask mode
        PolicyEngine::new(ApprovalMode::Ask).map_err(|e| {
            anyhow::anyhow!("Failed to create default policy engine: {}", e)
        })?
    };

    // Get rules count by checking if we can evaluate a dummy tool
    // Since we can't access rules directly, we'll use a workaround
    let approval_mode = engine.approval_mode();
    
    // For now, we'll need to add a method to PolicyEngine to get rules
    // For this implementation, let's use a simpler approach
    if json {
        // JSON output - simplified since we can't access rules directly
        println!("{}", serde_json::json!({
            "approval_mode": format!("{:?}", approval_mode).to_lowercase(),
            "note": "Rule details require PolicyEngine::get_rules() method",
        }));
    } else {
        // Human-readable output
        println!("Policy Configuration");
        println!("===================");
        println!("Approval Mode: {:?}", approval_mode);
        println!();
        println!("Note: Rule listing requires PolicyEngine API enhancement.");
        println!("Policy file: {}", policy_file.display());
            println!("No policy rules configured.");
            println!("Run 'rad policy init' to create a default policy.toml file.");
        } else {
            println!("Rules ({}):", engine.rules.len());
            println!();

            if verbose {
                // Detailed table format
                println!("{:<30} {:<10} {:<10} {:<20} {:<30}", "Name", "Priority", "Action", "Tool Pattern", "Arg Pattern");
                println!("{}", "-".repeat(100));
                for rule in &engine.rules {
                    let arg_pattern = rule.arg_pattern.as_deref().unwrap_or("(none)");
                    println!(
                        "{:<30} {:<10} {:<10} {:<20} {:<30}",
                        rule.name,
                        format!("{:?}", rule.priority),
                        format!("{:?}", rule.action),
                        rule.tool_pattern,
                        arg_pattern
                    );
                }
        }
        }
    }

    Ok(())
}

/// Check policy evaluation for a tool.
async fn check_policy(tool_name: String, args: Vec<String>, json: bool) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let policy_file = workspace.root().join(".radium").join("policy.toml");

    let engine = if policy_file.exists() {
        PolicyEngine::from_file(&policy_file).map_err(|e| {
            anyhow::anyhow!("Failed to load policy file {}: {}", policy_file.display(), e)
        })?
    } else {
        // Create default engine with Ask mode
        PolicyEngine::new(ApprovalMode::Ask).map_err(|e| {
            anyhow::anyhow!("Failed to create default policy engine: {}", e)
        })?
    };

    // Convert args to &[&str] for evaluation
    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let decision = engine.evaluate_tool(&tool_name, &args_refs).await.map_err(|e| {
        anyhow::anyhow!("Failed to evaluate tool: {}", e)
    })?;

    if json {
        println!("{}", serde_json::json!({
            "tool_name": tool_name,
            "args": args,
            "decision": {
                "action": format!("{:?}", decision.action()).to_lowercase(),
                "reason": decision.reason(),
                "matched_rule": decision.matched_rule(),
            }
        }));
    } else {
        println!("Policy Evaluation Result");
        println!("========================");
        println!("Tool: {}", tool_name);
        if !args.is_empty() {
            println!("Arguments: {}", args.join(" "));
        }
        println!("Decision: {:?}", decision.action);
        if let Some(ref reason) = decision.reason {
            println!("Reason: {}", reason);
        }
        if let Some(ref rule) = decision.matched_rule {
            println!("Matched Rule: {}", rule);
        }
    }

    Ok(())
}

/// Validate policy file syntax.
async fn validate_policy(file: Option<PathBuf>) -> anyhow::Result<()> {
    let policy_file = if let Some(f) = file {
        f
    } else {
        let workspace = Workspace::discover()?;
        workspace.root().join(".radium").join("policy.toml")
    };

    if !policy_file.exists() {
        eprintln!("Policy file not found: {}", policy_file.display());
        eprintln!("Run 'rad policy init' to create a default policy.toml file.");
        return Ok(());
    }

    match PolicyEngine::from_file(&policy_file) {
        Ok(engine) => {
            println!("✓ Policy file is valid: {}", policy_file.display());
            println!("  Approval Mode: {:?}", engine.approval_mode());
            println!("  Rules loaded successfully");

            // Note: Pattern validation happens during rule loading
            // If we got here, the patterns are valid

            if errors.is_empty() {
                println!("  All rule patterns are valid.");
                Ok(())
            } else {
                eprintln!("\n✗ Found {} pattern error(s):", errors.len());
                for error in errors {
                    eprintln!("  {}", error);
                }
                Err(anyhow::anyhow!("Policy file has invalid patterns"))
            }
        }
        Err(e) => {
            eprintln!("✗ Policy file is invalid: {}", policy_file.display());
            eprintln!("  Error: {}", e);
            Err(anyhow::anyhow!("Policy file validation failed: {}", e))
        }
    }
}

/// Initialize default policy file.
async fn init_policy(force: bool) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let radium_dir = workspace.root().join(".radium");
    let policy_file = radium_dir.join("policy.toml");

    // Ensure .radium directory exists
    std::fs::create_dir_all(&radium_dir)?;

    if policy_file.exists() && !force {
        eprintln!("Policy file already exists: {}", policy_file.display());
        eprintln!("Use --force to overwrite it.");
        return Ok(());
    }

    let default_policy = r#"# Radium Policy Configuration
# This file controls tool execution policies for Radium agents.

# Approval mode determines default behavior when no rules match
# Options: yolo (auto-approve all), autoEdit (auto-approve edits), ask (ask for all)
approval_mode = "ask"

# Policy rules are evaluated in priority order (admin > user > default)
# First matching rule wins

[[rules]]
name = "Allow safe file operations"
priority = "user"
action = "allow"
tool_pattern = "read_*"
reason = "Safe read operations are always allowed"

[[rules]]
name = "Allow file writes with approval"
priority = "user"
action = "ask_user"
tool_pattern = "write_*"
reason = "File writes require user approval"

[[rules]]
name = "Deny dangerous shell commands"
priority = "admin"
action = "deny"
tool_pattern = "run_terminal_cmd"
arg_pattern = "rm -rf *"
reason = "Prevent accidental deletion"

[[rules]]
name = "Require approval for MCP tools"
priority = "user"
action = "ask_user"
tool_pattern = "mcp_*"
reason = "MCP tools may have side effects"
"#;

    std::fs::write(&policy_file, default_policy)?;
    println!("Created default policy file: {}", policy_file.display());
    println!("Edit this file to customize your policy rules.");

    Ok(())
}


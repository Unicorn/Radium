//! Policy management commands.

use clap::Subcommand;
use radium_core::policy::{ApprovalMode, PolicyEngine};
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

    /// Add a new policy rule
    Add {
        /// Rule name
        name: String,
        /// Priority (admin, user, default)
        #[arg(long)]
        priority: Option<String>,
        /// Action (allow, deny, ask_user)
        #[arg(long)]
        action: Option<String>,
        /// Tool pattern (glob pattern)
        #[arg(long)]
        tool_pattern: Option<String>,
        /// Argument pattern (optional)
        #[arg(long)]
        arg_pattern: Option<String>,
        /// Reason for the rule
        #[arg(long)]
        reason: Option<String>,
    },

    /// Remove a policy rule by name
    Remove {
        /// Rule name to remove
        name: String,
    },
}

/// Execute policy command.
pub async fn execute_policy_command(command: PolicyCommand) -> anyhow::Result<()> {
    match command {
        PolicyCommand::List { json, verbose } => list_policies(json, verbose).await,
        PolicyCommand::Check { tool_name, args, json } => check_policy(tool_name, args, json).await,
        PolicyCommand::Validate { file } => validate_policy(file).await,
        PolicyCommand::Init { force } => init_policy(force).await,
        PolicyCommand::Add { name, priority, action, tool_pattern, arg_pattern, reason } => {
            add_policy(name, priority, action, tool_pattern, arg_pattern, reason).await
        }
        PolicyCommand::Remove { name } => remove_policy(name).await,
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

    if !policy_file.exists() {
        if json {
            println!("{}", serde_json::json!({
                "approval_mode": "ask",
                "rules": [],
                "file_exists": false,
            }));
        } else {
            println!("Policy Configuration");
            println!("===================");
            println!("No policy file found: {}", policy_file.display());
            println!("Run 'rad policy init' to create a default policy.toml file.");
        }
        return Ok(());
    }

    // Parse TOML directly to get rule details
    let content = std::fs::read_to_string(&policy_file)?;
    let config: toml::Value = toml::from_str(&content)?;

    let approval_mode_str = config
        .get("approval_mode")
        .and_then(|v| v.as_str())
        .unwrap_or("ask");
    let rules = config.get("rules").and_then(|v| v.as_array()).unwrap_or(&vec![]);

    if json {
        let rules_json: Vec<serde_json::Value> = rules
            .iter()
            .filter_map(|rule| {
                rule.as_table().map(|t| {
                    serde_json::json!({
                        "name": t.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                        "tool_pattern": t.get("tool_pattern").and_then(|v| v.as_str()).unwrap_or(""),
                        "arg_pattern": t.get("arg_pattern").and_then(|v| v.as_str()),
                        "action": t.get("action").and_then(|v| v.as_str()).unwrap_or(""),
                        "priority": t.get("priority").and_then(|v| v.as_str()).unwrap_or("user"),
                        "reason": t.get("reason").and_then(|v| v.as_str()),
                    })
                })
            })
            .collect();

        println!("{}", serde_json::json!({
            "approval_mode": approval_mode_str,
            "rules": rules_json,
            "rule_count": engine.rule_count(),
        }));
    } else {
        println!("Policy Configuration");
        println!("===================");
        println!("Approval Mode: {}", approval_mode_str);
        println!("Rules: {}", engine.rule_count());
        println!();

        if rules.is_empty() {
            println!("No policy rules configured.");
            println!("Edit {} to add rules.", policy_file.display());
        } else {
            if verbose {
                // Detailed table format
                println!("{:<30} {:<10} {:<10} {:<20} {:<30}", "Name", "Priority", "Action", "Tool Pattern", "Arg Pattern");
                println!("{}", "-".repeat(100));
                for rule in rules {
                    if let Some(rule_table) = rule.as_table() {
                        let name = rule_table.get("name").and_then(|v| v.as_str()).unwrap_or("(unnamed)");
                        let priority = rule_table.get("priority").and_then(|v| v.as_str()).unwrap_or("user");
                        let action = rule_table.get("action").and_then(|v| v.as_str()).unwrap_or("allow");
                        let tool_pattern = rule_table.get("tool_pattern").and_then(|v| v.as_str()).unwrap_or("");
                        let arg_pattern = rule_table.get("arg_pattern").and_then(|v| v.as_str()).unwrap_or("(none)");
                        println!(
                            "{:<30} {:<10} {:<10} {:<20} {:<30}",
                            name, priority, action, tool_pattern, arg_pattern
                        );
                    }
                }
            } else {
                // Simple list format
                for (i, rule) in rules.iter().enumerate() {
                    if let Some(rule_table) = rule.as_table() {
                        let name = rule_table.get("name").and_then(|v| v.as_str()).unwrap_or("(unnamed)");
                        let priority = rule_table.get("priority").and_then(|v| v.as_str()).unwrap_or("user");
                        let action = rule_table.get("action").and_then(|v| v.as_str()).unwrap_or("allow");
                        let tool_pattern = rule_table.get("tool_pattern").and_then(|v| v.as_str()).unwrap_or("");
                        println!("{}. {} ({} priority, {} action)", i + 1, name, priority, action);
                        println!("   Pattern: {}", tool_pattern);
                        if let Some(arg_pattern) = rule_table.get("arg_pattern").and_then(|v| v.as_str()) {
                            println!("   Arg Pattern: {}", arg_pattern);
                        }
                        if let Some(reason) = rule_table.get("reason").and_then(|v| v.as_str()) {
                            println!("   Reason: {}", reason);
                        }
                        println!();
                    }
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
                "action": format!("{:?}", decision.action).to_lowercase(),
                "reason": decision.reason.as_ref(),
                "matched_rule": decision.matched_rule.as_ref(),
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
            println!("  Rules: {}", engine.rule_count());
            println!("  All rule patterns are valid.");
            Ok(())
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


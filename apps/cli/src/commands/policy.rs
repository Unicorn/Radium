//! Policy management commands.

use clap::Subcommand;
use radium_core::policy::{ApprovalMode, ConflictDetector, ConflictResolver, PolicyEngine, ResolutionStrategy, merge_template, PolicyTemplate, TemplateDiscovery};
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

    /// Policy template management
    Templates {
        #[command(subcommand)]
        command: TemplateCommand,
    },

    /// Detect conflicts in policy rules
    Conflicts {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Resolve conflicts in policy rules
    Resolve {
        /// Resolution strategy (auto, higher-priority, more-specific, keep-first, keep-second, remove-both, rename)
        #[arg(long, default_value = "auto")]
        strategy: String,

        /// Auto-apply resolution (don't ask for confirmation)
        #[arg(long)]
        yes: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// Template management commands.
#[derive(Subcommand, Debug)]
pub enum TemplateCommand {
    /// List available policy templates
    List,
    /// Show template contents
    Show {
        /// Template name
        name: String,
    },
    /// Apply a template to workspace
    Apply {
        /// Template name
        name: String,
        /// Merge with existing rules (default: append)
        #[arg(long)]
        merge: bool,
        /// Replace all existing rules
        #[arg(long)]
        replace: bool,
        /// Preview changes without applying
        #[arg(long)]
        dry_run: bool,
    },
    /// Validate template syntax
    Validate {
        /// Template name
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
        PolicyCommand::Templates { command } => execute_template_command(command).await,
        PolicyCommand::Conflicts { json } => detect_conflicts(json).await,
        PolicyCommand::Resolve { strategy, yes, json } => resolve_conflicts(strategy, yes, json).await,
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

/// Add a new policy rule.
async fn add_policy(
    name: String,
    priority: Option<String>,
    action: Option<String>,
    tool_pattern: Option<String>,
    arg_pattern: Option<String>,
    reason: Option<String>,
) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let radium_dir = workspace.root().join(".radium");
    let policy_file = radium_dir.join("policy.toml");

    // Ensure .radium directory exists
    std::fs::create_dir_all(&radium_dir)?;

    // If no policy file exists, create one
    if !policy_file.exists() {
        let default_policy = r#"approval_mode = "ask"

"#;
        std::fs::write(&policy_file, default_policy)?;
    }

    // Read existing policy
    let content = std::fs::read_to_string(&policy_file)?;
    let mut config: toml::Value = toml::from_str(&content)?;

    // Get rules array or create new one
    let rules = config.get_mut("rules").and_then(|v| v.as_array_mut());
    let rules = if let Some(rules) = rules {
        rules
    } else {
        config.as_table_mut().unwrap().insert(
            "rules".to_string(),
            toml::Value::Array(vec![]),
        );
        config.get_mut("rules").unwrap().as_array_mut().unwrap()
    };

    // Collect inputs interactively if not provided
    use std::io::{self, Write};
    let priority = priority.unwrap_or_else(|| {
        print!("Priority (admin/user/default) [user]: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input.is_empty() { "user".to_string() } else { input.to_string() }
    });

    let action = action.unwrap_or_else(|| {
        print!("Action (allow/deny/ask_user) [ask_user]: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input.is_empty() { "ask_user".to_string() } else { input.to_string() }
    });

    let tool_pattern = tool_pattern.unwrap_or_else(|| {
        print!("Tool pattern (glob pattern, e.g., 'read_*'): ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    });

    if tool_pattern.is_empty() {
        return Err(anyhow::anyhow!("Tool pattern is required"));
    }

    // Validate priority
    let priority_lower = priority.to_lowercase();
    if !["admin", "user", "default"].contains(&priority_lower.as_str()) {
        return Err(anyhow::anyhow!("Priority must be one of: admin, user, default"));
    }

    // Validate action
    let action_lower = action.to_lowercase();
    if !["allow", "deny", "ask_user"].contains(&action_lower.as_str()) {
        return Err(anyhow::anyhow!("Action must be one of: allow, deny, ask_user"));
    }

    // Create new rule
    let mut rule = toml::map::Map::new();
    rule.insert("name".to_string(), toml::Value::String(name.clone()));
    rule.insert("priority".to_string(), toml::Value::String(priority_lower));
    rule.insert("action".to_string(), toml::Value::String(action_lower));
    rule.insert("tool_pattern".to_string(), toml::Value::String(tool_pattern.clone()));
    
    if let Some(arg_pattern) = arg_pattern {
        if !arg_pattern.is_empty() {
            rule.insert("arg_pattern".to_string(), toml::Value::String(arg_pattern));
        }
    }

    if let Some(reason) = reason {
        if !reason.is_empty() {
            rule.insert("reason".to_string(), toml::Value::String(reason));
        }
    }

    // Basic validation - check that tool_pattern is not empty (already done above)
    // Full validation will happen when PolicyEngine::from_file is called

    // Add rule to array
    rules.push(toml::Value::Table(rule));

    // Write back to file
    let new_content = toml::to_string_pretty(&config)?;
    std::fs::write(&policy_file, new_content)?;

    println!("✓ Added policy rule: {}", name);
    println!("  Tool pattern: {}", tool_pattern);
    println!("  Priority: {}", priority_lower);
    println!("  Action: {}", action_lower);

    Ok(())
}

/// Remove a policy rule by name.
async fn remove_policy(name: String) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let policy_file = workspace.root().join(".radium").join("policy.toml");

    if !policy_file.exists() {
        return Err(anyhow::anyhow!("Policy file not found: {}", policy_file.display()));
    }

    // Read existing policy
    let content = std::fs::read_to_string(&policy_file)?;
    let mut config: toml::Value = toml::from_str(&content)?;

    // Get rules array
    let Some(rules) = config.get_mut("rules").and_then(|v| v.as_array_mut()) else {
        return Err(anyhow::anyhow!("No rules found in policy file"));
    };

    // Find and remove rule by name
    let initial_len = rules.len();
    rules.retain(|rule| {
        if let Some(rule_table) = rule.as_table() {
            rule_table.get("name")
                .and_then(|v| v.as_str())
                .map(|n| n != name)
                .unwrap_or(true)
        } else {
            true
        }
    });

    if rules.len() == initial_len {
        return Err(anyhow::anyhow!("Rule '{}' not found", name));
    }

    // Confirm removal
    use std::io::{self, Write};
    print!("Remove rule '{}'? (y/N): ", name);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();
    if input != "y" && input != "yes" {
        println!("Cancelled.");
        return Ok(());
    }

    // Write back to file
    let new_content = toml::to_string_pretty(&config)?;
    std::fs::write(&policy_file, new_content)?;

    println!("✓ Removed policy rule: {}", name);

    Ok(())
}

/// Execute template command.
async fn execute_template_command(command: TemplateCommand) -> anyhow::Result<()> {
    match command {
        TemplateCommand::List => list_templates().await,
        TemplateCommand::Show { name } => show_template(name).await,
        TemplateCommand::Apply { name, merge, replace, dry_run } => {
            apply_template(name, merge, replace, dry_run).await
        }
        TemplateCommand::Validate { name } => validate_template(name).await,
    }
}

/// List available policy templates.
async fn list_templates() -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let templates_dir = workspace.root().join("templates");
    let mut discovery = TemplateDiscovery::new(&templates_dir);
    let templates = discovery.discover()?;

    if templates.is_empty() {
        println!("No policy templates found in {}", templates_dir.display());
        println!();
        println!("Templates should be placed in: templates/policies/*.toml");
        return Ok(());
    }

    println!("Available Policy Templates");
    println!("==========================");
    println!();
    for template in &templates {
        println!("  {} - {}", template.name, template.description);
    }
    println!();
    println!("Use 'rad policy templates show <name>' to view a template");
    println!("Use 'rad policy templates apply <name>' to apply a template");

    Ok(())
}

/// Show template contents.
async fn show_template(name: String) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let templates_dir = workspace.root().join("templates");
    let mut discovery = TemplateDiscovery::new(&templates_dir);
    discovery.discover()?;

    let template = discovery.get_template(&name)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", name))?;

    println!("Template: {}", template.name);
    println!("Description: {}", template.description);
    println!("Path: {}", template.path.display());
    println!();
    println!("Content:");
    println!("{}", "=".repeat(60));
    let content = template.get_content()?;
    println!("{}", content);

    Ok(())
}

/// Apply a template to workspace.
async fn apply_template(
    name: String,
    merge: bool,
    replace: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let templates_dir = workspace.root().join("templates");
    let policy_file = workspace.root().join(".radium").join("policy.toml");

    let mut discovery = TemplateDiscovery::new(&templates_dir);
    discovery.discover()?;

    let template = discovery.get_template(&name)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", name))?;

    // Validate template first
    let mut template_clone = template.clone();
    template_clone.validate().map_err(|e| {
        anyhow::anyhow!("Template validation failed: {}", e)
    })?;

    let template_content = template.get_content()?;

    // Determine merge strategy
    let should_replace = replace || (!merge && !policy_file.exists());

    // Merge template
    let merged_content = merge_template(&policy_file, &template_content, should_replace)?;

    if dry_run {
        println!("Dry run - preview of changes:");
        println!("{}", "=".repeat(60));
        println!("{}", merged_content);
        println!("{}", "=".repeat(60));
        println!();
        println!("To apply, run without --dry-run");
        return Ok(());
    }

    // Ensure .radium directory exists
    if let Some(parent) = policy_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write merged policy
    std::fs::write(&policy_file, merged_content)?;

    if should_replace {
        println!("✓ Applied template '{}' (replaced existing rules)", name);
    } else {
        println!("✓ Applied template '{}' (merged with existing rules)", name);
    }
    println!("  Policy file: {}", policy_file.display());

    Ok(())
}

/// Detect conflicts in policy rules.
async fn detect_conflicts(json: bool) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let policy_file = workspace.root().join(".radium").join("policy.toml");

    if !policy_file.exists() {
        if json {
            println!("{}", serde_json::json!({
                "conflicts": [],
                "conflict_count": 0,
                "file_exists": false,
            }));
        } else {
            println!("No policy file found: {}", policy_file.display());
            println!("Run 'rad policy init' to create a default policy.toml file.");
        }
        return Ok(());
    }

    let engine = PolicyEngine::from_file(&policy_file).map_err(|e| {
        anyhow::anyhow!("Failed to load policy file {}: {}", policy_file.display(), e)
    })?;

    let conflicts = engine.detect_conflicts().map_err(|e| {
        anyhow::anyhow!("Failed to detect conflicts: {}", e)
    })?;

    if json {
        let conflicts_json: Vec<serde_json::Value> = conflicts
            .iter()
            .map(|c| {
                serde_json::json!({
                    "type": format!("{:?}", c.conflict_type),
                    "rule1": {
                        "name": c.rule1.name,
                        "tool_pattern": c.rule1.tool_pattern,
                        "action": format!("{:?}", c.rule1.action),
                        "priority": format!("{:?}", c.rule1.priority),
                    },
                    "rule2": {
                        "name": c.rule2.name,
                        "tool_pattern": c.rule2.tool_pattern,
                        "action": format!("{:?}", c.rule2.action),
                        "priority": format!("{:?}", c.rule2.priority),
                    },
                    "example_tool": c.example_tool,
                    "description": c.conflict_type.description(),
                })
            })
            .collect();

        println!("{}", serde_json::json!({
            "conflicts": conflicts_json,
            "conflict_count": conflicts.len(),
        }));
    } else {
        println!("Policy Conflict Detection");
        println!("=========================");
        println!();

        if conflicts.is_empty() {
            println!("✓ No conflicts detected. All policy rules are compatible.");
        } else {
            println!("⚠ Found {} conflict(s):", conflicts.len());
            println!();

            for (i, conflict) in conflicts.iter().enumerate() {
                println!("Conflict {}:", i + 1);
                println!("  Type: {} ({})", format!("{:?}", conflict.conflict_type), conflict.conflict_type.description());
                println!("  Rule 1: {} (pattern: {}, action: {:?}, priority: {:?})", 
                    conflict.rule1.name, 
                    conflict.rule1.tool_pattern,
                    conflict.rule1.action,
                    conflict.rule1.priority);
                println!("  Rule 2: {} (pattern: {}, action: {:?}, priority: {:?})", 
                    conflict.rule2.name, 
                    conflict.rule2.tool_pattern,
                    conflict.rule2.action,
                    conflict.rule2.priority);
                println!("  Example tool: {}", conflict.example_tool);
                println!();
            }

            println!("To resolve conflicts, run: rad policy resolve [--strategy <strategy>]");
            println!("Available strategies: auto, higher-priority, more-specific, keep-first, keep-second, remove-both, rename");
        }
    }

    Ok(())
}

/// Resolve conflicts in policy rules.
async fn resolve_conflicts(strategy_str: String, yes: bool, json: bool) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let policy_file = workspace.root().join(".radium").join("policy.toml");

    if !policy_file.exists() {
        anyhow::bail!("No policy file found: {}", policy_file.display());
    }

    let mut engine = PolicyEngine::from_file(&policy_file).map_err(|e| {
        anyhow::anyhow!("Failed to load policy file {}: {}", policy_file.display(), e)
    })?;

    let conflicts = engine.detect_conflicts().map_err(|e| {
        anyhow::anyhow!("Failed to detect conflicts: {}", e)
    })?;

    if conflicts.is_empty() {
        if json {
            println!("{}", serde_json::json!({
                "resolved": false,
                "removed_rules": [],
                "conflict_count": 0,
                "message": "No conflicts to resolve",
            }));
        } else {
            println!("✓ No conflicts detected. Nothing to resolve.");
        }
        return Ok(());
    }

    let strategy = match strategy_str.as_str() {
        "auto" => ResolutionStrategy::KeepHigherPriority, // Use auto_resolve which is smarter
        "higher-priority" => ResolutionStrategy::KeepHigherPriority,
        "more-specific" => ResolutionStrategy::KeepMoreSpecific,
        "keep-first" => ResolutionStrategy::KeepFirst,
        "keep-second" => ResolutionStrategy::KeepSecond,
        "remove-both" => ResolutionStrategy::RemoveBoth,
        "rename" => ResolutionStrategy::Rename,
        _ => anyhow::bail!("Invalid strategy: {}. Valid strategies: auto, higher-priority, more-specific, keep-first, keep-second, remove-both, rename", strategy_str),
    };

    if !yes && !json {
        println!("Found {} conflict(s) to resolve.", conflicts.len());
        println!("Strategy: {}", strategy_str);
        println!();
        println!("Rules that will be removed:");
        for conflict in &conflicts {
            let to_remove = match strategy {
                ResolutionStrategy::KeepHigherPriority => {
                    if conflict.rule1.priority > conflict.rule2.priority {
                        &conflict.rule2.name
                    } else {
                        &conflict.rule1.name
                    }
                }
                ResolutionStrategy::KeepMoreSpecific => {
                    if ConflictDetector::is_more_specific(&conflict.rule1.tool_pattern, &conflict.rule2.tool_pattern) {
                        &conflict.rule2.name
                    } else {
                        &conflict.rule1.name
                    }
                }
                ResolutionStrategy::KeepFirst => &conflict.rule2.name,
                ResolutionStrategy::KeepSecond => &conflict.rule1.name,
                ResolutionStrategy::RemoveBoth => {
                    println!("  - {}", conflict.rule1.name);
                    &conflict.rule2.name
                }
                ResolutionStrategy::Rename => {
                    println!("  - {} (will be renamed)", conflict.rule2.name);
                    continue;
                }
            };
            println!("  - {}", to_remove);
        }
        println!();
        println!("Proceed with resolution? [y/N]: ");
        
        use std::io::{self, BufRead};
        let stdin = io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        if !line.trim().eq_ignore_ascii_case("y") && !line.trim().eq_ignore_ascii_case("yes") {
            println!("Resolution cancelled.");
            return Ok(());
        }
    }

    let removed = if strategy_str == "auto" {
        engine.auto_resolve_conflicts().map_err(|e| {
            anyhow::anyhow!("Failed to auto-resolve conflicts: {}", e)
        })?
    } else {
        engine.resolve_conflicts(strategy).map_err(|e| {
            anyhow::anyhow!("Failed to resolve conflicts: {}", e)
        })?
    };

    // Save resolved policy back to file
    use std::fs::File;
    use std::io::Write;
    use toml;

    let mut config = toml::value::Table::new();
    config.insert("approval_mode".to_string(), toml::Value::String(format!("{:?}", engine.approval_mode()).to_lowercase()));
    
    let rules_array: Vec<toml::Value> = engine.rules()
        .iter()
        .map(|rule| {
            let mut rule_table = toml::value::Table::new();
            rule_table.insert("name".to_string(), toml::Value::String(rule.name.clone()));
            rule_table.insert("tool_pattern".to_string(), toml::Value::String(rule.tool_pattern.clone()));
            rule_table.insert("action".to_string(), toml::Value::String(format!("{:?}", rule.action).to_lowercase()));
            rule_table.insert("priority".to_string(), toml::Value::String(format!("{:?}", rule.priority).to_lowercase()));
            if let Some(ref arg_pattern) = rule.arg_pattern {
                rule_table.insert("arg_pattern".to_string(), toml::Value::String(arg_pattern.clone()));
            }
            if let Some(ref reason) = rule.reason {
                rule_table.insert("reason".to_string(), toml::Value::String(reason.clone()));
            }
            toml::Value::Table(rule_table)
        })
        .collect();
    config.insert("rules".to_string(), toml::Value::Array(rules_array));

    let toml_string = toml::to_string_pretty(&config)?;
    let mut file = File::create(&policy_file)?;
    file.write_all(toml_string.as_bytes())?;

    if json {
        println!("{}", serde_json::json!({
            "resolved": true,
            "removed_rules": removed,
            "conflict_count": conflicts.len(),
            "remaining_rules": engine.rule_count(),
        }));
    } else {
        println!("✓ Resolved {} conflict(s).", conflicts.len());
        if !removed.is_empty() {
            println!("Removed rules:");
            for rule_name in &removed {
                println!("  - {}", rule_name);
            }
        }
        println!("Remaining rules: {}", engine.rule_count());
        println!("Policy saved to: {}", policy_file.display());
    }

    Ok(())
}

/// Validate template syntax.
async fn validate_template(name: String) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let templates_dir = workspace.root().join("templates");
    let mut discovery = TemplateDiscovery::new(&templates_dir);
    discovery.discover()?;

    let template = discovery.get_template(&name)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", name))?
        .clone();

    let mut template_mut = template;
    match template_mut.validate() {
        Ok(()) => {
            println!("✓ Template '{}' is valid", name);
            println!("  Path: {}", template_mut.path.display());
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Template '{}' is invalid", name);
            eprintln!("  Error: {}", e);
            Err(anyhow::anyhow!("Template validation failed: {}", e))
        }
    }
}


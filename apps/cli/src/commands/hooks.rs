//! Hook management commands.

use clap::Subcommand;
use radium_core::hooks::loader::HookLoader;
use radium_core::hooks::registry::{HookRegistry, HookType};
use radium_core::workspace::Workspace;
use std::sync::Arc;

/// Hook command options.
#[derive(Subcommand, Debug)]
pub enum HooksCommand {
    /// List all registered hooks
    List {
        /// Filter by hook type
        #[arg(long)]
        r#type: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show detailed information about a specific hook
    Info {
        /// Hook name
        name: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Enable a hook
    Enable {
        /// Hook name
        name: String,
    },

    /// Disable a hook
    Disable {
        /// Hook name
        name: String,
    },
}

/// Execute hook command.
pub async fn execute_hooks_command(command: HooksCommand) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let registry = Arc::new(HookRegistry::new());

    // Load hooks from extensions and workspace
    let _extension_count = HookLoader::load_from_extensions(&registry).await
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to load hooks from extensions: {}", e);
            0
        });

    let _workspace_count = HookLoader::load_from_workspace(workspace.root(), &registry).await
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to load hooks from workspace: {}", e);
            0
        });

    match command {
        HooksCommand::List { r#type, json, verbose } => {
            let hook_type = if let Some(type_str) = r#type {
                match type_str.as_str() {
                    "before_model" => Some(HookType::BeforeModel),
                    "after_model" => Some(HookType::AfterModel),
                    "before_tool" => Some(HookType::BeforeTool),
                    "after_tool" => Some(HookType::AfterTool),
                    "tool_selection" => Some(HookType::ToolSelection),
                    "error_interception" => Some(HookType::ErrorInterception),
                    "error_transformation" => Some(HookType::ErrorTransformation),
                    "error_recovery" => Some(HookType::ErrorRecovery),
                    "error_logging" => Some(HookType::ErrorLogging),
                    "telemetry_collection" => Some(HookType::TelemetryCollection),
                    "custom_logging" => Some(HookType::CustomLogging),
                    "metrics_aggregation" => Some(HookType::MetricsAggregation),
                    "performance_monitoring" => Some(HookType::PerformanceMonitoring),
                    _ => {
                        eprintln!("Invalid hook type: {}", type_str);
                        eprintln!("Valid types: before_model, after_model, before_tool, after_tool, tool_selection, error_interception, error_transformation, error_recovery, error_logging, telemetry_collection, custom_logging, metrics_aggregation, performance_monitoring");
                        return Ok(());
                    }
                }
            } else {
                None
            };

            let hooks = if let Some(ht) = hook_type {
                registry.get_hooks(ht).await
            } else {
                // Get all hooks by iterating through all hook types
                let mut all_hooks = Vec::new();
                for ht in [
                    HookType::BeforeModel,
                    HookType::AfterModel,
                    HookType::BeforeTool,
                    HookType::AfterTool,
                    HookType::ToolSelection,
                    HookType::ErrorInterception,
                    HookType::ErrorTransformation,
                    HookType::ErrorRecovery,
                    HookType::ErrorLogging,
                    HookType::TelemetryCollection,
                    HookType::CustomLogging,
                    HookType::MetricsAggregation,
                    HookType::PerformanceMonitoring,
                ] {
                    all_hooks.extend(registry.get_hooks(ht).await);
                }
                all_hooks
            };

            if hooks.is_empty() {
                println!("No hooks registered.");
                return Ok(());
            }

            if json {
                let json_hooks: Vec<serde_json::Value> = hooks
                    .iter()
                    .map(|h| {
                        serde_json::json!({
                            "name": h.name(),
                            "type": h.hook_type().as_str(),
                            "priority": h.priority().value(),
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&json_hooks)?);
            } else {
                println!("Registered hooks:");
                println!();
                for hook in hooks {
                    println!("  {} ({})", hook.name(), hook.hook_type().as_str());
                    if verbose {
                        println!("    Priority: {}", hook.priority().value());
                    }
                }
            }
        }

        HooksCommand::Info { name, json } => {
            // Search for hook by name across all types
            let mut found = false;
            for ht in [
                HookType::BeforeModel,
                HookType::AfterModel,
                HookType::BeforeTool,
                HookType::AfterTool,
                HookType::ToolSelection,
                HookType::ErrorInterception,
                HookType::ErrorTransformation,
                HookType::ErrorRecovery,
                HookType::ErrorLogging,
                HookType::TelemetryCollection,
                HookType::CustomLogging,
                HookType::MetricsAggregation,
                HookType::PerformanceMonitoring,
            ] {
                let hooks = registry.get_hooks(ht).await;
                if let Some(hook) = hooks.iter().find(|h| h.name() == name) {
                    found = true;
                    if json {
                        let json_hook = serde_json::json!({
                            "name": hook.name(),
                            "type": hook.hook_type().as_str(),
                            "priority": hook.priority().value(),
                        });
                        println!("{}", serde_json::to_string_pretty(&json_hook)?);
                    } else {
                        println!("Hook: {}", hook.name());
                        println!("  Type: {}", hook.hook_type().as_str());
                        println!("  Priority: {}", hook.priority().value());
                    }
                    break;
                }
            }

            if !found {
                eprintln!("Hook '{}' not found.", name);
            }
        }

        HooksCommand::Enable { name } => {
            // Note: Hook enable/disable functionality would need to be implemented
            // in the HookRegistry. For now, we'll just acknowledge the command.
            println!("Enabling hook '{}'...", name);
            println!("Note: Hook enable/disable functionality is not yet fully implemented.");
            println!("Hooks are automatically enabled when registered.");
        }

        HooksCommand::Disable { name } => {
            // Note: Hook enable/disable functionality would need to be implemented
            // in the HookRegistry. For now, we'll just acknowledge the command.
            println!("Disabling hook '{}'...", name);
            println!("Note: Hook enable/disable functionality is not yet fully implemented.");
            println!("Use 'rad hooks unregister' to remove a hook.");
        }
    }

    Ok(())
}


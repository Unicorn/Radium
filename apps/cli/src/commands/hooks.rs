//! Hook management commands.

use clap::Subcommand;
use colored::Colorize;
use radium_core::hooks::config::HookConfig;
use radium_core::hooks::error_hooks::{ErrorHookContext, ErrorHookType};
use radium_core::hooks::loader::HookLoader;
use radium_core::hooks::model::{ModelHookContext, ModelHookType};
use radium_core::hooks::registry::{HookRegistry, HookType};
use radium_core::hooks::telemetry::TelemetryHookContext;
use radium_core::hooks::tool::{ToolHookContext, ToolHookType};
use radium_core::hooks::types::HookContext;
use radium_core::workspace::Workspace;
use std::sync::Arc;
use std::time::Instant;

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

    /// Validate hook configurations
    Validate {
        /// Specific hook name to validate (validates all if omitted)
        name: Option<String>,

        /// Show detailed validation information
        #[arg(short, long)]
        verbose: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Test hook execution with sample context
    Test {
        /// Hook name
        name: String,

        /// Hook type (if not provided, will use the hook's actual type)
        #[arg(long)]
        r#type: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
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
            // Update registry
            if let Err(e) = registry.set_enabled(&name, true).await {
                eprintln!("Failed to enable hook '{}': {}", name, e);
                return Ok(());
            }
            
            // Update workspace config
            let hooks_config_path = workspace.root().join(".radium").join("hooks.toml");
            if hooks_config_path.exists() {
                match radium_core::hooks::config::HookConfig::from_file(&hooks_config_path) {
                    Ok(mut config) => {
                        if let Err(e) = config.set_hook_enabled(&name, true) {
                            // Hook not in config, that's okay - it might be from an extension
                            tracing::debug!("Hook '{}' not found in workspace config: {}", name, e);
                        } else {
                            if let Err(e) = config.save(&hooks_config_path) {
                                eprintln!("Warning: Failed to save hook config: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load hook config: {}", e);
                    }
                }
            }
            
            println!("Hook '{}' enabled.", name);
        }

        HooksCommand::Disable { name } => {
            // Update registry
            if let Err(e) = registry.set_enabled(&name, false).await {
                eprintln!("Failed to disable hook '{}': {}", name, e);
                return Ok(());
            }
            
            // Update workspace config
            let hooks_config_path = workspace.root().join(".radium").join("hooks.toml");
            if hooks_config_path.exists() {
                match radium_core::hooks::config::HookConfig::from_file(&hooks_config_path) {
                    Ok(mut config) => {
                        if let Err(e) = config.set_hook_enabled(&name, false) {
                            // Hook not in config, that's okay - it might be from an extension
                            tracing::debug!("Hook '{}' not found in workspace config: {}", name, e);
                        } else {
                            if let Err(e) = config.save(&hooks_config_path) {
                                eprintln!("Warning: Failed to save hook config: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load hook config: {}", e);
                    }
                }
            }
            
            println!("Hook '{}' disabled.", name);
        }

        HooksCommand::Validate { name, verbose, json } => {
            validate_hooks(workspace.root(), name.as_deref(), verbose, json).await?;
        }

        HooksCommand::Test { name, r#type, json } => {
            test_hook(&registry, &name, r#type.as_deref(), json).await?;
        }
    }

    Ok(())
}

/// Validate hook configurations.
async fn validate_hooks(
    workspace_root: &std::path::Path,
    name_filter: Option<&str>,
    verbose: bool,
    json: bool,
) -> anyhow::Result<()> {
    if !json {
        println!("{}", "rad hooks validate".bold().cyan());
        println!();
    }

    let mut config_files = Vec::new();
    let mut errors = Vec::new();
    let mut valid_count = 0;
    let mut invalid_count = 0;

    // Load workspace configuration
    let workspace_config_path = workspace_root.join(".radium").join("hooks.toml");
    if workspace_config_path.exists() {
        config_files.push(("workspace".to_string(), workspace_config_path));
    }

    // Load extension configurations
    match HookLoader::discover_config_files() {
        Ok(extension_configs) => {
            for path in extension_configs {
                let source = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .map(|s| format!("extension:{}", s))
                    .unwrap_or_else(|| "unknown".to_string());
                config_files.push((source, path));
            }
        }
        Err(e) => {
            if verbose {
                tracing::warn!("Failed to discover extension configs: {}", e);
            }
        }
    }

    if config_files.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "valid": 0,
                    "invalid": 0,
                    "errors": []
                })
            );
        } else {
            println!("  {} No hook configuration files found.", "•".dimmed());
        }
        return Ok(());
    }

    // Validate each configuration file
    for (source, config_path) in config_files {
        match HookConfig::from_file(&config_path) {
            Ok(config) => {
                // Filter hooks if name provided
                let hooks_to_validate: Vec<_> = if let Some(filter_name) = name_filter {
                    config
                        .hooks
                        .iter()
                        .filter(|h| h.name == filter_name)
                        .collect()
                } else {
                    config.hooks.iter().collect()
                };

                for hook in hooks_to_validate {
                    match config.validate() {
                        Ok(()) => {
                            valid_count += 1;
                            if verbose && !json {
                                println!(
                                    "  {} {} (from {})",
                                    "✓".green(),
                                    hook.name.cyan(),
                                    source.dimmed()
                                );
                            }
                        }
                        Err(e) => {
                            invalid_count += 1;
                            let error_msg = format!("{}: {}", config_path.display(), e);
                            errors.push((hook.name.clone(), error_msg.clone()));
                            if !json {
                                println!(
                                    "  {} {} {}",
                                    "✗".red(),
                                    hook.name.cyan(),
                                    "(invalid)".red()
                                );
                                if verbose {
                                    println!("    {}", format!("  • {}", error_msg).dimmed());
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                invalid_count += 1;
                let error_msg = format!("{}: Failed to parse: {}", config_path.display(), e);
                errors.push((config_path.display().to_string(), error_msg.clone()));
                if !json {
                    println!(
                        "  {} {} {}",
                        "✗".red(),
                        config_path.display().to_string().cyan(),
                        "(parse error)".red()
                    );
                    if verbose {
                        println!("    {}", format!("  • {}", error_msg).dimmed());
                    }
                }
            }
        }
    }

    if json {
        let json_result = serde_json::json!({
            "valid": valid_count,
            "invalid": invalid_count,
            "errors": errors.iter().map(|(name, msg)| serde_json::json!({
                "hook": name,
                "error": msg
            })).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&json_result)?);
    } else {
        println!();
        println!("  {} Valid: {}", "•".dimmed(), valid_count.to_string().green());
        println!("  {} Invalid: {}", "•".dimmed(), invalid_count.to_string().red());
        println!();
    }

    if invalid_count > 0 {
        anyhow::bail!("Validation failed: {} hook(s) have errors", invalid_count);
    }

    Ok(())
}

/// Test hook execution with sample context.
async fn test_hook(
    registry: &Arc<HookRegistry>,
    hook_name: &str,
    hook_type_override: Option<&str>,
    json: bool,
) -> anyhow::Result<()> {
    // Find the hook
    let mut found_hook: Option<(HookType, Arc<dyn radium_core::hooks::registry::Hook>)> = None;

    for hook_type in [
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
        let hooks = registry.get_hooks(hook_type).await;
        if let Some(h) = hooks.iter().find(|hook| hook.name() == hook_name) {
            found_hook = Some((hook_type, h.clone()));
            break;
        }
    }

    let (actual_hook_type, _hook) = match found_hook {
        Some((ht, h)) => (ht, h),
        None => {
            anyhow::bail!("Hook '{}' not found", hook_name);
        }
    };

    // Determine hook type to use (override or actual)
    let test_hook_type = if let Some(type_str) = hook_type_override {
        match type_str {
            "before_model" => HookType::BeforeModel,
            "after_model" => HookType::AfterModel,
            "before_tool" => HookType::BeforeTool,
            "after_tool" => HookType::AfterTool,
            "tool_selection" => HookType::ToolSelection,
            "error_interception" => HookType::ErrorInterception,
            "error_transformation" => HookType::ErrorTransformation,
            "error_recovery" => HookType::ErrorRecovery,
            "error_logging" => HookType::ErrorLogging,
            "telemetry_collection" => HookType::TelemetryCollection,
            "custom_logging" => HookType::CustomLogging,
            "metrics_aggregation" => HookType::MetricsAggregation,
            "performance_monitoring" => HookType::PerformanceMonitoring,
            _ => {
                anyhow::bail!("Invalid hook type: {}", type_str);
            }
        }
    } else {
        actual_hook_type
    };

    // Create sample context based on hook type
    let context = create_sample_context(test_hook_type)?;

    if !json {
        println!("{}", "rad hooks test".bold().cyan());
        println!();
        println!("  Testing hook: {}", hook_name.cyan());
        println!("  Type: {}", test_hook_type.as_str());
        println!();
    }

    // Execute hook
    let start_time = Instant::now();
    let result = registry.execute_hooks(test_hook_type, &context).await;
    let duration = start_time.elapsed();

    match result {
        Ok(results) => {
            if json {
                let json_result = serde_json::json!({
                    "hook": hook_name,
                    "type": test_hook_type.as_str(),
                    "duration_ms": duration.as_millis(),
                    "results": results.iter().map(|r| serde_json::json!({
                        "success": r.success,
                        "message": r.message,
                        "should_continue": r.should_continue,
                        "has_modified_data": r.modified_data.is_some()
                    })).collect::<Vec<_>>()
                });
                println!("{}", serde_json::to_string_pretty(&json_result)?);
            } else {
                println!("  {} Execution successful", "✓".green());
                println!("  {} Duration: {:?}", "•".dimmed(), duration);
                println!("  {} Results: {}", "•".dimmed(), results.len());
                for (i, result) in results.iter().enumerate() {
                    println!("    [{}] Success: {}, Continue: {}", i + 1, result.success, result.should_continue);
                    if let Some(msg) = &result.message {
                        println!("         Message: {}", msg);
                    }
                    if result.modified_data.is_some() {
                        println!("         Modified data: present");
                    }
                }
            }
        }
        Err(e) => {
            if json {
                let json_result = serde_json::json!({
                    "hook": hook_name,
                    "type": test_hook_type.as_str(),
                    "duration_ms": duration.as_millis(),
                    "error": format!("{}", e)
                });
                println!("{}", serde_json::to_string_pretty(&json_result)?);
            } else {
                println!("  {} Execution failed: {}", "✗".red(), e);
                println!("  {} Duration: {:?}", "•".dimmed(), duration);
            }
            anyhow::bail!("Hook test failed: {}", e);
        }
    }

    Ok(())
}

/// Create a sample context for testing based on hook type.
fn create_sample_context(hook_type: HookType) -> anyhow::Result<HookContext> {
    use serde_json::json;

    let context = match hook_type {
        HookType::BeforeModel => {
            let model_ctx = ModelHookContext::before(
                "Test input prompt".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
            );
            model_ctx.to_hook_context(ModelHookType::Before)
        }
        HookType::AfterModel => {
            let model_ctx = ModelHookContext::after(
                "Test input prompt".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
                "Test response".to_string(),
            );
            model_ctx.to_hook_context(ModelHookType::After)
        }
        HookType::BeforeTool => {
            let tool_ctx = ToolHookContext::before(
                "test_tool".to_string(),
                json!({"arg1": "value1", "arg2": 42}),
            );
            tool_ctx.to_hook_context(ToolHookType::Before)
        }
        HookType::AfterTool => {
            let tool_ctx = ToolHookContext::after(
                "test_tool".to_string(),
                json!({"arg1": "value1"}),
                json!({"result": "success"}),
            );
            tool_ctx.to_hook_context(ToolHookType::After)
        }
        HookType::ToolSelection => {
            let tool_ctx = ToolHookContext::selection(
                "test_tool".to_string(),
                json!({"arg1": "value1"}),
            );
            tool_ctx.to_hook_context(ToolHookType::Selection)
        }
        HookType::ErrorInterception => {
            let error_ctx = ErrorHookContext::interception(
                "Test error message".to_string(),
                "TestError".to_string(),
                Some("test_source".to_string()),
            );
            error_ctx.to_hook_context(ErrorHookType::Interception)
        }
        HookType::ErrorTransformation => {
            let error_ctx = ErrorHookContext::transformation(
                "Test error message".to_string(),
                "TestError".to_string(),
                Some("test_source".to_string()),
            );
            error_ctx.to_hook_context(ErrorHookType::Transformation)
        }
        HookType::ErrorRecovery => {
            let error_ctx = ErrorHookContext::recovery(
                "Test error message".to_string(),
                "TestError".to_string(),
                Some("test_source".to_string()),
            );
            error_ctx.to_hook_context(ErrorHookType::Recovery)
        }
        HookType::ErrorLogging => {
            let error_ctx = ErrorHookContext::logging(
                "Test error message".to_string(),
                "TestError".to_string(),
                Some("test_source".to_string()),
            );
            error_ctx.to_hook_context(ErrorHookType::Logging)
        }
        HookType::TelemetryCollection => {
            let telemetry_ctx = TelemetryHookContext::new(
                "test_event".to_string(),
                json!({"key": "value"}),
            );
            telemetry_ctx.to_hook_context("telemetry_collection")
        }
        HookType::CustomLogging => HookContext::new(
            "custom_logging",
            json!({
                "level": "info",
                "message": "Test log message"
            }),
        ),
        HookType::MetricsAggregation => HookContext::new(
            "metrics_aggregation",
            json!({
                "metric": "test_metric",
                "value": 42.0
            }),
        ),
        HookType::PerformanceMonitoring => HookContext::new(
            "performance_monitoring",
            json!({
                "operation": "test_operation",
                "duration_ms": 100
            }),
        ),
    };

    Ok(context)
}


//! Agents command implementation.
//!
//! Provides commands for discovering, searching, and managing agents.

use super::{AgentsCommand, MigrateSubcommand};
use colored::Colorize;
use radium_core::agents::config::{AgentConfig, AgentConfigFile, ReasoningEffort};
use radium_core::agents::discovery::AgentDiscovery;
use radium_core::agents::registry::{
    AgentRegistry, FilterCriteria, LogicMode, SearchMode, SortField, SortOrder,
};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tabled::{Table, Tabled, settings::Style};

/// Execute the agents command.
pub async fn execute(command: AgentsCommand) -> anyhow::Result<()> {
    match command {
        AgentsCommand::List { json, verbose } => list_agents(json, verbose).await,
        AgentsCommand::Search {
            query,
            json,
            category,
            engine,
            model,
            tags,
            sort,
            fuzzy,
            or,
        } => {
            search_agents(
                &query,
                json,
                category.as_deref(),
                engine.as_deref(),
                model.as_deref(),
                tags.as_deref(),
                sort.as_deref(),
                fuzzy,
                or,
            )
            .await
        }
        AgentsCommand::Info { id, json } => show_agent_info(&id, json).await,
        AgentsCommand::Validate { verbose, json, strict } => {
            validate_agents(verbose, json, strict).await
        }
        AgentsCommand::Lint { id, json, strict } => lint_agents(id.as_deref(), json, strict).await,
        AgentsCommand::Migrate { subcommand } => migrate_agents(subcommand).await,
        AgentsCommand::Create {
            id,
            name,
            description,
            category,
            engine,
            model,
            reasoning,
            output,
            template,
            interactive,
        } => {
            create_agent(
                id.as_deref(),
                name.as_deref(),
                description.as_deref(),
                category.as_deref(),
                engine.as_deref(),
                model.as_deref(),
                reasoning.as_deref(),
                output.as_deref(),
                template.as_deref(),
                interactive,
            )
            .await
        }
    }
}

/// List all available agents.
async fn list_agents(json_output: bool, verbose: bool) -> anyhow::Result<()> {
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all()?;

    if agents.is_empty() {
        if !json_output {
            println!("{}", "No agents found.".yellow());
            println!();
            println!("Try creating agents in:");
            println!("  • ./agents/ (project-local)");
            println!("  • ~/.radium/agents/ (user-level)");
        }
        return Ok(());
    }

    if json_output {
        let agent_list: Vec<_> = agents
            .iter()
            .map(|(id, config)| {
                json!({
                    "id": id,
                    "name": config.name,
                    "description": config.description,
                    "category": config.category,
                    "engine": config.engine,
                    "model": config.model,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&agent_list)?);
    } else {
        println!();
        println!("{}", format!("📦 Found {} agents", agents.len()).bold().green());
        println!();

        if verbose {
            display_agents_detailed(&agents);
        } else {
            display_agents_table(&agents);
        }
    }

    Ok(())
}

/// Search for agents by query with optional filters and sorting.
async fn search_agents(
    query: &str,
    json_output: bool,
    category_filter: Option<&str>,
    engine_filter: Option<&str>,
    model_filter: Option<&str>,
    tags_filter: Option<&str>,
    sort_option: Option<&str>,
    fuzzy: bool,
    or_logic: bool,
) -> anyhow::Result<()> {

    // Create registry and discover agents
    let registry = AgentRegistry::with_discovery()?;

    // Build filter criteria
    let mut criteria = FilterCriteria::default();
    if let Some(cat) = category_filter {
        criteria.category = Some(cat.to_string());
    }
    if let Some(eng) = engine_filter {
        criteria.engine = Some(eng.to_string());
    }
    if let Some(modl) = model_filter {
        criteria.model = Some(modl.to_string());
    }
    if let Some(tags_str) = tags_filter {
        criteria.tags = Some(tags_str.split(',').map(|s| s.trim().to_string()).collect());
    }

    // Set search mode and logic mode
    criteria.search_mode = if fuzzy {
        SearchMode::Fuzzy
    } else {
        SearchMode::Contains
    };
    criteria.logic_mode = if or_logic {
        LogicMode::Or
    } else {
        LogicMode::And
    };

    // Apply filters if any are specified
    let mut candidates = if criteria.category.is_some()
        || criteria.engine.is_some()
        || criteria.model.is_some()
        || criteria.tags.is_some()
    {
        registry.filter_combined(&criteria)?
    } else {
        registry.list_all()?
    };

    // Apply text search query using the search mode
    if !query.is_empty() {
        let search_mode = if fuzzy {
            SearchMode::Fuzzy
        } else {
            SearchMode::Contains
        };
        let search_results = registry.search_with_mode(query, search_mode)?;
        // Intersect with filtered results
        let search_ids: std::collections::HashSet<String> =
            search_results.iter().map(|a| a.id.clone()).collect();
        candidates.retain(|config| search_ids.contains(&config.id));
    }

    // Apply sorting if specified
    if let Some(sort_str) = sort_option {
        let sort_order = if sort_str.contains(',') {
            // Multi-field sort
            let fields: Vec<SortField> = sort_str
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter_map(|s| match s.as_str() {
                    "name" => Some(SortField::Name),
                    "category" => Some(SortField::Category),
                    "engine" => Some(SortField::Engine),
                    "model" => Some(SortField::Model),
                    "id" => Some(SortField::Id),
                    _ => None,
                })
                .collect();
            if fields.is_empty() {
                SortOrder::Name
            } else {
                SortOrder::Multiple(fields)
            }
        } else {
            match sort_str.to_lowercase().as_str() {
                "name" => SortOrder::Name,
                "category" => SortOrder::Category,
                "engine" => SortOrder::Engine,
                _ => {
                    eprintln!(
                        "{} Invalid sort option: {}. Valid options: name, category, engine, model, id or comma-separated (e.g., category,name)",
                        "⚠️".yellow(),
                        sort_str
                    );
                    SortOrder::Name // Default to name
                }
            }
        };
        // Create a temporary registry with filtered agents for sorting
        let temp_registry = AgentRegistry::new();
        for agent in &candidates {
            let _ = temp_registry.register_or_replace(agent.clone());
        }
        candidates = temp_registry.sort(sort_order)?;
    }

    if candidates.is_empty() {
        if !json_output {
            println!("{}", format!("No agents found matching '{}'", query).yellow());
            if category_filter.is_some() || engine_filter.is_some() || model_filter.is_some() {
                println!("  Applied filters:");
                if let Some(cat) = category_filter {
                    println!("    Category: {}", cat);
                }
                if let Some(eng) = engine_filter {
                    println!("    Engine: {}", eng);
                }
                if let Some(modl) = model_filter {
                    println!("    Model: {}", modl);
                }
            }
        }
        return Ok(());
    }

    // Convert to HashMap for display
    let matches: HashMap<String, AgentConfig> = candidates
        .into_iter()
        .map(|config| (config.id.clone(), config))
        .collect();

    if json_output {
        let results: Vec<_> = matches
            .iter()
            .map(|(id, config)| {
                json!({
                    "id": id,
                    "name": config.name,
                    "description": config.description,
                    "category": config.category,
                    "engine": config.engine,
                    "model": config.model,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        println!();
        println!(
            "{}",
            format!("🔍 Found {} matching agents for '{}'", matches.len(), query).bold().green()
        );
        if category_filter.is_some()
            || engine_filter.is_some()
            || model_filter.is_some()
            || tags_filter.is_some()
        {
            println!("  {} Filters applied:", "•".cyan());
            if let Some(cat) = category_filter {
                println!("    Category: {}", cat.cyan());
            }
            if let Some(eng) = engine_filter {
                println!("    Engine: {}", eng.cyan());
            }
            if let Some(modl) = model_filter {
                println!("    Model: {}", modl.cyan());
            }
            if let Some(tags) = tags_filter {
                println!("    Tags: {}", tags.cyan());
            }
            if fuzzy {
                println!("    Search mode: {}", "fuzzy".cyan());
            }
            if or_logic {
                println!("    Logic: {}", "OR".cyan());
            }
        }
        if let Some(sort_str) = sort_option {
            println!("  {} Sorted by: {}", "•".cyan(), sort_str.cyan());
        }
        println!();
        display_agents_table(&matches);
    }

    Ok(())
}

/// Show detailed information about a specific agent.
async fn show_agent_info(id: &str, json_output: bool) -> anyhow::Result<()> {
    let discovery = AgentDiscovery::new();
    let agent =
        discovery.find_by_id(id)?.ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", id))?;

    if json_output {
        let info = json!({
            "id": agent.id,
            "name": agent.name,
            "description": agent.description,
            "category": agent.category,
            "prompt_path": agent.prompt_path,
            "engine": agent.engine,
            "model": agent.model,
            "reasoning_effort": agent.reasoning_effort,
            "file_path": agent.file_path,
        });
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!();
        println!("{}", format!("📋 Agent: {}", agent.name).bold().cyan());
        println!();
        println!("{}", "Details:".bold());
        println!("  ID:          {}", agent.id.green());
        println!("  Name:        {}", agent.name);
        if !agent.description.is_empty() {
            println!("  Description: {}", agent.description);
        }
        if let Some(category) = &agent.category {
            println!("  Category:    {}", category.yellow());
        }
        println!();
        println!("{}", "Configuration:".bold());
        println!("  Prompt Path: {}", agent.prompt_path.display());
        if let Some(engine) = &agent.engine {
            println!("  Engine:      {}", engine.cyan());
        }
        if let Some(model) = &agent.model {
            println!("  Model:       {}", model.cyan());
        }
        if let Some(effort) = &agent.reasoning_effort {
            println!("  Reasoning:   {:?}", effort);
        }
        println!();
        if let Some(path) = &agent.file_path {
            println!("{}", "Source:".bold());
            println!("  {}", path.display().to_string().dimmed());
        }
        println!();
    }

    Ok(())
}

/// Validation result for a single agent.
#[derive(Debug, Clone)]
struct AgentValidationResult {
    id: String,
    file_path: Option<PathBuf>,
    valid: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
}

/// Validate all agent configurations.
async fn validate_agents(verbose: bool, json_output: bool, strict: bool) -> anyhow::Result<()> {
    use radium_core::agents::metadata::AgentMetadata;
    use radium_core::prompts::templates::PromptTemplate;

    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all()?;
    let validator = AgentValidatorImpl::new();

    let mut results = Vec::new();

    for (id, config) in &agents {
        let mut validation_result = AgentValidationResult {
            id: id.clone(),
            file_path: config.file_path.clone(),
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Reload and validate the config file to get comprehensive validation
        if let Some(config_path) = &config.file_path {
            // Use the new validation system
            match validator.validate(&config, Some(config_path)) {
                Ok(_) => {
                    // Additional validations beyond basic validation
                    match AgentConfigFile::load(config_path) {
                        Ok(config_file) => {
                            let agent = &config_file.agent;

                        // Validate agent ID format (kebab-case)
                        if !agent.id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                            validation_result.errors.push(format!(
                                "Agent ID '{}' must be in kebab-case (lowercase letters, numbers, hyphens only)",
                                agent.id
                            ));
                            validation_result.valid = false;
                        }

                        // Validate prompt file exists and is readable
                        let prompt_path = &agent.prompt_path;
                        let prompt_exists = if prompt_path.is_absolute() {
                            prompt_path.exists() && prompt_path.is_file()
                        } else if let Some(config_dir) = config_path.parent() {
                            let full_path = config_dir.join(prompt_path);
                            full_path.exists() && full_path.is_file()
                        } else {
                            false
                        };

                        if !prompt_exists {
                            validation_result.errors.push(format!(
                                "Prompt file not found or not readable: {}",
                                prompt_path.display()
                            ));
                            validation_result.valid = false;
                        } else {
                            // Try to load prompt template to verify it's valid
                            let resolved_path = if agent.prompt_path.is_absolute() {
                                agent.prompt_path.clone()
                            } else if let Some(config_dir) = config_path.parent() {
                                config_dir.join(&agent.prompt_path)
                            } else {
                                agent.prompt_path.clone()
                            };

                            if let Err(e) = PromptTemplate::load(&resolved_path) {
                                validation_result.errors.push(format!(
                                    "Failed to load prompt template: {}",
                                    e
                                ));
                                validation_result.valid = false;
                            } else {
                                // Try to parse YAML frontmatter if present
                                if let Ok(content) = fs::read_to_string(&resolved_path) {
                                    if content.trim_start().starts_with("---") {
                                        match AgentMetadata::from_markdown(&content) {
                                            Ok((metadata, _)) => {
                                                if verbose {
                                                    validation_result.warnings.push(format!(
                                                        "Metadata parsed: name={}, color={}",
                                                        metadata.name, metadata.color
                                                    ));
                                                }
                                            }
                                            Err(e) => {
                                                if strict {
                                                    validation_result.errors.push(format!(
                                                        "YAML frontmatter parsing error: {}",
                                                        e
                                                    ));
                                                    validation_result.valid = false;
                                                } else {
                                                    validation_result.warnings.push(format!(
                                                        "YAML frontmatter parsing warning: {}",
                                                        e
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                        // Validate loop behavior if present
                        if let Some(loop_behavior) = &agent.loop_behavior {
                            if loop_behavior.steps == 0 {
                                validation_result.errors.push(
                                    "Loop behavior: steps must be greater than 0".to_string(),
                                );
                                validation_result.valid = false;
                            }
                            if let Some(max_iter) = loop_behavior.max_iterations {
                                if max_iter == 0 {
                                    validation_result.errors.push(
                                        "Loop behavior: max_iterations must be greater than 0".to_string(),
                                    );
                                    validation_result.valid = false;
                                }
                            }
                        }

                        // Validate capabilities if present
                        if agent.capabilities.max_concurrent_tasks == 0 {
                            validation_result.errors.push(
                                "Capabilities: max_concurrent_tasks must be greater than 0".to_string(),
                            );
                            validation_result.valid = false;
                        }
                    }
                }
                Err(e) => {
                    validation_result.valid = false;
                    validation_result.errors.push(e.to_string());
                }
            }
        } else {
            // If file_path is not set, we can't reload, so do basic validation
            validation_result.valid = false;
            if config.name.is_empty() {
                validation_result.errors.push("Name is empty".to_string());
            }
            if config.prompt_path.as_os_str().is_empty() {
                validation_result.errors.push("Prompt path is empty".to_string());
            }
        }

        results.push(validation_result);
    }

    let valid_count = results.iter().filter(|r| r.valid && r.errors.is_empty()).count();
    let error_count = results.iter().filter(|r| !r.valid || !r.errors.is_empty()).count();

    if json_output {
        // Output JSON format
        let json_results: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                json!({
                    "id": r.id,
                    "file_path": r.file_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                    "valid": r.valid && r.errors.is_empty(),
                    "errors": r.errors,
                    "warnings": r.warnings,
                })
            })
            .collect();

        let output = json!({
            "summary": {
                "total": results.len(),
                "valid": valid_count,
                "invalid": error_count,
            },
            "results": json_results,
        });

        println!("{}", serde_json::to_string_pretty(&output)?);
        
        // Exit with non-zero code if there are errors
        if error_count > 0 {
            std::process::exit(1);
        }
    } else {
        // Output human-readable format
        println!();
        if error_count == 0 {
            println!(
                "{}",
                format!("✅ All {} agents validated successfully", results.len()).bold().green()
            );
        } else {
            println!(
                "{}",
                format!("⚠️  Validation: {} valid, {} with errors", valid_count, error_count)
                    .bold()
                    .yellow()
            );
            println!();

            if verbose {
                for result in &results {
                    if !result.valid || !result.errors.is_empty() || !result.warnings.is_empty() {
                        println!("{}", format!("  {} {}:", "❌".red(), result.id.red()));
                        if let Some(path) = &result.file_path {
                            println!("     {}", format!("File: {}", path.display()).dimmed());
                        }
                        for error in &result.errors {
                            println!("     • {}", error.red());
                        }
                        for warning in &result.warnings {
                            println!("     ⚠ {}", warning.yellow());
                        }
                        println!();
                    }
                }
            } else {
                println!("Run with {} for details", "--verbose".cyan());
            }
        }
        println!();
    }

    Ok(())
}

/// Lint agent prompt templates.
async fn lint_agents(agent_id: Option<&str>, json_output: bool, strict: bool) -> anyhow::Result<()> {
    let discovery = AgentDiscovery::new();
    let linter = PromptLinter::new();
    let mut results = Vec::new();

    let agents = if let Some(id) = agent_id {
        let agent = discovery
            .find_by_id(id)?
            .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", id))?;
        vec![(id.to_string(), agent)]
    } else {
        discovery.discover_all()?
    };

    for (id, config) in &agents {
        if let Some(config_path) = &config.file_path {
            let prompt_path = if config.prompt_path.is_absolute() {
                config.prompt_path.clone()
            } else if let Some(config_dir) = config_path.parent() {
                config_dir.join(&config.prompt_path)
            } else {
                config.prompt_path.clone()
            };

            if prompt_path.exists() {
                match linter.lint(&prompt_path) {
                    Ok(mut lint_result) => {
                        // In strict mode, treat warnings as errors
                        if strict && !lint_result.warnings.is_empty() {
                            lint_result.valid = false;
                            lint_result.errors.extend(lint_result.warnings.drain(..));
                        }
                        results.push((id.clone(), Some(config_path.clone()), lint_result));
                    }
                    Err(e) => {
                        let mut lint_result = LintResult::new();
                        lint_result.valid = false;
                        lint_result.errors.push(e.to_string());
                        results.push((id.clone(), Some(config_path.clone()), lint_result));
                    }
                }
            } else {
                let mut lint_result = LintResult::new();
                lint_result.valid = false;
                lint_result.errors.push(format!(
                    "Prompt file not found: {}",
                    prompt_path.display()
                ));
                results.push((id.clone(), Some(config_path.clone()), lint_result));
            }
        }
    }

    let valid_count = results.iter().filter(|(_, _, r)| r.valid).count();
    let error_count = results.len() - valid_count;

    if json_output {
        let json_results: Vec<serde_json::Value> = results
            .iter()
            .map(|(id, path, result)| {
                json!({
                    "id": id,
                    "file_path": path.as_ref().map(|p| p.to_string_lossy().to_string()),
                    "valid": result.valid,
                    "errors": result.errors,
                    "warnings": result.warnings,
                })
            })
            .collect();

        let output = json!({
            "summary": {
                "total": results.len(),
                "valid": valid_count,
                "invalid": error_count,
            },
            "results": json_results,
        });

        println!("{}", serde_json::to_string_pretty(&output)?);
        
        if error_count > 0 {
            std::process::exit(1);
        }
    } else {
        println!();
        if error_count == 0 {
            println!(
                "{}",
                format!("✅ All {} prompt templates linted successfully", results.len())
                    .bold()
                    .green()
            );
        } else {
            println!(
                "{}",
                format!("⚠️  Linting: {} valid, {} with errors", valid_count, error_count)
                    .bold()
                    .yellow()
            );
            println!();

            for (id, path, result) in &results {
                if !result.valid || !result.errors.is_empty() || !result.warnings.is_empty() {
                    println!("{}", format!("  {} {}:", "❌".red(), id.red()));
                    if let Some(p) = path {
                        println!("     {}", format!("File: {}", p.display()).dimmed());
                    }
                    for error in &result.errors {
                        println!("     • {}", error.red());
                    }
                    for warning in &result.warnings {
                        println!("     ⚠ {}", warning.yellow());
                    }
                    println!();
                }
            }
        }
        println!();
    }

    Ok(())
}

/// Display agents in a compact table format.
fn display_agents_table(agents: &HashMap<String, AgentConfig>) {
    #[derive(Tabled)]
    struct AgentRow {
        #[tabled(rename = "ID")]
        id: String,
        #[tabled(rename = "Name")]
        name: String,
        #[tabled(rename = "Category")]
        category: String,
        #[tabled(rename = "Engine")]
        engine: String,
        #[tabled(rename = "Model")]
        model: String,
    }

    let mut rows: Vec<AgentRow> = agents
        .iter()
        .map(|(id, config)| AgentRow {
            id: id.clone(),
            name: config.name.clone(),
            category: config.category.clone().unwrap_or_else(|| "-".to_string()),
            engine: config.engine.clone().unwrap_or_else(|| "-".to_string()),
            model: config.model.clone().unwrap_or_else(|| "-".to_string()),
        })
        .collect();

    // Sort by ID
    rows.sort_by(|a, b| a.id.cmp(&b.id));

    let table = Table::new(rows).with(Style::rounded()).to_string();

    println!("{}", table);
    println!();
}

/// Display agents in detailed format.
fn display_agents_detailed(agents: &HashMap<String, AgentConfig>) {
    let mut agent_list: Vec<_> = agents.iter().collect();
    agent_list.sort_by_key(|(id, _)| id.as_str());

    for (id, config) in agent_list {
        println!("{}", format!("  {} {}", "●".green(), id.bold()));
        println!("    Name:     {}", config.name);
        if !config.description.is_empty() {
            println!("    Desc:     {}", config.description);
        }
        if let Some(category) = &config.category {
            println!("    Category: {}", category.yellow());
        }
        if let Some(engine) = &config.engine {
            println!("    Engine:   {}", engine.cyan());
        }
        if let Some(model) = &config.model {
            println!("    Model:    {}", model.cyan());
        }
        println!();
    }
}

/// Create a new agent template.
#[allow(clippy::too_many_arguments)]
async fn create_agent(
    id: Option<&str>,
    name: Option<&str>,
    description: Option<&str>,
    category: Option<&str>,
    engine: Option<&str>,
    model: Option<&str>,
    reasoning: Option<&str>,
    output_dir: Option<&str>,
    template: Option<&str>,
    interactive: bool,
) -> anyhow::Result<()> {
    // Interactive mode: prompt for all fields
    let (id_str, name_str, description_str, category_str, engine_str, model_str, reasoning_str) = if interactive {
        interactive_prompt_agent_details(id, name, description, category, engine, model, reasoning)?
    } else {
        // Validate required fields for non-interactive mode
        let id = id.ok_or_else(|| anyhow::anyhow!("Agent ID is required (use --interactive to prompt)"))?;
        let name = name.ok_or_else(|| anyhow::anyhow!("Agent name is required (use --interactive to prompt)"))?;
        (
            id.to_string(),
            name.to_string(),
            description.map(|s| s.to_string()),
            category.map(|s| s.to_string()),
            engine.map(|s| s.to_string()),
            model.map(|s| s.to_string()),
            reasoning.map(|s| s.to_string()),
        )
    };
    
    let id = id_str.as_str();
    let name = name_str.as_str();
    let description = description_str.as_deref();
    let category = category_str.as_deref();
    let engine = engine_str.as_deref();
    let model = model_str.as_deref();
    let reasoning = reasoning_str.as_deref();

    // Validate agent ID
    if id.is_empty() {
        anyhow::bail!("Agent ID cannot be empty");
    }

    // Use default category if not provided
    let category = category.unwrap_or("custom");

    // Determine output directory
    let base_dir = output_dir.unwrap_or("./agents");
    let agent_dir = Path::new(base_dir).join(category);

    // Create directory structure
    fs::create_dir_all(&agent_dir)?;

    // Create prompts directory
    let prompts_dir = Path::new("./prompts/agents").join(category);
    fs::create_dir_all(&prompts_dir)?;

    // File paths
    let config_path = agent_dir.join(format!("{}.toml", id));
    let prompt_path = prompts_dir.join(format!("{}.md", id));

    // Check if agent already exists
    if config_path.exists() {
        anyhow::bail!("Agent '{}' already exists at {}", id, config_path.display());
    }

    // Load templates if specified
    let (config_template, prompt_template_content) = if let Some(template_name) = template {
        load_templates(template_name)?
    } else {
        (None, None)
    };

    // Parse reasoning effort
    let reasoning_effort = reasoning.and_then(|r| match r.to_lowercase().as_str() {
        "low" => Some(ReasoningEffort::Low),
        "medium" => Some(ReasoningEffort::Medium),
        "high" => Some(ReasoningEffort::High),
        _ => None,
    });

    // Build agent config - use template if available
    let prompt_path_relative = PathBuf::from(format!("prompts/agents/{}/{}.md", category, id));

    let mut agent = AgentConfig::new(id, name, prompt_path_relative);
    agent.description = description.unwrap_or("").to_string();

    if let Some(eng) = engine {
        agent = agent.with_engine(eng);
    }

    if let Some(mdl) = model {
        agent = agent.with_model(mdl);
    }

    if let Some(effort) = reasoning_effort {
        agent = agent.with_reasoning_effort(effort);
    }

    // Generate config content from template or default
    let config_content = if let Some(template) = config_template {
        substitute_template_variables(
            &template,
            id,
            name,
            description,
            &category,
            engine,
            model,
            reasoning,
        )?
    } else {
        // Use default generation
        let config_file = AgentConfigFile { agent: agent.clone() };
        toml::to_string_pretty(&config_file)
            .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?
    };

    // Save TOML configuration
    fs::write(&config_path, config_content)?;

    // Generate prompt template - use template if available
    let prompt_content = if let Some(template) = prompt_template_content {
        substitute_prompt_template_variables(&template, name, description)?
    } else {
        generate_prompt_template(name, description)
    };
    fs::write(&prompt_path, prompt_content)?;

    // Success output
    println!();
    println!("{}", "✅ Agent template created successfully!".bold().green());
    println!();
    println!("{}", "Files created:".bold());
    println!("  • Configuration: {}", config_path.display().to_string().cyan());
    println!("  • Prompt:        {}", prompt_path.display().to_string().cyan());
    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. Edit the prompt file to define agent behavior");
    println!("  2. Validate: {}", format!("rad agents validate").yellow());
    println!("  3. Test: {}", format!("rad agents info {}", id).yellow());
    println!();

    Ok(())
}

/// Interactive prompt for agent details.
fn interactive_prompt_agent_details(
    id: Option<&str>,
    name: Option<&str>,
    description: Option<&str>,
    category: Option<&str>,
    engine: Option<&str>,
    model: Option<&str>,
    reasoning: Option<&str>,
) -> anyhow::Result<(String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> {
    use std::io::{self, Write};

    println!();
    println!("{}", "Create New Agent (Interactive Mode)".bold().cyan());
    println!();

    // Prompt for ID
    let id = if let Some(default_id) = id {
        print!("Agent ID [{}]: ", default_id);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
            default_id.to_string()
        } else {
            input.to_string()
        }
    } else {
        print!("Agent ID: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let id = input.trim().to_string();
        if id.is_empty() {
            anyhow::bail!("Agent ID cannot be empty");
        }
        id
    };

    // Prompt for name
    let name = if let Some(default_name) = name {
        print!("Agent Name [{}]: ", default_name);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
            default_name.to_string()
        } else {
            input.to_string()
        }
    } else {
        print!("Agent Name: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    // Prompt for description
    let description = if let Some(default_desc) = description {
        print!("Description [{}]: ", default_desc);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
            Some(default_desc.to_string())
        } else if input.is_empty() {
            None
        } else {
            Some(input.to_string())
        }
    } else {
        print!("Description (optional): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
            None
        } else {
            Some(input.to_string())
        }
    };

    // Prompt for category
    let category = if let Some(default_cat) = category {
        print!("Category [{}]: ", default_cat);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
            Some(default_cat.to_string())
        } else if input.is_empty() {
            None
        } else {
            Some(input.to_string())
        }
    } else {
        print!("Category [custom]: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
            None
        } else {
            Some(input.to_string())
        }
    };

    // Prompt for engine
    let engine = if let Some(default_eng) = engine {
        print!("Engine [{}]: ", default_eng);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
            Some(default_eng.to_string())
        } else if input.is_empty() {
            None
        } else {
            Some(input.to_string())
        }
    } else {
        print!("Engine (optional, e.g., gemini, openai): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
            None
        } else {
            Some(input.to_string())
        }
    };

    // Prompt for model
    let model = if let Some(default_model) = model {
        print!("Model [{}]: ", default_model);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
            Some(default_model.to_string())
        } else if input.is_empty() {
            None
        } else {
            Some(input.to_string())
        }
    } else {
        print!("Model (optional): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
            None
        } else {
            Some(input.to_string())
        }
    };

    // Prompt for reasoning
    let reasoning = if let Some(default_reasoning) = reasoning {
        print!("Reasoning Effort [{}]: ", default_reasoning);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
            Some(default_reasoning.to_string())
        } else if input.is_empty() {
            None
        } else {
            Some(input.to_string())
        }
    } else {
        print!("Reasoning Effort [medium] (low, medium, high): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
            None
        } else {
            Some(input.to_string())
        }
    };

    Ok((id, name, description, category, engine, model, reasoning))
}

/// Load template files.
fn load_templates(template_name: &str) -> anyhow::Result<(Option<String>, Option<String>)> {
    // Try to find template in multiple locations
    let template_paths = vec![
        PathBuf::from("./templates"),
        PathBuf::from("~/.radium/templates"),
        PathBuf::from(format!("./templates/{}", template_name)),
    ];

    // For preset templates, use default templates
    let (config_template, prompt_template) = match template_name {
        "basic" | "advanced" | "workflow" => {
            // Use default templates from templates/ directory
            let config_path = PathBuf::from("./templates/agent-config.toml.template");
            let prompt_path = PathBuf::from("./templates/agent-prompt.md.template");
            
            let config_template = if config_path.exists() {
                Some(fs::read_to_string(&config_path)?)
            } else {
                None
            };
            
            let prompt_template = if prompt_path.exists() {
                Some(fs::read_to_string(&prompt_path)?)
            } else {
                None
            };
            
            (config_template, prompt_template)
        }
        _ => {
            // Custom template path
            let config_path = PathBuf::from(template_name).join("agent-config.toml.template");
            let prompt_path = PathBuf::from(template_name).join("agent-prompt.md.template");
            
            let config_template = if config_path.exists() {
                Some(fs::read_to_string(&config_path)?)
            } else {
                None
            };
            
            let prompt_template = if prompt_path.exists() {
                Some(fs::read_to_string(&prompt_path)?)
            } else {
                None
            };
            
            (config_template, prompt_template)
        }
    };

    Ok((config_template, prompt_template))
}

/// Substitute variables in config template.
fn substitute_template_variables(
    template: &str,
    id: &str,
    name: &str,
    description: Option<&str>,
    category: &str,
    engine: Option<&str>,
    model: Option<&str>,
    reasoning: Option<&str>,
) -> anyhow::Result<String> {
    let mut result = template.to_string();
    
    // Simple variable substitution ({{variable}})
    result = result.replace("{{id}}", id);
    result = result.replace("{{name}}", name);
    result = result.replace("{{description}}", description.unwrap_or(""));
    result = result.replace("{{category}}", category);
    
    // Optional fields with conditional blocks - simple removal for now
    // Remove conditional blocks for optional fields that are not set
    if let Some(eng) = engine {
        result = result.replace("{{#engine}}", "");
        result = result.replace("{{/engine}}", "");
        result = result.replace("{{engine}}", eng);
    } else {
        // Simple removal of conditional block (between {{#engine}} and {{/engine}})
        let start = result.find("{{#engine}}");
        let end = result.find("{{/engine}}");
        if let (Some(start_idx), Some(end_idx)) = (start, end) {
            let end_idx = end_idx + "{{/engine}}".len();
            result.replace_range(start_idx..end_idx, "");
        }
    }
    
    if let Some(mdl) = model {
        result = result.replace("{{#model}}", "");
        result = result.replace("{{/model}}", "");
        result = result.replace("{{model}}", mdl);
    } else {
        let start = result.find("{{#model}}");
        let end = result.find("{{/model}}");
        if let (Some(start_idx), Some(end_idx)) = (start, end) {
            let end_idx = end_idx + "{{/model}}".len();
            result.replace_range(start_idx..end_idx, "");
        }
    }
    
    if let Some(reasoning_val) = reasoning {
        result = result.replace("{{#reasoning}}", "");
        result = result.replace("{{/reasoning}}", "");
        result = result.replace("{{reasoning}}", reasoning_val);
    } else {
        let start = result.find("{{#reasoning}}");
        let end = result.find("{{/reasoning}}");
        if let (Some(start_idx), Some(end_idx)) = (start, end) {
            let end_idx = end_idx + "{{/reasoning}}".len();
            result.replace_range(start_idx..end_idx, "");
        }
    }
    
    // Remove any remaining conditional blocks (simple approach)
    while let Some(start) = result.find("{{#") {
        if let Some(end) = result[start..].find("{{/") {
            let end_marker = result[start + end..].find("}}");
            if let Some(end_marker) = end_marker {
                let end_idx = start + end + end_marker + 2;
                result.replace_range(start..end_idx, "");
            } else {
                break;
            }
        } else {
            break;
        }
    }
    
    Ok(result)
}

/// Substitute variables in prompt template.
fn substitute_prompt_template_variables(
    template: &str,
    name: &str,
    description: Option<&str>,
) -> anyhow::Result<String> {
    let mut result = template.to_string();
    
    result = result.replace("{{name}}", name);
    result = result.replace("{{description}}", description.unwrap_or("Add agent description here"));
    
    // Replace placeholder variables with defaults
    let placeholders = vec![
        ("{{domain}}", "your domain"),
        ("{{primary_responsibility}}", "performing specific tasks"),
        ("{{capability_1}}", "Capability 1"),
        ("{{capability_2}}", "Capability 2"),
        ("{{capability_3}}", "Capability 3"),
        ("{{input_1}}", "Input 1"),
        ("{{input_2}}", "Input 2"),
        ("{{input_3}}", "Input 3"),
        ("{{output_1}}", "Output 1"),
        ("{{output_2}}", "Output 2"),
        ("{{output_3}}", "Output 3"),
        ("{{step_1_title}}", "First step"),
        ("{{step_1_detail_1}}", "Detail 1"),
        ("{{step_1_detail_2}}", "Detail 2"),
        ("{{step_2_title}}", "Second step"),
        ("{{step_2_detail_1}}", "Detail 1"),
        ("{{step_2_detail_2}}", "Detail 2"),
        ("{{step_3_title}}", "Third step"),
        ("{{step_3_detail_1}}", "Detail 1"),
        ("{{step_3_detail_2}}", "Detail 2"),
        ("{{example_1_title}}", "Example Scenario"),
        ("{{example_1_input}}", "Sample input"),
        ("{{example_1_output}}", "Expected output"),
        ("{{note_1}}", "Important note 1"),
        ("{{note_2}}", "Important note 2"),
        ("{{note_3}}", "Important note 3"),
    ];
    
    for (placeholder, default) in placeholders {
        result = result.replace(placeholder, default);
    }
    
    Ok(result)
}

/// Generate a prompt template for a new agent.
fn generate_prompt_template(name: &str, description: Option<&str>) -> String {
    let desc = description.unwrap_or("Add agent description here");

    format!(
        r#"# {name}

{desc}

## Role

Define the agent's role and primary responsibilities here.

## Capabilities

- List the agent's core capabilities
- Include what tasks it can perform
- Specify any constraints or limitations

## Input

Describe what inputs this agent expects:
- Context from previous steps
- Required parameters
- Optional configuration

## Output

Describe what this agent produces:
- Expected output format
- Key deliverables
- Success criteria

## Instructions

Provide step-by-step instructions for the agent:

1. First step - explain what to do
2. Second step - detail the process
3. Third step - clarify expectations
4. Continue as needed...

## Examples

### Example 1: [Scenario Name]

**Input:**
```
Provide sample input
```

**Expected Output:**
```
Show expected result
```

### Example 2: [Another Scenario]

**Input:**
```
Different scenario input
```

**Expected Output:**
```
Corresponding output
```

## Notes

- Add any important notes
- Include edge cases to consider
- Document best practices
"#
    )
}


/// Execute migration subcommand.
async fn migrate_agents(subcommand: MigrateSubcommand) -> anyhow::Result<()> {
    match subcommand {
        MigrateSubcommand::FromJson {
            input_dir,
            output_dir,
            dry_run,
            force,
        } => migrate_from_json(&input_dir, &output_dir, dry_run, force).await,
        MigrateSubcommand::Validate { dir } => validate_migrated(&dir).await,
        MigrateSubcommand::Report { migration_id } => show_migration_report(&migration_id).await,
    }
}

/// Migrate agent configurations from JSON to TOML format.
async fn migrate_from_json(
    input_dir: &str,
    output_dir: &str,
    dry_run: bool,
    force: bool,
) -> anyhow::Result<()> {
    use std::fs;
    use std::path::Path;

    let input_path = Path::new(input_dir);
    let output_path = Path::new(output_dir);

    if !input_path.exists() {
        return Err(anyhow::anyhow!("Input directory does not exist: {}", input_dir));
    }

    if !dry_run {
        fs::create_dir_all(output_path)?;
    }

    let mut successful = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    // Find all JSON files in input directory
    let entries = fs::read_dir(input_path)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let file_name = path.file_stem()
                .and_then(|n| n.to_str())
                .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;
            
            let output_file = output_path.join(format!("{}.toml", file_name));

            if !dry_run && output_file.exists() && !force {
                errors.push(format!("File exists (use --force to overwrite): {}", output_file.display()));
                failed += 1;
                continue;
            }

            // Read and parse JSON
            let json_content = fs::read_to_string(&path)?;
            let json_value: serde_json::Value = serde_json::from_str(&json_content)?;

            // Convert JSON to AgentConfig
            let agent_config = json_to_agent_config(&json_value, file_name)?;

            // Create AgentConfigFile
            let config_file = AgentConfigFile {
                agent: agent_config,
            };

            // Validate the migrated config
            if let Err(e) = config_file.validate() {
                errors.push(format!("Validation failed for {}: {}", file_name, e));
                failed += 1;
                continue;
            }

            if dry_run {
                println!("Would migrate: {} -> {}", path.display(), output_file.display());
                successful += 1;
            } else {
                // Save as TOML
                config_file.save(&output_file)?;
                println!("Migrated: {} -> {}", path.display(), output_file.display());
                successful += 1;
            }
        }
    }

    println!();
    println!("Migration summary:");
    println!("  Successful: {}", successful);
    println!("  Failed: {}", failed);
    
    if !errors.is_empty() {
        println!();
        println!("Errors:");
        for error in &errors {
            println!("  • {}", error);
        }
    }

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Convert JSON value to AgentConfig.
fn json_to_agent_config(json: &serde_json::Value, default_id: &str) -> anyhow::Result<AgentConfig> {
    let id = json.get("id")
        .and_then(|v| v.as_str())
        .unwrap_or(default_id)
        .to_string();
    
    let name = json.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing required field: name"))?
        .to_string();
    
    let description = json.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    
    let prompt_path = json.get("prompt_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing required field: prompt_path"))?;
    
    let mut config = AgentConfig::new(id, name, PathBuf::from(prompt_path))
        .with_description(description);

    // Optional fields
    if let Some(engine) = json.get("engine").and_then(|v| v.as_str()) {
        config = config.with_engine(engine);
    }

    if let Some(model) = json.get("model").and_then(|v| v.as_str()) {
        config = config.with_model(model);
    }

    if let Some(reasoning) = json.get("reasoning_effort").and_then(|v| v.as_str()) {
        let effort = match reasoning.to_lowercase().as_str() {
            "low" => ReasoningEffort::Low,
            "medium" => ReasoningEffort::Medium,
            "high" => ReasoningEffort::High,
            _ => ReasoningEffort::Medium,
        };
        config = config.with_reasoning_effort(effort);
    }

    // Loop behavior
    if let Some(loop_behavior) = json.get("loop_behavior").and_then(|v| v.as_object()) {
        if let Some(steps) = loop_behavior.get("steps").and_then(|v| v.as_u64()) {
            let max_iterations = loop_behavior.get("max_iterations")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);
            let skip = loop_behavior.get("skip")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect())
                .unwrap_or_default();
            
            config = config.with_loop_behavior(radium_core::agents::config::AgentLoopBehavior {
                steps: steps as usize,
                max_iterations,
                skip,
            });
        }
    }

    // Trigger behavior
    if let Some(trigger_behavior) = json.get("trigger_behavior").and_then(|v| v.as_object()) {
        let trigger_agent_id = trigger_behavior.get("trigger_agent_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        config = config.with_trigger_behavior(radium_core::agents::config::AgentTriggerBehavior {
            trigger_agent_id,
        });
    }

    // Capabilities
    if let Some(capabilities) = json.get("capabilities").and_then(|v| v.as_object()) {
        use radium_core::agents::config::{AgentCapabilities, CostTier, ModelClass};
        
        let model_class = capabilities.get("model_class")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "fast" => Some(ModelClass::Fast),
                "balanced" => Some(ModelClass::Balanced),
                "reasoning" => Some(ModelClass::Reasoning),
                _ => None,
            })
            .unwrap_or(ModelClass::Balanced);
        
        let cost_tier = capabilities.get("cost_tier")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "low" => Some(CostTier::Low),
                "medium" => Some(CostTier::Medium),
                "high" => Some(CostTier::High),
                _ => None,
            })
            .unwrap_or(CostTier::Medium);
        
        let max_concurrent_tasks = capabilities.get("max_concurrent_tasks")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(5);
        
        config = config.with_capabilities(AgentCapabilities {
            model_class,
            cost_tier,
            max_concurrent_tasks,
        });
    }

    Ok(config)
}

/// Validate migrated configurations.
async fn validate_migrated(dir: &str) -> anyhow::Result<()> {
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all()?;

    let mut valid_count = 0;
    let mut errors = Vec::new();

    for (id, config) in &agents {
        if let Some(config_path) = &config.file_path {
            if config_path.starts_with(dir) {
                match AgentConfigFile::load(config_path) {
                    Ok(config_file) => {
                        if config_file.validate().is_ok() {
                            valid_count += 1;
                        } else {
                            errors.push((id.clone(), "Validation failed".to_string()));
                        }
                    }
                    Err(e) => {
                        errors.push((id.clone(), e.to_string()));
                    }
                }
            }
        }
    }

    println!();
    if errors.is_empty() {
        println!("{}", format!("✅ All {} migrated agents validated successfully", valid_count).bold().green());
    } else {
        println!("{}", format!("⚠️  Validation: {} valid, {} with errors", valid_count, errors.len()).bold().yellow());
        for (id, error) in &errors {
            println!("  {} {}: {}", "❌".red(), id.red(), error);
        }
    }
    println!();

    Ok(())
}

/// Show migration report.
async fn show_migration_report(migration_id: &str) -> anyhow::Result<()> {
    use std::path::PathBuf;
    
    let report_path = PathBuf::from(".radium/_internals/migrations").join(format!("{}.json", migration_id));
    
    if !report_path.exists() {
        return Err(anyhow::anyhow!("Migration report not found: {}", report_path.display()));
    }

    let content = std::fs::read_to_string(&report_path)?;
    let report: serde_json::Value = serde_json::from_str(&content)?;

    println!("{}", serde_json::to_string_pretty(&report)?);

    Ok(())
}

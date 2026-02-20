//! Agents command implementation.
//!
//! Provides commands for discovering, searching, and managing agents.

use super::AgentsCommand;
use colored::Colorize;
use radium_core::agents::analytics::AgentAnalyticsService;
use radium_core::agents::config::{AgentConfig, AgentConfigFile, ReasoningEffort};
use radium_core::agents::discovery::AgentDiscovery;
use radium_core::agents::linter::{AgentLinter, LintResult, PromptLinter};
use radium_core::agents::registry::{
    AgentRegistry, FilterCriteria, LogicMode, SearchMode, SortField, SortOrder,
};
use radium_core::agents::validation::AgentValidatorImpl;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tabled::{Table, Tabled, settings::Style};

/// Execute the agents command.
pub async fn execute(command: AgentsCommand) -> anyhow::Result<()> {
    match command {
        AgentsCommand::List { json, verbose, profile } => list_agents(json, verbose, profile.as_deref()).await,
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
        AgentsCommand::Persona { id, list, validate, json } => {
            if list {
                execute_persona_list(json).await
            } else if validate {
                let agent_id = id.ok_or_else(|| anyhow::anyhow!("Agent ID required for --validate"))?;
                execute_persona_validate(&agent_id, json).await
            } else {
                let agent_id = id.ok_or_else(|| anyhow::anyhow!("Agent ID required (or use --list)"))?;
                show_agent_persona(&agent_id, json).await
            }
        }
        AgentsCommand::Cost { id, input_tokens, output_tokens, json } => {
            show_agent_cost(&id, input_tokens, output_tokens, json).await
        }
        AgentsCommand::Validate { verbose, json, strict } => {
            validate_agents(verbose, json, strict).await
        }
        AgentsCommand::Lint { id, json, strict } => lint_agents(id.as_deref(), json, strict).await,
        AgentsCommand::Stats { json } => show_agent_stats(json).await,
        AgentsCommand::Popular { limit, json } => show_popular_agents(limit, json).await,
        AgentsCommand::Performance { limit, json } => show_performance_metrics(limit, json).await,
        AgentsCommand::Analytics {
            agent_id,
            all,
            category,
            from,
            to,
            json,
        } => {
            execute_analytics(agent_id.as_deref(), all, category.as_deref(), from.as_deref(), to.as_deref(), json).await
        }
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
            with_persona,
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
                with_persona,
            )
            .await
        }
    }
}

/// List all available agents.
async fn list_agents(json_output: bool, verbose: bool, profile_filter: Option<&str>) -> anyhow::Result<()> {
    let discovery = AgentDiscovery::new();
    let mut agents = discovery.discover_all()?;
    
    // Filter by performance profile if specified
    if let Some(profile) = profile_filter {
        use radium_core::agents::persona::PerformanceProfile;
        let profile_enum = match profile.to_lowercase().as_str() {
            "speed" => PerformanceProfile::Speed,
            "balanced" => PerformanceProfile::Balanced,
            "thinking" => PerformanceProfile::Thinking,
            "expert" => PerformanceProfile::Expert,
            _ => {
                eprintln!("{} Invalid profile: {}. Valid options: speed, balanced, thinking, expert", "⚠️".yellow(), profile);
                return Ok(());
            }
        };
        
        agents.retain(|_, config| {
            config.persona_config.as_ref()
                .map(|p| p.performance.profile == profile_enum)
                .unwrap_or(false)
        });
    }

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
        let mut info = json!({
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
        
        // Add persona metadata if available
        if let Some(persona) = &agent.persona_config {
            info["persona"] = json!({
                "models": {
                    "primary": {
                        "engine": persona.models.primary.engine,
                        "model": persona.models.primary.model,
                    },
                    "fallback": persona.models.fallback.as_ref().map(|f| json!({
                        "engine": f.engine,
                        "model": f.model,
                    })),
                    "premium": persona.models.premium.as_ref().map(|p| json!({
                        "engine": p.engine,
                        "model": p.model,
                    })),
                },
                "performance": {
                    "profile": format!("{:?}", persona.performance.profile),
                    "estimated_tokens": persona.performance.estimated_tokens,
                },
            });
        }
        
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
        
        // Show persona metadata if available
        if let Some(persona) = &agent.persona_config {
            println!();
            println!("{}", "Persona Configuration:".bold());
            println!("  Performance Profile: {}", format!("{:?}", persona.performance.profile).cyan());
            if let Some(tokens) = persona.performance.estimated_tokens {
                println!("  Estimated Tokens: {}", tokens.to_string().cyan());
            }
            println!("  Primary Model: {} / {}", 
                persona.models.primary.engine.cyan(),
                persona.models.primary.model.cyan());
            if let Some(fallback) = &persona.models.fallback {
                println!("  Fallback Model: {} / {}", 
                    fallback.engine.cyan(),
                    fallback.model.cyan());
            }
            if let Some(premium) = &persona.models.premium {
                println!("  Premium Model: {} / {}", 
                    premium.engine.cyan(),
                    premium.model.cyan());
            }
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

/// Show persona configuration for an agent.
async fn show_agent_persona(id: &str, json_output: bool) -> anyhow::Result<()> {
    let discovery = AgentDiscovery::new();
    let agent =
        discovery.find_by_id(id)?.ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", id))?;

    if let Some(persona) = &agent.persona_config {
        if json_output {
            let persona_json = json!({
                "agent_id": agent.id,
                "agent_name": agent.name,
                "models": {
                    "primary": {
                        "engine": persona.models.primary.engine,
                        "model": persona.models.primary.model,
                    },
                    "fallback": persona.models.fallback.as_ref().map(|f| json!({
                        "engine": f.engine,
                        "model": f.model,
                    })),
                    "premium": persona.models.premium.as_ref().map(|p| json!({
                        "engine": p.engine,
                        "model": p.model,
                    })),
                },
                "performance": {
                    "profile": format!("{:?}", persona.performance.profile),
                    "estimated_tokens": persona.performance.estimated_tokens,
                },
            });
            println!("{}", serde_json::to_string_pretty(&persona_json)?);
        } else {
            println!();
            println!("{}", format!("🎭 Persona: {}", agent.name).bold().cyan());
            println!();
            println!("{}", "Model Recommendations:".bold());
            println!(
                "  {} Primary:   {} / {}",
                "•".green(),
                persona.models.primary.engine.cyan(),
                persona.models.primary.model.cyan()
            );
            if let Some(fallback) = &persona.models.fallback {
                println!(
                    "  {} Fallback:  {} / {}",
                    "•".yellow(),
                    fallback.engine.cyan(),
                    fallback.model.cyan()
                );
            }
            if let Some(premium) = &persona.models.premium {
                println!(
                    "  {} Premium:   {} / {}",
                    "•".magenta(),
                    premium.engine.cyan(),
                    premium.model.cyan()
                );
            }
            println!();
            println!("{}", "Performance Profile:".bold());
            println!(
                "  Profile: {}",
                format!("{:?}", persona.performance.profile).cyan()
            );
            if let Some(tokens) = persona.performance.estimated_tokens {
                println!("  Estimated Tokens: {}", tokens.to_string().cyan());
            }
            println!();
        }
    } else {
        if json_output {
            let no_persona = json!({
                "agent_id": agent.id,
                "agent_name": agent.name,
                "persona": null,
                "message": "No persona configuration found for this agent",
            });
            println!("{}", serde_json::to_string_pretty(&no_persona)?);
        } else {
            println!();
            println!(
                "{}",
                format!("⚠️  No persona configuration found for agent '{}'", id).yellow()
            );
            println!();
            println!("Persona configuration can be added via:");
            println!("  1. YAML frontmatter in the agent's prompt file");
            println!("  2. TOML [agent.persona] section in the agent config file");
            println!();
        }
    }

    Ok(())
}

/// List all agents with persona configurations.
async fn execute_persona_list(json_output: bool) -> anyhow::Result<()> {
    use radium_core::agents::persona::ModelPricingDB;

    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all()?;
    let pricing_db = ModelPricingDB::new();

    let mut persona_agents = Vec::new();

    for (_, agent) in agents.iter() {
        if let Some(persona) = &agent.persona_config {
            let estimated_tokens = persona.performance.estimated_tokens.unwrap_or(2000);
            let input_tokens = (estimated_tokens as f64 * 0.7) as u64;
            let output_tokens = (estimated_tokens as f64 * 0.3) as u64;
            
            #[allow(unused_assignments)]
            let mut estimated_cost = 0.0;
            estimated_cost = pricing_db
                .get_pricing(&persona.models.primary.model)
                .estimate_cost(input_tokens, output_tokens);

            persona_agents.push((
                agent.id.clone(),
                agent.category.clone().unwrap_or_else(|| "unknown".to_string()),
                format!("{:?}", persona.performance.profile),
                format!("{}:{}", persona.models.primary.engine, persona.models.primary.model),
                estimated_cost,
            ));
        }
    }

    // Sort by category then agent ID
    persona_agents.sort_by(|a, b| {
        a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0))
    });

    if persona_agents.is_empty() {
        if !json_output {
            println!("{}", "No agents with persona configurations found.".yellow());
        }
        return Ok(());
    }

    if json_output {
        let output: Vec<_> = persona_agents
            .iter()
            .map(|(id, category, profile, model, cost)| {
                json!({
                    "agent_id": id,
                    "category": category,
                    "profile": profile,
                    "primary_model": model,
                    "estimated_cost_per_execution": cost,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!();
        println!("{}", "🎭 Agents with Persona Configurations".bold().cyan());
        println!();

        #[derive(Tabled)]
        struct PersonaListRow {
            #[tabled(rename = "Agent ID")]
            agent_id: String,
            #[tabled(rename = "Category")]
            category: String,
            #[tabled(rename = "Profile")]
            profile: String,
            #[tabled(rename = "Primary Model")]
            primary_model: String,
            #[tabled(rename = "Est. Cost/Exec ($)")]
            cost: String,
        }

        let rows: Vec<PersonaListRow> = persona_agents
            .iter()
            .map(|(id, cat, profile, model, cost)| PersonaListRow {
                agent_id: id.clone(),
                category: cat.clone(),
                profile: profile.clone(),
                primary_model: model.clone(),
                cost: format!("{:.4}", cost),
            })
            .collect();

        let table = Table::new(rows).with(Style::rounded()).to_string();
        println!("{}", table);
        println!();
    }

    Ok(())
}

/// Validate persona configuration for an agent.
async fn execute_persona_validate(agent_id: &str, json_output: bool) -> anyhow::Result<()> {
    use radium_core::agents::persona::{ModelPricingDB, PerformanceProfile};

    let discovery = AgentDiscovery::new();
    let agent = discovery
        .find_by_id(agent_id)?
        .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", agent_id))?;

    let pricing_db = ModelPricingDB::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if let Some(persona) = &agent.persona_config {
        // Validate primary model exists in pricing DB
        if pricing_db.get_pricing(&persona.models.primary.model).input_cost_per_million == 0.0
            && pricing_db.get_pricing(&persona.models.primary.model).output_cost_per_million == 0.0
        {
            warnings.push(format!(
                "Primary model '{}' not found in pricing database (using default pricing)",
                persona.models.primary.model
            ));
        }

        // Validate fallback model if present
        if let Some(fallback) = &persona.models.fallback {
            if pricing_db.get_pricing(&fallback.model).input_cost_per_million == 0.0
                && pricing_db.get_pricing(&fallback.model).output_cost_per_million == 0.0
            {
                warnings.push(format!(
                    "Fallback model '{}' not found in pricing database (using default pricing)",
                    fallback.model
                ));
            }
        }

        // Validate premium model if present
        if let Some(premium) = &persona.models.premium {
            if pricing_db.get_pricing(&premium.model).input_cost_per_million == 0.0
                && pricing_db.get_pricing(&premium.model).output_cost_per_million == 0.0
            {
                warnings.push(format!(
                    "Premium model '{}' not found in pricing database (using default pricing)",
                    premium.model
                ));
            }
        }

        // Profile is already validated by the type system, but we can check if it's reasonable
        match persona.performance.profile {
            PerformanceProfile::Expert => {
                if persona.models.premium.is_none() {
                    warnings.push("Expert profile typically requires a premium model".to_string());
                }
            }
            _ => {}
        }
    } else {
        errors.push("No persona configuration found".to_string());
    }

    let is_valid = errors.is_empty();

    if json_output {
        let output = json!({
            "agent_id": agent_id,
            "valid": is_valid,
            "errors": errors,
            "warnings": warnings,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!();
        if is_valid {
            println!("{}", format!("✅ Persona configuration for '{}' is valid", agent_id).bold().green());
            if !warnings.is_empty() {
                println!();
                println!("{}", "Warnings:".bold().yellow());
                for warning in &warnings {
                    println!("  ⚠ {}", warning.yellow());
                }
            }
        } else {
            println!("{}", format!("❌ Persona configuration for '{}' has errors", agent_id).bold().red());
            println!();
            println!("{}", "Errors:".bold().red());
            for error in &errors {
                println!("  • {}", error.red());
            }
            if !warnings.is_empty() {
                println!();
                println!("{}", "Warnings:".bold().yellow());
                for warning in &warnings {
                    println!("  ⚠ {}", warning.yellow());
                }
            }
        }
        println!();
    }

    if !is_valid {
        anyhow::bail!("Persona validation failed");
    }

    Ok(())
}

/// Show cost estimate for running an agent.
async fn show_agent_cost(
    id: &str,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    json_output: bool,
) -> anyhow::Result<()> {
    use radium_core::agents::persona::ModelPricingDB;
    
    let discovery = AgentDiscovery::new();
    let agent =
        discovery.find_by_id(id)?.ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", id))?;

    let pricing_db = ModelPricingDB::new();
    
    // Determine token estimates
    let input = input_tokens.or_else(|| {
        agent.persona_config.as_ref()
            .and_then(|p| p.performance.estimated_tokens)
            .map(|t| t / 2) // Rough estimate: half for input
    }).unwrap_or(1000);
    
    let output = output_tokens.or_else(|| {
        agent.persona_config.as_ref()
            .and_then(|p| p.performance.estimated_tokens)
            .map(|t| t / 2) // Rough estimate: half for output
    }).unwrap_or(500);

    if let Some(persona) = &agent.persona_config {
        // Calculate costs for all models in the chain
        let primary_cost = pricing_db.get_pricing(&persona.models.primary.model)
            .estimate_cost(input, output);
        let fallback_cost = persona.models.fallback.as_ref()
            .map(|f| pricing_db.get_pricing(&f.model).estimate_cost(input, output));
        let premium_cost = persona.models.premium.as_ref()
            .map(|p| pricing_db.get_pricing(&p.model).estimate_cost(input, output));

        if json_output {
            let cost_json = json!({
                "agent_id": agent.id,
                "agent_name": agent.name,
                "input_tokens": input,
                "output_tokens": output,
                "total_tokens": input + output,
                "costs": {
                    "primary": {
                        "model": format!("{}:{}", persona.models.primary.engine, persona.models.primary.model),
                        "cost": primary_cost
                    },
                    "fallback": fallback_cost.map(|c| json!({
                        "model": format!("{}:{}", persona.models.fallback.as_ref().unwrap().engine, persona.models.fallback.as_ref().unwrap().model),
                        "cost": c
                    })),
                    "premium": premium_cost.map(|c| json!({
                        "model": format!("{}:{}", persona.models.premium.as_ref().unwrap().engine, persona.models.premium.as_ref().unwrap().model),
                        "cost": c
                    }))
                },
                "estimated_cost": primary_cost
            });
            println!("{}", serde_json::to_string_pretty(&cost_json)?);
        } else {
            println!();
            println!("{}", format!("💰 Cost Estimate: {}", agent.name).bold().cyan());
            println!();
            println!("  Token Estimates:");
            println!("    Input:  {}", input.to_string().cyan());
            println!("    Output: {}", output.to_string().cyan());
            println!("    Total:  {}", (input + output).to_string().cyan());
            println!();
            println!("  Estimated Costs:");
            println!(
                "  {} Primary:   ${:.4} ({})",
                "•".green(),
                primary_cost,
                format!("{}:{}", persona.models.primary.engine, persona.models.primary.model).cyan()
            );
            if let Some((fallback, cost)) = persona.models.fallback.as_ref().zip(fallback_cost) {
                println!(
                    "  {} Fallback:  ${:.4} ({})",
                    "•".yellow(),
                    cost,
                    format!("{}:{}", fallback.engine, fallback.model).cyan()
                );
            }
            if let Some((premium, cost)) = persona.models.premium.as_ref().zip(premium_cost) {
                println!(
                    "  {} Premium:   ${:.4} ({})",
                    "•".magenta(),
                    cost,
                    format!("{}:{}", premium.engine, premium.model).cyan()
                );
            }
            println!();
        }
    } else {
        // No persona config - estimate based on default model if available
        if let Some(model) = &agent.model {
            let cost = pricing_db.get_pricing(model).estimate_cost(input, output);
            
            if json_output {
                let cost_json = json!({
                    "agent_id": agent.id,
                    "agent_name": agent.name,
                    "input_tokens": input,
                    "output_tokens": output,
                    "total_tokens": input + output,
                    "cost": cost,
                    "model": model,
                    "note": "No persona configuration - using default model"
                });
                println!("{}", serde_json::to_string_pretty(&cost_json)?);
            } else {
                println!();
                println!("{}", format!("💰 Cost Estimate: {}", agent.name).bold().cyan());
                println!();
                println!("  Token Estimates:");
                println!("    Input:  {}", input.to_string().cyan());
                println!("    Output: {}", output.to_string().cyan());
                println!("    Total:  {}", (input + output).to_string().cyan());
                println!();
                println!("  Estimated Cost: ${:.4} ({})", cost, model.cyan());
                println!();
                println!("  {} No persona configuration found.", "⚠️".yellow());
                println!("  Add persona metadata for more accurate cost estimates with fallback chains.");
                println!();
            }
        } else {
            if json_output {
                let cost_json = json!({
                    "agent_id": agent.id,
                    "agent_name": agent.name,
                    "error": "No model or persona configuration found"
                });
                println!("{}", serde_json::to_string_pretty(&cost_json)?);
            } else {
                println!();
                println!("{}", format!("💰 Cost Estimate: {}", agent.name).bold().cyan());
                println!();
                println!("  {} No model or persona configuration found.", "⚠️".yellow());
                println!("  Cannot estimate cost without model information.");
                println!();
            }
        }
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
                        Err(e) => {
                            validation_result.valid = false;
                            validation_result.errors.push(format!("Failed to load config file: {}", e));
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
        discovery.discover_all()?.into_iter().collect()
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
    with_persona: bool,
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
        use radium_core::agents::config::PersonaConfigToml;
        use radium_core::agents::config::PersonaModelsToml;
        use radium_core::agents::config::PersonaPerformanceToml;
        
        let mut config_file = AgentConfigFile { 
            agent: agent.clone(),
            persona: None,
            model: None,
            safety: None,
        };
        
        // Add persona configuration if requested
        if with_persona {
            let default_engine = engine.unwrap_or("gemini");
            let default_model = model.unwrap_or("gemini-2.0-flash-exp");
            
            config_file.persona = Some(PersonaConfigToml {
                models: Some(PersonaModelsToml {
                    primary: format!("{}:{}", default_engine, default_model),
                    fallback: Some(format!("{}:gemini-2.0-flash-thinking", default_engine)),
                    premium: Some(format!("{}:gemini-1.5-pro", default_engine)),
                }),
                performance: Some(PersonaPerformanceToml {
                    profile: "balanced".to_string(),
                    estimated_tokens: Some(1500),
                }),
            });
        }
        
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
    let _template_paths = vec![
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

/// Show agent usage statistics.
async fn show_agent_stats(json_output: bool) -> anyhow::Result<()> {
    use rusqlite::Connection;
    use std::path::PathBuf;

    // Get monitoring database path (default location)
    let db_path = PathBuf::from(".radium/monitoring.db");
    let conn = if db_path.exists() {
        Connection::open(&db_path)?
    } else {
        // Use in-memory if no database exists
        let conn = Connection::open_in_memory()?;
        radium_core::monitoring::initialize_schema(&conn)?;
        conn
    };

    let analytics = AgentAnalyticsService::new(conn);
    let stats = analytics.get_overall_stats()?;

    if json_output {
        let output = json!({
            "total_agents": stats.total_agents,
            "total_executions": stats.total_executions,
            "total_duration_ms": stats.total_duration_ms,
            "avg_duration_ms": stats.avg_duration_ms(),
            "total_tokens": stats.total_tokens,
            "total_successes": stats.total_successes,
            "total_failures": stats.total_failures,
            "success_rate": stats.success_rate(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!();
        println!("{}", "📊 Agent Usage Statistics".bold().cyan());
        println!();
        println!("  Total Agents:        {}", stats.total_agents.to_string().green());
        println!("  Total Executions:    {}", stats.total_executions.to_string().green());
        println!("  Total Duration:      {} ms", stats.total_duration_ms.to_string().cyan());
        println!("  Avg Duration:        {:.2} ms", stats.avg_duration_ms().to_string().cyan());
        println!("  Total Tokens:        {}", stats.total_tokens.to_string().yellow());
        println!("  Successes:           {}", stats.total_successes.to_string().green());
        println!("  Failures:            {}", stats.total_failures.to_string().red());
        println!("  Success Rate:        {:.1}%", stats.success_rate() * 100.0);
        println!();
    }

    Ok(())
}

/// Show most popular agents.
async fn show_popular_agents(limit: usize, json_output: bool) -> anyhow::Result<()> {
    use rusqlite::Connection;
    use std::path::PathBuf;

    let db_path = PathBuf::from(".radium/monitoring.db");
    let conn = if db_path.exists() {
        Connection::open(&db_path)?
    } else {
        let conn = Connection::open_in_memory()?;
        radium_core::monitoring::initialize_schema(&conn)?;
        conn
    };

    let analytics = AgentAnalyticsService::new(conn);
    let popular = analytics.get_popular_agents(limit)?;

    if popular.is_empty() {
        if !json_output {
            println!("{}", "No agent usage data available.".yellow());
        }
        return Ok(());
    }

    if json_output {
        let output: Vec<_> = popular
            .iter()
            .map(|a| {
                json!({
                    "agent_id": a.agent_id,
                    "execution_count": a.execution_count,
                    "avg_duration_ms": a.avg_duration_ms,
                    "total_tokens": a.total_tokens,
                    "success_rate": a.success_rate,
                    "category": a.category,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!();
        println!(
            "{}",
            format!("⭐ Most Popular Agents (Top {})", limit).bold().cyan()
        );
        println!();

        #[derive(Tabled)]
        struct PopularRow {
            #[tabled(rename = "Agent ID")]
            agent_id: String,
            #[tabled(rename = "Executions")]
            executions: String,
            #[tabled(rename = "Avg Duration")]
            avg_duration: String,
            #[tabled(rename = "Success Rate")]
            success_rate: String,
            #[tabled(rename = "Category")]
            category: String,
        }

        let rows: Vec<PopularRow> = popular
            .iter()
            .map(|a| PopularRow {
                agent_id: a.agent_id.clone(),
                executions: a.execution_count.to_string(),
                avg_duration: format!("{:.0} ms", a.avg_duration_ms),
                success_rate: format!("{:.1}%", a.success_rate * 100.0),
                category: a.category.as_ref().unwrap_or(&"-".to_string()).clone(),
            })
            .collect();

        let table = Table::new(rows).with(Style::rounded()).to_string();
        println!("{}", table);
        println!();
    }

    Ok(())
}

/// Show agent performance metrics.
async fn show_performance_metrics(limit: usize, json_output: bool) -> anyhow::Result<()> {
    use rusqlite::Connection;
    use std::path::PathBuf;

    let db_path = PathBuf::from(".radium/monitoring.db");
    let conn = if db_path.exists() {
        Connection::open(&db_path)?
    } else {
        let conn = Connection::open_in_memory()?;
        radium_core::monitoring::initialize_schema(&conn)?;
        conn
    };

    let analytics = AgentAnalyticsService::new(conn);
    let metrics = analytics.get_performance_metrics(limit)?;

    if metrics.is_empty() {
        if !json_output {
            println!("{}", "No agent performance data available.".yellow());
        }
        return Ok(());
    }

    if json_output {
        let output: Vec<_> = metrics
            .iter()
            .map(|a| {
                json!({
                    "agent_id": a.agent_id,
                    "execution_count": a.execution_count,
                    "avg_duration_ms": a.avg_duration_ms,
                    "total_duration_ms": a.total_duration_ms,
                    "total_tokens": a.total_tokens,
                    "success_rate": a.success_rate,
                    "category": a.category,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!();
        println!(
            "{}",
            format!("⚡ Agent Performance Metrics (Slowest {} by avg duration)", limit)
                .bold()
                .cyan()
        );
        println!();

        #[derive(Tabled)]
        struct PerformanceRow {
            #[tabled(rename = "Agent ID")]
            agent_id: String,
            #[tabled(rename = "Executions")]
            executions: String,
            #[tabled(rename = "Avg Duration")]
            avg_duration: String,
            #[tabled(rename = "Total Duration")]
            total_duration: String,
            #[tabled(rename = "Total Tokens")]
            total_tokens: String,
            #[tabled(rename = "Success Rate")]
            success_rate: String,
        }

        let rows: Vec<PerformanceRow> = metrics
            .iter()
            .map(|a| PerformanceRow {
                agent_id: a.agent_id.clone(),
                executions: a.execution_count.to_string(),
                avg_duration: format!("{:.0} ms", a.avg_duration_ms),
                total_duration: format!("{} ms", a.total_duration_ms),
                total_tokens: a.total_tokens.to_string(),
                success_rate: format!("{:.1}%", a.success_rate * 100.0),
            })
            .collect();

        let table = Table::new(rows).with(Style::rounded()).to_string();
        println!("{}", table);
        println!();
    }

    Ok(())
}

/// Execute analytics command with filtering.
async fn execute_analytics(
    agent_id: Option<&str>,
    all: bool,
    category: Option<&str>,
    from_date: Option<&str>,
    to_date: Option<&str>,
    json_output: bool,
) -> anyhow::Result<()> {
    use chrono::NaiveDate;
    use rusqlite::Connection;
    use std::path::PathBuf;
    use radium_core::agents::persona::ModelPricingDB;

    // Get monitoring database
    let db_path = PathBuf::from(".radium/monitoring.db");
    let conn = if db_path.exists() {
        Connection::open(&db_path)?
    } else {
        let conn = Connection::open_in_memory()?;
        radium_core::monitoring::initialize_schema(&conn)?;
        conn
    };

    let analytics_service = AgentAnalyticsService::new(conn);
    let discovery = AgentDiscovery::new();
    let pricing_db = ModelPricingDB::new();

    // Parse date filters
    let from_timestamp = from_date.and_then(|d| {
        NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .ok()
            .map(|date| date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
    });
    let to_timestamp = to_date.and_then(|d| {
        NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .ok()
            .map(|date| date.and_hms_opt(23, 59, 59).unwrap().and_utc().timestamp())
    });

    // Get analytics data based on filters
    let mut analytics_data = if let Some(id) = agent_id {
        // Single agent
        analytics_service
            .get_agent_analytics(id)?
            .map(|a| vec![a])
            .unwrap_or_default()
    } else if let Some(cat) = category {
        // By category
        analytics_service.get_analytics_by_category(cat)?
    } else if all {
        // All agents
        analytics_service.get_all_analytics()?
    } else {
        // Default: show all if no specific filter
        analytics_service.get_all_analytics()?
    };

    // Filter by date range if specified
    if from_timestamp.is_some() || to_timestamp.is_some() {
        analytics_data.retain(|a| {
            if let Some(last_used) = a.last_used_at {
                let timestamp = last_used.timestamp();
                let from_ok = from_timestamp.map_or(true, |from| timestamp >= from);
                let to_ok = to_timestamp.map_or(true, |to| timestamp <= to);
                from_ok && to_ok
            } else {
                false // Exclude agents without usage timestamps
            }
        });
    }

    if analytics_data.is_empty() {
        if !json_output {
            println!("{}", "No analytics data available for the specified filters.".yellow());
        }
        return Ok(());
    }

    // Calculate costs for each agent
    let mut rows_with_costs = Vec::new();
    for analytics in &analytics_data {
        let mut estimated_cost = 0.0;
        
        // Try to get agent config to estimate cost
        if let Ok(Some(agent)) = discovery.find_by_id(&analytics.agent_id) {
            if let Some(persona) = &agent.persona_config {
                // Estimate cost using primary model
                let estimated_tokens = persona.performance.estimated_tokens.unwrap_or(2000);
                let input_tokens = (estimated_tokens as f64 * 0.7) as u64;
                let output_tokens = (estimated_tokens as f64 * 0.3) as u64;
                
                // Calculate cost per execution, then multiply by execution count
                let cost_per_exec = pricing_db
                    .get_pricing(&persona.models.primary.model)
                    .estimate_cost(input_tokens, output_tokens);
                estimated_cost = cost_per_exec * analytics.execution_count as f64;
            }
        }

        rows_with_costs.push((analytics.clone(), estimated_cost));
    }

    if json_output {
        let output: Vec<_> = rows_with_costs
            .iter()
            .map(|(a, cost)| {
                json!({
                    "agent_id": a.agent_id,
                    "execution_count": a.execution_count,
                    "avg_duration_ms": a.avg_duration_ms,
                    "total_duration_ms": a.total_duration_ms,
                    "total_tokens": a.total_tokens,
                    "estimated_cost": cost,
                    "success_rate": a.success_rate,
                    "category": a.category,
                    "last_used_at": a.last_used_at.map(|d| d.to_rfc3339()),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!();
        let title = if let Some(id) = agent_id {
            format!("📊 Analytics: {}", id)
        } else if let Some(cat) = category {
            format!("📊 Analytics: {} category", cat)
        } else {
            "📊 Agent Analytics".to_string()
        };
        println!("{}", title.bold().cyan());
        println!();

        #[derive(Tabled)]
        struct AnalyticsRow {
            #[tabled(rename = "Agent ID")]
            agent_id: String,
            #[tabled(rename = "Executions")]
            executions: String,
            #[tabled(rename = "Avg Duration (ms)")]
            avg_duration: String,
            #[tabled(rename = "Total Tokens")]
            total_tokens: String,
            #[tabled(rename = "Est. Cost ($)")]
            cost: String,
            #[tabled(rename = "Success Rate (%)")]
            success_rate: String,
        }

        let rows: Vec<AnalyticsRow> = rows_with_costs
            .iter()
            .map(|(a, cost)| AnalyticsRow {
                agent_id: a.agent_id.clone(),
                executions: a.execution_count.to_string(),
                avg_duration: format!("{:.0}", a.avg_duration_ms),
                total_tokens: a.total_tokens.to_string(),
                cost: format!("{:.4}", cost),
                success_rate: format!("{:.1}", a.success_rate * 100.0),
            })
            .collect();

        let table = Table::new(rows).with(Style::rounded()).to_string();
        println!("{}", table);

        // Add summary row if showing all agents
        if all || (agent_id.is_none() && category.is_none()) {
            let total_executions: u64 = analytics_data.iter().map(|a| a.execution_count).sum();
            let total_tokens: u64 = analytics_data.iter().map(|a| a.total_tokens).sum();
            let total_cost: f64 = rows_with_costs.iter().map(|(_, c)| c).sum();
            let avg_success_rate = if !analytics_data.is_empty() {
                analytics_data.iter().map(|a| a.success_rate).sum::<f64>() / analytics_data.len() as f64
            } else {
                0.0
            };

            println!();
            println!("{}", "Summary:".bold());
            println!("  Total Executions:    {}", total_executions.to_string().green());
            println!("  Total Tokens:        {}", total_tokens.to_string().yellow());
            println!("  Estimated Cost:      ${:.4}", total_cost);
            println!("  Avg Success Rate:    {:.1}%", avg_success_rate * 100.0);
        }

        println!();
    }

    Ok(())
}

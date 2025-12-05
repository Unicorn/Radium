//! Templates command implementation.
//!
//! Provides commands for discovering, managing, and using workflow templates.

use super::TemplatesCommand;
use colored::Colorize;
use radium_core::workflow::{TemplateDiscovery, WorkflowTemplate};
use serde_json::json;
use tabled::{Table, Tabled, settings::Style};

/// Execute the templates command.
pub async fn execute(command: TemplatesCommand) -> anyhow::Result<()> {
    match command {
        TemplatesCommand::List { json, verbose } => list_templates(json, verbose).await,
        TemplatesCommand::Info { name, json } => show_template_info(&name, json).await,
        TemplatesCommand::Validate { verbose } => validate_templates(verbose).await,
    }
}

/// List all available templates.
async fn list_templates(json_output: bool, verbose: bool) -> anyhow::Result<()> {
    let discovery = TemplateDiscovery::new();
    let templates = discovery.discover_all()?;

    if templates.is_empty() {
        if !json_output {
            println!("{}", "No templates found.".yellow());
            println!();
            println!("Try creating templates in:");
            println!("  • ./templates/ (project-local)");
            println!("  • ~/.radium/templates/ (user-level)");
        }
        return Ok(());
    }

    if json_output {
        let template_list: Vec<_> = templates
            .values()
            .map(|template| {
                json!({
                    "name": template.name,
                    "description": template.description,
                    "steps": template.steps.len(),
                    "sub_agents": template.sub_agent_ids.len(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&template_list)?);
    } else {
        println!();
        println!("{}", format!("📋 Found {} templates", templates.len()).bold().green());
        println!();

        if verbose {
            display_templates_detailed(&templates);
        } else {
            display_templates_table(&templates);
        }
    }

    Ok(())
}

/// Show detailed information about a specific template.
async fn show_template_info(name: &str, json_output: bool) -> anyhow::Result<()> {
    let discovery = TemplateDiscovery::new();
    let template = discovery
        .find_by_name(name)?
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", name))?;

    if json_output {
        let info = json!({
            "name": template.name,
            "description": template.description,
            "steps": template.steps.len(),
            "sub_agent_ids": template.sub_agent_ids,
            "step_details": template.steps.iter().map(|step| {
                json!({
                    "agent_id": step.config.agent_id,
                    "agent_name": step.config.agent_name,
                    "type": format!("{:?}", step.config.step_type),
                    "execute_once": step.config.execute_once,
                })
            }).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!();
        println!("{}", format!("📋 Template: {}", template.name).bold().cyan());
        println!();

        if let Some(desc) = &template.description {
            println!("{}", "Description:".bold());
            println!("  {}", desc);
            println!();
        }

        println!("{}", "Statistics:".bold());
        println!("  Steps:      {}", template.steps.len());
        println!("  Sub-agents: {}", template.sub_agent_ids.len());
        println!();

        if !template.sub_agent_ids.is_empty() {
            println!("{}", "Sub-agents:".bold());
            for agent_id in &template.sub_agent_ids {
                println!("  • {}", agent_id.green());
            }
            println!();
        }

        println!("{}", "Steps:".bold());
        for (i, step) in template.steps.iter().enumerate() {
            println!("  {}. {}", i + 1, step.config.agent_id.cyan());
            if let Some(name) = &step.config.agent_name {
                println!("     Name: {}", name);
            }
            println!("     Type: {:?}", step.config.step_type);
            if step.config.execute_once {
                println!("     Execute once: {}", "true".yellow());
            }
        }
        println!();
    }

    Ok(())
}

/// Validate all template configurations.
async fn validate_templates(verbose: bool) -> anyhow::Result<()> {
    let discovery = TemplateDiscovery::new();
    let templates = discovery.discover_all()?;

    let mut valid_count = 0;
    let mut errors = Vec::new();

    for (name, template) in &templates {
        let mut template_errors = Vec::new();

        if template.name.is_empty() {
            template_errors.push("Name is empty");
        }

        if template.steps.is_empty() {
            template_errors.push("No steps defined");
        }

        // Check for duplicate step agent IDs
        let mut agent_ids = std::collections::HashSet::new();
        for step in &template.steps {
            if !agent_ids.insert(&step.config.agent_id) {
                template_errors.push("Duplicate agent IDs in steps");
                break;
            }
        }

        if template_errors.is_empty() {
            valid_count += 1;
        } else {
            errors.push((name.clone(), template_errors));
        }
    }

    println!();
    if errors.is_empty() {
        println!(
            "{}",
            format!("✅ All {} templates validated successfully", templates.len()).bold().green()
        );
    } else {
        println!(
            "{}",
            format!("⚠️  Validation: {} valid, {} with errors", valid_count, errors.len())
                .bold()
                .yellow()
        );
        println!();

        if verbose {
            for (name, template_errors) in &errors {
                println!("{}", format!("  {} {}:", "❌".red(), name.red()));
                for error in template_errors {
                    println!("     • {}", error);
                }
            }
        } else {
            println!("Run with {} for details", "--verbose".cyan());
        }
    }
    println!();

    Ok(())
}

/// Display templates in a compact table format.
fn display_templates_table(templates: &std::collections::HashMap<String, WorkflowTemplate>) {
    #[derive(Tabled)]
    struct TemplateRow {
        #[tabled(rename = "Name")]
        name: String,
        #[tabled(rename = "Description")]
        description: String,
        #[tabled(rename = "Steps")]
        steps: usize,
        #[tabled(rename = "Sub-agents")]
        sub_agents: usize,
    }

    let mut rows: Vec<TemplateRow> = templates
        .values()
        .map(|template| TemplateRow {
            name: template.name.clone(),
            description: template.description.clone().unwrap_or_else(|| "-".to_string()),
            steps: template.steps.len(),
            sub_agents: template.sub_agent_ids.len(),
        })
        .collect();

    // Sort by name
    rows.sort_by(|a, b| a.name.cmp(&b.name));

    let table = Table::new(rows).with(Style::rounded()).to_string();

    println!("{}", table);
    println!();
}

/// Display templates in detailed format.
fn display_templates_detailed(templates: &std::collections::HashMap<String, WorkflowTemplate>) {
    let mut template_list: Vec<_> = templates.values().collect();
    template_list.sort_by_key(|t| t.name.as_str());

    for template in template_list {
        println!("{}", format!("  {} {}", "●".green(), template.name.bold()));
        if let Some(desc) = &template.description {
            println!("    Desc:  {}", desc);
        }
        println!("    Steps: {}", template.steps.len());
        println!("    Agents: {}", template.sub_agent_ids.len());
        println!();
    }
}

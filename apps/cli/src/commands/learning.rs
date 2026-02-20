//! Learning management commands.
//!
//! Provides CLI commands for managing the learning system, including
//! viewing mistakes, adding skills, tagging skills, and viewing the skillbook.

use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use radium_core::learning::{
    LearningStore, LearningType, MetricsAggregator, STANDARD_SECTIONS,
};
use radium_core::workspace::Workspace;
use serde_json::json;
use std::path::Path;

/// Learning subcommands
#[derive(Subcommand, Debug)]
pub enum LearningCommand {
    /// List all learning entries
    List {
        /// Filter by category
        #[arg(long)]
        category: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Add a mistake entry
    AddMistake {
        /// Category for the mistake
        #[arg(short, long)]
        category: String,

        /// Description of the mistake
        #[arg(short, long)]
        description: String,

        /// Solution or explanation
        #[arg(short, long)]
        solution: String,
    },

    /// Add a skill to the skillbook
    AddSkill {
        /// Section for the skill
        #[arg(short, long)]
        section: String,

        /// Content/description of the skill
        #[arg(short, long)]
        content: String,
    },

    /// Tag a skill as helpful, harmful, or neutral
    TagSkill {
        /// Skill ID to tag
        #[arg(short, long)]
        skill_id: String,

        /// Tag type (helpful, harmful, or neutral)
        #[arg(short, long)]
        tag: String,

        /// Increment amount (default: 1)
        #[arg(short, long, default_value = "1")]
        increment: u32,
    },

    /// Show skillbook by section
    ShowSkillbook {
        /// Section to show (optional, shows all if not specified)
        #[arg(short, long)]
        section: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// View routing quality metrics (Phase 2 - REQ-246)
    Metrics {
        /// Export to JSON file
        #[arg(long)]
        export: Option<String>,

        /// Show detailed per-agent metrics
        #[arg(long)]
        detailed: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// Execute learning command
pub async fn execute(cmd: LearningCommand) -> Result<()> {
    let workspace = Workspace::discover()
        .context("No Radium workspace found. Run 'rad init' to create one.")?;

    let mut learning_store = LearningStore::new(workspace.root())
        .context("Failed to initialize learning store")?;

    match cmd {
        LearningCommand::List { category, json } => {
            list_command(&learning_store, category, json).await
        }
        LearningCommand::AddMistake { category, description, solution } => {
            add_mistake_command(&mut learning_store, category, description, solution).await
        }
        LearningCommand::AddSkill { section, content } => {
            add_skill_command(&mut learning_store, section, content).await
        }
        LearningCommand::TagSkill { skill_id, tag, increment } => {
            tag_skill_command(&mut learning_store, skill_id, tag, increment).await
        }
        LearningCommand::ShowSkillbook { section, json } => {
            show_skillbook_command(&learning_store, section, json).await
        }
        LearningCommand::Metrics { export, detailed, json } => {
            metrics_command(&workspace, export, detailed, json).await
        }
    }
}

async fn list_command(learning_store: &LearningStore, category: Option<String>, json: bool) -> Result<()> {
    if json {
        let all_entries = learning_store.get_all_entries();
        let entries: Vec<_> = if let Some(cat) = category {
            all_entries.get(&cat).cloned().unwrap_or_default()
        } else {
            all_entries.values().flat_map(|v| v.iter().cloned()).collect()
        };

        let json_output = json!({
            "entries": entries.iter().map(|e| json!({
                "type": format!("{:?}", e.entry_type),
                "category": e.category,
                "description": e.description,
                "solution": e.solution,
                "timestamp": e.timestamp.to_rfc3339(),
            })).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        println!("{}", "Learning Entries".bold().cyan());
        println!();

        let all_entries = learning_store.get_all_entries();
        let entries_to_show = if let Some(cat) = category {
            println!("  Category: {}\n", cat.cyan());
            all_entries.get(&cat).cloned().unwrap_or_default()
        } else {
            all_entries.values().flat_map(|v| v.iter().cloned()).collect()
        };

        if entries_to_show.is_empty() {
            println!("  {}", "No learning entries found.".dimmed());
            return Ok(());
        }

        for entry in entries_to_show {
            let type_label = match entry.entry_type {
                LearningType::Mistake => "Mistake".red(),
                LearningType::Preference => "Preference".yellow(),
                LearningType::Success => "Success".green(),
            };

            println!("  {} [{}] {}", "•".cyan(), type_label, entry.category.cyan());
            println!("    {}", entry.description);
            if let Some(solution) = entry.solution {
                println!("    {} Solution: {}", "→".dimmed(), solution.dimmed());
            }
            println!("    {} {}", "Timestamp:".dimmed(), entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
            println!();
        }
    }

    Ok(())
}

async fn add_mistake_command(
    learning_store: &mut LearningStore,
    category: String,
    description: String,
    solution: String,
) -> Result<()> {
    let (entry, added) = learning_store
        .add_entry(LearningType::Mistake, category.clone(), description.clone(), Some(solution.clone()))
        .context("Failed to add mistake entry")?;

    if added {
        println!("{}", "Mistake added successfully".green().bold());
        println!("  Category: {}", entry.category.cyan());
        println!("  Description: {}", entry.description);
        println!("  Solution: {}", entry.solution.as_ref().unwrap());
    } else {
        println!("{}", "Mistake not added (duplicate detected)".yellow().bold());
    }

    Ok(())
}

async fn add_skill_command(
    learning_store: &mut LearningStore,
    section: String,
    content: String,
) -> Result<()> {
    // Validate section
    if !STANDARD_SECTIONS.contains(&section.as_str()) {
        println!("{}", format!("Warning: '{}' is not a standard section. Standard sections: {}", section, STANDARD_SECTIONS.join(", ")).yellow());
    }

    let skill = learning_store
        .add_skill(section.clone(), content.clone(), None)
        .context("Failed to add skill")?;

    println!("{}", "Skill added successfully".green().bold());
    println!("  ID: {}", skill.id.cyan());
    println!("  Section: {}", skill.section.cyan());
    println!("  Content: {}", skill.content);

    Ok(())
}

async fn tag_skill_command(
    learning_store: &mut LearningStore,
    skill_id: String,
    tag: String,
    increment: u32,
) -> Result<()> {
    // Validate tag
    if !matches!(tag.as_str(), "helpful" | "harmful" | "neutral") {
        anyhow::bail!("Tag must be 'helpful', 'harmful', or 'neutral'");
    }

    learning_store
        .tag_skill(&skill_id, &tag, increment)
        .context("Failed to tag skill")?;

    println!("{}", format!("Skill {} tagged as {} (increment: {})", skill_id, tag, increment).green().bold());

    Ok(())
}

async fn show_skillbook_command(
    learning_store: &LearningStore,
    section: Option<String>,
    json: bool,
) -> Result<()> {
    if json {
        let sections_to_show = if let Some(sect) = section {
            vec![sect]
        } else {
            STANDARD_SECTIONS.iter().map(|s| s.to_string()).collect()
        };

        let mut skillbook_json = json!({});
        for sect in sections_to_show {
            let skills = learning_store.get_skills_by_section(&sect, false);
            skillbook_json[sect] = json!(skills.iter().map(|s| json!({
                "id": s.id,
                "section": s.section,
                "content": s.content,
                "helpful": s.helpful,
                "harmful": s.harmful,
                "neutral": s.neutral,
            })).collect::<Vec<_>>());
        }
        println!("{}", serde_json::to_string_pretty(&skillbook_json)?);
    } else {
        println!("{}", "Skillbook".bold().cyan());
        println!();

        let sections_to_show = if let Some(sect) = section {
            vec![sect]
        } else {
            STANDARD_SECTIONS.iter().map(|s| s.to_string()).collect()
        };

        for sect in sections_to_show {
            let skills = learning_store.get_skills_by_section(&sect, false);
            if !skills.is_empty() {
                println!("  {}", sect.bold());
                for skill in skills {
                    println!("    {} [{}]", "•".cyan(), skill.id.dimmed());
                    println!("      {}", skill.content);
                    println!("      {} Helpful: {} | Harmful: {} | Neutral: {}", 
                        "Stats:".dimmed(), 
                        skill.helpful.to_string().green(),
                        skill.harmful.to_string().red(),
                        skill.neutral.to_string().yellow()
                    );
                    println!();
                }
            }
        }
    }

    Ok(())
}


async fn metrics_command(
    workspace: &Workspace,
    export: Option<String>,
    detailed: bool,
    json_output: bool,
) -> Result<()> {
    // Load metrics from workspace
    let metrics = MetricsAggregator::aggregate_from_workspace(workspace.root())
        .map_err(anyhow::Error::msg)?;

    // Export to JSON file if requested
    if let Some(export_path) = export {
        let path = Path::new(&export_path);
        MetricsAggregator::export_to_json(&metrics, path)
            .map_err(anyhow::Error::msg)?;
        println!("{}", format!("Metrics exported to {}", export_path).green().bold());
        return Ok(());
    }

    // JSON output to stdout
    if json_output {
        let json_metrics = serde_json::to_string_pretty(&metrics)?;
        println!("{}", json_metrics);
        return Ok(());
    }

    // Human-readable output
    println!("{}", "═══════════════════════════════════════".cyan().bold());
    println!("{}", "      Routing Quality Metrics".cyan().bold());
    println!("{}", "═══════════════════════════════════════".cyan().bold());
    println!();

    if metrics.total_routings == 0 {
        println!("{}", "No routing decisions recorded yet.".dimmed());
        println!();
        println!("Start executing tasks with routing enabled to collect metrics.");
        return Ok(());
    }

    // Overall metrics
    println!("{}", "Overall Performance".bold());
    println!("  Total Routings:    {}", metrics.total_routings.to_string().cyan());
    println!("  Accuracy:          {}", format!("{:.1}%", metrics.accuracy * 100.0).green());
    println!("  Error Rate:        {}", format!("{:.1}%", metrics.error_rate * 100.0).red());
    println!("  Retry Rate:        {}", format!("{:.1}%", metrics.retry_rate * 100.0).yellow());
    println!("  Escalation Rate:   {}", format!("{:.1}%", metrics.escalation_rate * 100.0).yellow());
    println!();

    // Confidence statistics
    println!("{}", "Confidence Statistics".bold());
    println!("  Mean:      {}", format!("{:.3}", metrics.confidence_stats.mean).cyan());
    println!("  Min:       {}", format!("{:.3}", metrics.confidence_stats.min).dimmed());
    println!("  Max:       {}", format!("{:.3}", metrics.confidence_stats.max).dimmed());
    println!("  Std Dev:   {}", format!("{:.3}", metrics.confidence_stats.std_dev).dimmed());
    println!();

    // Accuracy drift warning
    if MetricsAggregator::detect_accuracy_drift(&metrics, 0.8) {
        println!("{}", "⚠️  WARNING: Routing accuracy below 80% threshold!".yellow().bold());
        println!("   {} Consider reviewing agent configuration or skill definitions.", "→".yellow());
        println!();
    }

    // Top agents by selection count
    println!("{}", "Top Agents (by selection count)".bold());
    let mut agents: Vec<_> = metrics.agent_performance.iter().collect();
    agents.sort_by(|a, b| b.1.total_selections.cmp(&a.1.total_selections));

    let top_count = if detailed { agents.len() } else { 5.min(agents.len()) };

    for (agent_id, agent_metrics) in agents.iter().take(top_count) {
        println!("  {}", agent_id.cyan().bold());
        println!("    Selections:        {}", agent_metrics.total_selections);
        println!("    Success Rate:      {}", format!("{:.1}%", agent_metrics.success_rate * 100.0).green());
        println!("    Avg Confidence:    {}", format!("{:.3}", agent_metrics.avg_confidence).dimmed());

        // Feedback breakdown
        let total_feedback = agent_metrics.positive_feedback + agent_metrics.negative_feedback + agent_metrics.neutral_feedback;
        if total_feedback > 0 {
            println!("    Feedback:          {} positive, {} negative, {} neutral",
                agent_metrics.positive_feedback.to_string().green(),
                agent_metrics.negative_feedback.to_string().red(),
                agent_metrics.neutral_feedback.to_string().yellow()
            );
        }
        println!();
    }

    if !detailed && agents.len() > 5 {
        println!("  {} Use --detailed to see all {} agents",
            "...".dimmed(),
            agents.len()
        );
        println!();
    }

    // Footer with recommendations
    println!("{}", "═══════════════════════════════════════".cyan().dimmed());
    println!();
    println!("{}", "Recommendations:".bold());

    if metrics.accuracy < 0.9 {
        println!("  • {} Review agent skill definitions for better matching", "→".cyan());
    }
    if metrics.error_rate > 0.1 {
        println!("  • {} Investigate failed executions and add fallback strategies", "→".cyan());
    }
    if metrics.retry_rate > 0.05 {
        println!("  • {} High retry rate suggests routing issues - check task descriptions", "→".cyan());
    }

    println!();
    println!("{}", "Export options:".dimmed());
    println!("  rad learning metrics --export metrics.json  {}", "# Export to JSON".dimmed());
    println!("  rad learning metrics --json                 {}", "# JSON to stdout".dimmed());
    println!("  rad learning metrics --detailed             {}", "# Show all agents".dimmed());

    Ok(())
}

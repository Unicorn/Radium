//! Cost reporting command implementation.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Subcommand;
use colored::Colorize;
use radium_core::{
    analytics::{CostQueryService, ExportOptions, ExportFormat},
    monitoring::MonitoringService,
    Workspace,
};
use std::path::PathBuf;

/// Cost reporting command.
#[derive(Subcommand, Debug)]
pub enum CostCommand {
    /// Generate cost report with tier breakdown
    Report {
        /// Plan/requirement ID to filter by (e.g., REQ-123)
        #[arg(long)]
        plan: Option<String>,

        /// Workflow ID to filter by
        #[arg(long)]
        workflow: Option<String>,

        /// Start date (ISO 8601 format, e.g., 2024-01-01T00:00:00Z)
        #[arg(long)]
        start: Option<String>,

        /// End date (ISO 8601 format, e.g., 2024-01-31T23:59:59Z)
        #[arg(long)]
        end: Option<String>,

        /// Output format (csv, json, markdown)
        #[arg(long, default_value = "markdown")]
        format: String,

        /// Output file path (optional, defaults to stdout)
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

/// Execute cost command.
pub async fn execute(cmd: CostCommand) -> Result<()> {
    match cmd {
        CostCommand::Report {
            plan,
            workflow,
            start,
            end,
            format,
            output,
        } => {
            execute_report(plan, workflow, start, end, format, output).await
        }
    }
}

/// Execute cost report command.
async fn execute_report(
    plan: Option<String>,
    workflow: Option<String>,
    start: Option<String>,
    end: Option<String>,
    format: String,
    output: Option<PathBuf>,
) -> Result<()> {
    println!("{}", "rad cost report".bold().cyan());
    println!();

    // Discover workspace
    let workspace = Workspace::discover().context("Failed to discover workspace")?;

    // Open monitoring service
    let monitoring_path = workspace.radium_dir().join("monitoring.db");
    let monitoring = MonitoringService::open(&monitoring_path)
        .context("Failed to open monitoring database")?;

    let query_service = CostQueryService::new(&monitoring);

    // Parse dates
    let start_date = start
        .as_deref()
        .map(|s| {
            DateTime::parse_from_rfc3339(s)
                .or_else(|_| DateTime::parse_from_str(s, "%Y-%m-%d"))
                .map(|dt| dt.with_timezone(&Utc))
        })
        .transpose()
        .context("Failed to parse start date")?;

    let end_date = end
        .as_deref()
        .map(|s| {
            DateTime::parse_from_rfc3339(s)
                .or_else(|_| DateTime::parse_from_str(s, "%Y-%m-%d"))
                .map(|dt| dt.with_timezone(&Utc))
        })
        .transpose()
        .context("Failed to parse end date")?;

    // Build export options
    let export_format = ExportFormat::from_str(&format)
        .map_err(|e| anyhow::anyhow!("Invalid format '{}': {}", format, e))?;

    let options = ExportOptions {
        format: export_format,
        start_date,
        end_date,
        plan_id: plan,
        provider: None,
        output_path: output.clone(),
    };

    // Query records
    println!("  {}", "Querying cost data...".dimmed());
    let records = query_service
        .query_records(&options)
        .context("Failed to query cost records")?;

    println!("  {} Found {} records", "✓".green(), records.len());
    println!();

    // Generate summary
    let summary = query_service.generate_summary(&records);

    // Format and output report
    let report = format_cost_report(&summary, &records)?;

    if let Some(ref output_path) = output {
        std::fs::write(output_path, &report)
            .context(format!("Failed to write report to {}", output_path.display()))?;
        println!("  {} Report written to {}", "✓".green(), output_path.display());
    } else {
        println!("{}", report);
    }

    Ok(())
}

/// Format cost report with tier breakdown.
fn format_cost_report(
    summary: &radium_core::analytics::CostSummary,
    _records: &[radium_core::analytics::CostRecord],
) -> Result<String> {
    let mut report = String::new();

    // Header
    report.push_str(&format!(
        "# Cost Report\n\n**Period:** {} - {}\n\n",
        summary.period.0.format("%Y-%m-%d %H:%M:%S UTC"),
        summary.period.1.format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // Summary
    report.push_str("## Summary\n\n");
    report.push_str(&format!("- **Total Cost:** ${:.4}\n", summary.total_cost));
    report.push_str(&format!("- **Total Tokens:** {}\n", summary.total_tokens));
    report.push_str("\n");

    // Tier Breakdown
    if let Some(ref tier_breakdown) = summary.tier_breakdown {
        report.push_str("## Tier Breakdown\n\n");
        report.push_str("| Tier | Requests | Input Tokens | Output Tokens | Cost |\n");
        report.push_str("|------|----------|--------------|---------------|------|\n");
        report.push_str(&format!(
            "| Smart | {} | {} | {} | ${:.4} |\n",
            tier_breakdown.smart_tier.request_count,
            tier_breakdown.smart_tier.input_tokens,
            tier_breakdown.smart_tier.output_tokens,
            tier_breakdown.smart_tier.cost
        ));
        report.push_str(&format!(
            "| Eco | {} | {} | {} | ${:.4} |\n",
            tier_breakdown.eco_tier.request_count,
            tier_breakdown.eco_tier.input_tokens,
            tier_breakdown.eco_tier.output_tokens,
            tier_breakdown.eco_tier.cost
        ));
        report.push_str("\n");
        report.push_str(&format!(
            "**Estimated Savings:** ${:.4}\n",
            tier_breakdown.estimated_savings
        ));
        report.push_str("\n");
    } else {
        report.push_str("## Tier Breakdown\n\n");
        report.push_str("*No tier data available. Routing was not used for these records.*\n\n");
    }

    // Provider breakdown
    if !summary.breakdown_by_provider.is_empty() {
        report.push_str("## Breakdown by Provider\n\n");
        report.push_str("| Provider | Cost |\n");
        report.push_str("|----------|------|\n");
        let mut providers: Vec<_> = summary.breakdown_by_provider.iter().collect();
        providers.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (provider, cost) in providers {
            report.push_str(&format!("| {} | ${:.4} |\n", provider, cost));
        }
        report.push_str("\n");
    }

    // Top plans
    if !summary.top_plans.is_empty() {
        report.push_str("## Top Plans by Cost\n\n");
        report.push_str("| Plan | Cost |\n");
        report.push_str("|------|------|\n");
        for (plan_id, cost) in &summary.top_plans {
            report.push_str(&format!("| {} | ${:.4} |\n", plan_id, cost));
        }
        report.push_str("\n");
    }

    Ok(report)
}


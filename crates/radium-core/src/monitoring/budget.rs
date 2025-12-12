//! Budget management for tracking and enforcing AI model costs.
//!
//! This module provides budget tracking, pre-execution cost checks, and budget warnings
//! to prevent cost overruns during agent execution.

#[cfg(feature = "monitoring")]
use crate::analytics::budget::{CostAnomaly, ForecastResult};
use crate::monitoring::{MonitoringService, Result as MonitoringResult};
use chrono::{DateTime, Utc};
use rusqlite::params;
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
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
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
    /// Velocity spike detected (spending rate increased significantly).
    VelocitySpike {
        current_rate: f64,
        previous_rate: f64,
        increase_pct: f64,
    },
    /// Budget will be exhausted soon based on forecast.
    ProjectedExhaustion {
        days_remaining: u32,
        exhaustion_date: DateTime<Utc>,
    },
    /// Pre-requirement warning (single requirement will consume significant portion).
    PreRequirementWarning {
        requirement_id: String,
        estimated_cost: f64,
        percentage_of_remaining: f64,
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
            BudgetError::VelocitySpike { current_rate, previous_rate, increase_pct } => {
                write!(
                    f,
                    "Velocity spike: Spending rate increased {:.1}% (${:.2}/day vs ${:.2}/day)",
                    increase_pct, current_rate, previous_rate
                )
            }
            BudgetError::ProjectedExhaustion { days_remaining, exhaustion_date } => {
                write!(
                    f,
                    "Projected exhaustion: Budget will run out in {} days (on {})",
                    days_remaining,
                    exhaustion_date.format("%Y-%m-%d")
                )
            }
            BudgetError::PreRequirementWarning { requirement_id, estimated_cost, percentage_of_remaining } => {
                write!(
                    f,
                    "Pre-requirement warning: {} will consume ${:.2} ({:.1}% of remaining budget)",
                    requirement_id, estimated_cost, percentage_of_remaining
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

/// Daily spend data point for trend analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySpend {
    /// Date in YYYY-MM-DD format.
    pub date: String,
    /// Amount spent on this day.
    pub amount: f64,
}

/// Budget warning configuration with configurable thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetWarningConfig {
    /// Velocity spike threshold (e.g., 1.5 = 50% increase).
    pub velocity_spike_threshold: f64,
    /// Days remaining threshold for exhaustion warning.
    pub projected_days_warning: u32,
    /// Percentage of remaining budget threshold for pre-requirement warning.
    pub requirement_percentage_warning: f64,
    /// Standard deviation threshold for anomaly detection.
    pub anomaly_std_dev_threshold: f64,
}

impl Default for BudgetWarningConfig {
    fn default() -> Self {
        Self {
            velocity_spike_threshold: 1.5, // 50% increase
            projected_days_warning: 7,
            requirement_percentage_warning: 0.20, // 20%
            anomaly_std_dev_threshold: 2.0,
        }
    }
}

/// Comprehensive budget analytics aggregating forecast, anomalies, and warnings.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BudgetAnalytics {
    /// Daily spend trend data (last 30 days).
    pub trend_data: Vec<DailySpend>,
    /// Forecast result with exhaustion projection.
    #[cfg(feature = "monitoring")]
    pub forecast: Option<ForecastResult>,
    /// Detected cost anomalies.
    #[cfg(feature = "monitoring")]
    pub anomalies: Vec<CostAnomaly>,
    /// Active budget warnings.
    pub warnings: Vec<BudgetError>,
}

/// Budget manager for tracking costs and enforcing limits.
#[derive(Clone)]
pub struct BudgetManager {
    config: BudgetConfig,
    spent_amount: Arc<Mutex<f64>>,
    /// Optional telemetry store for reading spent from database.
    telemetry_store: Option<Arc<MonitoringService>>,
    // Forecaster and anomaly detector fields commented out until analytics module is available
    // /// Optional budget forecaster for analytics.
    // forecaster: Option<Arc<BudgetForecaster>>,
    // /// Optional anomaly detector for cost analysis.
    // anomaly_detector: Option<Arc<AnomalyDetector>>,
    /// Warning configuration.
    warning_config: BudgetWarningConfig,
}

impl BudgetManager {
    /// Creates a new budget manager with the given configuration.
    #[must_use]
    pub fn new(config: BudgetConfig) -> Self {
        Self {
            config,
            spent_amount: Arc::new(Mutex::new(0.0)),
            telemetry_store: None,
            // forecaster: None,
            // anomaly_detector: None,
            warning_config: BudgetWarningConfig::default(),
        }
    }

    /// Creates a budget manager with a simple budget limit.
    #[must_use]
    pub fn with_limit(max_budget: f64) -> Self {
        Self::new(BudgetConfig::new(Some(max_budget)))
    }

    // TODO: Re-enable when analytics module is available
    // /// Creates a budget manager with analytics capabilities.
    // ///
    // /// # Arguments
    // /// * `config` - Budget configuration
    // /// * `telemetry_store` - Monitoring service for database access
    // /// * `forecaster` - Budget forecaster for projections
    // /// * `anomaly_detector` - Anomaly detector for cost analysis
    // /// * `warning_config` - Warning configuration (uses defaults if None)
    // ///
    // /// # Returns
    // /// BudgetManager with analytics enabled
    // pub fn with_analytics(
    //     config: BudgetConfig,
    //     telemetry_store: Arc<MonitoringService>,
    //     forecaster: Arc<BudgetForecaster>,
    //     anomaly_detector: Arc<AnomalyDetector>,
    //     warning_config: Option<BudgetWarningConfig>,
    // ) -> Self {
    //     Self {
    //         config,
    //         spent_amount: Arc::new(Mutex::new(0.0)),
    //         telemetry_store: Some(telemetry_store),
    //         forecaster: Some(forecaster),
    //         anomaly_detector: Some(anomaly_detector),
    //         warning_config: warning_config.unwrap_or_default(),
    //     }
    // }

    /// Checks if the estimated cost is within budget.
    ///
    /// Also checks for proactive warnings (non-blocking, logged only).
    ///
    /// # Errors
    /// Returns `BudgetError::BudgetExceeded` if the estimated cost would exceed the budget.
    /// Returns `BudgetError::BudgetWarning` if a warning threshold is reached.
    pub fn check_budget_available(&self, estimated_cost: f64) -> Result<(), BudgetError> {
        let spent = self.get_spent_from_source();

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

        // Check proactive warnings (non-blocking, log only)
        let warnings = self.check_proactive_warnings(estimated_cost, None);
        for warning in warnings {
            // Log warning but don't block execution
            tracing::warn!("Budget warning: {}", warning);
        }

        Ok(())
    }

    /// Records an actual cost after execution.
    pub fn record_cost(&self, actual_cost: f64) {
        let mut spent = self.spent_amount.lock().unwrap();
        *spent += actual_cost;
    }

    /// Gets the current spent amount, reading from telemetry if available.
    fn get_spent_from_source(&self) -> f64 {
        // Try to read from telemetry database first
        if let Some(ref store) = self.telemetry_store {
            let conn = store.conn();
            if let Ok(mut stmt) = conn.prepare("SELECT SUM(estimated_cost) FROM telemetry") {
                if let Ok(Some(spent)) = stmt.query_row([], |row| row.get(0)) {
                    return spent;
                }
            }
        }
        // Fallback to in-memory tracking
        *self.spent_amount.lock().unwrap()
    }

    /// Gets the current budget status.
    #[must_use]
    pub fn get_budget_status(&self) -> BudgetStatus {
        let spent = self.get_spent_from_source();

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
        self.get_spent_from_source()
    }

    /// Gets comprehensive budget analytics including forecast, anomalies, and warnings.
    ///
    /// # Returns
    /// BudgetAnalytics with all analytics data
    ///
    /// # Errors
    /// Returns error if analytics components are not available or queries fail
    pub fn get_analytics(&self) -> MonitoringResult<BudgetAnalytics> {
        // Get trend data (last 30 days)
        let trend_data = self.get_trend_data(30)?;

        // Get forecast if forecaster is available
        // TODO: Re-enable when analytics module is available
        #[cfg(feature = "monitoring")]
        let forecast: Option<ForecastResult> = None;
        #[cfg(not(feature = "monitoring"))]
        let forecast: Option<()> = None;
        // let forecast = if let (Some(ref forecaster), Some(limit)) = (&self.forecaster, self.config.max_budget) {
        //     let spent = self.get_spent_from_source();
        //     let remaining = (limit - spent).max(0.0);
        //     forecaster.forecast_exhaustion(remaining).ok()
        // } else {
        //     None
        // };

        // Get anomalies if detector is available
        // TODO: Re-enable when analytics module is available
        #[cfg(feature = "monitoring")]
        let anomalies = Vec::new();
        // let anomalies = if let Some(ref detector) = self.anomaly_detector {
        //     detector.detect_anomalies(30).unwrap_or_default()
        // } else {
        //     Vec::new()
        // };

        // Generate warnings
        #[cfg(feature = "monitoring")]
        let warnings = self.generate_warnings(&forecast)?;
        #[cfg(not(feature = "monitoring"))]
        let warnings = Vec::new();

        Ok(BudgetAnalytics {
            trend_data,
            #[cfg(feature = "monitoring")]
            forecast,
            #[cfg(feature = "monitoring")]
            anomalies,
            warnings,
        })
    }

    /// Gets daily spend trend data for the last N days.
    ///
    /// # Arguments
    /// * `days` - Number of days to retrieve
    ///
    /// # Returns
    /// Vector of daily spend data points
    fn get_trend_data(&self, days: u32) -> MonitoringResult<Vec<DailySpend>> {
        if let Some(ref store) = self.telemetry_store {
            let conn = store.conn();
            let now = Utc::now().timestamp() as i64;
            let start_timestamp = now - (days as i64 * 86400);

            // Try to use daily summaries first
            let mut stmt = conn.prepare(
                "SELECT date, total_cost FROM daily_spend_summary
                 WHERE date >= date('now', '-' || ?1 || ' days')
                 ORDER BY date"
            );

            if let Ok(mut stmt) = stmt {
                let summaries: Result<Vec<DailySpend>, _> = stmt
                    .query_map(params![days], |row| {
                        Ok(DailySpend {
                            date: row.get(0)?,
                            amount: row.get(1)?,
                        })
                    })?
                    .collect();
                if let Ok(summaries) = summaries {
                    if !summaries.is_empty() {
                        return Ok(summaries);
                    }
                }
            }

            // Fallback to raw telemetry aggregation
            let mut stmt = conn.prepare(
                "SELECT date(timestamp, 'unixepoch') as day, SUM(estimated_cost) as daily_cost
                 FROM telemetry
                 WHERE timestamp >= ?1 AND timestamp <= ?2
                 GROUP BY day
                 ORDER BY day"
            )?;

            let trend: Vec<DailySpend> = stmt
                .query_map(params![start_timestamp, now], |row| {
                    Ok(DailySpend {
                        date: row.get(0)?,
                        amount: row.get(1)?,
                    })
                })?
                .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()?;

            Ok(trend)
        } else {
            // No telemetry store - return empty
            Ok(Vec::new())
        }
    }

    /// Generates proactive warnings based on analytics.
    ///
    /// # Arguments
    /// * `forecast` - Optional forecast result
    ///
    /// # Returns
    /// Vector of budget warnings
    #[cfg(feature = "monitoring")]
    fn generate_warnings(&self, forecast: &Option<ForecastResult>) -> MonitoringResult<Vec<BudgetError>> {
        let mut warnings = Vec::new();

        // Check velocity spike
        // TODO: Re-enable when analytics module is available
        // if let Some(ref forecaster) = self.forecaster {
        //     let current_velocity = forecaster.calculate_spend_velocity(7)?; // Last 7 days
        //     let previous_velocity = forecaster.calculate_spend_velocity(14)? - current_velocity; // Previous 7 days
        //     let previous_7_day_velocity = if previous_velocity > 0.0 {
        //         previous_velocity
        //     } else {
        //         current_velocity * 0.5 // Fallback estimate
        //     };

        //     if previous_7_day_velocity > 0.0 {
        //         let increase_ratio = current_velocity / previous_7_day_velocity;
        //         if increase_ratio >= self.warning_config.velocity_spike_threshold {
        //             warnings.push(BudgetError::VelocitySpike {
        //                 current_rate: current_velocity,
        //                 previous_rate: previous_7_day_velocity,
        //                 increase_pct: (increase_ratio - 1.0) * 100.0,
        //             });
        //         }
        //     }
        // }

        // Check projected exhaustion
        if let Some(forecast_result) = forecast {
            if forecast_result.days_remaining <= self.warning_config.projected_days_warning {
                warnings.push(BudgetError::ProjectedExhaustion {
                    days_remaining: forecast_result.days_remaining,
                    exhaustion_date: forecast_result.exhaustion_date,
                });
            }
        }

        Ok(warnings)
    }

    /// Checks for proactive warnings before executing a requirement.
    ///
    /// # Arguments
    /// * `estimated_cost` - Estimated cost of the requirement
    /// * `requirement_id` - Optional requirement ID for context
    ///
    /// # Returns
    /// Vector of warnings (non-blocking, advisory only)
    pub fn check_proactive_warnings(
        &self,
        estimated_cost: f64,
        requirement_id: Option<&str>,
    ) -> Vec<BudgetError> {
        let mut warnings = Vec::new();

        // Check pre-requirement warning
        if let Some(limit) = self.config.max_budget {
            let spent = self.get_spent_from_source();
            let remaining = (limit - spent).max(0.0);
            if remaining > 0.0 {
                let percentage = estimated_cost / remaining;
                if percentage >= self.warning_config.requirement_percentage_warning {
                    warnings.push(BudgetError::PreRequirementWarning {
                        requirement_id: requirement_id.unwrap_or("unknown").to_string(),
                        estimated_cost,
                        percentage_of_remaining: percentage * 100.0,
                    });
                }
            }
        }

        // Get forecast-based warnings
        if let Ok(analytics) = self.get_analytics() {
            warnings.extend(analytics.warnings);
        }

        warnings
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
            if let (Some(model), Some(provider)) = (&record.model, &record.provider) {
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
        let mut max_savings: f64 = 0.0;
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


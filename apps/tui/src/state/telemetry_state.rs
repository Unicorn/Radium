//! Telemetry tracking for token usage and costs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use radium_core::monitoring::{ProviderCostBreakdown, TeamCostBreakdown};

/// Token usage metrics
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TokenMetrics {
    /// Input tokens
    pub input_tokens: u64,
    /// Output tokens
    pub output_tokens: u64,
    /// Cached tokens
    pub cached_tokens: u64,
    /// Total tokens
    pub total_tokens: u64,
}

impl TokenMetrics {
    /// Creates a new empty token metrics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds token usage to the metrics.
    pub fn add(&mut self, input: u64, output: u64, cached: u64) {
        self.input_tokens += input;
        self.output_tokens += output;
        self.cached_tokens += cached;
        self.total_tokens = self.input_tokens + self.output_tokens + self.cached_tokens;
    }

    /// Returns the total tokens.
    pub fn total(&self) -> u64 {
        self.total_tokens
    }

    /// Formats token usage as a string.
    pub fn format(&self) -> String {
        if self.cached_tokens > 0 {
            format!(
                "{}in / {}out / {}cached = {} total",
                format_number(self.input_tokens),
                format_number(self.output_tokens),
                format_number(self.cached_tokens),
                format_number(self.total_tokens)
            )
        } else {
            format!(
                "{}in / {}out = {} total",
                format_number(self.input_tokens),
                format_number(self.output_tokens),
                format_number(self.total_tokens)
            )
        }
    }
}

/// Telemetry state tracking
#[derive(Debug, Clone)]
pub struct TelemetryState {
    /// Overall token metrics
    pub overall_tokens: TokenMetrics,
    /// Token metrics per agent
    pub agent_tokens: HashMap<String, TokenMetrics>,
    /// Overall cost (USD)
    pub overall_cost: f64,
    /// Cost per agent (USD)
    pub agent_costs: HashMap<String, f64>,
    /// Model being used
    pub model: Option<String>,
    /// Provider/engine being used
    pub provider: Option<String>,
    /// Provider cost breakdown (for multi-provider view)
    pub provider_breakdown: Vec<ProviderCostBreakdown>,
    /// Team cost breakdown (for attribution view)
    pub team_breakdown: Vec<TeamCostBreakdown>,
    /// Whether to show provider breakdown
    pub show_provider_breakdown: bool,
    /// Whether to show team breakdown
    pub show_team_breakdown: bool,
}

impl TelemetryState {
    /// Creates a new telemetry state.
    pub fn new() -> Self {
        Self {
            overall_tokens: TokenMetrics::new(),
            agent_tokens: HashMap::new(),
            overall_cost: 0.0,
            agent_costs: HashMap::new(),
            model: None,
            provider: None,
            provider_breakdown: Vec::new(),
            team_breakdown: Vec::new(),
            show_provider_breakdown: false,
            show_team_breakdown: false,
        }
    }

    /// Updates provider and team breakdowns from MonitoringService.
    ///
    /// # Arguments
    /// * `monitoring` - MonitoringService instance to query
    pub fn update_breakdowns(&mut self, monitoring: &radium_core::monitoring::MonitoringService) {
        use radium_core::monitoring::BudgetManager;
        
        // Update provider breakdown
        if let Ok(breakdowns) = BudgetManager::get_provider_breakdown(monitoring) {
            self.provider_breakdown = breakdowns;
        }

        // Update team breakdown (limit to top 5)
        if let Ok(mut breakdowns) = BudgetManager::get_team_breakdown(monitoring) {
            breakdowns.truncate(5);
            self.team_breakdown = breakdowns;
        }
    }

    /// Toggles provider breakdown display.
    pub fn toggle_provider_breakdown(&mut self) {
        self.show_provider_breakdown = !self.show_provider_breakdown;
    }

    /// Toggles team breakdown display.
    pub fn toggle_team_breakdown(&mut self) {
        self.show_team_breakdown = !self.show_team_breakdown;
    }

    /// Records token usage for an agent.
    pub fn record_tokens(&mut self, agent_id: String, input: u64, output: u64, cached: u64) {
        // Update overall metrics
        self.overall_tokens.add(input, output, cached);

        // Update agent-specific metrics
        let metrics = self.agent_tokens.entry(agent_id).or_insert_with(TokenMetrics::new);
        metrics.add(input, output, cached);
    }

    /// Records cost for an agent.
    pub fn record_cost(&mut self, agent_id: String, cost: f64) {
        // Update overall cost
        self.overall_cost += cost;

        // Update agent-specific cost
        *self.agent_costs.entry(agent_id).or_insert(0.0) += cost;
    }

    /// Sets the model being used.
    pub fn set_model(&mut self, model: String) {
        self.model = Some(model);
    }

    /// Sets the provider being used.
    pub fn set_provider(&mut self, provider: String) {
        self.provider = Some(provider);
    }

    /// Returns the total token count.
    pub fn total_tokens(&self) -> u64 {
        self.overall_tokens.total()
    }

    /// Returns the total cost.
    pub fn total_cost(&self) -> f64 {
        self.overall_cost
    }

    /// Formats the overall telemetry as a string.
    pub fn format_summary(&self) -> String {
        let model_info = if let Some(ref model) = self.model {
            if let Some(ref provider) = self.provider {
                format!(" ({}/{})", provider, model)
            } else {
                format!(" ({})", model)
            }
        } else {
            String::new()
        };

        format!(
            "Tokens: {} | Cost: ${:.4}{}",
            self.overall_tokens.format(),
            self.overall_cost,
            model_info
        )
    }

    /// Returns token metrics for a specific agent.
    pub fn get_agent_tokens(&self, agent_id: &str) -> Option<&TokenMetrics> {
        self.agent_tokens.get(agent_id)
    }

    /// Returns cost for a specific agent.
    pub fn get_agent_cost(&self, agent_id: &str) -> Option<f64> {
        self.agent_costs.get(agent_id).copied()
    }
}

impl Default for TelemetryState {
    fn default() -> Self {
        Self::new()
    }
}

/// Formats a number with commas for readability.
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut count = 0;

    for c in s.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(c);
        count += 1;
    }

    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_metrics() {
        let mut metrics = TokenMetrics::new();

        assert_eq!(metrics.total(), 0);

        metrics.add(100, 50, 0);
        assert_eq!(metrics.input_tokens, 100);
        assert_eq!(metrics.output_tokens, 50);
        assert_eq!(metrics.total(), 150);

        metrics.add(50, 25, 10);
        assert_eq!(metrics.input_tokens, 150);
        assert_eq!(metrics.output_tokens, 75);
        assert_eq!(metrics.cached_tokens, 10);
        assert_eq!(metrics.total(), 235);
    }

    #[test]
    fn test_telemetry_state() {
        let mut telemetry = TelemetryState::new();

        assert_eq!(telemetry.total_tokens(), 0);
        assert_eq!(telemetry.total_cost(), 0.0);

        telemetry.record_tokens("agent-1".to_string(), 100, 50, 0);
        assert_eq!(telemetry.total_tokens(), 150);

        telemetry.record_tokens("agent-2".to_string(), 200, 100, 0);
        assert_eq!(telemetry.total_tokens(), 450);

        telemetry.record_cost("agent-1".to_string(), 0.01);
        telemetry.record_cost("agent-2".to_string(), 0.02);
        assert!((telemetry.total_cost() - 0.03).abs() < 0.0001);
    }

    #[test]
    fn test_agent_specific_metrics() {
        let mut telemetry = TelemetryState::new();

        telemetry.record_tokens("agent-1".to_string(), 100, 50, 0);
        telemetry.record_cost("agent-1".to_string(), 0.01);

        let metrics = telemetry.get_agent_tokens("agent-1").unwrap();
        assert_eq!(metrics.input_tokens, 100);
        assert_eq!(metrics.output_tokens, 50);

        let cost = telemetry.get_agent_cost("agent-1").unwrap();
        assert!((cost - 0.01).abs() < 0.0001);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(100), "100");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1000000), "1,000,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_token_metrics_format() {
        let mut metrics = TokenMetrics::new();
        metrics.add(1000, 500, 0);
        assert_eq!(metrics.format(), "1,000in / 500out = 1,500 total");

        let mut metrics_cached = TokenMetrics::new();
        metrics_cached.add(1000, 500, 250);
        assert_eq!(metrics_cached.format(), "1,000in / 500out / 250cached = 1,750 total");
    }

    #[test]
    fn test_telemetry_summary() {
        let mut telemetry = TelemetryState::new();
        telemetry.record_tokens("agent-1".to_string(), 1000, 500, 0);
        telemetry.record_cost("agent-1".to_string(), 0.0123);
        telemetry.set_model("gpt-4".to_string());
        telemetry.set_provider("openai".to_string());

        let summary = telemetry.format_summary();
        assert!(summary.contains("1,500 total"));
        assert!(summary.contains("0.0123"));
        assert!(summary.contains("openai/gpt-4"));
    }
}

//! Cost tracker for per-tier token and cost tracking.

use super::types::RoutingTier;
use radium_abstraction::ModelUsage;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::debug;

/// Metrics for a single tier.
#[derive(Debug, Clone, Default)]
pub struct TierMetrics {
    /// Number of requests.
    pub request_count: u64,
    /// Total input tokens.
    pub input_tokens: u64,
    /// Total output tokens.
    pub output_tokens: u64,
    /// Estimated cost in USD.
    pub estimated_cost: f64,
}

/// Overall cost metrics across all tiers.
#[derive(Debug, Clone)]
pub struct CostMetrics {
    /// Smart tier metrics.
    pub smart_tier: TierMetrics,
    /// Eco tier metrics.
    pub eco_tier: TierMetrics,
    /// Total cost across all tiers.
    pub total_cost: f64,
    /// Total tokens across all tiers.
    pub total_tokens: u64,
}

impl CostMetrics {
    /// Calculates estimated savings vs using all-Smart baseline.
    ///
    /// This estimates what the cost would have been if all requests
    /// used Smart tier models, then compares to actual cost.
    ///
    /// # Arguments
    /// * `smart_input_price` - Cost per 1M input tokens for Smart tier
    /// * `smart_output_price` - Cost per 1M output tokens for Smart tier
    ///
    /// # Returns
    /// Estimated savings in USD (positive = saved money, negative = spent more)
    pub fn calculate_savings(&self, smart_input_price: f64, smart_output_price: f64) -> f64 {
        // Calculate what total cost would have been with all-Smart
        let total_input_tokens = self.smart_tier.input_tokens + self.eco_tier.input_tokens;
        let total_output_tokens = self.smart_tier.output_tokens + self.eco_tier.output_tokens;

        let all_smart_cost = (total_input_tokens as f64 / 1_000_000.0) * smart_input_price
            + (total_output_tokens as f64 / 1_000_000.0) * smart_output_price;

        // Compare to actual cost
        all_smart_cost - self.total_cost
    }
}

/// Cost tracker for per-tier usage and cost metrics.
pub struct CostTracker {
    /// Internal metrics storage (thread-safe).
    metrics: Arc<RwLock<HashMap<RoutingTier, TierMetrics>>>,
    /// Pricing lookup function.
    pricing_fn: Box<dyn Fn(&str) -> (f64, f64) + Send + Sync>,
}

impl CostTracker {
    /// Creates a new cost tracker with default pricing.
    #[must_use]
    pub fn new() -> Self {
        Self::with_pricing(Box::new(Self::default_pricing))
    }

    /// Creates a new cost tracker with custom pricing function.
    ///
    /// # Arguments
    /// * `pricing_fn` - Function that takes model_id and returns (input_price_per_1m, output_price_per_1m)
    #[must_use]
    pub fn with_pricing(pricing_fn: Box<dyn Fn(&str) -> (f64, f64) + Send + Sync>) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            pricing_fn,
        }
    }

    /// Default pricing lookup based on model ID patterns.
    ///
    /// Maps common model names to pricing (per 1M tokens).
    fn default_pricing(model_id: &str) -> (f64, f64) {
        let lower = model_id.to_lowercase();
        
        // Smart tier models (high cost)
        if lower.contains("sonnet") || lower.contains("gpt-4") || lower.contains("gpt-4o") 
            || lower.contains("pro") && !lower.contains("mini") {
            // Claude Sonnet, GPT-4: $3/$15 per 1M tokens
            return (3.0, 15.0);
        }
        
        // Eco tier models (low cost)
        if lower.contains("haiku") || lower.contains("mini") || lower.contains("flash")
            || lower.contains("gpt-3.5") {
            // Claude Haiku, GPT-3.5, Gemini Flash: $0.25/$1.25 per 1M tokens
            return (0.25, 1.25);
        }
        
        // Default fallback
        (1.0, 2.0)
    }

    /// Tracks usage for a specific tier.
    ///
    /// # Arguments
    /// * `tier` - The routing tier
    /// * `usage` - Model usage statistics
    /// * `model_id` - Model identifier for pricing lookup
    ///
    /// # Errors
    /// Returns error if tracking fails (should be logged but non-blocking)
    pub fn track_usage(
        &self,
        tier: RoutingTier,
        usage: &ModelUsage,
        model_id: &str,
    ) -> Result<(), String> {
        // Calculate cost based on pricing
        let (input_price, output_price) = (self.pricing_fn)(model_id);
        
        let input_tokens = u64::from(usage.prompt_tokens);
        let output_tokens = u64::from(usage.completion_tokens);
        
        let cost = (input_tokens as f64 / 1_000_000.0) * input_price
            + (output_tokens as f64 / 1_000_000.0) * output_price;

        // Update metrics (thread-safe)
        let mut metrics = self.metrics.write().map_err(|e| format!("Lock poisoned: {}", e))?;
        let tier_metrics = metrics.entry(tier).or_insert_with(TierMetrics::default);
        
        tier_metrics.request_count += 1;
        tier_metrics.input_tokens += input_tokens;
        tier_metrics.output_tokens += output_tokens;
        tier_metrics.estimated_cost += cost;

        debug!(
            tier = ?tier,
            model_id = model_id,
            input_tokens = input_tokens,
            output_tokens = output_tokens,
            cost = cost,
            total_cost = tier_metrics.estimated_cost,
            "Tracked usage"
        );

        Ok(())
    }

    /// Gets current cost metrics across all tiers.
    ///
    /// # Returns
    /// CostMetrics with aggregated data
    ///
    /// # Errors
    /// Returns error if metrics lock fails
    pub fn get_metrics(&self) -> Result<CostMetrics, String> {
        let metrics = self.metrics.read().map_err(|e| format!("Lock poisoned: {}", e))?;
        
        let smart_tier = metrics.get(&RoutingTier::Smart).cloned().unwrap_or_default();
        let eco_tier = metrics.get(&RoutingTier::Eco).cloned().unwrap_or_default();
        
        let total_cost = smart_tier.estimated_cost + eco_tier.estimated_cost;
        let total_tokens = smart_tier.input_tokens + smart_tier.output_tokens
            + eco_tier.input_tokens + eco_tier.output_tokens;

        Ok(CostMetrics {
            smart_tier,
            eco_tier,
            total_cost,
            total_tokens,
        })
    }

    /// Resets all metrics to zero.
    ///
    /// # Errors
    /// Returns error if lock fails
    pub fn reset(&self) -> Result<(), String> {
        let mut metrics = self.metrics.write().map_err(|e| format!("Lock poisoned: {}", e))?;
        metrics.clear();
        debug!("Reset cost tracking metrics");
        Ok(())
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tracker() -> CostTracker {
        CostTracker::new()
    }

    #[test]
    fn test_track_usage_smart_tier() {
        let tracker = create_test_tracker();
        let usage = ModelUsage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
        };

        let result = tracker.track_usage(RoutingTier::Smart, &usage, "claude-sonnet-3.5");
        assert!(result.is_ok());

        let metrics = tracker.get_metrics().unwrap();
        assert_eq!(metrics.smart_tier.request_count, 1);
        assert_eq!(metrics.smart_tier.input_tokens, 1000);
        assert_eq!(metrics.smart_tier.output_tokens, 500);
        assert!(metrics.smart_tier.estimated_cost > 0.0);
    }

    #[test]
    fn test_track_usage_eco_tier() {
        let tracker = create_test_tracker();
        let usage = ModelUsage {
            prompt_tokens: 2000,
            completion_tokens: 1000,
            total_tokens: 3000,
        };

        let result = tracker.track_usage(RoutingTier::Eco, &usage, "claude-haiku-3.5");
        assert!(result.is_ok());

        let metrics = tracker.get_metrics().unwrap();
        assert_eq!(metrics.eco_tier.request_count, 1);
        assert_eq!(metrics.eco_tier.input_tokens, 2000);
        assert_eq!(metrics.eco_tier.output_tokens, 1000);
        assert!(metrics.eco_tier.estimated_cost > 0.0);
    }

    #[test]
    fn test_cost_calculation() {
        let tracker = create_test_tracker();
        
        // Track Smart tier usage
        let smart_usage = ModelUsage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
        };
        tracker.track_usage(RoutingTier::Smart, &smart_usage, "claude-sonnet").unwrap();

        // Track Eco tier usage
        let eco_usage = ModelUsage {
            prompt_tokens: 2000,
            completion_tokens: 1000,
            total_tokens: 3000,
        };
        tracker.track_usage(RoutingTier::Eco, &eco_usage, "claude-haiku").unwrap();

        let metrics = tracker.get_metrics().unwrap();
        
        // Eco tier should be cheaper
        assert!(metrics.eco_tier.estimated_cost < metrics.smart_tier.estimated_cost);
        
        // Total cost should be sum of both
        assert_eq!(
            metrics.total_cost,
            metrics.smart_tier.estimated_cost + metrics.eco_tier.estimated_cost
        );
    }

    #[test]
    fn test_aggregation_multiple_requests() {
        let tracker = create_test_tracker();
        
        // Track multiple requests
        for _ in 0..5 {
            let usage = ModelUsage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            };
            tracker.track_usage(RoutingTier::Smart, &usage, "test-model").unwrap();
        }

        let metrics = tracker.get_metrics().unwrap();
        assert_eq!(metrics.smart_tier.request_count, 5);
        assert_eq!(metrics.smart_tier.input_tokens, 500);
        assert_eq!(metrics.smart_tier.output_tokens, 250);
    }

    #[test]
    fn test_reset() {
        let tracker = create_test_tracker();
        
        let usage = ModelUsage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
        };
        tracker.track_usage(RoutingTier::Smart, &usage, "test-model").unwrap();

        let metrics_before = tracker.get_metrics().unwrap();
        assert!(metrics_before.smart_tier.request_count > 0);

        tracker.reset().unwrap();

        let metrics_after = tracker.get_metrics().unwrap();
        assert_eq!(metrics_after.smart_tier.request_count, 0);
        assert_eq!(metrics_after.eco_tier.request_count, 0);
        assert_eq!(metrics_after.total_cost, 0.0);
    }

    #[test]
    fn test_savings_calculation() {
        let metrics = CostMetrics {
            smart_tier: TierMetrics {
                request_count: 2,
                input_tokens: 2000,
                output_tokens: 1000,
                estimated_cost: 0.021, // Smart: $3/$15 per 1M
            },
            eco_tier: TierMetrics {
                request_count: 8,
                input_tokens: 8000,
                output_tokens: 4000,
                estimated_cost: 0.003, // Eco: $0.25/$1.25 per 1M
            },
            total_cost: 0.024,
            total_tokens: 15000,
        };

        // If all 10 requests used Smart tier:
        // (10000 input * $3 + 5000 output * $15) / 1M = $0.105
        // Actual cost: $0.024
        // Savings: $0.081
        let savings = metrics.calculate_savings(3.0, 15.0);
        assert!(savings > 0.0, "Expected positive savings");
    }
}


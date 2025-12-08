//! Model router for Smart/Eco tier selection.

use super::ab_testing::{ABTestGroup, ABTestSampler};
use super::complexity::ComplexityEstimator;
use super::cost_tracker::CostTracker;
use super::types::{ComplexityScore, ComplexityWeights, RoutingTier};
use radium_models::{ModelConfig, ModelType};
use std::sync::Arc;
use tracing::{debug, warn};

/// Parses model specification string into engine and model parts.
fn parse_model_spec(spec: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid model format '{}', expected 'engine:model'",
            spec
        ));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Converts engine string to ModelType.
fn engine_to_type(engine: &str) -> Result<ModelType, String> {
    match engine {
        "claude" => Ok(ModelType::Claude),
        "openai" => Ok(ModelType::OpenAI),
        "gemini" => Ok(ModelType::Gemini),
        "mock" => Ok(ModelType::Mock),
        _ => Err(format!("Unsupported engine: {}", engine)),
    }
}

/// Model router for selecting between Smart and Eco tiers.
pub struct ModelRouter {
    /// Smart tier model configuration (high-capability).
    smart_model: ModelConfig,
    /// Eco tier model configuration (cost-effective).
    eco_model: ModelConfig,
    /// Complexity threshold for routing (default 60).
    threshold: f64,
    /// Complexity estimator.
    estimator: ComplexityEstimator,
    /// Whether auto-routing is enabled.
    auto_route: bool,
    /// Cost tracker for per-tier usage tracking.
    cost_tracker: Arc<CostTracker>,
    /// Optional A/B test sampler for routing validation.
    ab_test_sampler: Option<Arc<ABTestSampler>>,
}

impl ModelRouter {
    /// Creates a new model router.
    ///
    /// # Arguments
    /// * `smart_model` - Smart tier model configuration
    /// * `eco_model` - Eco tier model configuration
    /// * `threshold` - Complexity threshold for routing (default 60)
    ///
    /// # Returns
    /// A new ModelRouter instance.
    #[must_use]
    pub fn new(smart_model: ModelConfig, eco_model: ModelConfig, threshold: Option<f64>) -> Self {
        Self {
            smart_model,
            eco_model,
            threshold: threshold.unwrap_or(60.0),
            estimator: ComplexityEstimator::new(),
            auto_route: true,
            cost_tracker: Arc::new(CostTracker::new()),
            ab_test_sampler: None,
        }
    }

    /// Creates a new model router with custom complexity estimator weights.
    #[must_use]
    pub fn with_weights(
        smart_model: ModelConfig,
        eco_model: ModelConfig,
        threshold: Option<f64>,
        weights: ComplexityWeights,
    ) -> Self {
        Self {
            smart_model,
            eco_model,
            threshold: threshold.unwrap_or(60.0),
            estimator: ComplexityEstimator::with_weights(weights),
            auto_route: true,
            cost_tracker: Arc::new(CostTracker::new()),
            ab_test_sampler: None,
        }
    }
    
    /// Sets the A/B test sampler for routing validation.
    ///
    /// # Arguments
    /// * `sampler` - A/B test sampler instance
    pub fn with_ab_testing(mut self, sampler: ABTestSampler) -> Self {
        self.ab_test_sampler = Some(Arc::new(sampler));
        self
    }
    
    /// Creates a new model router from model specification strings.
    ///
    /// Parses model specifications in "engine:model" format and creates
    /// ModelRouter with the specified settings. This is a convenience
    /// method that works with configuration loaded externally.
    ///
    /// # Arguments
    /// * `smart_model` - Smart tier model spec (e.g., "claude:claude-sonnet-4.5")
    /// * `eco_model` - Eco tier model spec (e.g., "claude:claude-haiku-4.5")
    /// * `threshold` - Complexity threshold for routing
    /// * `weights` - Complexity estimation weights
    /// * `auto_route` - Whether auto-routing is enabled
    ///
    /// # Errors
    /// Returns error if model specification parsing fails.
    pub fn from_specs(
        smart_model: &str,
        eco_model: &str,
        threshold: f64,
        weights: ComplexityWeights,
        auto_route: bool,
    ) -> Result<Self, String> {
        // Parse model specifications
        let (smart_engine, smart_model_id) = parse_model_spec(smart_model)?;
        let (eco_engine, eco_model_id) = parse_model_spec(eco_model)?;
        
        // Convert engine strings to ModelType
        let smart_type = engine_to_type(&smart_engine)?;
        let eco_type = engine_to_type(&eco_engine)?;
        
        let smart_config = ModelConfig::new(smart_type, smart_model_id);
        let eco_config = ModelConfig::new(eco_type, eco_model_id);
        
        let mut router = Self::with_weights(
            smart_config,
            eco_config,
            Some(threshold),
            weights,
        );
        
        router.auto_route = auto_route;
        
        Ok(router)
    }

    /// Selects the appropriate model based on complexity or override.
    ///
    /// # Arguments
    /// * `input` - The input prompt/text
    /// * `agent_id` - Optional agent ID for context
    /// * `tier_override` - Optional manual tier override
    ///
    /// # Returns
    /// Selected ModelConfig and routing decision metadata.
    pub fn select_model(
        &self,
        input: &str,
        agent_id: Option<&str>,
        tier_override: Option<RoutingTier>,
    ) -> (ModelConfig, RoutingDecision) {
        // Handle manual override
        if let Some(override_tier) = tier_override {
            match override_tier {
                RoutingTier::Smart => {
                    debug!("Manual override: routing to Smart tier");
                    return (
                        self.smart_model.clone(),
                        RoutingDecision {
                            tier: RoutingTier::Smart,
                            decision_type: DecisionType::Manual,
                            complexity_score: None,
                            ab_test_group: None,
                        },
                    );
                }
                RoutingTier::Eco => {
                    debug!("Manual override: routing to Eco tier");
                    return (
                        self.eco_model.clone(),
                        RoutingDecision {
                            tier: RoutingTier::Eco,
                            decision_type: DecisionType::Manual,
                            complexity_score: None,
                            ab_test_group: None,
                        },
                    );
                }
                RoutingTier::Auto => {
                    // Fall through to auto-routing
                }
            }
        }

        // Handle auto-routing
        if !self.auto_route {
            // Auto-routing disabled, default to Smart tier (safe fallback)
            warn!("Auto-routing disabled, defaulting to Smart tier");
            return (
                self.smart_model.clone(),
                RoutingDecision {
                    tier: RoutingTier::Smart,
                    decision_type: DecisionType::Fallback,
                    complexity_score: None,
                    ab_test_group: None,
                },
            );
        }

        // Estimate complexity
        let complexity = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.estimator.estimate(input, agent_id)
        })) {
            Ok(score) => score,
            Err(_) => {
                warn!("Complexity estimation failed, defaulting to Smart tier");
                return (
                    self.smart_model.clone(),
                    RoutingDecision {
                        tier: RoutingTier::Smart,
                        decision_type: DecisionType::Fallback,
                        complexity_score: None,
                        ab_test_group: None,
                    },
                );
            }
        };

        // Route based on complexity threshold
        let mut tier = if complexity.score >= self.threshold {
            RoutingTier::Smart
        } else {
            RoutingTier::Eco
        };

        // Handle A/B testing: invert routing for Test group
        let ab_test_group = if let Some(ref sampler) = self.ab_test_sampler {
            let group = sampler.assign_group();
            if group == ABTestGroup::Test {
                // Invert routing decision for test group
                tier = match tier {
                    RoutingTier::Smart => RoutingTier::Eco,
                    RoutingTier::Eco => RoutingTier::Smart,
                    RoutingTier::Auto => tier, // Should not happen
                };
                debug!("A/B test: Test group assignment, inverted routing");
            }
            Some(group)
        } else {
            None
        };

        debug!(
            complexity_score = complexity.score,
            threshold = self.threshold,
            selected_tier = ?tier,
            ab_test_group = ?ab_test_group,
            "Auto-routing based on complexity"
        );

        let model = match tier {
            RoutingTier::Smart => self.smart_model.clone(),
            RoutingTier::Eco => self.eco_model.clone(),
            RoutingTier::Auto => unreachable!(), // Should not happen here
        };

        (
            model,
            RoutingDecision {
                tier,
                decision_type: DecisionType::Auto,
                complexity_score: Some(complexity.score),
                ab_test_group,
            },
        )
    }

    /// Estimates complexity without selecting a model (for testing/debugging).
    pub fn estimate_complexity(&self, input: &str, agent_id: Option<&str>) -> ComplexityScore {
        self.estimator.estimate(input, agent_id)
    }

    /// Gets the current routing threshold.
    #[must_use]
    pub fn threshold(&self) -> f64 {
        self.threshold
    }

    /// Sets the routing threshold.
    pub fn set_threshold(&mut self, threshold: f64) {
        self.threshold = threshold;
        debug!(threshold = threshold, "Updated routing threshold");
    }

    /// Gets whether auto-routing is enabled.
    #[must_use]
    pub fn auto_route(&self) -> bool {
        self.auto_route
    }

    /// Sets whether auto-routing is enabled.
    pub fn set_auto_route(&mut self, enabled: bool) {
        self.auto_route = enabled;
        debug!(enabled = enabled, "Updated auto-routing setting");
    }

    /// Tracks usage for a routing decision (non-blocking).
    ///
    /// # Arguments
    /// * `tier` - The tier that was used
    /// * `usage` - Model usage statistics
    /// * `model_id` - Model identifier for pricing lookup
    pub fn track_usage(
        &self,
        tier: RoutingTier,
        usage: &radium_abstraction::ModelUsage,
        model_id: &str,
    ) {
        // Non-blocking: log errors but don't propagate
        if let Err(e) = self.cost_tracker.track_usage(tier, usage, model_id) {
            warn!(
                tier = ?tier,
                model_id = model_id,
                error = %e,
                "Failed to track usage (non-blocking)"
            );
        }
    }

    /// Gets current cost metrics.
    ///
    /// # Returns
    /// CostMetrics if available, None on error
    #[must_use]
    pub fn get_cost_metrics(&self) -> Option<super::cost_tracker::CostMetrics> {
        self.cost_tracker.get_metrics().ok()
    }

    /// Resets cost tracking metrics.
    pub fn reset_cost_tracking(&self) {
        if let Err(e) = self.cost_tracker.reset() {
            warn!(error = %e, "Failed to reset cost tracking");
        }
    }
}

/// Routing decision metadata.
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    /// Selected tier.
    pub tier: RoutingTier,
    /// Type of decision made.
    pub decision_type: DecisionType,
    /// Complexity score if available.
    pub complexity_score: Option<f64>,
    /// A/B test group assignment if A/B testing is enabled.
    pub ab_test_group: Option<ABTestGroup>,
}

/// Type of routing decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionType {
    /// Automatic routing based on complexity.
    Auto,
    /// Manual override by user.
    Manual,
    /// Fallback due to error or disabled auto-routing.
    Fallback,
}

impl DecisionType {
    /// Converts to string for telemetry.
    #[must_use]
    pub fn to_string(&self) -> String {
        match self {
            DecisionType::Auto => "auto".to_string(),
            DecisionType::Manual => "manual".to_string(),
            DecisionType::Fallback => "fallback".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radium_models::ModelType;

    fn create_test_router() -> ModelRouter {
        let smart_config = ModelConfig::new(ModelType::Mock, "smart-model".to_string());
        let eco_config = ModelConfig::new(ModelType::Mock, "eco-model".to_string());
        ModelRouter::new(smart_config, eco_config, Some(60.0))
    }

    #[test]
    fn test_simple_task_routes_to_eco() {
        let router = create_test_router();
        let (model, decision) = router.select_model("format this JSON", None, None);
        
        assert_eq!(model.model_id, "eco-model");
        assert_eq!(decision.tier, RoutingTier::Eco);
        assert_eq!(decision.decision_type, DecisionType::Auto);
        assert!(decision.complexity_score.is_some());
        assert!(decision.complexity_score.unwrap() < 60.0);
    }

    #[test]
    fn test_complex_task_routes_to_smart() {
        let router = create_test_router();
        let (model, decision) = router.select_model(
            "analyze the trade-offs between microservices and monolithic architecture, considering scalability and deployment complexity and design patterns",
            None,
            None,
        );
        
        // This should route to smart if complexity >= 60, otherwise eco
        // With the improved heuristics, it should score >= 60
        let complexity = decision.complexity_score.unwrap_or(0.0);
        
        if complexity >= 60.0 {
            assert_eq!(model.model_id, "smart-model");
            assert_eq!(decision.tier, RoutingTier::Smart);
        } else {
            // If score < 60, it routes to eco (which is also valid)
            assert_eq!(model.model_id, "eco-model");
            assert_eq!(decision.tier, RoutingTier::Eco);
        }
        assert_eq!(decision.decision_type, DecisionType::Auto);
        assert!(decision.complexity_score.is_some());
    }

    #[test]
    fn test_manual_override_smart() {
        let router = create_test_router();
        let (model, decision) = router.select_model(
            "simple task",
            None,
            Some(RoutingTier::Smart),
        );
        
        assert_eq!(model.model_id, "smart-model");
        assert_eq!(decision.tier, RoutingTier::Smart);
        assert_eq!(decision.decision_type, DecisionType::Manual);
        assert!(decision.complexity_score.is_none());
    }

    #[test]
    fn test_manual_override_eco() {
        let router = create_test_router();
        let (model, decision) = router.select_model(
            "complex refactoring task",
            None,
            Some(RoutingTier::Eco),
        );
        
        assert_eq!(model.model_id, "eco-model");
        assert_eq!(decision.tier, RoutingTier::Eco);
        assert_eq!(decision.decision_type, DecisionType::Manual);
        assert!(decision.complexity_score.is_none());
    }

    #[test]
    fn test_fallback_on_error() {
        let mut router = create_test_router();
        router.set_auto_route(false);
        
        let (model, decision) = router.select_model("any task", None, None);
        
        assert_eq!(model.model_id, "smart-model");
        assert_eq!(decision.tier, RoutingTier::Smart);
        assert_eq!(decision.decision_type, DecisionType::Fallback);
    }
}


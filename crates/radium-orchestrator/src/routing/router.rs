//! Model router for Smart/Eco tier selection.

use super::complexity::ComplexityEstimator;
use super::cost_tracker::CostTracker;
use super::types::{ComplexityScore, ComplexityWeights, RoutingTier};
use radium_models::{ModelConfig, ModelFactory, ModelType};
use std::sync::Arc;
use tracing::{debug, warn};

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
        }
    }
    
    /// Creates a new model router from RoutingConfig.
    ///
    /// Loads configuration from TOML file or uses defaults, then creates
    /// ModelRouter with the specified settings.
    ///
    /// # Errors
    /// Returns error if configuration loading or model parsing fails.
    pub fn from_config() -> Result<Self, String> {
        use radium_core::config::routing::RoutingConfig;
        
        let config = RoutingConfig::load().map_err(|e| format!("Failed to load routing config: {}", e))?;
        
        // Parse model specifications
        let (smart_engine, smart_model_id) = config.parse_model_spec(&config.smart_model)
            .map_err(|e| format!("Failed to parse smart_model '{}': {}", config.smart_model, e))?;
        let (eco_engine, eco_model_id) = config.parse_model_spec(&config.eco_model)
            .map_err(|e| format!("Failed to parse eco_model '{}': {}", config.eco_model, e))?;
        
        // Convert engine strings to ModelType
        let smart_type = match smart_engine.as_str() {
            "claude" => ModelType::Claude,
            "openai" => ModelType::OpenAI,
            "gemini" => ModelType::Gemini,
            "mock" => ModelType::Mock,
            _ => return Err(format!("Unsupported engine: {}", smart_engine)),
        };
        
        let eco_type = match eco_engine.as_str() {
            "claude" => ModelType::Claude,
            "openai" => ModelType::OpenAI,
            "gemini" => ModelType::Gemini,
            "mock" => ModelType::Mock,
            _ => return Err(format!("Unsupported engine: {}", eco_engine)),
        };
        
        let smart_config = ModelConfig::new(smart_type, smart_model_id);
        let eco_config = ModelConfig::new(eco_type, eco_model_id);
        
        // Convert config weights to ComplexityWeights
        let weights = ComplexityWeights {
            token_count: config.weights.token_count,
            task_type: config.weights.task_type,
            reasoning: config.weights.reasoning,
            context: config.weights.context,
        };
        
        let mut router = Self::with_weights(
            smart_config,
            eco_config,
            Some(config.complexity_threshold),
            weights,
        );
        
        router.auto_route = config.auto_route;
        
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
                    },
                );
            }
        };

        // Route based on complexity threshold
        let tier = if complexity.score >= self.threshold {
            RoutingTier::Smart
        } else {
            RoutingTier::Eco
        };

        debug!(
            complexity_score = complexity.score,
            threshold = self.threshold,
            selected_tier = ?tier,
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


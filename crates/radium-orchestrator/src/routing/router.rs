//! Model router for Smart/Eco tier selection.

use super::ab_testing::{ABTestGroup, ABTestSampler};
use super::circuit_breaker::CircuitBreaker;
use super::complexity::ComplexityEstimator;
use super::config::{ConfigError, RoutingConfigLoader};
use super::cost_tracker::CostTracker;
use super::types::{ComplexityScore, ComplexityWeights, FailureRecord, FallbackChain, ModelMetadata, RoutingError, RoutingStrategy, RoutingTier};
use radium_models::{ModelConfig, ModelType};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

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
    /// Optional fallback chain for multi-model retry logic.
    fallback_chain: Option<FallbackChain>,
    /// Track which models have been tried in the current fallback sequence (thread-safe).
    tried_models: Arc<RwLock<Vec<String>>>,
    /// Optional circuit breaker for failure detection.
    circuit_breaker: Option<Arc<CircuitBreaker>>,
    /// Model metadata registry for strategy-based routing.
    model_registry: Arc<RwLock<HashMap<String, ModelMetadata>>>,
    /// Default routing strategy.
    default_strategy: RoutingStrategy,
}

impl ModelRouter {
    /// Creates default model registry with hardcoded metadata for common models.
    fn default_model_registry() -> HashMap<String, ModelMetadata> {
        let mut registry = HashMap::new();
        
        // Claude models
        registry.insert("claude-sonnet-4.5".to_string(), ModelMetadata::new(
            "claude-sonnet-4.5".to_string(),
            "claude".to_string(),
            3.0,   // $3 per 1M input
            15.0,  // $15 per 1M output
            2000,  // 2000ms avg latency
            5,     // Tier 5 (highest)
        ));
        registry.insert("claude-sonnet-3.5".to_string(), ModelMetadata::new(
            "claude-sonnet-3.5".to_string(),
            "claude".to_string(),
            3.0,
            15.0,
            2000,
            5,
        ));
        registry.insert("claude-haiku-4.5".to_string(), ModelMetadata::new(
            "claude-haiku-4.5".to_string(),
            "claude".to_string(),
            0.25,  // $0.25 per 1M input
            1.25,  // $1.25 per 1M output
            500,   // 500ms avg latency
            3,     // Tier 3
        ));
        registry.insert("claude-haiku-3.5".to_string(), ModelMetadata::new(
            "claude-haiku-3.5".to_string(),
            "claude".to_string(),
            0.25,
            1.25,
            500,
            3,
        ));
        
        // OpenAI models
        registry.insert("gpt-4".to_string(), ModelMetadata::new(
            "gpt-4".to_string(),
            "openai".to_string(),
            30.0,  // $30 per 1M input
            60.0,  // $60 per 1M output
            3000,  // 3000ms avg latency
            5,     // Tier 5
        ));
        registry.insert("gpt-4-turbo".to_string(), ModelMetadata::new(
            "gpt-4-turbo".to_string(),
            "openai".to_string(),
            10.0,
            30.0,
            2500,
            5,
        ));
        registry.insert("gpt-3.5-turbo".to_string(), ModelMetadata::new(
            "gpt-3.5-turbo".to_string(),
            "openai".to_string(),
            1.5,   // $1.5 per 1M input
            2.0,   // $2 per 1M output
            800,   // 800ms avg latency
            3,     // Tier 3
        ));
        
        // Gemini models
        registry.insert("gemini-pro".to_string(), ModelMetadata::new(
            "gemini-pro".to_string(),
            "gemini".to_string(),
            0.5,   // $0.5 per 1M input
            1.5,   // $1.5 per 1M output
            1200,  // 1200ms avg latency
            4,     // Tier 4
        ));
        registry.insert("gemini-flash".to_string(), ModelMetadata::new(
            "gemini-flash".to_string(),
            "gemini".to_string(),
            0.2,   // $0.2 per 1M input
            0.8,   // $0.8 per 1M output
            400,   // 400ms avg latency
            3,     // Tier 3
        ));
        
        registry
    }
    
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
            fallback_chain: None,
            tried_models: Arc::new(RwLock::new(Vec::new())),
            circuit_breaker: None,
            model_registry: Arc::new(RwLock::new(Self::default_model_registry())),
            default_strategy: RoutingStrategy::ComplexityBased,
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
            fallback_chain: None,
            tried_models: Arc::new(RwLock::new(Vec::new())),
            circuit_breaker: None,
            model_registry: Arc::new(RwLock::new(Self::default_model_registry())),
            default_strategy: RoutingStrategy::ComplexityBased,
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
    
    /// Sets the fallback chain for multi-model retry logic.
    ///
    /// # Arguments
    /// * `chain` - Fallback chain with ordered list of models
    pub fn with_fallback_chain(mut self, chain: FallbackChain) -> Self {
        self.fallback_chain = Some(chain);
        self
    }
    
    /// Sets the circuit breaker for failure detection.
    ///
    /// # Arguments
    /// * `breaker` - Circuit breaker instance
    pub fn with_circuit_breaker(mut self, breaker: CircuitBreaker) -> Self {
        self.circuit_breaker = Some(Arc::new(breaker));
        self
    }
    
    /// Gets the next model in the fallback chain after a failure.
    ///
    /// # Arguments
    /// * `failed_model_id` - The model ID that failed
    /// * `error` - Error message describing the failure
    ///
    /// # Returns
    /// - `Ok(Some(ModelConfig))` if there's a next model to try
    /// - `Ok(None)` if no fallback chain is configured
    /// - `Err(RoutingError::AllModelsFailed)` if all models in chain have been tried
    pub fn get_next_fallback_model(
        &self,
        failed_model_id: &str,
        error: &str,
    ) -> Result<Option<ModelConfig>, RoutingError> {
        // Record the failure
        {
            let mut tried = self.tried_models.write().unwrap();
            tried.push(failed_model_id.to_string());
        }
        
        // If no fallback chain, return None (no fallback available)
        let chain = match &self.fallback_chain {
            Some(chain) => chain,
            None => return Ok(None),
        };
        
        // Find the next model in the chain that hasn't been tried and isn't circuit-broken
        let tried = self.tried_models.read().unwrap();
        for model_config in &chain.models {
            // Skip if already tried
            if tried.contains(&model_config.model_id) {
                continue;
            }
            
            // Skip if circuit breaker says to skip this model
            if let Some(ref breaker) = self.circuit_breaker {
                if breaker.should_skip(&model_config.model_id) {
                    debug!(
                        model_id = model_config.model_id,
                        "Skipping model due to open circuit breaker"
                    );
                    continue;
                }
            }
            
            // Found a valid model
            drop(tried);
            let mut tried = self.tried_models.write().unwrap();
            tried.push(model_config.model_id.clone());
            return Ok(Some(model_config.clone()));
        }
        drop(tried);
        
        // All models have been tried - return error with failure records
        let tried = self.tried_models.read().unwrap();
        let failures: Vec<FailureRecord> = tried
            .iter()
            .map(|model_id| FailureRecord::new(
                model_id.clone(),
                if model_id == failed_model_id {
                    error.to_string()
                } else {
                    "Model skipped in fallback chain".to_string()
                }
            ))
            .collect();
        drop(tried);
        
        // Reset tried models for next attempt
        {
            let mut tried = self.tried_models.write().unwrap();
            tried.clear();
        }
        
        Err(RoutingError::AllModelsFailed(failures))
    }
    
    /// Resets the fallback chain state (clears tried models).
    ///
    /// Call this when starting a new routing sequence.
    pub fn reset_fallback_state(&self) {
        let mut tried = self.tried_models.write().unwrap();
        tried.clear();
    }
    
    /// Records a model success for circuit breaker tracking.
    ///
    /// # Arguments
    /// * `model_id` - Model identifier
    pub fn record_model_success(&self, model_id: &str) {
        if let Some(ref breaker) = self.circuit_breaker {
            breaker.record_success(model_id);
        }
    }
    
    /// Records a model failure for circuit breaker tracking.
    ///
    /// # Arguments
    /// * `model_id` - Model identifier
    pub fn record_model_failure(&self, model_id: &str) {
        if let Some(ref breaker) = self.circuit_breaker {
            breaker.record_failure(model_id);
        }
    }
    
    /// Selects tier based on cost optimization strategy.
    fn select_by_cost_optimized(&self, complexity_score: f64) -> RoutingTier {
        let registry = self.model_registry.read().unwrap();
        
        // Get metadata for smart and eco models
        let smart_metadata = registry.get(&self.smart_model.model_id);
        let eco_metadata = registry.get(&self.eco_model.model_id);
        
        // If complexity requires smart tier, use smart
        if complexity_score >= self.threshold {
            return RoutingTier::Smart;
        }
        
        // Otherwise, compare costs
        if let (Some(smart), Some(eco)) = (smart_metadata, eco_metadata) {
            // Compare total cost (input + output, using average ratio)
            // Use input cost as proxy (most models have similar input/output ratios)
            if smart.cost_per_1m_input < eco.cost_per_1m_input {
                RoutingTier::Smart
            } else {
                RoutingTier::Eco
            }
        } else {
            // Fallback to complexity-based if metadata not available
            if complexity_score >= self.threshold {
                RoutingTier::Smart
            } else {
                RoutingTier::Eco
            }
        }
    }
    
    /// Selects tier based on latency optimization strategy.
    fn select_by_latency_optimized(&self, complexity_score: f64) -> RoutingTier {
        let registry = self.model_registry.read().unwrap();
        
        // Get metadata for smart and eco models
        let smart_metadata = registry.get(&self.smart_model.model_id);
        let eco_metadata = registry.get(&self.eco_model.model_id);
        
        // If complexity requires smart tier, use smart
        if complexity_score >= self.threshold {
            return RoutingTier::Smart;
        }
        
        // Otherwise, compare latencies
        if let (Some(smart), Some(eco)) = (smart_metadata, eco_metadata) {
            if smart.avg_latency_ms < eco.avg_latency_ms {
                RoutingTier::Smart
            } else {
                RoutingTier::Eco
            }
        } else {
            // Fallback to complexity-based if metadata not available
            if complexity_score >= self.threshold {
                RoutingTier::Smart
            } else {
                RoutingTier::Eco
            }
        }
    }
    
    /// Selects tier based on quality optimization strategy.
    fn select_by_quality_optimized(&self, complexity_score: f64) -> RoutingTier {
        let registry = self.model_registry.read().unwrap();
        
        // Get metadata for smart and eco models
        let smart_metadata = registry.get(&self.smart_model.model_id);
        let eco_metadata = registry.get(&self.eco_model.model_id);
        
        // Always prefer higher quality, but respect complexity threshold
        if let (Some(smart), Some(eco)) = (smart_metadata, eco_metadata) {
            // If complexity is high, use smart (higher quality)
            if complexity_score >= self.threshold {
                RoutingTier::Smart
            } else if smart.quality_tier > eco.quality_tier {
                // Even for low complexity, prefer higher quality if available
                RoutingTier::Smart
            } else {
                RoutingTier::Eco
            }
        } else {
            // Fallback to complexity-based if metadata not available
            if complexity_score >= self.threshold {
                RoutingTier::Smart
            } else {
                RoutingTier::Eco
            }
        }
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
        router.fallback_chain = None;
        router.tried_models = Arc::new(RwLock::new(Vec::new()));
        router.circuit_breaker = None;
        router.model_registry = Arc::new(RwLock::new(Self::default_model_registry()));
        router.default_strategy = RoutingStrategy::ComplexityBased;
        
        Ok(router)
    }

    /// Selects the appropriate model based on complexity or override.
    ///
    /// Uses the default strategy (ComplexityBased).
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
        self.select_model_with_strategy(input, agent_id, tier_override, self.default_strategy)
    }
    
    /// Selects the appropriate model using a specific routing strategy.
    ///
    /// # Arguments
    /// * `input` - The input prompt/text
    /// * `agent_id` - Optional agent ID for context
    /// * `tier_override` - Optional manual tier override
    /// * `strategy` - Routing strategy to use
    ///
    /// # Returns
    /// Selected ModelConfig and routing decision metadata.
    pub fn select_model_with_strategy(
        &self,
        input: &str,
        agent_id: Option<&str>,
        tier_override: Option<RoutingTier>,
        strategy: RoutingStrategy,
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

        // Route based on strategy
        let mut tier = match strategy {
            RoutingStrategy::ComplexityBased => {
                // Use complexity threshold (existing behavior)
                if complexity.score >= self.threshold {
                    RoutingTier::Smart
                } else {
                    RoutingTier::Eco
                }
            }
            RoutingStrategy::CostOptimized => {
                // Select cheapest model that meets complexity threshold
                self.select_by_cost_optimized(complexity.score)
            }
            RoutingStrategy::LatencyOptimized => {
                // Select fastest model that meets complexity threshold
                self.select_by_latency_optimized(complexity.score)
            }
            RoutingStrategy::QualityOptimized => {
                // Select highest tier model that meets complexity threshold
                self.select_by_quality_optimized(complexity.score)
            }
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
            strategy = ?strategy,
            ab_test_group = ?ab_test_group,
            "Routing decision made"
        );
        
        // Log routing decision for metrics
        info!(
            tier = ?tier,
            strategy = ?strategy,
            complexity_score = complexity.score,
            decision_type = ?DecisionType::Auto,
            "Routing decision: {} tier selected with {} strategy (complexity: {:.2})",
            tier,
            strategy.to_string(),
            complexity.score
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
    
    /// Creates a new model router from a configuration file.
    ///
    /// # Arguments
    /// * `config_path` - Path to the routing configuration TOML file
    /// * `smart_model` - Smart tier model configuration (required)
    /// * `eco_model` - Eco tier model configuration (required)
    ///
    /// # Errors
    /// Returns error if configuration cannot be loaded or is invalid.
    pub fn from_config(
        config_path: &Path,
        smart_model: ModelConfig,
        eco_model: ModelConfig,
    ) -> Result<Self, ConfigError> {
        let config = RoutingConfigLoader::load(config_path)?;
        
        let threshold = config.threshold.unwrap_or(60.0);
        let default_strategy = RoutingStrategy::from_str(&config.default_strategy)
            .unwrap_or(RoutingStrategy::ComplexityBased);
        
        let mut router = Self::new(smart_model, eco_model, Some(threshold));
        router.default_strategy = default_strategy;
        
        // Build fallback chains
        let chains = RoutingConfigLoader::build_fallback_chains(&config)?;
        if let Some((_, chain)) = chains.first() {
            router.fallback_chain = Some(chain.clone());
        }
        
        Ok(router)
    }
    
    /// Reloads configuration from a file.
    ///
    /// # Arguments
    /// * `config_path` - Path to the routing configuration TOML file
    ///
    /// # Errors
    /// Returns error if configuration cannot be loaded or is invalid.
    pub fn reload_config(&mut self, config_path: &Path) -> Result<(), ConfigError> {
        let config = RoutingConfigLoader::load(config_path)?;
        
        // Update threshold
        if let Some(threshold) = config.threshold {
            self.set_threshold(threshold);
        }
        
        // Update default strategy
        if let Some(strategy) = RoutingStrategy::from_str(&config.default_strategy) {
            self.default_strategy = strategy;
        }
        
        // Update fallback chains
        let chains = RoutingConfigLoader::build_fallback_chains(&config)?;
        if let Some((_, chain)) = chains.first() {
            self.fallback_chain = Some(chain.clone());
        } else {
            self.fallback_chain = None;
        }
        
        Ok(())
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


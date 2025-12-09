//! Model selection engine for intelligent model routing.
//!
//! This module provides intelligent model selection based on agent metadata,
//! including priority-based selection, cost estimation, budget tracking,
//! and automatic fallback chains.

use crate::agents::metadata::{AgentMetadata, CostTier, ModelPriority, ModelRecommendation};
use radium_abstraction::{Model, ModelError};
use radium_models::{ModelCache, ModelConfig, ModelFactory, ModelType};
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Error types for model selection.
#[derive(Error, Debug)]
pub enum SelectionError {
    /// No models could be created (all failed).
    #[error("No available models: {0}")]
    NoAvailableModels(String),

    /// Budget exceeded.
    #[error("Budget exceeded: estimated cost ${0:.4} exceeds limit ${1:.4}")]
    BudgetExceeded(f64, f64),

    /// Model creation failed.
    #[error("Model creation failed: {0}")]
    ModelCreationFailed(#[from] ModelError),

    /// Premium model requires approval.
    #[error("Premium model requires user approval")]
    ApprovalRequired,

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

/// Which model was selected from the recommendation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectedModel {
    /// Primary recommended model.
    Primary,
    /// Fallback model.
    Fallback,
    /// Premium model.
    Premium,
    /// Mock model (used when all others fail).
    Mock,
}

impl std::fmt::Display for SelectedModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primary => write!(f, "primary"),
            Self::Fallback => write!(f, "fallback"),
            Self::Premium => write!(f, "premium"),
            Self::Mock => write!(f, "mock"),
        }
    }
}

/// Options for model selection.
pub struct SelectionOptions<'a> {
    /// Agent metadata containing model recommendations.
    pub agent_metadata: &'a AgentMetadata,
    /// Estimated prompt tokens for cost estimation.
    pub estimated_prompt_tokens: Option<u32>,
    /// Estimated completion tokens for cost estimation.
    pub estimated_completion_tokens: Option<u32>,
    /// Whether to allow premium models without approval.
    pub allow_premium_without_approval: bool,
}

impl<'a> SelectionOptions<'a> {
    /// Create new selection options from agent metadata.
    #[must_use]
    pub fn new(agent_metadata: &'a AgentMetadata) -> Self {
        Self {
            agent_metadata,
            estimated_prompt_tokens: None,
            estimated_completion_tokens: None,
            allow_premium_without_approval: false,
        }
    }

    /// Set estimated token counts for cost calculation.
    #[must_use]
    pub fn with_token_estimate(mut self, prompt: u32, completion: u32) -> Self {
        self.estimated_prompt_tokens = Some(prompt);
        self.estimated_completion_tokens = Some(completion);
        self
    }

    /// Allow premium models without explicit approval.
    #[must_use]
    pub fn allow_premium(mut self) -> Self {
        self.allow_premium_without_approval = true;
        self
    }
}

/// Result of model selection.
pub struct SelectionResult {
    /// The selected model instance.
    pub model: Arc<dyn Model + Send + Sync>,
    /// Which recommendation was selected.
    pub selected: SelectedModel,
    /// Estimated cost for the operation (if token estimates provided).
    pub estimated_cost: Option<f64>,
    /// The model recommendation that was used.
    pub recommendation: Option<ModelRecommendation>,
}

/// Model selector for intelligent model routing.
pub struct ModelSelector {
    /// Budget limit per operation in dollars.
    budget_limit: Option<f64>,
    /// Override priority (overrides agent's recommendation).
    priority_override: Option<ModelPriority>,
    /// Total cost tracked across all selections.
    total_cost: f64,
    /// Total budget limit across all operations.
    total_budget_limit: Option<f64>,
    /// Optional model cache for transparent caching.
    cache: Option<Arc<ModelCache>>,
}

impl Default for ModelSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelSelector {
    /// Create a new model selector with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            budget_limit: None,
            priority_override: None,
            total_cost: 0.0,
            total_budget_limit: None,
            cache: None,
        }
    }

    /// Set a budget limit per operation.
    ///
    /// # Arguments
    /// * `limit` - Maximum cost per operation in dollars
    #[must_use]
    pub fn with_budget_limit(mut self, limit: f64) -> Self {
        self.budget_limit = Some(limit);
        self
    }

    /// Set a total budget limit across all operations.
    ///
    /// # Arguments
    /// * `limit` - Maximum total cost in dollars
    #[must_use]
    pub fn with_total_budget_limit(mut self, limit: f64) -> Self {
        self.total_budget_limit = Some(limit);
        self
    }

    /// Override the priority from agent metadata.
    ///
    /// # Arguments
    /// * `priority` - Priority to use instead of agent's recommendation
    #[must_use]
    pub fn with_priority_override(mut self, priority: ModelPriority) -> Self {
        self.priority_override = Some(priority);
        self
    }

    /// Set the model cache for transparent caching.
    ///
    /// # Arguments
    /// * `cache` - The model cache instance
    #[must_use]
    pub fn with_cache(mut self, cache: Arc<ModelCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Select a model based on agent metadata and options.
    ///
    /// Selection algorithm:
    /// 1. Check if premium model requested and approved
    /// 2. Try primary model
    /// 3. Try fallback model (if available)
    /// 4. Fall back to mock model
    ///
    /// # Arguments
    /// * `options` - Selection options including agent metadata
    ///
    /// # Errors
    /// Returns `SelectionError` if no model can be created or budget is exceeded.
    pub fn select_model(
        &mut self,
        options: &SelectionOptions<'_>,
    ) -> Result<SelectionResult, SelectionError> {
        let metadata = options.agent_metadata;

        // Check if agent has recommended models
        let recommended = metadata.recommended_models.as_ref().ok_or_else(|| {
            SelectionError::InvalidConfiguration(
                "Agent metadata has no model recommendations".to_string(),
            )
        })?;

        debug!(
            agent = %metadata.name,
            primary_model = %recommended.primary.model,
            "Selecting model for agent"
        );

        // Try premium model if requested and approved
        if let Some(ref premium) = recommended.premium {
            if self.should_use_premium(premium, options) {
                if let Ok(result) = self.try_select(premium, SelectedModel::Premium, options) {
                    info!(
                        model = %premium.model,
                        engine = %premium.engine,
                        "Selected premium model"
                    );
                    return Ok(result);
                }
            }
        }

        // Track budget errors separately - they should not fall back to mock
        let mut budget_error: Option<SelectionError> = None;

        // Try primary model
        match self.try_select(&recommended.primary, SelectedModel::Primary, options) {
            Ok(result) => {
                info!(
                    model = %recommended.primary.model,
                    engine = %recommended.primary.engine,
                    "Selected primary model"
                );
                return Ok(result);
            }
            Err(e) => {
                // Save budget errors - they should not trigger fallback
                if matches!(e, SelectionError::BudgetExceeded(_, _)) {
                    budget_error = Some(e);
                } else {
                    warn!(
                        model = %recommended.primary.model,
                        error = %e,
                        "Primary model unavailable, trying fallback"
                    );
                }
            }
        }

        // If we have a budget error, return it immediately (don't fall back)
        if let Some(err) = budget_error {
            return Err(err);
        }

        // Try fallback model
        if let Some(ref fallback) = recommended.fallback {
            match self.try_select(fallback, SelectedModel::Fallback, options) {
                Ok(result) => {
                    info!(
                        model = %fallback.model,
                        engine = %fallback.engine,
                        "Selected fallback model"
                    );
                    return Ok(result);
                }
                Err(e) => {
                    // Save budget errors - they should not trigger fallback
                    if matches!(e, SelectionError::BudgetExceeded(_, _)) {
                        return Err(e);
                    }
                    warn!(
                        model = %fallback.model,
                        error = %e,
                        "Fallback model unavailable, using mock"
                    );
                }
            }
        }

        // Fall back to mock model (only for non-budget failures)
        warn!("All recommended models unavailable, using mock model");
        Self::create_mock_model(metadata)
    }

    /// Check if premium model should be used.
    fn should_use_premium(
        &self,
        premium: &ModelRecommendation,
        options: &SelectionOptions<'_>,
    ) -> bool {
        // Check if approval is required
        if premium.requires_approval.unwrap_or(true) && !options.allow_premium_without_approval {
            return false;
        }

        // Check if priority override matches premium priority
        if let Some(ref override_priority) = self.priority_override {
            return override_priority == &premium.priority;
        }

        false
    }

    /// Try to select a specific model recommendation.
    fn try_select(
        &mut self,
        recommendation: &ModelRecommendation,
        selected: SelectedModel,
        options: &SelectionOptions<'_>,
    ) -> Result<SelectionResult, SelectionError> {
        // Check budget before creating model
        if let Some(estimated_cost) = Self::estimate_cost(recommendation, options) {
            self.check_budget(estimated_cost)?;
        }

        // Try to create the model
        let model = self.create_model(recommendation)?;

        // Calculate estimated cost
        let estimated_cost = Self::estimate_cost(recommendation, options);

        // Update total cost
        if let Some(cost) = estimated_cost {
            self.total_cost += cost;
            debug!(operation_cost = cost, total_cost = self.total_cost, "Updated cost tracking");
        }

        Ok(SelectionResult {
            model,
            selected,
            estimated_cost,
            recommendation: Some(recommendation.clone()),
        })
    }

    /// Create a model from a recommendation.
    fn create_model(
        &self,
        recommendation: &ModelRecommendation,
    ) -> Result<Arc<dyn Model + Send + Sync>, ModelError> {
        // Parse model type from engine string
        let model_type = ModelType::from_str(&recommendation.engine).map_err(|()| {
            ModelError::UnsupportedModelProvider(format!(
                "Unknown model engine: {}",
                recommendation.engine
            ))
        })?;

        // Create model config
        let config = ModelConfig::new(model_type, recommendation.model.clone());

        // Use cache if present and enabled, otherwise use factory directly
        if let Some(ref cache) = self.cache {
            if cache.config().enabled {
                return cache.get_or_create(config);
            }
        }

        // Fall back to direct factory creation
        ModelFactory::create(config)
    }

    /// Create a mock model as fallback.
    ///
    /// Mock models always bypass the cache to ensure fresh instances.
    fn create_mock_model(metadata: &AgentMetadata) -> Result<SelectionResult, SelectionError> {
        let config = ModelConfig::new(ModelType::Mock, format!("mock-{}", metadata.name));

        // Always create new mock model (bypass cache)
        let model = ModelFactory::create(config)?;

        Ok(SelectionResult {
            model,
            selected: SelectedModel::Mock,
            estimated_cost: Some(0.0), // Mock is free
            recommendation: None,
        })
    }

    /// Estimate cost for a model operation.
    ///
    /// Returns `None` if token estimates are not provided.
    fn estimate_cost(
        recommendation: &ModelRecommendation,
        options: &SelectionOptions<'_>,
    ) -> Option<f64> {
        let prompt_tokens = options.estimated_prompt_tokens?;
        let completion_tokens = options.estimated_completion_tokens?;

        // Cost per million tokens based on tier
        let cost_per_million = match recommendation.cost_tier {
            CostTier::Low => 0.05,     // $0.00 - $0.10 per 1M tokens
            CostTier::Medium => 0.50,  // $0.10 - $1.00 per 1M tokens
            CostTier::High => 5.0,     // $1.00 - $10.00 per 1M tokens
            CostTier::Premium => 50.0, // $10.00+ per 1M tokens
        };

        let total_tokens = prompt_tokens + completion_tokens;
        let cost = (f64::from(total_tokens) / 1_000_000.0) * cost_per_million;

        Some(cost)
    }

    /// Check if estimated cost is within budget.
    fn check_budget(&self, estimated_cost: f64) -> Result<(), SelectionError> {
        // Check per-operation budget
        if let Some(limit) = self.budget_limit {
            if estimated_cost > limit {
                return Err(SelectionError::BudgetExceeded(estimated_cost, limit));
            }
        }

        // Check total budget
        if let Some(total_limit) = self.total_budget_limit {
            let projected_total = self.total_cost + estimated_cost;
            if projected_total > total_limit {
                return Err(SelectionError::BudgetExceeded(projected_total, total_limit));
            }
        }

        Ok(())
    }

    /// Get the total cost tracked across all operations.
    #[must_use]
    pub fn get_total_cost(&self) -> f64 {
        self.total_cost
    }

    /// Reset cost tracking.
    pub fn reset_cost_tracking(&mut self) {
        self.total_cost = 0.0;
        debug!("Reset cost tracking");
    }

    /// Get the current per-operation budget limit.
    #[must_use]
    pub fn get_budget_limit(&self) -> Option<f64> {
        self.budget_limit
    }

    /// Get the current total budget limit.
    #[must_use]
    pub fn get_total_budget_limit(&self) -> Option<f64> {
        self.total_budget_limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::metadata::RecommendedModels;

    fn create_test_metadata() -> AgentMetadata {
        AgentMetadata {
            name: "test-agent".to_string(),
            display_name: Some("Test Agent".to_string()),
            category: Some("test".to_string()),
            color: "blue".to_string(),
            summary: Some("Test agent".to_string()),
            description: "Test agent description".to_string(),
            recommended_models: Some(RecommendedModels {
                primary: ModelRecommendation {
                    engine: "mock".to_string(),
                    model: "mock-primary".to_string(),
                    reasoning: "Fast and free".to_string(),
                    priority: ModelPriority::Speed,
                    cost_tier: CostTier::Low,
                    requires_approval: None,
                },
                fallback: Some(ModelRecommendation {
                    engine: "mock".to_string(),
                    model: "mock-fallback".to_string(),
                    reasoning: "Backup model".to_string(),
                    priority: ModelPriority::Balanced,
                    cost_tier: CostTier::Low,
                    requires_approval: None,
                }),
                premium: None,
            }),
            capabilities: None,
            performance_profile: None,
            quality_gates: None,
            works_well_with: None,
            typical_workflows: None,
            tools: None,
            constraints: None,
        }
    }

    #[test]
    fn test_selector_creation() {
        let selector = ModelSelector::new();
        assert_eq!(selector.get_total_cost(), 0.0);
        assert_eq!(selector.get_budget_limit(), None);
    }

    #[test]
    fn test_selector_with_budget() {
        let selector = ModelSelector::new().with_budget_limit(1.0);
        assert_eq!(selector.get_budget_limit(), Some(1.0));
    }

    #[test]
    fn test_selector_with_total_budget() {
        let selector = ModelSelector::new().with_total_budget_limit(10.0);
        assert_eq!(selector.get_total_budget_limit(), Some(10.0));
    }

    #[test]
    fn test_select_primary_model() {
        let mut selector = ModelSelector::new();
        let metadata = create_test_metadata();
        let options = SelectionOptions::new(&metadata);

        let result = selector.select_model(&options).unwrap();
        assert_eq!(result.selected, SelectedModel::Primary);
        assert_eq!(result.model.model_id(), "mock-primary");
    }

    #[test]
    fn test_cost_estimation() {
        let mut selector = ModelSelector::new();
        let metadata = create_test_metadata();
        let options = SelectionOptions::new(&metadata).with_token_estimate(1000, 500);

        let result = selector.select_model(&options).unwrap();
        assert!(result.estimated_cost.is_some());
        assert!(result.estimated_cost.unwrap() > 0.0);
    }

    #[test]
    fn test_cost_tracking() {
        let mut selector = ModelSelector::new();
        let metadata = create_test_metadata();

        // First selection
        let options1 = SelectionOptions::new(&metadata).with_token_estimate(1000, 500);
        selector.select_model(&options1).unwrap();
        let cost1 = selector.get_total_cost();
        assert!(cost1 > 0.0);

        // Second selection
        let options2 = SelectionOptions::new(&metadata).with_token_estimate(1000, 500);
        selector.select_model(&options2).unwrap();
        let cost2 = selector.get_total_cost();
        assert!(cost2 > cost1);

        // Reset
        selector.reset_cost_tracking();
        assert_eq!(selector.get_total_cost(), 0.0);
    }

    #[test]
    fn test_selected_model_display() {
        assert_eq!(SelectedModel::Primary.to_string(), "primary");
        assert_eq!(SelectedModel::Fallback.to_string(), "fallback");
        assert_eq!(SelectedModel::Premium.to_string(), "premium");
        assert_eq!(SelectedModel::Mock.to_string(), "mock");
    }

    #[test]
    fn test_selector_with_cache() {
        use radium_models::{CacheConfig, ModelCache};

        let cache = Arc::new(ModelCache::new(CacheConfig::default()).unwrap());
        let mut selector = ModelSelector::new().with_cache(Arc::clone(&cache));
        let metadata = create_test_metadata();
        let options = SelectionOptions::new(&metadata);

        // First selection - cache miss
        let result1 = selector.select_model(&options).unwrap();
        let stats1 = cache.get_stats();
        assert_eq!(stats1.total_misses, 1);
        assert_eq!(stats1.total_hits, 0);

        // Second selection - cache hit
        let result2 = selector.select_model(&options).unwrap();
        let stats2 = cache.get_stats();
        assert_eq!(stats2.total_misses, 1);
        assert_eq!(stats2.total_hits, 1);

        // Should be the same model instance (cached)
        assert!(Arc::ptr_eq(&result1.model, &result2.model));
    }

    #[test]
    fn test_selector_without_cache() {
        let mut selector = ModelSelector::new();
        let metadata = create_test_metadata();
        let options = SelectionOptions::new(&metadata);

        // Should work without cache (falls back to ModelFactory)
        let result = selector.select_model(&options).unwrap();
        assert_eq!(result.selected, SelectedModel::Primary);
    }
}

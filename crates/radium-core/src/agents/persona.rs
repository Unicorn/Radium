//! Persona system for agent model recommendations.
//!
//! Provides enhanced agent metadata with model recommendations and performance profiles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Performance profile for model selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PerformanceProfile {
    /// Optimize for speed - fast models, lower cost.
    Speed,
    /// Balanced speed and quality.
    Balanced,
    /// Optimize for deep reasoning.
    Thinking,
    /// Expert-level reasoning, highest cost.
    Expert,
}

impl Default for PerformanceProfile {
    fn default() -> Self {
        Self::Balanced
    }
}

impl std::fmt::Display for PerformanceProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Speed => write!(f, "speed"),
            Self::Balanced => write!(f, "balanced"),
            Self::Thinking => write!(f, "thinking"),
            Self::Expert => write!(f, "expert"),
        }
    }
}

/// Model recommendation configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelRecommendation {
    /// Engine to use (e.g., "gemini", "openai").
    pub engine: String,
    /// Model ID (e.g., "gemini-2.0-flash-exp").
    pub model: String,
}

/// Model recommendations for an agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecommendedModels {
    /// Primary recommended model for most tasks.
    pub primary: ModelRecommendation,
    /// Fallback model when primary is unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<ModelRecommendation>,
    /// Premium model for critical or complex tasks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub premium: Option<ModelRecommendation>,
}

/// Performance configuration for an agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceConfig {
    /// Performance profile.
    pub profile: PerformanceProfile,
    /// Estimated token usage per execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_tokens: Option<u64>,
}

/// Persona configuration for an agent.
///
/// This extends agent configuration with model recommendations and performance profiles.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersonaConfig {
    /// Model recommendations with fallback chain.
    pub models: RecommendedModels,
    /// Performance configuration.
    pub performance: PerformanceConfig,
}

impl PersonaConfig {
    /// Creates a new persona config with primary model only.
    pub fn new(engine: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            models: RecommendedModels {
                primary: ModelRecommendation {
                    engine: engine.into(),
                    model: model.into(),
                },
                fallback: None,
                premium: None,
            },
            performance: PerformanceConfig {
                profile: PerformanceProfile::Balanced,
                estimated_tokens: None,
            },
        }
    }

    /// Creates a persona config with full model recommendations.
    pub fn with_models(
        primary: ModelRecommendation,
        fallback: Option<ModelRecommendation>,
        premium: Option<ModelRecommendation>,
    ) -> Self {
        Self {
            models: RecommendedModels {
                primary,
                fallback,
                premium,
            },
            performance: PerformanceConfig {
                profile: PerformanceProfile::Balanced,
                estimated_tokens: None,
            },
        }
    }

    /// Sets the performance profile.
    #[must_use]
    pub fn with_performance_profile(mut self, profile: PerformanceProfile) -> Self {
        self.performance.profile = profile;
        self
    }

    /// Sets the estimated token usage.
    #[must_use]
    pub fn with_estimated_tokens(mut self, tokens: u64) -> Self {
        self.performance.estimated_tokens = Some(tokens);
        self
    }
}

/// Model pricing information for cost estimation.
#[derive(Debug, Clone)]
pub struct ModelPricing {
    /// Input tokens cost per 1M tokens (USD).
    pub input_cost_per_million: f64,
    /// Output tokens cost per 1M tokens (USD).
    pub output_cost_per_million: f64,
}

impl ModelPricing {
    /// Creates new pricing information.
    pub fn new(input_cost: f64, output_cost: f64) -> Self {
        Self {
            input_cost_per_million: input_cost,
            output_cost_per_million: output_cost,
        }
    }

    /// Estimates cost for token usage.
    pub fn estimate_cost(&self, input_tokens: u64, output_tokens: u64) -> f64 {
        let input_cost = (input_tokens as f64 / 1_000_000.0) * self.input_cost_per_million;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * self.output_cost_per_million;
        input_cost + output_cost
    }
}

/// Default model pricing database.
///
/// This is a simplified pricing database. In production, this would be
/// loaded from a configuration file or API.
pub struct ModelPricingDB {
    pricing: HashMap<String, ModelPricing>,
}

impl ModelPricingDB {
    /// Creates a new pricing database with default values.
    pub fn new() -> Self {
        let mut pricing = HashMap::new();
        
        // Gemini pricing (approximate)
        pricing.insert("gemini-2.0-flash-exp".to_string(), ModelPricing::new(0.075, 0.30));
        pricing.insert("gemini-2.0-flash-thinking".to_string(), ModelPricing::new(0.20, 0.80));
        pricing.insert("gemini-1.5-pro".to_string(), ModelPricing::new(1.25, 5.00));
        
        // OpenAI pricing (approximate)
        pricing.insert("gpt-4o".to_string(), ModelPricing::new(2.50, 10.00));
        pricing.insert("gpt-4o-mini".to_string(), ModelPricing::new(0.15, 0.60));
        
        // Claude pricing (approximate)
        pricing.insert("claude-3-opus".to_string(), ModelPricing::new(15.00, 75.00));
        pricing.insert("claude-3-sonnet".to_string(), ModelPricing::new(3.00, 15.00));
        
        Self { pricing }
    }

    /// Gets pricing for a model, or returns default if not found.
    pub fn get_pricing(&self, model: &str) -> ModelPricing {
        self.pricing
            .get(model)
            .cloned()
            .unwrap_or_else(|| ModelPricing::new(1.0, 1.0)) // Default fallback
    }
}

impl Default for ModelPricingDB {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_config_creation() {
        let persona = PersonaConfig::new("gemini", "gemini-2.0-flash-exp");
        assert_eq!(persona.models.primary.engine, "gemini");
        assert_eq!(persona.models.primary.model, "gemini-2.0-flash-exp");
        assert_eq!(persona.performance.profile, PerformanceProfile::Balanced);
    }

    #[test]
    fn test_model_pricing_estimation() {
        let pricing = ModelPricing::new(1.0, 2.0);
        let cost = pricing.estimate_cost(1_000_000, 500_000);
        assert!((cost - 2.0).abs() < 0.01); // 1.0 + 1.0 = 2.0
    }
}


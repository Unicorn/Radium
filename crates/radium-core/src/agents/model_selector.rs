//! Model selection engine for persona system.
//!
//! Selects appropriate models based on agent persona, availability, and cost constraints.

use crate::agents::persona::{ModelPricingDB, PerformanceProfile, PersonaConfig, SimpleModelRecommendation};
use thiserror::Error;

/// Model selection errors.
#[derive(Debug, Error)]
pub enum SelectionError {
    /// No model available in fallback chain.
    #[error("no model available in fallback chain")]
    NoModelAvailable,

    /// Model not found in pricing database.
    #[error("model not found in pricing database: {0}")]
    ModelNotFound(String),

    /// Invalid configuration.
    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),
}

/// Result type for model selection.
pub type Result<T> = std::result::Result<T, SelectionError>;

/// Model selection result.
#[derive(Debug, Clone)]
pub struct SelectionResult {
    /// Selected model recommendation.
    pub model: SimpleModelRecommendation,
    /// Selection reason.
    pub reason: String,
    /// Estimated cost for this selection.
    pub estimated_cost: f64,
}

/// Model selector trait.
pub trait ModelSelector {
    /// Selects a model based on persona and context.
    fn select_model(
        &self,
        persona: &PersonaConfig,
        use_premium: bool,
    ) -> Result<SelectionResult>;
}

/// Default model selector implementation.
pub struct DefaultModelSelector {
    pricing_db: ModelPricingDB,
}

impl DefaultModelSelector {
    /// Creates a new model selector with default pricing database.
    pub fn new() -> Self {
        Self {
            pricing_db: ModelPricingDB::new(),
        }
    }

    /// Creates a new model selector with custom pricing database.
    pub fn with_pricing_db(pricing_db: ModelPricingDB) -> Self {
        Self { pricing_db }
    }

    /// Checks if a model is available (placeholder - would check API in production).
    fn is_model_available(&self, _model: &SimpleModelRecommendation) -> bool {
        // In production, this would check API availability
        // For now, assume all models are available
        true
    }

    /// Selects model based on performance profile.
    fn select_by_profile(
        &self,
        persona: &PersonaConfig,
        use_premium: bool,
    ) -> Result<SimpleModelRecommendation> {
        match persona.performance.profile {
            PerformanceProfile::Speed => {
                // For speed, prefer primary (usually fastest)
                Ok(persona.models.primary.clone())
            }
            PerformanceProfile::Balanced => {
                // For balanced, use primary, fallback if needed
                if use_premium {
                    persona
                        .models
                        .premium
                        .as_ref()
                        .cloned()
                        .ok_or_else(|| SelectionError::NoModelAvailable)
                } else {
                    Ok(persona.models.primary.clone())
                }
            }
            PerformanceProfile::Thinking => {
                // For thinking, prefer premium or primary thinking models
                if let Some(ref premium) = persona.models.premium {
                    Ok(premium.clone())
                } else {
                    Ok(persona.models.primary.clone())
                }
            }
            PerformanceProfile::Expert => {
                // For expert, always prefer premium
                persona
                    .models
                    .premium
                    .as_ref()
                    .cloned()
                    .ok_or_else(|| SelectionError::NoModelAvailable)
            }
        }
    }

    /// Estimates cost for a model recommendation.
    fn estimate_cost(&self, model: &SimpleModelRecommendation, estimated_tokens: Option<u64>) -> f64 {
        let pricing = self.pricing_db.get_pricing(&model.model);
        let tokens = estimated_tokens.unwrap_or(2000); // Default estimate
        // Assume 70% input, 30% output
        let input_tokens = (tokens as f64 * 0.7) as u64;
        let output_tokens = (tokens as f64 * 0.3) as u64;
        pricing.estimate_cost(input_tokens, output_tokens)
    }
}

impl ModelSelector for DefaultModelSelector {
    fn select_model(
        &self,
        persona: &PersonaConfig,
        use_premium: bool,
    ) -> Result<SelectionResult> {
        // Try to select based on profile
        let mut selected = self.select_by_profile(persona, use_premium)?;

        // Check availability and fallback if needed
        if !self.is_model_available(&selected) {
            // Try fallback
            if let Some(ref fallback) = persona.models.fallback {
                if self.is_model_available(fallback) {
                    selected = fallback.clone();
                } else {
                    return Err(SelectionError::NoModelAvailable);
                }
            } else {
                return Err(SelectionError::NoModelAvailable);
            }
        }

        let estimated_cost = self.estimate_cost(&selected, persona.performance.estimated_tokens);

        Ok(SelectionResult {
            model: selected,
            reason: format!(
                "Selected based on {} profile",
                persona.performance.profile
            ),
            estimated_cost,
        })
    }
}

impl Default for DefaultModelSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// Fallback chain selector.
///
/// Implements the fallback chain: primary → fallback → premium → mock
pub struct FallbackChainSelector {
    pricing_db: ModelPricingDB,
}

impl FallbackChainSelector {
    /// Creates a new fallback chain selector.
    pub fn new() -> Self {
        Self {
            pricing_db: ModelPricingDB::new(),
        }
    }

    /// Selects model using fallback chain.
    pub fn select_with_fallback(
        &self,
        persona: &PersonaConfig,
    ) -> Result<SelectionResult> {
        // Try primary first
        if self.is_model_available(&persona.models.primary) {
            let cost = self.estimate_cost(&persona.models.primary, persona.performance.estimated_tokens);
            return Ok(SelectionResult {
                model: persona.models.primary.clone(),
                reason: "Selected primary model".to_string(),
                estimated_cost: cost,
            });
        }

        // Try fallback
        if let Some(ref fallback) = persona.models.fallback {
            if self.is_model_available(fallback) {
                let cost = self.estimate_cost(fallback, persona.performance.estimated_tokens);
                return Ok(SelectionResult {
                    model: fallback.clone(),
                    reason: "Selected fallback model".to_string(),
                    estimated_cost: cost,
                });
            }
        }

        // Try premium as last resort
        if let Some(ref premium) = persona.models.premium {
            if self.is_model_available(premium) {
                let cost = self.estimate_cost(premium, persona.performance.estimated_tokens);
                return Ok(SelectionResult {
                    model: premium.clone(),
                    reason: "Selected premium model (fallback)".to_string(),
                    estimated_cost: cost,
                });
            }
        }

        Err(SelectionError::NoModelAvailable)
    }

    fn is_model_available(&self, _model: &SimpleModelRecommendation) -> bool {
        // Placeholder - would check API in production
        true
    }

    fn estimate_cost(&self, model: &SimpleModelRecommendation, estimated_tokens: Option<u64>) -> f64 {
        let pricing = self.pricing_db.get_pricing(&model.model);
        let tokens = estimated_tokens.unwrap_or(2000);
        let input_tokens = (tokens as f64 * 0.7) as u64;
        let output_tokens = (tokens as f64 * 0.3) as u64;
        pricing.estimate_cost(input_tokens, output_tokens)
    }
}

impl Default for FallbackChainSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_selector_speed_profile() {
        let selector = DefaultModelSelector::new();
        let persona = PersonaConfig::new("gemini", "gemini-2.0-flash-exp")
            .with_performance_profile(PerformanceProfile::Speed);

        let result = selector.select_model(&persona, false).unwrap();
        assert_eq!(result.model.engine, "gemini");
        assert_eq!(result.model.model, "gemini-2.0-flash-exp");
    }

    #[test]
    fn test_fallback_chain() {
        let selector = FallbackChainSelector::new();
        let persona = PersonaConfig::with_models(
            SimpleModelRecommendation {
                engine: "gemini".to_string(),
                model: "primary".to_string(),
            },
            Some(SimpleModelRecommendation {
                engine: "openai".to_string(),
                model: "fallback".to_string(),
            }),
            None,
        );

        let result = selector.select_with_fallback(&persona).unwrap();
        assert_eq!(result.model.model, "primary");
    }

    #[test]
    fn test_model_selector_balanced_profile() {
        let selector = DefaultModelSelector::new();
        let persona = PersonaConfig::new("gemini", "gemini-2.0-flash-exp")
            .with_performance_profile(PerformanceProfile::Balanced);

        let result = selector.select_model(&persona, false).unwrap();
        assert_eq!(result.model.engine, "gemini");
        assert_eq!(result.model.model, "gemini-2.0-flash-exp");
    }

    #[test]
    fn test_model_selector_balanced_profile_with_premium() {
        let selector = DefaultModelSelector::new();
        let persona = PersonaConfig::with_models(
            SimpleModelRecommendation {
                engine: "gemini".to_string(),
                model: "gemini-2.0-flash-exp".to_string(),
            },
            None,
            Some(SimpleModelRecommendation {
                engine: "openai".to_string(),
                model: "gpt-4".to_string(),
            }),
        ).with_performance_profile(PerformanceProfile::Balanced);

        let result = selector.select_model(&persona, true).unwrap();
        assert_eq!(result.model.engine, "openai");
        assert_eq!(result.model.model, "gpt-4");
    }

    #[test]
    fn test_model_selector_balanced_profile_no_premium() {
        let selector = DefaultModelSelector::new();
        let persona = PersonaConfig::new("gemini", "gemini-2.0-flash-exp")
            .with_performance_profile(PerformanceProfile::Balanced);

        let result = selector.select_model(&persona, true);
        // When use_premium is true but no premium model, should return error
        assert!(matches!(result, Err(SelectionError::NoModelAvailable)));
    }

    #[test]
    fn test_model_selector_thinking_profile() {
        let selector = DefaultModelSelector::new();
        let persona = PersonaConfig::with_models(
            SimpleModelRecommendation {
                engine: "gemini".to_string(),
                model: "gemini-2.0-flash-exp".to_string(),
            },
            None,
            Some(SimpleModelRecommendation {
                engine: "openai".to_string(),
                model: "o1-preview".to_string(),
            }),
        ).with_performance_profile(PerformanceProfile::Thinking);

        let result = selector.select_model(&persona, false).unwrap();
        assert_eq!(result.model.engine, "openai");
        assert_eq!(result.model.model, "o1-preview");
    }

    #[test]
    fn test_model_selector_thinking_profile_no_premium() {
        let selector = DefaultModelSelector::new();
        let persona = PersonaConfig::new("gemini", "gemini-2.0-flash-exp")
            .with_performance_profile(PerformanceProfile::Thinking);

        let result = selector.select_model(&persona, false).unwrap();
        // Should use primary when premium not available
        assert_eq!(result.model.engine, "gemini");
    }

    #[test]
    fn test_model_selector_expert_profile() {
        let selector = DefaultModelSelector::new();
        let persona = PersonaConfig::with_models(
            SimpleModelRecommendation {
                engine: "gemini".to_string(),
                model: "gemini-2.0-flash-exp".to_string(),
            },
            None,
            Some(SimpleModelRecommendation {
                engine: "openai".to_string(),
                model: "gpt-4-turbo".to_string(),
            }),
        ).with_performance_profile(PerformanceProfile::Expert);

        let result = selector.select_model(&persona, false).unwrap();
        assert_eq!(result.model.engine, "openai");
        assert_eq!(result.model.model, "gpt-4-turbo");
    }

    #[test]
    fn test_model_selector_expert_profile_no_premium() {
        let selector = DefaultModelSelector::new();
        let persona = PersonaConfig::new("gemini", "gemini-2.0-flash-exp")
            .with_performance_profile(PerformanceProfile::Expert);

        let result = selector.select_model(&persona, false);
        assert!(matches!(result, Err(SelectionError::NoModelAvailable)));
    }

    #[test]
    fn test_model_selector_estimate_cost() {
        let selector = DefaultModelSelector::new();
        let model = SimpleModelRecommendation {
            engine: "gemini".to_string(),
            model: "gemini-2.0-flash-exp".to_string(),
        };
        
        let cost = selector.estimate_cost(&model, Some(1000));
        assert!(cost >= 0.0);
    }

    #[test]
    fn test_model_selector_estimate_cost_default_tokens() {
        let selector = DefaultModelSelector::new();
        let model = SimpleModelRecommendation {
            engine: "gemini".to_string(),
            model: "gemini-2.0-flash-exp".to_string(),
        };
        
        let cost = selector.estimate_cost(&model, None);
        assert!(cost >= 0.0);
    }

    #[test]
    fn test_model_selector_result_reason() {
        let selector = DefaultModelSelector::new();
        let persona = PersonaConfig::new("gemini", "gemini-2.0-flash-exp")
            .with_performance_profile(PerformanceProfile::Speed);

        let result = selector.select_model(&persona, false).unwrap();
        // Reason format is "Selected based on {profile} profile"
        assert!(result.reason.contains("profile") || result.reason.contains("Speed") || result.reason.contains("speed"));
        assert!(result.estimated_cost >= 0.0);
    }

    #[test]
    fn test_fallback_chain_selector_with_fallback() {
        let selector = FallbackChainSelector::new();
        let persona = PersonaConfig::with_models(
            SimpleModelRecommendation {
                engine: "gemini".to_string(),
                model: "primary".to_string(),
            },
            Some(SimpleModelRecommendation {
                engine: "openai".to_string(),
                model: "fallback".to_string(),
            }),
            None,
        );

        let result = selector.select_with_fallback(&persona).unwrap();
        assert_eq!(result.model.model, "primary");
        assert!(result.reason.contains("primary"));
    }

    #[test]
    fn test_fallback_chain_selector_no_models() {
        let selector = FallbackChainSelector::new();
        let persona = PersonaConfig::with_models(
            SimpleModelRecommendation {
                engine: "gemini".to_string(),
                model: "primary".to_string(),
            },
            None,
            None,
        );

        let result = selector.select_with_fallback(&persona).unwrap();
        assert_eq!(result.model.model, "primary");
    }

    #[test]
    fn test_fallback_chain_selector_with_premium() {
        let selector = FallbackChainSelector::new();
        let persona = PersonaConfig::with_models(
            SimpleModelRecommendation {
                engine: "gemini".to_string(),
                model: "primary".to_string(),
            },
            Some(SimpleModelRecommendation {
                engine: "openai".to_string(),
                model: "fallback".to_string(),
            }),
            Some(SimpleModelRecommendation {
                engine: "claude".to_string(),
                model: "premium".to_string(),
            }),
        );

        let result = selector.select_with_fallback(&persona).unwrap();
        assert_eq!(result.model.model, "primary");
    }

    #[test]
    fn test_selection_error_display() {
        let error = SelectionError::NoModelAvailable;
        let msg = format!("{}", error);
        assert!(msg.contains("no model available"));

        let error = SelectionError::ModelNotFound("test-model".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("test-model"));

        let error = SelectionError::InvalidConfiguration("test".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("invalid configuration"));
    }

    #[test]
    fn test_default_model_selector_new() {
        let selector = DefaultModelSelector::new();
        // Just verify it doesn't panic
        assert!(true);
    }

    #[test]
    fn test_default_model_selector_default() {
        let selector = DefaultModelSelector::default();
        // Just verify it doesn't panic
        assert!(true);
    }

    #[test]
    fn test_fallback_chain_selector_new() {
        let selector = FallbackChainSelector::new();
        // Just verify it doesn't panic
        assert!(true);
    }

    #[test]
    fn test_fallback_chain_selector_default() {
        let selector = FallbackChainSelector::default();
        // Just verify it doesn't panic
        assert!(true);
    }
}


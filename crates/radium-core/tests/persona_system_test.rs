//! Tests for persona system with model recommendations and fallback chains.

use radium_core::agents::model_selector::{DefaultModelSelector, FallbackChainSelector, ModelSelector};
use radium_core::agents::persona::{
    PerformanceProfile, PersonaConfig, SimpleModelRecommendation,
};

#[test]
fn test_model_selection_speed_profile() {
    let selector = DefaultModelSelector::new();
    let persona = PersonaConfig::new("gemini", "gemini-2.0-flash-exp")
        .with_performance_profile(PerformanceProfile::Speed);

    let result = selector.select_model(&persona, false).unwrap();
    assert_eq!(result.model.engine, "gemini");
    assert_eq!(result.model.model, "gemini-2.0-flash-exp");
    assert!(result.estimated_cost > 0.0);
}

#[test]
fn test_model_selection_expert_profile() {
    let selector = DefaultModelSelector::new();
    let persona = PersonaConfig::with_models(
        SimpleModelRecommendation {
            engine: "gemini".to_string(),
            model: "gemini-2.0-flash-exp".to_string(),
        },
        None,
        Some(SimpleModelRecommendation {
            engine: "gemini".to_string(),
            model: "gemini-1.5-pro".to_string(),
        }),
    )
    .with_performance_profile(PerformanceProfile::Expert);

    let result = selector.select_model(&persona, false).unwrap();
    // Expert profile should prefer premium
    assert_eq!(result.model.model, "gemini-1.5-pro");
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
    assert_eq!(result.reason, "Selected primary model");
}

#[test]
fn test_cost_estimation() {
    let selector = DefaultModelSelector::new();
    let persona = PersonaConfig::new("gemini", "gemini-2.0-flash-exp")
        .with_estimated_tokens(10000);

    let result = selector.select_model(&persona, false).unwrap();
    // Cost should be estimated based on tokens
    assert!(result.estimated_cost > 0.0);
}


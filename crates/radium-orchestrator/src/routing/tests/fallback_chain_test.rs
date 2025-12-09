//! Tests for multi-model fallback chain system.

use super::super::router::ModelRouter;
use super::super::types::{FallbackChain, RoutingTier};
use radium_models::{ModelConfig, ModelType};

fn create_test_router() -> ModelRouter {
    let smart_config = ModelConfig::new(ModelType::Mock, "smart-model".to_string());
    let eco_config = ModelConfig::new(ModelType::Mock, "eco-model".to_string());
    ModelRouter::new(smart_config, eco_config, Some(60.0))
}

#[test]
fn test_fallback_chain_creation() {
    let chain = FallbackChain::new(vec![
        ModelConfig::new(ModelType::Mock, "model-1".to_string()),
        ModelConfig::new(ModelType::Mock, "model-2".to_string()),
        ModelConfig::new(ModelType::Mock, "model-3".to_string()),
    ]);
    
    assert_eq!(chain.len(), 3);
    assert!(!chain.is_empty());
}

#[test]
fn test_fallback_chain_with_retries() {
    let chain = FallbackChain::with_retries(
        vec![
            ModelConfig::new(ModelType::Mock, "model-1".to_string()),
            ModelConfig::new(ModelType::Mock, "model-2".to_string()),
        ],
        3,
    );
    
    assert_eq!(chain.max_retries_per_model, 3);
    assert_eq!(chain.len(), 2);
}

#[test]
fn test_router_with_fallback_chain() {
    let chain = FallbackChain::new(vec![
        ModelConfig::new(ModelType::Mock, "fallback-1".to_string()),
        ModelConfig::new(ModelType::Mock, "fallback-2".to_string()),
    ]);
    
    let router = create_test_router().with_fallback_chain(chain);
    
    // Router should still work normally
    let (model, decision) = router.select_model("test input", None, None);
    assert!(!model.model_id.is_empty());
    assert_eq!(decision.decision_type, super::super::router::DecisionType::Auto);
}

#[test]
fn test_get_next_fallback_model_success() {
    let chain = FallbackChain::new(vec![
        ModelConfig::new(ModelType::Mock, "fallback-1".to_string()),
        ModelConfig::new(ModelType::Mock, "fallback-2".to_string()),
        ModelConfig::new(ModelType::Mock, "fallback-3".to_string()),
    ]);
    
    let router = create_test_router().with_fallback_chain(chain);
    
    // First failure should return first fallback model
    let next = router.get_next_fallback_model("primary-model", "Rate limit error")
        .unwrap();
    assert!(next.is_some());
    assert_eq!(next.unwrap().model_id, "fallback-1");
    
    // Second failure should return second fallback model
    let next = router.get_next_fallback_model("fallback-1", "Timeout error")
        .unwrap();
    assert!(next.is_some());
    assert_eq!(next.unwrap().model_id, "fallback-2");
}

#[test]
fn test_get_next_fallback_model_no_chain() {
    let router = create_test_router();
    // No fallback chain configured
    
    let next = router.get_next_fallback_model("failed-model", "Error")
        .unwrap();
    assert!(next.is_none());
}

#[test]
fn test_get_next_fallback_model_all_failed() {
    let chain = FallbackChain::new(vec![
        ModelConfig::new(ModelType::Mock, "fallback-1".to_string()),
        ModelConfig::new(ModelType::Mock, "fallback-2".to_string()),
    ]);
    
    let router = create_test_router().with_fallback_chain(chain);
    
    // Try first fallback
    let _ = router.get_next_fallback_model("primary-model", "Error 1").unwrap();
    
    // Try second fallback
    let _ = router.get_next_fallback_model("fallback-1", "Error 2").unwrap();
    
    // Try third fallback (should fail - all models exhausted)
    let result = router.get_next_fallback_model("fallback-2", "Error 3");
    
    assert!(result.is_err());
    if let Err(super::super::types::RoutingError::AllModelsFailed(failures)) = result {
        // Should have failure records for all attempted models
        assert!(failures.len() >= 2);
    } else {
        panic!("Expected AllModelsFailed error");
    }
}

#[test]
fn test_reset_fallback_state() {
    let chain = FallbackChain::new(vec![
        ModelConfig::new(ModelType::Mock, "fallback-1".to_string()),
    ]);
    
    let router = create_test_router().with_fallback_chain(chain);
    
    // Use a fallback model
    let _ = router.get_next_fallback_model("primary-model", "Error").unwrap();
    
    // Reset state
    router.reset_fallback_state();
    
    // Should be able to use fallback again after reset
    let next = router.get_next_fallback_model("primary-model-2", "Error 2")
        .unwrap();
    assert!(next.is_some());
    assert_eq!(next.unwrap().model_id, "fallback-1");
}

#[test]
fn test_fallback_chain_with_manual_override() {
    let chain = FallbackChain::new(vec![
        ModelConfig::new(ModelType::Mock, "fallback-1".to_string()),
    ]);
    
    let router = create_test_router().with_fallback_chain(chain);
    
    // Manual override should bypass fallback chain
    let (model, decision) = router.select_model(
        "test",
        None,
        Some(RoutingTier::Smart),
    );
    
    assert_eq!(model.model_id, "smart-model");
    assert_eq!(decision.decision_type, super::super::router::DecisionType::Manual);
}


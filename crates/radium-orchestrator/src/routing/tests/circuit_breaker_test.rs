//! Tests for circuit breaker pattern integration.

use super::super::circuit_breaker::{CircuitBreaker, CircuitState};
use super::super::router::ModelRouter;
use super::super::types::FallbackChain;
use radium_models::{ModelConfig, ModelType};
use std::time::Duration;

fn create_test_router() -> ModelRouter {
    let smart_config = ModelConfig::new(ModelType::Mock, "smart-model".to_string());
    let eco_config = ModelConfig::new(ModelType::Mock, "eco-model".to_string());
    ModelRouter::new(smart_config, eco_config, Some(60.0))
}

#[test]
fn test_circuit_breaker_integration() {
    let breaker = CircuitBreaker::with_settings(
        0.5, // 50% threshold
        Duration::from_secs(300),
        Duration::from_secs(60),
    );
    
    let chain = FallbackChain::new(vec![
        ModelConfig::new(ModelType::Mock, "model-1".to_string()),
        ModelConfig::new(ModelType::Mock, "model-2".to_string()),
    ]);
    
    let router = create_test_router()
        .with_fallback_chain(chain)
        .with_circuit_breaker(breaker);
    
    // Router should work normally
    let (model, _) = router.select_model("test", None, None);
    assert!(!model.model_id.is_empty());
}

#[test]
fn test_circuit_breaker_skips_open_circuit() {
    let breaker = CircuitBreaker::with_settings(
        0.5,
        Duration::from_secs(300),
        Duration::from_secs(60),
    );
    
    let chain = FallbackChain::new(vec![
        ModelConfig::new(ModelType::Mock, "failing-model".to_string()),
        ModelConfig::new(ModelType::Mock, "working-model".to_string()),
    ]);
    
    let router = create_test_router()
        .with_fallback_chain(chain)
        .with_circuit_breaker(breaker);
    
    // Open the circuit for the first model
    router.record_model_failure("failing-model");
    router.record_model_failure("failing-model");
    router.record_model_failure("failing-model");
    router.record_model_failure("failing-model");
    router.record_model_failure("failing-model");
    router.record_model_failure("failing-model");
    // 6 failures, 0 successes = 100% failure rate
    
    // Get next fallback model - should skip the failing model
    let next = router.get_next_fallback_model("primary-model", "Error")
        .unwrap();
    
    // Should get the second model, not the first (which has open circuit)
    assert!(next.is_some());
    let model = next.unwrap();
    assert_eq!(model.model_id, "working-model");
}

#[test]
fn test_record_model_success() {
    let breaker = CircuitBreaker::new();
    let router = create_test_router().with_circuit_breaker(breaker);
    
    // Record success
    router.record_model_success("test-model");
    
    // Circuit should remain closed
    assert!(!router.circuit_breaker.as_ref().unwrap().should_skip("test-model"));
}

#[test]
fn test_record_model_failure() {
    let breaker = CircuitBreaker::with_settings(
        0.5,
        Duration::from_secs(300),
        Duration::from_secs(60),
    );
    
    let router = create_test_router().with_circuit_breaker(breaker);
    
    // Record multiple failures to open circuit
    for _ in 0..6 {
        router.record_model_failure("test-model");
    }
    
    // Circuit should be open
    assert!(router.circuit_breaker.as_ref().unwrap().should_skip("test-model"));
}


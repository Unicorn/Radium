//! Tests for advanced routing strategies.

use super::super::router::ModelRouter;
use super::super::types::{RoutingStrategy, RoutingTier};
use radium_models::{ModelConfig, ModelType};

fn create_test_router() -> ModelRouter {
    // Use models that exist in the registry for strategy testing
    let smart_config = ModelConfig::new(ModelType::Claude, "claude-sonnet-4.5".to_string());
    let eco_config = ModelConfig::new(ModelType::Claude, "claude-haiku-4.5".to_string());
    ModelRouter::new(smart_config, eco_config, Some(60.0))
}

#[test]
fn test_complexity_based_strategy() {
    let router = create_test_router();
    
    // Simple task should route to eco
    let (model, decision) = router.select_model_with_strategy(
        "format this JSON",
        None,
        None,
        RoutingStrategy::ComplexityBased,
    );
    
    assert_eq!(model.model_id, "claude-haiku-4.5");
    assert_eq!(decision.tier, RoutingTier::Eco);
}

#[test]
fn test_cost_optimized_strategy() {
    let router = create_test_router();
    
    // Cost-optimized should select cheaper model for low complexity
    let (model, decision) = router.select_model_with_strategy(
        "simple task",
        None,
        None,
        RoutingStrategy::CostOptimized,
    );
    
    // Haiku is cheaper than Sonnet, so should be selected for low complexity
    assert_eq!(model.model_id, "claude-haiku-4.5");
    assert_eq!(decision.tier, RoutingTier::Eco);
}

#[test]
fn test_latency_optimized_strategy() {
    let router = create_test_router();
    
    // Latency-optimized should select faster model for low complexity
    let (model, decision) = router.select_model_with_strategy(
        "simple task",
        None,
        None,
        RoutingStrategy::LatencyOptimized,
    );
    
    // Haiku is faster than Sonnet, so should be selected for low complexity
    assert_eq!(model.model_id, "claude-haiku-4.5");
    assert_eq!(decision.tier, RoutingTier::Eco);
}

#[test]
fn test_quality_optimized_strategy() {
    let router = create_test_router();
    
    // Quality-optimized should prefer higher quality even for low complexity
    let (model, decision) = router.select_model_with_strategy(
        "simple task",
        None,
        None,
        RoutingStrategy::QualityOptimized,
    );
    
    // Sonnet has higher quality tier (5) than Haiku (3), so should be preferred
    assert_eq!(model.model_id, "claude-sonnet-4.5");
    assert_eq!(decision.tier, RoutingTier::Smart);
}

#[test]
fn test_strategy_string_conversion() {
    assert_eq!(
        RoutingStrategy::from_str("complexity_based"),
        Some(RoutingStrategy::ComplexityBased)
    );
    assert_eq!(
        RoutingStrategy::from_str("cost_optimized"),
        Some(RoutingStrategy::CostOptimized)
    );
    assert_eq!(
        RoutingStrategy::from_str("latency_optimized"),
        Some(RoutingStrategy::LatencyOptimized)
    );
    assert_eq!(
        RoutingStrategy::from_str("quality_optimized"),
        Some(RoutingStrategy::QualityOptimized)
    );
    assert_eq!(RoutingStrategy::from_str("invalid"), None);
}

#[test]
fn test_strategy_to_string() {
    assert_eq!(
        RoutingStrategy::ComplexityBased.to_string(),
        "complexity_based"
    );
    assert_eq!(
        RoutingStrategy::CostOptimized.to_string(),
        "cost_optimized"
    );
    assert_eq!(
        RoutingStrategy::LatencyOptimized.to_string(),
        "latency_optimized"
    );
    assert_eq!(
        RoutingStrategy::QualityOptimized.to_string(),
        "quality_optimized"
    );
}

#[test]
fn test_default_strategy_is_complexity_based() {
    let router = create_test_router();
    
    // Default select_model should use ComplexityBased strategy
    let (model1, _) = router.select_model("test", None, None);
    let (model2, _) = router.select_model_with_strategy(
        "test",
        None,
        None,
        RoutingStrategy::ComplexityBased,
    );
    
    // Should produce same result
    assert_eq!(model1.model_id, model2.model_id);
}


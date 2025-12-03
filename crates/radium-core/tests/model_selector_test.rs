//! Integration tests for ModelSelector with real agent metadata.

use radium_core::agents::metadata::{
    AgentMetadata, CostTier, ModelPriority, ModelRecommendation, RecommendedModels,
};
use radium_core::models::{ModelSelector, SelectedModel, SelectionOptions};

fn create_speed_optimized_agent() -> AgentMetadata {
    AgentMetadata {
        name: "fast-agent".to_string(),
        display_name: Some("Fast Agent".to_string()),
        category: Some("testing".to_string()),
        color: "blue".to_string(),
        summary: Some("Speed-optimized test agent".to_string()),
        description: "An agent optimized for fast iteration".to_string(),
        recommended_models: Some(RecommendedModels {
            primary: ModelRecommendation {
                engine: "mock".to_string(),
                model: "mock-fast-model".to_string(),
                reasoning: "Fast iteration for testing".to_string(),
                priority: ModelPriority::Speed,
                cost_tier: CostTier::Low,
                requires_approval: None,
            },
            fallback: Some(ModelRecommendation {
                engine: "mock".to_string(),
                model: "mock-fallback-fast".to_string(),
                reasoning: "Fast fallback".to_string(),
                priority: ModelPriority::Speed,
                cost_tier: CostTier::Low,
                requires_approval: None,
            }),
            premium: None,
        }),
        capabilities: Some(vec!["fast_processing".to_string()]),
        performance_profile: None,
        quality_gates: None,
        works_well_with: None,
        typical_workflows: None,
        tools: None,
        constraints: None,
    }
}

fn create_thinking_agent() -> AgentMetadata {
    AgentMetadata {
        name: "thinking-agent".to_string(),
        display_name: Some("Thinking Agent".to_string()),
        category: Some("analysis".to_string()),
        color: "purple".to_string(),
        summary: Some("Deep thinking agent".to_string()),
        description: "An agent for deep analysis".to_string(),
        recommended_models: Some(RecommendedModels {
            primary: ModelRecommendation {
                engine: "mock".to_string(),
                model: "mock-thinking-model".to_string(),
                reasoning: "Deep reasoning capability".to_string(),
                priority: ModelPriority::Thinking,
                cost_tier: CostTier::High,
                requires_approval: None,
            },
            fallback: Some(ModelRecommendation {
                engine: "mock".to_string(),
                model: "mock-balanced-model".to_string(),
                reasoning: "Balanced fallback".to_string(),
                priority: ModelPriority::Balanced,
                cost_tier: CostTier::Medium,
                requires_approval: None,
            }),
            premium: Some(ModelRecommendation {
                engine: "mock".to_string(),
                model: "mock-expert-model".to_string(),
                reasoning: "Expert-level reasoning".to_string(),
                priority: ModelPriority::Expert,
                cost_tier: CostTier::Premium,
                requires_approval: Some(true),
            }),
        }),
        capabilities: Some(vec!["deep_analysis".to_string(), "reasoning".to_string()]),
        performance_profile: None,
        quality_gates: None,
        works_well_with: None,
        typical_workflows: None,
        tools: None,
        constraints: None,
    }
}

#[test]
fn test_select_speed_optimized_agent() {
    let mut selector = ModelSelector::new();
    let agent = create_speed_optimized_agent();
    let options = SelectionOptions::new(&agent);

    let result = selector.select_model(&options).expect("Should select model");

    assert_eq!(result.selected, SelectedModel::Primary);
    assert_eq!(result.model.model_id(), "mock-fast-model");
    assert!(result.recommendation.is_some());

    let recommendation = result.recommendation.unwrap();
    assert_eq!(recommendation.priority, ModelPriority::Speed);
    assert_eq!(recommendation.cost_tier, CostTier::Low);
}

#[test]
fn test_select_thinking_agent() {
    let mut selector = ModelSelector::new();
    let agent = create_thinking_agent();
    let options = SelectionOptions::new(&agent);

    let result = selector.select_model(&options).expect("Should select model");

    assert_eq!(result.selected, SelectedModel::Primary);
    assert_eq!(result.model.model_id(), "mock-thinking-model");

    let recommendation = result.recommendation.unwrap();
    assert_eq!(recommendation.priority, ModelPriority::Thinking);
    assert_eq!(recommendation.cost_tier, CostTier::High);
}

#[test]
fn test_cost_estimation_with_tokens() {
    let mut selector = ModelSelector::new();
    let agent = create_speed_optimized_agent();
    let options = SelectionOptions::new(&agent).with_token_estimate(1000, 500);

    let result = selector.select_model(&options).expect("Should select model");

    assert!(result.estimated_cost.is_some());
    let cost = result.estimated_cost.unwrap();
    // Low tier: ~$0.05 per 1M tokens, 1500 tokens = $0.000075
    assert!(cost > 0.0);
    assert!(cost < 0.01); // Should be very small for low tier
}

#[test]
fn test_budget_limit_enforcement() {
    let mut selector = ModelSelector::new().with_budget_limit(0.001); // Strict budget: $0.001
    let agent = create_thinking_agent(); // High cost tier: $5 per 1M tokens
    let options = SelectionOptions::new(&agent).with_token_estimate(1_000_000, 1_000_000); // 2M tokens = $10

    let result = selector.select_model(&options);

    // Should fail due to budget constraint ($10 > $0.001)
    assert!(result.is_err());
    if let Err(e) = result {
        // Verify it's a budget error
        assert!(e.to_string().contains("Budget exceeded"));
    }
}

#[test]
fn test_total_budget_tracking() {
    let mut selector = ModelSelector::new().with_total_budget_limit(0.01);
    let agent = create_speed_optimized_agent();

    // First selection
    let options1 = SelectionOptions::new(&agent).with_token_estimate(1000, 500);
    selector.select_model(&options1).expect("First selection should succeed");

    let cost1 = selector.get_total_cost();
    assert!(cost1 > 0.0);

    // Second selection
    let options2 = SelectionOptions::new(&agent).with_token_estimate(1000, 500);
    selector.select_model(&options2).expect("Second selection should succeed");

    let cost2 = selector.get_total_cost();
    assert!(cost2 > cost1);
    assert!(cost2 < 0.01); // Should be within total budget

    // Reset and verify
    selector.reset_cost_tracking();
    assert_eq!(selector.get_total_cost(), 0.0);
}

#[test]
fn test_premium_model_without_approval() {
    let mut selector = ModelSelector::new();
    let agent = create_thinking_agent();
    let options = SelectionOptions::new(&agent); // No approval flag

    let result = selector.select_model(&options).expect("Should select model");

    // Should select primary, not premium (no approval)
    assert_eq!(result.selected, SelectedModel::Primary);
    assert_eq!(result.model.model_id(), "mock-thinking-model");
}

#[test]
fn test_premium_model_with_approval() {
    let mut selector = ModelSelector::new().with_priority_override(ModelPriority::Expert);
    let agent = create_thinking_agent();
    let options = SelectionOptions::new(&agent).allow_premium();

    let result = selector.select_model(&options).expect("Should select model");

    // Should select premium with approval and priority override
    assert_eq!(result.selected, SelectedModel::Premium);
    assert_eq!(result.model.model_id(), "mock-expert-model");
}

#[test]
fn test_fallback_chain() {
    // Create an agent with models that will fail (non-mock engines)
    let agent = AgentMetadata {
        name: "test-agent".to_string(),
        display_name: Some("Test Agent".to_string()),
        category: Some("test".to_string()),
        color: "red".to_string(),
        summary: Some("Test agent".to_string()),
        description: "Test description".to_string(),
        recommended_models: Some(RecommendedModels {
            primary: ModelRecommendation {
                engine: "gemini".to_string(), // Will fail without API key
                model: "gemini-pro".to_string(),
                reasoning: "Primary model".to_string(),
                priority: ModelPriority::Balanced,
                cost_tier: CostTier::Medium,
                requires_approval: None,
            },
            fallback: Some(ModelRecommendation {
                engine: "openai".to_string(), // Will also fail without API key
                model: "gpt-4".to_string(),
                reasoning: "Fallback model".to_string(),
                priority: ModelPriority::Balanced,
                cost_tier: CostTier::Medium,
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
    };

    let mut selector = ModelSelector::new();
    let options = SelectionOptions::new(&agent);

    let result = selector.select_model(&options).expect("Should fall back to mock");

    // Should fall back to mock model when both primary and fallback fail
    assert_eq!(result.selected, SelectedModel::Mock);
    assert!(result.model.model_id().contains("mock"));
}

#[test]
fn test_multiple_agents_sequential() {
    let mut selector = ModelSelector::new().with_total_budget_limit(1.0);

    // Select for fast agent
    let fast_agent = create_speed_optimized_agent();
    let options1 = SelectionOptions::new(&fast_agent).with_token_estimate(1000, 500);
    let result1 = selector.select_model(&options1).expect("Should select");
    assert_eq!(result1.selected, SelectedModel::Primary);

    let cost_after_first = selector.get_total_cost();

    // Select for thinking agent
    let thinking_agent = create_thinking_agent();
    let options2 = SelectionOptions::new(&thinking_agent).with_token_estimate(1000, 500);
    let result2 = selector.select_model(&options2).expect("Should select");
    assert_eq!(result2.selected, SelectedModel::Primary);

    let cost_after_second = selector.get_total_cost();
    assert!(cost_after_second > cost_after_first);
}

#[test]
fn test_cost_tiers_comparison() {
    let mut selector = ModelSelector::new();

    // Low cost agent
    let low_cost_agent = create_speed_optimized_agent();
    let options_low =
        SelectionOptions::new(&low_cost_agent).with_token_estimate(1_000_000, 1_000_000);
    let result_low = selector.select_model(&options_low).expect("Should select");
    let cost_low = result_low.estimated_cost.unwrap();

    selector.reset_cost_tracking();

    // High cost agent
    let high_cost_agent = create_thinking_agent();
    let options_high =
        SelectionOptions::new(&high_cost_agent).with_token_estimate(1_000_000, 1_000_000);
    let result_high = selector.select_model(&options_high).expect("Should select");
    let cost_high = result_high.estimated_cost.unwrap();

    // High cost tier should be more expensive than low cost tier
    assert!(cost_high > cost_low);
}

//! Integration tests for reasoning effort propagation and ModelParameters.

use radium_abstraction::{ModelParameters, ReasoningEffort};
use radium_core::agents::config::{AgentConfig, ReasoningEffort as CoreReasoningEffort};

#[test]
fn test_reasoning_effort_enum_conversion() {
    // Test conversion between radium-abstraction and radium-core ReasoningEffort
    let core_low = CoreReasoningEffort::Low;
    let core_medium = CoreReasoningEffort::Medium;
    let core_high = CoreReasoningEffort::High;

    // Verify Display implementation
    assert_eq!(core_low.to_string(), "low");
    assert_eq!(core_medium.to_string(), "medium");
    assert_eq!(core_high.to_string(), "high");

    // Verify default
    assert_eq!(CoreReasoningEffort::default(), CoreReasoningEffort::Medium);
}

#[test]
fn test_model_parameters_reasoning_effort() {
    // Test that ModelParameters can store reasoning effort
    let params_with_reasoning = ModelParameters {
        temperature: Some(0.7),
        top_p: Some(1.0),
        max_tokens: Some(512),
        top_k: None,
        frequency_penalty: None,
        presence_penalty: None,
        response_format: None,
        stop_sequences: None,
        enable_grounding: None,
        grounding_threshold: None,
        reasoning_effort: Some(ReasoningEffort::High),
    };

    assert_eq!(params_with_reasoning.reasoning_effort, Some(ReasoningEffort::High));

    // Test default (no reasoning effort)
    let params_default = ModelParameters::default();
    assert_eq!(params_default.reasoning_effort, None);
}

#[test]
fn test_reasoning_effort_serialization() {
    // Test that ReasoningEffort serializes correctly
    let params = ModelParameters {
        reasoning_effort: Some(ReasoningEffort::High),
        ..Default::default()
    };

    let json = serde_json::to_string(&params).unwrap();
    assert!(json.contains("high") || json.contains("\"high\""));

    let deserialized: ModelParameters = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.reasoning_effort, Some(ReasoningEffort::High));
}

#[test]
fn test_reasoning_effort_all_levels() {
    // Test all reasoning effort levels
    for effort in [ReasoningEffort::Low, ReasoningEffort::Medium, ReasoningEffort::High] {
        let params = ModelParameters {
            reasoning_effort: Some(effort),
            ..Default::default()
        };
        assert_eq!(params.reasoning_effort, Some(effort));
        assert_eq!(effort.to_string(), match effort {
            ReasoningEffort::Low => "low",
            ReasoningEffort::Medium => "medium",
            ReasoningEffort::High => "high",
        });
    }
}


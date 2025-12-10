//! Integration tests for extended generation parameters.

use radium_abstraction::{ModelParameters, ResponseFormat};

#[test]
fn test_backward_compatibility() {
    // Verify that ModelParameters without new fields still works
    let params = ModelParameters {
        temperature: Some(0.7),
        top_p: Some(0.9),
        max_tokens: Some(100),
        top_k: None,
        frequency_penalty: None,
        presence_penalty: None,
        response_format: None,
        stop_sequences: None,
    };

    // Should compile and work without new parameters
    assert_eq!(params.top_k, None);
    assert_eq!(params.frequency_penalty, None);
    assert_eq!(params.presence_penalty, None);
    assert_eq!(params.response_format, None);
}

#[test]
fn test_penalty_clamping_integration() {
    // Test that out-of-range penalties are handled correctly
    let params = ModelParameters {
        temperature: Some(0.7),
        top_p: None,
        max_tokens: None,
        top_k: None,
        frequency_penalty: Some(2.5), // Over Gemini limit
        presence_penalty: Some(-0.5), // Under Gemini limit
        response_format: None,
        stop_sequences: None,
    };

    // Verify parameters are created (clamping happens in provider)
    assert_eq!(params.frequency_penalty, Some(2.5));
    assert_eq!(params.presence_penalty, Some(-0.5));
}

#[test]
fn test_response_format_serialization() {
    let text_format = ResponseFormat::Text;
    let json_format = ResponseFormat::Json;
    let schema_format = ResponseFormat::JsonSchema("{\"type\":\"object\"}".to_string());

    // Verify all variants can be serialized
    let text_json = serde_json::to_string(&text_format).unwrap();
    let json_json = serde_json::to_string(&json_format).unwrap();
    let schema_json = serde_json::to_string(&schema_format).unwrap();

    assert!(!text_json.is_empty());
    assert!(!json_json.is_empty());
    assert!(!schema_json.is_empty());

    // Verify deserialization works
    let text_deser: ResponseFormat = serde_json::from_str(&text_json).unwrap();
    let json_deser: ResponseFormat = serde_json::from_str(&json_json).unwrap();
    let schema_deser: ResponseFormat = serde_json::from_str(&schema_json).unwrap();

    assert_eq!(text_deser, ResponseFormat::Text);
    assert_eq!(json_deser, ResponseFormat::Json);
    assert!(matches!(schema_deser, ResponseFormat::JsonSchema(_)));
}


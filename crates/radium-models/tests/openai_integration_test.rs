//! Integration tests for OpenAI provider ResponseFormat support.

use radium_abstraction::{ModelParameters, ResponseFormat};
use radium_models::OpenAIModel;
use serde_json;

/// Test that ResponseFormat::Json converts to OpenAI's json_object format.
///
/// This test validates the conversion logic by checking the internal request structure.
#[test]
fn test_openai_response_format_json() {
    let model = OpenAIModel::with_api_key("gpt-4".to_string(), "test-key".to_string());
    
    let params = ModelParameters {
        temperature: Some(0.7),
        top_p: None,
        max_tokens: None,
        top_k: None,
        frequency_penalty: None,
        presence_penalty: None,
        response_format: Some(ResponseFormat::Json),
        stop_sequences: None,
        enable_grounding: None,
        grounding_threshold: None,
        reasoning_effort: None,
    };

    // The conversion should produce json_object format
    // We can't directly test the internal structure without making it public,
    // but we can verify the conversion doesn't error
    let result = model.convert_response_format(&params.response_format);
    assert!(result.is_ok());
    
    let openai_format = result.unwrap();
    assert!(openai_format.is_some());
    
    // Serialize to verify structure
    let serialized = serde_json::to_string(&openai_format).unwrap();
    assert!(serialized.contains("json_object"));
}

/// Test that ResponseFormat::JsonSchema converts to OpenAI's json_schema format with strict mode.
#[test]
fn test_openai_response_format_json_schema() {
    let model = OpenAIModel::with_api_key("gpt-4".to_string(), "test-key".to_string());
    
    let schema_str = r#"{"type": "object", "properties": {"name": {"type": "string"}}}"#;
    let params = ModelParameters {
        temperature: Some(0.7),
        top_p: None,
        max_tokens: None,
        top_k: None,
        frequency_penalty: None,
        presence_penalty: None,
        response_format: Some(ResponseFormat::JsonSchema(schema_str.to_string())),
        stop_sequences: None,
        enable_grounding: None,
        grounding_threshold: None,
        reasoning_effort: None,
    };

    let result = model.convert_response_format(&params.response_format);
    assert!(result.is_ok());
    
    let openai_format = result.unwrap();
    assert!(openai_format.is_some());
    
    // Serialize to verify structure
    let serialized = serde_json::to_string(&openai_format).unwrap();
    assert!(serialized.contains("json_schema"));
    assert!(serialized.contains("strict"));
    assert!(serialized.contains("response_schema")); // Default name
    assert!(serialized.contains("name")); // Schema property
}

/// Test that invalid JSON schema returns ModelError::SerializationError.
#[test]
fn test_openai_response_format_invalid_schema() {
    let model = OpenAIModel::with_api_key("gpt-4".to_string(), "test-key".to_string());
    
    let invalid_schema = "{ invalid json }";
    let params = ModelParameters {
        temperature: Some(0.7),
        top_p: None,
        max_tokens: None,
        top_k: None,
        frequency_penalty: None,
        presence_penalty: None,
        response_format: Some(ResponseFormat::JsonSchema(invalid_schema.to_string())),
        stop_sequences: None,
        enable_grounding: None,
        grounding_threshold: None,
        reasoning_effort: None,
    };

    let result = model.convert_response_format(&params.response_format);
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert!(matches!(error, radium_abstraction::ModelError::SerializationError(_)));
    
    // Verify error message contains useful information
    let error_msg = error.to_string();
    assert!(error_msg.contains("Invalid JSON schema"));
}

/// Test that ResponseFormat::Text produces None (no response format).
#[test]
fn test_openai_response_format_text() {
    let model = OpenAIModel::with_api_key("gpt-4".to_string(), "test-key".to_string());
    
    let params = ModelParameters {
        temperature: Some(0.7),
        top_p: None,
        max_tokens: None,
        top_k: None,
        frequency_penalty: None,
        presence_penalty: None,
        response_format: Some(ResponseFormat::Text),
        stop_sequences: None,
        enable_grounding: None,
        grounding_threshold: None,
        reasoning_effort: None,
    };

    let result = model.convert_response_format(&params.response_format);
    assert!(result.is_ok());
    
    let openai_format = result.unwrap();
    assert!(openai_format.is_none()); // Text format should produce None
}

/// Test that None response_format produces None.
#[test]
fn test_openai_response_format_none() {
    let model = OpenAIModel::with_api_key("gpt-4".to_string(), "test-key".to_string());
    
    let result = model.convert_response_format(&None);
    assert!(result.is_ok());
    
    let openai_format = result.unwrap();
    assert!(openai_format.is_none());
}

/// Test complex nested schema conversion.
#[test]
fn test_openai_response_format_complex_schema() {
    let model = OpenAIModel::with_api_key("gpt-4".to_string(), "test-key".to_string());
    
    let complex_schema = r#"{
        "type": "object",
        "properties": {
            "user": {
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "age": {"type": "number"}
                },
                "required": ["name"]
            },
            "items": {
                "type": "array",
                "items": {"type": "string"}
            }
        },
        "required": ["user"]
    }"#;
    
    let params = ModelParameters {
        temperature: Some(0.7),
        top_p: None,
        max_tokens: None,
        top_k: None,
        frequency_penalty: None,
        presence_penalty: None,
        response_format: Some(ResponseFormat::JsonSchema(complex_schema.to_string())),
        stop_sequences: None,
        enable_grounding: None,
        grounding_threshold: None,
        reasoning_effort: None,
    };

    let result = model.convert_response_format(&params.response_format);
    assert!(result.is_ok());
    
    let openai_format = result.unwrap();
    assert!(openai_format.is_some());
    
    // Serialize to verify structure
    let serialized = serde_json::to_string(&openai_format).unwrap();
    assert!(serialized.contains("json_schema"));
    assert!(serialized.contains("strict"));
    assert!(serialized.contains("user")); // Schema content should be preserved
}


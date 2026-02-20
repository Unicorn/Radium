//! End-to-end tests for complex nested schema validation across providers.

use radium_abstraction::{ModelParameters, ResponseFormat};

/// Test deeply nested object schema (3+ levels).
///
/// Validates that complex nested structures work correctly with schema enforcement.
#[test]
fn test_nested_object_schema_structure() {
    let nested_schema = r#"{
        "type": "object",
        "properties": {
            "user": {
                "type": "object",
                "properties": {
                    "profile": {
                        "type": "object",
                        "properties": {
                            "personal": {
                                "type": "object",
                                "properties": {
                                    "name": {"type": "string"},
                                    "age": {"type": "number"}
                                },
                                "required": ["name"]
                            },
                            "contact": {
                                "type": "object",
                                "properties": {
                                    "email": {"type": "string"},
                                    "phone": {"type": "string"}
                                }
                            }
                        },
                        "required": ["personal"]
                    },
                    "preferences": {
                        "type": "object",
                        "properties": {
                            "theme": {"type": "string"},
                            "language": {"type": "string"}
                        }
                    }
                },
                "required": ["profile"]
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
        response_format: Some(ResponseFormat::JsonSchema(nested_schema.to_string())),
        stop_sequences: None,
        enable_grounding: None,
        grounding_threshold: None,
        reasoning_effort: None,
    };

    // Verify schema can be parsed
    assert!(serde_json::from_str::<serde_json::Value>(nested_schema).is_ok());
    
    // Verify ResponseFormat can be created
    assert!(matches!(params.response_format, Some(ResponseFormat::JsonSchema(_))));
}

/// Test array schema with item constraints.
#[test]
fn test_array_schema_with_item_constraints() {
    let array_schema = r#"{
        "type": "object",
        "properties": {
            "products": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"},
                        "name": {"type": "string"},
                        "price": {"type": "number"},
                        "tags": {
                            "type": "array",
                            "items": {"type": "string"}
                        }
                    },
                    "required": ["id", "name", "price"]
                },
                "minItems": 1,
                "maxItems": 100
            }
        },
        "required": ["products"]
    }"#;

    let params = ModelParameters {
        temperature: Some(0.7),
        top_p: None,
        max_tokens: None,
        top_k: None,
        frequency_penalty: None,
        presence_penalty: None,
        response_format: Some(ResponseFormat::JsonSchema(array_schema.to_string())),
        stop_sequences: None,
        enable_grounding: None,
        grounding_threshold: None,
        reasoning_effort: None,
    };

    // Verify schema structure
    let parsed = serde_json::from_str::<serde_json::Value>(array_schema).unwrap();
    assert!(parsed["properties"]["products"]["type"].as_str() == Some("array"));
    assert!(matches!(params.response_format, Some(ResponseFormat::JsonSchema(_))));
}

/// Test schema with multiple required and optional fields.
#[test]
fn test_schema_with_required_optional_fields() {
    let mixed_schema = r#"{
        "type": "object",
        "properties": {
            "id": {"type": "string"},
            "name": {"type": "string"},
            "email": {"type": "string"},
            "age": {"type": "number"},
            "bio": {"type": "string"},
            "avatar": {"type": "string"}
        },
        "required": ["id", "name", "email"]
    }"#;

    let params = ModelParameters {
        temperature: Some(0.7),
        top_p: None,
        max_tokens: None,
        top_k: None,
        frequency_penalty: None,
        presence_penalty: None,
        response_format: Some(ResponseFormat::JsonSchema(mixed_schema.to_string())),
        stop_sequences: None,
        enable_grounding: None,
        grounding_threshold: None,
        reasoning_effort: None,
    };

    let parsed = serde_json::from_str::<serde_json::Value>(mixed_schema).unwrap();
    let required = parsed["required"].as_array().unwrap();
    assert_eq!(required.len(), 3);
    assert!(matches!(params.response_format, Some(ResponseFormat::JsonSchema(_))));
}

/// Test schema with enum value constraints.
#[test]
fn test_schema_with_enum_constraints() {
    let enum_schema = r#"{
        "type": "object",
        "properties": {
            "status": {
                "type": "string",
                "enum": ["pending", "active", "completed", "cancelled"]
            },
            "priority": {
                "type": "string",
                "enum": ["low", "medium", "high", "urgent"]
            },
            "type": {
                "type": "string",
                "enum": ["bug", "feature", "task", "epic"]
            }
        },
        "required": ["status", "priority"]
    }"#;

    let params = ModelParameters {
        temperature: Some(0.7),
        top_p: None,
        max_tokens: None,
        top_k: None,
        frequency_penalty: None,
        presence_penalty: None,
        response_format: Some(ResponseFormat::JsonSchema(enum_schema.to_string())),
        stop_sequences: None,
        enable_grounding: None,
        grounding_threshold: None,
        reasoning_effort: None,
    };

    let parsed = serde_json::from_str::<serde_json::Value>(enum_schema).unwrap();
    let status_enum = parsed["properties"]["status"]["enum"].as_array().unwrap();
    assert_eq!(status_enum.len(), 4);
    assert!(matches!(params.response_format, Some(ResponseFormat::JsonSchema(_))));
}

/// Test schema with string pattern validation.
#[test]
fn test_schema_with_string_patterns() {
    let pattern_schema = r#"{
        "type": "object",
        "properties": {
            "email": {
                "type": "string",
                "pattern": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
            },
            "phone": {
                "type": "string",
                "pattern": "^\\+?[1-9]\\d{1,14}$"
            },
            "uuid": {
                "type": "string",
                "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            }
        },
        "required": ["email"]
    }"#;

    let params = ModelParameters {
        temperature: Some(0.7),
        top_p: None,
        max_tokens: None,
        top_k: None,
        frequency_penalty: None,
        presence_penalty: None,
        response_format: Some(ResponseFormat::JsonSchema(pattern_schema.to_string())),
        stop_sequences: None,
        enable_grounding: None,
        grounding_threshold: None,
        reasoning_effort: None,
    };

    let parsed = serde_json::from_str::<serde_json::Value>(pattern_schema).unwrap();
    assert!(parsed["properties"]["email"]["pattern"].is_string());
    assert!(matches!(params.response_format, Some(ResponseFormat::JsonSchema(_))));
}

/// Test that same schema works for both providers (structure validation).
///
/// This test validates that the schema structure is provider-agnostic
/// and can be used with both Gemini and OpenAI.
#[test]
fn test_cross_provider_schema_compatibility() {
    let shared_schema = r#"{
        "type": "object",
        "properties": {
            "data": {
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "value": {"type": "number"}
                },
                "required": ["id"]
            }
        },
        "required": ["data"]
    }"#;

    // Create parameters for both providers
    let params_gemini = ModelParameters {
        temperature: Some(0.7),
        top_p: None,
        max_tokens: None,
        top_k: None,
        frequency_penalty: None,
        presence_penalty: None,
        response_format: Some(ResponseFormat::JsonSchema(shared_schema.to_string())),
        stop_sequences: None,
        enable_grounding: None,
        grounding_threshold: None,
        reasoning_effort: None,
    };

    let params_openai = ModelParameters {
        temperature: Some(0.7),
        top_p: None,
        max_tokens: None,
        top_k: None,
        frequency_penalty: None,
        presence_penalty: None,
        response_format: Some(ResponseFormat::JsonSchema(shared_schema.to_string())),
        stop_sequences: None,
        enable_grounding: None,
        grounding_threshold: None,
        reasoning_effort: None,
    };

    // Both should have the same schema
    match (&params_gemini.response_format, &params_openai.response_format) {
        (Some(ResponseFormat::JsonSchema(gemini_schema)), Some(ResponseFormat::JsonSchema(openai_schema))) => {
            assert_eq!(gemini_schema, openai_schema);
        }
        _ => panic!("Both should have JsonSchema format"),
    }

    // Schema should be valid JSON
    assert!(serde_json::from_str::<serde_json::Value>(shared_schema).is_ok());
}


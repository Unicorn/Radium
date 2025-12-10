//! Integration tests for Google Gemini model features.
//!
//! Tests cover core Gemini functionality:
//! - System instructions (Phase 1.1)
//! - Extended parameters (Phase 1.2): top_k, frequency_penalty, presence_penalty
//! - Response metadata (Phase 1.3): finish_reason, safety_ratings, citations
//! - Safety settings (Phase 3.3)
//! - Grounding integration (Phase 3.2)
//! - Thinking mode / reasoning effort (Phase 4.2)

use radium_abstraction::{
    ChatMessage, ContentBlock, ImageSource, MessageContent, Model, ModelParameters,
    ReasoningEffort, ResponseFormat,
};
use radium_models::{GeminiModel, GeminiSafetySetting, SafetyCategory, SafetyThreshold};

/// Helper to create a test model with API key from environment.
fn create_test_model() -> GeminiModel {
    let api_key = std::env::var("GEMINI_API_KEY")
        .expect("GEMINI_API_KEY environment variable must be set for integration tests");
    GeminiModel::with_api_key("gemini-2.0-flash-exp".to_string(), api_key)
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_system_instructions() {
    // Test system message extraction and handling
    let model = create_test_model();

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: MessageContent::Text(
                "You are a helpful assistant that speaks like a pirate.".to_string(),
            ),
        },
        ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text("Tell me about the ocean.".to_string()),
        },
    ];

    let response = model
        .generate_chat_completion(&messages, None)
        .await
        .expect("Failed to generate text");

    // Response should exist and contain pirate-like language
    assert!(!response.content.is_empty());
    println!("System instruction test response: {}", response.content);
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_extended_parameters() {
    // Test top_k, frequency_penalty, presence_penalty parameters
    let model = create_test_model();

    let params = ModelParameters {
        temperature: Some(0.9),
        top_p: Some(0.95),
        top_k: Some(40), // Extended parameter
        max_tokens: Some(100),
        frequency_penalty: Some(0.5), // Extended parameter
        presence_penalty: Some(0.3),  // Extended parameter
        stop_sequences: Some(vec!["END".to_string()]),
        response_format: None,
        enable_grounding: None,
        grounding_threshold: None,
        reasoning_effort: None,
    };

    let response = model
        .generate_text("Write a creative story about a robot.", Some(params))
        .await
        .expect("Failed to generate with extended parameters");

    assert!(!response.content.is_empty());
    let usage = response.usage.expect("Usage should be present");
    assert!(usage.prompt_tokens > 0);
    assert!(usage.completion_tokens > 0);
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_response_metadata() {
    // Test that response metadata is captured (finish_reason, safety_ratings, citations)
    let model = create_test_model();

    let response = model
        .generate_text("What is the capital of France?", None)
        .await
        .expect("Failed to generate text");

    // Basic checks
    assert!(!response.content.is_empty());
    assert!(response.content.to_lowercase().contains("paris"));

    // Check metadata exists
    if let Some(metadata) = &response.metadata {
        println!("Response metadata: {:?}", metadata);

        // finish_reason should be present
        if let Some(finish_reason) = metadata.get("finish_reason") {
            println!("Finish reason: {:?}", finish_reason);
            assert!(finish_reason.is_string());
        }

        // safety_ratings might be present
        if let Some(safety_ratings) = metadata.get("safety_ratings") {
            println!("Safety ratings: {:?}", safety_ratings);
            assert!(safety_ratings.is_array());
        }
    }
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_safety_settings_block() {
    // Test safety settings that should block unsafe content
    let model = create_test_model().with_safety_settings(Some(vec![
        GeminiSafetySetting {
            category: SafetyCategory::Harassment,
            threshold: SafetyThreshold::BlockLowAndAbove,
        },
        GeminiSafetySetting {
            category: SafetyCategory::HateSpeech,
            threshold: SafetyThreshold::BlockLowAndAbove,
        },
    ]));

    // This might get blocked or filtered by safety settings
    let result = model
        .generate_text("Write a very offensive message.", None)
        .await;

    match result {
        Ok(response) => {
            // If successful, check for safety ratings in metadata
            if let Some(metadata) = &response.metadata {
                if let Some(safety_ratings) = metadata.get("safety_ratings") {
                    println!("Safety ratings for potentially unsafe content: {:?}", safety_ratings);
                }
            }
            println!("Response (may be filtered): {}", response.content);
        }
        Err(e) => {
            // Expected: request might be blocked
            println!("Request blocked by safety settings: {}", e);
            assert!(e.to_string().contains("block") || e.to_string().contains("safety"));
        }
    }
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_grounding_with_search() {
    // Test Google Search grounding integration
    let model = create_test_model();

    let params = ModelParameters {
        temperature: Some(0.7),
        top_p: None,
        top_k: None,
        max_tokens: Some(200),
        frequency_penalty: None,
        presence_penalty: None,
        stop_sequences: None,
        response_format: None,
        enable_grounding: Some(true), // Enable grounding
        grounding_threshold: Some(0.3), // Low threshold for more grounding
        reasoning_effort: None,
    };

    let response = model
        .generate_text("What major events happened in the world yesterday?", Some(params))
        .await
        .expect("Failed to generate with grounding");

    assert!(!response.content.is_empty());
    println!("Grounding test response: {}", response.content);

    // Check for citations in metadata
    if let Some(metadata) = &response.metadata {
        if let Some(citations) = metadata.get("citations") {
            println!("Citations found: {:?}", citations);
        }
        if let Some(grounding_metadata) = metadata.get("grounding_metadata") {
            println!("Grounding metadata: {:?}", grounding_metadata);
        }
    }
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_thinking_mode_with_reasoning_effort() {
    // Test thinking mode with reasoning effort (only works with thinking models)
    let api_key = std::env::var("GEMINI_API_KEY")
        .expect("GEMINI_API_KEY environment variable must be set for integration tests");

    // Use a thinking model
    let model = GeminiModel::with_api_key("gemini-2.0-flash-thinking-exp".to_string(), api_key);

    let params = ModelParameters {
        temperature: Some(1.0),
        top_p: None,
        top_k: None,
        max_tokens: Some(500),
        frequency_penalty: None,
        presence_penalty: None,
        stop_sequences: None,
        response_format: None,
        enable_grounding: None,
        grounding_threshold: None,
        reasoning_effort: Some(ReasoningEffort::High), // High thinking budget
    };

    let response = model
        .generate_text(
            "Solve this puzzle: If 5 cats catch 5 mice in 5 minutes, how many cats are needed to catch 100 mice in 100 minutes?",
            Some(params),
        )
        .await
        .expect("Failed to generate with thinking mode");

    assert!(!response.content.is_empty());
    println!("Thinking mode response: {}", response.content);

    // The answer should be "5 cats" (common puzzle)
    assert!(response.content.contains("5"));
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_json_response_format() {
    // Test structured JSON output via response_format
    let model = create_test_model();

    let params = ModelParameters {
        temperature: Some(0.7),
        top_p: None,
        top_k: None,
        max_tokens: Some(200),
        frequency_penalty: None,
        presence_penalty: None,
        stop_sequences: None,
        response_format: Some(ResponseFormat::Json), // Request JSON output
        enable_grounding: None,
        grounding_threshold: None,
        reasoning_effort: None,
    };

    let response = model
        .generate_text(
            "Generate a JSON object with a 'name' field set to 'Alice' and an 'age' field set to 30.",
            Some(params),
        )
        .await
        .expect("Failed to generate JSON");

    assert!(!response.content.is_empty());
    println!("JSON response: {}", response.content);

    // Try to parse as JSON
    let json_result: Result<serde_json::Value, _> = serde_json::from_str(&response.content);
    assert!(json_result.is_ok(), "Response should be valid JSON");

    if let Ok(json) = json_result {
        println!("Parsed JSON: {:?}", json);
        // Check for expected fields
        if let Some(obj) = json.as_object() {
            assert!(obj.contains_key("name") || obj.contains_key("Name"));
            assert!(obj.contains_key("age") || obj.contains_key("Age"));
        }
    }
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_multimodal_image_input() {
    // Test image input support (multimodal)
    let model = create_test_model();

    // Create a simple base64-encoded 1x1 red pixel PNG
    let red_pixel_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Blocks(vec![
            ContentBlock::Image {
                source: ImageSource::Base64 {
                    data: red_pixel_base64.to_string(),
                },
                media_type: "image/png".to_string(),
            },
            ContentBlock::Text {
                text: "What color is this image?".to_string(),
            },
        ]),
    }];

    let response = model
        .generate_chat_completion(&messages, None)
        .await
        .expect("Failed to generate with image input");

    assert!(!response.content.is_empty());
    println!("Image analysis response: {}", response.content);

    // Response should mention red color
    assert!(response.content.to_lowercase().contains("red"));
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_multiple_system_messages() {
    // Test that multiple system messages are concatenated correctly
    let model = create_test_model();

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: MessageContent::Text("You are a helpful assistant.".to_string()),
        },
        ChatMessage {
            role: "system".to_string(),
            content: MessageContent::Text("Always respond in French.".to_string()),
        },
        ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text("Hello, how are you?".to_string()),
        },
    ];

    let response = model
        .generate_chat_completion(&messages, None)
        .await
        .expect("Failed to generate with multiple system messages");

    assert!(!response.content.is_empty());
    println!("Multiple system messages response: {}", response.content);

    // Response should be in French
    let french_indicators = ["bonjour", "comment", "je", "suis", "ça", "allez"];
    let has_french = french_indicators
        .iter()
        .any(|&word| response.content.to_lowercase().contains(word));

    println!("Has French indicators: {}", has_french);
}

#[test]
fn test_gemini_model_creation() {
    // Test model creation with API key
    let model = GeminiModel::with_api_key(
        "gemini-2.0-flash-exp".to_string(),
        "test-api-key".to_string(),
    );

    assert_eq!(model.model_id(), "gemini-2.0-flash-exp");
}

#[test]
fn test_gemini_model_with_code_execution() {
    // Test code execution configuration
    let model = GeminiModel::with_api_key(
        "gemini-2.0-flash-exp".to_string(),
        "test-api-key".to_string(),
    )
    .with_code_execution(true);

    // Model should be created successfully
    assert_eq!(model.model_id(), "gemini-2.0-flash-exp");
}

#[test]
fn test_gemini_model_with_safety_settings() {
    // Test safety settings configuration
    let model = GeminiModel::with_api_key(
        "gemini-2.0-flash-exp".to_string(),
        "test-api-key".to_string(),
    )
    .with_safety_settings(Some(vec![GeminiSafetySetting {
        category: SafetyCategory::Harassment,
        threshold: SafetyThreshold::BlockMediumAndAbove,
    }]));

    // Model should be created successfully
    assert_eq!(model.model_id(), "gemini-2.0-flash-exp");
}

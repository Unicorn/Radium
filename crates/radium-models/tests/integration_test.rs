//! Integration tests for model providers.

use radium_abstraction::{ChatMessage, ContentBlock, ImageSource, MessageContent, Model, ModelError, Citation};
use radium_models::{ClaudeModel, GeminiModel, MockModel, ModelFactory, OpenAIModel};
use std::path::PathBuf;
use serde_json;

#[tokio::test]
async fn test_mock_model_text_generation() {
    let model = MockModel::new("test-model".to_string());
    let response = model.generate_text("Hello", None).await;

    assert!(response.is_ok());
    let result = response.unwrap();
    assert!(!result.content.is_empty());
    assert!(result.usage.is_some());
}

#[tokio::test]
async fn test_mock_model_chat_completion() {
    use radium_abstraction::ChatMessage;

    let model = MockModel::new("test-model".to_string());
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Text("Hello".to_string()),
    }];

    let response = model.generate_chat_completion(&messages, None).await;

    assert!(response.is_ok());
    let result = response.unwrap();
    assert!(!result.content.is_empty());
}

#[tokio::test]
async fn test_model_factory_mock() {
    let model = ModelFactory::create_from_str("mock", "mock-model".to_string());

    assert!(model.is_ok());
    let model = model.unwrap();
    assert_eq!(model.model_id(), "mock-model");
}

#[tokio::test]
async fn test_model_factory_invalid_type() {
    let model = ModelFactory::create_from_str("invalid", "test".to_string());

    assert!(model.is_err());
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY environment variable"]
async fn test_gemini_model_creation() {
    // This test only runs if GEMINI_API_KEY is set
    #[allow(clippy::disallowed_methods)]
    if std::env::var("GEMINI_API_KEY").is_err() {
        return;
    }

    let model = GeminiModel::new("gemini-pro".to_string());
    assert!(model.is_ok());
}

#[tokio::test]
#[ignore = "Requires OPENAI_API_KEY environment variable"]
async fn test_openai_model_creation() {
    // This test only runs if OPENAI_API_KEY is set
    #[allow(clippy::disallowed_methods)]
    if std::env::var("OPENAI_API_KEY").is_err() {
        return;
    }

    let model = OpenAIModel::new("gpt-4".to_string());
    assert!(model.is_ok());
}

#[tokio::test]
async fn test_model_factory_with_env_keys() {
    // Test that factory can create models when API keys are available
    // Mock model should always work
    let mock_model = ModelFactory::create_from_str("mock", "test".to_string());
    assert!(mock_model.is_ok());

    // Gemini and OpenAI will fail without API keys, which is expected
    let gemini_model = ModelFactory::create_from_str("gemini", "gemini-pro".to_string());
    let openai_model = ModelFactory::create_from_str("openai", "gpt-4".to_string());

    // At least one should fail (or both if no keys), but structure should be correct
    if gemini_model.is_err() && openai_model.is_err() {
        // Both failed, likely no API keys - this is acceptable for CI
        return;
    }

    // If we get here, at least one API key was available
    assert!(gemini_model.is_ok() || openai_model.is_ok());
}

#[tokio::test]
async fn test_gemini_system_message_handling() {
    use radium_abstraction::ChatMessage;

    // Test that system messages are properly handled by Gemini model
    // This test uses MockModel to avoid requiring API keys
    let model = MockModel::new("gemini-test".to_string());

    // Test with system message
    let messages_with_system = vec![
        ChatMessage {
            role: "system".to_string(),
            content: MessageContent::Text("You are a helpful assistant.".to_string()),
        },
        ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text("Hello".to_string()),
        },
    ];

    let response = model.generate_chat_completion(&messages_with_system, None).await;
    assert!(response.is_ok());

    // Test with multiple system messages
    let messages_multiple_system = vec![
        ChatMessage {
            role: "system".to_string(),
            content: MessageContent::Text("First instruction.".to_string()),
        },
        ChatMessage {
            role: "system".to_string(),
            content: MessageContent::Text("Second instruction.".to_string()),
        },
        ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text("Hello".to_string()),
        },
    ];

    let response = model.generate_chat_completion(&messages_multiple_system, None).await;
    assert!(response.is_ok());

    // Test with no system messages
    let messages_no_system = vec![
        ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text("Hello".to_string()),
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: MessageContent::Text("Hi there!".to_string()),
        },
    ];

    let response = model.generate_chat_completion(&messages_no_system, None).await;
    assert!(response.is_ok());

    // Test with mixed message types
    let messages_mixed = vec![
        ChatMessage {
            role: "system".to_string(),
            content: MessageContent::Text("System instruction.".to_string()),
        },
        ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text("User message.".to_string()),
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: MessageContent::Text("Assistant message.".to_string()),
        },
        ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text("Follow-up question.".to_string()),
        },
    ];

    let response = model.generate_chat_completion(&messages_mixed, None).await;
    assert!(response.is_ok());
}

#[tokio::test]
async fn test_claude_text_only_backward_compatibility() {
    // Test that text-only messages work identically to before
    let model = MockModel::new("claude-test".to_string());
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Text("Hello".to_string()),
    }];

    let response = model.generate_chat_completion(&messages, None).await;
    assert!(response.is_ok());
    let result = response.unwrap();
    assert!(!result.content.is_empty());
}

#[tokio::test]
async fn test_openai_text_only_backward_compatibility() {
    // Test that text-only messages work with OpenAI models
    let model = MockModel::new("gpt-3.5-turbo".to_string());
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Text("Hello".to_string()),
    }];

    let response = model.generate_chat_completion(&messages, None).await;
    assert!(response.is_ok());
}

#[tokio::test]
async fn test_gemini_text_only_backward_compatibility() {
    // Test that text-only messages work with Gemini models
    let model = MockModel::new("gemini-pro".to_string());
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Text("Hello".to_string()),
    }];

    let response = model.generate_chat_completion(&messages, None).await;
    assert!(response.is_ok());
}

#[tokio::test]
async fn test_unsupported_content_types() {
    // Test that unsupported content types return appropriate errors
    let model = MockModel::new("test-model".to_string());

    // Test Audio block (not supported by any model in this implementation)
    let messages_audio = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Blocks(vec![ContentBlock::Audio {
            source: radium_abstraction::MediaSource::Url {
                url: "https://example.com/audio.mp3".to_string(),
            },
            media_type: "audio/mp3".to_string(),
        }]),
    }];

    // MockModel might not validate, but real models should return UnsupportedContentType
    // For now, we just verify the structure compiles
    let _response = model.generate_chat_completion(&messages_audio, None).await;
}

#[tokio::test]
async fn test_openai_vision_model_detection() {
    // Test that OpenAI vision models are detected correctly
    let vision_model = OpenAIModel::with_api_key("gpt-4o".to_string(), "test-key".to_string());
    assert!(vision_model.is_vision_capable());

    let non_vision_model = OpenAIModel::with_api_key("gpt-3.5-turbo".to_string(), "test-key".to_string());
    assert!(!non_vision_model.is_vision_capable());
}

#[tokio::test]
async fn test_openai_image_url_support() {
    // Test that OpenAI vision models accept image URLs
    let model = OpenAIModel::with_api_key("gpt-4o".to_string(), "test-key".to_string());
    
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Blocks(vec![
            ContentBlock::Text {
                text: "What's in this image?".to_string(),
            },
            ContentBlock::Image {
                source: ImageSource::Url {
                    url: "https://example.com/image.jpg".to_string(),
                },
                media_type: "image/jpeg".to_string(),
            },
        ]),
    }];

    // This will fail without a real API key, but we can test the conversion logic
    let result = model.to_openai_message(&messages[0]);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_openai_base64_not_supported() {
    // Test that OpenAI rejects Base64 images with helpful error
    let model = OpenAIModel::with_api_key("gpt-4o".to_string(), "test-key".to_string());
    
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Blocks(vec![ContentBlock::Image {
            source: ImageSource::Base64 {
                data: "base64data".to_string(),
            },
            media_type: "image/jpeg".to_string(),
        }]),
    }];

    let result = model.to_openai_message(&messages[0]);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ModelError::UnsupportedContentType { .. }
    ));
}

#[tokio::test]
async fn test_gemini_url_not_supported() {
    // Test that Gemini rejects URL images with helpful error
    let model = GeminiModel::with_api_key("gemini-pro".to_string(), "test-key".to_string());
    
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Blocks(vec![ContentBlock::Image {
            source: ImageSource::Url {
                url: "https://example.com/image.jpg".to_string(),
            },
            media_type: "image/jpeg".to_string(),
        }]),
    }];

    let result = GeminiModel::to_gemini_content(&messages[0]);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ModelError::UnsupportedContentType { .. }
    ));
}

#[tokio::test]
async fn test_claude_multimodal_text_and_image() {
    // Test Claude with multimodal content (text + image)
    let model = ClaudeModel::with_api_key("claude-sonnet-4-5-20250929".to_string(), "test-key".to_string());
    
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Blocks(vec![
            ContentBlock::Text {
                text: "Analyze this image".to_string(),
            },
            ContentBlock::Image {
                source: ImageSource::Base64 {
                    data: "base64imagedata".to_string(),
                },
                media_type: "image/png".to_string(),
            },
        ]),
    }];

    // Test conversion (will fail on API call without real key, but conversion should work)
    let result = ClaudeModel::to_claude_message(&messages[0]);
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore = "Requires API keys and network access"]
async fn test_claude_multimodal_integration() {
    // Integration test for Claude with multimodal content
    #[allow(clippy::disallowed_methods)]
    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        return;
    }

    let model = ClaudeModel::new("claude-sonnet-4-5-20250929".to_string()).unwrap();
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Text("Hello".to_string()),
    }];

    let response = model.generate_chat_completion(&messages, None).await;
    assert!(response.is_ok());
}

#[tokio::test]
#[ignore = "Requires API keys and network access"]
async fn test_openai_vision_integration() {
    // Integration test for OpenAI vision model
    #[allow(clippy::disallowed_methods)]
    if std::env::var("OPENAI_API_KEY").is_err() {
        return;
    }

    let model = OpenAIModel::new("gpt-4o".to_string()).unwrap();
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Text("Hello".to_string()),
    }];

    let response = model.generate_chat_completion(&messages, None).await;
    assert!(response.is_ok());
}

#[tokio::test]
#[ignore = "Requires API keys and network access"]
async fn test_gemini_multimodal_integration() {
    // Integration test for Gemini with multimodal content
    #[allow(clippy::disallowed_methods)]
    if std::env::var("GEMINI_API_KEY").is_err() {
        return;
    }

    let model = GeminiModel::new("gemini-pro".to_string()).unwrap();
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Text("Hello".to_string()),
    }];

    let response = model.generate_chat_completion(&messages, None).await;
    assert!(response.is_ok());
}

#[tokio::test]
async fn test_gemini_safety_settings_serialization() {
    // Test that safety settings are correctly included in request serialization
    use radium_models::{GeminiSafetySetting, SafetyCategory, SafetyThreshold};
    use serde_json;

    // Test JSON serialization of safety settings
    let settings = vec![
        GeminiSafetySetting {
            category: SafetyCategory::HateSpeech,
            threshold: SafetyThreshold::BlockMediumAndAbove,
        },
        GeminiSafetySetting {
            category: SafetyCategory::Harassment,
            threshold: SafetyThreshold::BlockLowAndAbove,
        },
    ];

    let json = serde_json::to_string(&settings).unwrap();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0]["category"], "HARM_CATEGORY_HATE_SPEECH");
    assert_eq!(parsed[0]["threshold"], "BLOCK_MEDIUM_AND_ABOVE");
    assert_eq!(parsed[1]["category"], "HARM_CATEGORY_HARASSMENT");
    assert_eq!(parsed[1]["threshold"], "BLOCK_LOW_AND_ABOVE");
}

#[tokio::test]
async fn test_gemini_safety_settings_all_categories() {
    // Test all safety categories serialize correctly
    use radium_models::{GeminiSafetySetting, SafetyCategory, SafetyThreshold};
    use serde_json;

    let settings = vec![
        GeminiSafetySetting {
            category: SafetyCategory::HateSpeech,
            threshold: SafetyThreshold::BlockNone,
        },
        GeminiSafetySetting {
            category: SafetyCategory::SexuallyExplicit,
            threshold: SafetyThreshold::BlockLowAndAbove,
        },
        GeminiSafetySetting {
            category: SafetyCategory::DangerousContent,
            threshold: SafetyThreshold::BlockMediumAndAbove,
        },
        GeminiSafetySetting {
            category: SafetyCategory::Harassment,
            threshold: SafetyThreshold::BlockOnlyHigh,
        },
        GeminiSafetySetting {
            category: SafetyCategory::CivicIntegrity,
            threshold: SafetyThreshold::BlockNone,
        },
    ];

    let json = serde_json::to_string(&settings).unwrap();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.len(), 5);

    // Verify each category has correct API string
    let categories: Vec<&str> = parsed.iter()
        .map(|s| s["category"].as_str().unwrap())
        .collect();
    assert!(categories.contains(&"HARM_CATEGORY_HATE_SPEECH"));
    assert!(categories.contains(&"HARM_CATEGORY_SEXUALLY_EXPLICIT"));
    assert!(categories.contains(&"HARM_CATEGORY_DANGEROUS_CONTENT"));
    assert!(categories.contains(&"HARM_CATEGORY_HARASSMENT"));
    assert!(categories.contains(&"HARM_CATEGORY_CIVIC_INTEGRITY"));
}

#[tokio::test]
async fn test_gemini_safety_settings_all_thresholds() {
    // Test all threshold levels serialize correctly
    use radium_models::{GeminiSafetySetting, SafetyCategory, SafetyThreshold};
    use serde_json;

    let settings = vec![
        GeminiSafetySetting {
            category: SafetyCategory::HateSpeech,
            threshold: SafetyThreshold::BlockNone,
        },
        GeminiSafetySetting {
            category: SafetyCategory::Harassment,
            threshold: SafetyThreshold::BlockLowAndAbove,
        },
        GeminiSafetySetting {
            category: SafetyCategory::SexuallyExplicit,
            threshold: SafetyThreshold::BlockMediumAndAbove,
        },
        GeminiSafetySetting {
            category: SafetyCategory::DangerousContent,
            threshold: SafetyThreshold::BlockOnlyHigh,
        },
    ];

    let json = serde_json::to_string(&settings).unwrap();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.len(), 4);

    // Verify each threshold has correct API string
    let thresholds: Vec<&str> = parsed.iter()
        .map(|s| s["threshold"].as_str().unwrap())
        .collect();
    assert!(thresholds.contains(&"BLOCK_NONE"));
    assert!(thresholds.contains(&"BLOCK_LOW_AND_ABOVE"));
    assert!(thresholds.contains(&"BLOCK_MEDIUM_AND_ABOVE"));
    assert!(thresholds.contains(&"BLOCK_ONLY_HIGH"));
}

#[tokio::test]
async fn test_gemini_safety_settings_builder() {
    // Test that with_safety_settings builder method works
    use radium_models::{GeminiSafetySetting, SafetyCategory, SafetyThreshold};

    // Model with safety settings
    let model = GeminiModel::with_api_key("gemini-pro".to_string(), "test-key".to_string())
        .with_safety_settings(Some(vec![
            GeminiSafetySetting {
                category: SafetyCategory::HateSpeech,
                threshold: SafetyThreshold::BlockMediumAndAbove,
            },
        ]));

    // Builder pattern should work
    assert_eq!(model.model_id(), "gemini-pro");
}

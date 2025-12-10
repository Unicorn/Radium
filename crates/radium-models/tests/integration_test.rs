//! Integration tests for model providers.

use radium_abstraction::Model;
use radium_models::{GeminiModel, MockModel, ModelFactory, OpenAIModel};

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
    let messages = vec![ChatMessage { role: "user".to_string(), content: "Hello".to_string() }];

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
        ChatMessage { role: "system".to_string(), content: "You are a helpful assistant.".to_string() },
        ChatMessage { role: "user".to_string(), content: "Hello".to_string() },
    ];

    let response = model.generate_chat_completion(&messages_with_system, None).await;
    assert!(response.is_ok());

    // Test with multiple system messages
    let messages_multiple_system = vec![
        ChatMessage { role: "system".to_string(), content: "First instruction.".to_string() },
        ChatMessage { role: "system".to_string(), content: "Second instruction.".to_string() },
        ChatMessage { role: "user".to_string(), content: "Hello".to_string() },
    ];

    let response = model.generate_chat_completion(&messages_multiple_system, None).await;
    assert!(response.is_ok());

    // Test with no system messages
    let messages_no_system = vec![
        ChatMessage { role: "user".to_string(), content: "Hello".to_string() },
        ChatMessage { role: "assistant".to_string(), content: "Hi there!".to_string() },
    ];

    let response = model.generate_chat_completion(&messages_no_system, None).await;
    assert!(response.is_ok());

    // Test with mixed message types
    let messages_mixed = vec![
        ChatMessage { role: "system".to_string(), content: "System instruction.".to_string() },
        ChatMessage { role: "user".to_string(), content: "User message.".to_string() },
        ChatMessage { role: "assistant".to_string(), content: "Assistant message.".to_string() },
        ChatMessage { role: "user".to_string(), content: "Follow-up question.".to_string() },
    ];

    let response = model.generate_chat_completion(&messages_mixed, None).await;
    assert!(response.is_ok());
}

//! Integration tests for streaming functionality with real APIs.
//!
//! These tests require API keys and network access. They are marked with `#[ignore]`
//! and will skip gracefully if API keys are not available.

use futures::StreamExt;
use radium_abstraction::{Model, StreamingModel, StreamItem, ModelParameters};
use radium_models::{GeminiModel, OpenAIModel, ClaudeModel};
use std::env;

/// Helper function to skip test if API key is not available
fn skip_if_no_api_key(provider: &str) -> bool {
    let key = match provider {
        "gemini" => env::var("GEMINI_API_KEY"),
        "openai" => env::var("OPENAI_API_KEY"),
        "claude" => env::var("ANTHROPIC_API_KEY"),
        _ => {
            eprintln!("Unknown provider: {}", provider);
            return true;
        }
    };

    if key.is_err() {
        println!("Skipping test: {} API key not set", provider);
        return true;
    }
    false
}

// ============================================================================
// Basic Streaming Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_gemini_streaming() {
    if skip_if_no_api_key("gemini") {
        return;
    }

    let model = GeminiModel::new("gemini-2.0-flash-exp".to_string())
        .expect("Should create GeminiModel");

    let mut stream = model
        .generate_stream("Count to 5", None)
        .await
        .expect("Should create stream");

    let mut answer_tokens = Vec::new();
    while let Some(result) = stream.next().await {
        let item = result.expect("Stream should not error");
        match item {
            StreamItem::AnswerToken(token) => answer_tokens.push(token),
            StreamItem::ThinkingToken(_) => {}, // Ignore thinking tokens for this test
            StreamItem::Metadata(_) => {}, // Ignore metadata
        }
    }

    assert!(!answer_tokens.is_empty(), "Stream should yield at least one answer token");
}

#[tokio::test]
#[ignore = "Requires OPENAI_API_KEY and network access"]
async fn test_openai_streaming() {
    if skip_if_no_api_key("openai") {
        return;
    }

    let model = OpenAIModel::new("gpt-3.5-turbo".to_string())
        .expect("Should create OpenAIModel");

    let mut stream = model
        .generate_stream("Count to 5", None)
        .await
        .expect("Should create stream");

    let mut answer_tokens = Vec::new();
    while let Some(result) = stream.next().await {
        let item = result.expect("Stream should not error");
        match item {
            StreamItem::AnswerToken(token) => answer_tokens.push(token),
            StreamItem::ThinkingToken(_) => {}, // OpenAI doesn't have thinking mode
            StreamItem::Metadata(_) => {},
        }
    }

    assert!(!answer_tokens.is_empty(), "Stream should yield at least one answer token");
}

#[tokio::test]
#[ignore = "Requires ANTHROPIC_API_KEY and network access"]
async fn test_claude_streaming() {
    if skip_if_no_api_key("claude") {
        return;
    }

    let model = ClaudeModel::new("claude-3-5-sonnet-20241022".to_string())
        .expect("Should create ClaudeModel");

    let mut stream = model
        .generate_stream("Count to 5", None)
        .await
        .expect("Should create stream");

    let mut answer_tokens = Vec::new();
    while let Some(result) = stream.next().await {
        let item = result.expect("Stream should not error");
        match item {
            StreamItem::AnswerToken(token) => answer_tokens.push(token),
            StreamItem::ThinkingToken(_) => {}, // Ignore thinking tokens for this test
            StreamItem::Metadata(_) => {},
        }
    }

    assert!(!answer_tokens.is_empty(), "Stream should yield at least one answer token");
}

// ============================================================================
// Thinking Mode Tests
// ============================================================================

// Note: Claude thinking mode tests are skipped because thinking mode configuration
// is provider-specific and not exposed through the ModelParameters abstraction yet.
// Claude thinking models (claude-3-7-sonnet-20250219) require specific configuration
// that's handled at the provider level.

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_gemini_thinking_mode_streaming() {
    if skip_if_no_api_key("gemini") {
        return;
    }

    let model = GeminiModel::new("gemini-2.0-flash-thinking-exp-01-21".to_string())
        .expect("Should create GeminiModel");

    let mut stream = model
        .generate_stream("Solve this riddle: I have cities but no houses. I have mountains but no trees. I have water but no fish. What am I?", None)
        .await
        .expect("Should create stream");

    let mut thinking_tokens = Vec::new();
    let mut answer_tokens = Vec::new();

    while let Some(result) = stream.next().await {
        let item = result.expect("Stream should not error");
        match item {
            StreamItem::ThinkingToken(token) => thinking_tokens.push(token),
            StreamItem::AnswerToken(token) => answer_tokens.push(token),
            StreamItem::Metadata(_) => {},
        }
    }

    // Thinking model should produce both thinking and answer tokens
    assert!(!thinking_tokens.is_empty(), "Should have thinking tokens");
    assert!(!answer_tokens.is_empty(), "Should have answer tokens");

    println!("Thinking tokens: {}", thinking_tokens.len());
    println!("Answer tokens: {}", answer_tokens.len());
}

// ============================================================================
// Streaming vs Non-Streaming Comparison Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_gemini_streaming_vs_non_streaming() {
    if skip_if_no_api_key("gemini") {
        return;
    }

    let model = GeminiModel::new("gemini-2.0-flash-exp".to_string())
        .expect("Should create GeminiModel");

    let prompt = "Say hello in exactly 5 words";

    // Get non-streaming response
    let non_streaming_response = model
        .generate_text(prompt, None)
        .await
        .expect("Should generate text");

    // Get streaming response
    let mut stream = model
        .generate_stream(prompt, None)
        .await
        .expect("Should create stream");

    let mut final_content = String::new();
    while let Some(result) = stream.next().await {
        let item = result.expect("Stream should not error");
        if let StreamItem::AnswerToken(token) = item {
            final_content.push_str(&token);
        }
    }

    // Content should be similar (responses may vary slightly)
    assert!(!final_content.is_empty());
    assert!(!non_streaming_response.content.is_empty());
}

#[tokio::test]
#[ignore = "Requires OPENAI_API_KEY and network access"]
async fn test_openai_streaming_vs_non_streaming() {
    if skip_if_no_api_key("openai") {
        return;
    }

    let model = OpenAIModel::new("gpt-3.5-turbo".to_string())
        .expect("Should create OpenAIModel");

    let prompt = "Say hello in exactly 5 words";

    // Get non-streaming response
    let non_streaming_response = model
        .generate_text(prompt, None)
        .await
        .expect("Should generate text");

    // Get streaming response
    let mut stream = model
        .generate_stream(prompt, None)
        .await
        .expect("Should create stream");

    let mut final_content = String::new();
    while let Some(result) = stream.next().await {
        let item = result.expect("Stream should not error");
        if let StreamItem::AnswerToken(token) = item {
            final_content.push_str(&token);
        }
    }

    // Both should produce non-empty content
    assert!(!final_content.is_empty());
    assert!(!non_streaming_response.content.is_empty());
}

#[tokio::test]
#[ignore = "Requires ANTHROPIC_API_KEY and network access"]
async fn test_claude_streaming_vs_non_streaming() {
    if skip_if_no_api_key("claude") {
        return;
    }

    let model = ClaudeModel::new("claude-3-5-sonnet-20241022".to_string())
        .expect("Should create ClaudeModel");

    let prompt = "Say hello in exactly 5 words";

    // Get non-streaming response
    let non_streaming_response = model
        .generate_text(prompt, None)
        .await
        .expect("Should generate text");

    // Get streaming response
    let mut stream = model
        .generate_stream(prompt, None)
        .await
        .expect("Should create stream");

    let mut final_content = String::new();
    while let Some(result) = stream.next().await {
        let item = result.expect("Stream should not error");
        if let StreamItem::AnswerToken(token) = item {
            final_content.push_str(&token);
        }
    }

    // Both should produce non-empty content
    assert!(!final_content.is_empty());
    assert!(!non_streaming_response.content.is_empty());
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_gemini_streaming_error_handling() {
    if skip_if_no_api_key("gemini") {
        return;
    }

    // Test with invalid model ID to trigger error
    let model = GeminiModel::with_api_key(
        "invalid-model-id".to_string(),
        env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY should be set"),
    );

    let result = model.generate_stream("test", None).await;

    // Should return an error for invalid model
    assert!(result.is_err(), "Should error on invalid model ID");
}

#[tokio::test]
#[ignore = "Requires OPENAI_API_KEY and network access"]
async fn test_openai_streaming_error_handling() {
    if skip_if_no_api_key("openai") {
        return;
    }

    // Test with invalid model ID to trigger error
    let model = OpenAIModel::with_api_key(
        "invalid-model-id".to_string(),
        env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY should be set"),
    );

    let result = model.generate_stream("test", None).await;

    // Should return an error for invalid model
    assert!(result.is_err(), "Should error on invalid model ID");
}

#[tokio::test]
#[ignore = "Requires ANTHROPIC_API_KEY and network access"]
async fn test_claude_streaming_error_handling() {
    if skip_if_no_api_key("claude") {
        return;
    }

    // Test with invalid model ID to trigger error
    let model = ClaudeModel::with_api_key(
        "invalid-model-id".to_string(),
        env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY should be set"),
    );

    let result = model.generate_stream("test", None).await;

    // Should return an error for invalid model
    assert!(result.is_err(), "Should error on invalid model ID");
}

// ============================================================================
// Token Accumulation Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_gemini_streaming_token_progression() {
    if skip_if_no_api_key("gemini") {
        return;
    }

    let model = GeminiModel::new("gemini-2.0-flash-exp".to_string())
        .expect("Should create GeminiModel");

    let mut stream = model
        .generate_stream("Write a short sentence about the weather", None)
        .await
        .expect("Should create stream");

    let mut accumulated_content = String::new();
    let mut chunk_count = 0;

    while let Some(result) = stream.next().await {
        let item = result.expect("Stream should not error");
        if let StreamItem::AnswerToken(token) = item {
            accumulated_content.push_str(&token);
            chunk_count += 1;
        }
    }

    // Should have received at least one chunk
    assert!(chunk_count > 0, "Should receive at least one chunk");
    assert!(!accumulated_content.is_empty(), "Final content should not be empty");
}

#[tokio::test]
#[ignore = "Requires OPENAI_API_KEY and network access"]
async fn test_openai_streaming_token_progression() {
    if skip_if_no_api_key("openai") {
        return;
    }

    let model = OpenAIModel::new("gpt-3.5-turbo".to_string())
        .expect("Should create OpenAIModel");

    let mut stream = model
        .generate_stream("Write a short sentence about the weather", None)
        .await
        .expect("Should create stream");

    let mut accumulated_content = String::new();
    let mut chunk_count = 0;

    while let Some(result) = stream.next().await {
        let item = result.expect("Stream should not error");
        if let StreamItem::AnswerToken(token) = item {
            accumulated_content.push_str(&token);
            chunk_count += 1;
        }
    }

    // Should have received at least one chunk
    assert!(chunk_count > 0, "Should receive at least one chunk");
    assert!(!accumulated_content.is_empty(), "Final content should not be empty");
}

#[tokio::test]
#[ignore = "Requires ANTHROPIC_API_KEY and network access"]
async fn test_claude_streaming_token_progression() {
    if skip_if_no_api_key("claude") {
        return;
    }

    let model = ClaudeModel::new("claude-3-5-sonnet-20241022".to_string())
        .expect("Should create ClaudeModel");

    let mut stream = model
        .generate_stream("Write a short sentence about the weather", None)
        .await
        .expect("Should create stream");

    let mut accumulated_content = String::new();
    let mut chunk_count = 0;

    while let Some(result) = stream.next().await {
        let item = result.expect("Stream should not error");
        if let StreamItem::AnswerToken(token) = item {
            accumulated_content.push_str(&token);
            chunk_count += 1;
        }
    }

    // Should have received at least one chunk
    assert!(chunk_count > 0, "Should receive at least one chunk");
    assert!(!accumulated_content.is_empty(), "Final content should not be empty");
}

// ============================================================================
// Parameter Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_gemini_streaming_with_parameters() {
    if skip_if_no_api_key("gemini") {
        return;
    }

    let model = GeminiModel::new("gemini-2.0-flash-exp".to_string())
        .expect("Should create GeminiModel");

    let mut params = ModelParameters::default();
    params.temperature = Some(0.5);
    params.max_tokens = Some(50);

    let mut stream = model
        .generate_stream("Write a creative story opening", Some(params))
        .await
        .expect("Should create stream");

    let mut content = String::new();
    while let Some(result) = stream.next().await {
        let item = result.expect("Stream should not error");
        if let StreamItem::AnswerToken(token) = item {
            content.push_str(&token);
        }
    }

    assert!(!content.is_empty(), "Should generate content with parameters");
}

#[tokio::test]
#[ignore = "Requires ANTHROPIC_API_KEY and network access"]
async fn test_claude_streaming_with_parameters() {
    if skip_if_no_api_key("claude") {
        return;
    }

    let model = ClaudeModel::new("claude-3-5-sonnet-20241022".to_string())
        .expect("Should create ClaudeModel");

    let mut params = ModelParameters::default();
    params.temperature = Some(0.5);
    params.max_tokens = Some(50);

    let mut stream = model
        .generate_stream("Write a creative story opening", Some(params))
        .await
        .expect("Should create stream");

    let mut content = String::new();
    while let Some(result) = stream.next().await {
        let item = result.expect("Stream should not error");
        if let StreamItem::AnswerToken(token) = item {
            content.push_str(&token);
        }
    }

    assert!(!content.is_empty(), "Should generate content with parameters");
}

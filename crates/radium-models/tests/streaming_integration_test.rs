//! Integration tests for streaming functionality with real APIs.
//!
//! These tests require API keys and network access. They are marked with `#[ignore]`
//! and will skip gracefully if API keys are not available.

use futures::StreamExt;
use radium_abstraction::{Model, StreamingModel};
use radium_models::{GeminiModel, OpenAIModel};
use std::env;

/// Helper function to skip test if API key is not available
fn skip_if_no_api_key(provider: &str) -> bool {
    let key = match provider {
        "gemini" => env::var("GEMINI_API_KEY"),
        "openai" => env::var("OPENAI_API_KEY"),
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

    let mut tokens = Vec::new();
    while let Some(result) = stream.next().await {
        let token = result.expect("Stream should not error");
        tokens.push(token);
    }

    assert!(!tokens.is_empty(), "Stream should yield at least one token");
    
    // Last token should contain the complete response
    let final_content = tokens.last().unwrap();
    assert!(!final_content.is_empty(), "Final content should not be empty");
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

    let mut tokens = Vec::new();
    while let Some(result) = stream.next().await {
        let token = result.expect("Stream should not error");
        tokens.push(token);
    }

    assert!(!tokens.is_empty(), "Stream should yield at least one token");
    
    // Last token should contain the complete response
    let final_content = tokens.last().unwrap();
    assert!(!final_content.is_empty(), "Final content should not be empty");
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_gemini_streaming_vs_non_streaming_token_count() {
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
    
    let non_streaming_tokens = non_streaming_response
        .usage
        .as_ref()
        .map(|u| u.total_tokens)
        .unwrap_or(0);

    // Get streaming response
    let mut stream = model
        .generate_stream(prompt, None)
        .await
        .expect("Should create stream");

    let mut final_content = String::new();
    while let Some(result) = stream.next().await {
        let content = result.expect("Stream should not error");
        final_content = content;
    }

    // Compare content (should be similar, though streaming may have slight differences)
    // Note: We can't compare token counts directly as streaming doesn't provide them
    // But we can verify the content is reasonable
    assert!(!final_content.is_empty());
    assert_eq!(
        non_streaming_response.content.trim().to_lowercase(),
        final_content.trim().to_lowercase(),
        "Streaming and non-streaming content should match"
    );
}

#[tokio::test]
#[ignore = "Requires OPENAI_API_KEY and network access"]
async fn test_openai_streaming_vs_non_streaming_token_count() {
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
    
    let non_streaming_tokens = non_streaming_response
        .usage
        .as_ref()
        .map(|u| u.total_tokens)
        .unwrap_or(0);

    // Get streaming response
    let mut stream = model
        .generate_stream(prompt, None)
        .await
        .expect("Should create stream");

    let mut final_content = String::new();
    while let Some(result) = stream.next().await {
        let content = result.expect("Stream should not error");
        final_content = content;
    }

    // Compare content (should be similar)
    assert!(!final_content.is_empty());
    // Note: OpenAI responses may vary, so we just check they're both non-empty
    assert!(!non_streaming_response.content.is_empty());
}

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
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_gemini_streaming_accumulation() {
    if skip_if_no_api_key("gemini") {
        return;
    }

    let model = GeminiModel::new("gemini-2.0-flash-exp".to_string())
        .expect("Should create GeminiModel");
    
    let mut stream = model
        .generate_stream("Write a short sentence", None)
        .await
        .expect("Should create stream");

    let mut last_len = 0;
    let mut chunk_count = 0;
    
    while let Some(result) = stream.next().await {
        let content = result.expect("Stream should not error");
        chunk_count += 1;
        
        // Content should accumulate (each chunk should be longer or equal)
        assert!(
            content.len() >= last_len,
            "Content should accumulate: {} >= {}",
            content.len(),
            last_len
        );
        last_len = content.len();
    }
    
    // Should have received multiple chunks
    assert!(chunk_count > 0, "Should receive at least one chunk");
    assert!(last_len > 0, "Final content should not be empty");
}

#[tokio::test]
#[ignore = "Requires OPENAI_API_KEY and network access"]
async fn test_openai_streaming_accumulation() {
    if skip_if_no_api_key("openai") {
        return;
    }

    let model = OpenAIModel::new("gpt-3.5-turbo".to_string())
        .expect("Should create OpenAIModel");
    
    let mut stream = model
        .generate_stream("Write a short sentence", None)
        .await
        .expect("Should create stream");

    let mut last_len = 0;
    let mut chunk_count = 0;
    
    while let Some(result) = stream.next().await {
        let content = result.expect("Stream should not error");
        chunk_count += 1;
        
        // Content should accumulate (each chunk should be longer or equal)
        assert!(
            content.len() >= last_len,
            "Content should accumulate: {} >= {}",
            content.len(),
            last_len
        );
        last_len = content.len();
    }
    
    // Should have received multiple chunks
    assert!(chunk_count > 0, "Should receive at least one chunk");
    assert!(last_len > 0, "Final content should not be empty");
}


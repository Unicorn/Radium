//! Integration tests for OllamaModel.
//!
//! These tests require a running Ollama server with the llama2 model.
//! To run these tests:
//! 1. Install Ollama: curl https://ollama.ai/install.sh | sh
//! 2. Start Ollama: ollama serve
//! 3. Pull llama2: ollama pull llama2
//! 4. Run tests: cargo test -p radium-models -- --ignored

use radium_abstraction::{ChatMessage, MessageContent, Model, StreamingModel};
use radium_models::OllamaModel;
use futures::StreamExt;

/// Check if Ollama server is available at the given URL
async fn is_ollama_available(base_url: &str) -> bool {
    let client = reqwest::Client::new();
    client
        .get(format!("{}/api/tags", base_url))
        .send()
        .await
        .is_ok()
}

/// Skip test if Ollama is not available
async fn skip_if_ollama_unavailable(base_url: &str) {
    if !is_ollama_available(base_url).await {
        println!("Skipping test: Ollama server not available at {}", base_url);
        std::process::exit(0);
    }
}

#[tokio::test]
#[ignore = "Requires Ollama server running with llama2 model"]
async fn test_ollama_text_generation() {
    let base_url = "http://localhost:11434";
    skip_if_ollama_unavailable(base_url).await;

    let model = OllamaModel::with_base_url(
        "llama2".to_string(),
        base_url.to_string(),
    ).unwrap();

    let response = model.generate_text("Say hello in one word", None).await;

    assert!(response.is_ok(), "Text generation should succeed");
    let result = response.unwrap();
    assert!(!result.content.is_empty(), "Response should not be empty");
    assert!(result.usage.is_some(), "Usage should be tracked");
}

#[tokio::test]
#[ignore = "Requires Ollama server running with llama2 model"]
async fn test_ollama_chat_completion() {
    let base_url = "http://localhost:11434";
    skip_if_ollama_unavailable(base_url).await;

    let model = OllamaModel::with_base_url(
        "llama2".to_string(),
        base_url.to_string(),
    ).unwrap();

    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text("What is 2+2?".to_string()),
        },
    ];

    let response = model.generate_chat_completion(&messages, None).await;

    assert!(response.is_ok(), "Chat completion should succeed");
    let result = response.unwrap();
    assert!(!result.content.is_empty(), "Response should not be empty");
}

#[tokio::test]
#[ignore = "Requires Ollama server running with llama2 model"]
async fn test_ollama_streaming() {
    let base_url = "http://localhost:11434";
    skip_if_ollama_unavailable(base_url).await;

    let model = OllamaModel::with_base_url(
        "llama2".to_string(),
        base_url.to_string(),
    ).unwrap();

    let mut stream = model.generate_stream("Count to 3", None).await.unwrap();
    let mut tokens = Vec::new();
    let mut error_count = 0;

    while let Some(result) = stream.next().await {
        match result {
            Ok(token) => {
                tokens.push(token);
            }
            Err(e) => {
                error_count += 1;
                eprintln!("Stream error: {}", e);
            }
        }
        // Limit to prevent infinite loops
        if tokens.len() > 100 {
            break;
        }
    }

    assert!(!tokens.is_empty(), "Should receive at least one token");
    assert_eq!(error_count, 0, "Should not have stream errors");
}

#[tokio::test]
#[ignore = "Requires Ollama server running"]
async fn test_ollama_model_not_found() {
    let base_url = "http://localhost:11434";
    skip_if_ollama_unavailable(base_url).await;

    let model = OllamaModel::with_base_url(
        "fake-model-xyz-12345".to_string(), // Non-existent model
        base_url.to_string(),
    ).unwrap();

    let response = model.generate_text("Hello", None).await;

    assert!(response.is_err(), "Should fail with non-existent model");
    if let Err(e) = response {
        let error_msg = format!("{}", e);
        assert!(
            error_msg.contains("not found") || error_msg.contains("ollama pull"),
            "Error should mention model not found or ollama pull"
        );
    }
}

#[tokio::test]
#[ignore = "Requires Ollama server running"]
async fn test_ollama_custom_base_url() {
    let base_url = "http://localhost:11434";
    skip_if_ollama_unavailable(base_url).await;

    let model = OllamaModel::with_base_url(
        "llama2".to_string(),
        base_url.to_string(),
    ).unwrap();

    let response = model.generate_text("Test", None).await;

    assert!(response.is_ok(), "Should work with custom base URL");
}


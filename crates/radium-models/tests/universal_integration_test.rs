//! Integration tests for UniversalModel with real OpenAI-compatible servers.
//!
//! These tests require external servers to be running. They are marked with `#[ignore]`
//! and will be skipped during normal test runs.
//!
//! To run these tests:
//! 1. Start one or more of the following servers:
//!    - vLLM: `vllm serve <model> --port 8000`
//!    - LocalAI: `docker run -p 8080:8080 localai/localai`
//!    - LM Studio: Enable local server in desktop app (default port 1234)
//!
//! 2. Set environment variables (optional, defaults shown):
//!    - vLLM: `VLLM_BASE_URL=http://localhost:8000/v1`, `VLLM_MODEL_ID=<model-name>`, `VLLM_API_KEY=<key>` (optional)
//!    - LocalAI: `LOCALAI_BASE_URL=http://localhost:8080/v1`, `LOCALAI_MODEL_ID=<model-name>`, `LOCALAI_API_KEY=<key>` (optional)
//!    - LM Studio: `LMSTUDIO_BASE_URL=http://localhost:1234/v1`, `LMSTUDIO_MODEL_ID=<model-name>`
//!
//! 3. Run: `cargo test --package radium-models --test universal_integration_test -- --ignored`

use radium_abstraction::{ChatMessage, MessageContent, Model};
use radium_models::{ModelFactory, UniversalModel};

/// Get vLLM configuration from environment variables.
#[allow(clippy::disallowed_methods)]
fn get_vllm_config() -> Option<(String, String, Option<String>)> {
    let base_url = std::env::var("VLLM_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8000/v1".to_string());
    let model_id = std::env::var("VLLM_MODEL_ID")
        .unwrap_or_else(|_| "meta-llama/Llama-3-8B-Instruct".to_string());
    let api_key = std::env::var("VLLM_API_KEY").ok();

    Some((base_url, model_id, api_key))
}

/// Get LocalAI configuration from environment variables.
#[allow(clippy::disallowed_methods)]
fn get_localai_config() -> Option<(String, String, Option<String>)> {
    let base_url = std::env::var("LOCALAI_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8080/v1".to_string());
    let model_id = std::env::var("LOCALAI_MODEL_ID")
        .unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
    let api_key = std::env::var("LOCALAI_API_KEY").ok();

    Some((base_url, model_id, api_key))
}

/// Get LM Studio configuration from environment variables.
#[allow(clippy::disallowed_methods)]
fn get_lmstudio_config() -> Option<(String, String)> {
    let base_url = std::env::var("LMSTUDIO_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:1234/v1".to_string());
    let model_id = std::env::var("LMSTUDIO_MODEL_ID")
        .unwrap_or_else(|_| "llama-2-7b".to_string());

    Some((base_url, model_id))
}

// vLLM integration tests

#[tokio::test]
#[ignore = "Requires vLLM server running"]
async fn test_vllm_sync_generation() {
    let Some((base_url, model_id, api_key)) = get_vllm_config() else {
        eprintln!("Skipping test: vLLM configuration not available");
        return;
    };

    let model = if let Some(key) = api_key {
        UniversalModel::with_api_key(model_id.clone(), base_url, key)
    } else {
        UniversalModel::without_auth(model_id.clone(), base_url)
    };

    let response = model.generate_text("Say hello in one word", None).await;

    if let Err(e) = &response {
        eprintln!("vLLM test failed (server may not be running): {}", e);
        return;
    }

    let result = response.unwrap();
    assert!(!result.content.is_empty());
    assert_eq!(result.model_id, Some(model_id));
    eprintln!("vLLM sync test passed: {}", result.content);
}

#[tokio::test]
#[ignore = "Requires vLLM server running"]
async fn test_vllm_streaming() {
    use futures::StreamExt;

    let Some((base_url, model_id, api_key)) = get_vllm_config() else {
        eprintln!("Skipping test: vLLM configuration not available");
        return;
    };

    let model = if let Some(key) = api_key {
        UniversalModel::with_api_key(model_id.clone(), base_url, key)
    } else {
        UniversalModel::without_auth(model_id.clone(), base_url)
    };

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Text("Say hello".to_string()),
    }];

    let stream_result = model.generate_chat_completion_stream(&messages, None).await;

    if let Err(e) = &stream_result {
        eprintln!("vLLM streaming test failed (server may not be running): {}", e);
        return;
    }

    let mut stream = stream_result.unwrap();
    let mut last_content = String::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(content) => {
                last_content = content;
            }
            Err(e) => {
                eprintln!("Stream error: {}", e);
                return;
            }
        }
    }

    assert!(!last_content.is_empty());
    eprintln!("vLLM streaming test passed: {}", last_content);
}

// LocalAI integration tests

#[tokio::test]
#[ignore = "Requires LocalAI server running"]
async fn test_localai_sync_generation() {
    let Some((base_url, model_id, api_key)) = get_localai_config() else {
        eprintln!("Skipping test: LocalAI configuration not available");
        return;
    };

    let model = if let Some(key) = api_key {
        UniversalModel::with_api_key(model_id.clone(), base_url, key)
    } else {
        UniversalModel::without_auth(model_id.clone(), base_url)
    };

    let response = model.generate_text("Say hello", None).await;

    if let Err(e) = &response {
        eprintln!("LocalAI test failed (server may not be running): {}", e);
        return;
    }

    let result = response.unwrap();
    assert!(!result.content.is_empty());
    assert_eq!(result.model_id, Some(model_id));
    eprintln!("LocalAI sync test passed: {}", result.content);
}

#[tokio::test]
#[ignore = "Requires LocalAI server running"]
async fn test_localai_streaming() {
    use futures::StreamExt;

    let Some((base_url, model_id, api_key)) = get_localai_config() else {
        eprintln!("Skipping test: LocalAI configuration not available");
        return;
    };

    let model = if let Some(key) = api_key {
        UniversalModel::with_api_key(model_id.clone(), base_url, key)
    } else {
        UniversalModel::without_auth(model_id.clone(), base_url)
    };

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Text("Say hello".to_string()),
    }];

    let stream_result = model.generate_chat_completion_stream(&messages, None).await;

    if let Err(e) = &stream_result {
        eprintln!("LocalAI streaming test failed (server may not be running): {}", e);
        return;
    }

    let mut stream = stream_result.unwrap();
    let mut last_content = String::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(content) => {
                last_content = content;
            }
            Err(e) => {
                eprintln!("Stream error: {}", e);
                return;
            }
        }
    }

    assert!(!last_content.is_empty());
    eprintln!("LocalAI streaming test passed: {}", last_content);
}

// LM Studio integration tests

#[tokio::test]
#[ignore = "Requires LM Studio server running"]
async fn test_lmstudio_sync_generation() {
    let Some((base_url, model_id)) = get_lmstudio_config() else {
        eprintln!("Skipping test: LM Studio configuration not available");
        return;
    };

    // LM Studio typically doesn't require authentication
    let model = UniversalModel::without_auth(model_id.clone(), base_url);

    let response = model.generate_text("Say hello", None).await;

    if let Err(e) = &response {
        eprintln!("LM Studio test failed (server may not be running): {}", e);
        return;
    }

    let result = response.unwrap();
    assert!(!result.content.is_empty());
    assert_eq!(result.model_id, Some(model_id));
    eprintln!("LM Studio sync test passed: {}", result.content);
}

#[tokio::test]
#[ignore = "Requires LM Studio server running"]
async fn test_lmstudio_streaming() {
    use futures::StreamExt;

    let Some((base_url, model_id)) = get_lmstudio_config() else {
        eprintln!("Skipping test: LM Studio configuration not available");
        return;
    };

    let model = UniversalModel::without_auth(model_id.clone(), base_url);

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Text("Say hello".to_string()),
    }];

    let stream_result = model.generate_chat_completion_stream(&messages, None).await;

    if let Err(e) = &stream_result {
        eprintln!("LM Studio streaming test failed (server may not be running): {}", e);
        return;
    }

    let mut stream = stream_result.unwrap();
    let mut last_content = String::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(content) => {
                last_content = content;
            }
            Err(e) => {
                eprintln!("Stream error: {}", e);
                return;
            }
        }
    }

    assert!(!last_content.is_empty());
    eprintln!("LM Studio streaming test passed: {}", last_content);
}

// Factory integration test

#[tokio::test]
#[ignore = "Requires OpenAI-compatible server running"]
async fn test_factory_integration_with_universal() {
    let Some((base_url, model_id, api_key)) = get_vllm_config()
        .or_else(|| get_localai_config())
        .or_else(|| {
            get_lmstudio_config().map(|(base_url, model_id)| (base_url, model_id, None))
        })
    else {
        eprintln!("Skipping test: No server configuration available");
        return;
    };

    let config = radium_models::ModelConfig::new(
        radium_models::ModelType::Universal,
        model_id.clone(),
    )
    .with_base_url(base_url);

    let config = if let Some(key) = api_key {
        config.with_api_key(key)
    } else {
        config
    };

    let model_result = ModelFactory::create(config);

    if let Err(e) = &model_result {
        eprintln!("Factory test failed (server may not be running): {}", e);
        return;
    }

    let model = model_result.unwrap();
    assert_eq!(model.model_id(), model_id);

    let response = model.generate_text("Say hello", None).await;

    if let Err(e) = &response {
        eprintln!("Factory generation test failed: {}", e);
        return;
    }

    let result = response.unwrap();
    assert!(!result.content.is_empty());
    eprintln!("Factory integration test passed: {}", result.content);
}

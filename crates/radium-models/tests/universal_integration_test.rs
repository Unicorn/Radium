//! Integration tests for UniversalModel with real OpenAI-compatible servers.
//!
//! These tests require external servers to be running and configured via environment variables.
//! All tests are marked with `#[ignore]` and should be run with:
//! ```bash
//! cargo test -- --ignored
//! ```
//!
//! # Setup Instructions
//!
//! ## vLLM
//! ```bash
//! pip install vllm
//! vllm serve meta-llama/Llama-3-8B-Instruct --port 8000
//! export VLLM_BASE_URL=http://localhost:8000/v1
//! export VLLM_MODEL_ID=meta-llama/Llama-3-8B-Instruct
//! export VLLM_API_KEY=optional-key-if-needed
//! ```
//!
//! ## LocalAI
//! ```bash
//! docker run -p 8080:8080 localai/localai
//! export LOCALAI_BASE_URL=http://localhost:8080/v1
//! export LOCALAI_MODEL_ID=gpt-3.5-turbo
//! export LOCALAI_API_KEY=optional-key-if-needed
//! ```
//!
//! ## LM Studio
//! 1. Install LM Studio desktop app
//! 2. Download a model via the UI
//! 3. Start local server in settings
//! 4. Export environment variables:
//! ```bash
//! export LMSTUDIO_BASE_URL=http://localhost:1234/v1
//! export LMSTUDIO_MODEL_ID=llama-2-7b
//! ```

use radium_abstraction::ChatMessage;
use radium_models::UniversalModel;
use std::env;

fn get_vllm_config() -> Option<(String, String, Option<String>)> {
    let base_url = env::var("VLLM_BASE_URL").ok()?;
    let model_id = env::var("VLLM_MODEL_ID").ok()?;
    let api_key = env::var("VLLM_API_KEY").ok();
    Some((base_url, model_id, api_key))
}

fn get_localai_config() -> Option<(String, String, Option<String>)> {
    let base_url = env::var("LOCALAI_BASE_URL").ok()?;
    let model_id = env::var("LOCALAI_MODEL_ID").ok()?;
    let api_key = env::var("LOCALAI_API_KEY").ok();
    Some((base_url, model_id, api_key))
}

fn get_lmstudio_config() -> Option<(String, String)> {
    let base_url = env::var("LMSTUDIO_BASE_URL").ok()?;
    let model_id = env::var("LMSTUDIO_MODEL_ID").ok()?;
    Some((base_url, model_id))
}

#[tokio::test]
#[ignore = "Requires vLLM server running"]
async fn test_vllm_sync() {
    let Some((base_url, model_id, api_key)) = get_vllm_config() else {
        eprintln!("Skipping test: VLLM_BASE_URL and VLLM_MODEL_ID not set");
        return;
    };

    let model = if let Some(key) = api_key {
        UniversalModel::with_api_key(model_id, base_url, key)
    } else {
        UniversalModel::without_auth(model_id, base_url)
    };

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: "Say hello in one word".to_string(),
    }];

    let response = model
        .generate_chat_completion(&messages, None)
        .await
        .expect("Should generate response from vLLM");

    assert!(!response.content.is_empty());
    println!("vLLM response: {}", response.content);
}

#[tokio::test]
#[ignore = "Requires vLLM server running"]
async fn test_vllm_streaming() {
    use futures::StreamExt;

    let Some((base_url, model_id, api_key)) = get_vllm_config() else {
        eprintln!("Skipping test: VLLM_BASE_URL and VLLM_MODEL_ID not set");
        return;
    };

    let model = if let Some(key) = api_key {
        UniversalModel::with_api_key(model_id, base_url, key)
    } else {
        UniversalModel::without_auth(model_id, base_url)
    };

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: "Count to 5".to_string(),
    }];

    let mut stream = model
        .generate_chat_completion_stream(&messages, None)
        .await
        .expect("Should create streaming request");

    let mut last_content = String::new();
    while let Some(result) = stream.next().await {
        let content = result.expect("Should receive streaming content");
        assert!(!content.is_empty());
        assert!(content.len() >= last_content.len()); // Content should accumulate
        last_content = content;
    }

    assert!(!last_content.is_empty());
    println!("vLLM streaming response: {}", last_content);
}

#[tokio::test]
#[ignore = "Requires LocalAI server running"]
async fn test_localai_sync() {
    let Some((base_url, model_id, api_key)) = get_localai_config() else {
        eprintln!("Skipping test: LOCALAI_BASE_URL and LOCALAI_MODEL_ID not set");
        return;
    };

    let model = if let Some(key) = api_key {
        UniversalModel::with_api_key(model_id, base_url, key)
    } else {
        UniversalModel::without_auth(model_id, base_url)
    };

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: "Say hello".to_string(),
    }];

    let response = model
        .generate_chat_completion(&messages, None)
        .await
        .expect("Should generate response from LocalAI");

    assert!(!response.content.is_empty());
    println!("LocalAI response: {}", response.content);
}

#[tokio::test]
#[ignore = "Requires LocalAI server running"]
async fn test_localai_streaming() {
    use futures::StreamExt;

    let Some((base_url, model_id, api_key)) = get_localai_config() else {
        eprintln!("Skipping test: LOCALAI_BASE_URL and LOCALAI_MODEL_ID not set");
        return;
    };

    let model = if let Some(key) = api_key {
        UniversalModel::with_api_key(model_id, base_url, key)
    } else {
        UniversalModel::without_auth(model_id, base_url)
    };

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: "Say hello".to_string(),
    }];

    let mut stream = model
        .generate_chat_completion_stream(&messages, None)
        .await
        .expect("Should create streaming request");

    let mut last_content = String::new();
    while let Some(result) = stream.next().await {
        let content = result.expect("Should receive streaming content");
        last_content = content;
    }

    assert!(!last_content.is_empty());
    println!("LocalAI streaming response: {}", last_content);
}

#[tokio::test]
#[ignore = "Requires LM Studio server running"]
async fn test_lmstudio_sync() {
    let Some((base_url, model_id)) = get_lmstudio_config() else {
        eprintln!("Skipping test: LMSTUDIO_BASE_URL and LMSTUDIO_MODEL_ID not set");
        return;
    };

    let model = UniversalModel::without_auth(model_id, base_url);

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: "Say hello".to_string(),
    }];

    let response = model
        .generate_chat_completion(&messages, None)
        .await
        .expect("Should generate response from LM Studio");

    assert!(!response.content.is_empty());
    println!("LM Studio response: {}", response.content);
}

#[tokio::test]
#[ignore = "Requires LM Studio server running"]
async fn test_lmstudio_streaming() {
    use futures::StreamExt;

    let Some((base_url, model_id)) = get_lmstudio_config() else {
        eprintln!("Skipping test: LMSTUDIO_BASE_URL and LMSTUDIO_MODEL_ID not set");
        return;
    };

    let model = UniversalModel::without_auth(model_id, base_url);

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: "Say hello".to_string(),
    }];

    let mut stream = model
        .generate_chat_completion_stream(&messages, None)
        .await
        .expect("Should create streaming request");

    let mut last_content = String::new();
    while let Some(result) = stream.next().await {
        let content = result.expect("Should receive streaming content");
        last_content = content;
    }

    assert!(!last_content.is_empty());
    println!("LM Studio streaming response: {}", last_content);
}

#[tokio::test]
#[ignore = "Requires a real server running"]
async fn test_factory_integration() {
    use radium_models::{ModelConfig, ModelFactory, ModelType};

    let Some((base_url, model_id, _)) = get_vllm_config()
        .or_else(|| get_localai_config())
        .or_else(|| {
            get_lmstudio_config().map(|(b, m)| (b, m, None))
        })
    else {
        eprintln!("Skipping test: No server configuration found");
        return;
    };

    let config = ModelConfig::new(ModelType::Universal, model_id)
        .with_base_url(base_url);

    let model = ModelFactory::create(config)
        .expect("Should create Universal model via factory");

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: "Say hello".to_string(),
    }];

    let response = model
        .generate_chat_completion(&messages, None)
        .await
        .expect("Should generate response via factory-created model");

    assert!(!response.content.is_empty());
    println!("Factory integration response: {}", response.content);
}


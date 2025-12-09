//! Dedicated test suite for OllamaEngine.

use radium_core::engines::{
    Engine, EngineRegistry, ExecutionRequest, HealthStatus,
};
use radium_core::engines::providers::OllamaEngine;
use std::sync::Arc;

#[tokio::test]
async fn test_ollama_engine_registration() {
    let registry = EngineRegistry::new();
    let engine = Arc::new(OllamaEngine::new());

    registry.register(engine).unwrap();

    let retrieved = registry.get("ollama").unwrap();
    assert_eq!(retrieved.metadata().id, "ollama");
    assert_eq!(retrieved.metadata().name, "Ollama");
    assert!(!retrieved.metadata().requires_auth);
}

#[tokio::test]
async fn test_ollama_engine_default_model() {
    let engine = OllamaEngine::new();
    assert_eq!(engine.default_model(), "llama2:latest");
}

#[tokio::test]
async fn test_ollama_engine_is_authenticated() {
    let engine = OllamaEngine::new();
    let result = engine.is_authenticated().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

#[tokio::test]
async fn test_ollama_engine_available_models() {
    let engine = OllamaEngine::new();
    // Initially, available_models should return empty vec
    // (models haven't been fetched yet)
    let models = engine.available_models();
    assert!(models.is_empty());
}

#[tokio::test]
async fn test_ollama_engine_is_available() {
    let engine = OllamaEngine::new();
    // This test depends on whether Ollama is actually running
    // We just verify the method doesn't panic
    let available = engine.is_available().await;
    assert!(available == true || available == false);
}

#[tokio::test]
async fn test_ollama_engine_check_server_health() {
    let engine = OllamaEngine::new();
    // This test depends on whether Ollama is actually running
    // We just verify the method doesn't panic and returns a Result
    let result = engine.check_server_health().await;
    // Result should be Ok(version) if server is running, Err otherwise
    match result {
        Ok(version) => {
            // Server is running, version should be a non-empty string
            assert!(!version.is_empty());
        }
        Err(_) => {
            // Server is not running, which is fine for this test
        }
    }
}

#[tokio::test]
async fn test_ollama_engine_in_registry_health_check() {
    let registry = EngineRegistry::new();
    let engine = Arc::new(OllamaEngine::new());
    registry.register(engine).unwrap();

    let health_results = registry.check_health(5).await;
    // Should have at least one result (Ollama)
    assert!(!health_results.is_empty());
    
    // Find Ollama in results
    let ollama_health = health_results.iter().find(|h| h.engine_id == "ollama");
    assert!(ollama_health.is_some());
    
    let health = ollama_health.unwrap();
    assert_eq!(health.engine_id, "ollama");
    assert_eq!(health.engine_name, "Ollama");
    // Status depends on whether Ollama server is running
    // We just verify it's one of the expected statuses
    match &health.status {
        HealthStatus::Healthy => {}
        HealthStatus::Warning(_) => {}
        HealthStatus::Failed(_) => {}
    }
}

#[test]
fn test_ollama_format_size() {
    assert_eq!(OllamaEngine::format_size(3826793677), "3.8 GB");
    assert_eq!(OllamaEngine::format_size(512000000), "512.0 MB");
    assert_eq!(OllamaEngine::format_size(1024000), "1.0 MB");
    assert_eq!(OllamaEngine::format_size(512000), "512.0 KB");
    assert_eq!(OllamaEngine::format_size(1024), "1.0 KB");
    assert_eq!(OllamaEngine::format_size(512), "512 B");
}


//! Integration tests for engine abstraction layer.

use radium_core::engines::{
    Engine, EngineRegistry, ExecutionRequest, HealthStatus,
};
use radium_core::engines::providers::MockEngine;
use std::sync::Arc;

#[tokio::test]
async fn test_engine_registration_and_retrieval() {
    let registry = EngineRegistry::new();
    let engine = Arc::new(MockEngine::new());

    // Register engine
    registry.register(engine.clone()).unwrap();

    // Retrieve engine
    let retrieved = registry.get("mock").unwrap();
    assert_eq!(retrieved.metadata().id, "mock");
}

#[tokio::test]
async fn test_default_engine_selection() {
    let registry = EngineRegistry::new();
    let engine1 = Arc::new(MockEngine::new());
    let engine2 = Arc::new(MockEngine::new());

    // Register engines
    registry.register(engine1).unwrap();
    // Note: MockEngine always has id "mock", so we can't register two different ones
    // This test verifies default selection works

    // Set default
    registry.set_default("mock").unwrap();

    // Get default
    let default = registry.get_default().unwrap();
    assert_eq!(default.metadata().id, "mock");
}

#[tokio::test]
async fn test_engine_execution() {
    let registry = EngineRegistry::new();
    let engine = Arc::new(MockEngine::new());

    registry.register(engine).unwrap();

    let engine = registry.get("mock").unwrap();
    let request = ExecutionRequest::new("mock-model".to_string(), "Hello!".to_string());

    let response = engine.execute(request).await.unwrap();

    assert!(!response.content.is_empty());
    assert_eq!(response.model, "mock-model");
}

#[tokio::test]
async fn test_engine_health_check() {
    let registry = EngineRegistry::new();
    let engine = Arc::new(MockEngine::new());

    registry.register(engine).unwrap();

    let health_results = registry.check_health(5).await;
    assert_eq!(health_results.len(), 1);

    let health = &health_results[0];
    assert_eq!(health.engine_id, "mock");
    assert!(matches!(health.status, HealthStatus::Healthy));
    assert!(health.available);
}

#[tokio::test]
async fn test_engine_list() {
    let registry = EngineRegistry::new();
    let engine = Arc::new(MockEngine::new());

    registry.register(engine).unwrap();

    let engines = registry.list().unwrap();
    assert_eq!(engines.len(), 1);
    assert_eq!(engines[0].id, "mock");
}

#[tokio::test]
async fn test_config_persistence() {
    use tempfile::TempDir;
    use std::fs;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Create registry with config path
    let registry = EngineRegistry::with_config_path(&config_path);
    let engine = Arc::new(MockEngine::new());
    registry.register(engine).unwrap();

    // Set default
    registry.set_default("mock").unwrap();

    // Verify config file was created
    assert!(config_path.exists());

    // Create new registry and load config
    let registry2 = EngineRegistry::with_config_path(&config_path);
    let engine2 = Arc::new(MockEngine::new());
    registry2.register(engine2).unwrap();
    registry2.load_config().unwrap();

    // Verify default was loaded
    let default = registry2.get_default().unwrap();
    assert_eq!(default.metadata().id, "mock");
}

#[tokio::test]
async fn test_concurrent_engine_usage() {
    let registry = EngineRegistry::new();
    let engine = Arc::new(MockEngine::new());
    registry.register(engine).unwrap();

    // Get engine once and clone for concurrent use
    let engine = registry.get("mock").unwrap();

    // Spawn multiple concurrent executions using the same engine
    let mut handles = Vec::new();
    for i in 0..10 {
        let engine_clone = engine.clone();
        let handle = tokio::spawn(async move {
            let request = ExecutionRequest::new(
                format!("model-{}", i),
                format!("Request {}", i),
            );
            engine_clone.execute(request).await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}


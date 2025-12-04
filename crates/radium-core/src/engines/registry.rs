//! Engine registry for managing available engines.

use super::engine_trait::{Engine, EngineMetadata};
use super::error::{EngineError, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Engine registry for managing available engines.
pub struct EngineRegistry {
    /// Registered engines.
    engines: Arc<RwLock<HashMap<String, Arc<dyn Engine>>>>,

    /// Default engine ID.
    default_engine: Arc<RwLock<Option<String>>>,
}

impl EngineRegistry {
    /// Creates a new engine registry.
    pub fn new() -> Self {
        Self {
            engines: Arc::new(RwLock::new(HashMap::new())),
            default_engine: Arc::new(RwLock::new(None)),
        }
    }

    /// Registers an engine.
    ///
    /// # Arguments
    /// * `engine` - Engine to register
    ///
    /// # Errors
    /// Returns error if lock poisoned
    pub fn register(&self, engine: Arc<dyn Engine>) -> Result<()> {
        let id = engine.metadata().id.clone();
        let mut engines = self
            .engines
            .write()
            .map_err(|e| EngineError::RegistryError(format!("Lock poisoned: {}", e)))?;

        engines.insert(id, engine);
        Ok(())
    }

    /// Gets an engine by ID.
    ///
    /// # Arguments
    /// * `id` - Engine identifier
    ///
    /// # Returns
    /// Engine if found
    ///
    /// # Errors
    /// Returns error if engine not found or lock poisoned
    pub fn get(&self, id: &str) -> Result<Arc<dyn Engine>> {
        let engines = self
            .engines
            .read()
            .map_err(|e| EngineError::RegistryError(format!("Lock poisoned: {}", e)))?;

        engines.get(id).cloned().ok_or_else(|| EngineError::NotFound(id.to_string()))
    }

    /// Sets the default engine.
    ///
    /// # Arguments
    /// * `id` - Engine identifier
    ///
    /// # Errors
    /// Returns error if engine not found or lock poisoned
    pub fn set_default(&self, id: &str) -> Result<()> {
        // Verify engine exists
        self.get(id)?;

        let mut default = self
            .default_engine
            .write()
            .map_err(|e| EngineError::RegistryError(format!("Lock poisoned: {}", e)))?;

        *default = Some(id.to_string());
        Ok(())
    }

    /// Gets the default engine.
    ///
    /// # Returns
    /// Default engine if set
    ///
    /// # Errors
    /// Returns error if no default set or lock poisoned
    pub fn get_default(&self) -> Result<Arc<dyn Engine>> {
        let default = self
            .default_engine
            .read()
            .map_err(|e| EngineError::RegistryError(format!("Lock poisoned: {}", e)))?;

        let id = default
            .as_ref()
            .ok_or_else(|| EngineError::NotFound("No default engine set".to_string()))?;

        self.get(id)
    }

    /// Lists all registered engines.
    ///
    /// # Returns
    /// List of engine metadata
    ///
    /// # Errors
    /// Returns error if lock poisoned
    pub fn list(&self) -> Result<Vec<EngineMetadata>> {
        let engines = self
            .engines
            .read()
            .map_err(|e| EngineError::RegistryError(format!("Lock poisoned: {}", e)))?;

        Ok(engines.values().map(|engine| engine.metadata().clone()).collect())
    }

    /// Checks if an engine is registered.
    ///
    /// # Arguments
    /// * `id` - Engine identifier
    ///
    /// # Returns
    /// True if engine is registered
    pub fn has(&self, id: &str) -> bool {
        self.engines.read().map(|engines| engines.contains_key(id)).unwrap_or(false)
    }

    /// Removes an engine.
    ///
    /// # Arguments
    /// * `id` - Engine identifier
    ///
    /// # Errors
    /// Returns error if lock poisoned
    pub fn unregister(&self, id: &str) -> Result<()> {
        let mut engines = self
            .engines
            .write()
            .map_err(|e| EngineError::RegistryError(format!("Lock poisoned: {}", e)))?;

        engines.remove(id);

        // Clear default if it was this engine
        let default = self.default_engine.read().ok();
        if let Some(default) = default {
            if default.as_ref() == Some(&id.to_string()) {
                drop(default);
                if let Ok(mut default_write) = self.default_engine.write() {
                    *default_write = None;
                }
            }
        }

        Ok(())
    }

    /// Gets the number of registered engines.
    pub fn count(&self) -> usize {
        self.engines.read().map(|e| e.len()).unwrap_or(0)
    }
}

impl Default for EngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::engine_trait::{ExecutionRequest, ExecutionResponse};
    use async_trait::async_trait;

    // Mock engine for testing
    struct MockEngine {
        metadata: EngineMetadata,
    }

    impl MockEngine {
        fn new(id: &str) -> Self {
            Self {
                metadata: EngineMetadata::new(
                    id.to_string(),
                    format!("Mock {}", id),
                    "A mock engine".to_string(),
                ),
            }
        }
    }

    #[async_trait]
    impl Engine for MockEngine {
        fn metadata(&self) -> &EngineMetadata {
            &self.metadata
        }

        async fn is_available(&self) -> bool {
            true
        }

        async fn is_authenticated(&self) -> Result<bool> {
            Ok(true)
        }

        async fn execute(&self, _request: ExecutionRequest) -> Result<ExecutionResponse> {
            Ok(ExecutionResponse {
                content: "mock response".to_string(),
                usage: None,
                model: "mock-model".to_string(),
                raw: None,
            })
        }

        fn default_model(&self) -> String {
            "mock-model".to_string()
        }
    }

    #[test]
    fn test_registry_new() {
        let registry = EngineRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let registry = EngineRegistry::new();
        let engine = Arc::new(MockEngine::new("test-engine"));

        registry.register(engine).unwrap();
        assert_eq!(registry.count(), 1);
        assert!(registry.has("test-engine"));
    }

    #[test]
    fn test_registry_get() {
        let registry = EngineRegistry::new();
        let engine = Arc::new(MockEngine::new("test-engine"));

        registry.register(engine).unwrap();

        let retrieved = registry.get("test-engine").unwrap();
        assert_eq!(retrieved.metadata().id, "test-engine");
    }

    #[test]
    fn test_registry_get_not_found() {
        let registry = EngineRegistry::new();
        let result = registry.get("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_set_default() {
        let registry = EngineRegistry::new();
        let engine = Arc::new(MockEngine::new("test-engine"));

        registry.register(engine).unwrap();
        registry.set_default("test-engine").unwrap();

        let default = registry.get_default().unwrap();
        assert_eq!(default.metadata().id, "test-engine");
    }

    #[test]
    fn test_registry_list() {
        let registry = EngineRegistry::new();
        let engine1 = Arc::new(MockEngine::new("engine-1"));
        let engine2 = Arc::new(MockEngine::new("engine-2"));

        registry.register(engine1).unwrap();
        registry.register(engine2).unwrap();

        let list = registry.list().unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_registry_unregister() {
        let registry = EngineRegistry::new();
        let engine = Arc::new(MockEngine::new("test-engine"));

        registry.register(engine).unwrap();
        assert_eq!(registry.count(), 1);

        registry.unregister("test-engine").unwrap();
        assert_eq!(registry.count(), 0);
        assert!(!registry.has("test-engine"));
    }

    #[test]
    fn test_registry_unregister_default() {
        let registry = EngineRegistry::new();
        let engine = Arc::new(MockEngine::new("test-engine"));

        registry.register(engine).unwrap();
        registry.set_default("test-engine").unwrap();

        registry.unregister("test-engine").unwrap();

        // Default should be cleared
        let result = registry.get_default();
        assert!(result.is_err());
    }
}

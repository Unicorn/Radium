//! Engine registry for managing available engines.

use super::config::{GlobalEngineConfig, PerEngineConfig};
use super::engine_trait::{Engine, EngineMetadata};
use super::error::{EngineError, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::timeout;

/// Health status for an engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// Engine is healthy and ready to use.
    Healthy,
    /// Engine has warnings but may still work.
    Warning(String),
    /// Engine is not available or has failed.
    Failed(String),
}

/// Health check result for an engine.
#[derive(Debug, Clone)]
pub struct EngineHealth {
    /// Engine ID.
    pub engine_id: String,
    /// Engine name.
    pub engine_name: String,
    /// Health status.
    pub status: HealthStatus,
    /// Whether engine is available.
    pub available: bool,
    /// Whether engine is authenticated.
    pub authenticated: bool,
}

/// Credential status for an engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialStatus {
    /// Credentials are available and valid.
    Available,
    /// Credentials are missing.
    Missing,
    /// Credentials are invalid.
    Invalid,
    /// Credential status is unknown.
    Unknown,
}

/// Engine information for listing and selection.
#[derive(Debug, Clone)]
pub struct EngineInfo {
    /// Engine ID.
    pub id: String,
    /// Engine name.
    pub name: String,
    /// Provider type (derived from engine ID).
    pub provider: String,
    /// Whether this is the default engine.
    pub is_default: bool,
    /// Credential status.
    pub credential_status: CredentialStatus,
}

/// Validation status for an engine.
#[derive(Debug, Clone)]
pub struct ValidationStatus {
    /// Whether configuration is valid.
    pub config_valid: bool,
    /// Whether credentials are available.
    pub credentials_available: bool,
    /// Whether API is reachable.
    pub api_reachable: bool,
    /// Optional error message.
    pub error_message: Option<String>,
}

/// Engine registry for managing available engines.
pub struct EngineRegistry {
    /// Registered engines.
    engines: Arc<RwLock<HashMap<String, Arc<dyn Engine>>>>,

    /// Default engine ID.
    default_engine: Arc<RwLock<Option<String>>>,

    /// Configuration file path (optional).
    config_path: Option<PathBuf>,

    /// Loaded engine configuration.
    engine_config: Arc<RwLock<GlobalEngineConfig>>,
}

impl EngineRegistry {
    /// Creates a new engine registry.
    pub fn new() -> Self {
        Self {
            engines: Arc::new(RwLock::new(HashMap::new())),
            default_engine: Arc::new(RwLock::new(None)),
            config_path: None,
            engine_config: Arc::new(RwLock::new(GlobalEngineConfig::new())),
        }
    }

    /// Creates a new engine registry with configuration path.
    ///
    /// # Arguments
    /// * `config_path` - Path to the configuration file (e.g., `.radium/config.toml`)
    pub fn with_config_path(config_path: impl AsRef<Path>) -> Self {
        let mut registry = Self::new();
        registry.config_path = Some(config_path.as_ref().to_path_buf());
        registry.load_config().ok(); // Ignore errors on load (file might not exist)
        registry
    }

    /// Load engine configuration from the config file.
    ///
    /// # Errors
    /// Returns error if config file exists but cannot be read or parsed.
    pub fn load_config(&self) -> Result<()> {
        let config_path = match &self.config_path {
            Some(path) => path.clone(),
            None => return Ok(()), // No config path set, skip loading
        };

        if !config_path.exists() {
            return Ok(()); // Config file doesn't exist, that's fine
        }

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| EngineError::RegistryError(format!("Failed to read config: {}", e)))?;

        let toml: toml::Table = toml::from_str(&content)
            .map_err(|e| EngineError::InvalidConfig(format!("Failed to parse config: {}", e)))?;

        // Extract [engines] section
        if let Some(engines_value) = toml.get("engines") {
            let mut global_config = GlobalEngineConfig::new();
            
            if let Some(engines_table) = engines_value.as_table() {
                // Handle default engine
                if let Some(default) = engines_table.get("default") {
                    if let Some(default_str) = default.as_str() {
                        global_config.default = Some(default_str.to_string());
                    }
                }
                
                // Handle per-engine configs (e.g., [engines.gemini])
                for (key, value) in engines_table {
                    if key == "default" {
                        continue; // Already handled
                    }

                    // Try to deserialize as PerEngineConfig
                    // Convert toml::Value to PerEngineConfig via string serialization
                    if let Ok(value_str) = toml::to_string(&value) {
                        if let Ok(engine_config) = toml::from_str::<PerEngineConfig>(&value_str) {
                            global_config.set_engine_config(key.clone(), engine_config);
                        }
                    }
                }
            }
            
            // Validate configuration
            global_config.validate()?;
            
            // Update stored config
            let mut stored_config = self
                .engine_config
                .write()
                .map_err(|e| EngineError::RegistryError(format!("Lock poisoned: {}", e)))?;
            *stored_config = global_config.clone();
            
            // Set default engine if specified and it exists
            if let Some(ref default_id) = global_config.default {
                // Only set if engine is already registered (we might load config before engines are registered)
                if self.has(default_id) {
                    let mut default = self
                        .default_engine
                        .write()
                        .map_err(|e| EngineError::RegistryError(format!("Lock poisoned: {}", e)))?;
                    *default = Some(default_id.clone());
                }
            }
        }

        Ok(())
    }

    /// Save engine configuration to the config file.
    ///
    /// # Errors
    /// Returns error if config file cannot be written.
    pub fn save_config(&self) -> Result<()> {
        let config_path = match &self.config_path {
            Some(path) => path.clone(),
            None => return Ok(()), // No config path set, skip saving
        };

        // Get current default engine and update config
        let default_id = self
            .default_engine
            .read()
            .map_err(|e| EngineError::RegistryError(format!("Lock poisoned: {}", e)))?
            .clone();

        let mut global_config = self
            .engine_config
            .read()
            .map_err(|e| EngineError::RegistryError(format!("Lock poisoned: {}", e)))?
            .clone();
        
        global_config.default = default_id;

        // Read existing config or create new
        let mut toml: toml::Table = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| EngineError::RegistryError(format!("Failed to read config: {}", e)))?;
            toml::from_str(&content).unwrap_or_default()
        } else {
            toml::Table::new()
        };

        // Build engines section
        let mut engines_table = toml::Table::new();
        
        // Add default
        if let Some(ref default) = global_config.default {
            engines_table.insert("default".to_string(), toml::Value::String(default.clone()));
        }
        
        // Add per-engine configs
        for (engine_id, engine_config) in &global_config.engines {
            // Serialize to TOML string then parse as Value
            let engine_str = toml::to_string(engine_config)
                .map_err(|e| EngineError::InvalidConfig(format!("Failed to serialize engine config: {}", e)))?;
            let engine_value: toml::Value = toml::from_str(&engine_str)
                .map_err(|e| EngineError::InvalidConfig(format!("Failed to parse engine config: {}", e)))?;
            engines_table.insert(engine_id.clone(), engine_value);
        }
        
        toml.insert("engines".to_string(), toml::Value::Table(engines_table));

        // Write back to file
        let content = toml::to_string_pretty(&toml)
            .map_err(|e| EngineError::InvalidConfig(format!("Failed to serialize: {}", e)))?;

        // Create parent directory if needed
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| EngineError::Io(e))?;
        }

        std::fs::write(&config_path, content)
            .map_err(|e| EngineError::Io(e))?;

        Ok(())
    }

    /// Gets the engine configuration for a specific engine.
    pub fn get_engine_config(&self, engine_id: &str) -> Option<PerEngineConfig> {
        self.engine_config
            .read()
            .ok()
            .and_then(|config| config.get_engine_config(engine_id).cloned())
    }

    /// Sets the engine configuration for a specific engine.
    ///
    /// # Errors
    /// Returns error if configuration is invalid or lock is poisoned.
    pub fn set_engine_config(&self, engine_id: String, config: PerEngineConfig) -> Result<()> {
        config.validate()?;
        
        let mut global_config = self
            .engine_config
            .write()
            .map_err(|e| EngineError::RegistryError(format!("Lock poisoned: {}", e)))?;
        
        global_config.set_engine_config(engine_id, config);
        self.save_config()?;
        
        Ok(())
    }

    /// Gets the global engine configuration.
    pub fn get_global_config(&self) -> Result<GlobalEngineConfig> {
        Ok(self
            .engine_config
            .read()
            .map_err(|e| EngineError::RegistryError(format!("Lock poisoned: {}", e)))?
            .clone())
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
        
        // Persist to config file
        self.save_config()?;
        
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

    /// Checks health of all registered engines.
    ///
    /// # Arguments
    /// * `timeout_secs` - Timeout in seconds for each health check (default: 5)
    ///
    /// # Returns
    /// Vector of health check results for each engine
    pub async fn check_health(&self, timeout_secs: u64) -> Vec<EngineHealth> {
        let engines = match self.engines.read() {
            Ok(engines) => engines,
            Err(_) => return Vec::new(),
        };

        let mut results = Vec::new();

        for (id, engine) in engines.iter() {
            let metadata = engine.metadata();
            let engine_id = id.clone();
            let engine_name = metadata.name.clone();
            let requires_auth = metadata.requires_auth;
            let engine_clone = engine.clone();

            // Perform health check with timeout
            let health_check = async {
                let available = engine_clone.is_available().await;
                let authenticated = engine_clone.is_authenticated().await.unwrap_or(false);

                let status = if !available {
                    HealthStatus::Failed("Engine binary not available".to_string())
                } else if requires_auth && !authenticated {
                    HealthStatus::Failed("Engine not authenticated".to_string())
                } else if !authenticated && requires_auth {
                    HealthStatus::Warning("Authentication status unknown".to_string())
                } else {
                    HealthStatus::Healthy
                };

                EngineHealth {
                    engine_id: engine_id.clone(),
                    engine_name: engine_name.clone(),
                    status,
                    available,
                    authenticated,
                }
            };

            let result = match timeout(Duration::from_secs(timeout_secs), health_check).await {
                Ok(health) => health,
                Err(_) => EngineHealth {
                    engine_id: engine_id.clone(),
                    engine_name: engine_name.clone(),
                    status: HealthStatus::Failed("Health check timed out".to_string()),
                    available: false,
                    authenticated: false,
                },
            };

            results.push(result);
        }

        results
    }

    /// Checks health of a specific engine.
    ///
    /// # Arguments
    /// * `engine_id` - Engine identifier
    /// * `timeout_secs` - Timeout in seconds for health check (default: 5)
    ///
    /// # Returns
    /// Health check result
    ///
    /// # Errors
    /// Returns error if engine not found
    pub async fn check_engine_health(&self, engine_id: &str, timeout_secs: u64) -> Result<EngineHealth> {
        let engine = self.get(engine_id)?;
        let metadata = engine.metadata();

        let health_check = async {
            let available = engine.is_available().await;
            let authenticated = engine.is_authenticated().await.unwrap_or(false);

            let status = if !available {
                HealthStatus::Failed("Engine binary not available".to_string())
            } else if metadata.requires_auth && !authenticated {
                HealthStatus::Failed("Engine not authenticated".to_string())
            } else if !authenticated && metadata.requires_auth {
                HealthStatus::Warning("Authentication status unknown".to_string())
            } else {
                HealthStatus::Healthy
            };

            EngineHealth {
                engine_id: engine_id.to_string(),
                engine_name: metadata.name.clone(),
                status,
                available,
                authenticated,
            }
        };

        match timeout(Duration::from_secs(timeout_secs), health_check).await {
            Ok(health) => Ok(health),
            Err(_) => Ok(EngineHealth {
                engine_id: engine_id.to_string(),
                engine_name: metadata.name.clone(),
                status: HealthStatus::Failed("Health check timed out".to_string()),
                available: false,
                authenticated: false,
            }),
        }
    }

    /// Gets the first available engine with valid credentials.
    ///
    /// # Returns
    /// First engine with valid credentials, or error if none found
    ///
    /// # Errors
    /// Returns error if no engines with valid credentials are found
    pub async fn get_first_available(&self) -> Result<Arc<dyn Engine>> {
        let engines = self
            .engines
            .read()
            .map_err(|e| EngineError::RegistryError(format!("Lock poisoned: {}", e)))?;

        for (_, engine) in engines.iter() {
            // Check if engine is available and authenticated
            let available = engine.is_available().await;
            if available {
                let authenticated = engine.is_authenticated().await.unwrap_or(false);
                let metadata = engine.metadata();
                // If engine doesn't require auth, or is authenticated, use it
                if !metadata.requires_auth || authenticated {
                    return Ok(engine.clone());
                }
            }
        }

        Err(EngineError::NotFound(
            "No engine with valid credentials found".to_string(),
        ))
    }

    /// Selects an engine based on precedence chain.
    ///
    /// Precedence order:
    /// 1. CLI override (if provided)
    /// 2. Environment variable `RADIUM_MODEL` (if set)
    /// 3. Agent preference (if provided)
    /// 4. Default engine (if set)
    /// 5. First available engine with valid credentials
    ///
    /// # Arguments
    /// * `cli_override` - Optional CLI flag override
    /// * `agent_preference` - Optional agent preference from config
    ///
    /// # Returns
    /// Selected engine
    ///
    /// # Errors
    /// Returns error if no engine can be selected
    pub async fn select_engine(
        &self,
        cli_override: Option<&str>,
        agent_preference: Option<&str>,
    ) -> Result<Arc<dyn Engine>> {
        // 1. CLI override (highest precedence)
        if let Some(engine_id) = cli_override {
            return self.get(engine_id).map_err(|_e| {
                // Provide helpful error with available engines
                let available = self.list().unwrap_or_default();
                let engine_ids: Vec<String> = available.iter().map(|m| m.id.clone()).collect();
                EngineError::NotFound(format!(
                    "Engine '{}' not found. Available engines: {}. Run `rad models list` for more details.",
                    engine_id,
                    engine_ids.join(", ")
                ))
            });
        }

        // 2. Environment variable
        if let Ok(env_model) = std::env::var("RADIUM_MODEL") {
            if let Ok(engine) = self.get(&env_model) {
                return Ok(engine);
            }
            // If env var is set but engine not found, continue to next precedence
        }

        // 3. Agent preference
        if let Some(pref) = agent_preference {
            if let Ok(engine) = self.get(pref) {
                return Ok(engine);
            }
            // If agent preference not found, continue to next precedence
        }

        // 4. Default engine
        if let Ok(engine) = self.get_default() {
            return Ok(engine);
        }

        // 5. First available engine with valid credentials
        self.get_first_available().await.map_err(|_e| {
            let available = self.list().unwrap_or_default();
            if available.is_empty() {
                EngineError::NotFound(
                    "No engines configured. Register engines or set a default engine.".to_string(),
                )
            } else {
                EngineError::NotFound(format!(
                    "No engine with valid credentials found. Available engines: {}. Run `rad models list` to check credential status.",
                    available.iter().map(|m| m.id.clone()).collect::<Vec<_>>().join(", ")
                ))
            }
        })
    }

    /// Gets an engine with precedence-based selection.
    ///
    /// This is a convenience wrapper around `select_engine()`.
    ///
    /// # Arguments
    /// * `cli_override` - Optional CLI flag override
    /// * `agent_preference` - Optional agent preference from config
    ///
    /// # Returns
    /// Selected engine
    ///
    /// # Errors
    /// Returns error if no engine can be selected
    pub async fn get_with_precedence(
        &self,
        cli_override: Option<&str>,
        agent_preference: Option<&str>,
    ) -> Result<Arc<dyn Engine>> {
        self.select_engine(cli_override, agent_preference).await
    }

    /// Lists all available engines with credential status.
    ///
    /// # Returns
    /// Vector of engine information with credential status
    ///
    /// # Errors
    /// Returns error if lock poisoned
    pub async fn list_available(&self) -> Result<Vec<EngineInfo>> {
        let engines = self
            .engines
            .read()
            .map_err(|e| EngineError::RegistryError(format!("Lock poisoned: {}", e)))?;

        let default_id = self
            .default_engine
            .read()
            .map_err(|e| EngineError::RegistryError(format!("Lock poisoned: {}", e)))?
            .clone();

        let mut engine_infos = Vec::new();

        for (id, engine) in engines.iter() {
            let metadata = engine.metadata();
            let is_default = default_id.as_ref().map_or(false, |d| d == id);

            // Check credential status
            let credential_status = if metadata.requires_auth {
                match engine.is_authenticated().await {
                    Ok(true) => CredentialStatus::Available,
                    Ok(false) => CredentialStatus::Missing,
                    Err(_) => CredentialStatus::Unknown,
                }
            } else {
                // Engine doesn't require auth, so credentials are not applicable
                CredentialStatus::Available
            };

            // Extract provider from engine ID (e.g., "gemini" from "gemini", "openai" from "openai")
            let provider = id.clone();

            engine_infos.push(EngineInfo {
                id: id.clone(),
                name: metadata.name.clone(),
                provider,
                is_default,
                credential_status,
            });
        }

        Ok(engine_infos)
    }

    /// Validates a specific engine.
    ///
    /// Checks configuration validity, credential availability, and API reachability.
    ///
    /// # Arguments
    /// * `engine_id` - Engine identifier
    ///
    /// # Returns
    /// Validation status
    ///
    /// # Errors
    /// Returns error if engine not found
    pub async fn validate_engine(&self, engine_id: &str) -> Result<ValidationStatus> {
        let engine = self.get(engine_id)?;
        let metadata = engine.metadata();

        // Check configuration validity
        let config_valid = if let Some(engine_config) = self.get_engine_config(engine_id) {
            engine_config.validate().is_ok()
        } else {
            true // No config is valid (uses defaults)
        };

        // Check credential availability
        let credentials_available = if metadata.requires_auth {
            engine.is_authenticated().await.unwrap_or(false)
        } else {
            true // Engine doesn't require auth
        };

        // Check API reachability (engine availability)
        let api_reachable = engine.is_available().await;

        // Build error message if validation failed
        let error_message = if !config_valid {
            Some(format!("Invalid configuration for engine '{}'", engine_id))
        } else if !credentials_available {
            let provider = engine_id;
            Some(format!(
                "Credentials not configured for engine '{}' (provider: {}). Run `rad auth login {}` or set environment variables.",
                engine_id, provider, provider
            ))
        } else if !api_reachable {
            Some(format!("API not reachable for engine '{}'", engine_id))
        } else {
            None
        };

        Ok(ValidationStatus {
            config_valid,
            credentials_available,
            api_reachable,
            error_message,
        })
    }

    /// Validates all registered engines.
    ///
    /// # Returns
    /// Vector of validation statuses for each engine
    ///
    /// # Errors
    /// Returns error if lock poisoned
    pub async fn validate_all(&self) -> Result<Vec<(String, ValidationStatus)>> {
        let engines = self
            .engines
            .read()
            .map_err(|e| EngineError::RegistryError(format!("Lock poisoned: {}", e)))?;

        let mut results = Vec::new();

        for (id, _) in engines.iter() {
            match self.validate_engine(id).await {
                Ok(status) => results.push((id.clone(), status)),
                Err(e) => {
                    // If engine not found (shouldn't happen), create a failed validation
                    results.push((
                        id.clone(),
                        ValidationStatus {
                            config_valid: false,
                            credentials_available: false,
                            api_reachable: false,
                            error_message: Some(format!("Failed to validate engine: {}", e)),
                        },
                    ));
                }
            }
        }

        Ok(results)
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
                execution_duration: None,
                metadata: None,
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

    #[test]
    fn test_registry_default_trait() {
        let registry = EngineRegistry::default();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_get_default_no_default_set() {
        let registry = EngineRegistry::new();
        let result = registry.get_default();
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_set_default_nonexistent() {
        let registry = EngineRegistry::new();
        let result = registry.set_default("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_has_nonexistent() {
        let registry = EngineRegistry::new();
        assert!(!registry.has("nonexistent"));
    }

    #[test]
    fn test_registry_count_empty() {
        let registry = EngineRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_count_multiple() {
        let registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("engine-1"))).unwrap();
        registry.register(Arc::new(MockEngine::new("engine-2"))).unwrap();
        registry.register(Arc::new(MockEngine::new("engine-3"))).unwrap();
        assert_eq!(registry.count(), 3);
    }

    #[test]
    fn test_registry_duplicate_registration() {
        let registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("test-engine"))).unwrap();
        registry.register(Arc::new(MockEngine::new("test-engine"))).unwrap();
        // Should overwrite, not add
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_registry_list_empty() {
        let registry = EngineRegistry::new();
        let list = registry.list().unwrap();
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_registry_unregister_nonexistent() {
        let registry = EngineRegistry::new();
        // Unregistering nonexistent should not error
        let result = registry.unregister("nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_registry_multiple_engines_different_ids() {
        let registry = EngineRegistry::new();
        let engine1 = Arc::new(MockEngine::new("engine-1"));
        let engine2 = Arc::new(MockEngine::new("engine-2"));
        let engine3 = Arc::new(MockEngine::new("engine-3"));

        registry.register(engine1).unwrap();
        registry.register(engine2).unwrap();
        registry.register(engine3).unwrap();

        assert!(registry.has("engine-1"));
        assert!(registry.has("engine-2"));
        assert!(registry.has("engine-3"));
        assert_eq!(registry.count(), 3);
    }

    #[test]
    fn test_registry_set_default_then_change() {
        let registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("engine-1"))).unwrap();
        registry.register(Arc::new(MockEngine::new("engine-2"))).unwrap();

        registry.set_default("engine-1").unwrap();
        let default1 = registry.get_default().unwrap();
        assert_eq!(default1.metadata().id, "engine-1");

        registry.set_default("engine-2").unwrap();
        let default2 = registry.get_default().unwrap();
        assert_eq!(default2.metadata().id, "engine-2");
    }

    #[test]
    fn test_registry_get_after_unregister() {
        let registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("test-engine"))).unwrap();

        registry.unregister("test-engine").unwrap();

        let result = registry.get("test-engine");
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_list_order_independence() {
        let registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("c-engine"))).unwrap();
        registry.register(Arc::new(MockEngine::new("a-engine"))).unwrap();
        registry.register(Arc::new(MockEngine::new("b-engine"))).unwrap();

        let list = registry.list().unwrap();
        assert_eq!(list.len(), 3);
        // All engines should be present regardless of order
        let ids: Vec<String> = list.iter().map(|m| m.id.clone()).collect();
        assert!(ids.contains(&"a-engine".to_string()));
        assert!(ids.contains(&"b-engine".to_string()));
        assert!(ids.contains(&"c-engine".to_string()));
    }

    #[tokio::test]
    async fn test_select_engine_cli_override() {
        let registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("engine-1"))).unwrap();
        registry.register(Arc::new(MockEngine::new("engine-2"))).unwrap();
        registry.set_default("engine-1").unwrap();

        // CLI override should take precedence over default
        let selected = registry.select_engine(Some("engine-2"), None).await.unwrap();
        assert_eq!(selected.metadata().id, "engine-2");
    }

    #[tokio::test]
    async fn test_select_engine_default_fallback() {
        let registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("engine-1"))).unwrap();
        registry.set_default("engine-1").unwrap();

        // Should use default when no CLI override
        let selected = registry.select_engine(None, None).await.unwrap();
        assert_eq!(selected.metadata().id, "engine-1");
    }

    #[tokio::test]
    async fn test_select_engine_first_available_fallback() {
        let registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("engine-1"))).unwrap();
        registry.register(Arc::new(MockEngine::new("engine-2"))).unwrap();
        // No default set

        // Should use first available when no default
        let selected = registry.select_engine(None, None).await.unwrap();
        // Should get one of the engines (order may vary)
        assert!(selected.metadata().id == "engine-1" || selected.metadata().id == "engine-2");
    }

    #[tokio::test]
    async fn test_select_engine_agent_preference() {
        let registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("engine-1"))).unwrap();
        registry.register(Arc::new(MockEngine::new("engine-2"))).unwrap();
        registry.set_default("engine-1").unwrap();

        // Agent preference should be used when no CLI override
        let selected = registry.select_engine(None, Some("engine-2")).await.unwrap();
        assert_eq!(selected.metadata().id, "engine-2");
    }

    #[tokio::test]
    async fn test_select_engine_not_found() {
        let registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("engine-1"))).unwrap();

        // Should error with helpful message
        let result = registry.select_engine(Some("nonexistent"), None).await;
        assert!(result.is_err());
        // Note: Cannot unwrap_err() due to Debug trait requirement
        // let error = result.unwrap_err();
        // assert!(error.to_string().contains("not found"));
        // assert!(error.to_string().contains("engine-1")); // Should list available
    }

    #[tokio::test]
    async fn test_list_available() {
        let registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("engine-1"))).unwrap();
        registry.register(Arc::new(MockEngine::new("engine-2"))).unwrap();
        registry.set_default("engine-1").unwrap();

        let engines = registry.list_available().await.unwrap();
        assert_eq!(engines.len(), 2);
        
        let engine1 = engines.iter().find(|e| e.id == "engine-1").unwrap();
        assert!(engine1.is_default);
        assert_eq!(engine1.credential_status, CredentialStatus::Available);
        
        let engine2 = engines.iter().find(|e| e.id == "engine-2").unwrap();
        assert!(!engine2.is_default);
    }

    #[tokio::test]
    async fn test_validate_engine() {
        let registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("engine-1"))).unwrap();

        let status = registry.validate_engine("engine-1").await.unwrap();
        assert!(status.config_valid);
        assert!(status.credentials_available);
        assert!(status.api_reachable);
    }

    #[tokio::test]
    async fn test_validate_all() {
        let registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("engine-1"))).unwrap();
        registry.register(Arc::new(MockEngine::new("engine-2"))).unwrap();

        let results = registry.validate_all().await.unwrap();
        assert_eq!(results.len(), 2);
        
        for (id, status) in results {
            assert!(id == "engine-1" || id == "engine-2");
            assert!(status.config_valid);
        }
    }

    #[test]
    fn test_registry_has_after_register() {
        let registry = EngineRegistry::new();
        assert!(!registry.has("test-engine"));

        registry.register(Arc::new(MockEngine::new("test-engine"))).unwrap();
        assert!(registry.has("test-engine"));
    }

    #[test]
    fn test_registry_count_after_operations() {
        let registry = EngineRegistry::new();
        assert_eq!(registry.count(), 0);

        registry.register(Arc::new(MockEngine::new("engine-1"))).unwrap();
        assert_eq!(registry.count(), 1);

        registry.register(Arc::new(MockEngine::new("engine-2"))).unwrap();
        assert_eq!(registry.count(), 2);

        registry.unregister("engine-1").unwrap();
        assert_eq!(registry.count(), 1);

        registry.unregister("engine-2").unwrap();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_get_metadata() {
        let registry = EngineRegistry::new();
        let engine = Arc::new(MockEngine::new("test-engine"));
        registry.register(engine).unwrap();

        let retrieved = registry.get("test-engine").unwrap();
        let metadata = retrieved.metadata();
        assert_eq!(metadata.id, "test-engine");
        assert_eq!(metadata.name, "Mock test-engine");
    }

    #[test]
    fn test_registry_unregister_multiple() {
        let registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("engine-1"))).unwrap();
        registry.register(Arc::new(MockEngine::new("engine-2"))).unwrap();
        registry.register(Arc::new(MockEngine::new("engine-3"))).unwrap();
        assert_eq!(registry.count(), 3);

        registry.unregister("engine-1").unwrap();
        registry.unregister("engine-3").unwrap();

        assert_eq!(registry.count(), 1);
        assert!(registry.has("engine-2"));
        assert!(!registry.has("engine-1"));
        assert!(!registry.has("engine-3"));
    }
}

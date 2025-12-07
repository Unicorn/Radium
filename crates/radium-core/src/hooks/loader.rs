//! Hook loader for discovering and loading hooks from extensions.

use crate::extensions::integration::get_extension_hook_paths;
use crate::hooks::config::{HookConfig, HookDefinition};
use crate::hooks::error::{HookError, Result};
use crate::hooks::registry::{Hook, HookRegistry, HookType};
use crate::hooks::types::HookPriority;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Hook factory function type.
/// 
/// Factory functions create hook instances from configurations.
pub type HookFactory = fn(&HookDefinition) -> Result<Option<Arc<dyn Hook>>>;

/// Hook loader for discovering and loading hooks from extensions.
pub struct HookLoader {
    /// Registry of hook factories by hook name pattern or type.
    factories: HashMap<String, HookFactory>,
}

impl HookLoader {
    /// Create a new hook loader with default factories.
    pub fn new() -> Self {
        let mut factories = HashMap::new();
        
        // Register built-in hook factories
        // These can be extended by users to support custom hooks
        // For v1.0, we support configuration-based hook registration
        // Custom hooks need to be registered programmatically
        
        Self { factories }
    }

    /// Register a hook factory for a specific hook name or pattern.
    /// 
    /// # Arguments
    /// * `pattern` - Hook name pattern (exact match) or hook type
    /// * `factory` - Factory function that creates the hook
    pub fn register_factory(&mut self, pattern: impl Into<String>, factory: HookFactory) {
        self.factories.insert(pattern.into(), factory);
    }

    /// Create a hook instance from a hook definition.
    /// 
    /// This attempts to create a hook using registered factories.
    /// If no factory is found, returns None (hook must be registered programmatically).
    /// 
    /// # Arguments
    /// * `def` - Hook definition from configuration
    /// 
    /// # Returns
    /// Hook instance if factory exists, None otherwise
    fn create_hook_from_definition(&self, def: &HookDefinition) -> Result<Option<Arc<dyn Hook>>> {
        // Try to find a factory for this hook name
        if let Some(factory) = self.factories.get(&def.name) {
            return factory(def);
        }

        // Try to find a factory for this hook type
        if let Some(factory) = self.factories.get(&def.hook_type) {
            return factory(def);
        }

        // No factory found - hook must be registered programmatically
        // This is expected for custom hooks in v1.0
        Ok(None)
    }

    /// Load and register hooks from a configuration.
    /// 
    /// # Arguments
    /// * `config` - Hook configuration
    /// * `registry` - Hook registry to register hooks
    /// * `workspace_root` - Workspace root for resolving script paths
    /// 
    /// # Returns
    /// Number of hooks successfully loaded
    /// 
    /// # Errors
    /// Returns error if loading fails
    pub async fn load_hooks_from_config<P: AsRef<Path>>(
        &self,
        config: &HookConfig,
        registry: &Arc<HookRegistry>,
        workspace_root: Option<P>,
    ) -> Result<usize> {
        let mut loaded_count = 0;

        for hook_def in &config.hooks {
            // Skip disabled hooks
            if !hook_def.enabled {
                tracing::debug!(
                    hook_name = %hook_def.name,
                    "Skipping disabled hook"
                );
                continue;
            }

            // Try to create hook from definition
            match self.create_hook_from_definition(hook_def) {
                Ok(Some(hook)) => {
                    // Register the hook
                    if let Err(e) = registry.register(hook).await {
                        tracing::warn!(
                            hook_name = %hook_def.name,
                            error = %e,
                            "Failed to register hook"
                        );
                        continue;
                    }

                    // Set enabled state from config
                    if let Err(e) = registry.set_enabled(&hook_def.name, hook_def.enabled).await {
                        tracing::warn!(
                            hook_name = %hook_def.name,
                            error = %e,
                            "Failed to set enabled state"
                        );
                    }

                    loaded_count += 1;
                    tracing::debug!(
                        hook_name = %hook_def.name,
                        hook_type = %hook_def.hook_type,
                        "Loaded and registered hook"
                    );
                }
                Ok(None) => {
                    // No factory found - hook must be registered programmatically
                    // This is expected for custom hooks
                    tracing::debug!(
                        hook_name = %hook_def.name,
                        hook_type = %hook_def.hook_type,
                        "Hook configuration found but no factory available - must be registered programmatically"
                    );
                    
                    // Still set enabled state in case hook is registered later
                    // Note: This will fail if hook doesn't exist, which is expected
                    let _ = registry.set_enabled(&hook_def.name, hook_def.enabled).await;
                }
                Err(e) => {
                    tracing::warn!(
                        hook_name = %hook_def.name,
                        error = %e,
                        "Failed to create hook from definition"
                    );
                }
            }
        }

        Ok(loaded_count)
    }

    /// Discover and load all hooks from extensions.
    ///
    /// # Arguments
    /// * `registry` - Hook registry to register discovered hooks
    ///
    /// # Returns
    /// Number of hooks successfully loaded and registered
    ///
    /// # Errors
    /// Returns error if discovery or loading fails
    pub async fn load_from_extensions(registry: &Arc<HookRegistry>) -> Result<usize> {
        let loader = Self::new();
        loader.load_from_extensions_with_loader(registry).await
    }

    /// Internal method that uses this loader instance.
    async fn load_from_extensions_with_loader(&self, registry: &Arc<HookRegistry>) -> Result<usize> {
        let hook_paths = get_extension_hook_paths()
            .map_err(|e| HookError::Discovery(format!("Failed to discover extension hooks: {}", e)))?;

        let mut loaded_count = 0;

        for path in hook_paths {
            // Try to load as TOML configuration file
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Ok(config) = HookConfig::from_file(&path) {
                    // Validate configuration
                    if let Err(e) = config.validate() {
                        tracing::warn!(
                            path = %path.display(),
                            error = %e,
                            "Failed to validate hook configuration, skipping"
                        );
                        continue;
                    }

                    // Try to load hooks from this configuration
                    // Get workspace root from extension path (go up to find .radium)
                    let workspace_root = path.parent()
                        .and_then(|p| p.ancestors().find(|a| a.join(".radium").exists()));
                    
                    match self.load_hooks_from_config(&config, registry, workspace_root).await {
                        Ok(count) => {
                            loaded_count += count;
                            tracing::debug!(
                                path = %path.display(),
                                hooks_loaded = count,
                                total_hooks = config.hooks.len(),
                                "Loaded hooks from extension configuration"
                            );
                        }
                        Err(e) => {
                            tracing::warn!(
                                path = %path.display(),
                                error = %e,
                                "Failed to load hooks from configuration"
                            );
                        }
                    }
                }
            } else {
                // For non-TOML files, we could support dynamic library loading in the future
                tracing::debug!(
                    path = %path.display(),
                    "Skipping non-TOML hook file (dynamic library loading deferred to v2.0)"
                );
            }
        }

        Ok(loaded_count)
    }

    /// Load hooks from a specific directory.
    ///
    /// # Arguments
    /// * `dir` - Directory to search for hook files
    /// * `registry` - Hook registry to register discovered hooks
    ///
    /// # Returns
    /// Number of hooks successfully loaded and registered
    ///
    /// # Errors
    /// Returns error if loading fails
    pub async fn load_from_directory<P: AsRef<Path>>(
        dir: P,
        registry: &Arc<HookRegistry>,
    ) -> Result<usize> {
        let loader = Self::new();
        loader.load_from_directory_with_loader(dir, registry).await
    }

    /// Internal method that uses this loader instance.
    async fn load_from_directory_with_loader<P: AsRef<Path>>(
        &self,
        dir: P,
        registry: &Arc<HookRegistry>,
    ) -> Result<usize> {
        let dir = dir.as_ref();
        if !dir.exists() {
            return Ok(0);
        }

        let mut loaded_count = 0;

        // Search for TOML configuration files
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    if let Ok(config) = HookConfig::from_file(&path) {
                        if let Err(e) = config.validate() {
                            tracing::warn!(
                                path = %path.display(),
                                error = %e,
                                "Failed to validate hook configuration, skipping"
                            );
                            continue;
                        }

                        // Load hooks from this configuration
                        match self.load_hooks_from_config(&config, registry, Some(dir)).await {
                            Ok(count) => {
                                loaded_count += count;
                            }
                            Err(e) => {
                                tracing::warn!(
                                    path = %path.display(),
                                    error = %e,
                                    "Failed to load hooks from configuration"
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(loaded_count)
    }

    /// Load hooks from a workspace configuration file.
    ///
    /// # Arguments
    /// * `workspace_root` - Workspace root directory
    /// * `registry` - Hook registry to register discovered hooks
    ///
    /// # Returns
    /// Number of hooks successfully loaded and registered
    ///
    /// # Errors
    /// Returns error if loading fails
    pub async fn load_from_workspace<P: AsRef<Path>>(
        workspace_root: P,
        registry: &Arc<HookRegistry>,
    ) -> Result<usize> {
        let loader = Self::new();
        loader.load_from_workspace_with_loader(workspace_root, registry).await
    }

    /// Internal method that uses this loader instance.
    async fn load_from_workspace_with_loader<P: AsRef<Path>>(
        &self,
        workspace_root: P,
        registry: &Arc<HookRegistry>,
    ) -> Result<usize> {
        let workspace_root = workspace_root.as_ref();
        let hooks_config_path = workspace_root.join(".radium").join("hooks.toml");

        if !hooks_config_path.exists() {
            return Ok(0);
        }

        match HookConfig::from_file(&hooks_config_path) {
            Ok(config) => {
                if let Err(e) = config.validate() {
                    return Err(HookError::InvalidConfig(format!(
                        "Failed to validate workspace hook configuration: {}",
                        e
                    )));
                }

                // Load hooks from configuration
                self.load_hooks_from_config(&config, registry, Some(workspace_root)).await
            }
            Err(e) => {
                tracing::warn!(
                    path = %hooks_config_path.display(),
                    error = %e,
                    "Failed to load workspace hook configuration"
                );
                Err(e)
            }
        }
    }

    /// Discover all hook configuration files.
    ///
    /// # Returns
    /// Vector of paths to hook configuration files
    ///
    /// # Errors
    /// Returns error if discovery fails
    pub fn discover_config_files() -> Result<Vec<std::path::PathBuf>> {
        let hook_paths = get_extension_hook_paths()
            .map_err(|e| HookError::Discovery(format!("Failed to discover extension hooks: {}", e)))?;

        let mut config_files = Vec::new();

        for path in hook_paths {
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                config_files.push(path);
            }
        }

        Ok(config_files)
    }
}

impl Default for HookLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_load_from_directory_empty() {
        let temp_dir = TempDir::new().unwrap();
        let registry = Arc::new(HookRegistry::new());

        let count = HookLoader::load_from_directory(temp_dir.path(), &registry).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_load_from_directory_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let registry = Arc::new(HookRegistry::new());

        // Create an invalid TOML file
        let hooks_file = temp_dir.path().join("hooks.toml");
        std::fs::write(&hooks_file, "invalid toml content {").unwrap();

        // Should not error, just skip invalid files
        let count = HookLoader::load_from_directory(temp_dir.path(), &registry).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_load_from_workspace_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let registry = Arc::new(HookRegistry::new());

        let count = HookLoader::load_from_workspace(temp_dir.path(), &registry).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_discover_config_files() {
        // This will return empty if no extensions are installed
        // which is expected in test environment
        let config_files = HookLoader::discover_config_files();
        assert!(config_files.is_ok());
    }

    #[tokio::test]
    async fn test_load_hooks_from_config_no_factory() {
        let loader = HookLoader::new();
        let registry = Arc::new(HookRegistry::new());
        
        let config = HookConfig {
            hooks: vec![HookDefinition {
                name: "test-hook".to_string(),
                hook_type: "before_model".to_string(),
                priority: Some(100),
                enabled: true,
                script: Some("hooks/test.rs".to_string()),
                config: None,
            }],
        };

        // Should succeed but load 0 hooks (no factory available)
        let count = loader.load_hooks_from_config(&config, &registry, None::<&Path>).await.unwrap();
        assert_eq!(count, 0);
    }
}

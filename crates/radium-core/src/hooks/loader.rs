//! Hook loader for discovering and loading hooks from extensions.

use crate::extensions::integration::get_extension_hook_paths;
use crate::hooks::config::HookConfig;
use crate::hooks::error::{HookError, Result};
use crate::hooks::registry::HookRegistry;
use std::path::Path;
use std::sync::Arc;

/// Hook loader for discovering and loading hooks from extensions.
pub struct HookLoader;

impl HookLoader {
    /// Discover and load all hooks from extensions.
    ///
    /// # Arguments
    /// * `registry` - Hook registry to register discovered hooks
    ///
    /// # Returns
    /// Number of hooks discovered
    ///
    /// # Errors
    /// Returns error if discovery or loading fails
    pub async fn load_from_extensions(registry: &Arc<HookRegistry>) -> Result<usize> {
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

                    // For now, we just count the hooks
                    // Actual hook registration would require hook implementations
                    // which need to be compiled into the binary
                    loaded_count += config.hooks.len();
                    tracing::debug!(
                        path = %path.display(),
                        hooks = config.hooks.len(),
                        "Discovered hook configuration"
                    );
                }
            } else {
                // For non-TOML files, we could support other formats in the future
                tracing::debug!(
                    path = %path.display(),
                    "Skipping non-TOML hook file"
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
    /// Number of hooks discovered
    ///
    /// # Errors
    /// Returns error if loading fails
    pub async fn load_from_directory<P: AsRef<Path>>(
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

                        loaded_count += config.hooks.len();
                        tracing::debug!(
                            path = %path.display(),
                            hooks = config.hooks.len(),
                            "Discovered hook configuration"
                        );
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
    /// Number of hooks discovered
    ///
    /// # Errors
    /// Returns error if loading fails
    pub async fn load_from_workspace<P: AsRef<Path>>(
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

                // Set enabled state for hooks in config
                for hook_def in &config.hooks {
                    registry.set_enabled(&hook_def.name, hook_def.enabled).await
                        .unwrap_or_else(|e| {
                            tracing::warn!(
                                hook_name = %hook_def.name,
                                error = %e,
                                "Failed to set enabled state for hook"
                            );
                        });
                }

                let count = config.hooks.len();
                tracing::debug!(
                    path = %hooks_config_path.display(),
                    hooks = count,
                    "Loaded workspace hook configuration"
                );
                Ok(count)
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
}


//! Extension integration helpers.
//!
//! Provides utilities for integrating extension components into existing systems.
//! This module contains helper functions that can be called from agent discovery,
//! command registry, and MCP systems to load extension components.

use crate::extensions::{Extension, ExtensionDiscovery, ExtensionManager};
use std::path::PathBuf;

/// Gets all extension prompt directories.
///
/// # Returns
/// Vector of paths to extension prompts directories
pub fn get_extension_prompt_dirs() -> crate::extensions::Result<Vec<PathBuf>> {
    let discovery = ExtensionDiscovery::new();
    let extensions = discovery.discover_all()?;

    let mut dirs = Vec::new();
    for ext in extensions {
        let prompts_dir = ext.prompts_dir();
        if prompts_dir.exists() {
            dirs.push(prompts_dir);
        }
    }

    Ok(dirs)
}

/// Gets all extension command directories.
///
/// # Returns
/// Vector of paths to extension commands directories
pub fn get_extension_command_dirs() -> crate::extensions::Result<Vec<PathBuf>> {
    let discovery = ExtensionDiscovery::new();
    let extensions = discovery.discover_all()?;

    let mut dirs = Vec::new();
    for ext in extensions {
        let commands_dir = ext.commands_dir();
        if commands_dir.exists() {
            dirs.push(commands_dir);
        }
    }

    Ok(dirs)
}

/// Gets all extension MCP server configuration paths.
///
/// # Returns
/// Vector of paths to extension MCP server configuration files
pub fn get_extension_mcp_configs() -> crate::extensions::Result<Vec<PathBuf>> {
    let discovery = ExtensionDiscovery::new();
    let extensions = discovery.discover_all()?;

    let mut configs = Vec::new();
    for ext in extensions {
        let mcp_paths = ext.get_mcp_paths()?;
        configs.extend(mcp_paths);
    }

    Ok(configs)
}

/// Gets all installed extensions.
///
/// # Returns
/// Vector of all installed extensions
pub fn get_all_extensions() -> crate::extensions::Result<Vec<Extension>> {
    let manager = ExtensionManager::new().map_err(|e| crate::extensions::ExtensionError::Installer(e))?;
    manager.list().map_err(|e| crate::extensions::ExtensionError::Installer(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_extension_prompt_dirs_empty() {
        // This will return empty if no extensions are installed
        // which is expected in test environment
        let dirs = get_extension_prompt_dirs();
        // Should not error even if no extensions exist
        assert!(dirs.is_ok());
    }

    #[test]
    fn test_get_extension_command_dirs_empty() {
        let dirs = get_extension_command_dirs();
        assert!(dirs.is_ok());
    }

    #[test]
    fn test_get_extension_mcp_configs_empty() {
        let configs = get_extension_mcp_configs();
        assert!(configs.is_ok());
    }
}


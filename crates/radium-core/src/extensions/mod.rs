//! Extension system for Radium.
//!
//! Provides functionality for installing and managing extension packages
//! that bundle prompts, MCP servers, and custom commands.
//!
//! # Example
//!
//! ```rust,no_run
//! use radium_core::extensions::{ExtensionManager, InstallOptions};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let manager = ExtensionManager::new()?;
//! let options = InstallOptions::default();
//! let extension = manager.install(&Path::new("./my-extension"), options)?;
//! println!("Installed extension: {}", extension.name);
//! # Ok(())
//! # }
//! ```

pub mod manifest;
pub mod structure;
pub mod discovery;
pub mod installer;
pub mod integration;

pub use manifest::{ExtensionManifest, ExtensionManifestError};
pub use structure::{
    Extension, ExtensionStructureError,
    default_extensions_dir, workspace_extensions_dir,
};
pub use discovery::{
    ExtensionDiscovery, ExtensionDiscoveryError, DiscoveryOptions,
};
pub use installer::{
    ExtensionManager, ExtensionInstallerError, InstallOptions,
};
pub use integration::{
    get_all_extensions, get_extension_command_dirs, get_extension_mcp_configs,
    get_extension_prompt_dirs,
};

/// Unified error type for extension operations.
#[derive(Debug, thiserror::Error)]
pub enum ExtensionError {
    /// Manifest error.
    #[error("manifest error: {0}")]
    Manifest(#[from] ExtensionManifestError),

    /// Structure error.
    #[error("structure error: {0}")]
    Structure(#[from] ExtensionStructureError),

    /// Discovery error.
    #[error("discovery error: {0}")]
    Discovery(#[from] ExtensionDiscoveryError),

    /// Installer error.
    #[error("installer error: {0}")]
    Installer(#[from] ExtensionInstallerError),
}

/// Result type for extension operations.
pub type Result<T> = std::result::Result<T, ExtensionError>;


//! Podman container-based sandboxing.
//!
//! This is a stub implementation. Podman sandboxing will be implemented in the future.

use super::config::{SandboxConfig, SandboxType};
use super::error::{Result, SandboxError};
use super::sandbox::Sandbox;
use async_trait::async_trait;
use std::path::Path;
use std::process::Output;

/// Podman-based sandbox.
///
/// This is a stub implementation. Podman sandboxing functionality will be added in the future.
pub struct PodmanSandbox {
    /// Sandbox configuration.
    _config: SandboxConfig,
}

impl PodmanSandbox {
    /// Creates a new Podman sandbox.
    ///
    /// # Arguments
    /// * `config` - Sandbox configuration
    ///
    /// # Errors
    /// Returns error if Podman is not available
    pub fn new(config: SandboxConfig) -> Result<Self> {
        // Verify Podman is available
        std::process::Command::new("podman").arg("--version").output().map_err(|e| {
            SandboxError::ContainerRuntimeNotFound(format!("Podman not found: {}", e))
        })?;

        Ok(Self { _config: config })
    }
}

#[async_trait]
impl Sandbox for PodmanSandbox {
    async fn initialize(&mut self) -> Result<()> {
        // Stub implementation
        Err(SandboxError::NotAvailable("Podman sandboxing not yet implemented".to_string()))
    }

    async fn execute(
        &self,
        _command: &str,
        _args: &[String],
        _cwd: Option<&Path>,
    ) -> Result<Output> {
        Err(SandboxError::NotAvailable("Podman sandboxing not yet implemented".to_string()))
    }

    async fn cleanup(&mut self) -> Result<()> {
        Err(SandboxError::NotAvailable("Podman sandboxing not yet implemented".to_string()))
    }

    fn sandbox_type(&self) -> SandboxType {
        SandboxType::Podman
    }
}

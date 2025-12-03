//! Sandbox abstraction trait and factory.

use super::config::{SandboxConfig, SandboxType};
use super::error::{Result, SandboxError};
use async_trait::async_trait;
use std::path::Path;
use std::process::Output;

/// Sandbox trait for executing commands in isolated environments.
#[async_trait]
pub trait Sandbox: Send + Sync {
    /// Initializes the sandbox environment.
    ///
    /// # Errors
    /// Returns error if initialization fails
    async fn initialize(&mut self) -> Result<()>;

    /// Executes a command in the sandbox.
    ///
    /// # Arguments
    /// * `command` - Command to execute
    /// * `args` - Command arguments
    /// * `cwd` - Current working directory
    ///
    /// # Returns
    /// Command output
    ///
    /// # Errors
    /// Returns error if execution fails
    async fn execute(
        &self,
        command: &str,
        args: &[String],
        cwd: Option<&Path>,
    ) -> Result<Output>;

    /// Cleans up the sandbox environment.
    ///
    /// # Errors
    /// Returns error if cleanup fails
    async fn cleanup(&mut self) -> Result<()>;

    /// Gets the sandbox type.
    fn sandbox_type(&self) -> SandboxType;
}

/// Sandbox factory for creating sandbox instances.
pub struct SandboxFactory;

impl SandboxFactory {
    /// Creates a sandbox instance based on configuration.
    ///
    /// # Arguments
    /// * `config` - Sandbox configuration
    ///
    /// # Returns
    /// Boxed sandbox instance
    ///
    /// # Errors
    /// Returns error if sandbox type is not available
    pub fn create(config: SandboxConfig) -> Result<Box<dyn Sandbox>> {
        match config.sandbox_type {
            SandboxType::None => Ok(Box::new(NoSandbox::new())),
            SandboxType::Docker => {
                #[cfg(feature = "docker-sandbox")]
                {
                    use super::docker::DockerSandbox;
                    Ok(Box::new(DockerSandbox::new(config)?))
                }
                #[cfg(not(feature = "docker-sandbox"))]
                {
                    Err(SandboxError::NotAvailable("docker".to_string()))
                }
            }
            SandboxType::Podman => {
                #[cfg(feature = "podman-sandbox")]
                {
                    use super::podman::PodmanSandbox;
                    Ok(Box::new(PodmanSandbox::new(config)?))
                }
                #[cfg(not(feature = "podman-sandbox"))]
                {
                    Err(SandboxError::NotAvailable("podman".to_string()))
                }
            }
            SandboxType::Seatbelt => {
                #[cfg(all(target_os = "macos", feature = "seatbelt-sandbox"))]
                {
                    use super::seatbelt::SeatbeltSandbox;
                    Ok(Box::new(SeatbeltSandbox::new(config)?))
                }
                #[cfg(not(all(target_os = "macos", feature = "seatbelt-sandbox")))]
                {
                    Err(SandboxError::NotAvailable("seatbelt".to_string()))
                }
            }
        }
    }
}

/// No-op sandbox (executes commands directly without sandboxing).
pub struct NoSandbox;

impl NoSandbox {
    /// Creates a new no-op sandbox.
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoSandbox {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Sandbox for NoSandbox {
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    async fn execute(
        &self,
        command: &str,
        args: &[String],
        cwd: Option<&Path>,
    ) -> Result<Output> {
        use tokio::process::Command;

        let mut cmd = Command::new(command);
        cmd.args(args);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let output = cmd.output().await?;
        Ok(output)
    }

    async fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }

    fn sandbox_type(&self) -> SandboxType {
        SandboxType::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_sandbox_initialize() {
        let mut sandbox = NoSandbox::new();
        assert!(sandbox.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_no_sandbox_execute() {
        let sandbox = NoSandbox::new();
        let output = sandbox
            .execute("echo", &["hello".to_string()], None)
            .await
            .unwrap();

        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "hello"
        );
    }

    #[tokio::test]
    async fn test_no_sandbox_cleanup() {
        let mut sandbox = NoSandbox::new();
        assert!(sandbox.cleanup().await.is_ok());
    }

    #[tokio::test]
    async fn test_sandbox_factory_no_sandbox() {
        let config = SandboxConfig::default();
        let sandbox = SandboxFactory::create(config).unwrap();
        assert_eq!(sandbox.sandbox_type(), SandboxType::None);
    }
}

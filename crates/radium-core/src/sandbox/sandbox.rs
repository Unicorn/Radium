//! Sandbox abstraction trait and factory.

use super::config::{SandboxConfig, SandboxType};
use super::error::{Result, SandboxError};
use crate::security::SecretInjector;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::process::Output;
use std::sync::Arc;

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
    async fn execute(&self, command: &str, args: &[String], cwd: Option<&Path>) -> Result<Output>;

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
    pub fn create(config: &SandboxConfig) -> Result<Box<dyn Sandbox>> {
        match config.sandbox_type {
            SandboxType::None => Ok(Box::new(NoSandbox::new())),
            SandboxType::Docker => {
                #[cfg(feature = "docker-sandbox")]
                {
                    use super::docker::DockerSandbox;
                    Ok(Box::new(DockerSandbox::new(config.clone())?))
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
                    Ok(Box::new(PodmanSandbox::new(config.clone())?))
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
                    Ok(Box::new(SeatbeltSandbox::new(config.clone())?))
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
pub struct NoSandbox {
    /// Optional secret injector for credential injection.
    secret_injector: Option<Arc<SecretInjector>>,
}

impl NoSandbox {
    /// Creates a new no-op sandbox.
    pub fn new() -> Self {
        Self {
            secret_injector: None,
        }
    }

    /// Creates a new no-op sandbox with secret injector.
    pub fn with_secret_injector(injector: Arc<SecretInjector>) -> Self {
        Self {
            secret_injector: Some(injector),
        }
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

    async fn execute(&self, command: &str, args: &[String], cwd: Option<&Path>) -> Result<Output> {
        use tokio::process::Command;

        // Inject secrets if injector is available
        let (injected_command, injected_args) = if let Some(ref injector) = self.secret_injector {
            let injected_cmd = injector.inject_secrets(command)
                .map_err(|e| SandboxError::ExecutionFailed(format!("Secret injection failed: {}", e)))?;
            
            let injected_args: Vec<String> = args.iter()
                .map(|arg| {
                    injector.inject_secrets(arg)
                        .unwrap_or_else(|_| arg.clone())
                })
                .collect();
            
            (injected_cmd, injected_args)
        } else {
            (command.to_string(), args.to_vec())
        };

        let mut cmd = Command::new(&injected_command);
        cmd.args(&injected_args);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        // Inject secrets into environment variables
        if let Some(ref injector) = self.secret_injector {
            let mut env_vars = HashMap::new();
            // Get current environment
            for (key, value) in std::env::vars() {
                env_vars.insert(key, value);
            }
            
            // Inject secrets into environment
            injector.inject_env_vars(&mut env_vars)
                .map_err(|e| SandboxError::ExecutionFailed(format!("Environment injection failed: {}", e)))?;
            
            // Set all environment variables
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
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
        let output = sandbox.execute("echo", &["hello".to_string()], None).await.unwrap();

        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
    }

    #[tokio::test]
    async fn test_no_sandbox_cleanup() {
        let mut sandbox = NoSandbox::new();
        assert!(sandbox.cleanup().await.is_ok());
    }

    #[tokio::test]
    async fn test_sandbox_factory_no_sandbox() {
        let config = SandboxConfig::default();
        let sandbox = SandboxFactory::create(&config).unwrap();
        assert_eq!(sandbox.sandbox_type(), SandboxType::None);
    }

    #[tokio::test]
    async fn test_no_sandbox_execute_with_cwd() {
        let sandbox = NoSandbox::new();
        let cwd = std::env::current_dir().unwrap();
        let output = sandbox.execute("pwd", &[], Some(&cwd)).await.unwrap();

        assert!(output.status.success());
        let output_str = String::from_utf8_lossy(&output.stdout);
        assert!(output_str.contains(cwd.to_str().unwrap()));
    }

    #[tokio::test]
    async fn test_no_sandbox_execute_multiple_args() {
        let sandbox = NoSandbox::new();
        let args = vec!["arg1".to_string(), "arg2".to_string(), "arg3".to_string()];
        let output = sandbox.execute("echo", &args, None).await.unwrap();

        assert!(output.status.success());
        let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert_eq!(output_str, "arg1 arg2 arg3");
    }

    #[tokio::test]
    async fn test_no_sandbox_execute_failing_command() {
        let sandbox = NoSandbox::new();
        let result = sandbox.execute("false", &[], None).await;

        // Command should execute but return non-zero exit code
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.status.success());
    }

    #[tokio::test]
    async fn test_no_sandbox_execute_nonexistent_command() {
        let sandbox = NoSandbox::new();
        let result = sandbox.execute("nonexistent_command_12345", &[], None).await;

        // Should return error for command not found
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_no_sandbox_type() {
        let sandbox = NoSandbox::new();
        assert_eq!(sandbox.sandbox_type(), SandboxType::None);
    }

    #[tokio::test]
    async fn test_sandbox_factory_docker_not_available() {
        #[cfg(not(feature = "docker-sandbox"))]
        {
            let config = SandboxConfig { sandbox_type: SandboxType::Docker, ..Default::default() };
            let result = SandboxFactory::create(&config);
            assert!(result.is_err());
            if let Err(SandboxError::NotAvailable(sandbox_type)) = result {
                assert_eq!(sandbox_type, "docker");
            }
        }
    }

    #[tokio::test]
    async fn test_sandbox_factory_podman_not_available() {
        #[cfg(not(feature = "podman-sandbox"))]
        {
            let config = SandboxConfig { sandbox_type: SandboxType::Podman, ..Default::default() };
            let result = SandboxFactory::create(&config);
            assert!(result.is_err());
            if let Err(SandboxError::NotAvailable(sandbox_type)) = result {
                assert_eq!(sandbox_type, "podman");
            }
        }
    }

    #[tokio::test]
    async fn test_sandbox_factory_seatbelt_not_available() {
        #[cfg(not(all(target_os = "macos", feature = "seatbelt-sandbox")))]
        {
            let config =
                SandboxConfig { sandbox_type: SandboxType::Seatbelt, ..Default::default() };
            let result = SandboxFactory::create(&config);
            assert!(result.is_err());
            if let Err(SandboxError::NotAvailable(sandbox_type)) = result {
                assert_eq!(sandbox_type, "seatbelt");
            }
        }
    }

    #[test]
    fn test_no_sandbox_default() {
        let sandbox = NoSandbox::default();
        assert_eq!(sandbox.sandbox_type(), SandboxType::None);
    }

    #[tokio::test]
    async fn test_no_sandbox_execute_with_stderr() {
        let sandbox = NoSandbox::new();
        // Command that outputs to stderr
        let output = sandbox
            .execute("sh", &["-c".to_string(), "echo error >&2".to_string()], None)
            .await
            .unwrap();

        assert!(output.status.success());
        let stderr_str = String::from_utf8_lossy(&output.stderr).trim().to_string();
        assert_eq!(stderr_str, "error");
    }

    #[tokio::test]
    async fn test_no_sandbox_execute_exit_code() {
        let sandbox = NoSandbox::new();
        let output =
            sandbox.execute("sh", &["-c".to_string(), "exit 42".to_string()], None).await.unwrap();

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(42));
    }

    #[tokio::test]
    async fn test_no_sandbox_multiple_initializations() {
        let mut sandbox = NoSandbox::new();

        // Initialize multiple times should work
        assert!(sandbox.initialize().await.is_ok());
        assert!(sandbox.initialize().await.is_ok());
        assert!(sandbox.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_no_sandbox_multiple_cleanups() {
        let mut sandbox = NoSandbox::new();

        // Cleanup multiple times should work
        assert!(sandbox.cleanup().await.is_ok());
        assert!(sandbox.cleanup().await.is_ok());
        assert!(sandbox.cleanup().await.is_ok());
    }
}

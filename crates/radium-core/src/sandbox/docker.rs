//! Docker container-based sandboxing.

use super::config::{NetworkMode, SandboxConfig, SandboxType};
use super::error::{Result, SandboxError};
use super::sandbox::Sandbox;
use async_trait::async_trait;
use std::path::Path;
use std::process::Output;
use tokio::process::Command;

/// Docker-based sandbox.
pub struct DockerSandbox {
    /// Sandbox configuration.
    config: SandboxConfig,
    /// Container ID (if created).
    container_id: Option<String>,
}

impl DockerSandbox {
    /// Creates a new Docker sandbox.
    ///
    /// # Arguments
    /// * `config` - Sandbox configuration
    ///
    /// # Errors
    /// Returns error if Docker is not available
    pub fn new(config: SandboxConfig) -> Result<Self> {
        // Verify Docker is available
        std::process::Command::new("docker")
            .arg("--version")
            .output()
            .map_err(|e| {
                SandboxError::ContainerRuntimeNotFound(format!("Docker not found: {}", e))
            })?;

        Ok(Self {
            config,
            container_id: None,
        })
    }

    /// Gets the container image to use.
    fn get_image(&self) -> String {
        self.config
            .image
            .clone()
            .unwrap_or_else(|| "rust:latest".to_string())
    }

    /// Builds the docker run command arguments.
    fn build_run_args(&self) -> Vec<String> {
        let mut args = vec!["run".to_string(), "--rm".to_string()];

        // Network mode
        match self.config.network {
            NetworkMode::Open => {}
            NetworkMode::Closed => {
                args.push("--network=none".to_string());
            }
            NetworkMode::Proxied => {
                args.push("--network=host".to_string());
            }
        }

        // Volumes
        for volume in &self.config.volumes {
            args.push("-v".to_string());
            args.push(volume.clone());
        }

        // Working directory
        if let Some(ref working_dir) = self.config.working_dir {
            args.push("-w".to_string());
            args.push(working_dir.clone());
        }

        // Environment variables
        for (key, value) in &self.config.env {
            args.push("-e".to_string());
            args.push(format!("{}={}", key, value));
        }

        // Custom flags
        args.extend(self.config.custom_flags.clone());

        args
    }
}

#[async_trait]
impl Sandbox for DockerSandbox {
    async fn initialize(&mut self) -> Result<()> {
        // Pull image if not exists
        let image = self.get_image();

        let output = Command::new("docker")
            .args(["pull", &image])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SandboxError::InitFailed(format!(
                "Failed to pull Docker image: {}",
                stderr
            )));
        }

        Ok(())
    }

    async fn execute(
        &self,
        command: &str,
        args: &[String],
        _cwd: Option<&Path>,
    ) -> Result<Output> {
        let mut docker_args = self.build_run_args();
        docker_args.push(self.get_image());
        docker_args.push(command.to_string());
        docker_args.extend(args.iter().cloned());

        let output = Command::new("docker").args(&docker_args).output().await?;

        Ok(output)
    }

    async fn cleanup(&mut self) -> Result<()> {
        // Container is automatically removed with --rm flag
        Ok(())
    }

    fn sandbox_type(&self) -> SandboxType {
        SandboxType::Docker
    }
}

impl Drop for DockerSandbox {
    fn drop(&mut self) {
        // Cleanup container if still running
        if let Some(ref container_id) = self.container_id {
            let _ = std::process::Command::new("docker")
                .args(["rm", "-f", container_id])
                .output();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_docker_sandbox_new() {
        let config = SandboxConfig::new(SandboxType::Docker);

        // Test might fail if Docker not installed
        match DockerSandbox::new(config) {
            Ok(sandbox) => {
                assert_eq!(sandbox.sandbox_type(), SandboxType::Docker);
            }
            Err(SandboxError::ContainerRuntimeNotFound(_)) => {
                // Docker not available, skip test
                println!("Docker not available, skipping test");
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_docker_sandbox_build_args() {
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());

        let config = SandboxConfig::new(SandboxType::Docker)
            .with_network(NetworkMode::Closed)
            .with_working_dir("/app".to_string())
            .with_env(env)
            .with_volumes(vec!["/host:/container".to_string()]);

        if let Ok(sandbox) = DockerSandbox::new(config) {
            let args = sandbox.build_run_args();

            assert!(args.contains(&"--network=none".to_string()));
            assert!(args.contains(&"-w".to_string()));
            assert!(args.contains(&"/app".to_string()));
            assert!(args.contains(&"-v".to_string()));
            assert!(args.contains(&"/host:/container".to_string()));
            assert!(args.contains(&"-e".to_string()));
        }
    }

    #[tokio::test]
    async fn test_docker_sandbox_execute() {
        let config = SandboxConfig::new(SandboxType::Docker).with_image("alpine:latest".to_string());

        if let Ok(sandbox) = DockerSandbox::new(config) {
            let result = sandbox
                .execute("echo", &["hello".to_string()], None)
                .await;

            // This test will only pass if Docker is available
            match result {
                Ok(output) => {
                    assert!(output.status.success());
                    assert_eq!(
                        String::from_utf8_lossy(&output.stdout).trim(),
                        "hello"
                    );
                }
                Err(_) => {
                    println!("Docker not available or image pull failed, skipping test");
                }
            }
        }
    }
}

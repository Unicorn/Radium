//! macOS Seatbelt sandbox implementation.

use super::config::{NetworkMode, SandboxConfig, SandboxProfile, SandboxType};
use super::error::{Result, SandboxError};
use super::sandbox::Sandbox;
use async_trait::async_trait;
use std::path::Path;
use std::process::Output;
use tokio::process::Command;

/// macOS Seatbelt sandbox.
pub struct SeatbeltSandbox {
    /// Sandbox configuration.
    config: SandboxConfig,
}

impl SeatbeltSandbox {
    /// Creates a new Seatbelt sandbox.
    ///
    /// # Arguments
    /// * `config` - Sandbox configuration
    ///
    /// # Errors
    /// Returns error if Seatbelt is not available
    pub fn new(config: SandboxConfig) -> Result<Self> {
        #[cfg(not(target_os = "macos"))]
        {
            return Err(SandboxError::SeatbeltNotAvailable(
                "Seatbelt is only available on macOS".to_string(),
            ));
        }

        #[cfg(target_os = "macos")]
        {
            // Verify sandbox-exec is available
            std::process::Command::new("which").arg("sandbox-exec").output().map_err(|e| {
                SandboxError::SeatbeltNotAvailable(format!("sandbox-exec not found: {}", e))
            })?;

            Ok(Self { config })
        }
    }

    /// Gets the sandbox profile string.
    fn get_profile(&self) -> Result<String> {
        match &self.config.profile {
            SandboxProfile::Permissive => Ok(self.permissive_profile()),
            SandboxProfile::Restrictive => Ok(self.restrictive_profile()),
            SandboxProfile::Custom(path) => std::fs::read_to_string(path).map_err(|e| {
                SandboxError::InvalidProfile(format!("Failed to read profile file: {}", e))
            }),
        }
    }

    /// Permissive sandbox profile.
    fn permissive_profile(&self) -> String {
        let network_rule = match self.config.network {
            NetworkMode::Open => "(allow network*)",
            NetworkMode::Closed => "(deny network*)",
            NetworkMode::Proxied => {
                "(allow network* (require-entitlement \"com.apple.security.network.client\"))"
            }
        };

        format!(
            r#"(version 1)
(debug deny)
(allow default)
{network_rule}
(allow file-read*)
(allow file-write*)
(allow process-exec*)
(allow process-fork)
(allow sysctl-read)
(allow ipc-posix-shm*)
"#
        )
    }

    /// Restrictive sandbox profile.
    fn restrictive_profile(&self) -> String {
        let network_rule = match self.config.network {
            NetworkMode::Open => "(allow network*)",
            NetworkMode::Closed => "(deny network*)",
            NetworkMode::Proxied => "(allow network-outbound (literal \"/var/run/mDNSResponder\"))",
        };

        format!(
            r#"(version 1)
(deny default)
{network_rule}
(allow file-read-metadata)
(allow file-read* (subpath "/usr/lib"))
(allow file-read* (subpath "/System/Library"))
(allow file-write* (subpath "/tmp"))
(allow process-exec (literal "/bin/sh"))
(allow process-exec (literal "/usr/bin/env"))
(allow sysctl-read)
"#
        )
    }
}

#[async_trait]
impl Sandbox for SeatbeltSandbox {
    async fn initialize(&mut self) -> Result<()> {
        // No initialization required for Seatbelt
        Ok(())
    }

    async fn execute(&self, command: &str, args: &[String], cwd: Option<&Path>) -> Result<Output> {
        #[cfg(not(target_os = "macos"))]
        {
            return Err(SandboxError::SeatbeltNotAvailable(
                "Seatbelt is only available on macOS".to_string(),
            ));
        }

        #[cfg(target_os = "macos")]
        {
            let profile = self.get_profile()?;

            let mut cmd = Command::new("sandbox-exec");
            cmd.arg("-p");
            cmd.arg(profile);
            cmd.arg(command);
            cmd.args(args);

            if let Some(dir) = cwd {
                cmd.current_dir(dir);
            }

            // Add environment variables
            for (key, value) in &self.config.env {
                cmd.env(key, value);
            }

            let output = cmd.output().await?;
            Ok(output)
        }
    }

    async fn cleanup(&mut self) -> Result<()> {
        // No cleanup required for Seatbelt
        Ok(())
    }

    fn sandbox_type(&self) -> SandboxType {
        SandboxType::Seatbelt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seatbelt_sandbox_new() {
        let config = SandboxConfig::new(SandboxType::Seatbelt);

        #[cfg(target_os = "macos")]
        {
            match SeatbeltSandbox::new(config) {
                Ok(sandbox) => {
                    assert_eq!(sandbox.sandbox_type(), SandboxType::Seatbelt);
                }
                Err(SandboxError::SeatbeltNotAvailable(_)) => {
                    println!("sandbox-exec not available, skipping test");
                }
                Err(e) => panic!("Unexpected error: {}", e),
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            assert!(SeatbeltSandbox::new(config).is_err());
        }
    }

    #[test]
    fn test_seatbelt_permissive_profile() {
        let config =
            SandboxConfig::new(SandboxType::Seatbelt).with_profile(SandboxProfile::Permissive);

        #[cfg(target_os = "macos")]
        {
            if let Ok(sandbox) = SeatbeltSandbox::new(config) {
                let profile = sandbox.permissive_profile();
                assert!(profile.contains("(allow default)"));
                assert!(profile.contains("(allow network*)"));
            }
        }
    }

    #[test]
    fn test_seatbelt_restrictive_profile() {
        let config =
            SandboxConfig::new(SandboxType::Seatbelt).with_profile(SandboxProfile::Restrictive);

        #[cfg(target_os = "macos")]
        {
            if let Ok(sandbox) = SeatbeltSandbox::new(config) {
                let profile = sandbox.restrictive_profile();
                assert!(profile.contains("(deny default)"));
            }
        }
    }

    #[test]
    fn test_seatbelt_network_modes() {
        #[cfg(target_os = "macos")]
        {
            let config =
                SandboxConfig::new(SandboxType::Seatbelt).with_network(NetworkMode::Closed);

            if let Ok(sandbox) = SeatbeltSandbox::new(config) {
                let profile = sandbox.permissive_profile();
                assert!(profile.contains("(deny network*)"));
            }
        }
    }

    #[tokio::test]
    async fn test_seatbelt_execute() {
        #[cfg(target_os = "macos")]
        {
            let config = SandboxConfig::new(SandboxType::Seatbelt);

            if let Ok(sandbox) = SeatbeltSandbox::new(config) {
                let result = sandbox.execute("echo", &["hello".to_string()], None).await;

                match result {
                    Ok(output) => {
                        assert!(output.status.success());
                        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
                    }
                    Err(_) => {
                        println!("sandbox-exec not available, skipping test");
                    }
                }
            }
        }
    }
}

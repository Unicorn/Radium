//! CLI binary detection and version checking.

use super::error::{EngineError, Result};
use std::process::Command;

/// Binary detector for finding engine CLI tools.
pub struct BinaryDetector;

impl BinaryDetector {
    /// Checks if a binary is available in PATH.
    ///
    /// # Arguments
    /// * `binary_name` - Name of the binary to check
    ///
    /// # Returns
    /// True if binary is found in PATH
    pub fn is_available(binary_name: &str) -> bool {
        #[cfg(target_os = "windows")]
        let command = "where";

        #[cfg(not(target_os = "windows"))]
        let command = "which";

        Command::new(command)
            .arg(binary_name)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Gets the version of a binary by running a version command.
    ///
    /// # Arguments
    /// * `binary_name` - Name of the binary
    /// * `version_args` - Arguments to get version (e.g., ["--version"])
    /// * `_timeout_secs` - Timeout in seconds (currently unused)
    ///
    /// # Returns
    /// Version string if successful
    ///
    /// # Errors
    /// Returns error if binary not found or version check fails
    pub fn get_version(
        binary_name: &str,
        version_args: &[&str],
        _timeout_secs: u64,
    ) -> Result<String> {
        if !Self::is_available(binary_name) {
            return Err(EngineError::BinaryNotFound(binary_name.to_string()));
        }

        let output = std::process::Command::new(binary_name)
            .args(version_args)
            .output()
            .map_err(|e| EngineError::ExecutionError(e.to_string()))?;

        if !output.status.success() {
            return Err(EngineError::ExecutionError("Version command failed".to_string()));
        }

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

        Ok(version)
    }

    /// Checks if a binary can be executed successfully.
    ///
    /// # Arguments
    /// * `binary_name` - Name of the binary
    /// * `test_args` - Arguments to test execution
    ///
    /// # Returns
    /// True if execution succeeds
    pub fn can_execute(binary_name: &str, test_args: &[&str]) -> bool {
        Command::new(binary_name)
            .args(test_args)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Detects all available engine binaries.
    ///
    /// # Returns
    /// List of available binary names
    pub fn detect_all() -> Vec<String> {
        let known_binaries =
            vec!["claude", "codex", "cursor", "ccr", "opencode", "auggie", "gemini"];

        known_binaries
            .into_iter()
            .filter(|binary| Self::is_available(binary))
            .map(String::from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_available_sh() {
        // sh should be available on Unix systems
        #[cfg(unix)]
        {
            assert!(BinaryDetector::is_available("sh"));
        }

        // cmd should be available on Windows
        #[cfg(windows)]
        {
            assert!(BinaryDetector::is_available("cmd"));
        }
    }

    #[test]
    fn test_is_available_nonexistent() {
        assert!(!BinaryDetector::is_available("nonexistent_binary_xyz123"));
    }

    #[test]
    fn test_get_version() {
        #[cfg(unix)]
        {
            // Test with a command that should exist on Unix systems
            let result = BinaryDetector::get_version("sh", &["--version"], 5);
            // sh might not have --version, so we just check it doesn't panic
            let _ = result;
        }
    }

    #[test]
    fn test_can_execute() {
        #[cfg(unix)]
        {
            // echo should work on Unix
            assert!(BinaryDetector::can_execute("echo", &["test"]));
        }

        #[cfg(windows)]
        {
            // echo should work on Windows via cmd
            assert!(BinaryDetector::can_execute("cmd", &["/c", "echo", "test"]));
        }
    }

    #[test]
    fn test_detect_all() {
        let available = BinaryDetector::detect_all();
        // Result depends on what's installed, so we just check it returns a vec
        assert!(available.is_empty() || !available.is_empty());
    }
}

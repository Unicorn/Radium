//! Braingrid source reader.

use async_trait::async_trait;
use tokio::process::Command as TokioCommand;

use super::traits::SourceReader;
use super::types::{SourceError, SourceMetadata};

/// Reader for Braingrid node sources (braingrid:// scheme).
pub struct BraingridReader {
    /// Project ID for Braingrid (defaults to PROJ-14).
    project_id: String,

    /// Braingrid CLI token (optional, can use default auth).
    token: Option<String>,
}

impl BraingridReader {
    /// Creates a new Braingrid reader with default settings.
    ///
    /// Reads from environment variables:
    /// - `BRAINGRID_PROJECT_ID` - Project ID (defaults to "PROJ-14")
    /// - `BRAINGRID_TOKEN` - API token (optional)
    pub fn new() -> Self {
        #[allow(clippy::disallowed_methods)]
        let project_id = std::env::var("BRAINGRID_PROJECT_ID")
            .unwrap_or_else(|_| "PROJ-14".to_string());
        #[allow(clippy::disallowed_methods)]
        let token = std::env::var("BRAINGRID_TOKEN").ok();

        Self { project_id, token }
    }

    /// Creates a new Braingrid reader with explicit project ID.
    pub fn with_project_id(project_id: String) -> Self {
        #[allow(clippy::disallowed_methods)]
        let token = std::env::var("BRAINGRID_TOKEN").ok();
        Self { project_id, token }
    }

    /// Extracts node ID from URI.
    fn extract_node_id(&self, uri: &str) -> Result<String, SourceError> {
        // Remove braingrid:// scheme
        let node_id = uri.strip_prefix("braingrid://").unwrap_or(uri).trim();

        if node_id.is_empty() {
            return Err(SourceError::invalid_uri(&format!(
                "Invalid Braingrid URI format: {} (expected braingrid://NODE-ID)",
                uri
            )));
        }

        Ok(node_id.to_string())
    }

    /// Runs a Braingrid CLI command and returns the output.
    async fn run_braingrid_command(
        &self,
        args: &[&str],
    ) -> Result<String, SourceError> {
        let mut cmd = TokioCommand::new("braingrid");
        cmd.args(args);

        if let Some(ref token) = self.token {
            // If token is provided, we'd need to pass it via env or CLI arg
            // For now, assume it's set in environment or CLI config
        }

        let output = cmd.output().await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SourceError::other(
                    "braingrid CLI not found. Please ensure braingrid is installed and in PATH.",
                )
            } else {
                SourceError::other(format!("Failed to execute braingrid command: {}", e))
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not found") || stderr.contains("NotFound") {
                return Err(SourceError::not_found(&format!(
                    "Braingrid node not found: {}",
                    stderr
                )));
            }
            if stderr.contains("Unauthorized") || stderr.contains("authentication") {
                return Err(SourceError::unauthorized(&format!(
                    "Braingrid authentication failed: {}",
                    stderr
                )));
            }
            return Err(SourceError::other(format!(
                "Braingrid command failed: {}",
                stderr
            )));
        }

        String::from_utf8(output.stdout).map_err(|e| {
            SourceError::other(format!("Failed to parse braingrid output as UTF-8: {}", e))
        })
    }
}

impl Default for BraingridReader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourceReader for BraingridReader {
    fn scheme(&self) -> &str {
        "braingrid"
    }

    async fn verify(&self, uri: &str) -> Result<SourceMetadata, SourceError> {
        let node_id = self.extract_node_id(uri)?;

        // Use braingrid CLI to check if node exists
        // For requirements, use: braingrid requirement show NODE-ID -p PROJECT_ID
        // For tasks, we'd need to check differently
        // For simplicity, we'll try to fetch it and check the result
        let args = vec!["requirement", "show", &node_id, "-p", &self.project_id];

        match self.run_braingrid_command(&args).await {
            Ok(_) => {
                // Node exists - we don't have size info from CLI, so return minimal metadata
                Ok(SourceMetadata::new(true))
            }
            Err(SourceError::NotFound(_)) => {
                // Try as task instead
                let task_args = vec!["task", "show", &node_id, "-p", &self.project_id];
                match self.run_braingrid_command(&task_args).await {
                    Ok(_) => Ok(SourceMetadata::new(true)),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }

    async fn fetch(&self, uri: &str) -> Result<String, SourceError> {
        let node_id = self.extract_node_id(uri)?;

        // Try as requirement first
        let args = vec!["requirement", "show", &node_id, "-p", &self.project_id];
        match self.run_braingrid_command(&args).await {
            Ok(content) => Ok(content),
            Err(SourceError::NotFound(_)) => {
                // Try as task
                let task_args = vec!["task", "show", &node_id, "-p", &self.project_id];
                self.run_braingrid_command(&task_args).await
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_node_id() {
        let reader = BraingridReader::new();
        assert_eq!(
            reader.extract_node_id("braingrid://REQ-123").unwrap(),
            "REQ-123"
        );
        assert_eq!(
            reader.extract_node_id("braingrid://TASK-456").unwrap(),
            "TASK-456"
        );
    }

    #[test]
    fn test_scheme() {
        let reader = BraingridReader::new();
        assert_eq!(reader.scheme(), "braingrid");
    }
}

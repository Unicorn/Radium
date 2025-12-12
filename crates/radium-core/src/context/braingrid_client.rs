//! Braingrid Client - Bidirectional integration with Braingrid CLI
//!
//! Provides full read/write capabilities for Braingrid requirements and tasks,
//! enabling autonomous workflow execution with real-time status synchronization.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::process::Command as TokioCommand;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use regex::Regex;

/// Errors that can occur during Braingrid operations
#[derive(Debug, Error)]
pub enum BraingridError {
    /// Braingrid CLI not found or not in PATH
    #[error("Braingrid CLI not found at '{0}'. Please ensure braingrid is installed and in PATH, or set BRAINGRID_CLI_PATH environment variable.")]
    CliNotFound(String),

    /// Authentication failed
    #[error("Braingrid authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Requirement or task not found
    #[error("Braingrid resource not found: {0}")]
    NotFound(String),

    /// Invalid status transition
    #[error("Invalid status transition: {0}")]
    InvalidStatus(String),

    /// Breakdown operation failed
    #[error("Failed to breakdown requirement: {0}")]
    BreakdownFailed(String),

    /// Network or connection error
    #[error("Network error connecting to Braingrid: {0}")]
    NetworkError(String),

    /// JSON parsing error
    #[error("Failed to parse JSON response: {0}")]
    ParseError(String),

    /// Command execution timeout
    #[error("Braingrid command timed out after {0:?}")]
    Timeout(Duration),

    /// Generic command execution error
    #[error("Braingrid command failed: {0}")]
    CommandFailed(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl BraingridError {
    /// Get error code for the error type
    pub fn error_code(&self) -> &'static str {
        match self {
            BraingridError::CliNotFound(_) => "CLI_NOT_FOUND",
            BraingridError::AuthenticationFailed(_) => "AUTHENTICATION_FAILED",
            BraingridError::NotFound(_) => "NOT_FOUND",
            BraingridError::InvalidStatus(_) => "INVALID_STATUS",
            BraingridError::BreakdownFailed(_) => "BREAKDOWN_FAILED",
            BraingridError::NetworkError(_) => "NETWORK_ERROR",
            BraingridError::ParseError(_) => "PARSE_ERROR",
            BraingridError::Timeout(_) => "TIMEOUT",
            BraingridError::CommandFailed(_) => "COMMAND_FAILED",
            BraingridError::IoError(_) => "IO_ERROR",
        }
    }
}

/// Result type for Braingrid operations
pub type Result<T> = std::result::Result<T, BraingridError>;

/// Status values for Braingrid requirements
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequirementStatus {
    Idea,
    Planned,
    InProgress,
    Review,
    Completed,
    Cancelled,
}

/// Status values for Braingrid tasks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    Planned,
    InProgress,
    Completed,
    Cancelled,
}

/// Braingrid task representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraingridTask {
    pub id: String,
    #[serde(default)]
    pub short_id: Option<String>,
    pub number: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub status: TaskStatus,
    #[serde(default)]
    pub assigned_to: Option<String>,
    #[serde(default, rename = "blocked_by")]
    pub dependencies: Vec<String>,
}

impl BraingridTask {
    /// Get the task ID to use for Braingrid CLI commands.
    /// Uses short_id if available, otherwise constructs from number.
    pub fn task_id(&self) -> String {
        self.short_id
            .clone()
            .unwrap_or_else(|| format!("TASK-{}", self.number))
    }
}

/// Braingrid requirement representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraingridRequirement {
    pub id: String,
    pub name: String,
    pub content: String,
    pub status: RequirementStatus,
    #[serde(default)]
    pub assigned_to: Option<String>,
    #[serde(default)]
    pub tasks: Vec<BraingridTask>,
}

/// Cache entry for Braingrid data
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    data: T,
    timestamp: std::time::Instant,
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Current cache size (number of entries)
    pub size: usize,
}

impl CacheStats {
    /// Calculate hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// Braingrid client with read/write capabilities and caching
pub struct BraingridClient {
    project_id: String,
    /// Cache for requirement trees (requirement + tasks)
    requirement_cache: Arc<RwLock<HashMap<String, CacheEntry<BraingridRequirement>>>>,
    /// Cache statistics (hits, misses, size)
    cache_stats: Arc<RwLock<CacheStats>>,
    /// Cache TTL in seconds
    cache_ttl: std::time::Duration,
    /// Command timeout duration (default: 30 seconds)
    command_timeout: Duration,
    /// Braingrid CLI path (default: "braingrid")
    cli_path: String,
}

impl BraingridClient {
    /// Create a new Braingrid client
    pub fn new(project_id: impl Into<String>) -> Self {
        let cli_path = std::env::var("BRAINGRID_CLI_PATH")
            .unwrap_or_else(|_| "braingrid".to_string());
        
        Self {
            project_id: project_id.into(),
            requirement_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_stats: Arc::new(RwLock::new(CacheStats::default())),
            cache_ttl: std::time::Duration::from_secs(300), // 5 minutes default
            command_timeout: Duration::from_secs(30), // 30 seconds default
            cli_path,
        }
    }

    /// Set cache TTL
    pub fn with_cache_ttl(mut self, ttl_seconds: u64) -> Self {
        self.cache_ttl = std::time::Duration::from_secs(ttl_seconds);
        self
    }

    /// Set command timeout
    pub fn with_command_timeout(mut self, timeout_seconds: u64) -> Self {
        self.command_timeout = Duration::from_secs(timeout_seconds);
        self
    }

    /// Set CLI path
    pub fn with_cli_path(mut self, cli_path: impl Into<String>) -> Self {
        self.cli_path = cli_path.into();
        self
    }

    /// Execute a Braingrid CLI command with timeout and error handling
    async fn execute_command(&self, args: &[&str]) -> Result<String> {
        let mut cmd = TokioCommand::new(&self.cli_path);
        cmd.args(args);

        // Execute with timeout
        let output = timeout(self.command_timeout, cmd.output())
            .await
            .map_err(|_| BraingridError::Timeout(self.command_timeout))?
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    BraingridError::CliNotFound(self.cli_path.clone())
                } else {
                    BraingridError::IoError(e)
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let error_msg = format!("{}", stderr);
            
            // Check for common error patterns
            if stderr.contains("not found") || stderr.contains("NotFound") || stdout.contains("not found") {
                return Err(BraingridError::NotFound(error_msg));
            }
            if stderr.contains("Unauthorized") || stderr.contains("authentication") || stderr.contains("401") {
                return Err(BraingridError::AuthenticationFailed(error_msg));
            }
            if stderr.contains("network") || stderr.contains("connection") || stderr.contains("timeout") {
                return Err(BraingridError::NetworkError(error_msg));
            }
            if stderr.contains("invalid status") || stderr.contains("status transition") {
                return Err(BraingridError::InvalidStatus(error_msg));
            }
            
            return Err(BraingridError::CommandFailed(error_msg));
        }

        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| BraingridError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse command output as UTF-8: {}", e)
            )))?;
        
        Ok(stdout)
    }

    /// Fetch a single requirement (without tasks)
    pub async fn fetch_requirement(&self, req_id: &str) -> Result<BraingridRequirement> {
        let output = self
            .execute_command(&[
                "requirement",
                "show",
                req_id,
                "-p",
                &self.project_id,
                "--format",
                "json",
            ])
            .await
            .map_err(|e| match e {
                BraingridError::NotFound(_) => BraingridError::NotFound(format!("Requirement {} not found", req_id)),
                _ => e,
            })?;

        // Strip spinner animation and extract JSON (starts with '{')
        let json_start = output.find('{').unwrap_or(0);
        let json_str = &output[json_start..];
        serde_json::from_str(json_str)
            .map_err(|e| BraingridError::ParseError(format!("Failed to parse requirement JSON: {}", e)))
    }

    /// Create a new requirement using `braingrid specify`.
    ///
    /// Returns the created requirement ID (e.g., "REQ-173") when it can be extracted from output.
    pub async fn specify_requirement(&self, text: &str) -> Result<String> {
        // `braingrid specify -p PROJ-14 "text..."` returns human output.
        // We'll robustly extract the first REQ-### occurrence.
        let output = self
            .execute_command(&["specify", "-p", &self.project_id, text])
            .await?;

        let re = Regex::new(r"\bREQ-\d+\b").map_err(|e| {
            BraingridError::CommandFailed(format!("Failed to compile REQ regex: {}", e))
        })?;

        if let Some(m) = re.find(&output) {
            return Ok(m.as_str().to_string());
        }

        // If output didn't contain a REQ id, surface the output for debugging.
        Err(BraingridError::ParseError(format!(
            "Could not find created requirement ID in braingrid output. Output:\n{}",
            output
        )))
    }

    /// Update a requirement with a freeform action (Braingrid `requirement update --action ...`)
    pub async fn update_requirement_action(&self, req_id: &str, action: &str) -> Result<()> {
        self.execute_command(&[
            "requirement",
            "update",
            req_id,
            "-p",
            &self.project_id,
            "--action",
            action,
        ])
        .await
        .map_err(|e| match e {
            BraingridError::NotFound(_) => BraingridError::NotFound(format!("Requirement {} not found", req_id)),
            _ => e,
        })?;

        // Invalidate cache (requirement content/tasks may have changed)
        {
            let mut cache = self.requirement_cache.write().await;
            cache.remove(req_id);
            let mut stats = self.cache_stats.write().await;
            stats.size = cache.len();
        }

        Ok(())
    }

    /// Fetch requirement tree (requirement + all tasks) with caching
    pub async fn fetch_requirement_tree(&self, req_id: &str) -> Result<BraingridRequirement> {
        // Check cache first
        {
            let cache = self.requirement_cache.read().await;
            if let Some(entry) = cache.get(req_id) {
                if entry.timestamp.elapsed() < self.cache_ttl {
                    // Cache hit
                    {
                        let mut stats = self.cache_stats.write().await;
                        stats.hits += 1;
                    }
                    return Ok(entry.data.clone());
                }
            }
            // Cache miss or expired
            {
                let mut stats = self.cache_stats.write().await;
                stats.misses += 1;
            }
        }

        // Cache miss or expired - fetch from Braingrid
        let output = self
            .execute_command(&[
                "requirement",
                "build",
                req_id,
                "-p",
                &self.project_id,
                "--format",
                "json",
            ])
            .await
            .map_err(|e| match e {
                BraingridError::NotFound(_) => BraingridError::NotFound(format!("Requirement {} not found", req_id)),
                _ => e,
            })?;

        // Strip spinner animation and extract JSON (starts with '{')
        let json_start = output.find('{').unwrap_or(0);
        let json_str = &output[json_start..];
        let requirement: BraingridRequirement = serde_json::from_str(json_str)
            .map_err(|e| BraingridError::ParseError(format!("Failed to parse requirement tree JSON: {}", e)))?;

        // Update cache
        {
            let mut cache = self.requirement_cache.write().await;
            let was_new = !cache.contains_key(req_id);
            cache.insert(
                req_id.to_string(),
                CacheEntry {
                    data: requirement.clone(),
                    timestamp: std::time::Instant::now(),
                },
            );
            // Update cache size in stats
            {
                let mut stats = self.cache_stats.write().await;
                stats.size = cache.len();
            }
        }

        Ok(requirement)
    }

    /// List all tasks for a requirement
    pub async fn list_tasks(&self, req_id: &str) -> Result<Vec<BraingridTask>> {
        let output = self
            .execute_command(&[
                "task",
                "list",
                "-r",
                req_id,
                "-p",
                &self.project_id,
                "--format",
                "json",
            ])
            .await
            .map_err(|e| match e {
                BraingridError::NotFound(_) => BraingridError::NotFound(format!("Requirement {} not found", req_id)),
                _ => e,
            })?;

        serde_json::from_str(&output)
            .map_err(|e| BraingridError::ParseError(format!("Failed to parse task list JSON: {}", e)))
    }

    /// Update task status
    pub async fn update_task_status(
        &self,
        task_id: &str,
        requirement_id: &str,
        status: TaskStatus,
        notes: Option<&str>,
    ) -> Result<()> {
        let mut args = vec![
            "task",
            "update",
            task_id,
            "-r",
            requirement_id,
            "-p",
            &self.project_id,
            "--status",
        ];

        let status_str = match status {
            TaskStatus::Planned => "PLANNED",
            TaskStatus::InProgress => "IN_PROGRESS",
            TaskStatus::Completed => "COMPLETED",
            TaskStatus::Cancelled => "CANCELLED",
        };
        args.push(status_str);

        // Note: braingrid CLI doesn't support --notes option yet
        // TODO: Add notes support when braingrid CLI implements it
        let _ = notes; // Suppress unused warning

        self.execute_command(&args)
            .await
            .map_err(|e| match e {
                BraingridError::NotFound(_) => BraingridError::NotFound(format!("Task {} not found", task_id)),
                BraingridError::InvalidStatus(_) => BraingridError::InvalidStatus(format!("Invalid status transition for task {}", task_id)),
                _ => e,
            })?;

        // Invalidate cache for the requirement since task status changed
        {
            let mut cache = self.requirement_cache.write().await;
            cache.remove(requirement_id);
            // Update cache size in stats
            let mut stats = self.cache_stats.write().await;
            stats.size = cache.len();
        }

        Ok(())
    }

    /// Update requirement status
    pub async fn update_requirement_status(
        &self,
        req_id: &str,
        status: RequirementStatus,
    ) -> Result<()> {
        let status_str = match status {
            RequirementStatus::Idea => "IDEA",
            RequirementStatus::Planned => "PLANNED",
            RequirementStatus::InProgress => "IN_PROGRESS",
            RequirementStatus::Review => "REVIEW",
            RequirementStatus::Completed => "COMPLETED",
            RequirementStatus::Cancelled => "CANCELLED",
        };

        self.execute_command(&[
            "requirement",
            "update",
            req_id,
            "-p",
            &self.project_id,
            "--status",
            status_str,
        ])
        .await
        .map_err(|e| match e {
            BraingridError::NotFound(_) => BraingridError::NotFound(format!("Requirement {} not found", req_id)),
            BraingridError::InvalidStatus(_) => BraingridError::InvalidStatus(format!("Invalid status transition for requirement {}", req_id)),
            _ => e,
        })?;

        // Invalidate cache
        {
            let mut cache = self.requirement_cache.write().await;
            cache.remove(req_id);
            // Update cache size in stats
            let mut stats = self.cache_stats.write().await;
            stats.size = cache.len();
        }

        Ok(())
    }

    /// Trigger requirement breakdown (AI generates tasks)
    pub async fn breakdown_requirement(&self, req_id: &str) -> Result<Vec<BraingridTask>> {
        self.execute_command(&[
            "requirement",
            "breakdown",
            req_id,
            "-p",
            &self.project_id,
        ])
        .await
        .map_err(|e| match e {
            BraingridError::NotFound(_) => BraingridError::NotFound(format!("Requirement {} not found", req_id)),
            _ => BraingridError::BreakdownFailed(format!("Failed to breakdown requirement {}: {}", req_id, e)),
        })?;

        // Invalidate cache since tasks were added
        {
            let mut cache = self.requirement_cache.write().await;
            cache.remove(req_id);
            // Update cache size in stats
            let mut stats = self.cache_stats.write().await;
            stats.size = cache.len();
        }

        // Fetch updated task list
        self.list_tasks(req_id).await
    }

    /// Clear all cached data
    pub async fn clear_cache(&self) {
        let mut cache = self.requirement_cache.write().await;
        cache.clear();
        // Reset cache size in stats
        let mut stats = self.cache_stats.write().await;
        stats.size = 0;
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        let cache = self.requirement_cache.read().await;
        let stats = self.cache_stats.read().await;
        CacheStats {
            hits: stats.hits,
            misses: stats.misses,
            size: cache.len(), // Use actual cache size
        }
    }

    /// Get project ID
    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    /// Get cache TTL (for testing)
    #[cfg(test)]
    pub fn cache_ttl(&self) -> std::time::Duration {
        self.cache_ttl
    }

    /// Get command timeout (for testing)
    #[cfg(test)]
    pub fn command_timeout(&self) -> Duration {
        self.command_timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_braingrid_client_creation() {
        let client = BraingridClient::new("PROJ-14");
        assert_eq!(client.project_id(), "PROJ-14");
    }

    #[tokio::test]
    async fn test_cache_ttl_configuration() {
        let client = BraingridClient::new("PROJ-14").with_cache_ttl(600);
        assert_eq!(client.cache_ttl.as_secs(), 600);
    }

    #[tokio::test]
    async fn test_status_serialization() {
        let status = RequirementStatus::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"IN_PROGRESS\"");

        let task_status = TaskStatus::Completed;
        let json = serde_json::to_string(&task_status).unwrap();
        assert_eq!(json, "\"COMPLETED\"");
    }

    #[tokio::test]
    async fn test_cache_stats_default() {
        let stats = CacheStats::default();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.size, 0);
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[tokio::test]
    async fn test_cache_stats_hit_rate() {
        let mut stats = CacheStats::default();
        stats.hits = 80;
        stats.misses = 20;
        assert_eq!(stats.hit_rate(), 80.0);
    }

    #[tokio::test]
    async fn test_braingrid_error_codes() {
        let error = BraingridError::CliNotFound("braingrid".to_string());
        assert_eq!(error.error_code(), "CLI_NOT_FOUND");

        let error = BraingridError::NotFound("REQ-123".to_string());
        assert_eq!(error.error_code(), "NOT_FOUND");

        let error = BraingridError::AuthenticationFailed("Invalid token".to_string());
        assert_eq!(error.error_code(), "AUTHENTICATION_FAILED");
    }

    #[tokio::test]
    async fn test_braingrid_client_with_timeout() {
        let client = BraingridClient::new("PROJ-14").with_command_timeout(60);
        // Verify timeout is set
        assert_eq!(client.command_timeout().as_secs(), 60);
    }

    #[test]
    fn test_extract_req_id_regex() {
        let re = Regex::new(r"\bREQ-\d+\b").unwrap();
        let s = "Created requirement REQ-173 successfully";
        let m = re.find(s).unwrap();
        assert_eq!(m.as_str(), "REQ-173");
    }
}

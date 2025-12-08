//! Braingrid Client - Bidirectional integration with Braingrid CLI
//!
//! Provides full read/write capabilities for Braingrid requirements and tasks,
//! enabling autonomous workflow execution with real-time status synchronization.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::process::Command as TokioCommand;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use std::collections::HashMap;

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
            .map_err(|_| anyhow!("Braingrid command timed out after {:?}", self.command_timeout))?
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    anyhow!(
                        "Braingrid CLI not found at '{}'. Please ensure braingrid is installed and in PATH, or set BRAINGRID_CLI_PATH environment variable.",
                        self.cli_path
                    )
                } else {
                    anyhow!("Failed to execute braingrid command: {}", e)
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // Check for common error patterns
            if stderr.contains("not found") || stderr.contains("NotFound") || stdout.contains("not found") {
                return Err(anyhow!("Braingrid resource not found: {}", stderr));
            }
            if stderr.contains("Unauthorized") || stderr.contains("authentication") || stderr.contains("401") {
                return Err(anyhow!("Braingrid authentication failed: {}", stderr));
            }
            if stderr.contains("network") || stderr.contains("connection") || stderr.contains("timeout") {
                return Err(anyhow!("Network error connecting to Braingrid: {}", stderr));
            }
            
            return Err(anyhow!("Braingrid command failed: {}", stderr));
        }

        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| anyhow!("Failed to parse command output as UTF-8: {}", e))?;
        
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
            .context(format!("Failed to fetch requirement {}", req_id))?;

        // Strip spinner animation and extract JSON (starts with '{')
        let json_start = output.find('{').unwrap_or(0);
        let json_str = &output[json_start..];
        serde_json::from_str(json_str)
            .context("Failed to parse requirement JSON")
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
            .context(format!("Failed to build requirement tree for {}", req_id))?;

        // Strip spinner animation and extract JSON (starts with '{')
        let json_start = output.find('{').unwrap_or(0);
        let json_str = &output[json_start..];
        let requirement: BraingridRequirement = serde_json::from_str(json_str)
            .context("Failed to parse requirement tree JSON")?;

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
            .context(format!("Failed to list tasks for {}", req_id))?;

        serde_json::from_str(&output)
            .context("Failed to parse task list JSON")
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
            .context(format!("Failed to update task {} status", task_id))?;

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
        .context(format!("Failed to update requirement {} status", req_id))?;

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
        .context(format!("Failed to breakdown requirement {}", req_id))?;

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
}

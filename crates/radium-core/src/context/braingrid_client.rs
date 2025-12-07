//! Braingrid Client - Bidirectional integration with Braingrid CLI
//!
//! Provides full read/write capabilities for Braingrid requirements and tasks,
//! enabling autonomous workflow execution with real-time status synchronization.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;
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

/// Braingrid client with read/write capabilities and caching
pub struct BraingridClient {
    project_id: String,
    /// Cache for requirement trees (requirement + tasks)
    requirement_cache: Arc<RwLock<HashMap<String, CacheEntry<BraingridRequirement>>>>,
    /// Cache TTL in seconds
    cache_ttl: std::time::Duration,
}

impl BraingridClient {
    /// Create a new Braingrid client
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            requirement_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: std::time::Duration::from_secs(300), // 5 minutes default
        }
    }

    /// Set cache TTL
    pub fn with_cache_ttl(mut self, ttl_seconds: u64) -> Self {
        self.cache_ttl = std::time::Duration::from_secs(ttl_seconds);
        self
    }

    /// Fetch a single requirement (without tasks)
    pub async fn fetch_requirement(&self, req_id: &str) -> Result<BraingridRequirement> {
        let output = Command::new("braingrid")
            .args(&[
                "requirement",
                "show",
                req_id,
                "-p",
                &self.project_id,
                "--format",
                "json",
            ])
            .output()
            .context("Failed to execute braingrid command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "Failed to fetch requirement {}: {}",
                req_id,
                stderr
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Strip spinner animation and extract JSON (starts with '{')
        let json_start = stdout.find('{').unwrap_or(0);
        let json_str = &stdout[json_start..];
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
                    return Ok(entry.data.clone());
                }
            }
        }

        // Cache miss or expired - fetch from Braingrid
        let output = Command::new("braingrid")
            .args(&[
                "requirement",
                "build",
                req_id,
                "-p",
                &self.project_id,
                "--format",
                "json",
            ])
            .output()
            .context("Failed to execute braingrid requirement build")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "Failed to build requirement tree for {}: {}",
                req_id,
                stderr
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Strip spinner animation and extract JSON (starts with '{')
        let json_start = stdout.find('{').unwrap_or(0);
        let json_str = &stdout[json_start..];
        let requirement: BraingridRequirement = serde_json::from_str(json_str)
            .context("Failed to parse requirement tree JSON")?;

        // Update cache
        {
            let mut cache = self.requirement_cache.write().await;
            cache.insert(
                req_id.to_string(),
                CacheEntry {
                    data: requirement.clone(),
                    timestamp: std::time::Instant::now(),
                },
            );
        }

        Ok(requirement)
    }

    /// List all tasks for a requirement
    pub async fn list_tasks(&self, req_id: &str) -> Result<Vec<BraingridTask>> {
        let output = Command::new("braingrid")
            .args(&[
                "task",
                "list",
                "-r",
                req_id,
                "-p",
                &self.project_id,
                "--format",
                "json",
            ])
            .output()
            .context("Failed to execute braingrid task list")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "Failed to list tasks for {}: {}",
                req_id,
                stderr
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&stdout)
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

        let output = Command::new("braingrid")
            .args(&args)
            .output()
            .context("Failed to execute braingrid task update")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "Failed to update task {} status: {}",
                task_id,
                stderr
            ));
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

        let output = Command::new("braingrid")
            .args(&[
                "requirement",
                "update",
                req_id,
                "-p",
                &self.project_id,
                "--status",
                status_str,
            ])
            .output()
            .context("Failed to execute braingrid requirement update")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "Failed to update requirement {} status: {}",
                req_id,
                stderr
            ));
        }

        // Invalidate cache
        {
            let mut cache = self.requirement_cache.write().await;
            cache.remove(req_id);
        }

        Ok(())
    }

    /// Trigger requirement breakdown (AI generates tasks)
    pub async fn breakdown_requirement(&self, req_id: &str) -> Result<Vec<BraingridTask>> {
        let output = Command::new("braingrid")
            .args(&[
                "requirement",
                "breakdown",
                req_id,
                "-p",
                &self.project_id,
            ])
            .output()
            .context("Failed to execute braingrid requirement breakdown")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "Failed to breakdown requirement {}: {}",
                req_id,
                stderr
            ));
        }

        // Invalidate cache since tasks were added
        {
            let mut cache = self.requirement_cache.write().await;
            cache.remove(req_id);
        }

        // Fetch updated task list
        self.list_tasks(req_id).await
    }

    /// Clear all cached data
    pub async fn clear_cache(&self) {
        let mut cache = self.requirement_cache.write().await;
        cache.clear();
    }

    /// Get project ID
    pub fn project_id(&self) -> &str {
        &self.project_id
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

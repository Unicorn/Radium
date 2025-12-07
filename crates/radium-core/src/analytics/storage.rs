//! Session report storage and persistence.

use super::report::SessionReport;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Lightweight session metadata for efficient listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Session ID
    pub session_id: String,
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Session duration
    pub duration: Duration,
    /// Total tool calls
    pub tool_calls: u64,
}

/// Session report storage manager.
pub struct SessionStorage {
    sessions_dir: PathBuf,
}

impl SessionStorage {
    /// Create a new session storage manager.
    pub fn new(workspace_root: &Path) -> Result<Self> {
        let sessions_dir = workspace_root.join(".radium").join("_internals").join("sessions");

        fs::create_dir_all(&sessions_dir)?;

        Ok(Self { sessions_dir })
    }

    /// Save a session report to disk.
    pub fn save_report(&self, report: &SessionReport) -> Result<PathBuf> {
        let filename = format!("{}.json", report.metrics.session_id);
        let file_path = self.sessions_dir.join(&filename);

        let json = serde_json::to_string_pretty(report)?;
        fs::write(&file_path, json)?;

        Ok(file_path)
    }

    /// Load a session report by session ID.
    pub fn load_report(&self, session_id: &str) -> Result<SessionReport> {
        let filename = format!("{}.json", session_id);
        let file_path = self.sessions_dir.join(&filename);

        let content = fs::read_to_string(&file_path)?;
        let report: SessionReport = serde_json::from_str(&content)?;

        Ok(report)
    }

    /// List all stored session reports.
    ///
    /// For backward compatibility, this method loads all reports.
    /// For better performance with large numbers of sessions, use `list_reports_paginated()`.
    pub fn list_reports(&self) -> Result<Vec<SessionReport>> {
        self.list_reports_paginated(None, None)
    }

    /// List stored session reports with pagination.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of reports to return (None = all)
    /// * `offset` - Number of reports to skip (None = 0)
    ///
    /// # Returns
    /// Vector of session reports sorted by generation time (most recent first)
    pub fn list_reports_paginated(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<SessionReport>> {
        let mut reports = Vec::new();

        if !self.sessions_dir.exists() {
            return Ok(reports);
        }

        // First, collect all metadata to sort efficiently
        let mut metadata_list: Vec<(PathBuf, DateTime<Utc>)> = Vec::new();

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Try to read just enough to get generated_at timestamp
                if let Ok(content) = fs::read_to_string(&path) {
                    // Use a lightweight JSON parser to extract just generated_at
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(gen_at_str) = json_value.get("generated_at").and_then(|v| v.as_str()) {
                            if let Ok(generated_at) = gen_at_str.parse::<DateTime<Utc>>() {
                                metadata_list.push((path, generated_at));
                            }
                        }
                    }
                }
            }
        }

        // Sort by generation time (most recent first)
        metadata_list.sort_by(|a, b| b.1.cmp(&a.1));

        // Apply pagination
        let offset = offset.unwrap_or(0);
        let limit = limit.map(|l| l + offset).unwrap_or(metadata_list.len());
        let paginated_paths: Vec<_> = metadata_list
            .into_iter()
            .skip(offset)
            .take(limit - offset)
            .map(|(path, _)| path)
            .collect();

        // Now load only the paginated reports
        for path in paginated_paths {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(report) = serde_json::from_str::<SessionReport>(&content) {
                    reports.push(report);
                }
            }
        }

        // Ensure final sort (in case of any issues)
        reports.sort_by(|a, b| b.generated_at.cmp(&a.generated_at));

        Ok(reports)
    }

    /// List session metadata only (without loading full reports).
    ///
    /// This is more efficient for large numbers of sessions as it only
    /// reads minimal data from each JSON file.
    pub fn list_report_metadata(&self) -> Result<Vec<SessionMetadata>> {
        let mut metadata = Vec::new();

        if !self.sessions_dir.exists() {
            return Ok(metadata);
        }

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let (Some(metrics), Some(gen_at_str)) = (
                            json_value.get("metrics"),
                            json_value.get("generated_at").and_then(|v| v.as_str()),
                        ) {
                            if let Ok(generated_at) = gen_at_str.parse::<DateTime<Utc>>() {
                                let session_id = metrics
                                    .get("session_id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let tool_calls = metrics
                                    .get("tool_calls")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0);
                                
                                // Calculate duration from wall_time if available
                                // Duration serializes as { "secs": u64, "nanos": u32 }
                                let duration = if let Some(wall_time_obj) = metrics.get("wall_time") {
                                    if let Some(secs) = wall_time_obj.get("secs").and_then(|v| v.as_u64()) {
                                        let nanos = wall_time_obj.get("nanos").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                        Duration::new(secs, nanos)
                                    } else {
                                        Duration::ZERO
                                    }
                                } else {
                                    Duration::ZERO
                                };

                                metadata.push(SessionMetadata {
                                    session_id,
                                    generated_at,
                                    duration,
                                    tool_calls,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Sort by generation time (most recent first)
        metadata.sort_by(|a, b| b.generated_at.cmp(&a.generated_at));

        Ok(metadata)
    }

    /// Get the sessions directory path.
    pub fn sessions_dir(&self) -> &Path {
        &self.sessions_dir
    }
}

//! Session report storage and persistence.

use super::report::SessionReport;
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

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
    pub fn list_reports(&self) -> Result<Vec<SessionReport>> {
        let mut reports = Vec::new();

        if !self.sessions_dir.exists() {
            return Ok(reports);
        }

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(report) = serde_json::from_str::<SessionReport>(&content) {
                        reports.push(report);
                    }
                }
            }
        }

        // Sort by generation time (most recent first)
        reports.sort_by(|a, b| b.generated_at.cmp(&a.generated_at));

        Ok(reports)
    }

    /// Get the sessions directory path.
    pub fn sessions_dir(&self) -> &Path {
        &self.sessions_dir
    }
}

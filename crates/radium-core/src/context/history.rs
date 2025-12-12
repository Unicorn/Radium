//! Session-based conversation history tracking.
//!
//! Provides history continuity by tracking interactions per session ID
//! and generating summaries to prevent context window bloat.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Maximum number of interactions to keep per session.
const MAX_INTERACTIONS: usize = 10;

/// Number of recent interactions to include in summary.
const SUMMARY_INTERACTIONS: usize = 5;

/// Interaction record for history tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    /// Goal or objective from the interaction.
    pub goal: String,
    /// Plan or approach from the interaction.
    pub plan: String,
    /// Output or guidance from the interaction.
    pub output: String,
    /// Timestamp when this interaction occurred.
    pub timestamp: DateTime<Utc>,
    /// Optional metadata from the model response (e.g., finish_reason, safety_ratings, citations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Errors that can occur during history operations.
#[derive(Error, Debug)]
pub enum HistoryError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for history operations.
pub type Result<T> = std::result::Result<T, HistoryError>;

/// History manager for session-based conversation tracking.
pub struct HistoryManager {
    /// Path to the history file.
    history_path: PathBuf,
    /// In-memory cache of session histories.
    histories: HashMap<String, Vec<Interaction>>,
}

impl HistoryManager {
    /// Creates a new history manager.
    ///
    /// # Arguments
    /// * `data_dir` - Directory to store history files
    ///
    /// # Returns
    /// A new history manager instance
    ///
    /// # Errors
    /// Returns error if directory creation or file loading fails
    pub fn new(data_dir: impl AsRef<Path>) -> Result<Self> {
        let data_dir = data_dir.as_ref();
        fs::create_dir_all(data_dir)?;

        let history_path = data_dir.join("history.json");
        let histories =
            if history_path.exists() { Self::load_history(&history_path)? } else { HashMap::new() };

        Ok(Self { history_path, histories })
    }

    /// Adds an interaction to a session's history.
    ///
    /// # Arguments
    /// * `session_id` - The session identifier (defaults to "default" if None)
    /// * `goal` - Goal or objective
    /// * `plan` - Plan or approach
    /// * `output` - Output or guidance
    ///
    /// # Errors
    /// Returns error if save fails
    pub fn add_interaction(
        &mut self,
        session_id: Option<&str>,
        goal: String,
        plan: String,
        output: String,
    ) -> Result<()> {
        self.add_interaction_with_metadata(session_id, goal, plan, output, None)
    }

    /// Adds an interaction to a session's history with optional model metadata.
    pub fn add_interaction_with_metadata(
        &mut self,
        session_id: Option<&str>,
        goal: String,
        plan: String,
        output: String,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<()> {
        let session_id = session_id.unwrap_or("default").to_string();
        let interaction = Interaction { goal, plan, output, timestamp: Utc::now(), metadata };

        let session_history = self.histories.entry(session_id).or_default();
        session_history.push(interaction);

        // Keep only last MAX_INTERACTIONS
        if session_history.len() > MAX_INTERACTIONS {
            session_history.remove(0);
        }

        // Save to disk
        Self::save_history(&self.history_path, &self.histories)?;

        Ok(())
    }

    /// Gets a summary of recent interactions for a session.
    ///
    /// # Arguments
    /// * `session_id` - The session identifier (defaults to "default" if None)
    ///
    /// # Returns
    /// Summary string of recent interactions, or empty string if no history
    pub fn get_summary(&self, session_id: Option<&str>) -> String {
        let session_id = session_id.unwrap_or("default");
        let Some(session_history) = self.histories.get(session_id) else {
            return String::new();
        };

        if session_history.is_empty() {
            return String::new();
        }

        // Get last SUMMARY_INTERACTIONS interactions
        let recent: Vec<&Interaction> =
            session_history.iter().rev().take(SUMMARY_INTERACTIONS).collect();

        let mut summary = String::from("History Context:\n");
        for (i, interaction) in recent.iter().rev().enumerate() {
            let output_preview = if interaction.output.len() > 100 {
                format!("{}...", &interaction.output[..100])
            } else {
                interaction.output.clone()
            };

            writeln!(
                summary,
                "Interaction {}: Goal: {}, Plan: {}, Guidance: {}",
                i + 1,
                interaction.goal,
                interaction.plan,
                output_preview
            )
            .unwrap_or_else(|e| {
                tracing::error!("Failed to write history summary: {}", e);
            });
        }

        summary
    }

    /// Gets all interactions for a session.
    ///
    /// # Arguments
    /// * `session_id` - The session identifier (defaults to "default" if None)
    ///
    /// # Returns
    /// Vector of interactions for the session
    pub fn get_interactions(&self, session_id: Option<&str>) -> Vec<Interaction> {
        let session_id = session_id.unwrap_or("default");
        self.histories.get(session_id).cloned().unwrap_or_default()
    }

    /// Clears history for a session.
    ///
    /// # Arguments
    /// * `session_id` - The session identifier (defaults to "default" if None)
    ///
    /// # Errors
    /// Returns error if save fails
    pub fn clear_session(&mut self, session_id: Option<&str>) -> Result<()> {
        let session_id = session_id.unwrap_or("default").to_string();
        self.histories.remove(&session_id);
        Self::save_history(&self.history_path, &self.histories)?;
        Ok(())
    }

    /// Loads history from disk.
    fn load_history(path: &Path) -> Result<HashMap<String, Vec<Interaction>>> {
        let content = fs::read_to_string(path)?;
        let histories: HashMap<String, Vec<Interaction>> = serde_json::from_str(&content)?;
        Ok(histories)
    }

    /// Saves history to disk.
    fn save_history(path: &Path, histories: &HashMap<String, Vec<Interaction>>) -> Result<()> {
        let json = serde_json::to_string_pretty(histories)?;
        fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_history_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let _manager = HistoryManager::new(temp_dir.path()).unwrap();
        // File is created lazily when data is saved, not on new()
        // So we verify the directory exists and manager was created successfully
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_history_manager_add_interaction() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = HistoryManager::new(temp_dir.path()).unwrap();

        manager
            .add_interaction(
                Some("session-1"),
                "Build feature".to_string(),
                "Use React".to_string(),
                "Guidance here".to_string(),
            )
            .unwrap();

        let interactions = manager.get_interactions(Some("session-1"));
        assert_eq!(interactions.len(), 1);
        assert_eq!(interactions[0].goal, "Build feature");
    }

    #[test]
    fn test_history_manager_default_session() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = HistoryManager::new(temp_dir.path()).unwrap();

        manager
            .add_interaction(None, "Goal".to_string(), "Plan".to_string(), "Output".to_string())
            .unwrap();

        let interactions = manager.get_interactions(None);
        assert_eq!(interactions.len(), 1);
    }

    #[test]
    fn test_history_manager_get_summary() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = HistoryManager::new(temp_dir.path()).unwrap();

        for i in 0..3 {
            manager
                .add_interaction(
                    Some("session-1"),
                    format!("Goal {}", i),
                    format!("Plan {}", i),
                    format!("Output {}", i),
                )
                .unwrap();
        }

        let summary = manager.get_summary(Some("session-1"));
        assert!(summary.contains("History Context"));
        assert!(summary.contains("Goal 0"));
        assert!(summary.contains("Goal 2"));
    }

    #[test]
    fn test_history_manager_max_interactions() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = HistoryManager::new(temp_dir.path()).unwrap();

        // Add more than MAX_INTERACTIONS
        for i in 0..=MAX_INTERACTIONS {
            manager
                .add_interaction(
                    Some("session-1"),
                    format!("Goal {}", i),
                    "Plan".to_string(),
                    "Output".to_string(),
                )
                .unwrap();
        }

        let interactions = manager.get_interactions(Some("session-1"));
        assert_eq!(interactions.len(), MAX_INTERACTIONS);
        // First interaction should be removed
        assert!(!interactions.iter().any(|i| i.goal == "Goal 0"));
        // Last interaction should be present
        assert!(interactions.iter().any(|i| i.goal == format!("Goal {}", MAX_INTERACTIONS)));
    }

    #[test]
    fn test_history_manager_clear_session() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = HistoryManager::new(temp_dir.path()).unwrap();

        manager
            .add_interaction(
                Some("session-1"),
                "Goal".to_string(),
                "Plan".to_string(),
                "Output".to_string(),
            )
            .unwrap();

        manager.clear_session(Some("session-1")).unwrap();

        let interactions = manager.get_interactions(Some("session-1"));
        assert!(interactions.is_empty());
    }

    #[test]
    fn test_history_manager_empty_summary() {
        let temp_dir = TempDir::new().unwrap();
        let manager = HistoryManager::new(temp_dir.path()).unwrap();

        let summary = manager.get_summary(Some("nonexistent"));
        assert!(summary.is_empty());
    }
}

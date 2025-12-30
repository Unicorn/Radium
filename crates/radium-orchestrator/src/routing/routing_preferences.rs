//! Per-project routing preferences and feedback tracking (Phase 2 - REQ-246).
//!
//! This module provides persistent storage for routing preferences on a per-workspace basis,
//! allowing Radium to learn from user feedback and improve routing decisions over time.

use crate::routing::SkillRoutingResult;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Feedback rating for a routing decision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeedbackRating {
    /// Positive feedback (correct routing).
    Positive,
    /// Negative feedback (incorrect routing).
    Negative,
    /// Neutral / no strong opinion.
    Neutral,
}

/// Record of user feedback for a specific routing decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingFeedbackRecord {
    /// When the feedback was provided.
    pub timestamp: u64,

    /// The task description that was routed.
    pub task_description: String,

    /// The agent that was selected.
    pub selected_agent: String,

    /// Routing confidence score.
    pub confidence: f32,

    /// Routing method used ("skill" | "keyword" | "ml" | "adaptive").
    pub routing_method: String,

    /// User's feedback rating.
    pub user_feedback: FeedbackRating,

    /// Whether execution succeeded.
    pub execution_success: bool,

    /// Number of retries (implicit negative signal).
    pub retry_count: u32,

    /// Optional user comment.
    pub comment: Option<String>,
}

/// Task type categorization for routing preferences.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskType {
    /// Code generation/modification.
    Code,
    /// Testing and validation.
    Test,
    /// Documentation.
    Documentation,
    /// Architecture and design.
    Architecture,
    /// Debugging and troubleshooting.
    Debug,
    /// Data processing.
    Data,
    /// Deployment and infrastructure.
    Deployment,
    /// Security and compliance.
    Security,
    /// Other/unknown.
    Other,
}

/// Per-project routing preferences and learning data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingPreferences {
    /// Workspace root directory this preferences file is associated with.
    pub workspace_root: PathBuf,

    /// Preferred agents for each task type (learned from feedback).
    pub preferred_agents: HashMap<TaskType, Vec<String>>,

    /// Agents the user has explicitly blocked for this project.
    pub blocked_agents: Vec<String>,

    /// Skill confidence overrides (boost or penalize specific skills).
    /// Values > 1.0 boost confidence, < 1.0 penalize.
    pub skill_confidence_overrides: HashMap<String, f32>,

    /// Domain-specific agent preferences (e.g., "testing" -> "test-specialist").
    pub domain_preferences: HashMap<String, String>,

    /// Historical feedback records.
    pub feedback_history: Vec<RoutingFeedbackRecord>,

    /// Positive outcomes per agent (agent_id -> count).
    pub positive_outcomes: HashMap<String, u32>,

    /// Negative outcomes per agent (agent_id -> count).
    pub negative_outcomes: HashMap<String, u32>,

    /// When these preferences were last updated.
    pub last_updated: u64,

    /// Total number of routing decisions tracked.
    pub total_routings: u32,

    /// Estimated routing accuracy for this project (0.0-1.0).
    pub accuracy_estimate: f32,
}

impl RoutingPreferences {
    /// Create new routing preferences for a workspace.
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            preferred_agents: HashMap::new(),
            blocked_agents: Vec::new(),
            skill_confidence_overrides: HashMap::new(),
            domain_preferences: HashMap::new(),
            feedback_history: Vec::new(),
            positive_outcomes: HashMap::new(),
            negative_outcomes: HashMap::new(),
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            total_routings: 0,
            accuracy_estimate: 0.0,
        }
    }

    /// Load preferences from disk, or create new if not exists.
    pub fn load(workspace_root: &Path) -> Result<Self> {
        let prefs_file = Self::preferences_path(workspace_root);

        if prefs_file.exists() {
            let content = fs::read_to_string(&prefs_file)
                .with_context(|| format!("Failed to read preferences from {}", prefs_file.display()))?;

            let mut prefs: RoutingPreferences = serde_json::from_str(&content)
                .with_context(|| "Failed to parse routing preferences JSON")?;

            // Update workspace_root in case it moved
            prefs.workspace_root = workspace_root.to_path_buf();

            Ok(prefs)
        } else {
            Ok(Self::new(workspace_root.to_path_buf()))
        }
    }

    /// Save preferences to disk.
    pub fn save(&self) -> Result<()> {
        let prefs_file = Self::preferences_path(&self.workspace_root);

        // Create .radium directory if it doesn't exist
        if let Some(parent) = prefs_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        let content = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize routing preferences")?;

        fs::write(&prefs_file, content)
            .with_context(|| format!("Failed to write preferences to {}", prefs_file.display()))?;

        Ok(())
    }

    /// Get the path to the preferences file for a workspace.
    fn preferences_path(workspace_root: &Path) -> PathBuf {
        workspace_root.join(".radium").join("routing_preferences.json")
    }

    /// Record feedback for a routing decision.
    pub fn record_feedback(&mut self, feedback: RoutingFeedbackRecord) {
        // Update outcome counts
        match feedback.user_feedback {
            FeedbackRating::Positive => {
                *self.positive_outcomes.entry(feedback.selected_agent.clone()).or_insert(0) += 1;
            }
            FeedbackRating::Negative => {
                *self.negative_outcomes.entry(feedback.selected_agent.clone()).or_insert(0) += 1;
            }
            FeedbackRating::Neutral => {}
        }

        // Add to history
        self.feedback_history.push(feedback);

        // Update metadata
        self.total_routings += 1;
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Recalculate accuracy
        self.recalculate_accuracy();
    }

    /// Calculate routing accuracy based on feedback.
    fn recalculate_accuracy(&mut self) {
        if self.total_routings == 0 {
            self.accuracy_estimate = 0.0;
            return;
        }

        let positive = self.positive_outcomes.values().sum::<u32>() as f32;
        let negative = self.negative_outcomes.values().sum::<u32>() as f32;
        let total = positive + negative;

        if total > 0.0 {
            self.accuracy_estimate = positive / total;
        } else {
            self.accuracy_estimate = 0.0;
        }
    }

    /// Get confidence adjustment for a skill (1.0 = no adjustment).
    pub fn get_skill_confidence_multiplier(&self, skill_name: &str) -> f32 {
        self.skill_confidence_overrides
            .get(skill_name)
            .copied()
            .unwrap_or(1.0)
    }

    /// Check if an agent is blocked for this project.
    pub fn is_agent_blocked(&self, agent_id: &str) -> bool {
        self.blocked_agents.contains(&agent_id.to_string())
    }

    /// Get success rate for a specific agent (0.0-1.0).
    pub fn get_agent_success_rate(&self, agent_id: &str) -> f32 {
        let positive = self.positive_outcomes.get(agent_id).copied().unwrap_or(0) as f32;
        let negative = self.negative_outcomes.get(agent_id).copied().unwrap_or(0) as f32;
        let total = positive + negative;

        if total > 0.0 {
            positive / total
        } else {
            0.5 // Neutral prior if no data
        }
    }

    /// Get the most recent feedback records (up to limit).
    pub fn get_recent_feedback(&self, limit: usize) -> Vec<&RoutingFeedbackRecord> {
        self.feedback_history
            .iter()
            .rev()
            .take(limit)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_preferences() {
        let temp_dir = TempDir::new().unwrap();
        let prefs = RoutingPreferences::new(temp_dir.path().to_path_buf());

        assert_eq!(prefs.total_routings, 0);
        assert_eq!(prefs.accuracy_estimate, 0.0);
        assert!(prefs.feedback_history.is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut prefs = RoutingPreferences::new(temp_dir.path().to_path_buf());

        prefs.total_routings = 10;
        prefs.accuracy_estimate = 0.85;
        prefs.save().unwrap();

        let loaded = RoutingPreferences::load(temp_dir.path()).unwrap();
        assert_eq!(loaded.total_routings, 10);
        assert_eq!(loaded.accuracy_estimate, 0.85);
    }

    #[test]
    fn test_record_positive_feedback() {
        let temp_dir = TempDir::new().unwrap();
        let mut prefs = RoutingPreferences::new(temp_dir.path().to_path_buf());

        let feedback = RoutingFeedbackRecord {
            timestamp: 12345,
            task_description: "Test task".to_string(),
            selected_agent: "code-agent".to_string(),
            confidence: 0.9,
            routing_method: "skill".to_string(),
            user_feedback: FeedbackRating::Positive,
            execution_success: true,
            retry_count: 0,
            comment: None,
        };

        prefs.record_feedback(feedback);

        assert_eq!(prefs.total_routings, 1);
        assert_eq!(prefs.positive_outcomes.get("code-agent"), Some(&1));
        assert_eq!(prefs.accuracy_estimate, 1.0);
    }

    #[test]
    fn test_record_negative_feedback() {
        let temp_dir = TempDir::new().unwrap();
        let mut prefs = RoutingPreferences::new(temp_dir.path().to_path_buf());

        let feedback = RoutingFeedbackRecord {
            timestamp: 12345,
            task_description: "Test task".to_string(),
            selected_agent: "code-agent".to_string(),
            confidence: 0.3,
            routing_method: "keyword".to_string(),
            user_feedback: FeedbackRating::Negative,
            execution_success: false,
            retry_count: 2,
            comment: Some("Wrong agent selected".to_string()),
        };

        prefs.record_feedback(feedback);

        assert_eq!(prefs.total_routings, 1);
        assert_eq!(prefs.negative_outcomes.get("code-agent"), Some(&1));
        assert_eq!(prefs.accuracy_estimate, 0.0);
    }

    #[test]
    fn test_accuracy_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let mut prefs = RoutingPreferences::new(temp_dir.path().to_path_buf());

        // Add 3 positive and 1 negative
        for _ in 0..3 {
            prefs.record_feedback(RoutingFeedbackRecord {
                timestamp: 12345,
                task_description: "Test".to_string(),
                selected_agent: "test-agent".to_string(),
                confidence: 0.9,
                routing_method: "skill".to_string(),
                user_feedback: FeedbackRating::Positive,
                execution_success: true,
                retry_count: 0,
                comment: None,
            });
        }

        prefs.record_feedback(RoutingFeedbackRecord {
            timestamp: 12346,
            task_description: "Test".to_string(),
            selected_agent: "test-agent".to_string(),
            confidence: 0.5,
            routing_method: "keyword".to_string(),
            user_feedback: FeedbackRating::Negative,
            execution_success: false,
            retry_count: 1,
            comment: None,
        });

        assert_eq!(prefs.total_routings, 4);
        assert_eq!(prefs.accuracy_estimate, 0.75);
    }

    #[test]
    fn test_is_agent_blocked() {
        let temp_dir = TempDir::new().unwrap();
        let mut prefs = RoutingPreferences::new(temp_dir.path().to_path_buf());

        prefs.blocked_agents.push("bad-agent".to_string());

        assert!(prefs.is_agent_blocked("bad-agent"));
        assert!(!prefs.is_agent_blocked("good-agent"));
    }

    #[test]
    fn test_skill_confidence_multiplier() {
        let temp_dir = TempDir::new().unwrap();
        let mut prefs = RoutingPreferences::new(temp_dir.path().to_path_buf());

        prefs.skill_confidence_overrides.insert("boosted-skill".to_string(), 1.5);
        prefs.skill_confidence_overrides.insert("penalized-skill".to_string(), 0.5);

        assert_eq!(prefs.get_skill_confidence_multiplier("boosted-skill"), 1.5);
        assert_eq!(prefs.get_skill_confidence_multiplier("penalized-skill"), 0.5);
        assert_eq!(prefs.get_skill_confidence_multiplier("unknown-skill"), 1.0);
    }
}

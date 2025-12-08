//! Dynamic agent reassignment on failures.
//!
//! Provides functionality to automatically reassign tasks to alternative agents
//! when agent-specific failures occur.

use crate::agents::registry::AgentRegistry;
use crate::workflow::failure::FailureType;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;

/// Reason for agent reassignment.
#[derive(Debug, Clone, PartialEq)]
pub enum ReassignmentReason {
    /// Agent failed during execution.
    AgentFailure {
        /// Agent ID that failed.
        agent_id: String,
        /// Error message.
        error: String,
    },
    /// Agent is unavailable.
    AgentUnavailable {
        /// Agent ID that is unavailable.
        agent_id: String,
    },
    /// Agent performance below threshold.
    PerformanceThreshold {
        /// Agent ID with poor performance.
        agent_id: String,
        /// Failure rate (0.0-1.0).
        failure_rate: f32,
    },
}

/// Statistics for an agent's performance.
#[derive(Debug, Clone)]
pub struct AgentStats {
    /// Number of successful executions.
    pub success_count: u32,
    /// Number of failed executions.
    pub failure_count: u32,
    /// Average execution duration in milliseconds.
    pub avg_duration_ms: u64,
}

impl AgentStats {
    /// Creates new empty stats.
    pub fn new() -> Self {
        Self { success_count: 0, failure_count: 0, avg_duration_ms: 0 }
    }

    /// Calculates the failure rate (0.0-1.0).
    pub fn failure_rate(&self) -> f32 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            0.0
        } else {
            self.failure_count as f32 / total as f32
        }
    }

    /// Calculates the success rate (0.0-1.0).
    pub fn success_rate(&self) -> f32 {
        1.0 - self.failure_rate()
    }
}

impl Default for AgentStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracks agent performance metrics.
pub struct AgentPerformanceTracker {
    /// Statistics per agent.
    agent_stats: Arc<Mutex<HashMap<String, AgentStats>>>,
}

impl AgentPerformanceTracker {
    /// Creates a new performance tracker.
    pub fn new() -> Self {
        Self { agent_stats: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// Records an execution result.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID
    /// * `success` - Whether execution succeeded
    /// * `duration` - Execution duration
    pub fn record_execution(&self, agent_id: &str, success: bool, duration: Duration) {
        let mut stats = self.agent_stats.lock().unwrap();
        let agent_stat = stats.entry(agent_id.to_string()).or_insert_with(|| AgentStats::new());

        if success {
            agent_stat.success_count += 1;
        } else {
            agent_stat.failure_count += 1;
        }

        // Update average duration (simplified - just use latest)
        agent_stat.avg_duration_ms = duration.as_millis() as u64;
    }

    /// Gets the failure rate for an agent.
    pub fn get_failure_rate(&self, agent_id: &str) -> f32 {
        let stats = self.agent_stats.lock().unwrap();
        stats.get(agent_id).map(|s| s.failure_rate()).unwrap_or(0.0)
    }

    /// Checks if an agent should be reassigned based on failure rate threshold.
    pub fn should_reassign(&self, agent_id: &str, threshold: f32) -> bool {
        self.get_failure_rate(agent_id) >= threshold
    }

    /// Gets statistics for an agent.
    pub fn get_stats(&self, agent_id: &str) -> Option<AgentStats> {
        let stats = self.agent_stats.lock().unwrap();
        stats.get(agent_id).cloned()
    }
}

impl Default for AgentPerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Selects alternative agents for reassignment.
pub struct AgentSelector {
    /// Agent registry for finding agents.
    registry: Arc<AgentRegistry>,
    /// Performance tracker for ranking agents.
    performance_tracker: Arc<Mutex<AgentPerformanceTracker>>,
}

impl AgentSelector {
    /// Creates a new agent selector.
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            registry,
            performance_tracker: Arc::new(Mutex::new(AgentPerformanceTracker::new())),
        }
    }

    /// Finds alternative agents with matching category.
    ///
    /// # Arguments
    /// * `current_agent_id` - The current agent ID
    /// * `task_category` - Optional task category to match
    ///
    /// # Returns
    /// Vector of alternative agent IDs
    pub fn find_alternatives(&self, current_agent_id: &str, task_category: Option<&str>) -> Vec<String> {
        let mut alternatives = Vec::new();

        // Get current agent to find its category
        let current_agent = match self.registry.get(current_agent_id) {
            Ok(agent) => agent,
            Err(_) => return alternatives,
        };

        let target_category = task_category
            .or_else(|| current_agent.category.as_deref())
            .unwrap_or("");

        // Find agents with matching category
        if let Ok(all_agents) = self.registry.list_all() {
            for agent in all_agents {
                if agent.id != current_agent_id {
                    let matches = if target_category.is_empty() {
                        true // If no category, consider all agents
                    } else {
                        agent.category.as_deref().map(|c| c == target_category).unwrap_or(false)
                    };

                    if matches {
                        alternatives.push(agent.id.clone());
                    }
                }
            }
        }

        alternatives
    }

    /// Selects the best agent from candidates based on performance.
    ///
    /// # Arguments
    /// * `candidates` - Vector of candidate agent IDs
    ///
    /// # Returns
    /// The best agent ID, or None if no candidates
    pub fn select_best(&self, candidates: Vec<String>) -> Option<String> {
        if candidates.is_empty() {
            return None;
        }

        let tracker = self.performance_tracker.lock().unwrap();

        // Sort by success rate (highest first), then by failure rate (lowest first)
        let mut scored: Vec<(String, f32)> = candidates
            .into_iter()
            .map(|id| {
                let stats = tracker.get_stats(&id).unwrap_or_default();
                let score = stats.success_rate() - stats.failure_rate();
                (id, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored.first().map(|(id, _)| id.clone())
    }

    /// Gets the performance tracker.
    pub fn performance_tracker(&self) -> Arc<Mutex<AgentPerformanceTracker>> {
        self.performance_tracker.clone()
    }
}

/// Errors that can occur during agent reassignment.
#[derive(Debug, Error)]
pub enum ReassignmentError {
    /// No alternative agents available.
    #[error("No alternative agents available for reassignment")]
    NoAlternatives,

    /// Maximum reassignments exceeded.
    #[error("Maximum reassignments exceeded: {0}")]
    MaxReassignmentsExceeded(u32),

    /// Agent not found.
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
}

/// Record of a reassignment.
#[derive(Debug, Clone)]
pub struct ReassignmentRecord {
    /// Timestamp of reassignment.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Original agent ID.
    pub from_agent: String,
    /// New agent ID.
    pub to_agent: String,
    /// Reason for reassignment.
    pub reason: ReassignmentReason,
}

/// Manages agent reassignment logic.
pub struct AgentReassignment {
    /// Agent selector for finding alternatives.
    selector: AgentSelector,
    /// Maximum number of reassignments allowed per task.
    max_reassignments: u32,
    /// Reassignment history per task.
    reassignment_history: Arc<Mutex<HashMap<String, Vec<ReassignmentRecord>>>>,
}

impl AgentReassignment {
    /// Creates a new agent reassignment manager.
    ///
    /// # Arguments
    /// * `selector` - The agent selector
    /// * `max_reassignments` - Maximum reassignments per task (default: 2)
    pub fn new(selector: AgentSelector, max_reassignments: Option<u32>) -> Self {
        Self {
            selector,
            max_reassignments: max_reassignments.unwrap_or(2),
            reassignment_history: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Checks if reassignment should be attempted.
    ///
    /// # Arguments
    /// * `task_id` - The task ID
    /// * `failure_type` - The type of failure
    ///
    /// # Returns
    /// True if reassignment should be attempted
    pub fn should_reassign(&self, task_id: &str, failure_type: &FailureType) -> bool {
        // Only reassign for agent failures
        if !matches!(failure_type, FailureType::AgentFailure { .. }) {
            return false;
        }

        // Check reassignment count
        let history = self.reassignment_history.lock().unwrap();
        let count = history.get(task_id).map(|r| r.len()).unwrap_or(0) as u32;
        count < self.max_reassignments
    }

    /// Reassigns an agent for a task.
    ///
    /// # Arguments
    /// * `task_id` - The task ID
    /// * `current_agent` - The current agent ID
    /// * `reason` - The reason for reassignment
    /// * `task_category` - Optional task category for matching
    ///
    /// # Returns
    /// The new agent ID
    ///
    /// # Errors
    /// Returns error if reassignment fails
    pub fn reassign_agent(
        &self,
        task_id: &str,
        current_agent: &str,
        reason: ReassignmentReason,
        task_category: Option<&str>,
    ) -> Result<String, ReassignmentError> {
        // Check reassignment limit
        let history = self.reassignment_history.lock().unwrap();
        let count = history.get(task_id).map(|r| r.len()).unwrap_or(0) as u32;
        drop(history);

        if count >= self.max_reassignments {
            return Err(ReassignmentError::MaxReassignmentsExceeded(self.max_reassignments));
        }

        // Find alternatives
        let alternatives = self.selector.find_alternatives(current_agent, task_category);
        if alternatives.is_empty() {
            return Err(ReassignmentError::NoAlternatives);
        }

        // Select best alternative
        let new_agent = self.selector.select_best(alternatives)
            .ok_or(ReassignmentError::NoAlternatives)?;

        // Record reassignment
        let mut history = self.reassignment_history.lock().unwrap();
        let records = history.entry(task_id.to_string()).or_insert_with(Vec::new);
        records.push(ReassignmentRecord {
            timestamp: chrono::Utc::now(),
            from_agent: current_agent.to_string(),
            to_agent: new_agent.clone(),
            reason,
        });

        Ok(new_agent)
    }

    /// Gets reassignment history for a task.
    pub fn get_history(&self, task_id: &str) -> Vec<ReassignmentRecord> {
        let history = self.reassignment_history.lock().unwrap();
        history.get(task_id).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_stats_failure_rate() {
        let mut stats = AgentStats::new();
        stats.success_count = 7;
        stats.failure_count = 3;

        assert_eq!(stats.failure_rate(), 0.3);
        assert_eq!(stats.success_rate(), 0.7);
    }

    #[test]
    fn test_agent_performance_tracker() {
        let tracker = AgentPerformanceTracker::new();
        tracker.record_execution("agent-1", true, Duration::from_secs(5));
        tracker.record_execution("agent-1", false, Duration::from_secs(3));

        assert_eq!(tracker.get_failure_rate("agent-1"), 0.5);
        assert!(tracker.should_reassign("agent-1", 0.4));
        assert!(!tracker.should_reassign("agent-1", 0.6));
    }

    #[test]
    fn test_reassignment_should_reassign() {
        let registry = Arc::new(AgentRegistry::new());
        let selector = AgentSelector::new(registry);
        let reassignment = AgentReassignment::new(selector, Some(2));

        assert!(reassignment.should_reassign(
            "task-1",
            &FailureType::AgentFailure {
                agent_id: "agent-1".to_string(),
                reason: "error".to_string(),
            }
        ));
        assert!(!reassignment.should_reassign(
            "task-1",
            &FailureType::Transient { reason: "timeout".to_string() }
        ));
    }
}


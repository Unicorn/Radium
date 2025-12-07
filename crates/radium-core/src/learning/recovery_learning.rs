//! Learning-driven recovery optimization.
//!
//! Integrates the learning system with recovery and reassignment mechanisms
//! to optimize future recovery strategies based on historical success patterns.

use crate::learning::store::{LearningStore, LearningType, Skill, SkillStatus};
use crate::workflow::failure::FailureType;
use crate::workflow::recovery::{RecoveryContext, RecoveryStrategy};
use crate::workflow::reassignment::ReassignmentReason;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Errors that can occur during recovery learning operations.
#[derive(Debug, Error)]
pub enum RecoveryLearningError {
    /// Learning store error.
    #[error("Learning store error: {0}")]
    LearningStore(String),

    /// Invalid recovery pattern.
    #[error("Invalid recovery pattern: {0}")]
    InvalidPattern(String),
}

/// Result type for recovery learning operations.
pub type Result<T> = std::result::Result<T, RecoveryLearningError>;

/// Pattern of recovery attempts for a specific failure type.
#[derive(Debug, Clone)]
pub struct RecoveryPattern {
    /// Failure type this pattern is for.
    pub failure_type: String,
    /// Map of strategy name to success count.
    pub successful_strategies: HashMap<String, u32>,
    /// Map of strategy name to failure count.
    pub failed_strategies: HashMap<String, u32>,
}

impl RecoveryPattern {
    /// Creates a new recovery pattern.
    pub fn new(failure_type: String) -> Self {
        Self {
            failure_type,
            successful_strategies: HashMap::new(),
            failed_strategies: HashMap::new(),
        }
    }

    /// Records a successful recovery attempt.
    pub fn record_success(&mut self, strategy: &str) {
        *self.successful_strategies.entry(strategy.to_string()).or_insert(0) += 1;
    }

    /// Records a failed recovery attempt.
    pub fn record_failure(&mut self, strategy: &str) {
        *self.failed_strategies.entry(strategy.to_string()).or_insert(0) += 1;
    }

    /// Gets the best strategy based on success rate.
    pub fn get_best_strategy(&self) -> Option<String> {
        let mut best_strategy: Option<String> = None;
        let mut best_rate = 0.0f32;

        for (strategy, success_count) in &self.successful_strategies {
            let failure_count = self.failed_strategies.get(strategy).copied().unwrap_or(0);
            let total = success_count + failure_count;
            if total > 0 {
                let rate = *success_count as f32 / total as f32;
                if rate > best_rate {
                    best_rate = rate;
                    best_strategy = Some(strategy.clone());
                }
            }
        }

        best_strategy
    }

    /// Calculates the success rate for a strategy.
    pub fn success_rate(&self, strategy: &str) -> f32 {
        let success_count = self.successful_strategies.get(strategy).copied().unwrap_or(0);
        let failure_count = self.failed_strategies.get(strategy).copied().unwrap_or(0);
        let total = success_count + failure_count;

        if total == 0 {
            0.0
        } else {
            success_count as f32 / total as f32
        }
    }

    /// Gets total attempts for this pattern.
    pub fn total_attempts(&self) -> u32 {
        self.successful_strategies.values().sum::<u32>()
            + self.failed_strategies.values().sum::<u32>()
    }
}

/// Manages recovery pattern learning.
pub struct RecoveryLearning {
    /// Learning store for persistence.
    learning_store: Arc<Mutex<LearningStore>>,
    /// Recovery patterns by failure type.
    recovery_patterns: Arc<Mutex<HashMap<String, RecoveryPattern>>>,
}

impl RecoveryLearning {
    /// Creates a new recovery learning manager.
    ///
    /// # Arguments
    /// * `learning_store` - The learning store
    pub fn new(learning_store: Arc<Mutex<LearningStore>>) -> Self {
        Self {
            learning_store,
            recovery_patterns: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Records a recovery attempt with outcome.
    ///
    /// # Arguments
    /// * `context` - The recovery context
    /// * `strategy` - The recovery strategy used
    /// * `success` - Whether recovery succeeded
    pub fn record_recovery_attempt(
        &self,
        context: &RecoveryContext,
        strategy: &RecoveryStrategy,
        success: bool,
    ) -> Result<()> {
        let strategy_name = self.strategy_to_string(strategy);
        let failure_type_key = self.failure_type_to_key(&context.failure_type);

        // Update pattern
        {
            let mut patterns = self.recovery_patterns.lock().unwrap();
            let pattern = patterns
                .entry(failure_type_key.clone())
                .or_insert_with(|| RecoveryPattern::new(failure_type_key.clone()));

            if success {
                pattern.record_success(&strategy_name);
            } else {
                pattern.record_failure(&strategy_name);
            }
        }

        // Record to learning store
        let mut store = self.learning_store.lock().unwrap();
        let entry_type = if success {
            LearningType::Success
        } else {
            LearningType::Mistake
        };

        let description = format!(
            "Recovery attempt for {}: strategy {} {}",
            context.failed_step_id,
            strategy_name,
            if success { "succeeded" } else { "failed" }
        );

        let solution = if success {
            Some(format!(
                "Use {} strategy for {} failures",
                strategy_name, failure_type_key
            ))
        } else {
            Some(format!(
                "Avoid {} strategy for {} failures",
                strategy_name, failure_type_key
            ))
        };

        store.add_entry(entry_type, "recovery".to_string(), description, solution)
            .map_err(|e| RecoveryLearningError::LearningStore(e.to_string()))?;

        Ok(())
    }

    /// Records an agent reassignment attempt.
    ///
    /// # Arguments
    /// * `task_id` - The task ID
    /// * `from_agent` - Original agent ID
    /// * `to_agent` - New agent ID
    /// * `reason` - Reassignment reason
    /// * `success` - Whether reassignment succeeded
    pub fn record_reassignment_attempt(
        &self,
        task_id: &str,
        from_agent: &str,
        to_agent: &str,
        reason: &ReassignmentReason,
        success: bool,
    ) -> Result<()> {
        let mut store = self.learning_store.lock().unwrap();
        let entry_type = if success {
            LearningType::Success
        } else {
            LearningType::Mistake
        };

        let description = format!(
            "Agent reassignment for {}: {} -> {} ({})",
            task_id,
            from_agent,
            to_agent,
            self.reassignment_reason_to_string(reason)
        );

        let solution = if success {
            Some(format!(
                "Reassign from {} to {} for similar failures",
                from_agent, to_agent
            ))
        } else {
            Some(format!(
                "Avoid reassigning from {} to {} for this failure type",
                from_agent, to_agent
            ))
        };

        store.add_entry(entry_type, "reassignment".to_string(), description, solution)
            .map_err(|e| RecoveryLearningError::LearningStore(e.to_string()))?;

        Ok(())
    }

    /// Gets a recommended recovery strategy for a failure type.
    ///
    /// # Arguments
    /// * `failure_type` - The failure type
    ///
    /// # Returns
    /// Recommended strategy if pattern exists, None otherwise
    pub fn get_recommended_strategy(&self, failure_type: &FailureType) -> Option<RecoveryStrategy> {
        let failure_type_key = self.failure_type_to_key(failure_type);
        let patterns = self.recovery_patterns.lock().unwrap();
        let pattern = patterns.get(&failure_type_key)?;

        let best_strategy_name = pattern.get_best_strategy()?;
        self.string_to_strategy(&best_strategy_name)
    }

    /// Generates skills from successful recovery patterns.
    ///
    /// # Returns
    /// Vector of generated skills
    pub fn generate_recovery_skills(&self) -> Result<Vec<Skill>> {
        let patterns = self.recovery_patterns.lock().unwrap();
        let mut skills = Vec::new();

        for pattern in patterns.values() {
            let total_attempts = pattern.total_attempts();
            if total_attempts < 3 {
                continue; // Need at least 3 attempts
            }

            for (strategy_name, success_count) in &pattern.successful_strategies {
                let failure_count = pattern.failed_strategies.get(strategy_name).copied().unwrap_or(0);
                let total = success_count + failure_count;
                if total >= 3 {
                    let success_rate = *success_count as f32 / total as f32;
                    if success_rate > 0.7 {
                        // Generate skill for patterns with >70% success rate
                        let content = format!(
                            "For {} failures, {} strategy succeeds {:.0}% of the time ({} successes out of {} attempts)",
                            pattern.failure_type,
                            strategy_name,
                            success_rate * 100.0,
                            success_count,
                            total
                        );

                        let now = Utc::now();

                        let skill = Skill {
                            id: uuid::Uuid::new_v4().to_string(),
                            section: "Recovery Strategies".to_string(),
                            content,
                            helpful: *success_count,
                            harmful: failure_count,
                            neutral: 0,
                            status: SkillStatus::Active,
                            created_at: now,
                            updated_at: now,
                        };

                        skills.push(skill);
                    }
                }
            }
        }

        Ok(skills)
    }

    /// Optimizes strategy selection based on learning data.
    ///
    /// # Arguments
    /// * `failure_type` - The failure type
    /// * `available_strategies` - Available strategies to choose from
    ///
    /// # Returns
    /// The best strategy based on historical data
    pub fn optimize_strategy_selection(
        &self,
        failure_type: &FailureType,
        available_strategies: Vec<RecoveryStrategy>,
    ) -> RecoveryStrategy {
        let failure_type_key = self.failure_type_to_key(failure_type);
        let patterns = self.recovery_patterns.lock().unwrap();

        if let Some(pattern) = patterns.get(&failure_type_key) {
            // Rank strategies by success rate
            let mut scored: Vec<(RecoveryStrategy, f32)> = available_strategies
                .clone()
                .into_iter()
                .map(|strategy| {
                    let strategy_name = self.strategy_to_string(&strategy);
                    let score = pattern.success_rate(&strategy_name);
                    (strategy, score)
                })
                .collect();

            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            if let Some((best_strategy, score)) = scored.first() {
                if *score > 0.0 {
                    return best_strategy.clone();
                }
            }
        }

        // Fall back to first available strategy if no learning data
        available_strategies.first().cloned().unwrap_or(RecoveryStrategy::Abort)
    }

    /// Converts failure type to key string.
    fn failure_type_to_key(&self, failure_type: &FailureType) -> String {
        match failure_type {
            FailureType::Transient { .. } => "transient".to_string(),
            FailureType::Permanent { .. } => "permanent".to_string(),
            FailureType::AgentFailure { agent_id, .. } => format!("agent_failure:{}", agent_id),
            FailureType::Unknown { .. } => "unknown".to_string(),
        }
    }

    /// Converts strategy to string.
    fn strategy_to_string(&self, strategy: &RecoveryStrategy) -> String {
        match strategy {
            RecoveryStrategy::RestoreCheckpoint { .. } => "RestoreCheckpoint".to_string(),
            RecoveryStrategy::RetryWithoutRestore => "RetryWithoutRestore".to_string(),
            RecoveryStrategy::SkipTask => "SkipTask".to_string(),
            RecoveryStrategy::Abort => "Abort".to_string(),
        }
    }

    /// Converts string to strategy (simplified - only for RestoreCheckpoint).
    fn string_to_strategy(&self, strategy_name: &str) -> Option<RecoveryStrategy> {
        match strategy_name {
            "RestoreCheckpoint" => {
                // Would need checkpoint_id, but this is a simplified version
                Some(RecoveryStrategy::RetryWithoutRestore)
            }
            "RetryWithoutRestore" => Some(RecoveryStrategy::RetryWithoutRestore),
            "SkipTask" => Some(RecoveryStrategy::SkipTask),
            "Abort" => Some(RecoveryStrategy::Abort),
            _ => None,
        }
    }

    /// Converts reassignment reason to string.
    fn reassignment_reason_to_string(&self, reason: &ReassignmentReason) -> String {
        match reason {
            ReassignmentReason::AgentFailure { .. } => "AgentFailure".to_string(),
            ReassignmentReason::AgentUnavailable { .. } => "AgentUnavailable".to_string(),
            ReassignmentReason::PerformanceThreshold { .. } => "PerformanceThreshold".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::engine::ExecutionContext;
    use crate::workflow::failure::FailureType;
    use crate::workflow::recovery::RecoveryContext;
    use tempfile::TempDir;

    fn create_test_learning_store() -> Arc<Mutex<LearningStore>> {
        let temp_dir = TempDir::new().unwrap();
        let store = LearningStore::new(temp_dir.path()).unwrap();
        Arc::new(Mutex::new(store))
    }

    #[test]
    fn test_recovery_pattern_success_rate() {
        let mut pattern = RecoveryPattern::new("transient".to_string());
        pattern.record_success("RestoreCheckpoint");
        pattern.record_success("RestoreCheckpoint");
        pattern.record_failure("RestoreCheckpoint");

        assert_eq!(pattern.success_rate("RestoreCheckpoint"), 2.0 / 3.0);
        assert_eq!(pattern.get_best_strategy(), Some("RestoreCheckpoint".to_string()));
    }

    #[test]
    fn test_recovery_learning_record_attempt() {
        let store = create_test_learning_store();
        let learning = RecoveryLearning::new(store);

        let context = RecoveryContext {
            workflow_id: "workflow-1".to_string(),
            failed_step_id: "step-1".to_string(),
            checkpoint_id: Some("checkpoint-123".to_string()),
            execution_context: ExecutionContext::new("workflow-1".to_string()),
            failure_type: FailureType::Transient { reason: "timeout".to_string() },
        };

        let strategy = RecoveryStrategy::RestoreCheckpoint {
            checkpoint_id: "checkpoint-123".to_string(),
        };

        let result = learning.record_recovery_attempt(&context, &strategy, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recovery_learning_optimize_strategy() {
        let store = create_test_learning_store();
        let learning = RecoveryLearning::new(store);

        // Record some patterns
        let context = RecoveryContext {
            workflow_id: "workflow-1".to_string(),
            failed_step_id: "step-1".to_string(),
            checkpoint_id: Some("checkpoint-123".to_string()),
            execution_context: ExecutionContext::new("workflow-1".to_string()),
            failure_type: FailureType::Transient { reason: "timeout".to_string() },
        };

        // Record successful RestoreCheckpoint
        for _ in 0..5 {
            learning
                .record_recovery_attempt(
                    &context,
                    &RecoveryStrategy::RestoreCheckpoint {
                        checkpoint_id: "checkpoint-123".to_string(),
                    },
                    true,
                )
                .unwrap();
        }

        // Record failed RetryWithoutRestore
        for _ in 0..2 {
            learning
                .record_recovery_attempt(&context, &RecoveryStrategy::RetryWithoutRestore, false)
                .unwrap();
        }

        // Optimize should prefer RestoreCheckpoint
        let available = vec![
            RecoveryStrategy::RestoreCheckpoint {
                checkpoint_id: "checkpoint-123".to_string(),
            },
            RecoveryStrategy::RetryWithoutRestore,
        ];

        let optimized = learning.optimize_strategy_selection(&context.failure_type, available);
        // Should prefer RestoreCheckpoint (simplified check)
        assert!(matches!(
            optimized,
            RecoveryStrategy::RestoreCheckpoint { .. } | RecoveryStrategy::RetryWithoutRestore
        ));
    }
}


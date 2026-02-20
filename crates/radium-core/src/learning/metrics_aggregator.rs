//! Routing quality metrics aggregation for Phase 2 (REQ-246).
//!
//! This module provides functionality to aggregate and analyze routing decision quality
//! from user feedback and execution outcomes. Metrics include accuracy, retry rates,
//! escalation rates, and per-agent performance tracking.

use radium_orchestrator::routing::{FeedbackRating, RoutingFeedbackRecord, RoutingPreferences};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Aggregated routing quality metrics for a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingQualityMetrics {
    /// Total number of routing decisions made.
    pub total_routings: u32,

    /// Overall routing accuracy (0.0-1.0).
    /// Calculated as (positive_feedback + successful_executions) / total
    pub accuracy: f32,

    /// Retry rate (0.0-1.0).
    /// Proportion of tasks that were retried (indicates routing failures).
    pub retry_rate: f32,

    /// Escalation rate (0.0-1.0).
    /// Proportion of tasks where user manually changed the agent.
    pub escalation_rate: f32,

    /// Error rate (0.0-1.0).
    /// Proportion of executions that failed.
    pub error_rate: f32,

    /// Per-agent performance metrics.
    pub agent_performance: HashMap<String, AgentMetrics>,

    /// Per-task-type accuracy (if task types are tracked).
    pub task_type_accuracy: HashMap<String, f32>,

    /// Confidence score statistics.
    pub confidence_stats: ConfidenceStats,
}

/// Performance metrics for a specific agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// Total number of times this agent was selected.
    pub total_selections: u32,

    /// Number of positive feedback instances.
    pub positive_feedback: u32,

    /// Number of negative feedback instances.
    pub negative_feedback: u32,

    /// Number of neutral feedback instances.
    pub neutral_feedback: u32,

    /// Success rate (0.0-1.0).
    /// Calculated as successful_executions / total_selections
    pub success_rate: f32,

    /// Average routing confidence when this agent was selected.
    pub avg_confidence: f32,

    /// Average execution time (if tracked, in milliseconds).
    pub avg_execution_time_ms: Option<f64>,
}

/// Confidence score statistics across all routing decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceStats {
    /// Average confidence across all decisions.
    pub mean: f32,

    /// Minimum confidence observed.
    pub min: f32,

    /// Maximum confidence observed.
    pub max: f32,

    /// Standard deviation of confidence scores.
    pub std_dev: f32,
}

/// Metrics aggregator for routing quality analysis.
pub struct MetricsAggregator;

impl MetricsAggregator {
    /// Aggregates routing quality metrics from preferences.
    ///
    /// # Arguments
    /// * `preferences` - Routing preferences containing feedback history
    ///
    /// # Returns
    /// Aggregated routing quality metrics
    pub fn aggregate(preferences: &RoutingPreferences) -> RoutingQualityMetrics {
        let feedback_history = &preferences.feedback_history;
        let total_routings = feedback_history.len() as u32;

        if total_routings == 0 {
            return Self::empty_metrics();
        }

        // Calculate overall metrics
        let positive_count = feedback_history
            .iter()
            .filter(|f| matches!(f.user_feedback, FeedbackRating::Positive))
            .count();
        let negative_count = feedback_history
            .iter()
            .filter(|f| matches!(f.user_feedback, FeedbackRating::Negative))
            .count();
        let successful_executions = feedback_history
            .iter()
            .filter(|f| f.execution_success)
            .count();

        let accuracy = if total_routings > 0 {
            (positive_count + successful_executions) as f32 / (total_routings * 2) as f32
        } else {
            0.0
        };

        // Calculate retry rate (count of retries / total)
        let retry_rate = feedback_history
            .iter()
            .map(|f| f.retry_count as f32)
            .sum::<f32>()
            / total_routings as f32;

        // Escalation rate (approximated by negative feedback rate)
        let escalation_rate = negative_count as f32 / total_routings as f32;

        // Error rate
        let error_rate = feedback_history
            .iter()
            .filter(|f| !f.execution_success)
            .count() as f32
            / total_routings as f32;

        // Per-agent metrics
        let agent_performance = Self::calculate_agent_metrics(feedback_history);

        // Task type accuracy (if available - currently not tracked in feedback)
        let task_type_accuracy = HashMap::new();

        // Confidence statistics
        let confidence_stats = Self::calculate_confidence_stats(feedback_history);

        RoutingQualityMetrics {
            total_routings,
            accuracy,
            retry_rate,
            escalation_rate,
            error_rate,
            agent_performance,
            task_type_accuracy,
            confidence_stats,
        }
    }

    /// Loads preferences from workspace and aggregates metrics.
    ///
    /// # Arguments
    /// * `workspace_root` - Path to workspace root directory
    ///
    /// # Returns
    /// Aggregated metrics or error if preferences cannot be loaded
    pub fn aggregate_from_workspace(workspace_root: &Path) -> Result<RoutingQualityMetrics, String> {
        let preferences = RoutingPreferences::load(workspace_root)
            .map_err(|e| format!("Failed to load preferences: {}", e))?;

        Ok(Self::aggregate(&preferences))
    }

    /// Exports metrics to JSON file.
    ///
    /// # Arguments
    /// * `metrics` - The metrics to export
    /// * `output_path` - Path to output JSON file
    ///
    /// # Returns
    /// Ok(()) on success, error message on failure
    pub fn export_to_json(
        metrics: &RoutingQualityMetrics,
        output_path: &Path,
    ) -> Result<(), String> {
        let json = serde_json::to_string_pretty(metrics)
            .map_err(|e| format!("Failed to serialize metrics: {}", e))?;

        std::fs::write(output_path, json)
            .map_err(|e| format!("Failed to write metrics file: {}", e))?;

        Ok(())
    }

    /// Calculates per-agent performance metrics.
    fn calculate_agent_metrics(
        feedback_history: &[RoutingFeedbackRecord],
    ) -> HashMap<String, AgentMetrics> {
        let mut agent_stats: HashMap<String, Vec<&RoutingFeedbackRecord>> = HashMap::new();

        // Group feedback by agent
        for record in feedback_history {
            agent_stats
                .entry(record.selected_agent.clone())
                .or_default()
                .push(record);
        }

        // Calculate metrics for each agent
        agent_stats
            .into_iter()
            .map(|(agent_id, records)| {
                let total_selections = records.len() as u32;
                let positive_feedback = records
                    .iter()
                    .filter(|r| matches!(r.user_feedback, FeedbackRating::Positive))
                    .count() as u32;
                let negative_feedback = records
                    .iter()
                    .filter(|r| matches!(r.user_feedback, FeedbackRating::Negative))
                    .count() as u32;
                let neutral_feedback = records
                    .iter()
                    .filter(|r| matches!(r.user_feedback, FeedbackRating::Neutral))
                    .count() as u32;

                let successful_executions = records.iter().filter(|r| r.execution_success).count();
                let success_rate = if total_selections > 0 {
                    successful_executions as f32 / total_selections as f32
                } else {
                    0.0
                };

                let avg_confidence = if total_selections > 0 {
                    records.iter().map(|r| r.confidence).sum::<f32>() / total_selections as f32
                } else {
                    0.0
                };

                let metrics = AgentMetrics {
                    total_selections,
                    positive_feedback,
                    negative_feedback,
                    neutral_feedback,
                    success_rate,
                    avg_confidence,
                    avg_execution_time_ms: None, // Not tracked yet
                };

                (agent_id, metrics)
            })
            .collect()
    }

    /// Calculates confidence score statistics.
    fn calculate_confidence_stats(feedback_history: &[RoutingFeedbackRecord]) -> ConfidenceStats {
        if feedback_history.is_empty() {
            return ConfidenceStats {
                mean: 0.0,
                min: 0.0,
                max: 0.0,
                std_dev: 0.0,
            };
        }

        let confidences: Vec<f32> = feedback_history.iter().map(|r| r.confidence).collect();

        let mean = confidences.iter().sum::<f32>() / confidences.len() as f32;
        let min = confidences
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let max = confidences
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        // Calculate standard deviation
        let variance = confidences
            .iter()
            .map(|c| (c - mean).powi(2))
            .sum::<f32>()
            / confidences.len() as f32;
        let std_dev = variance.sqrt();

        ConfidenceStats {
            mean,
            min,
            max,
            std_dev,
        }
    }

    /// Returns empty metrics (for when no feedback exists).
    fn empty_metrics() -> RoutingQualityMetrics {
        RoutingQualityMetrics {
            total_routings: 0,
            accuracy: 0.0,
            retry_rate: 0.0,
            escalation_rate: 0.0,
            error_rate: 0.0,
            agent_performance: HashMap::new(),
            task_type_accuracy: HashMap::new(),
            confidence_stats: ConfidenceStats {
                mean: 0.0,
                min: 0.0,
                max: 0.0,
                std_dev: 0.0,
            },
        }
    }

    /// Detects accuracy drift (alerts if accuracy drops below threshold).
    ///
    /// # Arguments
    /// * `metrics` - Current metrics
    /// * `threshold` - Accuracy threshold (e.g., 0.8 for 80%)
    ///
    /// # Returns
    /// true if accuracy is below threshold
    pub fn detect_accuracy_drift(metrics: &RoutingQualityMetrics, threshold: f32) -> bool {
        metrics.accuracy < threshold && metrics.total_routings >= 10
    }

    /// Generates a human-readable summary of routing quality.
    pub fn generate_summary(metrics: &RoutingQualityMetrics) -> String {
        let mut summary = String::new();

        summary.push_str("=== Routing Quality Metrics ===\n\n");
        summary.push_str(&format!("Total Routings: {}\n", metrics.total_routings));
        summary.push_str(&format!(
            "Overall Accuracy: {:.1}%\n",
            metrics.accuracy * 100.0
        ));
        summary.push_str(&format!(
            "Error Rate: {:.1}%\n",
            metrics.error_rate * 100.0
        ));
        summary.push_str(&format!(
            "Retry Rate: {:.1}%\n",
            metrics.retry_rate * 100.0
        ));
        summary.push_str(&format!(
            "Escalation Rate: {:.1}%\n\n",
            metrics.escalation_rate * 100.0
        ));

        summary.push_str("=== Confidence Statistics ===\n");
        summary.push_str(&format!(
            "Mean: {:.3}, Min: {:.3}, Max: {:.3}, Std Dev: {:.3}\n\n",
            metrics.confidence_stats.mean,
            metrics.confidence_stats.min,
            metrics.confidence_stats.max,
            metrics.confidence_stats.std_dev
        ));

        summary.push_str("=== Top Agents by Selection Count ===\n");
        let mut agents: Vec<_> = metrics.agent_performance.iter().collect();
        agents.sort_by(|a, b| b.1.total_selections.cmp(&a.1.total_selections));

        for (agent_id, agent_metrics) in agents.iter().take(5) {
            summary.push_str(&format!(
                "- {}: {} selections, {:.1}% success, {:.3} avg confidence\n",
                agent_id,
                agent_metrics.total_selections,
                agent_metrics.success_rate * 100.0,
                agent_metrics.avg_confidence
            ));
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn create_test_feedback(
        agent: &str,
        confidence: f32,
        success: bool,
        rating: FeedbackRating,
    ) -> RoutingFeedbackRecord {
        RoutingFeedbackRecord {
            timestamp: SystemTime::now(),
            task_description: "Test task".to_string(),
            selected_agent: agent.to_string(),
            confidence,
            user_feedback: rating,
            execution_success: success,
            retry_count: 0,
        }
    }

    #[test]
    fn test_empty_metrics() {
        let prefs = RoutingPreferences::new(std::path::PathBuf::from("/test"));
        let metrics = MetricsAggregator::aggregate(&prefs);

        assert_eq!(metrics.total_routings, 0);
        assert_eq!(metrics.accuracy, 0.0);
        assert_eq!(metrics.error_rate, 0.0);
    }

    #[test]
    fn test_basic_metrics() {
        let mut prefs = RoutingPreferences::new(std::path::PathBuf::from("/test"));

        // Add some test feedback
        prefs.record_feedback(create_test_feedback(
            "code-agent",
            0.8,
            true,
            FeedbackRating::Positive,
        ));
        prefs.record_feedback(create_test_feedback(
            "code-agent",
            0.7,
            true,
            FeedbackRating::Positive,
        ));
        prefs.record_feedback(create_test_feedback(
            "review-agent",
            0.6,
            false,
            FeedbackRating::Negative,
        ));

        let metrics = MetricsAggregator::aggregate(&prefs);

        assert_eq!(metrics.total_routings, 3);
        assert!(metrics.accuracy > 0.5); // 2 positive + 2 successful out of 6 total
        assert_eq!(metrics.agent_performance.len(), 2);

        // Check code-agent metrics
        let code_agent = metrics.agent_performance.get("code-agent").unwrap();
        assert_eq!(code_agent.total_selections, 2);
        assert_eq!(code_agent.positive_feedback, 2);
        assert_eq!(code_agent.success_rate, 1.0);
    }

    #[test]
    fn test_confidence_stats() {
        let mut prefs = RoutingPreferences::new(std::path::PathBuf::from("/test"));

        prefs.record_feedback(create_test_feedback(
            "agent-1",
            0.5,
            true,
            FeedbackRating::Positive,
        ));
        prefs.record_feedback(create_test_feedback(
            "agent-2",
            0.9,
            true,
            FeedbackRating::Positive,
        ));

        let metrics = MetricsAggregator::aggregate(&prefs);

        assert_eq!(metrics.confidence_stats.min, 0.5);
        assert_eq!(metrics.confidence_stats.max, 0.9);
        assert!((metrics.confidence_stats.mean - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_accuracy_drift_detection() {
        let mut metrics = MetricsAggregator::empty_metrics();
        metrics.total_routings = 10;
        metrics.accuracy = 0.75;

        assert!(MetricsAggregator::detect_accuracy_drift(&metrics, 0.8));
        assert!(!MetricsAggregator::detect_accuracy_drift(&metrics, 0.7));
    }

    #[test]
    fn test_summary_generation() {
        let mut prefs = RoutingPreferences::new(std::path::PathBuf::from("/test"));

        prefs.record_feedback(create_test_feedback(
            "code-agent",
            0.8,
            true,
            FeedbackRating::Positive,
        ));

        let metrics = MetricsAggregator::aggregate(&prefs);
        let summary = MetricsAggregator::generate_summary(&metrics);

        assert!(summary.contains("Routing Quality Metrics"));
        assert!(summary.contains("Total Routings: 1"));
        assert!(summary.contains("code-agent"));
    }
}

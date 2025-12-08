//! Progress reporting for task dispatcher.
//!
//! This module provides real-time progress tracking and event broadcasting
//! for task execution monitoring.

use crate::ExecutionTelemetry;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::debug;

/// Progress event types.
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    /// A task has started execution.
    TaskStarted {
        /// Task ID.
        task_id: String,
        /// Agent ID.
        agent_id: String,
    },
    /// A task has completed successfully.
    TaskCompleted {
        /// Task ID.
        task_id: String,
        /// Agent ID.
        agent_id: String,
        /// Telemetry information.
        telemetry: Option<ExecutionTelemetry>,
    },
    /// A task has failed.
    TaskFailed {
        /// Task ID.
        task_id: String,
        /// Agent ID.
        agent_id: String,
        /// Error message.
        error: String,
    },
    /// Queue depth has changed.
    QueueDepthChanged {
        /// New queue depth.
        depth: usize,
    },
}

/// Progress metrics snapshot.
#[derive(Debug, Clone)]
pub struct ProgressMetrics {
    /// Current queue depth (pending tasks).
    pub queue_depth: usize,
    /// Number of active tasks.
    pub active_tasks: usize,
    /// Number of completed tasks.
    pub completed_tasks: usize,
    /// Number of failed tasks.
    pub failed_tasks: usize,
    /// Total cost in USD.
    pub total_cost: f64,
    /// Total tokens used.
    pub total_tokens: u64,
    /// Agent utilization map (agent_id -> utilization 0.0-1.0).
    pub agent_utilization: HashMap<String, f32>,
}

impl Default for ProgressMetrics {
    fn default() -> Self {
        Self {
            queue_depth: 0,
            active_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            total_cost: 0.0,
            total_tokens: 0,
            agent_utilization: HashMap::new(),
        }
    }
}

/// Progress reporter for task dispatcher.
pub struct ProgressReporter {
    /// Broadcast sender for progress events.
    broadcast_tx: broadcast::Sender<ProgressEvent>,
    /// Current metrics.
    metrics: Arc<Mutex<ProgressMetrics>>,
}

impl ProgressReporter {
    /// Creates a new progress reporter.
    #[must_use]
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        Self {
            broadcast_tx,
            metrics: Arc::new(Mutex::new(ProgressMetrics::default())),
        }
    }

    /// Subscribes to progress events.
    ///
    /// # Returns
    /// Returns a receiver for progress events.
    pub fn subscribe(&self) -> broadcast::Receiver<ProgressEvent> {
        self.broadcast_tx.subscribe()
    }

    /// Gets the current progress metrics snapshot.
    ///
    /// # Returns
    /// Returns a copy of the current metrics.
    pub async fn get_snapshot(&self) -> ProgressMetrics {
        self.metrics.lock().await.clone()
    }

    /// Emits a task started event.
    ///
    /// # Arguments
    /// * `task_id` - Task ID
    /// * `agent_id` - Agent ID
    pub fn emit_task_started(&self, task_id: String, agent_id: String) {
        let event = ProgressEvent::TaskStarted { task_id, agent_id };
        let _ = self.broadcast_tx.send(event.clone());
        debug!("Progress event: {:?}", event);
    }

    /// Emits a task completed event.
    ///
    /// # Arguments
    /// * `task_id` - Task ID
    /// * `agent_id` - Agent ID
    /// * `telemetry` - Optional telemetry information
    pub async fn emit_task_completed(
        &self,
        task_id: String,
        agent_id: String,
        telemetry: Option<ExecutionTelemetry>,
    ) {
        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.completed_tasks += 1;
            if metrics.active_tasks > 0 {
                metrics.active_tasks -= 1;
            }

            // Update cost and tokens from telemetry
            if let Some(ref tel) = telemetry {
                metrics.total_tokens += tel.total_tokens;
                // Estimate cost: rough calculation (would need model pricing in production)
                // For now, we'll use a placeholder
            }
        }

        let event = ProgressEvent::TaskCompleted {
            task_id,
            agent_id,
            telemetry,
        };
        let _ = self.broadcast_tx.send(event.clone());
        debug!("Progress event: {:?}", event);
    }

    /// Emits a task failed event.
    ///
    /// # Arguments
    /// * `task_id` - Task ID
    /// * `agent_id` - Agent ID
    /// * `error` - Error message
    pub async fn emit_task_failed(&self, task_id: String, agent_id: String, error: String) {
        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.failed_tasks += 1;
            if metrics.active_tasks > 0 {
                metrics.active_tasks -= 1;
            }
        }

        let event = ProgressEvent::TaskFailed {
            task_id,
            agent_id,
            error,
        };
        let _ = self.broadcast_tx.send(event.clone());
        debug!("Progress event: {:?}", event);
    }

    /// Updates queue depth.
    ///
    /// # Arguments
    /// * `depth` - New queue depth
    pub async fn update_queue_depth(&self, depth: usize) {
        {
            let mut metrics = self.metrics.lock().await;
            metrics.queue_depth = depth;
        }

        let event = ProgressEvent::QueueDepthChanged { depth };
        let _ = self.broadcast_tx.send(event.clone());
        debug!("Progress event: {:?}", event);
    }

    /// Updates active task count.
    ///
    /// # Arguments
    /// * `count` - Active task count
    pub async fn update_active_tasks(&self, count: usize) {
        let mut metrics = self.metrics.lock().await;
        metrics.active_tasks = count;
    }

    /// Updates agent utilization.
    ///
    /// # Arguments
    /// * `utilization` - Map of agent ID to utilization (0.0-1.0)
    pub async fn update_agent_utilization(&self, utilization: HashMap<String, f32>) {
        let mut metrics = self.metrics.lock().await;
        metrics.agent_utilization = utilization;
    }
}

impl Default for ProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_progress_reporter_new() {
        let reporter = ProgressReporter::new();
        let snapshot = reporter.get_snapshot().await;
        assert_eq!(snapshot.queue_depth, 0);
        assert_eq!(snapshot.completed_tasks, 0);
        assert_eq!(snapshot.failed_tasks, 0);
    }

    #[tokio::test]
    async fn test_progress_reporter_events() {
        let reporter = ProgressReporter::new();
        let mut rx = reporter.subscribe();

        // Emit task started
        reporter.emit_task_started("task-1".to_string(), "agent-1".to_string());
        let event = rx.recv().await.unwrap();
        assert!(matches!(event, ProgressEvent::TaskStarted { .. }));

        // Emit task completed
        let telemetry = ExecutionTelemetry {
            input_tokens: 100,
            output_tokens: 50,
            total_tokens: 150,
            model_id: Some("test-model".to_string()),
        };
        reporter
            .emit_task_completed("task-1".to_string(), "agent-1".to_string(), Some(telemetry))
            .await;
        let event = rx.recv().await.unwrap();
        assert!(matches!(event, ProgressEvent::TaskCompleted { .. }));

        // Check metrics
        let snapshot = reporter.get_snapshot().await;
        assert_eq!(snapshot.completed_tasks, 1);
        assert_eq!(snapshot.total_tokens, 150);
    }

    #[tokio::test]
    async fn test_progress_reporter_queue_depth() {
        let reporter = ProgressReporter::new();
        let mut rx = reporter.subscribe();

        reporter.update_queue_depth(5).await;
        let event = rx.recv().await.unwrap();
        assert!(matches!(event, ProgressEvent::QueueDepthChanged { depth: 5 }));

        let snapshot = reporter.get_snapshot().await;
        assert_eq!(snapshot.queue_depth, 5);
    }

    #[tokio::test]
    async fn test_progress_reporter_task_failed() {
        let reporter = ProgressReporter::new();
        let mut rx = reporter.subscribe();

        reporter
            .emit_task_failed("task-1".to_string(), "agent-1".to_string(), "Test error".to_string())
            .await;
        let event = rx.recv().await.unwrap();
        assert!(matches!(event, ProgressEvent::TaskFailed { .. }));

        let snapshot = reporter.get_snapshot().await;
        assert_eq!(snapshot.failed_tasks, 1);
    }
}


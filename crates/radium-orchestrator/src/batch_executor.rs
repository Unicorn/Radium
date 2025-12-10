//! Batch executor for orchestrator integration.

use radium_core::batch::{BatchProcessor, BatchResult, RetryPolicy};
use radium_core::monitoring::MonitoringService;
use crate::progress::{ProgressEvent, ProgressReporter};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

/// Batch executor that integrates with orchestrator and monitoring services.
pub struct BatchExecutor {
    /// Batch processor for parallel execution.
    batch_processor: BatchProcessor<String, String>,
    /// Monitoring service for telemetry tracking.
    monitoring_service: Arc<MonitoringService>,
    /// Progress reporter for event emission.
    progress_reporter: Arc<ProgressReporter>,
}

impl BatchExecutor {
    /// Create a new batch executor.
    pub fn new(
        concurrency: usize,
        timeout: Duration,
        monitoring_service: Arc<MonitoringService>,
        progress_reporter: Arc<ProgressReporter>,
    ) -> Self {
        let retry_policy = RetryPolicy::default();
        let batch_processor = BatchProcessor::new(concurrency, timeout, retry_policy);

        Self {
            batch_processor,
            monitoring_service,
            progress_reporter,
        }
    }

    /// Execute a batch of agent operations.
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID to execute
    /// * `inputs` - Vector of input strings (prompts)
    /// * `processor_fn` - Async function that processes each input
    ///
    /// # Returns
    /// Batch result with aggregated telemetry and execution statistics.
    pub async fn execute_batch<F, Fut>(
        &self,
        agent_id: &str,
        inputs: Vec<String>,
        processor_fn: F,
    ) -> BatchResult<String>
    where
        F: Fn(String) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<String, String>> + Send,
    {
        info!(
            agent_id = %agent_id,
            total_inputs = inputs.len(),
            "Starting batch execution"
        );

        // Emit batch started event
        self.progress_reporter.emit(ProgressEvent::BatchStarted {
            agent_id: agent_id.to_string(),
            total_items: inputs.len(),
        });

        // Create progress callback that emits events
        let progress_reporter = Arc::clone(&self.progress_reporter);
        let agent_id_clone = agent_id.to_string();
        let progress_callback: Arc<dyn Fn(usize, usize, usize, usize, usize) + Send + Sync> =
            Arc::new(move |index, completed, active, successful, failed| {
                // Emit task completed event for each item
                progress_reporter.emit(ProgressEvent::TaskCompleted {
                    task_id: format!("batch-{}-{}", agent_id_clone, index),
                    agent_id: agent_id_clone.clone(),
                    telemetry: None, // Would include telemetry in real implementation
                });
            });

        // Execute batch
        let result = self
            .batch_processor
            .process_batch(inputs, processor_fn, Some(progress_callback))
            .await;

        // Aggregate telemetry (simplified - in real implementation would track per-request)
        let total_tokens = 0; // Would be aggregated from individual requests
        let total_cost = 0.0; // Would be aggregated from individual requests

        info!(
            agent_id = %agent_id,
            successful = result.successful.len(),
            failed = result.failed.len(),
            success_rate = result.success_rate,
            "Batch execution completed"
        );

        // Emit batch completed event
        self.progress_reporter.emit(ProgressEvent::BatchCompleted {
            agent_id: agent_id.to_string(),
            total_items: result.total_items(),
            successful: result.successful.len(),
            failed: result.failed.len(),
            total_tokens,
            total_cost,
        });

        result
    }
}


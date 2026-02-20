//! Batch executor for orchestrator integration.

use radium_abstraction::batch::{BatchProcessor, BatchResult, RetryPolicy};
use crate::progress::ProgressReporter;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

/// Batch executor that integrates with orchestrator progress reporting.
pub struct BatchExecutor {
    /// Batch processor for parallel execution.
    batch_processor: BatchProcessor<String, String>,
    /// Progress reporter for event emission.
    progress_reporter: Arc<ProgressReporter>,
}

impl BatchExecutor {
    /// Create a new batch executor.
    pub fn new(
        concurrency: usize,
        timeout: Duration,
        progress_reporter: Arc<ProgressReporter>,
    ) -> Self {
        let retry_policy = RetryPolicy::default();
        let batch_processor = BatchProcessor::new(concurrency, timeout, retry_policy);

        Self {
            batch_processor,
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
        self.progress_reporter.emit_batch_started(
            agent_id.to_string(),
            inputs.len(),
        );

        // Create progress callback that emits events
        let progress_reporter = Arc::clone(&self.progress_reporter);
        let agent_id_clone = agent_id.to_string();
        let progress_callback: Arc<dyn Fn(usize, usize, usize, usize, usize) + Send + Sync> =
            Arc::new(move |index, _completed, _active, _successful, _failed| {
                // Emit task started event for each item (synchronous version)
                let task_id = format!("batch-{}-{}", agent_id_clone, index);
                progress_reporter.emit_task_started(task_id, agent_id_clone.clone());
                // Note: We can't emit task completed here since the callback runs during processing
                // Task completion would need to be tracked differently in a real implementation
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
        self.progress_reporter.emit_batch_completed(
            agent_id.to_string(),
            result.total_items(),
            result.successful.len(),
            result.failed.len(),
            total_tokens,
            total_cost,
        ).await;

        result
    }
}


//! Core batch processor for parallel execution.

use crate::batch::error::BatchError;
use crate::batch::types::{BatchResult, RetryPolicy};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::timeout;
use tracing::{debug, error};

/// Generic batch processor for parallel execution of async operations.
///
/// Processes items concurrently with configurable concurrency limits,
/// timeout handling, and retry logic.
pub struct BatchProcessor<T, R> {
    /// Maximum number of concurrent operations.
    concurrency: usize,
    /// Timeout per request.
    timeout: Duration,
    /// Retry policy.
    retry_policy: RetryPolicy,
    /// Semaphore for concurrency control.
    semaphore: Arc<Semaphore>,
    /// Phantom data to hold type parameters.
    _phantom: PhantomData<(T, R)>,
}

impl<T, R> BatchProcessor<T, R>
where
    T: Send + Sync + Clone + Debug + 'static,
    R: Send + Clone + 'static,
{
    /// Create a new batch processor.
    ///
    /// # Arguments
    /// * `concurrency` - Maximum number of concurrent operations
    /// * `timeout` - Timeout per request
    /// * `retry_policy` - Retry policy for failed requests
    pub fn new(concurrency: usize, timeout: Duration, retry_policy: RetryPolicy) -> Self {
        Self {
            concurrency,
            timeout,
            retry_policy,
            semaphore: Arc::new(Semaphore::new(concurrency)),
            _phantom: PhantomData,
        }
    }

    /// Process a batch of items concurrently.
    ///
    /// # Arguments
    /// * `items` - Items to process
    /// * `processor` - Async function that processes each item
    /// * `progress_callback` - Optional callback for progress updates (index, completed, active, successful, failed)
    ///
    /// # Returns
    /// Batch result with successful and failed items, timing stats, and success rate.
    pub async fn process_batch<F, Fut>(
        &self,
        items: Vec<T>,
        processor: F,
        progress_callback: Option<crate::batch::types::ProgressCallback>,
    ) -> BatchResult<R>
    where
        F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<R, String>> + Send,
    {
        let start_time = Instant::now();
        let total = items.len();

        if total == 0 {
            return BatchResult::new(vec![], vec![], start_time.elapsed());
        }

        debug!(
            total_items = total,
            concurrency = self.concurrency,
            timeout_secs = self.timeout.as_secs(),
            "Starting batch processing"
        );

        // Track state
        let successful = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let failed = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let completed = Arc::new(tokio::sync::Mutex::new(0usize));
        let active = Arc::new(tokio::sync::Mutex::new(0usize));
        let successful_count = Arc::new(tokio::sync::Mutex::new(0usize));
        let failed_count = Arc::new(tokio::sync::Mutex::new(0usize));

        // Spawn tasks for each item
        let mut handles = Vec::new();

        for (index, item) in items.into_iter().enumerate() {
            let processor = processor.clone();
            let semaphore = Arc::clone(&self.semaphore);
            let timeout_duration = self.timeout;
            let retry_policy = self.retry_policy.clone();
            let successful = Arc::clone(&successful);
            let failed = Arc::clone(&failed);
            let completed = Arc::clone(&completed);
            let active = Arc::clone(&active);
            let successful_count = Arc::clone(&successful_count);
            let failed_count = Arc::clone(&failed_count);
            let progress_callback = progress_callback.clone();

            let handle = tokio::spawn(async move {
                // Acquire semaphore permit
                let permit = match semaphore.acquire().await {
                    Ok(p) => p,
                    Err(e) => {
                        error!(index = index, "Failed to acquire semaphore: {}", e);
                        let mut failed_vec = failed.lock().await;
                        let input_str = format!("{:?}", item);
                        failed_vec.push(BatchError::ItemError {
                            index,
                            input: input_str,
                            error: format!("Semaphore error: {}", e),
                            error_type: "SemaphoreError".to_string(),
                        });
                        let mut failed_cnt = failed_count.lock().await;
                        *failed_cnt += 1;
                        let mut completed_cnt = completed.lock().await;
                        *completed_cnt += 1;
                        if let Some(cb) = &progress_callback {
                            let active_cnt = active.lock().await;
                            cb(index, *completed_cnt, *active_cnt, *successful_count.lock().await, *failed_cnt);
                        }
                        return;
                    }
                };

                // Update active count
                {
                    let mut active_cnt = active.lock().await;
                    *active_cnt += 1;
                }

                // Process with retries
                let mut last_error: Option<String> = None;
                let mut retry_count = 0;

                loop {
                    // Process with timeout
                    match timeout(timeout_duration, processor(item.clone())).await {
                        Ok(Ok(result)) => {
                            // Success
                            let mut successful_vec = successful.lock().await;
                            successful_vec.push((index, result));
                            let mut successful_cnt = successful_count.lock().await;
                            *successful_cnt += 1;
                            break;
                        }
                        Ok(Err(e)) => {
                            // Error from processor
                            last_error = Some(e);
                        }
                        Err(_) => {
                            // Timeout
                            last_error = Some(format!("Timeout after {:?}", timeout_duration));
                        }
                    }

                    // Check if we should retry
                    if retry_count >= retry_policy.max_retries {
                        // Max retries exceeded, record failure
                        let error_msg = last_error.unwrap_or_else(|| "Unknown error".to_string());
                        let input_str = format!("{:?}", item);
                        let mut failed_vec = failed.lock().await;
                        failed_vec.push(BatchError::ItemError {
                            index,
                            input: input_str,
                            error: error_msg.clone(),
                            error_type: if error_msg.contains("timeout") {
                                "TimeoutError".to_string()
                            } else {
                                "ProcessingError".to_string()
                            },
                        });
                        let mut failed_cnt = failed_count.lock().await;
                        *failed_cnt += 1;
                        break;
                    }

                    // Calculate backoff delay
                    let delay = retry_policy.calculate_delay(retry_count);
                    debug!(
                        index = index,
                        retry_count = retry_count + 1,
                        delay_ms = delay.as_millis(),
                        "Retrying after backoff"
                    );
                    tokio::time::sleep(delay).await;
                    retry_count += 1;
                }

                // Update counts
                {
                    let mut active_cnt = active.lock().await;
                    *active_cnt -= 1;
                }
                {
                    let mut completed_cnt = completed.lock().await;
                    *completed_cnt += 1;
                }

                // Invoke progress callback
                if let Some(cb) = &progress_callback {
                    let active_cnt = active.lock().await;
                    let successful_cnt = successful_count.lock().await;
                    let failed_cnt = failed_count.lock().await;
                    cb(index, *completed.lock().await, *active_cnt, *successful_cnt, *failed_cnt);
                }

                // Release permit
                drop(permit);
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            if let Err(e) = handle.await {
                error!("Task join error: {}", e);
            }
        }

        let total_duration = start_time.elapsed();
        let successful_vec: Vec<(usize, R)> = {
            let guard = successful.lock().await;
            guard.clone()
        };
        let failed_vec: Vec<BatchError> = {
            let guard = failed.lock().await;
            guard.clone()
        };

        debug!(
            total_items = total,
            successful = successful_vec.len(),
            failed = failed_vec.len(),
            duration_secs = total_duration.as_secs(),
            "Batch processing completed"
        );

        BatchResult::new(successful_vec, failed_vec, total_duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    async fn mock_processor(item: usize) -> Result<String, String> {
        // Simulate some work
        tokio::time::sleep(Duration::from_millis(10)).await;
        if item == 5 {
            Err("Simulated error".to_string())
        } else {
            Ok(format!("result-{}", item))
        }
    }

    #[tokio::test]
    async fn test_batch_processing_concurrency() {
        let processor = BatchProcessor::new(
            3,
            Duration::from_secs(10),
            RetryPolicy::default(),
        );

        let items: Vec<usize> = (0..10).collect();
        let result = processor
            .process_batch(items, mock_processor, None)
            .await;

        assert_eq!(result.total_items(), 10);
        assert_eq!(result.successful.len(), 9); // One item fails
        assert_eq!(result.failed.len(), 1);
        assert!(result.success_rate > 80.0);
    }

    #[tokio::test]
    async fn test_batch_processing_timeout() {
        async fn slow_processor(item: usize) -> Result<String, String> {
            if item == 0 {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            Ok(format!("result-{}", item))
        }

        let processor = BatchProcessor::new(
            3,
            Duration::from_millis(100),
            RetryPolicy::default(),
        );

        let items: Vec<usize> = (0..5).collect();
        let result = processor
            .process_batch(items, slow_processor, None)
            .await;

        // At least one should timeout
        assert!(result.failed.len() >= 1);
    }

    #[tokio::test]
    async fn test_batch_processing_retry() {
        let mut attempt = std::sync::Arc::new(tokio::sync::Mutex::new(0));

        async fn retry_processor(item: usize) -> Result<String, String> {
            let mut cnt = attempt.lock().await;
            *cnt += 1;
            if *cnt < 3 {
                Err("Temporary error".to_string())
            } else {
                Ok(format!("result-{}", item))
            }
        }

        let retry_policy = RetryPolicy::new(
            3,
            Duration::from_millis(10),
            Duration::from_secs(1),
            2.0,
        );

        let processor = BatchProcessor::new(
            1,
            Duration::from_secs(10),
            retry_policy,
        );

        let items: Vec<usize> = vec![0];
        let result = processor
            .process_batch(items, retry_processor, None)
            .await;

        assert_eq!(result.successful.len(), 1);
        assert_eq!(result.failed.len(), 0);
    }
}


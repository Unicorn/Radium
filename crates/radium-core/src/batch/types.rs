//! Data types for batch processing.

use crate::batch::error::BatchError;
use std::time::Duration;

/// Result of batch processing.
#[derive(Debug, Clone)]
pub struct BatchResult<R> {
    /// Successfully processed items.
    pub successful: Vec<(usize, R)>,
    /// Failed items with error details.
    pub failed: Vec<BatchError>,
    /// Total duration of batch processing.
    pub total_duration: Duration,
    /// Success rate as a percentage (0.0 to 100.0).
    pub success_rate: f64,
}

impl<R> BatchResult<R> {
    /// Create a new batch result.
    pub fn new(
        successful: Vec<(usize, R)>,
        failed: Vec<BatchError>,
        total_duration: Duration,
    ) -> Self {
        let total = successful.len() + failed.len();
        let success_rate = if total > 0 {
            (successful.len() as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Self {
            successful,
            failed,
            total_duration,
            success_rate,
        }
    }

    /// Get total number of items processed.
    pub fn total_items(&self) -> usize {
        self.successful.len() + self.failed.len()
    }

    /// Check if all items were successful.
    pub fn is_complete_success(&self) -> bool {
        self.failed.is_empty()
    }
}

/// Retry policy for batch processing.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retries.
    pub max_retries: u32,
    /// Initial delay before first retry.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Backoff multiplier (e.g., 2.0 for exponential backoff).
    pub multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 0,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy.
    pub fn new(
        max_retries: u32,
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
    ) -> Self {
        Self {
            max_retries,
            initial_delay,
            max_delay,
            multiplier,
        }
    }

    /// Calculate the delay for a given retry attempt.
    ///
    /// Uses exponential backoff: initial_delay * multiplier^retry_count, capped at max_delay.
    pub fn calculate_delay(&self, retry_count: u32) -> Duration {
        let delay_ms = (self.initial_delay.as_millis() as f64
            * self.multiplier.powi(retry_count as i32))
        .min(self.max_delay.as_millis() as f64) as u64;
        Duration::from_millis(delay_ms)
    }
}

/// Progress callback function type.
pub type ProgressCallback = Box<dyn Fn(usize, usize, usize, usize, usize) + Send + Sync>;


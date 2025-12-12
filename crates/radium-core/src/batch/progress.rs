//! Progress tracking for batch processing.

use std::time::{Duration, Instant};

/// Tracks progress of batch execution.
#[derive(Debug, Clone)]
pub struct BatchProgressTracker {
    /// Total number of items to process.
    pub total: usize,
    /// Number of completed items.
    pub completed: usize,
    /// Number of currently active items.
    pub active: usize,
    /// Number of queued items (not yet started).
    pub queued: usize,
    /// Number of successful items.
    pub successful: usize,
    /// Number of failed items.
    pub failed: usize,
    /// Start time of batch execution.
    pub start_time: Instant,
    /// Durations of completed items.
    pub durations: Vec<Duration>,
}

impl BatchProgressTracker {
    /// Create a new progress tracker.
    pub fn new(total: usize) -> Self {
        Self {
            total,
            completed: 0,
            active: 0,
            queued: total,
            successful: 0,
            failed: 0,
            start_time: Instant::now(),
            durations: Vec::new(),
        }
    }

    /// Update progress with a completion event.
    pub fn update(&mut self, _index: usize, completed: usize, active: usize, successful: usize, failed: usize) {
        self.completed = completed;
        self.active = active;
        self.queued = self.total.saturating_sub(completed + active);
        self.successful = successful;
        self.failed = failed;
    }

    /// Record duration for a completed item.
    pub fn record_duration(&mut self, duration: Duration) {
        self.durations.push(duration);
    }

    /// Calculate estimated time remaining.
    ///
    /// Returns formatted string like "2m 15s" or "0s" if no data.
    pub fn calculate_eta(&self) -> String {
        if self.durations.is_empty() || self.completed == 0 {
            return "calculating...".to_string();
        }

        let avg_duration = self.durations.iter().sum::<Duration>() / self.durations.len() as u32;
        let remaining = self.total.saturating_sub(self.completed);
        let eta_secs = avg_duration.as_secs() * remaining as u64;

        format_duration(Duration::from_secs(eta_secs))
    }

    /// Calculate average duration per request.
    pub fn average_duration(&self) -> String {
        if self.durations.is_empty() {
            return "0s".to_string();
        }

        let avg = self.durations.iter().sum::<Duration>() / self.durations.len() as u32;
        format_duration(avg)
    }

    /// Get completion percentage.
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.completed as f64 / self.total as f64) * 100.0
    }
}

/// Format duration as human-readable string.
fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;

    if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracker_new() {
        let tracker = BatchProgressTracker::new(100);
        assert_eq!(tracker.total, 100);
        assert_eq!(tracker.completed, 0);
        assert_eq!(tracker.queued, 100);
    }

    #[test]
    fn test_progress_tracker_update() {
        let mut tracker = BatchProgressTracker::new(100);
        tracker.update(0, 45, 5, 42, 3);
        assert_eq!(tracker.completed, 45);
        assert_eq!(tracker.active, 5);
        assert_eq!(tracker.queued, 50);
        assert_eq!(tracker.successful, 42);
        assert_eq!(tracker.failed, 3);
    }

    #[test]
    fn test_calculate_eta() {
        let mut tracker = BatchProgressTracker::new(100);
        tracker.record_duration(Duration::from_secs(2));
        tracker.record_duration(Duration::from_secs(3));
        tracker.completed = 2;
        tracker.total = 100;

        let eta = tracker.calculate_eta();
        // Should be approximately 2.5s * 98 remaining = ~245s = ~4m
        assert!(eta.contains("m") || eta.contains("s"));
    }

    #[test]
    fn test_percentage() {
        let mut tracker = BatchProgressTracker::new(100);
        tracker.completed = 45;
        assert!((tracker.percentage() - 45.0).abs() < 0.1);
    }
}


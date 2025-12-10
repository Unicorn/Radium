//! Terminal UI rendering for batch progress.

use crate::batch::error::BatchError;
use crate::batch::progress::BatchProgressTracker;
use std::io::{self, Write};

/// Render progress bar and metrics to terminal.
///
/// Uses ANSI escape codes to update in place.
pub fn render_progress(tracker: &BatchProgressTracker, agent_id: &str) -> io::Result<()> {
    let percentage = tracker.percentage();
    let bar_width = 40;
    let filled = (bar_width as f64 * percentage / 100.0) as usize;
    let empty = bar_width - filled;

    // Clear current line and move to beginning
    print!("\r\x1B[K");

    // Progress bar
    let filled_bar = "━".repeat(filled);
    let empty_bar = "─".repeat(empty);
    print!(
        "Batch Execution: {}\n",
        agent_id
    );
    print!(
        "{}{} {}/{} ({:.1}%)\n",
        filled_bar, empty_bar, tracker.completed, tracker.total, percentage
    );

    // Metrics
    print!(
        "Active: {} | Queued: {} | Success: {} | Failed: {}\n",
        tracker.active, tracker.queued, tracker.successful, tracker.failed
    );

    // Timing
    let eta = tracker.calculate_eta();
    let avg = tracker.average_duration();
    print!("ETA: {} | Avg: {}/request", eta, avg);

    io::stdout().flush()?;
    Ok(())
}

/// Render summary report after batch completion.
pub fn render_summary(
    tracker: &BatchProgressTracker,
    failed_errors: &[BatchError],
    output_dir: Option<&std::path::Path>,
) -> io::Result<()> {
    println!("\n");
    println!("Batch Execution Complete");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Summary stats
    println!("Total Requests: {}", tracker.total);
    println!(
        "Successful: {} ({:.1}%)",
        tracker.successful,
        if tracker.total > 0 {
            (tracker.successful as f64 / tracker.total as f64) * 100.0
        } else {
            0.0
        }
    );
    println!(
        "Failed: {} ({:.1}%)",
        tracker.failed,
        if tracker.total > 0 {
            (tracker.failed as f64 / tracker.total as f64) * 100.0
        } else {
            0.0
        }
    );

    // Timing
    let total_duration = tracker.start_time.elapsed();
    let total_duration_str = format_duration(total_duration);
    println!("Total Duration: {}", total_duration_str);
    println!("Average: {}/request", tracker.average_duration());

    // Failed requests
    if !failed_errors.is_empty() {
        println!("\nFailed Requests:");
        for error in failed_errors {
            if let BatchError::ItemError { index, error, .. } = error {
                println!("  #{}: {}", index, error);
            }
        }
    }

    // Output directory
    if let Some(dir) = output_dir {
        println!("\nResults saved to: {}", dir.display());
    }

    Ok(())
}

/// Format duration as human-readable string.
fn format_duration(duration: std::time::Duration) -> String {
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
    fn test_render_progress() {
        let mut tracker = BatchProgressTracker::new(100);
        tracker.update(0, 45, 5, 42, 3);
        tracker.record_duration(std::time::Duration::from_secs(2));

        // Just verify it doesn't panic
        let _ = render_progress(&tracker, "test-agent");
    }

    #[test]
    fn test_render_summary() {
        let mut tracker = BatchProgressTracker::new(100);
        tracker.update(0, 100, 0, 97, 3);
        tracker.record_duration(std::time::Duration::from_secs(2));

        let errors = vec![
            BatchError::ItemError {
                index: 12,
                input: "test".to_string(),
                error: "Timeout".to_string(),
                error_type: "TimeoutError".to_string(),
            },
        ];

        // Just verify it doesn't panic
        let _ = render_summary(&tracker, &errors, None);
    }
}


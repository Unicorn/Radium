//! Real-time progress reporting with terminal UI.
//!
//! This module provides progress display for requirement execution, showing
//! task status, completion percentage, and streaming agent output.

use crate::context::braingrid_client::BraingridRequirement;
use crate::workflow::execution_state::{ExecutionState, TaskExecutionStatus};
use crate::workflow::parallel_executor::ExecutionReport;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;

/// Progress reporter for terminal display.
pub struct ProgressReporter {
    /// Requirement being executed.
    requirement: BraingridRequirement,
    /// Total number of tasks.
    total_tasks: usize,
    /// Multi-progress bar manager.
    multi_progress: MultiProgress,
    /// Overall progress bar.
    overall_progress: ProgressBar,
    /// Task-specific progress bars.
    task_progress_bars: std::collections::HashMap<String, ProgressBar>,
}

impl ProgressReporter {
    /// Creates a new progress reporter.
    ///
    /// # Arguments
    /// * `requirement` - The requirement being executed
    /// * `total_tasks` - Total number of tasks
    pub fn new(requirement: BraingridRequirement, total_tasks: usize) -> Self {
        let multi_progress = MultiProgress::new();

        // Display requirement header
        println!("\n{}", "=".repeat(80));
        println!("📋 Requirement: {}", requirement.name);
        println!("🆔 ID: {}", requirement.id);
        println!("{}", "=".repeat(80));
        println!();

        // Create overall progress bar
        let overall_progress = multi_progress.add(ProgressBar::new(total_tasks as u64));
        overall_progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} tasks ({percent}%)")
                .unwrap()
                .progress_chars("#>-"),
        );
        overall_progress.set_message("Overall progress");

        Self {
            requirement,
            total_tasks,
            multi_progress,
            overall_progress,
            task_progress_bars: std::collections::HashMap::new(),
        }
    }

    /// Updates the status of a task.
    ///
    /// # Arguments
    /// * `task_id` - The task ID
    /// * `status` - The new task status
    pub fn update_task_status(&mut self, task_id: &str, status: TaskExecutionStatus) {
        let status_icon = match status {
            TaskExecutionStatus::Pending => "⏳",
            TaskExecutionStatus::Running => "▶️",
            TaskExecutionStatus::Completed => "✅",
            TaskExecutionStatus::Failed => "❌",
            TaskExecutionStatus::Blocked => "🚫",
        };

        // Get or create task progress bar
        let task_bar = self.task_progress_bars.entry(task_id.to_string()).or_insert_with(|| {
            let bar = self.multi_progress.add(ProgressBar::new(1));
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("{msg}")
                    .unwrap(),
            );
            bar
        });

        task_bar.set_message(format!("{} Task {}", status_icon, task_id));

        // Update overall progress
        let completed = match status {
            TaskExecutionStatus::Completed => 1,
            _ => 0,
        };
        self.overall_progress.inc(completed);
    }

    /// Sets the current task being executed.
    ///
    /// # Arguments
    /// * `task_id` - The task ID
    /// * `task_title` - The task title
    pub fn set_current_task(&self, task_id: &str, task_title: &str) {
        println!("\n▶️ Executing {}: {}", task_id, task_title);
    }

    /// Streams output from an agent.
    ///
    /// # Arguments
    /// * `task_id` - The task ID
    /// * `line` - A line of output
    pub fn stream_output(&self, task_id: &str, line: &str) {
        println!("  [{}] {}", task_id, line);
    }

    /// Finishes the progress display and shows final summary.
    ///
    /// # Arguments
    /// * `report` - The execution report
    pub fn finish(&self, report: &ExecutionReport) {
        self.overall_progress.finish();

        println!("\n{}", "=".repeat(80));
        println!("📊 Execution Summary");
        println!("{}", "=".repeat(80));
        println!();
        println!("  Total Tasks:    {}", report.total_tasks);
        println!("  ✅ Completed:   {}", report.completed_tasks);
        println!("  ❌ Failed:      {}", report.failed_tasks);
        println!("  🚫 Blocked:     {}", report.blocked_tasks);
        println!("  ⏱️  Duration:    {}s", report.total_execution_time_secs);
        println!();

        if report.success {
            println!("  ✓ All tasks completed successfully!");
        } else {
            println!("  ⚠ Some tasks failed or were blocked");
        }

        println!("{}", "=".repeat(80));
        println!();
    }
}


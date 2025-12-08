//! Completion report generator with git commits, test results, and execution metrics.
//!
//! This module generates comprehensive markdown reports summarizing task execution results.

use crate::context::braingrid_client::BraingridRequirement;
use crate::workflow::execution_state::{ExecutionState, TaskResult};
use crate::workflow::parallel_executor::ExecutionReport;
use chrono::Utc;
use std::path::{Path, PathBuf};

/// Completion report data structure.
#[derive(Debug, Clone)]
pub struct CompletionReport {
    /// Requirement ID.
    pub requirement_id: String,
    /// Requirement title.
    pub requirement_title: String,
    /// Total number of tasks.
    pub total_tasks: usize,
    /// Number of completed tasks.
    pub completed_tasks: usize,
    /// Number of failed tasks.
    pub failed_tasks: usize,
    /// Number of blocked tasks.
    pub blocked_tasks: usize,
    /// Total execution time in seconds.
    pub total_execution_time_secs: u64,
    /// Task summaries.
    pub task_summaries: Vec<TaskSummary>,
    /// Git commits (if available).
    pub git_commits: Vec<CommitInfo>,
    /// Test results summary (if available).
    pub test_results: Option<TestResultsSummary>,
}

/// Task summary for the report.
#[derive(Debug, Clone)]
pub struct TaskSummary {
    /// Task ID.
    pub task_id: String,
    /// Task title.
    pub task_title: String,
    /// Agent ID that executed the task.
    pub agent_id: String,
    /// Execution duration in seconds.
    pub duration_secs: u64,
    /// Task status.
    pub status: String,
    /// Error message if failed.
    pub error_message: Option<String>,
}

/// Git commit information.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Commit hash.
    pub hash: String,
    /// Commit message.
    pub message: String,
    /// Author name.
    pub author: String,
    /// Commit timestamp.
    pub timestamp: chrono::DateTime<Utc>,
    /// Commit URL (if available).
    pub url: Option<String>,
}

/// Test results summary.
#[derive(Debug, Clone)]
pub struct TestResultsSummary {
    /// Total number of tests.
    pub total: usize,
    /// Number of passed tests.
    pub passed: usize,
    /// Number of failed tests.
    pub failed: usize,
    /// Number of skipped tests.
    pub skipped: usize,
    /// Coverage percentage (if available).
    pub coverage_percent: Option<f64>,
}

/// Report generator for creating completion reports.
pub struct ReportGenerator {
    /// Workspace root path.
    workspace_path: PathBuf,
}

impl ReportGenerator {
    /// Creates a new report generator.
    ///
    /// # Arguments
    /// * `workspace_path` - Path to the workspace root
    pub fn new(workspace_path: impl AsRef<Path>) -> Self {
        Self {
            workspace_path: workspace_path.as_ref().to_path_buf(),
        }
    }

    /// Generates a completion report.
    ///
    /// # Arguments
    /// * `requirement` - The requirement that was executed
    /// * `execution_state` - The execution state with task results
    /// * `execution_report` - The execution report with summary statistics
    ///
    /// # Returns
    /// A completion report
    pub fn generate_report(
        &self,
        requirement: &BraingridRequirement,
        execution_state: &ExecutionState,
        execution_report: &ExecutionReport,
    ) -> CompletionReport {
        // Extract task summaries
        let mut task_summaries = Vec::new();
        let completed_tasks = execution_state.completed_tasks();
        let failed_tasks = execution_state.failed_tasks();

        for task_id in completed_tasks.iter().chain(failed_tasks.iter()) {
            if let Some(result) = execution_state.get_result(task_id) {
                let status = if execution_state.is_completed(task_id) {
                    "Completed"
                } else {
                    "Failed"
                };

                task_summaries.push(TaskSummary {
                    task_id: task_id.clone(),
                    task_title: format!("Task {}", task_id), // TODO: Get actual title from requirement
                    agent_id: result.agent_id,
                    duration_secs: result.duration_secs(),
                    status: status.to_string(),
                    error_message: result.error_message,
                });
            }
        }

        // TODO: Extract git commits from workspace
        let git_commits = vec![];

        // TODO: Aggregate test results from task results
        let test_results = None;

        CompletionReport {
            requirement_id: requirement.id.clone(),
            requirement_title: requirement.name.clone(),
            total_tasks: execution_report.total_tasks,
            completed_tasks: execution_report.completed_tasks,
            failed_tasks: execution_report.failed_tasks,
            blocked_tasks: execution_report.blocked_tasks,
            total_execution_time_secs: execution_report.total_execution_time_secs,
            task_summaries,
            git_commits,
            test_results,
        }
    }

    /// Saves the completion report to a markdown file.
    ///
    /// # Arguments
    /// * `report` - The completion report to save
    /// * `req_id` - The requirement ID for the filename
    ///
    /// # Returns
    /// Path to the saved report file
    pub fn save_report(
        &self,
        report: &CompletionReport,
        req_id: &str,
    ) -> Result<PathBuf, std::io::Error> {
        // Create reports directory
        let reports_dir = self.workspace_path.join(".radium").join("reports");
        std::fs::create_dir_all(&reports_dir)?;

        // Generate markdown content
        let markdown = self.format_report_markdown(report);

        // Write to file
        let filename = format!("{}-completion.md", req_id);
        let file_path = reports_dir.join(&filename);
        std::fs::write(&file_path, markdown)?;

        Ok(file_path)
    }

    /// Formats the report as markdown.
    fn format_report_markdown(&self, report: &CompletionReport) -> String {
        let mut md = String::new();

        md.push_str(&format!("# Completion Report: {} - {}\n\n", report.requirement_id, report.requirement_title));
        md.push_str("## Summary\n\n");
        md.push_str(&format!("- **Status**: {}\n", if report.failed_tasks == 0 && report.blocked_tasks == 0 { "COMPLETED" } else { "PARTIAL" }));
        md.push_str(&format!("- **Total Tasks**: {} ({} completed, {} failed, {} blocked)\n", report.total_tasks, report.completed_tasks, report.failed_tasks, report.blocked_tasks));
        md.push_str(&format!("- **Execution Time**: {}s\n\n", report.total_execution_time_secs));

        md.push_str("## Task Results\n\n");
        for summary in &report.task_summaries {
            let status_icon = if summary.status == "Completed" { "✅" } else { "❌" };
            md.push_str(&format!("### {} {}: {}\n", status_icon, summary.task_id, summary.task_title));
            md.push_str(&format!("- **Agent**: {}\n", summary.agent_id));
            md.push_str(&format!("- **Duration**: {}s\n", summary.duration_secs));
            md.push_str(&format!("- **Status**: {}\n", summary.status));
            if let Some(ref error) = summary.error_message {
                md.push_str(&format!("- **Error**: {}\n", error));
            }
            md.push_str("\n");
        }

        if !report.git_commits.is_empty() {
            md.push_str("## Git Commits\n\n");
            for commit in &report.git_commits {
                md.push_str(&format!("- [{}] {}\n", &commit.hash[..8], commit.message));
            }
            md.push_str("\n");
        }

        if let Some(ref test_results) = report.test_results {
            md.push_str("## Test Results\n\n");
            md.push_str(&format!("- **Total Tests**: {}\n", test_results.total));
            md.push_str(&format!("- **Passed**: {}\n", test_results.passed));
            md.push_str(&format!("- **Failed**: {}\n", test_results.failed));
            md.push_str(&format!("- **Skipped**: {}\n", test_results.skipped));
            if let Some(coverage) = test_results.coverage_percent {
                md.push_str(&format!("- **Coverage**: {:.1}%\n", coverage));
            }
            md.push_str("\n");
        }

        md
    }

    /// Displays a summary of the report in the terminal.
    ///
    /// # Arguments
    /// * `report` - The completion report to display
    pub fn display_summary(&self, report: &CompletionReport) {
        println!("\n{}", "=".repeat(80));
        println!("📊 Completion Report Summary");
        println!("{}", "=".repeat(80));
        println!();
        println!("  Requirement: {} - {}", report.requirement_id, report.requirement_title);
        println!("  Total Tasks: {}", report.total_tasks);
        println!("  ✅ Completed: {}", report.completed_tasks);
        println!("  ❌ Failed: {}", report.failed_tasks);
        println!("  🚫 Blocked: {}", report.blocked_tasks);
        println!("  ⏱️  Duration: {}s", report.total_execution_time_secs);
        println!();
    }
}


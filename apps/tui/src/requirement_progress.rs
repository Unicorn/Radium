//! Progress tracking for async requirement execution.

// Re-export from radium-core
pub use radium_core::workflow::RequirementProgress;

/// Active requirement execution state.
pub struct ActiveRequirement {
    /// Requirement ID being executed.
    pub req_id: String,

    /// Progress receiver.
    pub progress_rx: tokio::sync::mpsc::Receiver<RequirementProgress>,

    /// Current status message for display.
    pub status: String,

    /// Number of tasks completed.
    pub tasks_completed: usize,

    /// Number of tasks failed.
    pub tasks_failed: usize,

    /// Total number of tasks.
    pub total_tasks: usize,

    /// Current task being executed.
    pub current_task: Option<String>,
}

impl ActiveRequirement {
    /// Creates a new active requirement tracker.
    pub fn new(req_id: String, progress_rx: tokio::sync::mpsc::Receiver<RequirementProgress>) -> Self {
        Self {
            req_id,
            progress_rx,
            status: "⠋ Initializing...".to_string(),
            tasks_completed: 0,
            tasks_failed: 0,
            total_tasks: 0,
            current_task: None,
        }
    }

    /// Updates state based on progress message.
    pub fn update(&mut self, progress: RequirementProgress) {
        match progress {
            RequirementProgress::Started { total_tasks, .. } => {
                self.total_tasks = total_tasks;
                self.status = format!("⠋ Starting execution ({} tasks)...", total_tasks);
            }
            RequirementProgress::TaskStarted { task_title, task_number, total_tasks, .. } => {
                self.current_task = Some(task_title.clone());
                self.status = format!("⠋ Executing task {}/{}: {}", task_number, total_tasks, task_title);
            }
            RequirementProgress::TaskCompleted { task_title, .. } => {
                self.tasks_completed += 1;
                self.status = format!("● Completed: {}", task_title);
            }
            RequirementProgress::TaskFailed { task_title, error, .. } => {
                self.tasks_failed += 1;
                self.status = format!("✗ Failed: {} ({})", task_title, error);
            }
            RequirementProgress::Completed { .. } => {
                self.status = format!("✓ Completed ({} tasks)", self.tasks_completed);
                self.current_task = None;
            }
            RequirementProgress::Failed { error } => {
                self.status = format!("✗ Failed: {}", error);
                self.current_task = None;
            }
        }
    }

    /// Returns a progress percentage (0-100).
    pub fn progress_percentage(&self) -> u8 {
        if self.total_tasks == 0 {
            0
        } else {
            ((self.tasks_completed + self.tasks_failed) as f32 / self.total_tasks as f32 * 100.0) as u8
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radium_core::workflow::RequirementExecutionResult;
    use radium_core::context::braingrid_client::RequirementStatus;

    #[test]
    fn test_active_requirement_initialization() {
        let (_, progress_rx) = tokio::sync::mpsc::channel(10);
        let active_req = ActiveRequirement::new("REQ-178".to_string(), progress_rx);

        assert_eq!(active_req.req_id, "REQ-178");
        assert_eq!(active_req.tasks_completed, 0);
        assert_eq!(active_req.tasks_failed, 0);
        assert_eq!(active_req.total_tasks, 0);
        assert!(active_req.current_task.is_none());
        assert_eq!(active_req.status, "⠋ Initializing...");
    }

    #[test]
    fn test_active_requirement_started_update() {
        let (_, progress_rx) = tokio::sync::mpsc::channel(10);
        let mut active_req = ActiveRequirement::new("REQ-178".to_string(), progress_rx);

        let progress = RequirementProgress::Started {
            req_id: "REQ-178".to_string(),
            total_tasks: 5,
        };

        active_req.update(progress);

        assert_eq!(active_req.total_tasks, 5);
        assert_eq!(active_req.status, "⠋ Starting execution (5 tasks)...");
    }

    #[test]
    fn test_active_requirement_task_completed_update() {
        let (_, progress_rx) = tokio::sync::mpsc::channel(10);
        let mut active_req = ActiveRequirement::new("REQ-178".to_string(), progress_rx);

        active_req.update(RequirementProgress::Started {
            req_id: "REQ-178".to_string(),
            total_tasks: 3,
        });

        let progress = RequirementProgress::TaskCompleted {
            task_id: "TASK-1".to_string(),
            task_title: "Implement feature".to_string(),
        };

        active_req.update(progress);

        assert_eq!(active_req.tasks_completed, 1);
        assert_eq!(active_req.status, "● Completed: Implement feature");
    }

    #[test]
    fn test_progress_percentage() {
        let (_, progress_rx) = tokio::sync::mpsc::channel(10);
        let mut active_req = ActiveRequirement::new("REQ-178".to_string(), progress_rx);

        active_req.update(RequirementProgress::Started {
            req_id: "REQ-178".to_string(),
            total_tasks: 10,
        });

        // Complete 3 tasks
        for i in 1..=3 {
            active_req.update(RequirementProgress::TaskCompleted {
                task_id: format!("TASK-{}", i),
                task_title: format!("Task {}", i),
            });
        }

        assert_eq!(active_req.progress_percentage(), 30);
    }

    #[tokio::test]
    async fn test_progress_channel_communication() {
        let (progress_tx, progress_rx) = tokio::sync::mpsc::channel(10);
        let mut active_req = ActiveRequirement::new("REQ-178".to_string(), progress_rx);

        // Spawn a task to send progress updates
        tokio::spawn(async move {
            let _ = progress_tx.send(RequirementProgress::Started {
                req_id: "REQ-178".to_string(),
                total_tasks: 2,
            }).await;

            let _ = progress_tx.send(RequirementProgress::TaskCompleted {
                task_id: "TASK-1".to_string(),
                task_title: "Test task".to_string(),
            }).await;
        });

        // Receive updates
        let mut updates_received = 0;
        while let Some(progress) = active_req.progress_rx.recv().await {
            active_req.update(progress);
            updates_received += 1;

            if updates_received == 2 {
                break;
            }
        }

        assert_eq!(updates_received, 2);
        assert_eq!(active_req.total_tasks, 2);
        assert_eq!(active_req.tasks_completed, 1);
    }
}

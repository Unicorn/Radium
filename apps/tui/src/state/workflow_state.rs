//! Workflow execution state tracking.

use super::{AgentState, TelemetryState, CheckpointState, OutputBuffer};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Overall workflow status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowStatus {
    /// Workflow is idle
    Idle,
    /// Workflow is running
    Running,
    /// Workflow is paused (waiting for user input)
    Paused,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed
    Failed,
    /// Workflow was cancelled
    Cancelled,
}

impl WorkflowStatus {
    /// Returns a display string for the status.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Running => "Running",
            Self::Paused => "Paused",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }

    /// Returns whether the workflow is in an active state.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Paused)
    }

    /// Returns whether the workflow is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

/// Workflow UI state tracking
#[derive(Debug, Clone)]
pub struct WorkflowUIState {
    /// Workflow ID
    pub workflow_id: String,
    /// Workflow name
    pub workflow_name: String,
    /// Current workflow status
    pub status: WorkflowStatus,
    /// Start time
    pub start_time: Option<Instant>,
    /// End time
    pub end_time: Option<Instant>,
    /// Current step index
    pub current_step: usize,
    /// Total number of steps
    pub total_steps: usize,
    /// Agent states indexed by agent ID
    pub agents: HashMap<String, AgentState>,
    /// Main workflow output buffer
    pub output_buffer: OutputBuffer,
    /// Telemetry tracking
    pub telemetry: TelemetryState,
    /// Checkpoint tracking
    pub checkpoint: CheckpointState,
    /// Error message if failed
    pub error_message: Option<String>,
}

impl WorkflowUIState {
    /// Creates a new workflow UI state.
    pub fn new(workflow_id: String, workflow_name: String, total_steps: usize) -> Self {
        Self {
            workflow_id,
            workflow_name,
            status: WorkflowStatus::Idle,
            start_time: None,
            end_time: None,
            current_step: 0,
            total_steps,
            agents: HashMap::new(),
            output_buffer: OutputBuffer::new(1000),
            telemetry: TelemetryState::new(),
            checkpoint: CheckpointState::new(),
            error_message: None,
        }
    }

    /// Starts the workflow.
    pub fn start(&mut self) {
        self.status = WorkflowStatus::Running;
        self.start_time = Some(Instant::now());
        self.output_buffer.append_line(format!(
            "[{}] Workflow started: {}",
            chrono::Utc::now().format("%H:%M:%S"),
            self.workflow_name
        ));
    }

    /// Pauses the workflow.
    pub fn pause(&mut self) {
        if self.status == WorkflowStatus::Running {
            self.status = WorkflowStatus::Paused;
            self.output_buffer.append_line(format!(
                "[{}] Workflow paused",
                chrono::Utc::now().format("%H:%M:%S")
            ));
        }
    }

    /// Resumes the workflow.
    pub fn resume(&mut self) {
        if self.status == WorkflowStatus::Paused {
            self.status = WorkflowStatus::Running;
            self.output_buffer.append_line(format!(
                "[{}] Workflow resumed",
                chrono::Utc::now().format("%H:%M:%S")
            ));
        }
    }

    /// Completes the workflow.
    pub fn complete(&mut self) {
        self.status = WorkflowStatus::Completed;
        self.end_time = Some(Instant::now());
        self.output_buffer.append_line(format!(
            "[{}] Workflow completed successfully",
            chrono::Utc::now().format("%H:%M:%S")
        ));
    }

    /// Fails the workflow with an error message.
    pub fn fail(&mut self, error: String) {
        self.status = WorkflowStatus::Failed;
        self.end_time = Some(Instant::now());
        self.error_message = Some(error.clone());
        self.output_buffer.append_line(format!(
            "[{}] Workflow failed: {}",
            chrono::Utc::now().format("%H:%M:%S"),
            error
        ));
    }

    /// Cancels the workflow.
    pub fn cancel(&mut self) {
        self.status = WorkflowStatus::Cancelled;
        self.end_time = Some(Instant::now());
        self.output_buffer.append_line(format!(
            "[{}] Workflow cancelled",
            chrono::Utc::now().format("%H:%M:%S")
        ));
    }

    /// Advances to the next step.
    pub fn next_step(&mut self) {
        if self.current_step < self.total_steps {
            self.current_step += 1;
            self.output_buffer.append_line(format!(
                "[{}] Step {}/{} started",
                chrono::Utc::now().format("%H:%M:%S"),
                self.current_step,
                self.total_steps
            ));
        }
    }

    /// Registers a new agent.
    pub fn register_agent(&mut self, agent_id: String, agent_name: String) {
        let agent_state = AgentState::new(agent_id.clone(), agent_name);
        self.agents.insert(agent_id, agent_state);
    }

    /// Gets a mutable reference to an agent state.
    pub fn get_agent_mut(&mut self, agent_id: &str) -> Option<&mut AgentState> {
        self.agents.get_mut(agent_id)
    }

    /// Gets an immutable reference to an agent state.
    pub fn get_agent(&self, agent_id: &str) -> Option<&AgentState> {
        self.agents.get(agent_id)
    }

    /// Returns the elapsed time since workflow start.
    pub fn elapsed_time(&self) -> Option<Duration> {
        self.start_time.map(|start| {
            if let Some(end) = self.end_time {
                end.duration_since(start)
            } else {
                Instant::now().duration_since(start)
            }
        })
    }

    /// Returns the progress percentage (0-100).
    pub fn progress_percentage(&self) -> u8 {
        if self.total_steps == 0 {
            return 0;
        }
        ((self.current_step as f64 / self.total_steps as f64) * 100.0) as u8
    }

    /// Appends a line to the workflow output buffer.
    pub fn log(&mut self, line: String) {
        self.output_buffer.append_line(line);
    }

    /// Returns a summary of the workflow state.
    pub fn summary(&self) -> String {
        format!(
            "Workflow: {} | Status: {} | Step: {}/{} | Progress: {}%",
            self.workflow_name,
            self.status.as_str(),
            self.current_step,
            self.total_steps,
            self.progress_percentage()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_status() {
        assert_eq!(WorkflowStatus::Running.as_str(), "Running");
        assert!(WorkflowStatus::Running.is_active());
        assert!(!WorkflowStatus::Completed.is_active());
        assert!(WorkflowStatus::Completed.is_terminal());
        assert!(!WorkflowStatus::Running.is_terminal());
    }

    #[test]
    fn test_workflow_lifecycle() {
        let mut state = WorkflowUIState::new(
            "wf-1".to_string(),
            "Test Workflow".to_string(),
            3
        );

        assert_eq!(state.status, WorkflowStatus::Idle);

        state.start();
        assert_eq!(state.status, WorkflowStatus::Running);
        assert!(state.start_time.is_some());

        state.pause();
        assert_eq!(state.status, WorkflowStatus::Paused);

        state.resume();
        assert_eq!(state.status, WorkflowStatus::Running);

        state.complete();
        assert_eq!(state.status, WorkflowStatus::Completed);
        assert!(state.end_time.is_some());
    }

    #[test]
    fn test_workflow_progress() {
        let mut state = WorkflowUIState::new(
            "wf-1".to_string(),
            "Test Workflow".to_string(),
            4
        );

        assert_eq!(state.progress_percentage(), 0);

        state.next_step();
        assert_eq!(state.current_step, 1);
        assert_eq!(state.progress_percentage(), 25);

        state.next_step();
        assert_eq!(state.current_step, 2);
        assert_eq!(state.progress_percentage(), 50);

        state.next_step();
        state.next_step();
        assert_eq!(state.current_step, 4);
        assert_eq!(state.progress_percentage(), 100);
    }

    #[test]
    fn test_agent_registration() {
        let mut state = WorkflowUIState::new(
            "wf-1".to_string(),
            "Test Workflow".to_string(),
            1
        );

        state.register_agent("agent-1".to_string(), "Test Agent".to_string());
        assert!(state.get_agent("agent-1").is_some());
        assert_eq!(state.get_agent("agent-1").unwrap().agent_name, "Test Agent");
    }

    #[test]
    fn test_workflow_failure() {
        let mut state = WorkflowUIState::new(
            "wf-1".to_string(),
            "Test Workflow".to_string(),
            2
        );

        state.start();
        state.fail("Test error".to_string());

        assert_eq!(state.status, WorkflowStatus::Failed);
        assert_eq!(state.error_message, Some("Test error".to_string()));
        assert!(state.end_time.is_some());
    }
}

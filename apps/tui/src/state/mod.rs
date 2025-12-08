//! TUI state management for workflow execution tracking.
//!
//! This module provides state management for the enhanced TUI,
//! tracking workflow execution, agent states, telemetry, and checkpoints.

mod agent_state;
mod checkpoint_state;
mod checkpoint_interrupt_state;
mod command_suggestions;
mod execution_history;
mod telemetry_state;
mod task_list_state;
mod workflow_state;

pub use agent_state::{AgentState, AgentStatus, SubAgentState};
pub use checkpoint_state::{CheckpointInfo, CheckpointState};
pub use checkpoint_interrupt_state::{CheckpointInterruptState, InterruptAction, InterruptTrigger};
pub use command_suggestions::{CommandSuggestion, CommandSuggestionState, SuggestionSource, SuggestionType, TriggerMode};
pub use execution_history::{AggregateStats, ExecutionHistory, ExecutionRecord, ExecutionStatus};
pub use telemetry_state::{TelemetryState, TokenMetrics};
pub use task_list_state::{TaskListState, TaskListItem};
pub use workflow_state::{WorkflowStatus, WorkflowUIState};

/// Output buffer for agent execution
#[derive(Debug, Clone)]
pub struct OutputBuffer {
    /// Lines of output
    pub lines: Vec<String>,
    /// Maximum number of lines to keep
    pub max_lines: usize,
    /// Current scroll position
    pub scroll_position: usize,
}

impl OutputBuffer {
    /// Creates a new output buffer with the specified capacity.
    pub fn new(max_lines: usize) -> Self {
        Self { lines: Vec::new(), max_lines, scroll_position: 0 }
    }

    /// Appends a line to the buffer, removing oldest if at capacity.
    /// Trailing whitespace is trimmed to prevent weird spacing in display.
    pub fn append_line(&mut self, line: String) {
        let trimmed = line.trim_end().to_string();
        self.lines.push(trimmed);
        if self.lines.len() > self.max_lines {
            self.lines.remove(0);
        }
        // Auto-scroll to bottom on new content
        self.scroll_position = self.lines.len().saturating_sub(1);
    }

    /// Appends multiple lines to the buffer.
    pub fn append_lines(&mut self, lines: Vec<String>) {
        for line in lines {
            self.append_line(line);
        }
    }

    /// Clears the buffer.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.scroll_position = 0;
    }

    /// Scrolls up by the specified number of lines.
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_position = self.scroll_position.saturating_sub(amount);
    }

    /// Scrolls down by the specified number of lines.
    pub fn scroll_down(&mut self, amount: usize) {
        let max_scroll = self.lines.len().saturating_sub(1);
        self.scroll_position = (self.scroll_position + amount).min(max_scroll);
    }

    /// Scrolls to the top.
    pub fn scroll_to_top(&mut self) {
        self.scroll_position = 0;
    }

    /// Scrolls to the bottom.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_position = self.lines.len().saturating_sub(1);
    }

    /// Returns the visible lines for the current scroll position.
    pub fn visible_lines(&self, viewport_height: usize) -> Vec<String> {
        let start = self.scroll_position;
        let end = (start + viewport_height).min(self.lines.len());
        self.lines[start..end].to_vec()
    }

    /// Returns whether we're scrolled to the bottom.
    pub fn is_at_bottom(&self) -> bool {
        self.scroll_position == self.lines.len().saturating_sub(1)
    }
}

#[cfg(test)]
mod command_suggestions_test;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_buffer_append() {
        let mut buffer = OutputBuffer::new(5);
        buffer.append_line("Line 1".to_string());
        buffer.append_line("Line 2".to_string());

        assert_eq!(buffer.lines.len(), 2);
        assert_eq!(buffer.lines[0], "Line 1");
        assert_eq!(buffer.lines[1], "Line 2");
    }

    #[test]
    fn test_output_buffer_capacity() {
        let mut buffer = OutputBuffer::new(3);
        buffer.append_line("Line 1".to_string());
        buffer.append_line("Line 2".to_string());
        buffer.append_line("Line 3".to_string());
        buffer.append_line("Line 4".to_string());

        assert_eq!(buffer.lines.len(), 3);
        assert_eq!(buffer.lines[0], "Line 2");
        assert_eq!(buffer.lines[2], "Line 4");
    }

    #[test]
    fn test_output_buffer_scroll() {
        let mut buffer = OutputBuffer::new(10);
        for i in 1..=10 {
            buffer.append_line(format!("Line {}", i));
        }

        buffer.scroll_to_top();
        assert_eq!(buffer.scroll_position, 0);

        buffer.scroll_down(5);
        assert_eq!(buffer.scroll_position, 5);

        buffer.scroll_up(2);
        assert_eq!(buffer.scroll_position, 3);

        buffer.scroll_to_bottom();
        assert_eq!(buffer.scroll_position, 9);
    }

    #[test]
    fn test_output_buffer_visible_lines() {
        let mut buffer = OutputBuffer::new(10);
        for i in 1..=5 {
            buffer.append_line(format!("Line {}", i));
        }

        buffer.scroll_to_top();
        let visible = buffer.visible_lines(3);
        assert_eq!(visible.len(), 3);
        assert_eq!(visible[0], "Line 1");
        assert_eq!(visible[2], "Line 3");
    }

    #[test]
    fn test_output_buffer_clear() {
        let mut buffer = OutputBuffer::new(5);
        buffer.append_line("Line 1".to_string());
        buffer.append_line("Line 2".to_string());

        buffer.clear();
        assert_eq!(buffer.lines.len(), 0);
        assert_eq!(buffer.scroll_position, 0);
    }
}

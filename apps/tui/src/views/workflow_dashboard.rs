//! Workflow dashboard view - enhanced TUI for workflow execution tracking.

use crate::components::{AgentTimeline, LoopIndicator, OutputWindow, StatusFooter, TelemetryBar};
use crate::state::{WorkflowStatus, WorkflowUIState};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Workflow dashboard view
pub struct WorkflowDashboard;

impl WorkflowDashboard {
    /// Renders the complete workflow dashboard.
    pub fn render(frame: &mut Frame, workflow_state: &WorkflowUIState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),      // Title
                Constraint::Length(7),      // Telemetry
                Constraint::Percentage(40), // Agent timeline
                Constraint::Percentage(40), // Output window
                Constraint::Length(4),      // Status footer
            ])
            .split(frame.area());

        // Title
        Self::render_title(frame, chunks[0], workflow_state);

        // Telemetry bar with progress
        TelemetryBar::render_extended(
            frame,
            chunks[1],
            &workflow_state.telemetry,
            workflow_state.progress_percentage(),
        );

        // Agent timeline
        let agents: Vec<_> = workflow_state.agents.values().cloned().collect();
        AgentTimeline::render(frame, chunks[2], &agents, None);

        // Output window
        OutputWindow::render(frame, chunks[3], &workflow_state.output_buffer, "Workflow Output");

        // Status footer
        let elapsed = workflow_state.elapsed_time().map(|d| d.as_secs_f64()).unwrap_or(0.0);

        StatusFooter::render_extended(
            frame,
            chunks[4],
            workflow_state.status,
            "[q] Quit | [p] Pause | [r] Resume | [c] Cancel | [l] Logs",
            elapsed,
            workflow_state.current_step,
            workflow_state.total_steps,
        );
    }

    /// Renders the dashboard title.
    fn render_title(frame: &mut Frame, area: Rect, workflow_state: &WorkflowUIState) {
        let title_text = format!("🔆 Workflow: {}", workflow_state.workflow_name);

        let title = Paragraph::new(title_text)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(title, area);
    }

    /// Renders the dashboard with loop indicator.
    pub fn render_with_loop(frame: &mut Frame, workflow_state: &WorkflowUIState) {
        // Check if there's an active loop
        let has_loop = workflow_state.checkpoint.get_current_loop_iteration().is_some();

        if has_loop {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),      // Title
                    Constraint::Length(7),      // Telemetry
                    Constraint::Length(8),      // Loop indicator
                    Constraint::Percentage(30), // Agent timeline
                    Constraint::Percentage(30), // Output window
                    Constraint::Length(4),      // Status footer
                ])
                .split(frame.area());

            // Title
            Self::render_title(frame, chunks[0], workflow_state);

            // Telemetry bar with progress
            TelemetryBar::render_extended(
                frame,
                chunks[1],
                &workflow_state.telemetry,
                workflow_state.progress_percentage(),
            );

            // Loop indicator
            LoopIndicator::render(frame, chunks[2], &workflow_state.checkpoint);

            // Agent timeline
            let agents: Vec<_> = workflow_state.agents.values().cloned().collect();
            AgentTimeline::render(frame, chunks[3], &agents, None);

            // Output window
            OutputWindow::render(
                frame,
                chunks[4],
                &workflow_state.output_buffer,
                "Workflow Output",
            );

            // Status footer
            let elapsed = workflow_state.elapsed_time().map(|d| d.as_secs_f64()).unwrap_or(0.0);

            StatusFooter::render_extended(
                frame,
                chunks[5],
                workflow_state.status,
                "[q] Quit | [p] Pause | [r] Resume | [c] Cancel",
                elapsed,
                workflow_state.current_step,
                workflow_state.total_steps,
            );
        } else {
            // No loop, render normal dashboard
            Self::render(frame, workflow_state);
        }
    }

    /// Renders a compact dashboard view.
    pub fn render_compact(frame: &mut Frame, workflow_state: &WorkflowUIState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),      // Title
                Constraint::Length(3),      // Telemetry compact
                Constraint::Percentage(50), // Agent timeline
                Constraint::Percentage(50), // Output window
            ])
            .split(frame.area());

        // Title
        Self::render_title(frame, chunks[0], workflow_state);

        // Compact telemetry
        TelemetryBar::render_compact(frame, chunks[1], &workflow_state.telemetry);

        // Agent timeline
        let agents: Vec<_> = workflow_state.agents.values().cloned().collect();
        AgentTimeline::render(frame, chunks[2], &agents, None);

        // Output window
        OutputWindow::render(frame, chunks[3], &workflow_state.output_buffer, "Output");
    }

    /// Renders an error state.
    pub fn render_error(frame: &mut Frame, workflow_state: &WorkflowUIState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),      // Title
                Constraint::Percentage(30), // Error message
                Constraint::Percentage(40), // Agent timeline
                Constraint::Percentage(30), // Output
            ])
            .split(frame.area());

        // Title
        Self::render_title(frame, chunks[0], workflow_state);

        // Error message
        if let Some(ref error) = workflow_state.error_message {
            let error_text = format!("ERROR:\n\n{}", error);
            let error_widget = Paragraph::new(error_text)
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title(" Error "));
            frame.render_widget(error_widget, chunks[1]);
        }

        // Agent timeline
        let agents: Vec<_> = workflow_state.agents.values().cloned().collect();
        AgentTimeline::render(frame, chunks[2], &agents, None);

        // Output window
        OutputWindow::render(frame, chunks[3], &workflow_state.output_buffer, "Output");
    }

    /// Checks if the workflow is in an error state.
    pub fn is_error_state(workflow_state: &WorkflowUIState) -> bool {
        workflow_state.status == WorkflowStatus::Failed && workflow_state.error_message.is_some()
    }

    /// Checks if the workflow has active loops.
    pub fn has_active_loop(workflow_state: &WorkflowUIState) -> bool {
        workflow_state.checkpoint.get_current_loop_iteration().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_error_state() {
        let mut state = WorkflowUIState::new("wf-1".to_string(), "Test".to_string(), 1);
        assert!(!WorkflowDashboard::is_error_state(&state));

        state.fail("Test error".to_string());
        assert!(WorkflowDashboard::is_error_state(&state));
    }

    #[test]
    fn test_has_active_loop() {
        let mut state = WorkflowUIState::new("wf-1".to_string(), "Test".to_string(), 1);
        assert!(!WorkflowDashboard::has_active_loop(&state));

        state.checkpoint.start_loop_iteration(1, 1);
        assert!(WorkflowDashboard::has_active_loop(&state));

        state.checkpoint.complete_loop_iteration(3);
        assert!(!WorkflowDashboard::has_active_loop(&state));
    }
}

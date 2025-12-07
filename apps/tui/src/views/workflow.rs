//! Workflow execution view with split-panel layout.

use crate::components::{AgentTimeline, OutputWindow, StatusFooter, TelemetryBar};
use crate::state::{AgentState, WorkflowStatus, WorkflowUIState};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Renders the workflow execution view with split-panel layout.
pub fn render_workflow(
    frame: &mut Frame,
    area: Rect,
    workflow_state: &WorkflowUIState,
    selected_agent_id: Option<&str>,
) {
    let theme = crate::theme::get_theme();

    // Main layout: Header, Content (split), Telemetry, Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(10),   // Main content (split panel)
            Constraint::Length(3), // Telemetry bar
            Constraint::Length(1), // Status footer
        ])
        .split(area);

    // Header
    render_header(frame, chunks[0], workflow_state, &theme);

    // Split panel: Agent timeline (left) and Output (right)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // Agent timeline
            Constraint::Percentage(65), // Output window
        ])
        .split(chunks[1]);

    // Left panel: Agent timeline
    let agents: Vec<AgentState> = workflow_state.agents.values().cloned().collect();
    let selected_index = selected_agent_id
        .and_then(|id| agents.iter().position(|a| a.agent_id == id));
    AgentTimeline::render(frame, content_chunks[0], &agents, selected_index);

    // Right panel: Output window
    let output_title = selected_agent_id
        .and_then(|id| workflow_state.agents.get(id))
        .map(|agent| agent.agent_name.clone())
        .unwrap_or_else(|| "Workflow Output".to_string());

    let output_buffer = selected_agent_id
        .and_then(|id| workflow_state.agents.get(id))
        .map(|agent| &agent.output_buffer)
        .unwrap_or(&workflow_state.output_buffer);

    OutputWindow::render(frame, content_chunks[1], output_buffer, &output_title);

    // Telemetry bar with runtime and status
    let runtime = workflow_state
        .elapsed_time()
        .map(|d| {
            let secs = d.as_secs();
            format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
        })
        .unwrap_or_else(|| "00:00:00".to_string());
    TelemetryBar::render_with_status(
        frame,
        chunks[2],
        &workflow_state.telemetry,
        Some(&runtime),
        Some(workflow_state.status),
        None, // TODO: Add loop iteration when available
    );

    // Status footer
    let elapsed = workflow_state
        .elapsed_time()
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    StatusFooter::render_compact(frame, chunks[3], workflow_state.status, elapsed);
}

/// Renders the workflow header.
fn render_header(
    frame: &mut Frame,
    area: Rect,
    workflow_state: &WorkflowUIState,
    theme: &crate::theme::RadiumTheme,
) {
    let status_color = match workflow_state.status {
        WorkflowStatus::Running => theme.info,
        WorkflowStatus::Completed => theme.success,
        WorkflowStatus::Failed => theme.error,
        WorkflowStatus::Paused => theme.warning,
        _ => theme.text_muted,
    };

    let elapsed = workflow_state
        .elapsed_time()
        .map(|d| {
            let secs = d.as_secs();
            format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
        })
        .unwrap_or_else(|| "00:00:00".to_string());

    let header_text = format!(
        "{} | {} | Step {}/{} | {}",
        workflow_state.workflow_name,
        workflow_state.status.as_str(),
        workflow_state.current_step,
        workflow_state.total_steps,
        elapsed
    );

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(status_color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_panel)),
        );

    frame.render_widget(header, area);
}


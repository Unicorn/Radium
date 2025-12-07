//! Progress bar component for requirement execution.

use crate::requirement_progress::ActiveRequirement;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
};

/// Renders a progress bar for an active requirement.
pub fn render_requirement_progress(frame: &mut Frame, area: Rect, active_req: &ActiveRequirement) {
    let theme = crate::theme::get_theme();

    // Split area into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with requirement ID
            Constraint::Length(3), // Progress bar
            Constraint::Length(3), // Current task
            Constraint::Length(3), // Task stats
            Constraint::Min(0),    // Spacing
        ])
        .split(area);

    // Header
    let header_text = format!("Requirement: {}", active_req.req_id);
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_panel)),
        );
    frame.render_widget(header, chunks[0]);

    // Progress bar
    let progress = active_req.progress_percentage();
    let progress_label = format!(
        "{}/{} tasks ({}%)",
        active_req.tasks_completed + active_req.tasks_failed,
        active_req.total_tasks,
        progress
    );

    let progress_bar = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_panel))
                .title(" Progress "),
        )
        .gauge_style(Style::default().fg(theme.primary).bg(theme.bg_panel))
        .percent(progress as u16)
        .label(progress_label);
    frame.render_widget(progress_bar, chunks[1]);

    // Current task
    let task_text = if let Some(ref task) = active_req.current_task {
        format!("Current: {}", task)
    } else {
        "Waiting...".to_string()
    };

    let current_task = Paragraph::new(task_text)
        .style(Style::default().fg(theme.text))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_panel))
                .title(" Current Task "),
        );
    frame.render_widget(current_task, chunks[2]);

    // Task stats
    let stats_text = vec![
        Line::from(vec![
            Span::styled(" Completed: ", Style::default().fg(theme.success)),
            Span::styled(
                active_req.tasks_completed.to_string(),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Failed: ", Style::default().fg(theme.error)),
            Span::styled(
                active_req.tasks_failed.to_string(),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let task_stats = Paragraph::new(stats_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_panel))
                .title(" Statistics "),
        );
    frame.render_widget(task_stats, chunks[3]);
}

/// Renders a minimal inline progress indicator (for use in status line).
pub fn render_inline_progress(active_req: &ActiveRequirement) -> String {
    let spinner = get_spinner(active_req.tasks_completed);
    let progress = active_req.progress_percentage();

    if let Some(ref task) = active_req.current_task {
        format!("{} {} ({}%)", spinner, truncate_task_name(task, 30), progress)
    } else {
        format!("{} Waiting... ({}%)", spinner, progress)
    }
}

/// Returns an animated spinner character based on frame count.
fn get_spinner(frame: usize) -> char {
    let spinners = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    spinners[frame % spinners.len()]
}

/// Truncates a task name to a maximum length with ellipsis.
fn truncate_task_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}...", &name[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_spinner() {
        let s1 = get_spinner(0);
        let s2 = get_spinner(1);
        assert_ne!(s1, s2); // Should return different characters
    }

    #[test]
    fn test_truncate_task_name() {
        assert_eq!(truncate_task_name("Short", 10), "Short");
        assert_eq!(truncate_task_name("This is a very long task name", 15), "This is a ve...");
        assert_eq!(truncate_task_name("Exact", 5), "Exact");
    }

    #[test]
    fn test_inline_progress_format() {
        let (_, progress_rx) = tokio::sync::mpsc::channel(10);
        let mut active_req = ActiveRequirement::new("REQ-178".to_string(), progress_rx);

        active_req.total_tasks = 10;
        active_req.tasks_completed = 3;
        active_req.current_task = Some("Test task".to_string());

        let inline = render_inline_progress(&active_req);
        assert!(inline.contains("Test task"));
        assert!(inline.contains("30%")); // 3/10 = 30%
    }
}

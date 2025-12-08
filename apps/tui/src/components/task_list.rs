//! Task list component for displaying tasks with status indicators.

use crate::components::status_icon::render_status_icon;
use crate::state::AgentStatus;
use crate::theme::get_theme;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Represents a single task item.
#[derive(Debug, Clone)]
pub struct TaskItem {
    /// Task name/description
    pub name: String,
    /// Task status
    pub status: AgentStatus,
    /// Elapsed time in seconds (None if not started)
    pub elapsed_seconds: Option<f64>,
}

/// Task list component for displaying tasks with status indicators.
#[derive(Debug, Clone)]
pub struct TaskList {
    /// List of tasks to display
    pub tasks: Vec<TaskItem>,
}

impl TaskList {
    /// Creates a new empty task list.
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Creates a new task list with the given tasks.
    pub fn with_tasks(tasks: Vec<TaskItem>) -> Self {
        Self { tasks }
    }

    /// Renders the task list with status indicators.
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        frame_counter: usize,
        animations_enabled: bool,
        reduced_motion: bool,
    ) {
        let theme = get_theme();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Title
                Constraint::Min(5),    // Task list
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Tasks")
            .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            );
        frame.render_widget(title, chunks[0]);

        // Build task list items
        let items: Vec<ListItem> = self
            .tasks
            .iter()
            .map(|task| {
                let (icon, icon_style) = render_status_icon(
                    task.status,
                    frame_counter,
                    animations_enabled,
                    reduced_motion,
                );
                let status_color = icon_style.fg.unwrap_or(theme.text);

                let elapsed_str = task
                    .elapsed_seconds
                    .map(|s| format!(" - {}", format_elapsed(s)))
                    .unwrap_or_else(String::new);

                let content = format!(
                    "  {} {} ({}){}",
                    icon,
                    task.name,
                    task.status.as_str(),
                    elapsed_str
                );

                ListItem::new(content).style(Style::default().fg(status_color))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel))
                    .title(" Task List "),
            );
        frame.render_widget(list, chunks[1]);
    }
}

impl Default for TaskList {
    fn default() -> Self {
        Self::new()
    }
}

/// Renders a task progress summary.
pub fn render_task_summary(
    frame: &mut Frame,
    area: Rect,
    completed: usize,
    total: usize,
) {
    let theme = get_theme();
    let summary_text = format!("({}/{} tasks)", completed, total);

    let summary = Paragraph::new(summary_text)
        .style(Style::default().fg(theme.text))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_panel))
                .title(" Progress "),
        );
    frame.render_widget(summary, area);
}

/// Formats elapsed time in a human-readable format.
fn format_elapsed(seconds: f64) -> String {
    if seconds < 60.0 {
        format!("{:.1}s", seconds)
    } else if seconds < 3600.0 {
        let minutes = (seconds / 60.0) as u64;
        let secs = (seconds % 60.0) as u64;
        if secs > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}m", minutes)
        }
    } else {
        let hours = (seconds / 3600.0) as u64;
        let minutes = ((seconds % 3600.0) / 60.0) as u64;
        if minutes > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}h", hours)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_elapsed() {
        assert_eq!(format_elapsed(0.0), "0.0s");
        assert_eq!(format_elapsed(5.5), "5.5s");
        assert_eq!(format_elapsed(45.0), "45.0s");
        assert_eq!(format_elapsed(60.0), "1m 0s");
        assert_eq!(format_elapsed(90.0), "1m 30s");
        assert_eq!(format_elapsed(125.0), "2m 5s");
        assert_eq!(format_elapsed(3600.0), "1h 0m");
        assert_eq!(format_elapsed(3665.0), "1h 1m");
    }

    #[test]
    fn test_task_list_new() {
        let task_list = TaskList::new();
        assert_eq!(task_list.tasks.len(), 0);
    }

    #[test]
    fn test_task_list_with_tasks() {
        let tasks = vec![
            TaskItem {
                name: "Task 1".to_string(),
                status: AgentStatus::Completed,
                elapsed_seconds: Some(45.0),
            },
            TaskItem {
                name: "Task 2".to_string(),
                status: AgentStatus::Running,
                elapsed_seconds: Some(120.0),
            },
        ];
        let task_list = TaskList::with_tasks(tasks);
        assert_eq!(task_list.tasks.len(), 2);
    }
}


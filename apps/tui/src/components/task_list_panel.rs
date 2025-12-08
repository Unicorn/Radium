//! Task list panel component for displaying tasks with status and agent assignments.
//!
//! This component displays a list of tasks with their status, name, and assigned agent ID
//! in a scrollable table format.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use crate::state::{TaskListState, TaskListItem};
use crate::theme::get_theme;
use crate::icons::Icons;
use radium_core::models::task::TaskState;

/// Task list panel component
#[derive(Debug, Clone)]
pub struct TaskListPanel {
    /// Current scroll offset
    scroll_offset: usize,
    /// Whether the panel is focused
    focused: bool,
}

impl TaskListPanel {
    /// Creates a new task list panel.
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            focused: false,
        }
    }

    /// Sets the focus state of the panel.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Scrolls up by the specified number of lines.
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// Scrolls down by the specified number of lines.
    pub fn scroll_down(&mut self, amount: usize, max_items: usize) {
        let max_scroll = max_items.saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + amount).min(max_scroll);
    }

    /// Scrolls to the top.
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scrolls to the bottom.
    pub fn scroll_to_bottom(&mut self, max_items: usize) {
        self.scroll_offset = max_items.saturating_sub(1);
    }

    /// Renders the task list panel.
    ///
    /// # Arguments
    /// * `frame` - Frame to render into
    /// * `area` - Area to render in
    /// * `task_state` - Task list state to display
    /// * `focused` - Whether the panel is focused
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        task_state: &TaskListState,
        focused: bool,
    ) {
        self.focused = focused;
        let theme = get_theme();

        // Split area: progress summary (3 lines) and task table (remaining)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Progress summary
                Constraint::Min(5),    // Task list
            ])
            .split(area);

        // Render progress summary
        let (completed, failed, total) = task_state.get_progress();
        let summary_text = if total > 0 {
            format!("Tasks: {}/{} completed, {} failed", completed, total, failed)
        } else {
            "No active workflow".to_string()
        };

        let summary_style = if total > 0 {
            Style::default().fg(theme.text)
        } else {
            Style::default().fg(theme.text_muted)
        };

        let summary = Paragraph::new(summary_text)
            .style(summary_style)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if focused {
                        Style::default().fg(theme.border_active)
                    } else {
                        Style::default().fg(theme.border)
                    })
                    .title(" Progress "),
            );

        frame.render_widget(summary, chunks[0]);

        // Render task list
        if task_state.is_empty() {
            let empty_text = "No active workflow";
            let empty_widget = Paragraph::new(empty_text)
                .style(Style::default().fg(theme.text_muted))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(if focused {
                            Style::default().fg(theme.border_active)
                        } else {
                            Style::default().fg(theme.border)
                        })
                        .title(" Task List "),
                );
            frame.render_widget(empty_widget, chunks[1]);
            return;
        }

        // Get tasks and apply scroll offset
        let tasks = task_state.get_tasks();
        let viewport_height = chunks[1].height.saturating_sub(2) as usize; // Subtract borders
        let start_idx = self.scroll_offset.min(tasks.len().saturating_sub(1));
        let end_idx = (start_idx + viewport_height).min(tasks.len());
        let visible_tasks = &tasks[start_idx..end_idx];

        // Create table rows
        let rows: Vec<Row> = visible_tasks
            .iter()
            .map(|task| {
                // Get status icon and color
                let (icon, status_color) = Self::get_status_icon_and_color(&task.status);
                
                // Format task name (truncate if too long)
                let task_name = if task.name.len() > 30 {
                    format!("{}...", &task.name[..27])
                } else {
                    task.name.clone()
                };

                // Format agent ID (truncate if too long)
                let agent_id = if task.agent_id.len() > 20 {
                    format!("{}...", &task.agent_id[..17])
                } else {
                    task.agent_id.clone()
                };

                // Create cells with appropriate styling
                let status_cell = Cell::from(icon.to_string())
                    .style(Style::default().fg(status_color));
                
                let name_cell = Cell::from(task_name)
                    .style(if matches!(task.status, TaskState::Running) {
                        Style::default().fg(theme.text).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    });
                
                let agent_cell = Cell::from(agent_id)
                    .style(Style::default().fg(theme.text_muted));

                Row::new(vec![status_cell, name_cell, agent_cell]).height(1)
            })
            .collect();

        // Create table with fixed column widths: 20% status, 50% name, 30% agent
        let table = Table::new(
            rows,
            [
                Constraint::Percentage(20), // Status icon
                Constraint::Percentage(50),  // Task name
                Constraint::Percentage(30), // Agent ID
            ],
        )
        .header(
            Row::new(vec![
                Cell::from("Status").style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Cell::from("Task Name").style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Cell::from("Agent ID").style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            ])
            .height(1),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if focused {
                    Style::default().fg(theme.border_active)
                } else {
                    Style::default().fg(theme.border)
                })
                .title(" Task List "),
        );

        frame.render_widget(table, chunks[1]);
    }

    /// Gets the status icon and color for a task state.
    fn get_status_icon_and_color(state: &TaskState) -> (&'static str, ratatui::style::Color) {
        let theme = get_theme();
        match state {
            TaskState::Queued => (Icons::PENDING, theme.text_muted),
            TaskState::Running => (Icons::RUNNING, theme.warning),
            TaskState::Completed => (Icons::COMPLETED, theme.success),
            TaskState::Error(_) => (Icons::FAILED, theme.error),
            TaskState::Paused => (Icons::IDLE, theme.info),
            TaskState::Cancelled => (Icons::CANCELLED, theme.text_dim),
        }
    }
}

impl Default for TaskListPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_list_panel_new() {
        let panel = TaskListPanel::new();
        assert_eq!(panel.scroll_offset, 0);
        assert!(!panel.focused);
    }

    #[test]
    fn test_scroll_operations() {
        let mut panel = TaskListPanel::new();
        
        panel.scroll_down(5, 10);
        assert_eq!(panel.scroll_offset, 5);
        
        panel.scroll_up(2);
        assert_eq!(panel.scroll_offset, 3);
        
        panel.scroll_to_top();
        assert_eq!(panel.scroll_offset, 0);
        
        panel.scroll_to_bottom(10);
        assert_eq!(panel.scroll_offset, 9);
    }

    #[test]
    fn test_get_status_icon_and_color() {
        // Just verify the function doesn't panic
        let _ = TaskListPanel::get_status_icon_and_color(&TaskState::Queued);
        let _ = TaskListPanel::get_status_icon_and_color(&TaskState::Running);
        let _ = TaskListPanel::get_status_icon_and_color(&TaskState::Completed);
        let _ = TaskListPanel::get_status_icon_and_color(&TaskState::Error("test".to_string()));
        let _ = TaskListPanel::get_status_icon_and_color(&TaskState::Paused);
        let _ = TaskListPanel::get_status_icon_and_color(&TaskState::Cancelled);
    }
}


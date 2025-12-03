//! Task viewer

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use crate::app::AppState;
use radium_core::proto::{ListTasksRequest, Task};
use tonic::Request;

/// Task view data
pub struct TaskViewData {
    pub tasks: Vec<Task>,
    pub selected_index: usize,
}

impl TaskViewData {
    pub async fn fetch(app_state: &AppState) -> anyhow::Result<Self> {
        let client_guard = app_state.client.lock().await;
        let mut client =
            client_guard.as_ref().ok_or_else(|| anyhow::anyhow!("Not connected"))?.clone();

        let request = Request::new(ListTasksRequest {});
        let response = client.list_tasks(request).await?;
        let tasks = response.into_inner().tasks;

        Ok(Self { tasks, selected_index: 0 })
    }

    pub fn next_task(&mut self) {
        if !self.tasks.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.tasks.len();
        }
    }

    pub fn previous_task(&mut self) {
        if !self.tasks.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.tasks.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn selected_task(&self) -> Option<&Task> {
        self.tasks.get(self.selected_index)
    }
}

/// Render the task viewer
pub fn render_task_view(frame: &mut Frame, area: Rect, data: &TaskViewData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(5),     // Task list
            Constraint::Length(10), // Task details
            Constraint::Length(3),  // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new("📋 Task Viewer")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Task list
    let task_rows: Vec<Row> = data
        .tasks
        .iter()
        .enumerate()
        .map(|(idx, task)| {
            let style = if idx == data.selected_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            Row::new(vec![
                task.id.clone(),
                task.name.clone(),
                task.agent_id.clone(),
                task.state.clone(),
            ])
            .style(style)
        })
        .collect();

    let task_table = Table::new(
        task_rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .block(Block::default().borders(Borders::ALL).title(" Tasks "))
    .header(
        Row::new(vec!["ID", "Name", "Agent ID", "State"]).style(Style::default().fg(Color::Yellow)),
    );
    frame.render_widget(task_table, chunks[1]);

    // Task details
    if let Some(task) = data.selected_task() {
        let details = vec![
            Line::from(vec![
                Span::styled("ID: ", Style::default().fg(Color::Cyan)),
                Span::raw(&task.id),
            ]),
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Cyan)),
                Span::raw(&task.name),
            ]),
            Line::from(vec![
                Span::styled("Description: ", Style::default().fg(Color::Cyan)),
                Span::raw(&task.description),
            ]),
            Line::from(vec![
                Span::styled("Agent ID: ", Style::default().fg(Color::Cyan)),
                Span::raw(&task.agent_id),
            ]),
            Line::from(vec![
                Span::styled("State: ", Style::default().fg(Color::Cyan)),
                Span::raw(&task.state),
            ]),
            Line::from(vec![
                Span::styled("Input: ", Style::default().fg(Color::Cyan)),
                Span::raw(&task.input_json),
            ]),
            Line::from(vec![
                Span::styled("Result: ", Style::default().fg(Color::Cyan)),
                Span::raw(&task.result_json),
            ]),
        ];

        let details_para = Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title(" Task Details "))
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(details_para, chunks[2]);
    } else {
        let no_task = Paragraph::new("No tasks available")
            .block(Block::default().borders(Borders::ALL).title(" Task Details "));
        frame.render_widget(no_task, chunks[2]);
    }

    // Help
    let help = Paragraph::new("  [↑↓] Navigate  [Enter] View Details  [1] Dashboard  [q] Quit")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title(" Keys "));
    frame.render_widget(help, chunks[3]);
}

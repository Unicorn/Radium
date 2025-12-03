//! Workflow management view

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use crate::app::AppState;
use radium_core::proto::{ListWorkflowsRequest, Workflow};
use tonic::Request;

/// Workflow view data
pub struct WorkflowViewData {
    pub workflows: Vec<Workflow>,
    pub selected_index: usize,
}

impl WorkflowViewData {
    pub async fn fetch(app_state: &AppState) -> anyhow::Result<Self> {
        let client_guard = app_state.client.lock().await;
        let mut client =
            client_guard.as_ref().ok_or_else(|| anyhow::anyhow!("Not connected"))?.clone();

        let request = Request::new(ListWorkflowsRequest {});
        let response = client.list_workflows(request).await?;
        let workflows = response.into_inner().workflows;

        Ok(Self { workflows, selected_index: 0 })
    }

    pub fn next_workflow(&mut self) {
        if !self.workflows.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.workflows.len();
        }
    }

    pub fn previous_workflow(&mut self) {
        if !self.workflows.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.workflows.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn selected_workflow(&self) -> Option<&Workflow> {
        self.workflows.get(self.selected_index)
    }
}

/// Render the workflow management view
pub fn render_workflow_view(frame: &mut Frame, area: Rect, data: &WorkflowViewData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(5),     // Workflow list
            Constraint::Length(10), // Workflow details
            Constraint::Length(3),  // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new("⚙️  Workflow Management")
        .style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Workflow list
    let workflow_rows: Vec<Row> = data
        .workflows
        .iter()
        .enumerate()
        .map(|(idx, workflow)| {
            let style = if idx == data.selected_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            Row::new(vec![
                workflow.id.clone(),
                workflow.name.clone(),
                format!("{}", workflow.steps.len()),
                workflow.state.clone(),
            ])
            .style(style)
        })
        .collect();

    let workflow_table = Table::new(
        workflow_rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .block(Block::default().borders(Borders::ALL).title(" Workflows "))
    .header(
        Row::new(vec!["ID", "Name", "Steps", "State"]).style(Style::default().fg(Color::Yellow)),
    );
    frame.render_widget(workflow_table, chunks[1]);

    // Workflow details
    if let Some(workflow) = data.selected_workflow() {
        let mut details = vec![
            Line::from(vec![
                Span::styled("ID: ", Style::default().fg(Color::Cyan)),
                Span::raw(&workflow.id),
            ]),
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Cyan)),
                Span::raw(&workflow.name),
            ]),
            Line::from(vec![
                Span::styled("Description: ", Style::default().fg(Color::Cyan)),
                Span::raw(&workflow.description),
            ]),
            Line::from(vec![
                Span::styled("State: ", Style::default().fg(Color::Cyan)),
                Span::raw(&workflow.state),
            ]),
            Line::from(vec![
                Span::styled("Steps: ", Style::default().fg(Color::Cyan)),
                Span::raw(workflow.steps.len().to_string()),
            ]),
        ];

        if !workflow.steps.is_empty() {
            details.push(Line::from("Steps:"));
            for step in &workflow.steps {
                details.push(Line::from(format!("  - {} (Task: {})", step.name, step.task_id)));
            }
        }

        let details_para = Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title(" Workflow Details "))
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(details_para, chunks[2]);
    } else {
        let no_workflow = Paragraph::new("No workflows available")
            .block(Block::default().borders(Borders::ALL).title(" Workflow Details "));
        frame.render_widget(no_workflow, chunks[2]);
    }

    // Help
    let help = Paragraph::new("  [↑↓] Navigate  [Enter] View Details  [1] Dashboard  [q] Quit")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title(" Keys "));
    frame.render_widget(help, chunks[3]);
}

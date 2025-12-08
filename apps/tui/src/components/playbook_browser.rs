//! Playbook browser component for displaying and navigating playbooks.

use crate::theme::get_theme;
use radium_core::playbooks::types::{Playbook, PlaybookPriority};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Playbook browser component state.
#[derive(Debug, Clone)]
pub struct PlaybookBrowser {
    /// List of playbooks to display.
    pub playbooks: Vec<Playbook>,
    /// Currently selected index.
    pub selected_index: usize,
    /// Whether to show detail view.
    pub show_detail: bool,
}

impl PlaybookBrowser {
    /// Create a new playbook browser.
    pub fn new(playbooks: Vec<Playbook>) -> Self {
        Self {
            playbooks,
            selected_index: 0,
            show_detail: false,
        }
    }

    /// Move selection up.
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down.
    pub fn move_down(&mut self) {
        if self.selected_index < self.playbooks.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Toggle detail view.
    pub fn toggle_detail(&mut self) {
        self.show_detail = !self.show_detail;
    }

    /// Get currently selected playbook.
    pub fn selected_playbook(&self) -> Option<&Playbook> {
        self.playbooks.get(self.selected_index)
    }

    /// Render the playbook browser.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let theme = get_theme();

        if self.show_detail {
            self.render_detail(frame, area, theme);
        } else {
            self.render_list(frame, area, theme);
        }
    }

    /// Render list view.
    fn render_list(&self, frame: &mut Frame, area: Rect, theme: &crate::theme::RadiumTheme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(5),   // Playbook list
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Organizational Playbooks")
            .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            );
        frame.render_widget(title, chunks[0]);

        // Build list items
        let items: Vec<ListItem> = self
            .playbooks
            .iter()
            .enumerate()
            .map(|(idx, playbook)| {
                let is_selected = idx == self.selected_index;
                let priority_color = match playbook.priority {
                    PlaybookPriority::Required => Color::Red,
                    PlaybookPriority::Recommended => Color::Yellow,
                    PlaybookPriority::Optional => Color::Green,
                };

                let priority_str = format!("[{}]", playbook.priority);
                let tags_str = if playbook.tags.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", playbook.tags.join(", "))
                };

                let text = format!(
                    "{} {} {}{}",
                    if is_selected { "▶" } else { " " },
                    priority_str,
                    playbook.description,
                    tags_str
                );

                let style = if is_selected {
                    Style::default()
                        .fg(theme.highlight_fg)
                        .bg(theme.highlight_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };

                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            )
            .highlight_style(
                Style::default()
                    .fg(theme.highlight_fg)
                    .bg(theme.highlight_bg)
                    .add_modifier(Modifier::BOLD),
            );

        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(self.selected_index));

        frame.render_stateful_widget(list, chunks[1], &mut state);
    }

    /// Render detail view.
    fn render_detail(&self, frame: &mut Frame, area: Rect, theme: &crate::theme::RadiumTheme) {
        if let Some(playbook) = self.selected_playbook() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Min(5),   // Content
                ])
                .split(area);

            // Header
            let header = Paragraph::new(format!("Playbook: {}", playbook.uri))
                .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border))
                        .style(Style::default().bg(theme.bg_panel)),
                );
            frame.render_widget(header, chunks[0]);

            // Content
            let content = format!(
                "Description: {}\n\nPriority: {}\nTags: {}\nScope: {}\n\n{}",
                playbook.description,
                playbook.priority,
                if playbook.tags.is_empty() {
                    "(none)".to_string()
                } else {
                    playbook.tags.join(", ")
                },
                if playbook.applies_to.is_empty() {
                    "all".to_string()
                } else {
                    playbook.applies_to.join(", ")
                },
                playbook.content
            );

            let content_para = Paragraph::new(content)
                .style(Style::default().fg(theme.text))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border))
                        .style(Style::default().bg(theme.bg_panel)),
                )
                .wrap(ratatui::widgets::Wrap { trim: true });

            frame.render_widget(content_para, chunks[1]);
        }
    }
}


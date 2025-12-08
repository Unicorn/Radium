//! Interactive table component for menu selections.
//!
//! Provides a table widget with keyboard navigation support.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};

/// Interactive table component for displaying selectable lists
pub struct InteractiveTable {
    /// Table state for selection tracking
    state: TableState,
    /// Table items (rows)
    items: Vec<Vec<String>>,
    /// Column headers
    headers: Vec<String>,
    /// Column widths (constraints)
    widths: Vec<Constraint>,
}

impl InteractiveTable {
    /// Creates a new interactive table
    pub fn new(headers: Vec<String>, widths: Vec<Constraint>) -> Self {
        Self {
            state: TableState::default(),
            items: Vec::new(),
            headers,
            widths,
        }
    }

    /// Sets the table items
    pub fn set_items(&mut self, items: Vec<Vec<String>>) {
        self.items = items;
        // Reset selection if out of bounds
        if let Some(selected) = self.state.selected() {
            if selected >= self.items.len() {
                self.state.select(Some(0));
            }
        } else if !self.items.is_empty() {
            self.state.select(Some(0));
        }
    }

    /// Moves selection up
    pub fn previous(&mut self) {
        let i = self
            .state
            .selected()
            .map_or(0, |i| if i == 0 { self.items.len().saturating_sub(1) } else { i - 1 });
        self.state.select(Some(i));
    }

    /// Moves selection down
    pub fn next(&mut self) {
        let i = self
            .state
            .selected()
            .map_or(0, |i| if i >= self.items.len().saturating_sub(1) { 0 } else { i + 1 });
        self.state.select(Some(i));
    }

    /// Gets the currently selected index
    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    /// Sets the selected index
    pub fn set_selected(&mut self, index: Option<usize>) {
        if let Some(idx) = index {
            if idx < self.items.len() {
                self.state.select(Some(idx));
            }
        } else {
            self.state.select(None);
        }
    }

    /// Renders the table
    pub fn render(&mut self, frame: &mut Frame, area: Rect, title: Option<&str>) {
        let theme = crate::theme::get_theme();

        if self.items.is_empty() {
            let empty_text = "No items to display";
            let empty_widget = Paragraph::new(empty_text)
                .style(Style::default().fg(theme.text_muted))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border))
                        .title(title.unwrap_or("")),
                );
            frame.render_widget(empty_widget, area);
            return;
        }

        // Create header row
        let header_row = Row::new(
            self.headers
                .iter()
                .map(|h| Cell::from(h.as_str()).style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))),
        )
        .height(1);

        // Create data rows
        let rows: Vec<Row> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = self.state.selected() == Some(i);
                let style = if is_selected {
                    Style::default()
                        .fg(theme.bg_primary)
                        .bg(theme.primary)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };

                Row::new(
                    item.iter()
                        .map(|cell| Cell::from(cell.as_str()).style(style)),
                )
                .height(1)
            })
            .collect();

        // Create table widget
        let table = Table::new(rows, &self.widths)
            .header(header_row)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(title.unwrap_or("")),
            )
            .row_highlight_style(
                Style::default()
                    .fg(theme.bg_primary)
                    .bg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(table, area, &mut self.state);
    }
}


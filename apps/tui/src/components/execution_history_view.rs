//! Execution history view component for browsing task executions.
//!
//! Displays execution records in a sortable, filterable table with keyboard navigation.

use crate::state::{ExecutionRecord, ExecutionStatus};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};

/// Column to sort by
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    TaskName,
    Status,
    Duration,
    Tokens,
    Cost,
    RetryAttempt,
    CycleNumber,
}

/// Execution history view component
pub struct ExecutionHistoryView {
    /// Filtered and sorted records to display
    records: Vec<ExecutionRecord>,
    /// Table state for selection
    state: TableState,
    /// Column to sort by
    sort_column: SortColumn,
    /// Sort direction (true = ascending, false = descending)
    sort_ascending: bool,
    /// Optional status filter
    status_filter: Option<ExecutionStatus>,
    /// Column headers
    headers: Vec<String>,
    /// Column width constraints
    widths: Vec<Constraint>,
}

impl ExecutionHistoryView {
    /// Creates a new execution history view with records.
    pub fn new(records: Vec<ExecutionRecord>) -> Self {
        let mut view = Self {
            records: Vec::new(),
            state: TableState::default(),
            sort_column: SortColumn::TaskName,
            sort_ascending: true,
            status_filter: None,
            headers: vec![
                "Task Name".to_string(),
                "Status".to_string(),
                "Duration".to_string(),
                "Tokens".to_string(),
                "Cost".to_string(),
                "Retry".to_string(),
                "Cycle".to_string(),
            ],
            widths: vec![
                Constraint::Percentage(25), // Task Name
                Constraint::Percentage(12), // Status
                Constraint::Percentage(12), // Duration
                Constraint::Percentage(15), // Tokens
                Constraint::Percentage(10), // Cost
                Constraint::Percentage(8),  // Retry
                Constraint::Percentage(8),  // Cycle
            ],
        };
        view.set_records(records);
        view
    }

    /// Sets the records and applies filtering/sorting.
    pub fn set_records(&mut self, records: Vec<ExecutionRecord>) {
        self.records = records;
        self.apply_filter();
        self.apply_sort();
        
        // Reset selection if out of bounds
        if let Some(selected) = self.state.selected() {
            if selected >= self.records.len() {
                self.state.select(if self.records.is_empty() { None } else { Some(0) });
            }
        } else if !self.records.is_empty() {
            self.state.select(Some(0));
        }
    }

    /// Applies the status filter.
    fn apply_filter(&mut self) {
        if let Some(filter) = self.status_filter {
            self.records.retain(|r| r.status == filter);
        }
    }

    /// Applies sorting to records.
    fn apply_sort(&mut self) {
        self.records.sort_by(|a, b| {
            let cmp = match self.sort_column {
                SortColumn::TaskName => a.task_name.cmp(&b.task_name),
                SortColumn::Status => a.status.as_str().cmp(b.status.as_str()),
                SortColumn::Duration => {
                    let a_dur = a.duration_secs.unwrap_or(0);
                    let b_dur = b.duration_secs.unwrap_or(0);
                    a_dur.cmp(&b_dur)
                }
                SortColumn::Tokens => {
                    a.tokens.total_tokens.cmp(&b.tokens.total_tokens)
                }
                SortColumn::Cost => {
                    a.cost.partial_cmp(&b.cost).unwrap_or(std::cmp::Ordering::Equal)
                }
                SortColumn::RetryAttempt => a.retry_attempt.cmp(&b.retry_attempt),
                SortColumn::CycleNumber => a.cycle_number.cmp(&b.cycle_number),
            };
            
            if self.sort_ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });
    }

    /// Moves selection up.
    pub fn select_previous(&mut self) {
        let i = self
            .state
            .selected()
            .map_or(0, |i| if i == 0 { self.records.len().saturating_sub(1) } else { i - 1 });
        self.state.select(if self.records.is_empty() { None } else { Some(i) });
    }

    /// Moves selection down.
    pub fn select_next(&mut self) {
        let i = self
            .state
            .selected()
            .map_or(0, |i| if i >= self.records.len().saturating_sub(1) { 0 } else { i + 1 });
        self.state.select(if self.records.is_empty() { None } else { Some(i) });
    }

    /// Gets the currently selected record.
    pub fn get_selected_record(&self) -> Option<&ExecutionRecord> {
        self.state.selected().and_then(|i| self.records.get(i))
    }

    /// Toggles sort column and direction.
    pub fn toggle_sort(&mut self, column: SortColumn) {
        if self.sort_column == column {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = column;
            self.sort_ascending = true;
        }
        self.apply_sort();
    }

    /// Sets status filter.
    pub fn set_filter(&mut self, status: Option<ExecutionStatus>) {
        self.status_filter = status;
        // Need to reset records to original set, then reapply filter
        // For now, we'll need to call set_records again from outside
        // This is a limitation - we'd need to store original records separately
    }

    /// Clears status filter.
    pub fn clear_filter(&mut self) {
        self.status_filter = None;
        // Same limitation as set_filter
    }

    /// Handles keyboard input.
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Option<Action> {
        match key.code {
            crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                self.select_previous();
                None
            }
            crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                self.select_next();
                None
            }
            crossterm::event::KeyCode::Enter => {
                self.get_selected_record().map(|_| Action::ViewDetail)
            }
            crossterm::event::KeyCode::Char('s') => {
                // Cycle sort columns
                let next_col = match self.sort_column {
                    SortColumn::TaskName => SortColumn::Status,
                    SortColumn::Status => SortColumn::Duration,
                    SortColumn::Duration => SortColumn::Tokens,
                    SortColumn::Tokens => SortColumn::Cost,
                    SortColumn::Cost => SortColumn::RetryAttempt,
                    SortColumn::RetryAttempt => SortColumn::CycleNumber,
                    SortColumn::CycleNumber => SortColumn::TaskName,
                };
                self.toggle_sort(next_col);
                None
            }
            crossterm::event::KeyCode::Char(' ') => {
                // Toggle sort direction
                self.sort_ascending = !self.sort_ascending;
                self.apply_sort();
                None
            }
            _ => None,
        }
    }

    /// Renders the execution history view.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let theme = crate::theme::get_theme();

        if self.records.is_empty() {
            let empty_text = if self.status_filter.is_some() {
                "No executions match the current filter"
            } else {
                "No executions recorded"
            };
            let empty_widget = Paragraph::new(empty_text)
                .style(Style::default().fg(theme.text_muted))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border))
                        .title(" Execution History "),
                );
            frame.render_widget(empty_widget, area);
            return;
        }

        // Create header row with sort indicators
        let header_row = Row::new(
            self.headers
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    let mut cell_text = h.clone();
                    if i == self.sort_column_index() {
                        let indicator = if self.sort_ascending { " ↑" } else { " ↓" };
                        cell_text.push_str(indicator);
                    }
                    Cell::from(cell_text.as_str())
                        .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
                }),
        )
        .height(1);

        // Create data rows
        let rows: Vec<Row> = self
            .records
            .iter()
            .enumerate()
            .map(|(i, record)| {
                let is_selected = self.state.selected() == Some(i);
                let base_style = if is_selected {
                    Style::default()
                        .fg(theme.bg_primary)
                        .bg(theme.primary)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };

                let status_color = Self::status_color(record.status, &theme);
                let status_style = if is_selected {
                    base_style
                } else {
                    Style::default().fg(status_color)
                };

                let duration_text = if let Some(dur) = record.duration_secs {
                    Self::format_duration(dur)
                } else {
                    "-".to_string()
                };

                let tokens_text = format!("{}", format_number(record.tokens.total_tokens));
                let cost_text = format!("${:.4}", record.cost);

                Row::new(vec![
                    Cell::from(record.task_name.as_str()).style(base_style),
                    Cell::from(record.status.as_str()).style(status_style),
                    Cell::from(duration_text.as_str()).style(base_style),
                    Cell::from(tokens_text.as_str()).style(base_style),
                    Cell::from(cost_text.as_str()).style(base_style),
                    Cell::from(record.retry_attempt.to_string().as_str()).style(base_style),
                    Cell::from(record.cycle_number.to_string().as_str()).style(base_style),
                ])
                .height(1)
            })
            .collect();

        // Build title with filter info
        let mut title = " Execution History ".to_string();
        if let Some(filter) = self.status_filter {
            title.push_str(&format!(" [Filter: {}]", filter.as_str()));
        }

        // Create table widget
        let table = Table::new(rows, &self.widths)
            .header(header_row)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(title),
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

    /// Gets the index of the current sort column.
    fn sort_column_index(&self) -> usize {
        match self.sort_column {
            SortColumn::TaskName => 0,
            SortColumn::Status => 1,
            SortColumn::Duration => 2,
            SortColumn::Tokens => 3,
            SortColumn::Cost => 4,
            SortColumn::RetryAttempt => 5,
            SortColumn::CycleNumber => 6,
        }
    }

    /// Returns the color for an execution status.
    fn status_color(status: ExecutionStatus, theme: &crate::theme::RadiumTheme) -> Color {
        match status {
            ExecutionStatus::Running => theme.info,
            ExecutionStatus::Completed => theme.success,
            ExecutionStatus::Failed => theme.error,
            ExecutionStatus::Pending => theme.warning,
            ExecutionStatus::Cancelled => theme.text_muted,
        }
    }

    /// Formats duration in human-readable format.
    fn format_duration(secs: u64) -> String {
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
}

/// Action to take after handling input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// View detail for selected record
    ViewDetail,
}

/// Formats a number with commas.
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut count = 0;

    for c in s.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(c);
        count += 1;
    }

    result.chars().rev().collect()
}


//! Process panel component for displaying running processes.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};
use radium_core::process::{ProcessHandle, ProcessStatus};
use crate::theme::get_theme;

/// Process panel component for displaying processes with their status.
#[derive(Debug, Clone)]
pub struct ProcessPanel {
    /// Current scroll offset.
    scroll_offset: usize,
    /// Whether the panel is focused.
    focused: bool,
    /// Table state for selection.
    table_state: TableState,
}

impl ProcessPanel {
    /// Creates a new process panel.
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            focused: false,
            table_state: TableState::default(),
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

    /// Renders the process panel.
    ///
    /// # Arguments
    /// * `frame` - Frame to render into
    /// * `area` - Area to render in
    /// * `processes` - List of processes to display
    /// * `focused` - Whether the panel is focused
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        processes: &[ProcessHandle],
        selected_index: Option<usize>,
        focused: bool,
    ) {
        self.focused = focused;
        let theme = get_theme();

        // Set table state selection
        self.table_state.select(selected_index);

        // Create header
        let header_cells = ["Status", "PID", "Command", "Uptime", "Restarts", "Status Details"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(theme.text_muted).bold()));
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        // Create rows from processes
        let rows: Vec<Row> = processes
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .map(|(idx, process)| {
                let is_selected = Some(idx) == selected_index;

                // Status icon
                let (status_icon, status_color) = match process.status {
                    ProcessStatus::Running => ("●", Color::Green),
                    ProcessStatus::Stopped => ("○", Color::Gray),
                    ProcessStatus::Crashed => ("✗", Color::Red),
                    ProcessStatus::Restarting => ("⟳", Color::Yellow),
                };

                // PID display
                let pid_str = process.pid.map_or("N/A".to_string(), |p| p.to_string());

                // Command (truncate if too long)
                let cmd_display = if process.command.len() > 20 {
                    format!("{}...", &process.command[..17])
                } else {
                    process.command.clone()
                };

                // Uptime display
                let uptime = process.uptime_display();

                // Restart count
                let restarts = process.restart_count.to_string();

                // Status details
                let status_details = match process.status {
                    ProcessStatus::Crashed => {
                        if let Some(code) = process.exit_code {
                            format!("Exit code: {}", code)
                        } else {
                            "Crashed".to_string()
                        }
                    }
                    ProcessStatus::Stopped => {
                        if let Some(code) = process.exit_code {
                            format!("Exited: {}", code)
                        } else {
                            "Stopped".to_string()
                        }
                    }
                    _ => "".to_string(),
                };

                let cells = vec![
                    Cell::from(status_icon).style(Style::default().fg(status_color)),
                    Cell::from(pid_str),
                    Cell::from(cmd_display),
                    Cell::from(uptime),
                    Cell::from(restarts),
                    Cell::from(status_details),
                ];

                let row_style = if is_selected {
                    Style::default().bg(theme.bg_element).fg(theme.text)
                } else {
                    Style::default()
                };

                Row::new(cells).style(row_style)
            })
            .collect();

        // Create widths
        let widths = [
            Constraint::Length(6),  // Status
            Constraint::Length(8),  // PID
            Constraint::Length(20), // Command
            Constraint::Length(10), // Uptime
            Constraint::Length(10), // Restarts
            Constraint::Min(15),    // Status Details
        ];

        // Create table
        let border_style = if self.focused {
            Style::default().fg(theme.border_active)
        } else {
            Style::default().fg(theme.border)
        };

        let title = if processes.is_empty() {
            " Processes (0) "
        } else {
            // Use a Box to allocate on heap for the formatted string
            Box::leak(Box::new(format!(" Processes ({}) ", processes.len()))).as_str()
        };

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title(title)
                    .title_style(Style::default().fg(theme.text).bold()),
            )
            .row_highlight_style(
                Style::default()
                    .bg(theme.bg_element)
                    .fg(theme.text)
                    .add_modifier(Modifier::BOLD),
            );

        // Render table with state
        frame.render_stateful_widget(table, area, &mut self.table_state);

        // Render help text if there are no processes
        if processes.is_empty() {
            let help_text = Paragraph::new("No processes running. Use Ctrl+N to spawn a new process.")
                .style(Style::default().fg(theme.text_muted))
                .alignment(Alignment::Center);

            let inner_area = Rect {
                x: area.x + 2,
                y: area.y + (area.height / 2),
                width: area.width.saturating_sub(4),
                height: 1,
            };

            frame.render_widget(help_text, inner_area);
        }
    }

    /// Renders process details for the selected process.
    pub fn render_details(
        &self,
        frame: &mut Frame,
        area: Rect,
        process: &ProcessHandle,
    ) {
        let theme = get_theme();

        // Create details text
        let details = vec![
            format!("Process ID: {}", process.id),
            format!("PID: {}", process.pid.map_or("N/A".to_string(), |p| p.to_string())),
            format!("Command: {} {}", process.command, process.args.join(" ")),
            format!("Working Directory: {}", process.working_dir.display()),
            format!("Status: {}", process.status),
            format!("Uptime: {}", process.uptime_display()),
            format!("Restart Count: {}", process.restart_count),
            format!("Log File: {}", process.log_file.display()),
        ];

        let text: Vec<Line> = details
            .into_iter()
            .map(|line| Line::from(line))
            .collect();

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Process Details ")
                    .title_style(Style::default().fg(theme.text).bold()),
            )
            .style(Style::default().fg(theme.text));

        frame.render_widget(paragraph, area);
    }
}

impl Default for ProcessPanel {
    fn default() -> Self {
        Self::new()
    }
}

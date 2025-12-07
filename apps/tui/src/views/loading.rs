//! Loading states and progress indicators for async operations.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph},
};

use crate::icons::Icons;
use crate::theme::THEME;

/// Loading state information.
#[derive(Debug, Clone)]
pub struct LoadingState {
    /// Loading message
    pub message: String,
    /// Progress percentage (0-100), None for indeterminate
    pub progress: Option<u16>,
}

impl LoadingState {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            progress: None,
        }
    }

    pub fn with_progress(mut self, progress: u16) -> Self {
        self.progress = Some(progress.min(100));
        self
    }
}

/// Render a loading indicator.
pub fn render_loading(frame: &mut Frame, area: Rect, state: &LoadingState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    // Loading message
    let message_text = vec![
        Line::from(""),
        Line::from(
            Span::styled(
                format!("{} {}", Icons::LOADING, state.message),
                Style::default().fg(THEME.info),
            )
        ),
    ];

    let message = Paragraph::new(message_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border))
        );

    frame.render_widget(message, chunks[0]);

    // Progress bar if available
    if let Some(progress) = state.progress {
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(THEME.border))
            )
            .gauge_style(
                Style::default()
                    .fg(THEME.primary)
                    .bg(THEME.bg_element)
            )
            .percent(progress as u16)
            .label(format!("{}%", progress));

        frame.render_widget(gauge, chunks[1]);
    }
}


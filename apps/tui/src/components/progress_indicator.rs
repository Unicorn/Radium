//! Progress indicator component using ratatui Gauge widget.
//!
//! Provides a consistent way to display progress bars throughout the application.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge},
};

/// Renders a progress gauge with consistent styling
pub fn render_progress_gauge(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    progress: f64,
    _label_style: Option<Style>,
) {
    let theme = crate::theme::get_theme();

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(label),
        )
        .gauge_style(
            Style::default()
                .fg(theme.primary)
                .bg(theme.bg_element),
        )
        .percent((progress * 100.0) as u16)
        .label(format!("{:.1}%", progress * 100.0));

    frame.render_widget(gauge, area);
}

/// Renders a progress gauge with custom colors
pub fn render_progress_gauge_custom(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    progress: f64,
    fill_color: Color,
    unfilled_color: Color,
) {
    let theme = crate::theme::get_theme();

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(label),
        )
        .gauge_style(
            Style::default()
                .fg(fill_color)
                .bg(unfilled_color),
        )
        .percent((progress * 100.0) as u16)
        .label(format!("{:.1}%", progress * 100.0));

    frame.render_widget(gauge, area);
}

/// Renders a simple progress bar without borders
pub fn render_progress_bar_simple(
    frame: &mut Frame,
    area: Rect,
    progress: f64,
    fill_color: Color,
) {
    let theme = crate::theme::get_theme();

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(
            Style::default()
                .fg(fill_color)
                .bg(theme.bg_element),
        )
        .percent((progress * 100.0) as u16);

    frame.render_widget(gauge, area);
}


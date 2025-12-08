//! Artistic ASCII logo component for Radium TUI.

use ratatui::{
    prelude::*,
    widgets::Paragraph,
};

/// ASCII art for "RADIUM" in block style
const RADIUM_TEXT: &[&str] = &[
    "▗▄▄▖  ▗▄▖ ▗▄▄▄ ▗▄▄▄▖▗▖ ▗▖▗▖  ▗▖",
    "▐▌ ▐▌▐▌ ▐▌▐▌  █  █  ▐▌ ▐▌▐▛▚▞▜▌",
    "▐▛▀▚▖▐▛▀▜▌▐▌  █  █  ▐▌ ▐▌▐▌  ▐▌",
    "▐▌ ▐▌▐▌ ▐▌▐▙▄▄▀▗▄█▄▖▝▚▄▞▘▐▌  ▐▌",
];

/// Simple compact version for small terminals
const RADIUM_SIMPLE: &[&str] = &[
    "█▀█ █▀▀ █▀█ █ █ █▄█ █▀▄",
    "█▀▄ ██▄ █▀█ █▄█ █ █ █▄▀",
];

/// Renders a line with purple coloring
fn render_colored_line(line: &str, purple: Color) -> Line<'_> {
    Line::from(Span::styled(line.to_string(), Style::default().fg(purple)))
}

/// Renders the Radium logo with responsive sizing
pub fn render_logo(frame: &mut Frame, area: Rect) {
    let theme = crate::theme::get_theme();
    let purple = theme.purple;

    let is_narrow = area.width < 100;
    let is_short = area.height < 22;

    let logo_lines: Vec<Line> = if is_narrow || is_short {
        // Simple compact version for small terminals
        RADIUM_SIMPLE
            .iter()
            .map(|line| Line::from(Span::styled(*line, Style::default().fg(purple))))
            .collect()
    } else {
        // Full size - use purple for all lines
        RADIUM_TEXT
            .iter()
            .map(|line| render_colored_line(line, purple))
            .collect()
    };

    let logo = Paragraph::new(logo_lines)
        .alignment(Alignment::Center)
        .block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::NONE));

    frame.render_widget(logo, area);
}

/// ASCII art for "RADIUM" in compact single-line style (for title bar)
const RADIUM_COMPACT: &[&str] = &[
    "██████╗  █████╗ ██████╗ ██╗██╗   ██╗███╗   ███╗",
    "██╔══██╗██╔══██╗██╔══██╗██║██║   ██║████╗ ████║",
    "██████╔╝███████║██║  ██║██║██║   ██║██╔████╔██║",
    "██╔══██╗██╔══██║██║  ██║██║██║   ██║██║╚██╔╝██║",
    "██║  ██║██║  ██║██████╔╝██║╚██████╔╝██║ ╚═╝ ██║",
    "╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝ ╚═╝ ╚═════╝ ╚═╝     ╚═╝",
];

/// Simple single-line ASCII art for title bar
const RADIUM_SINGLE_LINE: &str = "█▀█ █▀▀ █▀█ █ █ █▄█ █▀▄";

/// Renders a compact logo for the title bar
pub fn render_logo_compact(frame: &mut Frame, area: Rect) {
    let theme = crate::theme::get_theme();
    let purple = theme.purple;

    // Use single-line ASCII art if area is wide enough
    if area.width >= 30 {
        // Single-line compact ASCII art version
        let logo_text = RADIUM_SINGLE_LINE;
        let logo = Paragraph::new(logo_text)
            .style(Style::default().fg(purple).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Left)
            .block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::NONE));
        frame.render_widget(logo, area);
    } else {
        // Fallback to text if too small
        let logo_text = "RADIUM";
        let logo = Paragraph::new(logo_text)
            .style(Style::default().fg(purple).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Left)
            .block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::NONE));
        frame.render_widget(logo, area);
    }
}


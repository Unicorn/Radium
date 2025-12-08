//! Title bar component for the global layout.
//!
//! Displays Radium branding on the left and status/metadata on the right.

use ratatui::{
    prelude::*,
    widgets::Paragraph,
};

/// Renders the title bar with branding and status information
pub fn render_title_bar(
    frame: &mut Frame,
    area: Rect,
    version: &str,
    _model_info: Option<&str>,
    orchestration_status: Option<&str>,
    connected_services: &[String],
) {
    let theme = crate::theme::get_theme();
    
    // Neon green color for "Radium" text (#39FF14 is a bright neon green)
    let neon_green = Color::Rgb(57, 255, 20);
    
    // Subtle light gray for pipe dividers (barely visible)
    let divider_color = Color::Rgb(80, 80, 80);
    
    // Light purple for bottom border
    let light_purple = theme.purple;

    // Create content area (excluding the bottom border line)
    let content_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height.saturating_sub(1), // Reserve last line for border
    };

    // Split into left and right sections
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20), // Left side
            Constraint::Fill(1), // Spacer
            Constraint::Min(40), // Right side
        ])
        .split(content_area);

    // Left side: "Radium" (neon green, bold) | Version (normal text)
    let left_parts = vec![
        Span::styled(
            "Radium",
            Style::default().fg(neon_green).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            version,
            Style::default().fg(theme.text),
        ),
    ];

    let left_widget = Paragraph::new(Line::from(left_parts))
        .alignment(Alignment::Left);
    frame.render_widget(left_widget, chunks[0]);

    // Right side: Orchestration status | Connected Services
    let mut right_parts = Vec::new();

    // Orchestration status
    if let Some(status) = orchestration_status {
        let status_color = if status == "enabled" {
            theme.success // Green when enabled
        } else {
            theme.warning // Yellow when disabled
        };
        right_parts.push(Span::styled(
            format!("Orchestration: {}", status),
            Style::default().fg(status_color),
        ));
    }

    // Pipe divider (subtle gray)
    if !right_parts.is_empty() && !connected_services.is_empty() {
        right_parts.push(Span::styled(" | ", Style::default().fg(divider_color)));
    }

    // Connected Services
    if !connected_services.is_empty() {
        let services_text = format!("Connected Services: {}", connected_services.join(", "));
        right_parts.push(Span::styled(
            services_text,
            Style::default().fg(theme.text),
        ));
    }

    let right_widget = Paragraph::new(Line::from(right_parts))
        .alignment(Alignment::Right);
    frame.render_widget(right_widget, chunks[2]);

    // Draw bottom border in light purple on the last line of the title area
    let border_y = area.bottom().saturating_sub(1);
    if border_y >= area.y && border_y < area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = frame.buffer_mut().cell_mut((x, border_y)) {
                cell.set_char('─');
                cell.set_style(Style::default().fg(light_purple));
            }
        }
    }
}


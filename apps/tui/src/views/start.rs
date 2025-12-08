//! Start page view matching CodeMachine aesthetic.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::components::{help_row::render_help_row, logo::render_logo};
use crate::views::prompt::PromptData;

/// Renders the start page with logo, commands, and input prompt
pub fn render_start_page(frame: &mut Frame, area: Rect, prompt_data: &PromptData) {
    let theme = crate::theme::get_theme();
    
    // Create a centered layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10), // Top spacing
            Constraint::Length(4),      // Logo
            Constraint::Length(1),      // Runtime info
            Constraint::Length(1),      // Spacing
            Constraint::Length(1),      // /chat command
            Constraint::Length(1),      // /agents command
            Constraint::Length(1),      // /auth command
            Constraint::Length(1),      // Spacing
            Constraint::Length(1),      // Spec path instruction
            Constraint::Length(1),      // Spacing
            Constraint::Length(3),      // Input prompt
            Constraint::Percentage(20), // Bottom spacing
        ])
        .split(area);

    // Logo
    render_logo(frame, chunks[1]);

    // Runtime info line
    let version = env!("CARGO_PKG_VERSION");
    let runtime_text = format!("Rust Runtime Edition • v{}", version);
    let runtime_widget = Paragraph::new(runtime_text)
        .style(Style::default().fg(theme.text_muted))
        .alignment(Alignment::Center);
    frame.render_widget(runtime_widget, chunks[2]);

    // Command help rows
    let help_area = chunks[4];
    let help_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // /chat
            Constraint::Length(1), // /agents
            Constraint::Length(1), // /auth
        ])
        .split(help_area);

    render_help_row(frame, help_chunks[0], "chat", "Start chat with an agent");
    render_help_row(frame, help_chunks[1], "agents", "List all available agents");
    render_help_row(frame, help_chunks[2], "auth", "Authenticate with AI providers");

    // Instruction text
    let instruction_text = "Type naturally to use orchestration, or /help for all commands";
    let instruction_widget = Paragraph::new(instruction_text)
        .style(Style::default().fg(theme.text_muted))
        .alignment(Alignment::Center);
    frame.render_widget(instruction_widget, chunks[8]);

    // Input prompt - centered with responsive width
    let prompt_width = (area.width as f32 * 0.8) as u16;
    let prompt_width = prompt_width.min(100).max(50);
    let prompt_area = Rect {
        x: (area.width.saturating_sub(prompt_width)) / 2,
        y: chunks[10].y,
        width: prompt_width,
        height: chunks[10].height,
    };

    let placeholder = if prompt_data.input.is_empty() {
        "Type /help for commands or chat naturally"
    } else {
        ""
    };
    
    let prompt_text = if prompt_data.input.is_empty() {
        format!("{}_", placeholder)
    } else {
        format!("{}_", prompt_data.input)
    };

    let prompt_widget = Paragraph::new(prompt_text)
        .style(Style::default().fg(theme.purple))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_active))
        );
    frame.render_widget(prompt_widget, prompt_area);
}


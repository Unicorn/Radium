//! Start page view matching CodeMachine aesthetic.

use ratatui::{
    prelude::*,
    widgets::Paragraph,
};

use crate::components::{help_row::render_help_row, logo::render_logo};
use crate::views::prompt::PromptData;

/// Renders the start page with logo, commands, and input prompt
pub fn render_start_page(frame: &mut Frame, area: Rect, _prompt_data: &PromptData) {
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
            Constraint::Percentage(30), // Bottom spacing (input is in status footer)
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
    
    // Note: Input is now always in the status footer for consistency
}


//! Status footer component for displaying overall status and help text.

use crate::commands::DisplayContext;
use crate::state::{PrivacyState, WorkflowStatus};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Application mode for context-aware footer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Prompt,
    Workflow,
    Chat,
    History,
    Setup,
    Requirement,
}

impl AppMode {
    /// Returns display name for the mode.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Prompt => "Prompt",
            Self::Workflow => "Workflow",
            Self::Chat => "Chat",
            Self::History => "History",
            Self::Setup => "Setup",
            Self::Requirement => "Requirement",
        }
    }

    /// Returns keyboard shortcuts for the mode.
    pub fn shortcuts(&self) -> &'static str {
        match self {
            Self::Prompt => "[Ctrl+P] Privacy | [Ctrl+C] Quit | [?] Help",
            Self::Workflow => "[↑↓] Navigate | [Enter] Select | [Esc] Close | [Ctrl+C] Cancel",
            Self::Chat => "[Enter] Send | [↑↓] Scroll | [Esc] Back | [Ctrl+C] Quit",
            Self::History => "[↑↓] Navigate | [Enter] View | [Esc] Back | [Ctrl+C] Quit",
            Self::Setup => "[Enter] Continue | [Esc] Skip | [Ctrl+C] Quit",
            Self::Requirement => "[Ctrl+S] Checkpoint | [↑↓] Scroll | [Esc] Cancel | [Ctrl+C] Force Quit",
        }
    }
}

/// Status footer component
pub struct StatusFooter;

impl StatusFooter {
    /// Renders a context-aware status footer.
    pub fn render_context_aware(
        frame: &mut Frame,
        area: Rect,
        mode: AppMode,
        context: Option<&DisplayContext>,
        selection_info: Option<&str>,
        privacy_state: Option<&PrivacyState>,
    ) {
        let theme = crate::theme::get_theme();
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12), // Mode
                Constraint::Min(10),    // Selection info
                Constraint::Length(25), // Privacy indicator
                Constraint::Percentage(50), // Shortcuts
            ])
            .split(area);

        // Mode indicator
        let mode_text = format!("Mode: {}", mode.as_str());
        let mode_widget = Paragraph::new(mode_text)
            .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            );
        frame.render_widget(mode_widget, chunks[0]);

        // Selection/Context info
        let info_text = if let Some(info) = selection_info {
            info.to_string()
        } else if let Some(ctx) = context {
            match ctx {
                DisplayContext::Chat { agent_id, session_id } => {
                    format!("Agent: {} | Session: {}", agent_id, session_id)
                }
                DisplayContext::AgentList => "Select an agent".to_string(),
                DisplayContext::SessionList => "Select a session".to_string(),
                DisplayContext::ModelSelector => "Select a model".to_string(),
                DisplayContext::Dashboard => "Dashboard".to_string(),
                DisplayContext::Help => "Help".to_string(),
            }
        } else {
            String::new()
        };

        let info_widget = Paragraph::new(info_text)
            .style(Style::default().fg(theme.text_muted))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            );
        frame.render_widget(info_widget, chunks[1]);

        // Privacy indicator
        let privacy_text = if let Some(privacy) = privacy_state {
            if privacy.enabled {
                if privacy.redaction_count > 0 {
                    format!("Privacy: ON ({} redactions)", privacy.redaction_count)
                } else {
                    "Privacy: ON".to_string()
                }
            } else {
                "Privacy: OFF".to_string()
            }
        } else {
            "Privacy: OFF".to_string()
        };
        let privacy_color = privacy_state
            .map(|p| if p.enabled { Color::Green } else { Color::DarkGray })
            .unwrap_or(Color::DarkGray);
        let privacy_widget = Paragraph::new(privacy_text)
            .style(Style::default().fg(privacy_color).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            );
        frame.render_widget(privacy_widget, chunks[2]);

        // Keyboard shortcuts
        let shortcuts_text = mode.shortcuts();
        let shortcuts_widget = Paragraph::new(shortcuts_text)
            .style(Style::default().fg(theme.text_dim))
            .alignment(Alignment::Right)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            );
        frame.render_widget(shortcuts_widget, chunks[3]);
    }

    /// Renders the status footer (legacy method for backward compatibility).
    pub fn render(frame: &mut Frame, area: Rect, status: WorkflowStatus, status_message: &str) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        // Status
        let status_color = Self::status_color(status);
        let status_text = format!("Status: {}", status.as_str());

        let status_widget = Paragraph::new(status_text)
            .style(Style::default().fg(status_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(status_widget, chunks[0]);

        // Message/Help
        let message = Paragraph::new(status_message)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Help "));
        frame.render_widget(message, chunks[1]);
    }

    /// Returns the color for a workflow status.
    fn status_color(status: WorkflowStatus) -> Color {
        match status {
            WorkflowStatus::Idle => Color::Gray,
            WorkflowStatus::Running => Color::Blue,
            WorkflowStatus::Paused => Color::Yellow,
            WorkflowStatus::Completed => Color::Green,
            WorkflowStatus::Failed => Color::Red,
            WorkflowStatus::Cancelled => Color::DarkGray,
        }
    }

    /// Renders a compact status footer in a single line.
    pub fn render_compact(frame: &mut Frame, area: Rect, status: WorkflowStatus, elapsed: f64) {
        let status_color = Self::status_color(status);
        let status_text = format!(
            "{} | {:.1}s | [q] Quit [p] Pause [r] Resume [c] Cancel",
            status.as_str(),
            elapsed
        );

        let widget = Paragraph::new(status_text)
            .style(Style::default().fg(status_color))
            .block(Block::default().borders(Borders::ALL).title(" Status "));
        frame.render_widget(widget, area);
    }

    /// Renders an extended status footer with additional information.
    pub fn render_extended(
        frame: &mut Frame,
        area: Rect,
        status: WorkflowStatus,
        status_message: &str,
        elapsed: f64,
        step: usize,
        total_steps: usize,
    ) {
        Self::render_extended_with_provider(frame, area, status, status_message, elapsed, step, total_steps, None)
    }

    /// Renders a codemachine-style footer with branding, version, CWD, and template.
    pub fn render_codemachine_style(
        frame: &mut Frame,
        area: Rect,
        version: &str,
        cwd: Option<&str>,
        template_name: Option<&str>,
    ) {
        let theme = crate::theme::get_theme();
        
        // Background for entire footer
        let footer_bg = Paragraph::new("")
            .style(Style::default().bg(theme.bg_panel));
        frame.render_widget(footer_bg, area);
        
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),    // Left: Branding + Version + CWD
                Constraint::Length(25), // Right: Template
            ])
            .split(area);

        // Left side: Branding + Version + CWD
        let mut left_parts = vec![
            Span::styled("Radium", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::raw(format!(" v{}", version)),
        ];
        
        if let Some(cwd) = cwd {
            left_parts.push(Span::raw(" "));
            left_parts.push(Span::styled(cwd, Style::default().fg(theme.text_muted)));
        }
        
        let left_widget = Paragraph::new(Line::from(left_parts))
            .style(Style::default().bg(theme.bg_panel))
            .block(Block::default().borders(Borders::NONE).padding(ratatui::widgets::Padding::new(0, 1, 0, 1)));
        frame.render_widget(left_widget, chunks[0]);

        // Right side: Template
        if let Some(template) = template_name {
            let template_text = format!("Template: {}", template.to_uppercase());
            let right_widget = Paragraph::new(template_text)
                .style(Style::default().fg(theme.text_muted))
                .alignment(Alignment::Right)
                .block(Block::default().borders(Borders::NONE).padding(ratatui::widgets::Padding::new(0, 1, 0, 1)));
            frame.render_widget(right_widget, chunks[1]);
        } else {
            // Show "Template: DEFAULT" or similar
            let template_text = "Template: DEFAULT";
            let right_widget = Paragraph::new(template_text)
                .style(Style::default().fg(theme.text_muted))
                .alignment(Alignment::Right)
                .block(Block::default().borders(Borders::NONE).padding(ratatui::widgets::Padding::new(0, 1, 0, 1)));
            frame.render_widget(right_widget, chunks[1]);
        }
    }

    /// Renders an extended status footer with provider information.
    pub fn render_extended_with_provider(
        frame: &mut Frame,
        area: Rect,
        status: WorkflowStatus,
        status_message: &str,
        elapsed: f64,
        step: usize,
        total_steps: usize,
        provider: Option<&str>,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // Status
                Constraint::Percentage(20), // Step info
                Constraint::Percentage(20), // Time
                Constraint::Percentage(20), // Provider
                Constraint::Percentage(20), // Help
            ])
            .split(area);

        // Status
        let status_color = Self::status_color(status);
        let status_widget = Paragraph::new(status.as_str())
            .style(Style::default().fg(status_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Status "));
        frame.render_widget(status_widget, chunks[0]);

        // Step info
        let step_text = format!("{}/{}", step, total_steps);
        let step_widget = Paragraph::new(step_text)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Step "));
        frame.render_widget(step_widget, chunks[1]);

        // Time
        let time_text = format!("{:.1}s", elapsed);
        let time_widget = Paragraph::new(time_text)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Elapsed "));
        frame.render_widget(time_widget, chunks[2]);

        // Provider
        let provider_text = provider.unwrap_or("N/A");
        let provider_widget = Paragraph::new(provider_text)
            .style(Style::default().fg(Color::Magenta))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Provider "));
        frame.render_widget(provider_widget, chunks[3]);

        // Help
        let help_widget = Paragraph::new(status_message)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Keys "));
        frame.render_widget(help_widget, chunks[4]);
    }

    /// Renders the status bar with fixed input prompt.
    /// This is the new universal status bar that always includes the input prompt.
    /// When in Chat context, the TextArea is rendered in the split-pane view instead.
    pub fn render_with_input(
        frame: &mut Frame,
        area: Rect,
        input: &crate::components::textarea::TextArea,
        mode: AppMode,
        context: Option<&DisplayContext>,
        model_id: Option<&str>,
        privacy_state: Option<&PrivacyState>,
    ) {
        let theme = crate::theme::get_theme();
        
        // Check if we're in Chat context - if so, don't render TextArea in status bar
        let is_chat_context = matches!(context, Some(DisplayContext::Chat { .. }));
        
        // Adjust layout constraints based on whether we show the input
        let chunks = if is_chat_context {
            // In Chat context: context info, privacy, shortcuts (no input)
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(15),        // Context info
                    Constraint::Length(25),    // Privacy indicator
                    Constraint::Min(30),       // Keyboard shortcuts
                ])
                .split(area)
        } else {
            // Other contexts: context info, input prompt, privacy, shortcuts
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(15),        // Context info
                    Constraint::Fill(1),       // Input prompt (flexible, centered)
                    Constraint::Length(25),    // Privacy indicator
                    Constraint::Min(30),       // Keyboard shortcuts
                ])
                .split(area)
        };

        // Left: Context info with model
        let context_text = if let Some(ctx) = context {
            match ctx {
                DisplayContext::Chat { agent_id, session_id } => {
                    if let Some(model) = model_id {
                        format!("Agent: {} | Session: {} | Model: {}", agent_id, session_id, model)
                    } else {
                        format!("Agent: {} | Session: {}", agent_id, session_id)
                    }
                }
                DisplayContext::AgentList => {
                    if let Some(model) = model_id {
                        format!("Select an agent | Model: {}", model)
                    } else {
                        "Select an agent".to_string()
                    }
                }
                DisplayContext::SessionList => {
                    if let Some(model) = model_id {
                        format!("Select a session | Model: {}", model)
                    } else {
                        "Select a session".to_string()
                    }
                }
                DisplayContext::ModelSelector => "Select a model".to_string(),
                DisplayContext::Dashboard => {
                    if let Some(model) = model_id {
                        format!("Dashboard | Model: {}", model)
                    } else {
                        "Dashboard".to_string()
                    }
                }
                DisplayContext::Help => {
                    if let Some(model) = model_id {
                        format!("Help | Model: {}", model)
                    } else {
                        "Help".to_string()
                    }
                }
            }
        } else {
            if let Some(model) = model_id {
                format!("Mode: {} | Model: {}", mode.as_str(), model)
            } else {
                format!("Mode: {}", mode.as_str())
            }
        };

        let context_widget = Paragraph::new(context_text)
            .style(Style::default().fg(theme.text_muted))
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .padding(ratatui::widgets::Padding::new(0, 1, 0, 0)),
            );
        frame.render_widget(context_widget, chunks[0]);

        // Center: Input prompt (only if not in Chat context)
        let input_index = if is_chat_context { 0 } else { 1 };
        if !is_chat_context {
            // Render TextArea widget directly (it implements Widget)
            frame.render_widget(input.clone(), chunks[input_index]);
        }

        // Privacy indicator
        let privacy_index = if is_chat_context { 1 } else { 2 };
        let privacy_text = if let Some(privacy) = privacy_state {
            if privacy.enabled {
                if privacy.redaction_count > 0 {
                    format!("Privacy: ON ({})", privacy.redaction_count)
                } else {
                    "Privacy: ON".to_string()
                }
            } else {
                "Privacy: OFF".to_string()
            }
        } else {
            "Privacy: OFF".to_string()
        };
        let privacy_color = privacy_state
            .map(|p| if p.enabled { Color::Green } else { Color::DarkGray })
            .unwrap_or(Color::DarkGray);
        let privacy_widget = Paragraph::new(privacy_text)
            .style(Style::default().fg(privacy_color).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .padding(ratatui::widgets::Padding::new(0, 1, 0, 1)),
            );
        frame.render_widget(privacy_widget, chunks[privacy_index]);

        // Right: Keyboard shortcuts
        let shortcuts_text = mode.shortcuts();
        let shortcuts_index = if is_chat_context { 2 } else { 3 };
        let shortcuts_widget = Paragraph::new(shortcuts_text)
            .style(Style::default().fg(theme.text_dim))
            .alignment(Alignment::Right)
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .padding(ratatui::widgets::Padding::new(0, 0, 0, 1)),
            );
        frame.render_widget(shortcuts_widget, chunks[shortcuts_index]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_color() {
        assert_eq!(StatusFooter::status_color(WorkflowStatus::Running), Color::Blue);
        assert_eq!(StatusFooter::status_color(WorkflowStatus::Completed), Color::Green);
        assert_eq!(StatusFooter::status_color(WorkflowStatus::Failed), Color::Red);
        assert_eq!(StatusFooter::status_color(WorkflowStatus::Paused), Color::Yellow);
    }
}

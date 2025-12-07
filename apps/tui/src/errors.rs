//! Error formatting and display for the TUI.
//!
//! Provides user-friendly error messages with actionable guidance.

use crate::icons::Icons;
use crate::theme::THEME;
use ratatui::{
    prelude::*,
    text::Span,
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Error types that can occur in the TUI.
#[derive(Debug, Clone)]
pub enum TuiError {
    /// Authentication is required.
    AuthRequired {
        provider: String,
        message: String,
    },
    /// Model error occurred.
    ModelError {
        message: String,
        suggestion: Option<String>,
    },
    /// Agent not found.
    AgentNotFound {
        agent_id: String,
    },
    /// Session error.
    SessionError {
        message: String,
    },
    /// Generic error with custom message.
    Generic {
        title: String,
        message: String,
        suggestion: Option<String>,
    },
}

impl TuiError {
    /// Creates an auth required error.
    pub fn auth_required(provider: impl Into<String>) -> Self {
        let provider = provider.into();
        let message = format!(
            "No {} API key found. You need to authenticate before chatting with agents.",
            provider.to_uppercase()
        );
        Self::AuthRequired { provider, message }
    }

    /// Creates a model error.
    pub fn model_error(message: impl Into<String>) -> Self {
        Self::ModelError {
            message: message.into(),
            suggestion: Some("Check your API key or try a different model".to_string()),
        }
    }

    /// Creates an agent not found error.
    pub fn agent_not_found(agent_id: impl Into<String>) -> Self {
        Self::AgentNotFound {
            agent_id: agent_id.into(),
        }
    }

    /// Creates a session error.
    pub fn session_error(message: impl Into<String>) -> Self {
        Self::SessionError {
            message: message.into(),
        }
    }

    /// Creates a generic error.
    pub fn generic(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Generic {
            title: title.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Adds a suggestion to the error.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        match &mut self {
            Self::ModelError { suggestion: s, .. } | Self::Generic { suggestion: s, .. } => {
                *s = Some(suggestion.into());
            }
            _ => {}
        }
        self
    }

    /// Returns the error title.
    pub fn title(&self) -> String {
        match self {
            Self::AuthRequired { .. } => format!("{} Authentication Required", Icons::WARNING),
            Self::ModelError { .. } => format!("{} Model Error", Icons::ERROR),
            Self::AgentNotFound { .. } => format!("{} Agent Not Found", Icons::ERROR),
            Self::SessionError { .. } => format!("{} Session Error", Icons::ERROR),
            Self::Generic { title, .. } => format!("{} {}", Icons::ERROR, title),
        }
    }

    /// Returns the error message lines.
    pub fn message_lines(&self) -> Vec<String> {
        let mut lines = vec![];

        match self {
            Self::AuthRequired { message, provider } => {
                lines.push(message.clone());
                lines.push("".to_string());
                lines.push("Quick fix:".to_string());
                lines.push(format!("  rad auth login {}", provider.to_lowercase()));
                lines.push("".to_string());
                lines.push("Or set environment variable:".to_string());
                lines.push(format!(
                    "  export {}_API_KEY='your-key-here'",
                    provider.to_uppercase()
                ));
                lines.push("".to_string());
                lines.push("Press 'a' to authenticate now, or Esc to continue".to_string());
            }
            Self::ModelError { message, suggestion } => {
                lines.push(message.clone());
                if let Some(s) = suggestion {
                    lines.push("".to_string());
                    lines.push(format!("{} {}", Icons::INFO, s));
                }
            }
            Self::AgentNotFound { agent_id } => {
                lines.push(format!("Agent '{}' not found.", agent_id));
                lines.push("".to_string());
                lines.push("Available agents:".to_string());
                lines.push("  /agents - List all available agents".to_string());
            }
            Self::SessionError { message } => {
                lines.push(message.clone());
                lines.push("".to_string());
                lines.push("Try:".to_string());
                lines.push("  /sessions - View session history".to_string());
                lines.push("  /chat <agent> - Start a new chat".to_string());
            }
            Self::Generic { message, suggestion, .. } => {
                lines.push(message.clone());
                if let Some(s) = suggestion {
                    lines.push("".to_string());
                    lines.push(format!("{} {}", Icons::INFO, s));
                }
            }
        }

        lines
    }

    /// Renders the error as a widget.
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(THEME.error))
            .title(
                ratatui::widgets::block::Title::from(
                    Span::styled(
                        self.title(),
                        Style::default().fg(THEME.error).add_modifier(Modifier::BOLD)
                    )
                )
                .alignment(Alignment::Left),
            );

        let inner = block.inner(area);
        block.render(area, buf);

        let message_lines = self.message_lines();
        let text = message_lines.join("\n");

        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(THEME.text))
            .wrap(Wrap { trim: true });

        paragraph.render(inner, buf);
    }
}

/// Converts model errors to TUI errors.
impl From<radium_core::error::RadiumError> for TuiError {
    fn from(err: radium_core::error::RadiumError) -> Self {
        TuiError::model_error(format!("{}", err))
    }
}

/// Helper function to create an error box for rendering.
pub fn error_box(error: &TuiError) -> impl Widget + '_ {
    ErrorBox { error }
}

struct ErrorBox<'a> {
    error: &'a TuiError,
}

impl<'a> Widget for ErrorBox<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.error.render(area, buf);
    }
}

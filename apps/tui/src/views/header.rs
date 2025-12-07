//! Branded header component for Radium TUI.
//!
//! Always-visible header showing branding, session info, and auth status.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::icons::Icons;
use crate::theme::THEME;
use radium_core::auth::{CredentialStore, ProviderType};

/// Header information to display.
#[derive(Debug, Clone, Default)]
pub struct HeaderInfo {
    /// Current session ID
    pub session_id: Option<String>,
    /// Current agent ID
    pub agent_id: Option<String>,
}

impl HeaderInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_agent(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }
}

/// Render the branded header.
pub fn render_header(frame: &mut Frame, area: Rect, info: &HeaderInfo) {
    // Check auth status
    let (gemini_auth, openai_auth) = if let Ok(store) = CredentialStore::new() {
        (store.is_configured(ProviderType::Gemini), store.is_configured(ProviderType::OpenAI))
    } else {
        (false, false)
    };

    // Build header text
    let mut header_parts = vec![Span::styled(
        format!("{} Radium", Icons::ROCKET),
        Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD),
    )];

    // Add session info if available
    if let Some(session_id) = &info.session_id {
        header_parts.push(Span::raw(" | "));
        header_parts.push(Span::styled(
            format!("{} {}", Icons::SESSION, session_id),
            Style::default().fg(THEME.text_muted()),
        ));
    }

    // Add agent info if available
    if let Some(agent_id) = &info.agent_id {
        header_parts.push(Span::raw(" | "));
        header_parts.push(Span::styled(
            format!("{} {}", Icons::AGENT, agent_id),
            Style::default().fg(THEME.info()),
        ));
    }

    // Add detailed auth status showing which providers are connected
    header_parts.push(Span::raw(" | "));
    header_parts
        .push(Span::styled(format!("{} ", Icons::AUTH), Style::default().fg(THEME.text_muted())));

    // Gemini status
    if gemini_auth {
        header_parts.push(Span::styled("Gemini✓", Style::default().fg(THEME.success())));
    } else {
        header_parts.push(Span::styled("Gemini✗", Style::default().fg(THEME.text_dim())));
    }

    header_parts.push(Span::raw(" "));

    // OpenAI status
    if openai_auth {
        header_parts.push(Span::styled("OpenAI✓", Style::default().fg(THEME.success())));
    } else {
        header_parts.push(Span::styled("OpenAI✗", Style::default().fg(THEME.text_dim())));
    }

    // Warning if no auth at all
    if !gemini_auth && !openai_auth {
        header_parts.push(Span::raw(" "));
        header_parts.push(Span::styled("(Type /auth)", Style::default().fg(THEME.warning())));
    }

    let header_line = Line::from(header_parts);

    let header = Paragraph::new(header_line)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(THEME.border())),
        )
        .style(Style::default().bg(THEME.bg_panel()));

    frame.render_widget(header, area);
}

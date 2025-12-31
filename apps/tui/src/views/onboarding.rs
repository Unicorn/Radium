//! First-run onboarding wizard for new users.
//!
//! Provides a 4-step interactive guide:
//! 1. Welcome screen with ASCII art and overview
//! 2. API key setup and provider selection
//! 3. Agent selection and configuration
//! 4. First chat prompt

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::theme::THEME;
use crate::icons::Icons;

/// Onboarding wizard steps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingStep {
    Welcome,
    ApiKey,
    Agents,
    FirstChat,
}

impl OnboardingStep {
    /// Returns the next step in the wizard.
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Welcome => Some(Self::ApiKey),
            Self::ApiKey => Some(Self::Agents),
            Self::Agents => Some(Self::FirstChat),
            Self::FirstChat => None,
        }
    }

    /// Returns the previous step in the wizard.
    pub fn previous(&self) -> Option<Self> {
        match self {
            Self::Welcome => None,
            Self::ApiKey => Some(Self::Welcome),
            Self::Agents => Some(Self::ApiKey),
            Self::FirstChat => Some(Self::Agents),
        }
    }

    /// Returns the step number (1-indexed).
    pub fn step_number(&self) -> usize {
        match self {
            Self::Welcome => 1,
            Self::ApiKey => 2,
            Self::Agents => 3,
            Self::FirstChat => 4,
        }
    }

    /// Returns the step title.
    pub fn title(&self) -> &'static str {
        match self {
            Self::Welcome => "Welcome to Radium",
            Self::ApiKey => "API Configuration",
            Self::Agents => "Agent Selection",
            Self::FirstChat => "Ready to Start",
        }
    }
}

/// Onboarding view state
#[derive(Debug, Clone)]
pub struct OnboardingView {
    /// Current step in the wizard
    pub current_step: OnboardingStep,
    /// Selected provider for API key setup
    pub selected_provider: usize,
    /// Selected agent for first chat
    pub selected_agent: usize,
    /// Available providers
    pub providers: Vec<String>,
    /// Available agents
    pub agents: Vec<String>,
    /// Whether API key has been configured
    pub api_key_configured: bool,
}

impl OnboardingView {
    /// Creates a new onboarding view.
    pub fn new() -> Self {
        Self {
            current_step: OnboardingStep::Welcome,
            selected_provider: 0,
            selected_agent: 0,
            providers: vec![
                "Gemini (Google)".to_string(),
                "OpenAI (ChatGPT)".to_string(),
                "Anthropic (Claude)".to_string(),
            ],
            agents: vec![
                "chat-assistant (General purpose)".to_string(),
                "code-reviewer (Code analysis)".to_string(),
                "architect (System design)".to_string(),
            ],
            api_key_configured: false,
        }
    }

    /// Advances to the next step.
    pub fn next_step(&mut self) -> bool {
        if let Some(next) = self.current_step.next() {
            self.current_step = next;
            true
        } else {
            false
        }
    }

    /// Goes back to the previous step.
    pub fn previous_step(&mut self) -> bool {
        if let Some(prev) = self.current_step.previous() {
            self.current_step = prev;
            true
        } else {
            false
        }
    }

    /// Selects the next provider.
    pub fn next_provider(&mut self) {
        if self.selected_provider < self.providers.len() - 1 {
            self.selected_provider += 1;
        }
    }

    /// Selects the previous provider.
    pub fn previous_provider(&mut self) {
        if self.selected_provider > 0 {
            self.selected_provider -= 1;
        }
    }

    /// Selects the next agent.
    pub fn next_agent(&mut self) {
        if self.selected_agent < self.agents.len() - 1 {
            self.selected_agent += 1;
        }
    }

    /// Selects the previous agent.
    pub fn previous_agent(&mut self) {
        if self.selected_agent > 0 {
            self.selected_agent -= 1;
        }
    }

    /// Checks if the wizard is complete.
    pub fn is_complete(&self) -> bool {
        self.current_step == OnboardingStep::FirstChat
    }
}

impl Default for OnboardingView {
    fn default() -> Self {
        Self::new()
    }
}

/// Renders the onboarding wizard.
pub fn render_onboarding(frame: &mut Frame, area: Rect, view: &OnboardingView) {
    // Create layout with header, content, and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header (step indicator)
            Constraint::Min(10),    // Content area
            Constraint::Length(3),  // Footer (navigation hints)
        ])
        .split(area);

    // Render step indicator header
    render_step_indicator(frame, chunks[0], view.current_step);

    // Render current step content
    match view.current_step {
        OnboardingStep::Welcome => render_welcome_step(frame, chunks[1]),
        OnboardingStep::ApiKey => render_api_key_step(frame, chunks[1], view),
        OnboardingStep::Agents => render_agents_step(frame, chunks[1], view),
        OnboardingStep::FirstChat => render_first_chat_step(frame, chunks[1]),
    }

    // Render navigation footer
    render_navigation_footer(frame, chunks[2], view.current_step);
}

/// Renders the step indicator at the top.
fn render_step_indicator(frame: &mut Frame, area: Rect, step: OnboardingStep) {
    let step_num = step.step_number();
    let title = step.title();

    let text = format!("Step {}/4: {}", step_num, title);
    let widget = Paragraph::new(text)
        .style(Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border())),
        );

    frame.render_widget(widget, area);
}

/// Renders the welcome step.
fn render_welcome_step(frame: &mut Frame, area: Rect) {
    let ascii_logo = r#"
    ____            ___
   / __ \____ _____/ (_)_  ______ ___
  / /_/ / __ `/ __  / / / / / __ `__ \
 / _, _/ /_/ / /_/ / / /_/ / / / / / /
/_/ |_|\__,_/\__,_/_/\__,_/_/ /_/ /_/

"#;

    let welcome_text = vec![
        Line::from(vec![
            Span::styled(ascii_logo, Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Welcome to Radium TUI!", Style::default().fg(THEME.text()).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("Radium is an AI-powered terminal assistant that helps you with:"),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{} ", Icons::SUCCESS), Style::default().fg(THEME.success())),
            Span::raw("Code analysis and review"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{} ", Icons::SUCCESS), Style::default().fg(THEME.success())),
            Span::raw("System architecture design"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{} ", Icons::SUCCESS), Style::default().fg(THEME.success())),
            Span::raw("Task orchestration and automation"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{} ", Icons::SUCCESS), Style::default().fg(THEME.success())),
            Span::raw("Interactive chat with multiple AI providers"),
        ]),
        Line::from(""),
        Line::from("This wizard will guide you through the initial setup in 4 simple steps."),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(THEME.text_muted())),
            Span::styled("Enter", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)),
            Span::styled(" to continue or ", Style::default().fg(THEME.text_muted())),
            Span::styled("Esc", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)),
            Span::styled(" to skip setup.", Style::default().fg(THEME.text_muted())),
        ]),
    ];

    let widget = Paragraph::new(welcome_text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border())),
        );

    frame.render_widget(widget, area);
}

/// Renders the API key configuration step.
fn render_api_key_step(frame: &mut Frame, area: Rect, view: &OnboardingView) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Instructions
            Constraint::Min(8),     // Provider list
            Constraint::Length(4),  // Configuration hint
        ])
        .split(area);

    // Instructions
    let instructions = vec![
        Line::from(""),
        Line::from("Select an AI provider to configure:"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Use ", Style::default().fg(THEME.text_muted())),
            Span::styled("↑/↓", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)),
            Span::styled(" to navigate, ", Style::default().fg(THEME.text_muted())),
            Span::styled("Enter", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)),
            Span::styled(" to configure.", Style::default().fg(THEME.text_muted())),
        ]),
    ];

    let instructions_widget = Paragraph::new(instructions)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border())),
        );
    frame.render_widget(instructions_widget, chunks[0]);

    // Provider list
    let list_items: Vec<ListItem> = view.providers.iter().enumerate().map(|(i, provider)| {
        let is_selected = i == view.selected_provider;
        let icon = if is_selected { "▶" } else { " " };

        let line = Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{} ", icon), Style::default().fg(THEME.primary())),
            Span::styled(
                provider,
                if is_selected {
                    Style::default().fg(THEME.text()).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(THEME.text())
                },
            ),
        ]);

        ListItem::new(line)
    }).collect();

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border()))
                .title(" Available Providers "),
        )
        .highlight_style(Style::default().bg(THEME.bg_element()));

    frame.render_widget(list, chunks[1]);

    // Configuration hint
    let hint_text = if view.api_key_configured {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("{} ", Icons::SUCCESS), Style::default().fg(THEME.success())),
                Span::styled("API key configured!", Style::default().fg(THEME.success())),
            ]),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("{} ", Icons::INFO), Style::default().fg(THEME.warning())),
                Span::raw("You can configure your API key later in settings."),
            ]),
        ]
    };

    let hint_widget = Paragraph::new(hint_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border())),
        );
    frame.render_widget(hint_widget, chunks[2]);
}

/// Renders the agent selection step.
fn render_agents_step(frame: &mut Frame, area: Rect, view: &OnboardingView) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Instructions
            Constraint::Min(8),     // Agent list
        ])
        .split(area);

    // Instructions
    let instructions = vec![
        Line::from(""),
        Line::from("Choose a default agent for your first session:"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Use ", Style::default().fg(THEME.text_muted())),
            Span::styled("↑/↓", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)),
            Span::styled(" to navigate, ", Style::default().fg(THEME.text_muted())),
            Span::styled("Enter", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)),
            Span::styled(" to select.", Style::default().fg(THEME.text_muted())),
        ]),
    ];

    let instructions_widget = Paragraph::new(instructions)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border())),
        );
    frame.render_widget(instructions_widget, chunks[0]);

    // Agent list
    let list_items: Vec<ListItem> = view.agents.iter().enumerate().map(|(i, agent)| {
        let is_selected = i == view.selected_agent;
        let icon = if is_selected { "▶" } else { " " };

        let line = Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{} ", icon), Style::default().fg(THEME.primary())),
            Span::styled(format!("{} ", Icons::AGENT), Style::default().fg(THEME.primary())),
            Span::styled(
                agent,
                if is_selected {
                    Style::default().fg(THEME.text()).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(THEME.text())
                },
            ),
        ]);

        ListItem::new(line)
    }).collect();

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border()))
                .title(" Available Agents "),
        )
        .highlight_style(Style::default().bg(THEME.bg_element()));

    frame.render_widget(list, chunks[1]);
}

/// Renders the first chat step.
fn render_first_chat_step(frame: &mut Frame, area: Rect) {
    let completion_text = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{} ", Icons::SUCCESS), Style::default().fg(THEME.success()).add_modifier(Modifier::BOLD)),
            Span::styled("Setup Complete!", Style::default().fg(THEME.success()).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from("You're all set to start using Radium!"),
        Line::from(""),
        Line::from("Quick tips:"),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("• ", Style::default().fg(THEME.primary())),
            Span::raw("Type your message and press "),
            Span::styled("Enter", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)),
            Span::raw(" to send"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("• ", Style::default().fg(THEME.primary())),
            Span::raw("Press "),
            Span::styled("?", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)),
            Span::raw(" for help at any time"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("• ", Style::default().fg(THEME.primary())),
            Span::raw("Use "),
            Span::styled("Ctrl+C", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)),
            Span::raw(" to quit"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("• ", Style::default().fg(THEME.primary())),
            Span::raw("Check the hints bar at the bottom for available actions"),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(THEME.text_muted())),
            Span::styled("Enter", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)),
            Span::styled(" to start your first chat!", Style::default().fg(THEME.text_muted())),
        ]),
    ];

    let widget = Paragraph::new(completion_text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border())),
        );

    frame.render_widget(widget, area);
}

/// Renders the navigation footer.
fn render_navigation_footer(frame: &mut Frame, area: Rect, step: OnboardingStep) {
    let can_go_back = step.previous().is_some();
    let can_go_forward = step.next().is_some();

    let nav_hints = vec![
        Line::from(vec![
            if can_go_back {
                Span::styled("←/Backspace", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("←/Backspace", Style::default().fg(THEME.text_dim()))
            },
            Span::raw(" Back  "),
            if can_go_forward {
                Span::styled("Enter/→", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("Enter", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD))
            },
            Span::raw(" Next  "),
            Span::styled("Esc", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)),
            Span::raw(" Skip Onboarding"),
        ]),
    ];

    let widget = Paragraph::new(nav_hints)
        .alignment(Alignment::Center)
        .style(Style::default().fg(THEME.text()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border())),
        );

    frame.render_widget(widget, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onboarding_view_creation() {
        let view = OnboardingView::new();
        assert_eq!(view.current_step, OnboardingStep::Welcome);
        assert!(!view.is_complete());
    }

    #[test]
    fn test_step_navigation() {
        let mut view = OnboardingView::new();

        // Forward navigation
        assert!(view.next_step());
        assert_eq!(view.current_step, OnboardingStep::ApiKey);

        assert!(view.next_step());
        assert_eq!(view.current_step, OnboardingStep::Agents);

        assert!(view.next_step());
        assert_eq!(view.current_step, OnboardingStep::FirstChat);
        assert!(view.is_complete());

        // Can't go forward from last step
        assert!(!view.next_step());

        // Backward navigation
        assert!(view.previous_step());
        assert_eq!(view.current_step, OnboardingStep::Agents);

        assert!(view.previous_step());
        assert_eq!(view.current_step, OnboardingStep::ApiKey);

        assert!(view.previous_step());
        assert_eq!(view.current_step, OnboardingStep::Welcome);

        // Can't go back from first step
        assert!(!view.previous_step());
    }

    #[test]
    fn test_provider_selection() {
        let mut view = OnboardingView::new();
        assert_eq!(view.selected_provider, 0);

        view.next_provider();
        assert_eq!(view.selected_provider, 1);

        view.next_provider();
        assert_eq!(view.selected_provider, 2);

        // Can't go beyond last provider
        view.next_provider();
        assert_eq!(view.selected_provider, 2);

        view.previous_provider();
        assert_eq!(view.selected_provider, 1);

        view.previous_provider();
        assert_eq!(view.selected_provider, 0);

        // Can't go before first provider
        view.previous_provider();
        assert_eq!(view.selected_provider, 0);
    }

    #[test]
    fn test_agent_selection() {
        let mut view = OnboardingView::new();
        assert_eq!(view.selected_agent, 0);

        view.next_agent();
        assert_eq!(view.selected_agent, 1);

        view.next_agent();
        assert_eq!(view.selected_agent, 2);

        // Can't go beyond last agent
        view.next_agent();
        assert_eq!(view.selected_agent, 2);

        view.previous_agent();
        assert_eq!(view.selected_agent, 1);
    }

    #[test]
    fn test_step_properties() {
        assert_eq!(OnboardingStep::Welcome.step_number(), 1);
        assert_eq!(OnboardingStep::ApiKey.step_number(), 2);
        assert_eq!(OnboardingStep::Agents.step_number(), 3);
        assert_eq!(OnboardingStep::FirstChat.step_number(), 4);

        assert_eq!(OnboardingStep::Welcome.title(), "Welcome to Radium");
        assert_eq!(OnboardingStep::ApiKey.title(), "API Configuration");
    }
}

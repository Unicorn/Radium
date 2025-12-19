//! Fix Approval Modal Component
//!
//! Displays proposed fixes from the error triage agent.
//! Shows fix details and provides options to approve, reject, or view more details.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use radium_orchestrator::FixProposal;

/// Outcome of fix approval dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalAction {
    /// Approve the fix and apply it
    Approved,
    /// Reject the fix
    Rejected,
    /// View full details (toggles expanded view)
    ViewDetails,
    /// Cancel dialog without action
    Cancelled,
}

/// Fix approval modal state
#[derive(Debug)]
pub struct FixApprovalModal {
    /// Fix proposal to display
    pub proposal: FixProposal,
    /// Currently selected option index
    pub selected_index: usize,
    /// Available options
    options: Vec<ApprovalOption>,
    /// Whether details view is expanded
    pub details_expanded: bool,
    /// Channel to send the user's decision back
    pub response_tx: Option<tokio::sync::oneshot::Sender<ApprovalAction>>,
}

/// A selectable option in the approval modal
#[derive(Debug, Clone)]
struct ApprovalOption {
    /// Display text
    label: String,
    /// Short description
    description: String,
    /// Action when selected
    action: ApprovalAction,
}

impl FixApprovalModal {
    /// Creates a new fix approval modal
    pub fn new(
        proposal: FixProposal,
        response_tx: tokio::sync::oneshot::Sender<ApprovalAction>,
    ) -> Self {
        let options = vec![
            ApprovalOption {
                label: "Approve".to_string(),
                description: "Apply the proposed fix".to_string(),
                action: ApprovalAction::Approved,
            },
            ApprovalOption {
                label: "View Details".to_string(),
                description: "Show/hide full fix details".to_string(),
                action: ApprovalAction::ViewDetails,
            },
            ApprovalOption {
                label: "Reject".to_string(),
                description: "Decline the proposed fix".to_string(),
                action: ApprovalAction::Rejected,
            },
            ApprovalOption {
                label: "Cancel".to_string(),
                description: "Go back without any action".to_string(),
                action: ApprovalAction::Cancelled,
            },
        ];

        Self {
            proposal,
            selected_index: 0,
            options,
            details_expanded: false,
            response_tx: Some(response_tx),
        }
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        let max_index = self.options.len().saturating_sub(1);
        self.selected_index = (self.selected_index + 1).min(max_index);
    }

    /// Get the action of the currently selected option
    pub fn selected_action(&self) -> ApprovalAction {
        self.options
            .get(self.selected_index)
            .map(|opt| opt.action)
            .unwrap_or(ApprovalAction::Cancelled)
    }

    /// Toggle details view expansion
    pub fn toggle_details(&mut self) {
        self.details_expanded = !self.details_expanded;
    }
}

/// Renders the fix approval modal
pub fn render_fix_approval_modal(frame: &mut Frame, area: Rect, modal: &FixApprovalModal) {
    let theme = crate::theme::get_theme();

    // Calculate modal size (adaptive based on details expansion)
    let modal_width = 90;
    let modal_height = if modal.details_expanded { 28 } else { 20 };

    // Center modal
    let modal_area = Rect {
        x: (area.width.saturating_sub(modal_width)) / 2,
        y: (area.height.saturating_sub(modal_height)) / 2,
        width: modal_width.min(area.width),
        height: modal_height.min(area.height),
    };

    // Render backdrop
    let backdrop = Paragraph::new("")
        .style(Style::default().bg(Color::Black))
        .block(Block::default().style(Style::default().bg(Color::Black)));
    frame.render_widget(backdrop, area);

    // Split modal area
    let chunks = if modal.details_expanded {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),  // Title
                Constraint::Length(10), // Error summary
                Constraint::Min(10),    // Expanded details
                Constraint::Length(5),  // Options list
                Constraint::Length(2),  // Help text
            ])
            .split(modal_area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Title
                Constraint::Length(8), // Error summary
                Constraint::Min(5),    // Options list
                Constraint::Length(2), // Help text
            ])
            .split(modal_area)
    };

    // Title
    let confidence_indicator = if modal.proposal.confidence >= 0.9 {
        "🟢"
    } else if modal.proposal.confidence >= 0.7 {
        "🟡"
    } else {
        "🔴"
    };

    let title = Paragraph::new(format!("{}  Fix Proposal from {}", confidence_indicator, modal.proposal.agent_id))
        .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .padding(ratatui::widgets::Padding::new(1, 1, 0, 0)),
        );
    frame.render_widget(title, chunks[0]);

    // Error summary
    let error_lines = modal.proposal.error_context.error_lines.join("\n");
    let error_display = if error_lines.len() > 200 {
        format!("{}...", &error_lines[..197])
    } else {
        error_lines
    };

    let summary_text = format!(
        "Process: {}\nSeverity: {} | Confidence: {:.0}%\n\nError:\n{}\n\nProposed Fix:\n{}",
        modal.proposal.error_context.process_id,
        modal.proposal.error_context.severity,
        modal.proposal.confidence * 100.0,
        error_display,
        if modal.proposal.proposed_fix.len() > 150 {
            format!("{}...", &modal.proposal.proposed_fix[..147])
        } else {
            modal.proposal.proposed_fix.clone()
        }
    );

    let summary = Paragraph::new(summary_text)
        .style(Style::default().fg(theme.text))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::NONE)
                .padding(ratatui::widgets::Padding::new(2, 2, 0, 1)),
        );
    frame.render_widget(summary, chunks[1]);

    // Expanded details (if enabled)
    let options_chunk_idx = if modal.details_expanded {
        let details_text = format!(
            "Root Cause:\n{}\n\nImpact:\n{}\n\nRollback Strategy:\n{}\n\nFull Fix Details:\n{}",
            modal.proposal.root_cause,
            modal.proposal.impact,
            modal.proposal.rollback_strategy,
            modal.proposal.proposed_fix
        );

        let details = Paragraph::new(details_text)
            .style(Style::default().fg(theme.text))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(theme.border))
                    .padding(ratatui::widgets::Padding::new(2, 2, 1, 1)),
            );
        frame.render_widget(details, chunks[2]);
        3
    } else {
        2
    };

    // Options list
    let items: Vec<ListItem> = modal
        .options
        .iter()
        .enumerate()
        .map(|(idx, option)| {
            let is_selected = idx == modal.selected_index;
            let style = if is_selected {
                Style::default()
                    .fg(theme.bg_primary)
                    .bg(theme.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let prefix = if is_selected { "● " } else { "○ " };
            let content = format!("{}{} - {}", prefix, option.label, option.description);
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::NONE)
            .padding(ratatui::widgets::Padding::new(2, 2, 0, 1)),
    );
    frame.render_widget(list, chunks[options_chunk_idx]);

    // Help text
    let help_text = "↑/↓ Navigate • Enter to confirm • d to toggle details • Esc to cancel";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(theme.text_muted))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .padding(ratatui::widgets::Padding::new(1, 1, 0, 0)),
        );
    frame.render_widget(help, chunks[options_chunk_idx + 1]);

    // Render border around entire modal
    let border_color = if modal.proposal.confidence >= 0.8 {
        theme.success
    } else if modal.proposal.confidence >= 0.6 {
        theme.warning
    } else {
        theme.danger
    };

    let modal_border = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme.bg_panel));
    frame.render_widget(modal_border, modal_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use radium_orchestrator::{ErrorContext, ProposalStatus};
    use serde_json::json;
    use std::time::SystemTime;

    fn create_test_proposal() -> FixProposal {
        FixProposal {
            id: "test-proposal-123".to_string(),
            error_context: ErrorContext {
                process_id: "dev-server-456".to_string(),
                error_lines: vec![
                    "TypeError: Cannot read property 'id' of undefined".to_string(),
                    "    at UserProfile (src/components/UserProfile.tsx:42:18)".to_string(),
                ],
                severity: "high".to_string(),
                error_type: "type_error".to_string(),
                confidence: 0.85,
                suggested_agent: Some("typescript-agent".to_string()),
                metadata: json!({
                    "command": "npm",
                    "args": ["run", "dev"]
                }),
            },
            proposed_fix: "Add null check before accessing user.id".to_string(),
            agent_id: "typescript-agent".to_string(),
            confidence: 0.85,
            root_cause: "User object is undefined when component renders".to_string(),
            impact: "Low risk: Defensive programming that prevents crash".to_string(),
            rollback_strategy: "Revert file using checkpoint".to_string(),
            status: ProposalStatus::Pending,
            created_at: SystemTime::now(),
            approval_requested_at: None,
            decided_at: None,
        }
    }

    #[test]
    fn test_modal_creation() {
        let proposal = create_test_proposal();
        let (response_tx, _response_rx) = tokio::sync::oneshot::channel();
        let modal = FixApprovalModal::new(proposal, response_tx);

        assert_eq!(modal.selected_index, 0);
        assert_eq!(modal.options.len(), 4);
        assert_eq!(modal.selected_action(), ApprovalAction::Approved);
        assert!(!modal.details_expanded);
    }

    #[test]
    fn test_modal_navigation() {
        let proposal = create_test_proposal();
        let (response_tx, _response_rx) = tokio::sync::oneshot::channel();
        let mut modal = FixApprovalModal::new(proposal, response_tx);

        assert_eq!(modal.selected_index, 0);
        assert_eq!(modal.selected_action(), ApprovalAction::Approved);

        modal.move_down();
        assert_eq!(modal.selected_index, 1);
        assert_eq!(modal.selected_action(), ApprovalAction::ViewDetails);

        modal.move_down();
        assert_eq!(modal.selected_index, 2);
        assert_eq!(modal.selected_action(), ApprovalAction::Rejected);

        modal.move_down();
        assert_eq!(modal.selected_index, 3);
        assert_eq!(modal.selected_action(), ApprovalAction::Cancelled);

        modal.move_down(); // Should not go beyond
        assert_eq!(modal.selected_index, 3);

        modal.move_up();
        assert_eq!(modal.selected_index, 2);

        modal.move_up();
        modal.move_up();
        modal.move_up(); // Should not go below 0
        assert_eq!(modal.selected_index, 0);
    }

    #[test]
    fn test_details_toggle() {
        let proposal = create_test_proposal();
        let (response_tx, _response_rx) = tokio::sync::oneshot::channel();
        let mut modal = FixApprovalModal::new(proposal, response_tx);

        assert!(!modal.details_expanded);

        modal.toggle_details();
        assert!(modal.details_expanded);

        modal.toggle_details();
        assert!(!modal.details_expanded);
    }

    #[test]
    fn test_action_selection() {
        let proposal = create_test_proposal();
        let (response_tx, _response_rx) = tokio::sync::oneshot::channel();
        let mut modal = FixApprovalModal::new(proposal, response_tx);

        // Test each action
        assert_eq!(modal.selected_action(), ApprovalAction::Approved);

        modal.move_down();
        assert_eq!(modal.selected_action(), ApprovalAction::ViewDetails);

        modal.move_down();
        assert_eq!(modal.selected_action(), ApprovalAction::Rejected);

        modal.move_down();
        assert_eq!(modal.selected_action(), ApprovalAction::Cancelled);
    }
}

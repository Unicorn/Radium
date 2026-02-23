//! Feedback collection view for routing decisions (Phase 2 - REQ-246).
//!
//! Provides an interactive UI for users to provide feedback on agent routing decisions,
//! helping improve routing accuracy over time through continuous learning.

use crossterm::event::{KeyCode, KeyEvent};
use radium_orchestrator::routing::{FeedbackRating, RoutingFeedbackRecord};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Feedback collection state.
#[derive(Debug, Clone, PartialEq)]
pub enum FeedbackState {
    /// Not collecting feedback.
    Idle,
    /// Showing rating prompt (thumbs up/down/skip).
    RatingPrompt,
    /// Collecting optional comment.
    CommentInput,
    /// Feedback completed.
    Completed,
}

/// Routing decision information for feedback.
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    /// Task description that was routed.
    pub task_description: String,
    /// Agent that was selected.
    pub selected_agent: String,
    /// Routing method used.
    pub routing_method: String,
    /// Confidence score (0.0-1.0).
    pub confidence: f32,
    /// Alternative agents considered.
    pub alternatives: Vec<(String, f32)>,
    /// Whether execution succeeded.
    pub execution_success: bool,
}

/// Feedback collection view.
pub struct FeedbackView {
    /// Current state of feedback collection.
    state: FeedbackState,
    /// The routing decision being rated.
    decision: Option<RoutingDecision>,
    /// Current rating selection.
    rating: Option<FeedbackRating>,
    /// Comment input buffer.
    comment: String,
    /// Cursor position in comment input.
    cursor_pos: usize,
    /// Collected feedback record (if completed).
    feedback_record: Option<RoutingFeedbackRecord>,
}

impl FeedbackView {
    /// Create a new feedback view.
    pub fn new() -> Self {
        Self {
            state: FeedbackState::Idle,
            decision: None,
            rating: None,
            comment: String::new(),
            cursor_pos: 0,
            feedback_record: None,
        }
    }

    /// Start collecting feedback for a routing decision.
    pub fn start_feedback(&mut self, decision: RoutingDecision) {
        self.state = FeedbackState::RatingPrompt;
        self.decision = Some(decision);
        self.rating = None;
        self.comment.clear();
        self.cursor_pos = 0;
        self.feedback_record = None;
    }

    /// Check if feedback collection is active.
    pub fn is_active(&self) -> bool {
        self.state != FeedbackState::Idle && self.state != FeedbackState::Completed
    }

    /// Check if feedback collection is completed.
    pub fn is_completed(&self) -> bool {
        self.state == FeedbackState::Completed
    }

    /// Get the collected feedback record.
    pub fn get_feedback(&mut self) -> Option<RoutingFeedbackRecord> {
        self.feedback_record.take()
    }

    /// Reset to idle state.
    pub fn reset(&mut self) {
        self.state = FeedbackState::Idle;
        self.decision = None;
        self.rating = None;
        self.comment.clear();
        self.cursor_pos = 0;
        self.feedback_record = None;
    }

    /// Handle keyboard input.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.state {
            FeedbackState::RatingPrompt => self.handle_rating_input(key),
            FeedbackState::CommentInput => self.handle_comment_input(key),
            _ => false,
        }
    }

    fn handle_rating_input(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.rating = Some(FeedbackRating::Positive);
                self.state = FeedbackState::CommentInput;
                true
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.rating = Some(FeedbackRating::Negative);
                self.state = FeedbackState::CommentInput;
                true
            }
            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Esc => {
                // Skip feedback
                self.state = FeedbackState::Completed;
                true
            }
            _ => false,
        }
    }

    fn handle_comment_input(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Enter => {
                self.complete_feedback();
                true
            }
            KeyCode::Esc => {
                // Submit without comment
                self.complete_feedback();
                true
            }
            KeyCode::Char(c) => {
                self.comment.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                true
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.comment.remove(self.cursor_pos);
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.comment.len() {
                    self.comment.remove(self.cursor_pos);
                }
                true
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                true
            }
            KeyCode::Right => {
                if self.cursor_pos < self.comment.len() {
                    self.cursor_pos += 1;
                }
                true
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                true
            }
            KeyCode::End => {
                self.cursor_pos = self.comment.len();
                true
            }
            _ => false,
        }
    }

    fn complete_feedback(&mut self) {
        if let (Some(decision), Some(rating)) = (&self.decision, &self.rating) {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let comment = if self.comment.is_empty() {
                None
            } else {
                Some(self.comment.clone())
            };

            self.feedback_record = Some(RoutingFeedbackRecord {
                timestamp,
                task_description: decision.task_description.clone(),
                selected_agent: decision.selected_agent.clone(),
                confidence: decision.confidence,
                routing_method: decision.routing_method.clone(),
                user_feedback: rating.clone(),
                execution_success: decision.execution_success,
                retry_count: 0, // Will be set by caller if needed
                comment,
            });

            self.state = FeedbackState::Completed;
        }
    }

    /// Render the feedback view.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        match self.state {
            FeedbackState::RatingPrompt => self.render_rating_prompt(frame, area),
            FeedbackState::CommentInput => self.render_comment_input(frame, area),
            _ => {}
        }
    }

    fn render_rating_prompt(&self, frame: &mut Frame, area: Rect) {
        if let Some(decision) = &self.decision {
            // Create centered modal
            let modal_area = Self::centered_rect(60, 50, area);

            // Clear background
            frame.render_widget(Clear, modal_area);

            // Split into sections
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(5),    // Decision info
                    Constraint::Length(5), // Buttons
                ])
                .split(modal_area);

            // Title
            let title = Paragraph::new("Routing Feedback")
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(title, chunks[0]);

            // Decision info
            let info_text = vec![
                Line::from(vec![
                    Span::styled("Task: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&decision.task_description),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Selected Agent: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        &decision.selected_agent,
                        Style::default().fg(Color::Green),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Method: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&decision.routing_method),
                    Span::raw(format!(" (confidence: {:.0}%)", decision.confidence * 100.0)),
                ]),
                Line::from(vec![
                    Span::styled("Result: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        if decision.execution_success {
                            "Success"
                        } else {
                            "Failed"
                        },
                        if decision.execution_success {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default().fg(Color::Red)
                        },
                    ),
                ]),
            ];

            // Append alternatives section if any exist
            let mut info_text = info_text;
            if !decision.alternatives.is_empty() {
                info_text.push(Line::from(""));
                info_text.push(Line::from(Span::styled(
                    "Alternatives considered:",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                for (agent, score) in &decision.alternatives {
                    info_text.push(Line::from(vec![
                        Span::raw("  • "),
                        Span::styled(agent.as_str(), Style::default().fg(Color::Yellow)),
                        Span::raw(format!(" ({:.0}%)", score * 100.0)),
                    ]));
                }
            }
            let info_text = info_text;

            let info = Paragraph::new(info_text)
                .block(Block::default().borders(Borders::ALL).title("Decision Details"))
                .wrap(Wrap { trim: true });
            frame.render_widget(info, chunks[1]);

            // Buttons
            let button_text = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("[Y] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw("Correct routing   "),
                    Span::styled("[N] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::raw("Incorrect routing   "),
                    Span::styled("[S] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("Skip"),
                ]),
                Line::from(""),
            ];

            let buttons = Paragraph::new(button_text)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(buttons, chunks[2]);
        }
    }

    fn render_comment_input(&self, frame: &mut Frame, area: Rect) {
        // Create centered modal
        let modal_area = Self::centered_rect(60, 30, area);

        // Clear background
        frame.render_widget(Clear, modal_area);

        // Split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(3),    // Input
                Constraint::Length(3), // Instructions
            ])
            .split(modal_area);

        // Title
        let rating_text = match &self.rating {
            Some(FeedbackRating::Positive) => "👍 Positive Feedback",
            Some(FeedbackRating::Negative) => "👎 Negative Feedback",
            _ => "Feedback",
        };

        let title = Paragraph::new(rating_text)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Comment input
        let input_text = if self.comment.is_empty() {
            Text::from(Span::styled(
                "(optional comment...)",
                Style::default().fg(Color::DarkGray),
            ))
        } else {
            Text::from(&*self.comment)
        };

        let input = Paragraph::new(input_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Comment (optional)"),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(input, chunks[1]);

        // Instructions
        let instructions = Paragraph::new("Press Enter to submit, Esc to skip comment")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(instructions, chunks[2]);
    }

    /// Helper to create a centered rectangle.
    fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

impl Default for FeedbackView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feedback_view() {
        let view = FeedbackView::new();
        assert_eq!(view.state, FeedbackState::Idle);
        assert!(!view.is_active());
    }

    #[test]
    fn test_start_feedback() {
        let mut view = FeedbackView::new();
        let decision = RoutingDecision {
            task_description: "Test task".to_string(),
            selected_agent: "code-agent".to_string(),
            routing_method: "skill".to_string(),
            confidence: 0.9,
            alternatives: vec![],
            execution_success: true,
        };

        view.start_feedback(decision);
        assert!(view.is_active());
        assert_eq!(view.state, FeedbackState::RatingPrompt);
    }

    #[test]
    fn test_positive_rating() {
        let mut view = FeedbackView::new();
        let decision = RoutingDecision {
            task_description: "Test task".to_string(),
            selected_agent: "code-agent".to_string(),
            routing_method: "skill".to_string(),
            confidence: 0.9,
            alternatives: vec![],
            execution_success: true,
        };

        view.start_feedback(decision);

        // Press 'y' for positive
        let key = KeyEvent::from(KeyCode::Char('y'));
        view.handle_key(key);

        assert_eq!(view.state, FeedbackState::CommentInput);
        assert_eq!(view.rating, Some(FeedbackRating::Positive));
    }

    #[test]
    fn test_negative_rating() {
        let mut view = FeedbackView::new();
        let decision = RoutingDecision {
            task_description: "Test task".to_string(),
            selected_agent: "wrong-agent".to_string(),
            routing_method: "keyword".to_string(),
            confidence: 0.3,
            alternatives: vec![],
            execution_success: false,
        };

        view.start_feedback(decision);

        // Press 'n' for negative
        let key = KeyEvent::from(KeyCode::Char('n'));
        view.handle_key(key);

        assert_eq!(view.state, FeedbackState::CommentInput);
        assert_eq!(view.rating, Some(FeedbackRating::Negative));
    }

    #[test]
    fn test_skip_feedback() {
        let mut view = FeedbackView::new();
        let decision = RoutingDecision {
            task_description: "Test task".to_string(),
            selected_agent: "code-agent".to_string(),
            routing_method: "skill".to_string(),
            confidence: 0.9,
            alternatives: vec![],
            execution_success: true,
        };

        view.start_feedback(decision);

        // Press 's' to skip
        let key = KeyEvent::from(KeyCode::Char('s'));
        view.handle_key(key);

        assert_eq!(view.state, FeedbackState::Completed);
        assert!(view.is_completed());
    }

    #[test]
    fn test_comment_input() {
        let mut view = FeedbackView::new();
        let decision = RoutingDecision {
            task_description: "Test task".to_string(),
            selected_agent: "code-agent".to_string(),
            routing_method: "skill".to_string(),
            confidence: 0.9,
            alternatives: vec![],
            execution_success: true,
        };

        view.start_feedback(decision);
        view.handle_key(KeyEvent::from(KeyCode::Char('y')));

        // Type a comment
        view.handle_key(KeyEvent::from(KeyCode::Char('g')));
        view.handle_key(KeyEvent::from(KeyCode::Char('o')));
        view.handle_key(KeyEvent::from(KeyCode::Char('o')));
        view.handle_key(KeyEvent::from(KeyCode::Char('d')));

        assert_eq!(view.comment, "good");
        assert_eq!(view.cursor_pos, 4);

        // Submit
        view.handle_key(KeyEvent::from(KeyCode::Enter));

        assert!(view.is_completed());
        let feedback = view.get_feedback().unwrap();
        assert_eq!(feedback.user_feedback, FeedbackRating::Positive);
        assert_eq!(feedback.comment, Some("good".to_string()));
    }

    #[test]
    fn test_backspace_in_comment() {
        let mut view = FeedbackView::new();
        view.state = FeedbackState::CommentInput;
        view.comment = "test".to_string();
        view.cursor_pos = 4;

        view.handle_key(KeyEvent::from(KeyCode::Backspace));

        assert_eq!(view.comment, "tes");
        assert_eq!(view.cursor_pos, 3);
    }
}

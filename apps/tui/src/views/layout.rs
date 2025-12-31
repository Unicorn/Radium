//! Global layout structure for the TUI application.
//!
//! Provides a consistent four-tier layout:
//! - Title bar (fixed, height 2): Logo and metadata
//! - Main area (flexible): Content area that can be split
//! - Status bar (fixed, height 5): Agent info (1 line) + Input prompt (3 lines) + border (1 line)
//! - Hints bar (fixed, height 1): gitui-style contextual keyboard shortcuts

use ratatui::{
    prelude::*,
    layout::{Constraint, Layout, Rect},
};

/// Global layout structure for the TUI
pub struct GlobalLayout;

impl GlobalLayout {
    /// Creates the base four-tier vertical layout
    pub fn create(area: Rect) -> [Rect; 4] {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Title bar (2 lines: 1 for content, 1 for border)
                Constraint::Min(0),    // Main area (flexible)
                Constraint::Length(5), // Status bar (5 lines: 1 for agent info + 3 for input + 1 for border)
                Constraint::Length(1), // Hints bar (1 line: gitui-style contextual shortcuts)
            ])
            .split(area);
        [chunks[0], chunks[1], chunks[2], chunks[3]]
    }

    /// Splits the main area horizontally for split-panel views
    pub fn split_main_horizontal(main_area: Rect) -> [Rect; 2] {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1), // Left panel
                Constraint::Fill(1), // Right panel
            ])
            .split(main_area);
        [chunks[0], chunks[1]]
    }

    /// Splits the main area with custom constraints
    pub fn split_main_with_constraints(main_area: Rect, constraints: &[Constraint]) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(main_area)
            .to_vec()
    }
}


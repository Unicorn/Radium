//! Global layout structure for the TUI application.
//!
//! Provides a consistent three-tier layout:
//! - Title bar (fixed, height 1): Logo and metadata
//! - Main area (flexible): Content area that can be split
//! - Status bar (fixed, height 1): Input prompt and context

use ratatui::{
    prelude::*,
    layout::{Constraint, Layout, Rect},
};

/// Global layout structure for the TUI
pub struct GlobalLayout;

impl GlobalLayout {
    /// Creates the base three-tier vertical layout
    pub fn create(area: Rect) -> [Rect; 3] {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Title bar (2 lines: 1 for content, 1 for border)
                Constraint::Min(0),    // Main area (flexible)
                Constraint::Length(1),  // Status bar
            ])
            .split(area);
        [chunks[0], chunks[1], chunks[2]]
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


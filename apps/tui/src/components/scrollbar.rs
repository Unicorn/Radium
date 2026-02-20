//! Scrollbar indicator component for long lists.
//!
//! Provides visual feedback for scroll position in lists with viewport culling.

use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};

/// Scrollbar position information
#[derive(Debug, Clone, Copy)]
pub struct ScrollbarState {
    /// Total number of items
    pub total_items: usize,
    /// Currently visible start index
    pub visible_start: usize,
    /// Currently visible end index
    pub visible_end: usize,
    /// Height of the viewport
    pub viewport_height: usize,
}

impl ScrollbarState {
    /// Create a new scrollbar state
    pub fn new(total_items: usize, visible_start: usize, visible_end: usize, viewport_height: usize) -> Self {
        Self {
            total_items,
            visible_start,
            visible_end,
            viewport_height,
        }
    }

    /// Calculate scroll percentage (0-100)
    pub fn scroll_percentage(&self) -> u8 {
        if self.total_items <= self.viewport_height {
            100 // Showing everything
        } else {
            let scrollable_range = self.total_items.saturating_sub(self.viewport_height);
            if scrollable_range == 0 {
                100
            } else {
                ((self.visible_start as f64 / scrollable_range as f64) * 100.0).min(100.0) as u8
            }
        }
    }

    /// Check if scrolled to top
    pub fn is_at_top(&self) -> bool {
        self.visible_start == 0
    }

    /// Check if scrolled to bottom
    pub fn is_at_bottom(&self) -> bool {
        self.visible_end >= self.total_items
    }

    /// Get scrollbar position indicator character
    pub fn get_indicator(&self) -> &'static str {
        if self.total_items <= self.viewport_height {
            "━" // Full bar - showing everything
        } else if self.is_at_top() {
            "▀" // At top
        } else if self.is_at_bottom() {
            "▄" // At bottom
        } else {
            "█" // Middle
        }
    }
}

/// Render a simple scrollbar indicator in the given area
pub fn render_scrollbar_indicator(
    frame: &mut Frame,
    area: Rect,
    state: &ScrollbarState,
    theme_border_color: Color,
) {
    if state.total_items == 0 {
        return;
    }

    let percentage = state.scroll_percentage();
    let indicator = state.get_indicator();

    let text = if state.total_items <= state.viewport_height {
        format!("All {} items", state.total_items)
    } else {
        format!(
            "{} {}-{}/{} ({}%)",
            indicator,
            state.visible_start + 1,
            state.visible_end,
            state.total_items,
            percentage
        )
    };

    let scrollbar_widget = Paragraph::new(text)
        .style(Style::default().fg(theme_border_color))
        .alignment(Alignment::Right)
        .block(Block::default());

    frame.render_widget(scrollbar_widget, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_percentage_at_top() {
        let state = ScrollbarState::new(100, 0, 10, 10);
        assert_eq!(state.scroll_percentage(), 0);
        assert!(state.is_at_top());
        assert!(!state.is_at_bottom());
    }

    #[test]
    fn test_scroll_percentage_at_bottom() {
        let state = ScrollbarState::new(100, 90, 100, 10);
        assert_eq!(state.scroll_percentage(), 100);
        assert!(!state.is_at_top());
        assert!(state.is_at_bottom());
    }

    #[test]
    fn test_scroll_percentage_middle() {
        let state = ScrollbarState::new(100, 45, 55, 10);
        let percentage = state.scroll_percentage();
        assert!(percentage > 0 && percentage < 100);
        assert!(!state.is_at_top());
        assert!(!state.is_at_bottom());
    }

    #[test]
    fn test_all_visible() {
        let state = ScrollbarState::new(10, 0, 10, 10);
        assert_eq!(state.scroll_percentage(), 100);
    }

    #[test]
    fn test_indicator_characters() {
        let state_top = ScrollbarState::new(100, 0, 10, 10);
        assert_eq!(state_top.get_indicator(), "▀");

        let state_bottom = ScrollbarState::new(100, 90, 100, 10);
        assert_eq!(state_bottom.get_indicator(), "▄");

        let state_middle = ScrollbarState::new(100, 45, 55, 10);
        assert_eq!(state_middle.get_indicator(), "█");

        let state_all = ScrollbarState::new(10, 0, 10, 10);
        assert_eq!(state_all.get_indicator(), "━");
    }
}

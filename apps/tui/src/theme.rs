//! Color theme system for Radium TUI.
//!
//! Provides a vibrant, professional color scheme inspired by CodeMachine.

use ratatui::style::Color;

/// Radium TUI color theme.
#[derive(Debug, Clone)]
pub struct RadiumTheme {
    // Primary brand colors
    pub primary: Color,
    pub secondary: Color,

    // Status colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    // Text colors
    pub text: Color,
    pub text_muted: Color,
    pub text_dim: Color,

    // Background colors
    pub bg_primary: Color,
    pub bg_panel: Color,
    pub bg_element: Color,

    // Border colors
    pub border: Color,
    pub border_active: Color,
    pub border_subtle: Color,
}

impl Default for RadiumTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl RadiumTheme {
    /// Creates the default dark theme.
    pub fn dark() -> Self {
        Self {
            // Primary: Cyan (#00D9FF)
            primary: Color::Rgb(0, 217, 255),
            // Secondary: Purple (#A78BFA)
            secondary: Color::Rgb(167, 139, 250),

            // Status colors
            success: Color::Rgb(16, 185, 129), // Green
            warning: Color::Rgb(245, 158, 11), // Yellow
            error: Color::Rgb(239, 68, 68),    // Red
            info: Color::Rgb(6, 182, 212),     // Blue

            // Text colors
            text: Color::Rgb(238, 238, 238),       // White
            text_muted: Color::Rgb(128, 128, 128), // Gray
            text_dim: Color::Rgb(96, 96, 96),      // Dark Gray

            // Background colors
            bg_primary: Color::Rgb(24, 29, 39), // Dark blue-gray
            bg_panel: Color::Rgb(20, 20, 20),   // Very dark
            bg_element: Color::Rgb(30, 30, 30), // Dark gray

            // Border colors
            border: Color::Rgb(72, 72, 72),        // Medium gray
            border_active: Color::Rgb(96, 96, 96), // Lighter gray
            border_subtle: Color::Rgb(60, 60, 60), // Subtle gray
        }
    }

    /// Creates a light theme (for future use).
    #[allow(dead_code)]
    pub fn light() -> Self {
        Self {
            primary: Color::Rgb(8, 145, 178),
            secondary: Color::Rgb(124, 58, 237),

            success: Color::Rgb(5, 150, 105),
            warning: Color::Rgb(217, 119, 6),
            error: Color::Rgb(220, 38, 38),
            info: Color::Rgb(14, 116, 144),

            text: Color::Rgb(26, 26, 26),
            text_muted: Color::Rgb(107, 114, 128),
            text_dim: Color::Rgb(156, 163, 175),

            bg_primary: Color::Rgb(255, 255, 255),
            bg_panel: Color::Rgb(250, 250, 250),
            bg_element: Color::Rgb(245, 245, 245),

            border: Color::Rgb(184, 184, 184),
            border_active: Color::Rgb(160, 160, 160),
            border_subtle: Color::Rgb(212, 212, 212),
        }
    }
}

/// Global theme instance.
pub static THEME: RadiumTheme = RadiumTheme {
    primary: Color::Rgb(0, 217, 255),
    secondary: Color::Rgb(167, 139, 250),
    success: Color::Rgb(16, 185, 129),
    warning: Color::Rgb(245, 158, 11),
    error: Color::Rgb(239, 68, 68),
    info: Color::Rgb(6, 182, 212),
    text: Color::Rgb(238, 238, 238),
    text_muted: Color::Rgb(128, 128, 128),
    text_dim: Color::Rgb(96, 96, 96),
    bg_primary: Color::Rgb(24, 29, 39),
    bg_panel: Color::Rgb(20, 20, 20),
    bg_element: Color::Rgb(30, 30, 30),
    border: Color::Rgb(72, 72, 72),
    border_active: Color::Rgb(96, 96, 96),
    border_subtle: Color::Rgb(60, 60, 60),
};

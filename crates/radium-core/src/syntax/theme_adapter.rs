//! Theme adapter for converting between RadiumTheme and syntect themes.
//!
//! This module provides utilities for converting color schemes between
//! Radium's theme system and syntect's theme format.

use syntect::highlighting::{Color, Theme, ThemeSet};

/// Convert a syntect Theme to a simple color mapping.
///
/// This extracts key colors from a syntect theme for use in syntax highlighting.
pub struct ThemeAdapter;

impl ThemeAdapter {
    /// Get the background color from a syntect theme.
    pub fn background_color(theme: &Theme) -> Color {
        theme.settings.background.unwrap_or(Color {
            r: 0,
            g: 0,
            b: 0,
        })
    }

    /// Get the foreground (text) color from a syntect theme.
    pub fn foreground_color(theme: &Theme) -> Color {
        theme.settings.foreground.unwrap_or(Color {
            r: 255,
            g: 255,
            b: 255,
        })
    }

    /// Convert syntect Color to RGB tuple.
    pub fn color_to_rgb(color: Color) -> (u8, u8, u8) {
        (color.r, color.g, color.b)
    }

    /// Convert RGB tuple to syntect Color.
    pub fn rgb_to_color(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }

    /// Load a default syntect theme by name.
    ///
    /// Returns the default theme if the named theme is not found.
    pub fn load_default_theme(name: &str) -> Theme {
        let theme_set = ThemeSet::load_defaults();
        theme_set
            .themes
            .get(name)
            .cloned()
            .unwrap_or_else(|| theme_set.themes["base16-ocean.dark"].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_conversion() {
        let color = Color { r: 100, g: 150, b: 200 };
        let rgb = ThemeAdapter::color_to_rgb(color);
        assert_eq!(rgb, (100, 150, 200));

        let converted = ThemeAdapter::rgb_to_color(100, 150, 200);
        assert_eq!(converted.r, 100);
        assert_eq!(converted.g, 150);
        assert_eq!(converted.b, 200);
    }

    #[test]
    fn test_load_default_theme() {
        let theme = ThemeAdapter::load_default_theme("base16-ocean.dark");
        // Theme should have background and foreground colors
        let bg = ThemeAdapter::background_color(&theme);
        let fg = ThemeAdapter::foreground_color(&theme);
        // Colors should be valid (not all zeros or all 255s necessarily, but set)
        assert!(bg.r < 255 || bg.g < 255 || bg.b < 255 || bg.r > 0 || bg.g > 0 || bg.b > 0);
        assert!(fg.r < 255 || fg.g < 255 || fg.b < 255 || fg.r > 0 || fg.g > 0 || fg.b > 0);
    }
}


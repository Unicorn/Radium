//! TextMate .tmTheme file loader.
//!
//! Loads custom themes from TextMate .tmTheme files and converts them
//! to RadiumTheme format.

use anyhow::Result;
use std::path::Path;

/// Load a RadiumTheme from a TextMate .tmTheme file.
///
/// Maps tmTheme scopes to RadiumTheme color properties.
///
/// TODO: Implement proper tmTheme loading using plist crate.
/// The syntect 5.x API doesn't provide Theme::from_reader anymore.
/// We need to parse the .tmTheme plist file manually and construct a Theme.
pub fn load_tmtheme(_path: &Path) -> Result<RadiumTheme> {
    // For now, return a default theme as a placeholder
    // This allows the codebase to compile while we implement proper theme loading
    anyhow::bail!("Custom .tmTheme loading not yet implemented for syntect 5.x")
}

/// Convert a syntect Theme to RadiumTheme.
///
/// Maps tmTheme scopes to RadiumTheme colors using a simple mapping strategy.
///
/// TODO: This function will be used once proper tmTheme loading is implemented.
#[allow(dead_code)]
fn convert_theme(theme: &syntect::highlighting::Theme) -> Result<RadiumTheme> {
    use ratatui::style::Color;

    // Get base colors from theme settings
    let background = theme.settings.background.unwrap_or(syntect::highlighting::Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    });
    let foreground = theme.settings.foreground.unwrap_or(syntect::highlighting::Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    });

    // Try to extract colors from scope settings
    // This is a simplified mapping - a full implementation would parse all scopes
    let mut primary = Color::Rgb(0, 217, 255); // Default cyan
    let mut secondary = Color::Rgb(167, 139, 250); // Default purple
    let mut success = Color::Rgb(16, 185, 129); // Default green
    let warning = Color::Rgb(245, 158, 11); // Default yellow
    let mut error = Color::Rgb(239, 68, 68); // Default red
    let mut info = Color::Rgb(6, 182, 212); // Default blue

    // Try to find colors from common scopes
    for scope in &theme.scopes {
        let scope_name = format!("{:?}", scope.scope);

        // Only process if foreground color is present
        if let Some(color) = scope.style.foreground {
            // Map common scopes to theme colors
            if scope_name.contains("keyword") || scope_name.contains("storage") {
                primary = Color::Rgb(color.r, color.g, color.b);
            } else if scope_name.contains("string") {
                success = Color::Rgb(color.r, color.g, color.b);
            } else if scope_name.contains("comment") {
                // Comments typically use muted colors
            } else if scope_name.contains("invalid") || scope_name.contains("error") {
                error = Color::Rgb(color.r, color.g, color.b);
            } else if scope_name.contains("function") || scope_name.contains("entity") {
                info = Color::Rgb(color.r, color.g, color.b);
            } else if scope_name.contains("constant") || scope_name.contains("number") {
                secondary = Color::Rgb(color.r, color.g, color.b);
            }
        }
    }

    // Create RadiumTheme from extracted colors
    Ok(RadiumTheme {
        primary,
        secondary,
        purple: secondary, // Use secondary as purple
        success,
        warning,
        error,
        info,
        text: Color::Rgb(foreground.r, foreground.g, foreground.b),
        text_muted: Color::Rgb(
            (foreground.r as u16 * 2 / 3) as u8,
            (foreground.g as u16 * 2 / 3) as u8,
            (foreground.b as u16 * 2 / 3) as u8,
        ),
        text_dim: Color::Rgb(
            (foreground.r as u16 / 2) as u8,
            (foreground.g as u16 / 2) as u8,
            (foreground.b as u16 / 2) as u8,
        ),
        bg_primary: Color::Rgb(background.r, background.g, background.b),
        bg_panel: Color::Rgb(
            (background.r as u16 + 10).min(255) as u8,
            (background.g as u16 + 10).min(255) as u8,
            (background.b as u16 + 10).min(255) as u8,
        ),
        bg_element: Color::Rgb(
            (background.r as u16 + 20).min(255) as u8,
            (background.g as u16 + 20).min(255) as u8,
            (background.b as u16 + 20).min(255) as u8,
        ),
        border: Color::Rgb(
            ((background.r as u16 + foreground.r as u16) / 2) as u8,
            ((background.g as u16 + foreground.g as u16) / 2) as u8,
            ((background.b as u16 + foreground.b as u16) / 2) as u8,
        ),
        border_active: primary,
        border_subtle: Color::Rgb(
            ((background.r as u16 * 3 + foreground.r as u16) / 4) as u8,
            ((background.g as u16 * 3 + foreground.g as u16) / 4) as u8,
            ((background.b as u16 * 3 + foreground.b as u16) / 4) as u8,
        ),
    })
}

/// RadiumTheme structure (re-exported from TUI for use in core).
///
/// This is a simplified version that matches the TUI theme structure.
/// In a real implementation, this would be shared between crates.
#[derive(Debug, Clone)]
pub struct RadiumTheme {
    pub primary: ratatui::style::Color,
    pub secondary: ratatui::style::Color,
    pub purple: ratatui::style::Color,
    pub success: ratatui::style::Color,
    pub warning: ratatui::style::Color,
    pub error: ratatui::style::Color,
    pub info: ratatui::style::Color,
    pub text: ratatui::style::Color,
    pub text_muted: ratatui::style::Color,
    pub text_dim: ratatui::style::Color,
    pub bg_primary: ratatui::style::Color,
    pub bg_panel: ratatui::style::Color,
    pub bg_element: ratatui::style::Color,
    pub border: ratatui::style::Color,
    pub border_active: ratatui::style::Color,
    pub border_subtle: ratatui::style::Color,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_invalid_path() {
        let path = Path::new("/nonexistent/theme.tmTheme");
        let result = load_tmtheme(path);
        assert!(result.is_err());
    }

    // Note: Full integration test would require a sample .tmTheme file
    // This is left as a placeholder for future testing
}


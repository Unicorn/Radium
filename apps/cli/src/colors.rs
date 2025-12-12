//! Radium brand colors for CLI applications.
//!
//! Provides consistent brand color palette across all CLI commands with
//! cross-platform terminal support (16-color, 256-color, and truecolor).
//!
//! # Usage
//!
//! ```rust
//! use colored::Colorize;
//! use crate::colors::RadiumBrandColors;
//!
//! // Get brand colors
//! let colors = RadiumBrandColors::new();
//!
//! // Use with colored crate
//! println!("{}", "Primary text".color(colors.primary_color()));
//! println!("{}", "Success message".color(colors.success_color()));
//!
//! // Or use convenience methods
//! println!("{}", "Primary text".color(colors.primary()));
//! ```
//!
//! # Brand Colors
//!
//! - Primary: Cyan (#00D9FF) - RGB(0, 217, 255)
//! - Secondary: Purple (#A78BFA) - RGB(167, 139, 250)
//! - Purple Accent: (#6250d0) - RGB(98, 80, 208)
//! - Success: Green (#10B981) - RGB(16, 185, 129)
//! - Warning: Yellow (#F59E0B) - RGB(245, 158, 11)
//! - Error: Red (#EF4444) - RGB(239, 68, 68)
//! - Info: Blue (#06B6D4) - RGB(6, 182, 212)

use colored::Color;
use radium_core::terminal::{ColorSupport, TerminalCapabilities, color_conversion};

/// Radium brand color palette.
///
/// Provides brand colors with automatic terminal capability detection
/// and color space conversion for cross-platform compatibility.
pub struct RadiumBrandColors {
    /// Terminal color support level
    color_support: ColorSupport,
}

impl RadiumBrandColors {
    /// Create a new instance with automatic terminal capability detection.
    pub fn new() -> Self {
        Self {
            color_support: TerminalCapabilities::detect(),
        }
    }

    /// Create a new instance with explicit color support level.
    pub fn with_support(support: ColorSupport) -> Self {
        Self {
            color_support: support,
        }
    }

    /// Get the detected terminal color support level.
    pub fn color_support(&self) -> ColorSupport {
        self.color_support
    }

    // Brand color RGB values (matching TUI theme)
    /// Primary brand color RGB: Cyan (#00D9FF)
    pub const PRIMARY_RGB: (u8, u8, u8) = (0, 217, 255);
    /// Secondary brand color RGB: Purple (#A78BFA)
    pub const SECONDARY_RGB: (u8, u8, u8) = (167, 139, 250);
    /// Purple accent color RGB: (#6250d0)
    pub const PURPLE_RGB: (u8, u8, u8) = (98, 80, 208);
    /// Success color RGB: Green (#10B981)
    pub const SUCCESS_RGB: (u8, u8, u8) = (16, 185, 129);
    /// Warning color RGB: Yellow (#F59E0B)
    pub const WARNING_RGB: (u8, u8, u8) = (245, 158, 11);
    /// Error color RGB: Red (#EF4444)
    pub const ERROR_RGB: (u8, u8, u8) = (239, 68, 68);
    /// Info color RGB: Blue (#06B6D4)
    pub const INFO_RGB: (u8, u8, u8) = (6, 182, 212);

    /// Convert RGB to colored::Color based on terminal capabilities.
    fn rgb_to_color(&self, rgb: (u8, u8, u8)) -> Color {
        match self.color_support {
            ColorSupport::Truecolor => {
                Color::TrueColor {
                    r: rgb.0,
                    g: rgb.1,
                    b: rgb.2,
                }
            }
            ColorSupport::Color256 => {
                // `colored` doesn't expose an ANSI 256-color variant across all versions.
                // Prefer TrueColor here; terminals that only support 256 colors will typically
                // approximate this reasonably.
                Color::TrueColor {
                    r: rgb.0,
                    g: rgb.1,
                    b: rgb.2,
                }
            }
            ColorSupport::Color16 => {
                // For 16-color, use closest ANSI color
                // Primary cyan -> Cyan, Secondary purple -> Magenta, etc.
                let index = color_conversion::rgb_to_16(rgb.0, rgb.1, rgb.2);
                // Map to colored's 16-color palette
                // 0-7: standard, 8-15: bright
                if index < 8 {
                    match index {
                        0 => Color::Black,
                        1 => Color::Red,
                        2 => Color::Green,
                        3 => Color::Yellow,
                        4 => Color::Blue,
                        5 => Color::Magenta,
                        6 => Color::Cyan,
                        7 => Color::White,
                        _ => Color::Cyan, // fallback
                    }
                } else {
                    match index {
                        8 => Color::BrightBlack,
                        9 => Color::BrightRed,
                        10 => Color::BrightGreen,
                        11 => Color::BrightYellow,
                        12 => Color::BrightBlue,
                        13 => Color::BrightMagenta,
                        14 => Color::BrightCyan,
                        15 => Color::BrightWhite,
                        _ => Color::BrightCyan, // fallback
                    }
                }
            }
        }
    }

    /// Get primary brand color (cyan #00D9FF).
    pub fn primary(&self) -> Color {
        self.rgb_to_color(Self::PRIMARY_RGB)
    }

    /// Get secondary brand color (purple #A78BFA).
    pub fn secondary(&self) -> Color {
        self.rgb_to_color(Self::SECONDARY_RGB)
    }

    /// Get purple accent color (#6250d0).
    pub fn purple(&self) -> Color {
        self.rgb_to_color(Self::PURPLE_RGB)
    }

    /// Get success color (green #10B981).
    pub fn success(&self) -> Color {
        self.rgb_to_color(Self::SUCCESS_RGB)
    }

    /// Get warning color (yellow #F59E0B).
    pub fn warning(&self) -> Color {
        self.rgb_to_color(Self::WARNING_RGB)
    }

    /// Get error color (red #EF4444).
    pub fn error(&self) -> Color {
        self.rgb_to_color(Self::ERROR_RGB)
    }

    /// Get info color (blue #06B6D4).
    pub fn info(&self) -> Color {
        self.rgb_to_color(Self::INFO_RGB)
    }

    // Convenience aliases for common usage patterns
    /// Alias for primary() - for backward compatibility with .cyan() usage.
    pub fn primary_color(&self) -> Color {
        self.primary()
    }

    /// Alias for success() - for backward compatibility with .green() usage.
    pub fn success_color(&self) -> Color {
        self.success()
    }

    /// Alias for error() - for backward compatibility with .red() usage.
    pub fn error_color(&self) -> Color {
        self.error()
    }

    /// Alias for warning() - for backward compatibility with .yellow() usage.
    pub fn warning_color(&self) -> Color {
        self.warning()
    }

    /// Alias for info() - for backward compatibility with .blue() usage.
    pub fn info_color(&self) -> Color {
        self.info()
    }
}

impl Default for RadiumBrandColors {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common color operations with brand colors.
impl RadiumBrandColors {
    /// Apply primary brand color to a string (for use with colored crate).
    #[allow(dead_code)]
    pub fn color_primary(&self, text: &str) -> colored::ColoredString {
        use colored::Colorize;
        text.color(self.primary())
    }

    /// Apply success color to a string (for use with colored crate).
    #[allow(dead_code)]
    pub fn color_success(&self, text: &str) -> colored::ColoredString {
        use colored::Colorize;
        text.color(self.success())
    }

    /// Apply error color to a string (for use with colored crate).
    #[allow(dead_code)]
    pub fn color_error(&self, text: &str) -> colored::ColoredString {
        use colored::Colorize;
        text.color(self.error())
    }

    /// Apply warning color to a string (for use with colored crate).
    #[allow(dead_code)]
    pub fn color_warning(&self, text: &str) -> colored::ColoredString {
        use colored::Colorize;
        text.color(self.warning())
    }

    /// Apply info color to a string (for use with colored crate).
    #[allow(dead_code)]
    pub fn color_info(&self, text: &str) -> colored::ColoredString {
        use colored::Colorize;
        text.color(self.info())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brand_colors_creation() {
        let colors = RadiumBrandColors::new();
        // Should not panic
        let _ = colors.primary();
        let _ = colors.secondary();
        let _ = colors.success();
        let _ = colors.error();
    }

    #[test]
    fn test_color_support_detection() {
        let colors = RadiumBrandColors::new();
        let support = colors.color_support();
        // Should return a valid ColorSupport variant
        match support {
            ColorSupport::Color16
            | ColorSupport::Color256
            | ColorSupport::Truecolor => {}
        }
    }

    #[test]
    fn test_explicit_color_support() {
        let colors = RadiumBrandColors::with_support(ColorSupport::Truecolor);
        assert_eq!(colors.color_support(), ColorSupport::Truecolor);
    }

    #[test]
    fn test_color_conversion() {
        // Test that colors can be converted for different support levels
        let truecolor = RadiumBrandColors::with_support(ColorSupport::Truecolor);
        let color256 = RadiumBrandColors::with_support(ColorSupport::Color256);
        let color16 = RadiumBrandColors::with_support(ColorSupport::Color16);

        // All should produce valid colors
        let _ = truecolor.primary();
        let _ = color256.primary();
        let _ = color16.primary();
    }
}

//! Terminal capability detection.
//!
//! Detects terminal color support (16-color, 256-color, or truecolor)
//! by checking environment variables.

use once_cell::sync::OnceCell;
use std::env;

/// Terminal color support levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSupport {
    /// 16-color ANSI support
    Color16,
    /// 256-color support
    Color256,
    /// Truecolor (24-bit RGB) support
    Truecolor,
}

/// Terminal capabilities detector.
pub struct TerminalCapabilities;

static DETECTED_CAPABILITIES: OnceCell<ColorSupport> = OnceCell::new();

impl TerminalCapabilities {
    /// Detect terminal color capabilities.
    ///
    /// Checks environment variables to determine color support:
    /// - COLORTERM=truecolor or COLORTERM=24bit → Truecolor
    /// - TERM contains "256color" → Color256
    /// - Otherwise → Color16
    ///
    /// The result is cached after first detection.
    pub fn detect() -> ColorSupport {
        *DETECTED_CAPABILITIES.get_or_init(|| {
            // Check for truecolor support
            if let Ok(colorterm) = env::var("COLORTERM") {
                let colorterm_lower = colorterm.to_lowercase();
                if colorterm_lower == "truecolor" || colorterm_lower == "24bit" {
                    return ColorSupport::Truecolor;
                }
            }

            // Check for 256-color support
            if let Ok(term) = env::var("TERM") {
                let term_lower = term.to_lowercase();
                if term_lower.contains("256color") || term_lower.contains("256-color") {
                    return ColorSupport::Color256;
                }
            }

            // Default to 16-color
            ColorSupport::Color16
        })
    }

    /// Get the detected color support (cached).
    pub fn color_support() -> ColorSupport {
        Self::detect()
    }

    /// Check if terminal supports truecolor.
    pub fn supports_truecolor() -> bool {
        Self::detect() == ColorSupport::Truecolor
    }

    /// Check if terminal supports 256 colors.
    pub fn supports_256_colors() -> bool {
        matches!(Self::detect(), ColorSupport::Color256 | ColorSupport::Truecolor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_support_enum() {
        // Verify enum variants are distinct
        assert_ne!(ColorSupport::Color16, ColorSupport::Color256);
        assert_ne!(ColorSupport::Color256, ColorSupport::Truecolor);
        assert_ne!(ColorSupport::Color16, ColorSupport::Truecolor);
    }

    #[test]
    fn test_detection_does_not_panic() {
        // Should not panic regardless of environment
        let _ = TerminalCapabilities::detect();
    }
}


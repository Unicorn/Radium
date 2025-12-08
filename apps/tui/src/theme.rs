//! Color theme system for Radium TUI.
//!
//! Provides a vibrant, professional color scheme inspired by CodeMachine.

use anyhow::{Context, Result};
use ratatui::style::Color;

use crate::config::TuiConfig;

/// Radium TUI color theme.
#[derive(Debug, Clone)]
pub struct RadiumTheme {
    // Primary brand colors
    pub primary: Color,
    pub secondary: Color,
    pub purple: Color,

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
    /// Parse hex color string to RGB Color.
    /// 
    /// Accepts formats: "#RRGGBB" or "RRGGBB"
    fn parse_hex_color(hex: &str) -> Result<Color> {
        let hex = hex.trim().trim_start_matches('#');
        
        if hex.len() != 6 {
            return Err(anyhow::anyhow!("Invalid hex color length: {}", hex));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .with_context(|| format!("Invalid red component in hex color: {}", hex))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .with_context(|| format!("Invalid green component in hex color: {}", hex))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .with_context(|| format!("Invalid blue component in hex color: {}", hex))?;

        Ok(Color::Rgb(r, g, b))
    }

    /// Load theme from configuration.
    pub fn from_config() -> Self {
        match TuiConfig::load() {
            Ok(config) => {
                match config.theme.preset.as_str() {
                    "light" => Self::light(),
                    "github" => Self::github(),
                    "monokai" => Self::monokai(),
                    "onedark" => Self::onedark(),
                    "solarized-dark" => Self::solarized(),
                    "dracula" => Self::dracula(),
                    "custom" => {
                        if let Some(ref colors) = config.theme.colors {
                            Self::from_custom_colors(colors).unwrap_or_else(|e| {
                                eprintln!("Warning: Failed to load custom theme: {}. Using default.", e);
                                Self::dark()
                            })
                        } else {
                            eprintln!("Warning: Custom preset selected but no colors defined. Using default.");
                            Self::dark()
                        }
                    }
                    _ => Self::dark(), // Default to dark
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to load config: {}. Using default theme.", e);
                Self::dark()
            }
        }
    }

    /// Create theme from custom colors.
    fn from_custom_colors(colors: &crate::config::CustomColors) -> Result<Self> {
        let mut theme = Self::dark(); // Start with dark as base

        if let Some(ref hex) = colors.primary {
            theme.primary = Self::parse_hex_color(hex)?;
        }
        if let Some(ref hex) = colors.secondary {
            theme.secondary = Self::parse_hex_color(hex)?;
        }
        // Note: purple not in config yet, but can be added later
        if let Some(ref hex) = colors.success {
            theme.success = Self::parse_hex_color(hex)?;
        }
        if let Some(ref hex) = colors.warning {
            theme.warning = Self::parse_hex_color(hex)?;
        }
        if let Some(ref hex) = colors.error {
            theme.error = Self::parse_hex_color(hex)?;
        }
        if let Some(ref hex) = colors.info {
            theme.info = Self::parse_hex_color(hex)?;
        }
        if let Some(ref hex) = colors.text {
            theme.text = Self::parse_hex_color(hex)?;
        }
        if let Some(ref hex) = colors.text_muted {
            theme.text_muted = Self::parse_hex_color(hex)?;
        }
        if let Some(ref hex) = colors.text_dim {
            theme.text_dim = Self::parse_hex_color(hex)?;
        }
        if let Some(ref hex) = colors.bg_primary {
            theme.bg_primary = Self::parse_hex_color(hex)?;
        }
        if let Some(ref hex) = colors.bg_panel {
            theme.bg_panel = Self::parse_hex_color(hex)?;
        }
        if let Some(ref hex) = colors.bg_element {
            theme.bg_element = Self::parse_hex_color(hex)?;
        }
        if let Some(ref hex) = colors.border {
            theme.border = Self::parse_hex_color(hex)?;
        }
        if let Some(ref hex) = colors.border_active {
            theme.border_active = Self::parse_hex_color(hex)?;
        }
        if let Some(ref hex) = colors.border_subtle {
            theme.border_subtle = Self::parse_hex_color(hex)?;
        }

        Ok(theme)
    }

    /// Creates the default dark theme.
    pub fn dark() -> Self {
        Self {
            // Primary: Cyan (#00D9FF) - matches codemachine
            primary: Color::Rgb(0, 217, 255),
            // Secondary: Purple (#A78BFA)
            secondary: Color::Rgb(167, 139, 250),
            // Purple: (#6250d0) - for logo accents, matches codemachine
            purple: Color::Rgb(98, 80, 208),

            // Status colors
            success: Color::Rgb(16, 185, 129), // Green
            warning: Color::Rgb(245, 158, 11), // Yellow
            error: Color::Rgb(239, 68, 68),    // Red
            info: Color::Rgb(6, 182, 212),     // Blue

            // Text colors - matches codemachine
            text: Color::Rgb(238, 238, 238),       // #eeeeee (darkStep12)
            text_muted: Color::Rgb(128, 128, 128), // #808080 (darkStep11)
            text_dim: Color::Rgb(96, 96, 96),      // Dark Gray

            // Background colors - matches codemachine
            bg_primary: Color::Rgb(24, 29, 39), // #181D27 (darkStep1)
            bg_panel: Color::Rgb(20, 20, 20),   // #141414 (darkStep2)
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
            purple: Color::Rgb(98, 80, 208),

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

    /// Creates GitHub theme (light theme with blue accents).
    pub fn github() -> Self {
        Self {
            // Primary: GitHub blue (#0366d6)
            primary: Color::Rgb(3, 102, 214),
            // Secondary: Purple (#6f42c1)
            secondary: Color::Rgb(111, 66, 193),
            // Purple accent
            purple: Color::Rgb(111, 66, 193),

            // Status colors
            success: Color::Rgb(34, 134, 58),   // Green
            warning: Color::Rgb(251, 188, 5),   // Yellow
            error: Color::Rgb(179, 29, 40),      // Red
            info: Color::Rgb(3, 102, 214),      // Blue

            // Text colors
            text: Color::Rgb(36, 41, 46),       // Dark gray
            text_muted: Color::Rgb(88, 96, 105), // Medium gray
            text_dim: Color::Rgb(106, 115, 125), // Light gray

            // Background colors (light)
            bg_primary: Color::Rgb(255, 255, 255), // White
            bg_panel: Color::Rgb(246, 248, 250),   // Light gray
            bg_element: Color::Rgb(250, 251, 252),  // Very light gray

            // Border colors
            border: Color::Rgb(209, 213, 218),      // Light border
            border_active: Color::Rgb(3, 102, 214),  // Blue border
            border_subtle: Color::Rgb(225, 228, 232), // Subtle border
        }
    }

    /// Creates Monokai theme (classic dark theme with vibrant colors).
    pub fn monokai() -> Self {
        Self {
            // Primary: Monokai orange (#FD971F)
            primary: Color::Rgb(253, 151, 31),
            // Secondary: Monokai purple (#AE81FF)
            secondary: Color::Rgb(174, 129, 255),
            // Purple accent
            purple: Color::Rgb(174, 129, 255),

            // Status colors
            success: Color::Rgb(166, 226, 46),  // Green
            warning: Color::Rgb(253, 151, 31),  // Orange
            error: Color::Rgb(249, 38, 114),    // Pink/Red
            info: Color::Rgb(102, 217, 239),    // Cyan

            // Text colors
            text: Color::Rgb(248, 248, 242),    // Off-white
            text_muted: Color::Rgb(117, 113, 94), // Gray
            text_dim: Color::Rgb(73, 72, 62),   // Dark gray

            // Background colors (dark)
            bg_primary: Color::Rgb(39, 40, 34),  // Dark brown-gray
            bg_panel: Color::Rgb(46, 46, 40),    // Slightly lighter
            bg_element: Color::Rgb(55, 56, 48),  // Element background

            // Border colors
            border: Color::Rgb(73, 72, 62),      // Dark border
            border_active: Color::Rgb(253, 151, 31), // Orange border
            border_subtle: Color::Rgb(65, 64, 54),  // Subtle border
        }
    }

    /// Creates OneDark theme (Atom's dark theme with blue-green accents).
    pub fn onedark() -> Self {
        Self {
            // Primary: OneDark cyan (#56B6C2)
            primary: Color::Rgb(86, 182, 194),
            // Secondary: OneDark purple (#C678DD)
            secondary: Color::Rgb(198, 120, 221),
            // Purple accent
            purple: Color::Rgb(198, 120, 221),

            // Status colors
            success: Color::Rgb(152, 195, 121),  // Green
            warning: Color::Rgb(229, 192, 123),  // Yellow
            error: Color::Rgb(224, 108, 117),    // Red
            info: Color::Rgb(97, 175, 239),      // Blue

            // Text colors
            text: Color::Rgb(171, 178, 191),     // Light gray
            text_muted: Color::Rgb(101, 108, 124), // Medium gray
            text_dim: Color::Rgb(92, 99, 112),   // Dark gray

            // Background colors (dark)
            bg_primary: Color::Rgb(40, 44, 52),   // Dark blue-gray
            bg_panel: Color::Rgb(35, 38, 46),     // Darker
            bg_element: Color::Rgb(50, 54, 62),   // Element background

            // Border colors
            border: Color::Rgb(92, 99, 112),      // Gray border
            border_active: Color::Rgb(86, 182, 194), // Cyan border
            border_subtle: Color::Rgb(75, 81, 94),   // Subtle border
        }
    }

    /// Creates Solarized Dark theme (Ethan Schoonover's balanced palette).
    pub fn solarized() -> Self {
        Self {
            // Primary: Solarized cyan (#2AA198)
            primary: Color::Rgb(42, 161, 152),
            // Secondary: Solarized violet (#6C71C4)
            secondary: Color::Rgb(108, 113, 196),
            // Purple accent
            purple: Color::Rgb(108, 113, 196),

            // Status colors
            success: Color::Rgb(133, 153, 0),     // Green
            warning: Color::Rgb(181, 137, 0),    // Yellow
            error: Color::Rgb(220, 50, 47),      // Red
            info: Color::Rgb(38, 139, 210),      // Blue

            // Text colors
            text: Color::Rgb(131, 148, 150),     // Base0
            text_muted: Color::Rgb(88, 110, 117), // Base1
            text_dim: Color::Rgb(101, 123, 131),  // Base00

            // Background colors (dark)
            bg_primary: Color::Rgb(0, 43, 54),    // Base03 (darkest)
            bg_panel: Color::Rgb(7, 54, 66),      // Base02
            bg_element: Color::Rgb(0, 43, 54),    // Base03

            // Border colors
            border: Color::Rgb(7, 54, 66),        // Base02
            border_active: Color::Rgb(42, 161, 152), // Cyan
            border_subtle: Color::Rgb(0, 43, 54),   // Base03
        }
    }

    /// Creates Dracula theme (purple/pink dark theme).
    pub fn dracula() -> Self {
        Self {
            // Primary: Dracula cyan (#8BE9FD)
            primary: Color::Rgb(139, 233, 253),
            // Secondary: Dracula purple (#BD93F9)
            secondary: Color::Rgb(189, 147, 249),
            // Purple accent
            purple: Color::Rgb(189, 147, 249),

            // Status colors
            success: Color::Rgb(80, 250, 123),    // Green
            warning: Color::Rgb(255, 184, 108),  // Orange
            error: Color::Rgb(255, 85, 85),      // Red
            info: Color::Rgb(139, 233, 253),    // Cyan

            // Text colors
            text: Color::Rgb(248, 248, 242),      // Foreground
            text_muted: Color::Rgb(189, 147, 249), // Purple
            text_dim: Color::Rgb(139, 233, 253),  // Cyan

            // Background colors (dark)
            bg_primary: Color::Rgb(40, 42, 54),   // Background
            bg_panel: Color::Rgb(68, 71, 90),    // Selection
            bg_element: Color::Rgb(50, 52, 68),   // Current line

            // Border colors
            border: Color::Rgb(68, 71, 90),       // Selection
            border_active: Color::Rgb(189, 147, 249), // Purple
            border_subtle: Color::Rgb(50, 52, 68),   // Current line
        }
    }
}

use std::sync::{Mutex, OnceLock};

/// Global theme instance (loads from config on first access).
static THEME_INSTANCE: OnceLock<Mutex<RadiumTheme>> = OnceLock::new();

fn get_theme_instance() -> &'static Mutex<RadiumTheme> {
    THEME_INSTANCE.get_or_init(|| Mutex::new(RadiumTheme::from_config()))
}

/// Get the current theme (thread-safe).
pub fn get_theme() -> RadiumTheme {
    get_theme_instance().lock().unwrap().clone()
}

/// Update the global theme (for /reload-config).
pub fn update_theme(theme: RadiumTheme) {
    *get_theme_instance().lock().unwrap() = theme;
}

/// Global theme accessor (for backward compatibility with existing code).
/// 
/// This provides a struct that views can use to access theme colors.
/// It loads from config on first access and updates when config is reloaded.
pub struct ThemeAccessor;

impl ThemeAccessor {
    fn theme() -> RadiumTheme {
        get_theme()
    }
}

impl std::ops::Deref for ThemeAccessor {
    type Target = RadiumTheme;
    
    fn deref(&self) -> &Self::Target {
        // This is a bit of a hack - we return a reference to a static that we update
        // For now, we'll use a thread-local or just clone on access
        // Actually, let's use a different approach - make THEME a function that returns the theme
        panic!("THEME should not be dereferenced directly. Use THEME.primary, etc. which are implemented via methods");
    }
}

// Implement field access via methods for backward compatibility
impl ThemeAccessor {
    /// Primary color
    pub fn primary(&self) -> Color {
        Self::theme().primary
    }
    /// Secondary color
    pub fn secondary(&self) -> Color {
        Self::theme().secondary
    }
    /// Purple color
    pub fn purple(&self) -> Color {
        Self::theme().purple
    }
    /// Success color
    pub fn success(&self) -> Color {
        Self::theme().success
    }
    /// Warning color
    pub fn warning(&self) -> Color {
        Self::theme().warning
    }
    /// Error color
    pub fn error(&self) -> Color {
        Self::theme().error
    }
    /// Info color
    pub fn info(&self) -> Color {
        Self::theme().info
    }
    /// Text color
    pub fn text(&self) -> Color {
        Self::theme().text
    }
    /// Muted text color
    pub fn text_muted(&self) -> Color {
        Self::theme().text_muted
    }
    /// Dim text color
    pub fn text_dim(&self) -> Color {
        Self::theme().text_dim
    }
    /// Primary background color
    pub fn bg_primary(&self) -> Color {
        Self::theme().bg_primary
    }
    /// Panel background color
    pub fn bg_panel(&self) -> Color {
        Self::theme().bg_panel
    }
    /// Element background color
    pub fn bg_element(&self) -> Color {
        Self::theme().bg_element
    }
    /// Border color
    pub fn border(&self) -> Color {
        Self::theme().border
    }
    /// Active border color
    pub fn border_active(&self) -> Color {
        Self::theme().border_active
    }
    /// Subtle border color
    pub fn border_subtle(&self) -> Color {
        Self::theme().border_subtle
    }
    /// Color for user-typed text/input
    pub fn user_input_color(&self) -> Color {
        Self::theme().primary
    }
    /// Color for AI/agent responses
    pub fn machine_output_color(&self) -> Color {
        Self::theme().info
    }
    /// Color for system messages/logs
    pub fn system_message_color(&self) -> Color {
        Self::theme().text_muted
    }
    /// Color for prompts/questions
    pub fn question_color(&self) -> Color {
        Self::theme().warning
    }
}

/// Global theme constant for backward compatibility.
/// 
/// Access theme colors via THEME.primary, THEME.bg_primary, etc.
/// The theme loads from config on first access.
pub static THEME: ThemeAccessor = ThemeAccessor;

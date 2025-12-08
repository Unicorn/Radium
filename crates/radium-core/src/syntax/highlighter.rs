//! Syntax highlighter using syntect.
//!
//! Provides a simple API for highlighting code blocks with language-specific
//! syntax coloring.

use crate::syntax::languages::LanguageRegistry;
use crate::syntax::theme_adapter::ThemeAdapter;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// A styled span of text with color information.
#[derive(Debug, Clone)]
pub struct StyledSpan {
    /// The text content
    pub text: String,
    /// RGB color for foreground (text)
    pub foreground: (u8, u8, u8),
    /// RGB color for background (optional)
    pub background: Option<(u8, u8, u8)>,
    /// Whether text should be bold
    pub bold: bool,
    /// Whether text should be italic
    pub italic: bool,
    /// Whether text should be underlined
    pub underline: bool,
}

/// A line of styled text.
#[derive(Debug, Clone)]
pub struct StyledLine {
    /// The styled spans that make up this line
    pub spans: Vec<StyledSpan>,
}

/// Syntax highlighter for code blocks.
///
/// Uses syntect to provide language-specific syntax highlighting.
pub struct SyntaxHighlighter {
    language_registry: LanguageRegistry,
    default_theme: Theme,
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter with default settings.
    pub fn new() -> Self {
        let language_registry = LanguageRegistry::new();
        let theme_set = ThemeSet::load_defaults();
        // Use a popular dark theme as default
        let default_theme = theme_set
            .themes
            .get("base16-ocean.dark")
            .cloned()
            .unwrap_or_else(|| {
                // Fallback to first available theme
                theme_set.themes.values().next().unwrap().clone()
            });

        Self {
            language_registry,
            default_theme,
        }
    }

    /// Create a syntax highlighter with a custom theme.
    pub fn with_theme(theme_name: &str) -> Self {
        let language_registry = LanguageRegistry::new();
        let default_theme = ThemeAdapter::load_default_theme(theme_name);

        Self {
            language_registry,
            default_theme,
        }
    }

    /// Highlight code with the specified language.
    ///
    /// Returns a vector of styled lines. If the language is not supported,
    /// returns the code as plain text with default styling.
    pub fn highlight_code(&self, code: &str, language: &str) -> Vec<StyledLine> {
        let syntax = match self.language_registry.find_syntax(language) {
            Some(s) => s,
            None => {
                // Unknown language - return as plain text
                return self.plain_text_lines(code);
            }
        };

        let syntax_set = self.language_registry.syntax_set();
        let mut highlighter = HighlightLines::new(syntax, &self.default_theme);

        let mut styled_lines = Vec::new();

        for line in LinesWithEndings::from(code) {
            let mut spans = Vec::new();

            match highlighter.highlight_line(line, syntax_set) {
                Ok(highlighted) => {
                    for (style, text) in highlighted {
                        spans.push(self.style_to_span(style, text));
                    }
                }
                Err(_) => {
                    // If highlighting fails, use plain text
                    spans.push(StyledSpan {
                        text: line.to_string(),
                        foreground: ThemeAdapter::color_to_rgb(
                            ThemeAdapter::foreground_color(&self.default_theme),
                        ),
                        background: None,
                        bold: false,
                        italic: false,
                        underline: false,
                    });
                }
            }

            styled_lines.push(StyledLine { spans });
        }

        styled_lines
    }

    /// Convert syntect Style to StyledSpan.
    fn style_to_span(&self, style: Style, text: &str) -> StyledSpan {
        let fg = ThemeAdapter::color_to_rgb(style.foreground);
        let bg = if style.background.alpha > 0 {
            Some(ThemeAdapter::color_to_rgb(style.background))
        } else {
            None
        };

        StyledSpan {
            text: text.to_string(),
            foreground: fg,
            background: bg,
            bold: style.font_style.contains(syntect::highlighting::FontStyle::BOLD),
            italic: style.font_style.contains(syntect::highlighting::FontStyle::ITALIC),
            underline: style.font_style.contains(syntect::highlighting::FontStyle::UNDERLINE),
        }
    }

    /// Convert plain text to styled lines (fallback for unknown languages).
    fn plain_text_lines(&self, code: &str) -> Vec<StyledLine> {
        let fg = ThemeAdapter::color_to_rgb(ThemeAdapter::foreground_color(&self.default_theme));

        code.lines()
            .map(|line| {
                StyledLine {
                    spans: vec![StyledSpan {
                        text: line.to_string(),
                        foreground: fg,
                        background: None,
                        bold: false,
                        italic: false,
                        underline: false,
                    }],
                }
            })
            .collect()
    }

    /// Check if a language is supported.
    pub fn is_language_supported(&self, language: &str) -> bool {
        self.language_registry.is_supported(language)
    }

    /// Get list of supported languages.
    pub fn supported_languages(&self) -> Vec<String> {
        self.language_registry.supported_languages()
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_highlighting() {
        let highlighter = SyntaxHighlighter::new();
        let code = "fn main() {\n    println!(\"Hello, world!\");\n}";
        let lines = highlighter.highlight_code(code, "rust");

        assert!(!lines.is_empty());
        // Should have multiple spans with different colors
        assert!(lines[0].spans.len() > 0);
    }

    #[test]
    fn test_unknown_language() {
        let highlighter = SyntaxHighlighter::new();
        let code = "some code here";
        let lines = highlighter.highlight_code(code, "unknown_lang");

        // Should return plain text, not empty
        assert!(!lines.is_empty());
        assert_eq!(lines[0].spans[0].text.trim(), "some code here");
    }

    #[test]
    fn test_empty_code() {
        let highlighter = SyntaxHighlighter::new();
        let lines = highlighter.highlight_code("", "rust");

        // Empty code should return at least one empty line
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_language_support_check() {
        let highlighter = SyntaxHighlighter::new();
        assert!(highlighter.is_language_supported("rust"));
        assert!(highlighter.is_language_supported("python"));
        assert!(!highlighter.is_language_supported("unknown_lang_xyz"));
    }
}


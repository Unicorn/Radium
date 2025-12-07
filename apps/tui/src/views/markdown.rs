//! Markdown rendering for agent responses.
//!
//! Provides basic markdown formatting for TUI display.

use ratatui::{
    prelude::*,
    text::{Line, Span},
};

use crate::theme::THEME;

/// Render markdown text into styled spans.
///
/// Supports:
/// - **bold** text
/// - *italic* text
/// - `code` inline code
/// - Code blocks (```code```)
/// - Lists (- item)
pub fn render_markdown(text: &str) -> Vec<Line<'_>> {
    let mut lines = Vec::new();
    let mut in_code_block = false;
    let mut code_block_lang = String::new();

    for line in text.lines() {
        // Handle code blocks
        if line.trim().starts_with("```") {
            if in_code_block {
                in_code_block = false;
                code_block_lang.clear();
                // Don't add the closing ``` line
            } else {
                in_code_block = true;
                // Extract language if present
                let lang = line.trim().strip_prefix("```").unwrap_or("");
                code_block_lang = lang.trim().to_string();
                // Add a line indicating code block start
                if !code_block_lang.is_empty() {
                    lines.push(Line::from(Span::styled(
                        format!("Code ({})", code_block_lang),
                        Style::default().fg(THEME.info()),
                    )));
                }
            }
            continue;
        }

        if in_code_block {
            // Render code block line
            lines.push(Line::from(Span::styled(
                format!("  {}", line),
                Style::default().fg(THEME.text()).bg(THEME.bg_element()).add_modifier(Modifier::ITALIC),
            )));
        } else {
            // Parse markdown in regular line
            lines.push(parse_markdown_line(line));
        }
    }

    lines
}

/// Parse a single line of markdown into styled spans.
fn parse_markdown_line(line: &str) -> Line<'_> {
    let mut spans = Vec::new();
    let mut chars = line.chars().peekable();
    let mut current_text = String::new();
    let mut state = ParseState::Normal;

    while let Some(ch) = chars.next() {
        match state {
            ParseState::Normal => {
                match ch {
                    '*' if chars.peek() == Some(&'*') => {
                        // **bold**
                        if !current_text.is_empty() {
                            spans.push(Span::styled(
                                current_text.clone(),
                                Style::default().fg(THEME.text()),
                            ));
                            current_text.clear();
                        }
                        chars.next(); // consume second *
                        state = ParseState::Bold;
                    }
                    '*' => {
                        // *italic*
                        if !current_text.is_empty() {
                            spans.push(Span::styled(
                                current_text.clone(),
                                Style::default().fg(THEME.text()),
                            ));
                            current_text.clear();
                        }
                        state = ParseState::Italic;
                    }
                    '`' => {
                        // `code`
                        if !current_text.is_empty() {
                            spans.push(Span::styled(
                                current_text.clone(),
                                Style::default().fg(THEME.text()),
                            ));
                            current_text.clear();
                        }
                        state = ParseState::Code;
                    }
                    '-' if current_text.trim().is_empty() && chars.peek() == Some(&' ') => {
                        // List item
                        if !current_text.is_empty() {
                            spans.push(Span::styled(
                                current_text.clone(),
                                Style::default().fg(THEME.text()),
                            ));
                            current_text.clear();
                        }
                        chars.next(); // consume space
                        spans.push(Span::styled("• ", Style::default().fg(THEME.primary())));
                    }
                    _ => {
                        current_text.push(ch);
                    }
                }
            }
            ParseState::Bold => {
                if ch == '*' && chars.peek() == Some(&'*') {
                    // End bold
                    if !current_text.is_empty() {
                        spans.push(Span::styled(
                            current_text.clone(),
                            Style::default().fg(THEME.text()).add_modifier(Modifier::BOLD),
                        ));
                        current_text.clear();
                    }
                    chars.next(); // consume second *
                    state = ParseState::Normal;
                } else {
                    current_text.push(ch);
                }
            }
            ParseState::Italic => {
                if ch == '*' {
                    // End italic
                    if !current_text.is_empty() {
                        spans.push(Span::styled(
                            current_text.clone(),
                            Style::default().fg(THEME.text()).add_modifier(Modifier::ITALIC),
                        ));
                        current_text.clear();
                    }
                    state = ParseState::Normal;
                } else {
                    current_text.push(ch);
                }
            }
            ParseState::Code => {
                if ch == '`' {
                    // End code
                    if !current_text.is_empty() {
                        spans.push(Span::styled(
                            current_text.clone(),
                            Style::default()
                                .fg(THEME.secondary())
                                .bg(THEME.bg_element())
                                .add_modifier(Modifier::ITALIC),
                        ));
                        current_text.clear();
                    }
                    state = ParseState::Normal;
                } else {
                    current_text.push(ch);
                }
            }
        }
    }

    // Add remaining text
    if !current_text.is_empty() {
        let style = match state {
            ParseState::Bold => Style::default().fg(THEME.text()).add_modifier(Modifier::BOLD),
            ParseState::Italic => Style::default().fg(THEME.text()).add_modifier(Modifier::ITALIC),
            ParseState::Code => Style::default()
                .fg(THEME.secondary())
                .bg(THEME.bg_element())
                .add_modifier(Modifier::ITALIC),
            ParseState::Normal => Style::default().fg(THEME.text()),
        };
        spans.push(Span::styled(current_text, style));
    }

    if spans.is_empty() { Line::from(Span::raw("")) } else { Line::from(spans) }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ParseState {
    Normal,
    Bold,
    Italic,
    Code,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bold_parsing() {
        let line = parse_markdown_line("This is **bold** text");
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_italic_parsing() {
        let line = parse_markdown_line("This is *italic* text");
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_code_parsing() {
        let line = parse_markdown_line("This is `code` text");
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_code_block() {
        let text = "```rust\nfn main() {}\n```";
        let lines = render_markdown(text);
        // Should have: code header, code line, and possibly empty line
        assert!(lines.len() >= 2);
    }
}

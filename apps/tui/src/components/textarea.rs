//! Simple multiline textarea widget for ratatui.
//!
//! Provides basic multiline text editing with cursor management.
//! Designed to work with ratatui 0.29 and crossterm 0.28.

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::Widget,
};

/// Simple multiline textarea widget.
#[derive(Debug, Clone)]
pub struct TextArea {
    /// Lines of text
    lines: Vec<String>,
    /// Cursor row (0-indexed)
    cursor_row: usize,
    /// Cursor column (0-indexed, in characters)
    cursor_col: usize,
}

impl Default for TextArea {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
        }
    }
}

impl TextArea {
    /// Create a new TextArea with the given lines.
    pub fn new(lines: Vec<String>) -> Self {
        let cursor_row = lines.len().saturating_sub(1);
        let cursor_col = lines
            .last()
            .map(|s| s.len())
            .unwrap_or(0);
        Self {
            lines: if lines.is_empty() {
                vec![String::new()]
            } else {
                lines
            },
            cursor_row,
            cursor_col,
        }
    }

    /// Get all lines as a vector of strings.
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Get the current text content as a single string with newlines.
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Clear all text.
    pub fn clear(&mut self) {
        self.lines = vec![String::new()];
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    /// Set the text content.
    pub fn set_text(&mut self, text: &str) {
        self.lines = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(|s| s.to_string()).collect()
        };
        self.cursor_row = self.lines.len().saturating_sub(1);
        self.cursor_col = self
            .lines
            .last()
            .map(|s| s.len())
            .unwrap_or(0);
    }

    /// Handle a key event.
    pub fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Char(c) if !modifiers.contains(KeyModifiers::CONTROL) => {
                self.insert_char(c);
            }
            KeyCode::Backspace => {
                self.backspace();
            }
            KeyCode::Delete => {
                self.delete();
            }
            KeyCode::Enter => {
                self.insert_newline();
            }
            KeyCode::Left => {
                self.move_left();
            }
            KeyCode::Right => {
                self.move_right();
            }
            KeyCode::Up => {
                self.move_up();
            }
            KeyCode::Down => {
                self.move_down();
            }
            KeyCode::Home => {
                self.move_home();
            }
            KeyCode::End => {
                self.move_end();
            }
            KeyCode::Tab => {
                // Insert spaces for tab (4 spaces)
                for _ in 0..4 {
                    self.insert_char(' ');
                }
            }
            _ => {
                // Ignore other keys
            }
        }
    }

    /// Insert a character at the cursor position.
    fn insert_char(&mut self, c: char) {
        if self.cursor_row >= self.lines.len() {
            // Ensure we have enough lines
            while self.lines.len() <= self.cursor_row {
                self.lines.push(String::new());
            }
        }

        let line = &mut self.lines[self.cursor_row];
        if self.cursor_col > line.len() {
            // Pad with spaces if cursor is beyond line end
            line.push_str(&" ".repeat(self.cursor_col - line.len()));
        }
        line.insert(self.cursor_col, c);
        self.cursor_col += 1;
    }

    /// Insert a newline at the cursor position.
    fn insert_newline(&mut self) {
        if self.cursor_row >= self.lines.len() {
            self.lines.push(String::new());
            self.cursor_row = self.lines.len() - 1;
            self.cursor_col = 0;
            return;
        }

        let line = &mut self.lines[self.cursor_row];
        let rest = if self.cursor_col < line.len() {
            line.split_off(self.cursor_col)
        } else {
            String::new()
        };

        // Insert new line after current
        self.lines.insert(self.cursor_row + 1, rest);
        self.cursor_row += 1;
        self.cursor_col = 0;
    }

    /// Delete character at cursor (backspace).
    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            // Delete character before cursor
            let line = &mut self.lines[self.cursor_row];
            line.remove(self.cursor_col - 1);
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            // Merge with previous line
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            let prev_line = &mut self.lines[self.cursor_row];
            self.cursor_col = prev_line.len();
            prev_line.push_str(&current_line);
        }
    }

    /// Delete character after cursor.
    fn delete(&mut self) {
        if self.cursor_row >= self.lines.len() {
            return;
        }
        
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col < line_len {
            self.lines[self.cursor_row].remove(self.cursor_col);
        } else if self.cursor_row < self.lines.len() - 1 {
            // Merge with next line
            let next_line = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next_line);
        }
    }

    /// Move cursor left.
    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    /// Move cursor right.
    fn move_right(&mut self) {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    /// Move cursor up.
    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            // Keep column within bounds of new line
            let line_len = self.lines[self.cursor_row].len();
            self.cursor_col = self.cursor_col.min(line_len);
        }
    }

    /// Move cursor down.
    fn move_down(&mut self) {
        if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            // Keep column within bounds of new line
            let line_len = self.lines[self.cursor_row].len();
            self.cursor_col = self.cursor_col.min(line_len);
        }
    }

    /// Move cursor to start of line.
    fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    /// Move cursor to end of line.
    fn move_end(&mut self) {
        if self.cursor_row < self.lines.len() {
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    /// Get cursor position (row, col).
    pub fn cursor(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }
}

impl Widget for TextArea {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default();
        let mut y = area.y;

        for (i, line) in self.lines.iter().enumerate() {
            if y >= area.y + area.height {
                break;
            }

            // Render line
            let line_text = if line.is_empty() && i == self.cursor_row && self.cursor_col == 0 {
                // Show cursor on empty line
                " "
            } else {
                line.as_str()
            };

            // Split line if it's too long (basic word wrap)
            let max_width = area.width as usize;
            if line_text.len() <= max_width {
                buf.set_stringn(area.x, y, line_text, max_width, style);
                // Show cursor if on this line
                if i == self.cursor_row {
                    let cursor_x = (area.x + self.cursor_col as u16).min(area.x + area.width - 1);
                    if cursor_x < area.x + area.width {
                        if let Some(cell) = buf.cell_mut((cursor_x, y)) {
                            cell.set_char('_'); // Cursor indicator
                            cell.set_style(style);
                        }
                    }
                }
                y += 1;
            } else {
                // Wrap long lines (simple character-based wrapping)
                let mut chars = line_text.chars().peekable();
                let mut col = 0;
                let mut current_line = String::new();

                while let Some(ch) = chars.next() {
                    let char_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1) as u16;
                    
                    if col + char_width > area.width {
                        // Render current line and start new one
                        buf.set_stringn(area.x, y, &current_line, area.width as usize, style);
                        y += 1;
                        if y >= area.y + area.height {
                            break;
                        }
                        current_line.clear();
                        col = 0;
                    }
                    
                    current_line.push(ch);
                    col += char_width;
                }
                
                if !current_line.is_empty() && y < area.y + area.height {
                    buf.set_stringn(area.x, y, &current_line, area.width as usize, style);
                    y += 1;
                }
            }
        }

        // Fill remaining area if needed
        while y < area.y + area.height {
            buf.set_stringn(area.x, y, "", area.width as usize, style);
            y += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_textarea_default() {
        let textarea = TextArea::default();
        assert_eq!(textarea.lines().len(), 1);
        assert_eq!(textarea.lines()[0], "");
    }

    #[test]
    fn test_textarea_insert_char() {
        let mut textarea = TextArea::default();
        textarea.handle_key(KeyCode::Char('h'), KeyModifiers::NONE);
        textarea.handle_key(KeyCode::Char('i'), KeyModifiers::NONE);
        assert_eq!(textarea.text(), "hi");
    }

    #[test]
    fn test_textarea_newline() {
        let mut textarea = TextArea::default();
        textarea.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
        textarea.handle_key(KeyCode::Enter, KeyModifiers::NONE);
        textarea.handle_key(KeyCode::Char('b'), KeyModifiers::NONE);
        assert_eq!(textarea.text(), "a\nb");
    }

    #[test]
    fn test_textarea_backspace() {
        let mut textarea = TextArea::default();
        textarea.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
        textarea.handle_key(KeyCode::Char('b'), KeyModifiers::NONE);
        textarea.handle_key(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(textarea.text(), "a");
    }

    #[test]
    fn test_textarea_set_text() {
        let mut textarea = TextArea::default();
        textarea.set_text("line1\nline2");
        assert_eq!(textarea.lines().len(), 2);
        assert_eq!(textarea.text(), "line1\nline2");
    }
}


//! Keyboard shortcut help overlay.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::theme::THEME;

/// Keyboard shortcut information.
#[derive(Debug, Clone)]
pub struct Shortcut {
    pub keys: String,
    pub description: String,
}

/// Shortcut category.
#[derive(Debug, Clone)]
pub struct ShortcutCategory {
    pub name: String,
    pub shortcuts: Vec<Shortcut>,
}

/// Get all keyboard shortcuts organized by category.
pub fn get_shortcuts() -> Vec<ShortcutCategory> {
    vec![
        ShortcutCategory {
            name: "Global".to_string(),
            shortcuts: vec![
                Shortcut {
                    keys: "Ctrl+C / Ctrl+D".to_string(),
                    description: "Quit application".to_string(),
                },
                Shortcut {
                    keys: "Ctrl+P".to_string(),
                    description: "Open command palette".to_string(),
                },
                Shortcut {
                    keys: "? / F1".to_string(),
                    description: "Show keyboard shortcuts".to_string(),
                },
                Shortcut {
                    keys: "Esc".to_string(),
                    description: "Cancel/go back".to_string(),
                },
            ],
        },
        ShortcutCategory {
            name: "Input & Commands".to_string(),
            shortcuts: vec![
                Shortcut {
                    keys: "Enter".to_string(),
                    description: "Execute command or send message".to_string(),
                },
                Shortcut {
                    keys: "Tab".to_string(),
                    description: "Autocomplete selected command".to_string(),
                },
                Shortcut {
                    keys: "Backspace".to_string(),
                    description: "Delete character".to_string(),
                },
            ],
        },
        ShortcutCategory {
            name: "Navigation".to_string(),
            shortcuts: vec![
                Shortcut {
                    keys: "↑ / ↓".to_string(),
                    description: "Navigate lists and suggestions".to_string(),
                },
                Shortcut {
                    keys: "PgUp / PgDn".to_string(),
                    description: "Scroll conversation history".to_string(),
                },
                Shortcut {
                    keys: "Home".to_string(),
                    description: "Scroll to top".to_string(),
                },
                Shortcut {
                    keys: "End".to_string(),
                    description: "Scroll to bottom".to_string(),
                },
            ],
        },
        ShortcutCategory {
            name: "Command Palette".to_string(),
            shortcuts: vec![
                Shortcut {
                    keys: "Ctrl+P".to_string(),
                    description: "Open command palette".to_string(),
                },
                Shortcut {
                    keys: "↑ / ↓".to_string(),
                    description: "Navigate suggestions".to_string(),
                },
                Shortcut {
                    keys: "Enter".to_string(),
                    description: "Execute selected command".to_string(),
                },
                Shortcut {
                    keys: "Esc".to_string(),
                    description: "Close command palette".to_string(),
                },
            ],
        },
    ]
}

/// Render the keyboard shortcut help overlay.
pub fn render_shortcuts(frame: &mut Frame, area: Rect) {
    // Calculate overlay size (80% of screen, centered)
    let overlay_width = (area.width as f32 * 0.8) as u16;
    let overlay_height = (area.height as f32 * 0.8) as u16;
    let overlay_x = (area.width.saturating_sub(overlay_width)) / 2;
    let overlay_y = (area.height.saturating_sub(overlay_height)) / 2;

    let overlay_area = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

    // Create chunks for title, content, and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(5),      // Content
            Constraint::Length(2),  // Footer
        ])
        .split(overlay_area);

    // Title
    let title = Paragraph::new("Keyboard Shortcuts")
        .style(Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(THEME.border()))
                .style(Style::default().bg(THEME.bg_panel())),
        );
    frame.render_widget(title, chunks[0]);

    // Content - scrollable list of shortcuts
    let shortcuts = get_shortcuts();
    let mut items = Vec::new();

    for category in &shortcuts {
        // Category header
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!("{} ", category.name),
                Style::default()
                    .fg(THEME.primary())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "─".repeat(overlay_width as usize - category.name.len() - 1),
                Style::default().fg(THEME.border()),
            ),
        ])));

        // Shortcuts in category
        for shortcut in &category.shortcuts {
            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {:<20} ", shortcut.keys),
                    Style::default().fg(THEME.secondary()).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    shortcut.description.clone(),
                    Style::default().fg(THEME.text()),
                ),
            ])));
        }

        // Spacing between categories
        items.push(ListItem::new(Line::from("")));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(THEME.border()))
                .style(Style::default().bg(THEME.bg_panel())),
        )
        .style(Style::default().fg(THEME.text()));

    frame.render_widget(list, chunks[1]);

    // Footer
    let footer = Paragraph::new("Press Esc or any key to close")
        .style(Style::default().fg(THEME.text_muted()))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(THEME.border()))
                .style(Style::default().bg(THEME.bg_panel())),
        );
    frame.render_widget(footer, chunks[2]);
}


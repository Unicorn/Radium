//! Toast notification component for non-intrusive feedback.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::time::{Duration, Instant};

/// Toast notification variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastVariant {
    Success,
    Error,
    Info,
    Warning,
}

impl ToastVariant {
    /// Returns the color for this variant.
    pub fn color(&self) -> Color {
        let theme = crate::theme::get_theme();
        match self {
            Self::Success => theme.success,
            Self::Error => theme.error,
            Self::Info => theme.info,
            Self::Warning => theme.warning,
        }
    }

    /// Returns the icon for this variant.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Success => "✓",
            Self::Error => "✗",
            Self::Info => "ℹ",
            Self::Warning => "⚠",
        }
    }
}

/// A single toast notification.
#[derive(Debug, Clone)]
pub struct Toast {
    /// Toast variant
    pub variant: ToastVariant,
    /// Message text
    pub message: String,
    /// When this toast was created
    pub created_at: Instant,
    /// Duration before auto-dismiss (None = no auto-dismiss)
    pub duration: Option<Duration>,
}

impl Toast {
    /// Creates a new toast notification.
    pub fn new(variant: ToastVariant, message: String) -> Self {
        Self {
            variant,
            message,
            created_at: Instant::now(),
            duration: Some(Duration::from_secs(3)),
        }
    }

    /// Creates a toast with custom duration.
    pub fn with_duration(variant: ToastVariant, message: String, duration: Duration) -> Self {
        Self {
            variant,
            message,
            created_at: Instant::now(),
            duration: Some(duration),
        }
    }

    /// Creates a persistent toast (no auto-dismiss).
    pub fn persistent(variant: ToastVariant, message: String) -> Self {
        Self {
            variant,
            message,
            created_at: Instant::now(),
            duration: None,
        }
    }

    /// Returns whether this toast should be dismissed.
    pub fn should_dismiss(&self) -> bool {
        if let Some(duration) = self.duration {
            self.created_at.elapsed() >= duration
        } else {
            false
        }
    }

    /// Returns the remaining time before auto-dismiss (if applicable).
    pub fn remaining_time(&self) -> Option<Duration> {
        self.duration.map(|duration| {
            let elapsed = self.created_at.elapsed();
            duration.saturating_sub(elapsed)
        })
    }
}

/// Toast manager for handling multiple toasts.
#[derive(Debug, Default)]
pub struct ToastManager {
    /// Active toasts (newest first)
    toasts: Vec<Toast>,
    /// Maximum number of toasts to show
    max_toasts: usize,
}

impl ToastManager {
    /// Creates a new toast manager.
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            max_toasts: 5,
        }
    }

    /// Creates a toast manager with custom max toasts.
    pub fn with_max_toasts(max_toasts: usize) -> Self {
        Self {
            toasts: Vec::new(),
            max_toasts,
        }
    }

    /// Shows a new toast notification.
    pub fn show(&mut self, toast: Toast) {
        self.toasts.insert(0, toast);
        // Keep only the most recent toasts
        if self.toasts.len() > self.max_toasts {
            self.toasts.truncate(self.max_toasts);
        }
    }

    /// Shows a success toast.
    pub fn success(&mut self, message: String) {
        self.show(Toast::new(ToastVariant::Success, message));
    }

    /// Shows an error toast.
    pub fn error(&mut self, message: String) {
        self.show(Toast::new(ToastVariant::Error, message));
    }

    /// Shows an info toast.
    pub fn info(&mut self, message: String) {
        self.show(Toast::new(ToastVariant::Info, message));
    }

    /// Shows a warning toast.
    pub fn warning(&mut self, message: String) {
        self.show(Toast::new(ToastVariant::Warning, message));
    }

    /// Updates the toast manager (removes expired toasts).
    pub fn update(&mut self) {
        self.toasts.retain(|toast| !toast.should_dismiss());
    }

    /// Returns a reference to active toasts.
    pub fn toasts(&self) -> &[Toast] {
        &self.toasts
    }

    /// Clears all toasts.
    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    /// Dismisses a specific toast by index.
    pub fn dismiss(&mut self, index: usize) {
        if index < self.toasts.len() {
            self.toasts.remove(index);
        }
    }
}

/// Renders toast notifications in the top-right corner.
pub fn render_toasts(frame: &mut Frame, area: Rect, manager: &ToastManager) {
    let _areas = render_toasts_with_areas(frame, area, manager);
}

/// Renders toast notifications and returns their areas for animation targeting.
pub fn render_toasts_with_areas(frame: &mut Frame, area: Rect, manager: &ToastManager) -> Vec<Rect> {
    let toasts = manager.toasts();
    if toasts.is_empty() {
        return Vec::new();
    }

    let theme = crate::theme::get_theme();
    let max_width = 50u16;
    let spacing = 1u16; // Space between toasts
    let toast_height = 3u16; // Height per toast (1 line text + 2 for borders/padding)

    // Calculate total height needed
    let total_height = (toasts.len() as u16 * (toast_height + spacing)).saturating_sub(spacing);
    
    // Position in top-right corner with some margin
    let margin = 2u16;
    let toast_area = Rect {
        x: area.width.saturating_sub(max_width + margin),
        y: margin,
        width: max_width.min(area.width.saturating_sub(margin * 2)),
        height: total_height.min(area.height.saturating_sub(margin * 2)),
    };

    // Render each toast and collect areas
    let mut y_offset = 0u16;
    let mut areas = Vec::new();
    for (_idx, toast) in toasts.iter().enumerate() {
        if y_offset >= toast_area.height {
            break;
        }

        let toast_rect = Rect {
            x: toast_area.x,
            y: toast_area.y + y_offset,
            width: toast_area.width,
            height: toast_height.min(toast_area.height.saturating_sub(y_offset)),
        };

        render_single_toast(frame, toast_rect, toast, &theme);
        areas.push(toast_rect);
        y_offset += toast_height + spacing;
    }
    
    areas
}

/// Renders a single toast notification.
fn render_single_toast(frame: &mut Frame, area: Rect, toast: &Toast, theme: &crate::theme::RadiumTheme) {
    let variant_color = toast.variant.color();
    let icon = toast.variant.icon();

    // Format message with icon
    let message = format!("{} {}", icon, toast.message);
    
    // Wrap text if needed
    let wrapped_lines: Vec<String> = textwrap::wrap(&message, (area.width.saturating_sub(4)) as usize)
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Create styled lines
    let lines: Vec<Line> = wrapped_lines
        .iter()
        .map(|line| {
            Line::from(vec![
                Span::styled(icon, Style::default().fg(variant_color)),
                Span::raw(" "),
                Span::styled(line.trim_start(), Style::default().fg(theme.text)),
            ])
        })
        .collect();

    let widget = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(variant_color))
                .style(Style::default().bg(theme.bg_panel))
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(widget, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_creation() {
        let toast = Toast::new(ToastVariant::Success, "Test message".to_string());
        assert_eq!(toast.variant, ToastVariant::Success);
        assert_eq!(toast.message, "Test message");
        assert!(toast.duration.is_some());
    }

    #[test]
    fn test_toast_persistent() {
        let toast = Toast::persistent(ToastVariant::Error, "Persistent error".to_string());
        assert!(toast.duration.is_none());
        assert!(!toast.should_dismiss());
    }

    #[test]
    fn test_toast_manager() {
        let mut manager = ToastManager::new();
        manager.success("Success!".to_string());
        manager.error("Error!".to_string());
        
        assert_eq!(manager.toasts().len(), 2);
        
        manager.update();
        assert_eq!(manager.toasts().len(), 2); // Not expired yet
        
        manager.clear();
        assert_eq!(manager.toasts().len(), 0);
    }

    #[test]
    fn test_toast_variant_colors() {
        let theme = crate::theme::get_theme();
        assert_eq!(ToastVariant::Success.color(), theme.success);
        assert_eq!(ToastVariant::Error.color(), theme.error);
        assert_eq!(ToastVariant::Info.color(), theme.info);
        assert_eq!(ToastVariant::Warning.color(), theme.warning);
    }
}


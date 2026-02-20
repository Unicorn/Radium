//! Progress Indicator Components
//!
//! Provides visual feedback for long-running operations with three types:
//! - **Determinate**: Progress bar with percentage (file operations, batch tasks)
//! - **Indeterminate**: Spinner for unknown-duration operations
//! - **Streaming**: Token count and rate display for LLM streaming

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph},
};
use std::time::{Duration, Instant};

/// Type of progress indicator to display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressType {
    /// Progress bar with known total (current/total)
    Determinate,
    /// Spinner for unknown duration
    Indeterminate,
    /// Token count and rate for streaming
    Streaming,
}

/// Progress indicator state and configuration.
#[derive(Debug, Clone)]
pub struct ProgressIndicator {
    /// Type of progress indicator
    pub progress_type: ProgressType,
    /// Label/description of the operation
    pub label: String,
    /// Current progress (for determinate)
    pub current: u64,
    /// Total items/bytes (for determinate)
    pub total: u64,
    /// Token count (for streaming)
    pub token_count: u64,
    /// Start time for rate calculation
    pub start_time: Instant,
    /// Last update time (for animation)
    pub last_update: Instant,
    /// Animation frame (for spinner)
    pub animation_frame: usize,
    /// Whether the operation is complete
    pub is_complete: bool,
}

impl ProgressIndicator {
    /// Create a new determinate progress indicator.
    pub fn new_determinate(label: String, total: u64) -> Self {
        Self {
            progress_type: ProgressType::Determinate,
            label,
            current: 0,
            total,
            token_count: 0,
            start_time: Instant::now(),
            last_update: Instant::now(),
            animation_frame: 0,
            is_complete: false,
        }
    }

    /// Create a new indeterminate progress indicator (spinner).
    pub fn new_indeterminate(label: String) -> Self {
        Self {
            progress_type: ProgressType::Indeterminate,
            label,
            current: 0,
            total: 0,
            token_count: 0,
            start_time: Instant::now(),
            last_update: Instant::now(),
            animation_frame: 0,
            is_complete: false,
        }
    }

    /// Create a new streaming progress indicator.
    pub fn new_streaming(label: String) -> Self {
        Self {
            progress_type: ProgressType::Streaming,
            label,
            current: 0,
            total: 0,
            token_count: 0,
            start_time: Instant::now(),
            last_update: Instant::now(),
            animation_frame: 0,
            is_complete: false,
        }
    }

    /// Update progress (for determinate indicators).
    pub fn update_progress(&mut self, current: u64) {
        self.current = current;
        self.last_update = Instant::now();
        if current >= self.total && self.total > 0 {
            self.is_complete = true;
        }
    }

    /// Increment token count (for streaming indicators).
    pub fn increment_tokens(&mut self, count: u64) {
        self.token_count += count;
        self.last_update = Instant::now();
    }

    /// Update animation frame (call every ~100ms).
    pub fn tick(&mut self) {
        self.animation_frame = (self.animation_frame + 1) % 8;
        self.last_update = Instant::now();
    }

    /// Mark the operation as complete.
    pub fn complete(&mut self) {
        self.is_complete = true;
        self.last_update = Instant::now();
    }

    /// Get the progress percentage (0-100) for determinate indicators.
    pub fn percentage(&self) -> u16 {
        if self.total == 0 {
            return 0;
        }
        ((self.current as f64 / self.total as f64) * 100.0).min(100.0) as u16
    }

    /// Get the elapsed time since start.
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get the current rate (tokens/sec for streaming, items/sec for determinate).
    pub fn rate(&self) -> f64 {
        let elapsed_secs = self.elapsed().as_secs_f64();
        if elapsed_secs < 0.1 {
            return 0.0;
        }
        match self.progress_type {
            ProgressType::Streaming => self.token_count as f64 / elapsed_secs,
            ProgressType::Determinate => self.current as f64 / elapsed_secs,
            ProgressType::Indeterminate => 0.0,
        }
    }

    /// Get the spinner character for the current animation frame.
    fn spinner_char(&self) -> &str {
        const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
        SPINNER_FRAMES[self.animation_frame % SPINNER_FRAMES.len()]
    }
}

/// Render a progress indicator in the given area.
pub fn render_progress_indicator(
    frame: &mut Frame,
    area: Rect,
    indicator: &ProgressIndicator,
) {
    use crate::theme::THEME;

    match indicator.progress_type {
        ProgressType::Determinate => {
            // Render progress bar with percentage
            let percentage = indicator.percentage();
            let label = if indicator.is_complete {
                format!("{} - Complete!", indicator.label)
            } else {
                format!(
                    "{} - {}/{} ({}%)",
                    indicator.label, indicator.current, indicator.total, percentage
                )
            };

            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(THEME.border()))
                        .style(Style::default().bg(THEME.bg_panel())),
                )
                .gauge_style(
                    Style::default()
                        .fg(if indicator.is_complete {
                            THEME.success()
                        } else {
                            THEME.primary()
                        })
                        .bg(THEME.bg_element()),
                )
                .label(label)
                .percent(percentage);

            frame.render_widget(gauge, area);
        }

        ProgressType::Indeterminate => {
            // Render spinner with elapsed time
            let elapsed = indicator.elapsed();
            let elapsed_str = if elapsed.as_secs() < 60 {
                format!("{}s", elapsed.as_secs())
            } else {
                format!("{}m {}s", elapsed.as_secs() / 60, elapsed.as_secs() % 60)
            };

            let text = if indicator.is_complete {
                format!("✓ {} - Complete!", indicator.label)
            } else {
                format!("{} {} - {}", indicator.spinner_char(), indicator.label, elapsed_str)
            };

            let spinner = Paragraph::new(text)
                .style(
                    Style::default()
                        .fg(if indicator.is_complete {
                            THEME.success()
                        } else {
                            THEME.primary()
                        })
                        .add_modifier(Modifier::BOLD),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(THEME.border()))
                        .style(Style::default().bg(THEME.bg_panel())),
                );

            frame.render_widget(spinner, area);
        }

        ProgressType::Streaming => {
            // Render token count and rate
            let rate = indicator.rate();
            let elapsed = indicator.elapsed();
            let elapsed_str = if elapsed.as_secs() < 60 {
                format!("{}s", elapsed.as_secs())
            } else {
                format!("{}m {}s", elapsed.as_secs() / 60, elapsed.as_secs() % 60)
            };

            let text = if indicator.is_complete {
                format!(
                    "✓ {} - {} tokens in {} ({:.1} tok/s)",
                    indicator.label, indicator.token_count, elapsed_str, rate
                )
            } else {
                format!(
                    "{} {} - {} tokens ({:.1} tok/s)",
                    indicator.spinner_char(),
                    indicator.label,
                    indicator.token_count,
                    rate
                )
            };

            let streaming = Paragraph::new(text)
                .style(
                    Style::default()
                        .fg(if indicator.is_complete {
                            THEME.success()
                        } else {
                            THEME.secondary()
                        })
                        .add_modifier(Modifier::BOLD),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(THEME.border()))
                        .style(Style::default().bg(THEME.bg_panel())),
                );

            frame.render_widget(streaming, area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determinate_progress() {
        let mut indicator = ProgressIndicator::new_determinate("Processing files".to_string(), 100);
        assert_eq!(indicator.percentage(), 0);

        indicator.update_progress(50);
        assert_eq!(indicator.percentage(), 50);
        assert!(!indicator.is_complete);

        indicator.update_progress(100);
        assert_eq!(indicator.percentage(), 100);
        assert!(indicator.is_complete);
    }

    #[test]
    fn test_streaming_tokens() {
        let mut indicator = ProgressIndicator::new_streaming("Generating response".to_string());
        assert_eq!(indicator.token_count, 0);

        indicator.increment_tokens(10);
        assert_eq!(indicator.token_count, 10);

        indicator.increment_tokens(5);
        assert_eq!(indicator.token_count, 15);
    }

    #[test]
    fn test_spinner_animation() {
        let mut indicator = ProgressIndicator::new_indeterminate("Loading".to_string());
        let first_char = indicator.spinner_char().to_string();

        indicator.tick();
        let second_char = indicator.spinner_char().to_string();

        // Should cycle through frames
        assert_ne!(first_char, second_char);
    }

    #[test]
    fn test_rate_calculation() {
        let mut indicator = ProgressIndicator::new_streaming("Streaming".to_string());

        // Wait a bit and add tokens
        std::thread::sleep(std::time::Duration::from_millis(100));
        indicator.increment_tokens(100);

        let rate = indicator.rate();
        // Rate should be positive
        assert!(rate > 0.0);
    }

    #[test]
    fn test_completion() {
        let mut indicator = ProgressIndicator::new_indeterminate("Task".to_string());
        assert!(!indicator.is_complete);

        indicator.complete();
        assert!(indicator.is_complete);
    }
}

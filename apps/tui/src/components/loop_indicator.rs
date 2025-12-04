//! Loop indicator component for displaying workflow loop iterations.

use crate::state::CheckpointState;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

/// Loop indicator component
pub struct LoopIndicator;

impl LoopIndicator {
    /// Renders the loop indicator.
    pub fn render(frame: &mut Frame, area: Rect, checkpoint_state: &CheckpointState) {
        if let Some(current_loop) = checkpoint_state.get_current_loop_iteration() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Title
                    Constraint::Length(4),  // Current iteration info
                    Constraint::Min(5),     // History
                ])
                .split(area);

            // Title
            let title = Paragraph::new("Loop Iteration")
                .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(title, chunks[0]);

            // Current iteration info
            let elapsed = if let Ok(duration) = std::time::SystemTime::now().duration_since(current_loop.start_time) {
                format!("{:.1}s", duration.as_secs_f64())
            } else {
                "?".to_string()
            };

            let info_text = format!(
                "Iteration: {}\nStart Step: {}\nElapsed: {}",
                current_loop.iteration, current_loop.start_step, elapsed
            );

            let info = Paragraph::new(info_text)
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title(" Current "));
            frame.render_widget(info, chunks[1]);

            // Loop history
            Self::render_loop_history(frame, chunks[2], checkpoint_state);
        } else {
            // No active loop
            let text = "No active loop iterations";
            let widget = Paragraph::new(text)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title(" Loop Indicator "));
            frame.render_widget(widget, area);
        }
    }

    /// Renders loop iteration history.
    fn render_loop_history(frame: &mut Frame, area: Rect, checkpoint_state: &CheckpointState) {
        let items: Vec<ListItem> = checkpoint_state
            .loop_iterations
            .iter()
            .map(|loop_iter| {
                let status = if loop_iter.is_active() {
                    "Active"
                } else {
                    "Completed"
                };

                let style = if loop_iter.is_active() {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Green)
                };

                let end_info = if let Some(end_step) = loop_iter.end_step {
                    format!(" → {}", end_step)
                } else {
                    String::new()
                };

                let content = format!(
                    "Iteration {} | Step {}{} | {}",
                    loop_iter.iteration, loop_iter.start_step, end_info, status
                );

                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" History "));
        frame.render_widget(list, area);
    }

    /// Renders a compact loop indicator.
    pub fn render_compact(frame: &mut Frame, area: Rect, checkpoint_state: &CheckpointState) {
        let text = if let Some(current_loop) = checkpoint_state.get_current_loop_iteration() {
            format!(
                "Loop Iteration {} (Step {}) | Total Iterations: {}",
                current_loop.iteration,
                current_loop.start_step,
                checkpoint_state.total_loop_iterations()
            )
        } else if checkpoint_state.total_loop_iterations() > 0 {
            format!(
                "No active loop | Total Iterations: {}",
                checkpoint_state.total_loop_iterations()
            )
        } else {
            "No loop iterations".to_string()
        };

        let widget = Paragraph::new(text)
            .style(Style::default().fg(Color::Magenta))
            .block(Block::default().borders(Borders::ALL).title(" Loop Status "));
        frame.render_widget(widget, area);
    }

    /// Renders loop progress as a gauge.
    pub fn render_progress(
        frame: &mut Frame,
        area: Rect,
        current_iteration: usize,
        max_iterations: usize,
    ) {
        let ratio = if max_iterations > 0 {
            current_iteration as f64 / max_iterations as f64
        } else {
            0.0
        };

        let label = format!("Loop: {}/{}", current_iteration, max_iterations);

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Loop Progress "))
            .gauge_style(
                Style::default()
                    .fg(if current_iteration >= max_iterations { Color::Green } else { Color::Magenta })
                    .bg(Color::Black)
            )
            .label(label)
            .ratio(ratio);

        frame.render_widget(gauge, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_indicator_creation() {
        // This is a rendering component, so we just ensure it compiles
        let _component = LoopIndicator;
    }
}

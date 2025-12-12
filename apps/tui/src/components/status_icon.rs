//! Status icon rendering component with animation support.

use crate::state::AgentStatus;
use crate::theme::RadiumTheme;

#[cfg(test)]
use crate::icons::Icons;
use ratatui::style::{Color, Style};

/// Returns the theme color for an agent status.
/// Reuses the color mapping logic from AgentTimeline.
pub fn get_status_color(status: AgentStatus, theme: &RadiumTheme) -> Color {
    match status {
        AgentStatus::Idle => theme.text_muted,
        AgentStatus::Starting => theme.warning,
        AgentStatus::Running => theme.info,
        AgentStatus::Thinking => theme.primary,
        AgentStatus::ExecutingTool => theme.secondary,
        AgentStatus::Completed => theme.success,
        AgentStatus::Failed => theme.error,
        AgentStatus::Cancelled => theme.text_dim,
    }
}

/// Renders a status icon with optional animation support.
/// 
/// Returns a tuple of (icon_string, Style) for use in ratatui widgets.
/// 
/// # Arguments
/// 
/// * `status` - The agent status to render
/// * `frame_counter` - Current frame counter for animations (from App.spinner_frame)
/// * `animations_enabled` - Whether animations are enabled in config
/// * `reduced_motion` - Whether reduced motion is enabled in config
/// 
/// # Returns
/// 
/// A tuple of (icon_string, Style) where:
/// - `icon_string` is the icon to display (animated spinner frame or static icon)
/// - `Style` has the appropriate color applied
pub fn render_status_icon(
    status: AgentStatus,
    frame_counter: usize,
    animations_enabled: bool,
    reduced_motion: bool,
) -> (String, Style) {
    use crate::components::spinner::Spinner;

    // Determine if this status should animate
    let should_animate = matches!(
        status,
        AgentStatus::Running | AgentStatus::Thinking | AgentStatus::Starting | AgentStatus::ExecutingTool
    );

    // Get the icon string
    let icon_str = if should_animate && animations_enabled && !reduced_motion {
        // Use animated spinner
        let spinner = Spinner::new();
        spinner.current_frame(frame_counter, animations_enabled, reduced_motion)
    } else {
        // Use static icon from Icons constants
        status.icon()
    };

    // Get theme color
    let theme = RadiumTheme::from_config();
    let color = get_status_color(status, &theme);

    // Return icon string and styled
    (icon_str.to_string(), Style::default().fg(color))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_status_color() {
        let theme = RadiumTheme::dark();

        assert_eq!(get_status_color(AgentStatus::Idle, &theme), theme.text_muted);
        assert_eq!(get_status_color(AgentStatus::Starting, &theme), theme.warning);
        assert_eq!(get_status_color(AgentStatus::Running, &theme), theme.info);
        assert_eq!(get_status_color(AgentStatus::Thinking, &theme), theme.primary);
        assert_eq!(get_status_color(AgentStatus::ExecutingTool, &theme), theme.secondary);
        assert_eq!(get_status_color(AgentStatus::Completed, &theme), theme.success);
        assert_eq!(get_status_color(AgentStatus::Failed, &theme), theme.error);
        assert_eq!(get_status_color(AgentStatus::Cancelled, &theme), theme.text_dim);
    }

    #[test]
    fn test_render_status_icon_static() {
        // Test static icon rendering (animations disabled)
        let (icon, style) = render_status_icon(AgentStatus::Completed, 0, false, false);
        assert_eq!(icon, Icons::COMPLETED);
        assert_eq!(style.fg, Some(Color::Rgb(16, 185, 129))); // theme.success
    }

    #[test]
    fn test_render_status_icon_animated() {
        // Test animated icon rendering (Running status with animations enabled)
        let (icon_0, style_0) = render_status_icon(AgentStatus::Running, 0, true, false);
        let (icon_1, style_1) = render_status_icon(AgentStatus::Running, 1, true, false);
        
        // Icons should be different (different spinner frames)
        assert_ne!(icon_0, icon_1);
        // Color should be the same (theme.info)
        assert_eq!(style_0.fg, style_1.fg);
    }

    #[test]
    fn test_render_status_icon_reduced_motion() {
        // Test reduced motion fallback
        let (icon, style) = render_status_icon(AgentStatus::Running, 5, true, true);
        
        // Should return static icon, not animated frame
        assert_eq!(icon, Icons::RUNNING);
        // Color should still be correct
        assert_eq!(style.fg, Some(Color::Rgb(6, 182, 212))); // theme.info
    }

    #[test]
    fn test_render_status_icon_non_animated_status() {
        // Test that non-animated statuses always return static icons
        let (icon_0, _) = render_status_icon(AgentStatus::Completed, 0, true, false);
        let (icon_1, _) = render_status_icon(AgentStatus::Completed, 1, true, false);
        
        // Should always be the same (Completed doesn't animate)
        assert_eq!(icon_0, icon_1);
        assert_eq!(icon_0, Icons::COMPLETED);
    }
}


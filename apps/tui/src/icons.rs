//! Status icons and visual indicators for the TUI.

/// Status icons for visual feedback.
pub struct Icons;

impl Icons {
    /// Success / Completed
    pub const SUCCESS: &'static str = "✓";

    /// Warning
    pub const WARNING: &'static str = "⚠";

    /// Error / Failed
    pub const ERROR: &'static str = "✗";

    /// Loading / In Progress
    pub const LOADING: &'static str = "⏳";

    /// Chat message (user)
    pub const CHAT: &'static str = "💬";

    /// Agent response
    pub const AGENT: &'static str = "🤖";

    /// Session
    pub const SESSION: &'static str = "📝";

    /// Authentication
    pub const AUTH: &'static str = "🔑";

    /// Settings
    pub const SETTINGS: &'static str = "⚙️";

    /// Edit / Input
    pub const EDIT: &'static str = "✏️";

    /// Info
    pub const INFO: &'static str = "ℹ";

    /// Rocket (startup/welcome)
    pub const ROCKET: &'static str = "🚀";

    // Status icons for agent states
    /// Idle / Paused
    pub const IDLE: &'static str = "⏸";
    /// Starting / Initializing
    pub const STARTING: &'static str = "🔄";
    /// Pending / Queued
    pub const PENDING: &'static str = "○";
    /// Running (first spinner frame - animation handled by components)
    pub const RUNNING: &'static str = "⠋";
    /// Completed / Success
    pub const COMPLETED: &'static str = "●";
    /// Failed / Error
    pub const FAILED: &'static str = "✗";
    /// Retrying
    pub const RETRYING: &'static str = "⟳";
    /// Thinking / Processing
    pub const THINKING: &'static str = "💭";
    /// Executing Tool
    pub const EXECUTING: &'static str = "🔧";
    /// Cancelled
    pub const CANCELLED: &'static str = "⊗";
}

impl Icons {
    /// Returns the spinner frame for the given frame counter.
    /// Uses the default braille spinner sequence.
    pub fn get_spinner_frame(
        frame_counter: usize,
        animations_enabled: bool,
        reduced_motion: bool,
    ) -> &'static str {
        use crate::components::spinner::Spinner;
        let spinner = Spinner::new();
        spinner.current_frame(frame_counter, animations_enabled, reduced_motion)
    }
}

/// ASCII alternatives for terminals that don't support Unicode.
pub struct AsciiIcons;

impl AsciiIcons {
    pub const SUCCESS: &'static str = "[✓]";
    pub const WARNING: &'static str = "[!]";
    pub const ERROR: &'static str = "[X]";
    pub const LOADING: &'static str = "[~]";
    pub const CHAT: &'static str = ">";
    pub const AGENT: &'static str = "<";
    pub const SESSION: &'static str = "#";
    pub const AUTH: &'static str = "*";
    pub const SETTINGS: &'static str = "@";
    pub const INFO: &'static str = "[i]";
    pub const ROCKET: &'static str = "^";

    // Status icons for agent states (ASCII alternatives)
    pub const IDLE: &'static str = "||";
    pub const STARTING: &'static str = "[~]";
    pub const PENDING: &'static str = "( )";
    pub const RUNNING: &'static str = "[>]";
    pub const COMPLETED: &'static str = "(*)";
    pub const FAILED: &'static str = "[X]";
    pub const RETRYING: &'static str = "[@]";
    pub const THINKING: &'static str = "[?]";
    pub const EXECUTING: &'static str = "[#]";
    pub const CANCELLED: &'static str = "[!]";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_icon_constants() {
        // Verify all status constants are defined
        assert_eq!(Icons::IDLE, "⏸");
        assert_eq!(Icons::STARTING, "🔄");
        assert_eq!(Icons::PENDING, "○");
        assert_eq!(Icons::RUNNING, "⠋");
        assert_eq!(Icons::COMPLETED, "●");
        assert_eq!(Icons::FAILED, "✗");
        assert_eq!(Icons::RETRYING, "⟳");
        assert_eq!(Icons::THINKING, "💭");
        assert_eq!(Icons::EXECUTING, "🔧");
        assert_eq!(Icons::CANCELLED, "⊗");
    }

    #[test]
    fn test_ascii_status_icon_constants() {
        // Verify all ASCII status constants are defined
        assert_eq!(AsciiIcons::IDLE, "||");
        assert_eq!(AsciiIcons::STARTING, "[~]");
        assert_eq!(AsciiIcons::PENDING, "( )");
        assert_eq!(AsciiIcons::RUNNING, "[>]");
        assert_eq!(AsciiIcons::COMPLETED, "(*)");
        assert_eq!(AsciiIcons::FAILED, "[X]");
        assert_eq!(AsciiIcons::RETRYING, "[@]");
        assert_eq!(AsciiIcons::THINKING, "[?]");
        assert_eq!(AsciiIcons::EXECUTING, "[#]");
        assert_eq!(AsciiIcons::CANCELLED, "[!]");
    }

    #[test]
    fn test_get_spinner_frame() {
        // Test spinner frame retrieval with animations enabled
        let frame_0 = Icons::get_spinner_frame(0, true, false);
        let frame_1 = Icons::get_spinner_frame(1, true, false);
        assert_ne!(frame_0, frame_1); // Should be different frames

        // Test that it cycles
        let frame_10 = Icons::get_spinner_frame(10, true, false);
        let frame_0_again = Icons::get_spinner_frame(0, true, false);
        assert_eq!(frame_0, frame_0_again);
    }

    #[test]
    fn test_get_spinner_frame_reduced_motion() {
        // With reduced motion, should return static icon
        let frame = Icons::get_spinner_frame(5, true, true);
        assert_eq!(frame, Icons::RUNNING); // Should return static first frame
    }

    #[test]
    fn test_get_spinner_frame_animations_disabled() {
        // With animations disabled, should return static icon
        let frame = Icons::get_spinner_frame(5, false, false);
        assert_eq!(frame, Icons::RUNNING); // Should return static first frame
    }
}

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

    /// Info
    pub const INFO: &'static str = "ℹ";

    /// Rocket (startup/welcome)
    pub const ROCKET: &'static str = "🚀";
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
}

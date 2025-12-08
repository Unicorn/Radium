//! Privacy state management for TUI.

/// Privacy state tracking for the TUI session.
#[derive(Debug, Clone)]
pub struct PrivacyState {
    /// Whether privacy mode is enabled.
    pub enabled: bool,
    /// Total number of redactions performed in this session.
    pub redaction_count: usize,
}

impl PrivacyState {
    /// Creates a new privacy state.
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            redaction_count: 0,
        }
    }

    /// Toggles privacy mode.
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Increments the redaction count.
    pub fn increment_redaction_count(&mut self, count: usize) {
        self.redaction_count += count;
    }

    /// Resets the redaction count.
    pub fn reset_redaction_count(&mut self) {
        self.redaction_count = 0;
    }
}

impl Default for PrivacyState {
    fn default() -> Self {
        Self::new(false)
    }
}


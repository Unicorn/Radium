//! Navigation and view management

/// Current view in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Agents,
    Workflows,
    Tasks,
    Chat,
}

/// Navigation state
pub struct Navigation {
    current_view: View,
}

impl Navigation {
    pub fn new() -> Self {
        Self { current_view: View::Chat }
    }

    pub fn current_view(&self) -> View {
        self.current_view
    }

    pub fn set_view(&mut self, view: View) {
        self.current_view = view;
    }
}

impl Default for Navigation {
    fn default() -> Self {
        Self::new()
    }
}

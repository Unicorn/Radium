//! Dirty Flags System for Smart Rendering
//!
//! Tracks which parts of the UI need re-rendering to avoid unnecessary CPU usage.
//! Inspired by bottom's efficient rendering pattern.

/// Dirty flags for smart rendering - only render when state changes
#[derive(Debug, Clone)]
pub struct DirtyFlags {
    /// Layout changed (window resize)
    layout: bool,
    /// Content changed (new message, state update, user input)
    content: bool,
    /// Effects running (animations, spinners)
    effects: bool,
    /// Force render (used for initial render and explicit refreshes)
    force: bool,
}

impl DirtyFlags {
    /// Create a new DirtyFlags with all flags set to false
    pub fn new() -> Self {
        Self {
            layout: false,
            content: false,
            effects: false,
            force: false,
        }
    }

    /// Create a new DirtyFlags with force render enabled (for initial render)
    pub fn force_render() -> Self {
        Self {
            layout: false,
            content: false,
            effects: false,
            force: true,
        }
    }

    /// Check if any dirty flag is set
    pub fn is_dirty(&self) -> bool {
        self.layout || self.content || self.effects || self.force
    }

    /// Clear all dirty flags
    pub fn clear(&mut self) {
        self.layout = false;
        self.content = false;
        self.effects = false;
        self.force = false;
    }

    /// Mark layout as dirty (window resize)
    pub fn mark_layout_dirty(&mut self) {
        self.layout = true;
    }

    /// Mark content as dirty (state change, new message, user input)
    pub fn mark_content_dirty(&mut self) {
        self.content = true;
    }

    /// Mark effects as dirty (animations running)
    pub fn mark_effects_dirty(&mut self) {
        self.effects = true;
    }

    /// Force a render on the next frame
    pub fn mark_force_render(&mut self) {
        self.force = true;
    }

    /// Mark all flags as dirty (complete re-render)
    pub fn mark_all_dirty(&mut self) {
        self.layout = true;
        self.content = true;
        self.effects = true;
        self.force = true;
    }
}

impl Default for DirtyFlags {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flags_not_dirty() {
        let flags = DirtyFlags::new();
        assert!(!flags.is_dirty());
    }

    #[test]
    fn test_force_render_is_dirty() {
        let flags = DirtyFlags::force_render();
        assert!(flags.is_dirty());
    }

    #[test]
    fn test_mark_content_dirty() {
        let mut flags = DirtyFlags::new();
        assert!(!flags.is_dirty());

        flags.mark_content_dirty();
        assert!(flags.is_dirty());
    }

    #[test]
    fn test_mark_layout_dirty() {
        let mut flags = DirtyFlags::new();
        assert!(!flags.is_dirty());

        flags.mark_layout_dirty();
        assert!(flags.is_dirty());
    }

    #[test]
    fn test_mark_effects_dirty() {
        let mut flags = DirtyFlags::new();
        assert!(!flags.is_dirty());

        flags.mark_effects_dirty();
        assert!(flags.is_dirty());
    }

    #[test]
    fn test_clear_flags() {
        let mut flags = DirtyFlags::force_render();
        assert!(flags.is_dirty());

        flags.clear();
        assert!(!flags.is_dirty());
    }

    #[test]
    fn test_mark_all_dirty() {
        let mut flags = DirtyFlags::new();
        assert!(!flags.is_dirty());

        flags.mark_all_dirty();
        assert!(flags.is_dirty());

        // Clear and verify all flags were set
        flags.clear();
        assert!(!flags.is_dirty());
    }
}

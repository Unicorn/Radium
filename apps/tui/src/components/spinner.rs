//! Spinner animation infrastructure for status indicators.

/// Predefined spinner frame sequences.
pub struct SpinnerFrames;

impl SpinnerFrames {
    /// Braille spinner frames (default, smooth animation)
    pub const BRAILLE: &'static [&'static str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    /// Circle spinner frames (alternative style)
    pub const CIRCLE: &'static [&'static str] = &["◐", "◓", "◑", "◒"];

    /// Dots spinner frames (alternative style)
    pub const DOTS: &'static [&'static str] = &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
}

/// Spinner animation handler.
pub struct Spinner {
    /// Frame sequence to use
    frames: &'static [&'static str],
    /// Static icon to show when animations are disabled
    static_icon: &'static str,
}

impl Spinner {
    /// Creates a new spinner with the default braille frame sequence.
    pub fn new() -> Self {
        Self {
            frames: SpinnerFrames::BRAILLE,
            static_icon: SpinnerFrames::BRAILLE[0],
        }
    }

    /// Creates a new spinner with a custom frame sequence.
    pub fn with_frames(frames: &'static [&'static str], static_icon: &'static str) -> Self {
        Self { frames, static_icon }
    }

    /// Returns the current frame for the given frame counter.
    /// 
    /// The frame counter should increment on each render cycle (target 60fps).
    /// When animations are disabled or reduced_motion is enabled, returns the static icon.
    pub fn current_frame(
        &self,
        frame_counter: usize,
        animations_enabled: bool,
        reduced_motion: bool,
    ) -> &'static str {
        if !animations_enabled || reduced_motion {
            return self.static_icon;
        }

        if self.frames.is_empty() {
            return self.static_icon;
        }

        let frame_index = frame_counter % self.frames.len();
        self.frames[frame_index]
    }

    /// Returns the static icon (first frame) for this spinner.
    pub fn static_icon(&self) -> &'static str {
        self.static_icon
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_frame_cycling() {
        let spinner = Spinner::new();
        
        // Test cycling through frames
        for i in 0..10 {
            let frame = spinner.current_frame(i, true, false);
            assert_eq!(frame, SpinnerFrames::BRAILLE[i % SpinnerFrames::BRAILLE.len()]);
        }
    }

    #[test]
    fn test_spinner_wraps_at_end() {
        let spinner = Spinner::new();
        let frame_count = SpinnerFrames::BRAILLE.len();
        
        // Test that it wraps correctly
        let frame_0 = spinner.current_frame(0, true, false);
        let frame_wrapped = spinner.current_frame(frame_count, true, false);
        assert_eq!(frame_0, frame_wrapped);
    }

    #[test]
    fn test_spinner_reduced_motion() {
        let spinner = Spinner::new();
        
        // With reduced motion, should return static icon
        let frame = spinner.current_frame(5, true, true);
        assert_eq!(frame, spinner.static_icon());
    }

    #[test]
    fn test_spinner_animations_disabled() {
        let spinner = Spinner::new();
        
        // With animations disabled, should return static icon
        let frame = spinner.current_frame(5, false, false);
        assert_eq!(frame, spinner.static_icon());
    }

    #[test]
    fn test_custom_spinner_frames() {
        let custom_frames = &["A", "B", "C"];
        let spinner = Spinner::with_frames(custom_frames, "A");
        
        assert_eq!(spinner.current_frame(0, true, false), "A");
        assert_eq!(spinner.current_frame(1, true, false), "B");
        assert_eq!(spinner.current_frame(2, true, false), "C");
        assert_eq!(spinner.current_frame(3, true, false), "A"); // Wraps
    }
}


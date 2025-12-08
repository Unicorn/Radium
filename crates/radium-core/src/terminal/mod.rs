//! Terminal capability detection and color conversion.
//!
//! Provides utilities for detecting terminal color support and converting
//! colors between different color spaces (truecolor, 256-color, 16-color).

pub mod capabilities;
pub mod color_conversion;

pub use capabilities::{TerminalCapabilities, ColorSupport};
pub use color_conversion::{rgb_to_256, rgb_to_16};


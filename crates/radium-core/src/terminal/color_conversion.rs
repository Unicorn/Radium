//! Color conversion utilities.
//!
//! Converts RGB colors to lower color spaces (256-color, 16-color)
//! for terminals with limited color support.

/// Convert RGB color to nearest 256-color palette index.
///
/// Uses the standard 256-color palette:
/// - 0-15: Standard ANSI colors
/// - 16-231: 6x6x6 color cube
/// - 232-255: Grayscale ramp
pub fn rgb_to_256(r: u8, g: u8, b: u8) -> u8 {
    // If color is very close to a standard ANSI color, use it
    // Otherwise, use the 6x6x6 color cube
    
    // Map to 6 levels (0, 95, 135, 175, 215, 255)
    let r6 = ((r as u16 * 5) / 255) as u8;
    let g6 = ((g as u16 * 5) / 255) as u8;
    let b6 = ((b as u16 * 5) / 255) as u8;
    
    // Calculate index in 6x6x6 cube (16-231)
    16 + (r6 * 36) + (g6 * 6) + b6
}

/// Convert RGB color to nearest 16-color ANSI code.
///
/// Maps RGB to the standard 16 ANSI colors using Euclidean distance.
pub fn rgb_to_16(r: u8, g: u8, b: u8) -> u8 {
    // Standard 16-color ANSI palette (approximate RGB values)
    let ansi_colors: [(u8, u8, u8); 16] = [
        (0, 0, 0),       // 0: Black
        (128, 0, 0),     // 1: Red
        (0, 128, 0),     // 2: Green
        (128, 128, 0),   // 3: Yellow
        (0, 0, 128),     // 4: Blue
        (128, 0, 128),   // 5: Magenta
        (0, 128, 128),   // 6: Cyan
        (192, 192, 192), // 7: White (light gray)
        (128, 128, 128), // 8: Bright Black (dark gray)
        (255, 0, 0),     // 9: Bright Red
        (0, 255, 0),     // 10: Bright Green
        (255, 255, 0),   // 11: Bright Yellow
        (0, 0, 255),     // 12: Bright Blue
        (255, 0, 255),   // 13: Bright Magenta
        (0, 255, 255),   // 14: Bright Cyan
        (255, 255, 255), // 15: Bright White
    ];

    let mut min_distance = u32::MAX;
    let mut best_index = 0;

    for (i, &(ar, ag, ab)) in ansi_colors.iter().enumerate() {
        // Calculate Euclidean distance
        let dr = (r as i32 - ar as i32) as u32;
        let dg = (g as i32 - ag as i32) as u32;
        let db = (b as i32 - ab as i32) as u32;
        let distance = dr * dr + dg * dg + db * db;

        if distance < min_distance {
            min_distance = distance;
            best_index = i;
        }
    }

    best_index as u8
}

/// Convert RGB tuple to color code based on terminal capabilities.
///
/// Returns the appropriate color representation for the terminal's
/// color support level.
pub fn rgb_to_terminal_color(r: u8, g: u8, b: u8, support: crate::terminal::ColorSupport) -> String {
    match support {
        crate::terminal::ColorSupport::Truecolor => {
            format!("\x1b[38;2;{};{};{}m", r, g, b)
        }
        crate::terminal::ColorSupport::Color256 => {
            let index = rgb_to_256(r, g, b);
            format!("\x1b[38;5;{}m", index)
        }
        crate::terminal::ColorSupport::Color16 => {
            let index = rgb_to_16(r, g, b);
            format!("\x1b[{}m", if index < 8 { 30 + index } else { 90 + (index - 8) })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_256() {
        // Test that conversion produces valid 256-color indices
        let index = rgb_to_256(255, 0, 0);
        assert!(index >= 16 && index <= 231, "Index should be in color cube range");
    }

    #[test]
    fn test_rgb_to_16() {
        // Test that conversion produces valid 16-color indices
        let index = rgb_to_16(255, 0, 0);
        assert!(index < 16, "Index should be < 16");
    }

    #[test]
    fn test_black_conversion() {
        // Black should map to black (0) in 16-color
        assert_eq!(rgb_to_16(0, 0, 0), 0);
    }

    #[test]
    fn test_white_conversion() {
        // White should map to bright white (15) in 16-color
        assert_eq!(rgb_to_16(255, 255, 255), 15);
    }
}


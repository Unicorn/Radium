# Radium Brand Colors

This document describes the Radium brand color palette and how to use it consistently across the codebase.

## Brand Color Palette

The Radium brand uses the following color palette:

- **Primary**: Cyan (#00D9FF / RGB(0, 217, 255)) - Main brand color
- **Secondary**: Purple (#A78BFA / RGB(167, 139, 250)) - Secondary brand color
- **Purple Accent**: (#6250d0 / RGB(98, 80, 208)) - Logo accents
- **Success**: Green (#10B981 / RGB(16, 185, 129)) - Success states
- **Warning**: Yellow (#F59E0B / RGB(245, 158, 11)) - Warnings
- **Error**: Red (#EF4444 / RGB(239, 68, 68)) - Errors
- **Info**: Blue (#06B6D4 / RGB(6, 182, 212)) - Informational messages

## Usage

### CLI Applications

For CLI applications, use the `RadiumBrandColors` utility module:

```rust
use crate::colors::RadiumBrandColors;
use colored::Colorize;

let colors = RadiumBrandColors::new();

// Use brand colors
println!("{}", "Primary text".color(colors.primary()));
println!("{}", "Success message".color(colors.success()));
println!("{}", "Warning message".color(colors.warning()));
println!("{}", "Error message".color(colors.error()));
```

### TUI Applications

For TUI applications, use the `RadiumTheme` with the "radium" preset:

```rust
use radium_tui::theme::RadiumTheme;

let theme = RadiumTheme::radium();
// Use theme.primary, theme.success, etc.
```

Or configure in TUI config:

```toml
[theme]
preset = "radium"
```

## Cross-Platform Support

The `RadiumBrandColors` utility automatically detects terminal color capabilities:

- **Truecolor (24-bit RGB)**: Full brand colors
- **256-color**: Approximated brand colors
- **16-color**: Closest ANSI color match

Color conversion is handled automatically based on terminal capabilities.

## Migration Guide

To migrate existing code from hardcoded colors to brand colors:

1. Import `RadiumBrandColors`:
   ```rust
   use crate::colors::RadiumBrandColors;
   ```

2. Create a colors instance in your function:
   ```rust
   let colors = RadiumBrandColors::new();
   ```

3. Replace color methods:
   - `.cyan()` → `.color(colors.primary())`
   - `.green()` → `.color(colors.success())`
   - `.yellow()` → `.color(colors.warning())`
   - `.red()` → `.color(colors.error())`
   - `.blue()` → `.color(colors.info())`

## Implementation Status

### Completed

- ✅ `RadiumBrandColors` utility module (`apps/cli/src/colors.rs`)
- ✅ TUI "radium" theme preset (`apps/tui/src/theme.rs`)
- ✅ comfy-table styling (`apps/cli/src/commands/code.rs`)
- ✅ indicatif progress indicators (`apps/cli/src/commands/tool_execution.rs`, `crates/radium-core/src/workflow/progress_reporter.rs`)
- ✅ Priority CLI commands:
  - `braingrid.rs` - Fully migrated
  - `requirement.rs` - Partially migrated (execute function)
  - `main.rs` - Info messages migrated

### Remaining Work

The following CLI command files still use hardcoded colors and should be migrated incrementally:

- `apps/cli/src/commands/*.rs` (approximately 50+ files)

Migration pattern is established - remaining files can be updated using the same approach as `braingrid.rs`.

## Testing

### Manual Testing

Test color rendering on different platforms:

1. **macOS**: Terminal.app, iTerm2
2. **Linux**: Various terminal emulators (gnome-terminal, xterm, etc.)
3. **Windows**: Windows Terminal, PowerShell

### Color Support Levels

Test with different color support levels:

- 16-color: Set `TERM` to a basic terminal type
- 256-color: Set `TERM` to include "256color"
- Truecolor: Set `COLORTERM=truecolor`

### Verification

1. Run CLI commands and verify colors match brand palette
2. Test TUI with `preset = "radium"` in config
3. Verify color fallbacks work on limited-color terminals

## References

- Brand colors defined in: `apps/tui/src/theme.rs` (RadiumTheme::dark())
- CLI utility: `apps/cli/src/colors.rs` (RadiumBrandColors)
- Terminal capabilities: `crates/radium-core/src/terminal/capabilities.rs`
- Color conversion: `crates/radium-core/src/terminal/color_conversion.rs`

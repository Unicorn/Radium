//! Theme management commands for CLI.
//!
//! Provides commands to list, set, and preview available themes.

use anyhow::{Context, Result};
use clap::Subcommand;
use colored::*;
use radium_tui::config::TuiConfig;
use radium_tui::theme::RadiumTheme;
use std::path::PathBuf;

/// Theme management subcommands
#[derive(Subcommand, Debug)]
pub enum ThemeCommand {
    /// List all available theme presets
    List,
    /// Set the active theme preset
    Set {
        /// Theme name (dark, light, github, monokai, onedark, solarized-dark, dracula, custom)
        name: String,
    },
    /// Preview a theme's color palette
    Preview {
        /// Theme name to preview
        name: String,
    },
}

/// Execute theme command
pub async fn execute(cmd: ThemeCommand) -> Result<()> {
    match cmd {
        ThemeCommand::List => list_themes().await,
        ThemeCommand::Set { name } => set_theme(&name).await,
        ThemeCommand::Preview { name } => preview_theme(&name).await,
    }
}

/// List all available themes
async fn list_themes() -> Result<()> {
    let themes = vec![
        ("dark", "Default dark theme (Radium default)"),
        ("light", "Light theme for bright environments"),
        ("github", "GitHub-style light theme with blue accents"),
        ("monokai", "Classic Monokai dark theme with vibrant colors"),
        ("onedark", "Atom One Dark theme with blue-green accents"),
        ("solarized-dark", "Solarized Dark balanced palette"),
        ("dracula", "Dracula purple/pink dark theme"),
        ("custom", "Custom theme with user-defined colors"),
    ];

    println!("{}", "Available themes:".bold());
    println!();

    for (name, description) in themes {
        let current = is_current_theme(name)?;
        let marker = if current { "→ " } else { "  " };
        let name_display = if current {
            name.bright_green().bold()
        } else {
            name.white()
        };
        println!("{} {} - {}", marker, name_display, description);
    }

    Ok(())
}

/// Set the active theme
async fn set_theme(name: &str) -> Result<()> {
    let valid_themes = vec![
        "dark", "light", "github", "monokai", "onedark", "solarized-dark", "dracula", "custom",
    ];

    if !valid_themes.contains(&name.to_lowercase().as_str()) {
        eprintln!("{} Invalid theme name: {}", "Error:".red().bold(), name);
        eprintln!();
        eprintln!("Valid themes are:");
        for theme in valid_themes {
            eprintln!("  - {}", theme);
        }
        return Err(anyhow::anyhow!("Invalid theme name: {}", name));
    }

    let mut config = TuiConfig::load()
        .context("Failed to load configuration")?;

    config.theme.preset = name.to_lowercase();

    config.save()
        .context("Failed to save configuration")?;

    println!("{} Theme set to: {}", "✓".green(), name.bright_green().bold());
    println!();
    println!("The new theme will be applied the next time you start the TUI.");

    Ok(())
}

/// Preview a theme's color palette
async fn preview_theme(name: &str) -> Result<()> {
    let theme = match name.to_lowercase().as_str() {
        "dark" => RadiumTheme::dark(),
        "light" => RadiumTheme::light(),
        "github" => RadiumTheme::github(),
        "monokai" => RadiumTheme::monokai(),
        "onedark" => RadiumTheme::onedark(),
        "solarized-dark" => RadiumTheme::solarized(),
        "dracula" => RadiumTheme::dracula(),
        _ => {
            return Err(anyhow::anyhow!("Unknown theme: {}. Use 'radium theme list' to see available themes.", name));
        }
    };

    println!("{}", format!("Theme Preview: {}", name).bold());
    println!();

    // Display color swatches
    print_color_swatch("Primary", theme.primary);
    print_color_swatch("Secondary", theme.secondary);
    print_color_swatch("Success", theme.success);
    print_color_swatch("Warning", theme.warning);
    print_color_swatch("Error", theme.error);
    print_color_swatch("Info", theme.info);
    print_color_swatch("Text", theme.text);
    print_color_swatch("Text Muted", theme.text_muted);
    print_color_swatch("Background", theme.bg_primary);
    print_color_swatch("Panel Background", theme.bg_panel);

    Ok(())
}

/// Print a color swatch
fn print_color_swatch(label: &str, color: ratatui::style::Color) {
    let (r, g, b) = match color {
        ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
        _ => (128, 128, 128), // Fallback for non-RGB colors
    };

    let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
    let colored_block = "███".truecolor(r, g, b);
    
    // Pad label to 20 characters
    let padded_label = format!("{:<20}", label);
    println!("  {} {} {}", colored_block, padded_label, hex);
}

/// Check if a theme is currently active
fn is_current_theme(name: &str) -> Result<bool> {
    let config = TuiConfig::load()
        .context("Failed to load configuration")?;
    Ok(config.theme.preset.to_lowercase() == name.to_lowercase())
}


//! Radium TUI - Unified Prompt Interface
//!
//! A CLI-like terminal interface with slash commands and chat functionality.

use std::io::{self, stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use radium_tui::app::App;
use radium_tui::views::{render_prompt, render_setup_wizard, render_splash};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize minimal logging
    let log_enabled = std::env::var("RUST_LOG_TUI").is_ok();
    if log_enabled {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "radium_tui=warn,error".into()),
            )
            .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "off".into()),
            )
            .with(tracing_subscriber::fmt::layer().with_writer(|| std::io::sink()))
            .init();
    }

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Show splash screen
    let start_time = std::time::Instant::now();
    let splash_duration = Duration::from_millis(800);

    while start_time.elapsed() < splash_duration {
        terminal.draw(|frame| {
            render_splash(frame, frame.area(), "Loading workspace...");
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Skip splash on any key press
                    break;
                }
            }
        }
    }

    // Create app
    let mut app = App::new();

    // Main loop
    loop {
        // Draw UI
        terminal.draw(|frame| {
            let area = frame.area();

            // Render setup wizard if active, otherwise render normal prompt
            if let Some(wizard) = &app.setup_wizard {
                render_setup_wizard(frame, area, wizard);
            } else {
                render_prompt(frame, area, &app.prompt_data);
            }
        })?;

        // Handle events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code, key.modifiers).await?;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    let _ = disable_raw_mode();
    let _ = stdout().execute(LeaveAlternateScreen);

    Ok(())
}

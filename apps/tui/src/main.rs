//! Radium TUI - Terminal User Interface
//!
//! A terminal-based dashboard for managing Radium agents and workflows.

use std::io::{self, stdout};
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use radium_tui::app::App;
use radium_tui::navigation::View;
use radium_tui::views::{render_agent_view, render_dashboard, render_task_view, render_workflow_view};

/// Default server address
const DEFAULT_SERVER_ADDR: &str = "http://127.0.0.1:50051";

/// Radium TUI - Terminal dashboard for Radium
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Server address to connect to
    #[arg(short, long, default_value = DEFAULT_SERVER_ADDR)]
    server: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging - but disable output during TUI to avoid interfering with rendering
    // Logs will be captured in debug_logs instead
    // Only show logs if RUST_LOG is explicitly set and we're not in TUI mode
    let log_to_stderr = std::env::var("RUST_LOG_TUI").is_ok();
    if log_to_stderr {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "radium_tui=warn,error".into()),
            )
            .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
            .init();
    } else {
        // Initialize a no-op subscriber to prevent tracing panics
        // Use a closure that returns a sink to satisfy MakeWriter trait
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

    // Parse command line arguments
    let args = Args::parse();

    // Create app with configured server address
    let mut app = App::new(args.server);

    // Try to connect on startup
    if let Err(e) = app.app_state.connect().await {
        let err_msg = format!("Failed to connect to server: {}", e);
        app.debug_logs.push(format!("ERROR: {}", err_msg));
        // Don't show error immediately - let refresh_current_view handle it
    } else {
        app.debug_logs.push("INFO: Connected to server".to_string());
        // Load initial dashboard data
        app.refresh_current_view().await;
    }

    // Setup signal handler for Ctrl+C (cross-platform)
    // Spawn a task that listens for Ctrl+C and sets the quit flag
    let app_quit = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let app_quit_clone = app_quit.clone();
    tokio::spawn(async move {
        if let Err(_e) = tokio::signal::ctrl_c().await {
            // Silently fail - signal handler setup error is not critical
            return;
        }
        app_quit_clone.store(true, std::sync::atomic::Ordering::Relaxed);
    });

    // Main loop
    loop {
        // Draw UI
        terminal.draw(|frame| {
            let area = frame.area();
            match app.navigation.current_view() {
                View::Dashboard => {
                    if let Some(data) = &app.dashboard_data {
                        render_dashboard(frame, area, &app.app_state, data);
                    } else {
                        let error_text =
                            app.error_message.as_deref().unwrap_or("Failed to load dashboard data");
                        let error = ratatui::widgets::Paragraph::new(error_text)
                            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red))
                            .wrap(ratatui::widgets::Wrap { trim: true })
                            .block(
                                ratatui::widgets::Block::default()
                                    .borders(ratatui::widgets::Borders::ALL)
                                    .title(" Error "),
                            );
                        frame.render_widget(error, area);
                    }
                }
                View::Agents => {
                    if let Some(data) = &app.agent_data {
                        render_agent_view(frame, area, data);
                    } else {
                        let error_text =
                            app.error_message.as_deref().unwrap_or("Failed to load agent data");
                        let error = ratatui::widgets::Paragraph::new(error_text)
                            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red))
                            .block(
                                ratatui::widgets::Block::default()
                                    .borders(ratatui::widgets::Borders::ALL)
                                    .title(" Error "),
                            );
                        frame.render_widget(error, area);
                    }
                }
                View::Workflows => {
                    if let Some(data) = &app.workflow_data {
                        render_workflow_view(frame, area, data);
                    } else {
                        let error_text =
                            app.error_message.as_deref().unwrap_or("Failed to load workflow data");
                        let error = ratatui::widgets::Paragraph::new(error_text)
                            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red))
                            .block(
                                ratatui::widgets::Block::default()
                                    .borders(ratatui::widgets::Borders::ALL)
                                    .title(" Error "),
                            );
                        frame.render_widget(error, area);
                    }
                }
                View::Tasks => {
                    if let Some(data) = &app.task_data {
                        render_task_view(frame, area, data);
                    } else {
                        let error_text =
                            app.error_message.as_deref().unwrap_or("Failed to load task data");
                        let error = ratatui::widgets::Paragraph::new(error_text)
                            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red))
                            .block(
                                ratatui::widgets::Block::default()
                                    .borders(ratatui::widgets::Borders::ALL)
                                    .title(" Error "),
                            );
                        frame.render_widget(error, area);
                    }
                }
            }
        })?;

        // Check if Ctrl+C was pressed
        if app_quit.load(std::sync::atomic::Ordering::Relaxed) {
            app.should_quit = true;
        }

        // Handle events with timeout for async operations
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code, key.modifiers).await?;
                }
            }
        }

        // Auto-refresh dashboard every 5 seconds
        if matches!(app.navigation.current_view(), View::Dashboard) {
            // This is a simple approach - in a real app you'd use a timer
            // For now, we'll refresh on 'r' key press
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal (ensure this happens even on error)
    let _ = disable_raw_mode();
    let _ = stdout().execute(LeaveAlternateScreen);

    Ok(())
}
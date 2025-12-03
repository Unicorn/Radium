//! Radium TUI - Terminal User Interface
//!
//! A terminal-based dashboard for managing Radium agents and workflows.

mod app;
mod navigation;
mod views;

use std::io::{self, stdout};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use app::AppState;
use navigation::{Navigation, View};
use views::{render_agent_view, render_dashboard, render_task_view, render_workflow_view};

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

/// Main application
struct App {
    /// Whether to quit the application
    should_quit: bool,
    /// Application state
    #[allow(clippy::struct_field_names)]
    app_state: AppState,
    /// Navigation state
    navigation: Navigation,
    /// Dashboard data cache
    dashboard_data: Option<views::dashboard::DashboardData>,
    /// Agent view data cache
    agent_data: Option<views::agent::AgentViewData>,
    /// Workflow view data cache
    workflow_data: Option<views::workflow::WorkflowViewData>,
    /// Task view data cache
    task_data: Option<views::task::TaskViewData>,
    /// Error message for current view
    error_message: Option<String>,
    /// Debug log buffer
    debug_logs: Vec<String>,
}

impl App {
    fn new(server_addr: String) -> Self {
        Self {
            should_quit: false,
            app_state: AppState::new(server_addr),
            navigation: Navigation::new(),
            dashboard_data: None,
            agent_data: None,
            workflow_data: None,
            task_data: None,
            error_message: None,
            debug_logs: Vec::new(),
        }
    }

    async fn capture_debug_info(&mut self) -> String {
        use std::fmt::Write;

        let mut output = String::new();

        // Connection status
        let is_connected = {
            let client_guard = self.app_state.client.lock().await;
            client_guard.is_some()
        };
        writeln!(output, "=== Radium TUI Debug Info ===\n").unwrap();
        writeln!(output, "Timestamp: {}", chrono::Utc::now().to_rfc3339()).unwrap();
        writeln!(output, "Server: {}", self.app_state.server_addr).unwrap();
        writeln!(output, "Connected: {}", is_connected).unwrap();
        writeln!(output, "Current View: {:?}", self.navigation.current_view()).unwrap();
        writeln!(output, "\n=== Data Cache Status ===").unwrap();
        writeln!(
            output,
            "Dashboard Data: {}",
            if self.dashboard_data.is_some() { "Loaded" } else { "Not loaded" }
        )
        .unwrap();
        writeln!(
            output,
            "Agent Data: {}",
            if self.agent_data.is_some() { "Loaded" } else { "Not loaded" }
        )
        .unwrap();
        writeln!(
            output,
            "Workflow Data: {}",
            if self.workflow_data.is_some() { "Loaded" } else { "Not loaded" }
        )
        .unwrap();
        writeln!(
            output,
            "Task Data: {}",
            if self.task_data.is_some() { "Loaded" } else { "Not loaded" }
        )
        .unwrap();

        if let Some(ref err) = self.error_message {
            writeln!(output, "\n=== Current Error ===").unwrap();
            writeln!(output, "{}", err).unwrap();
        }

        if !self.debug_logs.is_empty() {
            writeln!(output, "\n=== Recent Logs (last 50) ===").unwrap();
            for log in self.debug_logs.iter().rev().take(50) {
                writeln!(output, "{}", log).unwrap();
            }
        }

        output
    }

    async fn refresh_current_view(&mut self) {
        // Check if connected first
        let is_connected = {
            let client_guard = self.app_state.client.lock().await;
            client_guard.is_some()
        };

        if !is_connected {
            self.error_message = Some(format!(
                "Not connected to server.\nServer: {}\n\nPress 'c' to connect or 'q' to quit.",
                self.app_state.server_addr
            ));
            return;
        }

        // Clear previous error
        self.error_message = None;

        match self.navigation.current_view() {
            View::Dashboard => {
                match views::dashboard::DashboardData::fetch(&self.app_state).await {
                    Ok(data) => {
                        self.dashboard_data = Some(data);
                        self.error_message = None;
                    }
                    Err(e) => {
                        // Store error in debug logs
                        let err_str = format!("Failed to load dashboard data: {}", e);
                        self.debug_logs.push(format!("ERROR: {}", err_str));
                        if self.debug_logs.len() > 100 {
                            self.debug_logs.remove(0);
                        }
                        // Show user-friendly error message
                        let error_msg = format!(
                            "Failed to load dashboard data\n\nError: {}\n\nPress 'r' to retry or 'c' to reconnect\nPress Ctrl+I to capture debug info",
                            e
                        );
                        self.error_message = Some(error_msg);
                        self.dashboard_data = None;
                    }
                }
            }
            View::Agents => match views::agent::AgentViewData::fetch(&self.app_state).await {
                Ok(data) => {
                    self.agent_data = Some(data);
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load agent data: {}", e));
                    self.agent_data = None;
                }
            },
            View::Workflows => {
                match views::workflow::WorkflowViewData::fetch(&self.app_state).await {
                    Ok(data) => {
                        self.workflow_data = Some(data);
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load workflow data: {}", e));
                        self.workflow_data = None;
                    }
                }
            }
            View::Tasks => match views::task::TaskViewData::fetch(&self.app_state).await {
                Ok(data) => {
                    self.task_data = Some(data);
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load task data: {}", e));
                    self.task_data = None;
                }
            },
        }
    }

    async fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+C
                self.should_quit = true;
            }
            KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+D (EOF)
                self.should_quit = true;
            }
            KeyCode::Char('1') => {
                self.navigation.set_view(View::Dashboard);
                self.refresh_current_view().await;
            }
            KeyCode::Char('2') => {
                self.navigation.set_view(View::Agents);
                self.refresh_current_view().await;
            }
            KeyCode::Char('3') => {
                self.navigation.set_view(View::Workflows);
                self.refresh_current_view().await;
            }
            KeyCode::Char('4') => {
                self.navigation.set_view(View::Tasks);
                self.refresh_current_view().await;
            }
            KeyCode::Up => match self.navigation.current_view() {
                View::Agents => {
                    if let Some(data) = &mut self.agent_data {
                        data.previous_agent();
                    }
                }
                View::Workflows => {
                    if let Some(data) = &mut self.workflow_data {
                        data.previous_workflow();
                    }
                }
                View::Tasks => {
                    if let Some(data) = &mut self.task_data {
                        data.previous_task();
                    }
                }
                _ => {}
            },
            KeyCode::Down => match self.navigation.current_view() {
                View::Agents => {
                    if let Some(data) = &mut self.agent_data {
                        data.next_agent();
                    }
                }
                View::Workflows => {
                    if let Some(data) = &mut self.workflow_data {
                        data.next_workflow();
                    }
                }
                View::Tasks => {
                    if let Some(data) = &mut self.task_data {
                        data.next_task();
                    }
                }
                _ => {}
            },
            KeyCode::Char('r') => {
                self.refresh_current_view().await;
            }
            KeyCode::Char('c') => {
                // Try to reconnect
                let log_msg =
                    format!("Attempting to reconnect to server: {}", self.app_state.server_addr);
                self.debug_logs.push(format!("INFO: {}", log_msg));
                if self.debug_logs.len() > 100 {
                    self.debug_logs.remove(0);
                }

                match self.app_state.connect().await {
                    Err(e) => {
                        let err_msg = format!("Failed to reconnect: {}", e);
                        self.debug_logs.push(format!("ERROR: {}", err_msg));
                        self.error_message =
                            Some(format!("Failed to connect: {}\n\nPress 'c' to try again", e));
                    }
                    _ => {
                        self.debug_logs.push("INFO: Reconnected successfully".to_string());
                        self.error_message = None;
                        self.refresh_current_view().await;
                    }
                }
            }
            KeyCode::Char('i') if modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+I: Capture debug info to file
                let debug_info = self.capture_debug_info().await;
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let filename = format!("radium-tui-debug-{}.txt", timestamp);
                let path = PathBuf::from(&filename);

                if let Err(e) = std::fs::write(&path, &debug_info) {
                    self.error_message = Some(format!("Failed to write debug file: {}", e));
                } else {
                    self.error_message =
                        Some(format!("Debug info saved to: {}\n\nPress 'r' to refresh", filename));
                }
            }
            _ => {}
        }
        Ok(())
    }
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
    match app.app_state.connect().await {
        Err(e) => {
            let err_msg = format!("Failed to connect to server: {}", e);
            app.debug_logs.push(format!("ERROR: {}", err_msg));
            // Don't show error immediately - let refresh_current_view handle it
        }
        _ => {
            app.debug_logs.push("INFO: Connected to server".to_string());
            // Load initial dashboard data
            app.refresh_current_view().await;
        }
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

//! Application state management

use radium_core::radium_client::RadiumClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use std::path::PathBuf;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};

use crate::navigation::{Navigation, View};
use crate::views;

/// Application state
#[derive(Clone)]
pub struct AppState {
    /// gRPC client (if connected)
    pub client: Arc<Mutex<Option<RadiumClient<Channel>>>>,
    /// Server address
    pub server_addr: String,
    /// Connection status
    pub connection_status: Arc<Mutex<String>>,
}

impl AppState {
    pub fn new(server_addr: String) -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            server_addr,
            connection_status: Arc::new(Mutex::new("Disconnected".to_string())),
        }
    }

    pub async fn connect(&self) -> anyhow::Result<()> {
        *self.connection_status.lock().await = format!("Connecting to {}...", self.server_addr);

        let channel = Channel::from_shared(self.server_addr.clone())
            .map_err(|e| anyhow::anyhow!("Invalid URI: {}", e))?
            .connect()
            .await?;

        let client = RadiumClient::new(channel);
        *self.client.lock().await = Some(client);
        *self.connection_status.lock().await = format!("Connected to {}", self.server_addr);

        Ok(())
    }
}

/// Main application
pub struct App {
    /// Whether to quit the application
    pub should_quit: bool,
    /// Application state
    pub app_state: AppState,
    /// Navigation state
    pub navigation: Navigation,
    /// Dashboard data cache
    pub dashboard_data: Option<views::dashboard::DashboardData>,
    /// Agent view data cache
    pub agent_data: Option<views::agent::AgentViewData>,
    /// Workflow view data cache
    pub workflow_data: Option<views::workflow::WorkflowViewData>,
    /// Task view data cache
    pub task_data: Option<views::task::TaskViewData>,
    /// Error message for current view
    pub error_message: Option<String>,
    /// Debug log buffer
    pub debug_logs: Vec<String>,
}

impl App {
    pub fn new(server_addr: String) -> Self {
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

    pub async fn capture_debug_info(&mut self) -> String {
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

    pub async fn refresh_current_view(&mut self) {
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

    pub async fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
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

                if let Err(e) = self.app_state.connect().await {
                    let err_msg = format!("Failed to reconnect: {}", e);
                    self.debug_logs.push(format!("ERROR: {}", err_msg));
                    self.error_message =
                        Some(format!("Failed to connect: {}\n\nPress 'c' to try again", e));
                } else {
                    self.debug_logs.push("INFO: Reconnected successfully".to_string());
                    self.error_message = None;
                    self.refresh_current_view().await;
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

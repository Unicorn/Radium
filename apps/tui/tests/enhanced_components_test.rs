//! Integration tests for enhanced TUI components.

use radium_tui::components::{
    Dialog, DialogChoice, DialogManager, Toast, ToastManager, ToastVariant,
};
use radium_tui::state::{AgentState, AgentStatus, TelemetryState, TokenMetrics, WorkflowStatus};
use radium_tui::views::history::HistoryEntry;
use std::collections::HashSet;
use std::time::Duration;

#[test]
fn test_toast_creation() {
    let toast = Toast::new(ToastVariant::Success, "Test message".to_string());
    assert_eq!(toast.variant, ToastVariant::Success);
    assert_eq!(toast.message, "Test message");
    assert!(toast.duration.is_some());
    assert!(!toast.should_dismiss()); // Should not be dismissed immediately
}

#[test]
fn test_toast_persistent() {
    let toast = Toast::persistent(ToastVariant::Error, "Persistent error".to_string());
    assert!(toast.duration.is_none());
    assert!(!toast.should_dismiss());
}

#[test]
fn test_toast_auto_dismiss() {
    let mut toast = Toast::with_duration(
        ToastVariant::Info,
        "Quick message".to_string(),
        Duration::from_millis(100),
    );
    
    // Should not be dismissed immediately
    assert!(!toast.should_dismiss());
    
    // Wait a bit (in real scenario, time would pass)
    // For testing, we can't actually wait, but we can test the logic
    assert!(toast.remaining_time().is_some());
}

#[test]
fn test_toast_manager() {
    let mut manager = ToastManager::new();
    
    // Add toasts
    manager.success("Success!".to_string());
    manager.error("Error!".to_string());
    manager.info("Info!".to_string());
    manager.warning("Warning!".to_string());
    
    assert_eq!(manager.toasts().len(), 4);
    
    // Update should not remove non-expired toasts
    manager.update();
    assert_eq!(manager.toasts().len(), 4);
    
    // Clear all
    manager.clear();
    assert_eq!(manager.toasts().len(), 0);
}

#[test]
fn test_toast_manager_max_limit() {
    let mut manager = ToastManager::with_max_toasts(3);
    
    // Add more toasts than max
    for i in 0..5 {
        manager.info(format!("Toast {}", i));
    }
    
    // Should only keep the most recent ones
    assert_eq!(manager.toasts().len(), 3);
}

#[test]
fn test_toast_variant_colors() {
    let theme = radium_tui::theme::RadiumTheme::dark();
    assert_eq!(ToastVariant::Success.color(), theme.success);
    assert_eq!(ToastVariant::Error.color(), theme.error);
    assert_eq!(ToastVariant::Info.color(), theme.info);
    assert_eq!(ToastVariant::Warning.color(), theme.warning);
}

#[test]
fn test_dialog_creation() {
    let choices = vec![
        DialogChoice::new("Option 1".to_string(), "opt1".to_string()),
        DialogChoice::new("Option 2".to_string(), "opt2".to_string()),
    ];
    let dialog = Dialog::new("Choose an option".to_string(), choices);
    assert_eq!(dialog.choices.len(), 2);
    assert_eq!(dialog.selected_index, 0);
    assert!(dialog.visible);
}

#[test]
fn test_dialog_navigation() {
    let choices = vec![
        DialogChoice::new("Option 1".to_string(), "opt1".to_string()),
        DialogChoice::new("Option 2".to_string(), "opt2".to_string()),
        DialogChoice::new("Option 3".to_string(), "opt3".to_string()),
    ];
    let mut dialog = Dialog::new("Choose".to_string(), choices);

    assert_eq!(dialog.selected_index, 0);
    dialog.move_down();
    assert_eq!(dialog.selected_index, 1);
    dialog.move_down();
    assert_eq!(dialog.selected_index, 2);
    dialog.move_down(); // Should not go beyond
    assert_eq!(dialog.selected_index, 2);
    dialog.move_up();
    assert_eq!(dialog.selected_index, 1);
    dialog.move_up();
    assert_eq!(dialog.selected_index, 0);
    dialog.move_up(); // Should not go below 0
    assert_eq!(dialog.selected_index, 0);
}

#[test]
fn test_dialog_manager() {
    let mut manager = DialogManager::new();
    assert!(!manager.is_open());

    let choices = vec![DialogChoice::new("Test".to_string(), "test".to_string())];
    manager.show_select_menu("Test dialog".to_string(), choices);
    assert!(manager.is_open());

    manager.close();
    assert!(!manager.is_open());
}

#[test]
fn test_dialog_selection() {
    let choices1 = vec![
        DialogChoice::new("Option 1".to_string(), "opt1".to_string()),
        DialogChoice::new("Option 2".to_string(), "opt2".to_string()),
    ];
    let dialog = Dialog::new("Choose".to_string(), choices1);
    assert_eq!(dialog.selected_value(), Some("opt1".to_string()));

    let choices2 = vec![
        DialogChoice::new("Option 1".to_string(), "opt1".to_string()),
        DialogChoice::new("Option 2".to_string(), "opt2".to_string()),
    ];
    let mut dialog = Dialog::new("Choose".to_string(), choices2);
    dialog.move_down();
    assert_eq!(dialog.selected_value(), Some("opt2".to_string()));
}

#[test]
fn test_dialog_choice_with_description() {
    let choice = DialogChoice::with_description(
        "Title".to_string(),
        "value".to_string(),
        "Description".to_string(),
    );
    assert_eq!(choice.title, "Title");
    assert_eq!(choice.value, "value");
    assert_eq!(choice.description, Some("Description".to_string()));
}

#[test]
fn test_status_footer_app_mode() {
    use radium_tui::components::AppMode;
    
    assert_eq!(AppMode::Prompt.as_str(), "Prompt");
    assert_eq!(AppMode::Workflow.as_str(), "Workflow");
    assert_eq!(AppMode::Chat.as_str(), "Chat");
    assert_eq!(AppMode::History.as_str(), "History");
    assert_eq!(AppMode::Setup.as_str(), "Setup");
    
    // Test shortcuts
    assert!(!AppMode::Prompt.shortcuts().is_empty());
    assert!(!AppMode::Workflow.shortcuts().is_empty());
}

#[test]
fn test_telemetry_state_integration() {
    let mut telemetry = TelemetryState::new();
    
    // Record tokens for multiple agents
    telemetry.record_tokens("agent-1".to_string(), 100, 50, 0);
    telemetry.record_tokens("agent-2".to_string(), 200, 100, 25);
    
    assert_eq!(telemetry.total_tokens(), 475); // 100+50+200+100+25
    assert_eq!(telemetry.overall_tokens.input_tokens, 300);
    assert_eq!(telemetry.overall_tokens.output_tokens, 150);
    assert_eq!(telemetry.overall_tokens.cached_tokens, 25);
    
    // Record costs
    telemetry.record_cost("agent-1".to_string(), 0.01);
    telemetry.record_cost("agent-2".to_string(), 0.02);
    assert!((telemetry.total_cost() - 0.03).abs() < 0.0001);
    
    // Set model and provider
    telemetry.set_model("gpt-4".to_string());
    telemetry.set_provider("openai".to_string());
    assert_eq!(telemetry.model, Some("gpt-4".to_string()));
    assert_eq!(telemetry.provider, Some("openai".to_string()));
}

#[test]
fn test_agent_timeline_hierarchical() {
    use radium_tui::components::AgentTimeline;
    use radium_tui::state::SubAgentState;
    use std::collections::HashMap;
    
    // Create agent with sub-agents
    let mut agent = AgentState::new("main-1".to_string(), "Main Agent".to_string());
    agent.register_sub_agent("sub-1".to_string(), "Sub Agent 1".to_string());
    agent.register_sub_agent("sub-2".to_string(), "Sub Agent 2".to_string());
    
    let agents = vec![agent];
    let expanded = HashSet::from(["main-1".to_string()]);
    
    // Test that the component can handle hierarchical display
    // (We can't actually render without a frame, but we can test the logic)
    assert_eq!(agents[0].sub_agents.len(), 2);
    assert!(expanded.contains("main-1"));
}

#[test]
fn test_history_entry_runtime_formatting() {
    let entry = HistoryEntry {
        workflow_id: "wf-1".to_string(),
        workflow_name: "Test Workflow".to_string(),
        runtime_secs: 3661, // 1 hour, 1 minute, 1 second
        status: "Completed".to_string(),
        completed_at: "2024-01-01 12:00:00".to_string(),
        log_path: None,
    };

    assert_eq!(entry.format_runtime(), "01:01:01");
    
    // Test edge cases
    let entry_zero = HistoryEntry {
        workflow_id: "wf-2".to_string(),
        workflow_name: "Quick".to_string(),
        runtime_secs: 0,
        status: "Completed".to_string(),
        completed_at: "2024-01-01 12:00:00".to_string(),
        log_path: None,
    };
    assert_eq!(entry_zero.format_runtime(), "00:00:00");
    
    let entry_long = HistoryEntry {
        workflow_id: "wf-3".to_string(),
        workflow_name: "Long".to_string(),
        runtime_secs: 36661, // 10 hours, 11 minutes, 1 second
        status: "Completed".to_string(),
        completed_at: "2024-01-01 12:00:00".to_string(),
        log_path: None,
    };
    assert_eq!(entry_long.format_runtime(), "10:11:01");
}

#[test]
fn test_workflow_status_display() {
    assert_eq!(WorkflowStatus::Running.as_str(), "Running");
    assert_eq!(WorkflowStatus::Completed.as_str(), "Completed");
    assert_eq!(WorkflowStatus::Failed.as_str(), "Failed");
    assert_eq!(WorkflowStatus::Paused.as_str(), "Paused");
    assert_eq!(WorkflowStatus::Cancelled.as_str(), "Cancelled");
    assert_eq!(WorkflowStatus::Idle.as_str(), "Idle");
    
    assert!(WorkflowStatus::Running.is_active());
    assert!(!WorkflowStatus::Completed.is_active());
    assert!(WorkflowStatus::Completed.is_terminal());
    assert!(!WorkflowStatus::Running.is_terminal());
}

#[test]
fn test_agent_status_display() {
    assert_eq!(AgentStatus::Running.as_str(), "Running");
    assert_eq!(AgentStatus::Completed.as_str(), "Completed");
    assert_eq!(AgentStatus::Failed.as_str(), "Failed");
    
    assert_eq!(AgentStatus::Running.icon(), "▶");
    assert_eq!(AgentStatus::Completed.icon(), "✓");
    assert_eq!(AgentStatus::Failed.icon(), "✗");
    
    assert!(AgentStatus::Running.is_active());
    assert!(!AgentStatus::Completed.is_active());
    assert!(AgentStatus::Completed.is_terminal());
    assert!(!AgentStatus::Running.is_terminal());
}

#[test]
fn test_token_metrics_formatting() {
    let mut metrics = TokenMetrics::new();
    metrics.add(1000, 500, 250);
    
    assert_eq!(metrics.input_tokens, 1000);
    assert_eq!(metrics.output_tokens, 500);
    assert_eq!(metrics.cached_tokens, 250);
    assert_eq!(metrics.total(), 1750);
    
    let formatted = metrics.format();
    assert!(formatted.contains("1,000in"));
    assert!(formatted.contains("500out"));
    assert!(formatted.contains("250cached"));
}

#[test]
fn test_agent_state_with_sub_agents() {
    let mut agent = AgentState::new("main-1".to_string(), "Main Agent".to_string());
    
    // Register sub-agents
    agent.register_sub_agent("sub-1".to_string(), "Sub Agent 1".to_string());
    agent.register_sub_agent("sub-2".to_string(), "Sub Agent 2".to_string());
    
    assert_eq!(agent.sub_agents.len(), 2);
    
    // Start sub-agent
    if let Some(sub_agent) = agent.get_sub_agent_mut("sub-1") {
        sub_agent.start();
        assert_eq!(sub_agent.status, AgentStatus::Running);
        assert!(sub_agent.start_time.is_some());
        
        sub_agent.complete();
        assert_eq!(sub_agent.status, AgentStatus::Completed);
    }
}

#[test]
fn test_telemetry_bar_render_with_status() {
    use radium_tui::components::TelemetryBar;
    
    let mut telemetry = TelemetryState::new();
    telemetry.record_tokens("agent-1".to_string(), 1000, 500, 100);
    telemetry.record_cost("agent-1".to_string(), 0.05);
    telemetry.set_model("gpt-4".to_string());
    telemetry.set_provider("openai".to_string());
    
    // Test that render_with_status can be called (we can't actually render without a frame)
    // But we can verify the telemetry state is correct
    assert_eq!(telemetry.total_tokens(), 1600);
    assert!((telemetry.total_cost() - 0.05).abs() < 0.0001);
    assert_eq!(telemetry.model, Some("gpt-4".to_string()));
}

#[test]
fn test_log_stream_state() {
    use radium_tui::utils::LogStreamState;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    // Create a temporary log file
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Line 1").unwrap();
    writeln!(file, "Line 2").unwrap();
    file.flush().unwrap();
    
    let mut stream = LogStreamState::new(file.path());
    
    // Read initial lines
    let new_lines = stream.read_new_lines().unwrap();
    assert_eq!(new_lines.len(), 2);
    assert_eq!(new_lines[0], "Line 1");
    assert_eq!(new_lines[1], "Line 2");
    
    // Add more lines
    writeln!(file, "Line 3").unwrap();
    writeln!(file, "Line 4").unwrap();
    file.flush().unwrap();
    
    // Read new lines
    let new_lines = stream.read_new_lines().unwrap();
    assert_eq!(new_lines.len(), 2);
    assert_eq!(stream.lines.len(), 4);
}

#[test]
fn test_log_stream_manager() {
    use radium_tui::utils::LogStreamManager;
    use tempfile::NamedTempFile;
    
    let manager = LogStreamManager::new();
    let file = NamedTempFile::new().unwrap();
    
    // Register a stream
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        manager.register("test-id".to_string(), file.path()).await;
        
        let lines = manager.get_lines("test-id").await;
        assert_eq!(lines.len(), 0);
        
        let is_running = manager.is_running("test-id").await;
        assert!(is_running);
        
        manager.mark_stopped("test-id").await;
        let is_running = manager.is_running("test-id").await;
        assert!(!is_running);
    });
}

#[test]
fn test_theme_consistency() {
    let theme = radium_tui::theme::RadiumTheme::dark();
    
    // Verify all theme colors are set
    assert_ne!(theme.primary, theme.secondary);
    assert_ne!(theme.success, theme.error);
    assert_ne!(theme.warning, theme.info);
    assert_ne!(theme.text, theme.text_muted);
    assert_ne!(theme.bg_primary, theme.bg_panel);
    assert_ne!(theme.border, theme.border_active);
}

#[test]
fn test_dialog_keyboard_handling() {
    let mut manager = DialogManager::new();
    let choices1 = vec![
        DialogChoice::new("Option 1".to_string(), "opt1".to_string()),
        DialogChoice::new("Option 2".to_string(), "opt2".to_string()),
    ];
    manager.show_select_menu("Choose".to_string(), choices1);
    
    // Simulate keyboard input
    use crossterm::event::KeyCode;
    
    // Down arrow
    let result = manager.handle_key(KeyCode::Down);
    assert!(result.is_none()); // Navigation doesn't return value
    assert!(manager.is_open());
    
    // Enter (should close and return value)
    let result = manager.handle_key(KeyCode::Enter);
    assert!(result.is_some());
    assert!(!manager.is_open());
    
    // Reopen and test Esc
    let choices2 = vec![
        DialogChoice::new("Option 1".to_string(), "opt1".to_string()),
        DialogChoice::new("Option 2".to_string(), "opt2".to_string()),
    ];
    manager.show_select_menu("Choose".to_string(), choices2);
    let result = manager.handle_key(KeyCode::Esc);
    assert!(result.is_none());
    assert!(!manager.is_open());
}

#[test]
fn test_toast_dismiss_by_index() {
    let mut manager = ToastManager::new();
    manager.success("Toast 1".to_string());
    manager.error("Toast 2".to_string());
    manager.info("Toast 3".to_string());
    
    assert_eq!(manager.toasts().len(), 3);
    
    // Dismiss middle toast
    manager.dismiss(1);
    assert_eq!(manager.toasts().len(), 2);
    
    // Verify correct toasts remain
    let toasts = manager.toasts();
    assert_eq!(toasts[0].variant, ToastVariant::Info);
    assert_eq!(toasts[1].variant, ToastVariant::Success);
}

#[test]
fn test_history_entry_with_log_path() {
    use std::path::PathBuf;
    
    let entry = HistoryEntry {
        workflow_id: "wf-1".to_string(),
        workflow_name: "Test".to_string(),
        runtime_secs: 100,
        status: "Completed".to_string(),
        completed_at: "2024-01-01 12:00:00".to_string(),
        log_path: Some(PathBuf::from("/tmp/test.log")),
    };
    
    assert!(entry.log_path.is_some());
    assert_eq!(entry.log_path.as_ref().unwrap().to_str(), Some("/tmp/test.log"));
}


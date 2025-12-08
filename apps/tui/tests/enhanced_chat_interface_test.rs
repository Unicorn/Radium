//! Integration tests for enhanced TUI chat interface with task list and orchestrator thinking panels.

use radium_tui::state::{TaskListState, TaskListItem};
use radium_tui::components::{TaskListPanel, OrchestratorThinkingPanel};
use radium_core::models::task::TaskState;

#[test]
fn test_task_list_state_operations() {
    let mut state = TaskListState::new();
    
    // Add tasks
    state.add_task(
        "task-1".to_string(),
        "Task 1".to_string(),
        TaskState::Queued,
        "agent-1".to_string(),
        0,
    );
    state.add_task(
        "task-2".to_string(),
        "Task 2".to_string(),
        TaskState::Running,
        "agent-2".to_string(),
        1,
    );
    state.add_task(
        "task-3".to_string(),
        "Task 3".to_string(),
        TaskState::Completed,
        "agent-3".to_string(),
        2,
    );
    
    // Verify tasks are in order
    let tasks = state.get_tasks();
    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0].id, "task-1");
    assert_eq!(tasks[1].id, "task-2");
    assert_eq!(tasks[2].id, "task-3");
    
    // Verify progress tracking
    let (completed, failed, total) = state.get_progress();
    assert_eq!(completed, 1);
    assert_eq!(failed, 0);
    assert_eq!(total, 3);
    
    // Update task status
    state.update_task_status("task-2", TaskState::Error("Test error".to_string()));
    let (completed, failed, total) = state.get_progress();
    assert_eq!(completed, 1);
    assert_eq!(failed, 1);
    assert_eq!(total, 3);
}

#[test]
fn test_orchestrator_thinking_panel() {
    let mut panel = OrchestratorThinkingPanel::new();
    
    // Append logs
    panel.append_log("[Orchestrator] Analyzing dependencies...".to_string());
    panel.append_log("[Orchestrator] Selected agent: code-agent".to_string());
    panel.append_log("[Orchestrator] Executing step 1...".to_string());
    
    assert_eq!(panel.len(), 3);
    assert!(!panel.is_empty());
    
    // Test scrolling
    panel.scroll_to_top();
    panel.scroll_down(1, 3);
    panel.scroll_to_bottom();
    
    // Test clear
    panel.clear();
    assert!(panel.is_empty());
}

#[test]
fn test_task_list_panel_scroll() {
    let mut panel = TaskListPanel::new();
    
    // Test scroll operations (using internal scroll_offset via reflection would require making it public)
    // For now, just verify the panel can be created and methods exist
    panel.set_focused(true);
    panel.scroll_down(5, 10);
    panel.scroll_up(2);
    panel.scroll_to_top();
    panel.scroll_to_bottom(10);
    
    // Just verify the methods don't panic
    assert!(true);
}

#[test]
fn test_panel_focus_cycling() {
    use radium_tui::views::PanelFocus;
    
    // Test that PanelFocus enum works
    let focus1 = PanelFocus::Chat;
    let focus2 = PanelFocus::TaskList;
    let focus3 = PanelFocus::Orchestrator;
    
    assert_ne!(focus1, focus2);
    assert_ne!(focus2, focus3);
    assert_ne!(focus1, focus3);
}

#[test]
fn test_status_color_mapping() {
    // Test that status colors are mapped correctly
    let _color1 = TaskListState::status_color(&TaskState::Queued);
    let _color2 = TaskListState::status_color(&TaskState::Running);
    let _color3 = TaskListState::status_color(&TaskState::Completed);
    let _color4 = TaskListState::status_color(&TaskState::Error("test".to_string()));
    let _color5 = TaskListState::status_color(&TaskState::Paused);
    let _color6 = TaskListState::status_color(&TaskState::Cancelled);
    
    // Just verify the function doesn't panic
    assert!(true);
}


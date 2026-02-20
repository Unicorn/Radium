//! Task list state management for tracking tasks with agent assignments.
//!
//! This module provides state management for displaying task lists in the TUI,
//! tracking task status, agent assignments, and progress.

use radium_core::models::task::TaskState;
use std::collections::HashMap;
use crate::theme::get_theme;

/// Represents a single task item in the task list.
#[derive(Debug, Clone)]
pub struct TaskListItem {
    /// Task ID
    pub id: String,
    /// Task name
    pub name: String,
    /// Task status
    pub status: TaskState,
    /// Agent ID assigned to this task
    pub agent_id: String,
    /// Order index for maintaining task order
    pub order: usize,
}

/// State management for task list tracking.
#[derive(Debug, Clone)]
pub struct TaskListState {
    /// Tasks indexed by task ID for O(1) lookups
    tasks: HashMap<String, TaskListItem>,
    /// Task IDs in order for maintaining display order
    task_order: Vec<String>,
    /// Total number of tasks
    total_tasks: usize,
    /// Number of completed tasks
    completed_tasks: usize,
    /// Number of failed tasks (Error state)
    failed_tasks: usize,
}

impl TaskListState {
    /// Creates a new empty task list state.
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            task_order: Vec::new(),
            total_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
        }
    }

    /// Adds a task to the list, maintaining order.
    ///
    /// # Arguments
    /// * `id` - Task ID
    /// * `name` - Task name
    /// * `status` - Task status
    /// * `agent_id` - Agent ID assigned to the task
    /// * `order` - Order index for display
    pub fn add_task(&mut self, id: String, name: String, status: TaskState, agent_id: String, order: usize) {
        // If task already exists, update it
        if self.tasks.contains_key(&id) {
            self.update_task_status(&id, status);
            return;
        }

        // Add new task
        let item = TaskListItem {
            id: id.clone(),
            name,
            status: status.clone(),
            agent_id,
            order,
        };

        // Update counters based on initial status
        match status {
            TaskState::Completed => {
                self.completed_tasks += 1;
            }
            TaskState::Error(_) => {
                self.failed_tasks += 1;
            }
            _ => {}
        }

        self.tasks.insert(id.clone(), item);
        
        // Insert into order vector at the correct position
        if order >= self.task_order.len() {
            self.task_order.push(id);
        } else {
            self.task_order.insert(order, id);
        }
        
        self.total_tasks += 1;
    }

    /// Updates the status of an existing task.
    ///
    /// # Arguments
    /// * `task_id` - Task ID to update
    /// * `new_status` - New task status
    pub fn update_task_status(&mut self, task_id: &str, new_status: TaskState) {
        if let Some(item) = self.tasks.get_mut(task_id) {
            let old_status = item.status.clone();
            
            // Update counters: remove old status, add new status
            match old_status {
                TaskState::Completed => {
                    self.completed_tasks = self.completed_tasks.saturating_sub(1);
                }
                TaskState::Error(_) => {
                    self.failed_tasks = self.failed_tasks.saturating_sub(1);
                }
                _ => {}
            }
            
            match new_status {
                TaskState::Completed => {
                    self.completed_tasks += 1;
                }
                TaskState::Error(_) => {
                    self.failed_tasks += 1;
                }
                _ => {}
            }
            
            item.status = new_status;
        }
    }

    /// Returns an ordered list of all tasks.
    pub fn get_tasks(&self) -> Vec<&TaskListItem> {
        self.task_order
            .iter()
            .filter_map(|id| self.tasks.get(id))
            .collect()
    }

    /// Returns progress information: (completed, failed, total).
    pub fn get_progress(&self) -> (usize, usize, usize) {
        (self.completed_tasks, self.failed_tasks, self.total_tasks)
    }

    /// Maps a TaskState to a theme color for display.
    ///
    /// # Arguments
    /// * `state` - Task state to map
    ///
    /// # Returns
    /// Theme color for the task state
    pub fn status_color(state: &TaskState) -> ratatui::style::Color {
        let theme = get_theme();
        match state {
            TaskState::Queued => theme.text_muted,
            TaskState::Running => theme.warning,
            TaskState::Completed => theme.success,
            TaskState::Error(_) => theme.error,
            TaskState::Paused => theme.info,
            TaskState::Cancelled => theme.text_dim,
        }
    }

    /// Clears all tasks from the state.
    pub fn clear(&mut self) {
        self.tasks.clear();
        self.task_order.clear();
        self.total_tasks = 0;
        self.completed_tasks = 0;
        self.failed_tasks = 0;
    }

    /// Returns the number of tasks in the list.
    pub fn len(&self) -> usize {
        self.total_tasks
    }

    /// Returns whether the task list is empty.
    pub fn is_empty(&self) -> bool {
        self.total_tasks == 0
    }
}

impl Default for TaskListState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_list_state_new() {
        let state = TaskListState::new();
        assert_eq!(state.len(), 0);
        assert!(state.is_empty());
        assert_eq!(state.get_progress(), (0, 0, 0));
    }

    #[test]
    fn test_add_task() {
        let mut state = TaskListState::new();
        state.add_task(
            "task-1".to_string(),
            "Task 1".to_string(),
            TaskState::Queued,
            "agent-1".to_string(),
            0,
        );
        
        assert_eq!(state.len(), 1);
        assert!(!state.is_empty());
        let tasks = state.get_tasks();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "task-1");
        assert_eq!(tasks[0].name, "Task 1");
        assert_eq!(tasks[0].agent_id, "agent-1");
    }

    #[test]
    fn test_task_ordering() {
        let mut state = TaskListState::new();
        state.add_task("task-1".to_string(), "Task 1".to_string(), TaskState::Queued, "agent-1".to_string(), 0);
        state.add_task("task-2".to_string(), "Task 2".to_string(), TaskState::Queued, "agent-2".to_string(), 1);
        state.add_task("task-3".to_string(), "Task 3".to_string(), TaskState::Queued, "agent-3".to_string(), 2);
        
        let tasks = state.get_tasks();
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].id, "task-1");
        assert_eq!(tasks[1].id, "task-2");
        assert_eq!(tasks[2].id, "task-3");
    }

    #[test]
    fn test_update_task_status() {
        let mut state = TaskListState::new();
        state.add_task("task-1".to_string(), "Task 1".to_string(), TaskState::Queued, "agent-1".to_string(), 0);
        
        assert_eq!(state.get_progress(), (0, 0, 1));
        
        state.update_task_status("task-1", TaskState::Running);
        assert_eq!(state.get_progress(), (0, 0, 1));
        
        state.update_task_status("task-1", TaskState::Completed);
        assert_eq!(state.get_progress(), (1, 0, 1));
        
        state.update_task_status("task-1", TaskState::Error("Test error".to_string()));
        assert_eq!(state.get_progress(), (0, 1, 1));
    }

    #[test]
    fn test_progress_tracking() {
        let mut state = TaskListState::new();
        state.add_task("task-1".to_string(), "Task 1".to_string(), TaskState::Queued, "agent-1".to_string(), 0);
        state.add_task("task-2".to_string(), "Task 2".to_string(), TaskState::Running, "agent-2".to_string(), 1);
        state.add_task("task-3".to_string(), "Task 3".to_string(), TaskState::Queued, "agent-3".to_string(), 2);
        
        assert_eq!(state.get_progress(), (0, 0, 3));
        
        state.update_task_status("task-1", TaskState::Completed);
        state.update_task_status("task-2", TaskState::Error("Error".to_string()));
        
        assert_eq!(state.get_progress(), (1, 1, 3));
    }

    #[test]
    fn test_clear() {
        let mut state = TaskListState::new();
        state.add_task("task-1".to_string(), "Task 1".to_string(), TaskState::Queued, "agent-1".to_string(), 0);
        state.add_task("task-2".to_string(), "Task 2".to_string(), TaskState::Completed, "agent-2".to_string(), 1);
        
        assert_eq!(state.len(), 2);
        assert_eq!(state.get_progress(), (1, 0, 2));
        
        state.clear();
        
        assert_eq!(state.len(), 0);
        assert!(state.is_empty());
        assert_eq!(state.get_progress(), (0, 0, 0));
    }

    #[test]
    fn test_status_color() {
        // Just verify the function doesn't panic and returns a color
        let _color1 = TaskListState::status_color(&TaskState::Queued);
        let _color2 = TaskListState::status_color(&TaskState::Running);
        let _color3 = TaskListState::status_color(&TaskState::Completed);
        let _color4 = TaskListState::status_color(&TaskState::Error("test".to_string()));
        let _color5 = TaskListState::status_color(&TaskState::Paused);
        let _color6 = TaskListState::status_color(&TaskState::Cancelled);
    }
}




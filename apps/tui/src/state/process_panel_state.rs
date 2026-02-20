//! State management for the process panel.

use radium_core::process::ProcessHandle;
use radium_orchestrator::FixProposal;
use std::collections::VecDeque;
use std::time::SystemTime;

/// State for the process panel.
#[derive(Debug, Clone)]
pub struct ProcessPanelState {
    /// List of all tracked processes.
    pub processes: Vec<ProcessHandle>,
    /// Index of the currently selected process (if any).
    pub selected_index: Option<usize>,
    /// Queue of pending fix approval requests.
    pub pending_approvals: VecDeque<FixProposal>,
    /// Last time the process list was refreshed.
    pub last_refresh: SystemTime,
    /// Scroll offset for the process list.
    pub scroll_offset: usize,
}

impl ProcessPanelState {
    /// Creates a new process panel state.
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            selected_index: None,
            pending_approvals: VecDeque::new(),
            last_refresh: SystemTime::now(),
            scroll_offset: 0,
        }
    }

    /// Updates the list of processes.
    pub fn update_processes(&mut self, processes: Vec<ProcessHandle>) {
        self.processes = processes;
        self.last_refresh = SystemTime::now();

        // Adjust selected index if out of bounds
        if let Some(idx) = self.selected_index {
            if idx >= self.processes.len() {
                self.selected_index = if self.processes.is_empty() {
                    None
                } else {
                    Some(self.processes.len() - 1)
                };
            }
        }
    }

    /// Adds a fix approval request to the queue.
    pub fn add_approval_request(&mut self, proposal: FixProposal) {
        self.pending_approvals.push_back(proposal);
    }

    /// Gets the next pending approval request.
    pub fn get_next_approval(&mut self) -> Option<FixProposal> {
        self.pending_approvals.pop_front()
    }

    /// Gets the currently selected process.
    pub fn get_selected_process(&self) -> Option<&ProcessHandle> {
        self.selected_index
            .and_then(|idx| self.processes.get(idx))
    }

    /// Selects the next process in the list.
    pub fn select_next(&mut self) {
        if self.processes.is_empty() {
            self.selected_index = None;
            return;
        }

        self.selected_index = Some(match self.selected_index {
            Some(idx) => (idx + 1).min(self.processes.len() - 1),
            None => 0,
        });
    }

    /// Selects the previous process in the list.
    pub fn select_previous(&mut self) {
        if self.processes.is_empty() {
            self.selected_index = None;
            return;
        }

        self.selected_index = Some(match self.selected_index {
            Some(idx) => idx.saturating_sub(1),
            None => 0,
        });
    }

    /// Clears the selection.
    pub fn clear_selection(&mut self) {
        self.selected_index = None;
    }

    /// Returns the number of pending approvals.
    pub fn pending_approval_count(&self) -> usize {
        self.pending_approvals.len()
    }

    /// Returns whether there are any pending approvals.
    pub fn has_pending_approvals(&self) -> bool {
        !self.pending_approvals.is_empty()
    }

    /// Scrolls up in the process list.
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// Scrolls down in the process list.
    pub fn scroll_down(&mut self, amount: usize) {
        let max_scroll = self.processes.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + amount).min(max_scroll);
    }
}

impl Default for ProcessPanelState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radium_core::process::ProcessStatus;
    use std::path::PathBuf;

    fn create_test_process(id: &str, pid: u32) -> ProcessHandle {
        ProcessHandle {
            id: id.to_string(),
            pid: Some(pid),
            command: "test".to_string(),
            args: vec![],
            working_dir: PathBuf::from("/test"),
            status: ProcessStatus::Running,
            spawned_at: SystemTime::now(),
            exited_at: None,
            exit_code: None,
            restart_count: 0,
            log_file: PathBuf::from("/tmp/test.log"),
            shutdown_requested: false,
        }
    }

    #[test]
    fn test_new_state() {
        let state = ProcessPanelState::new();
        assert!(state.processes.is_empty());
        assert_eq!(state.selected_index, None);
        assert_eq!(state.pending_approval_count(), 0);
    }

    #[test]
    fn test_update_processes() {
        let mut state = ProcessPanelState::new();
        let processes = vec![
            create_test_process("p1", 100),
            create_test_process("p2", 200),
        ];

        state.update_processes(processes);
        assert_eq!(state.processes.len(), 2);
    }

    #[test]
    fn test_selection_navigation() {
        let mut state = ProcessPanelState::new();
        let processes = vec![
            create_test_process("p1", 100),
            create_test_process("p2", 200),
            create_test_process("p3", 300),
        ];
        state.update_processes(processes);

        // Initially no selection
        assert_eq!(state.selected_index, None);

        // Select next
        state.select_next();
        assert_eq!(state.selected_index, Some(0));

        state.select_next();
        assert_eq!(state.selected_index, Some(1));

        // Select previous
        state.select_previous();
        assert_eq!(state.selected_index, Some(0));

        // Can't go below 0
        state.select_previous();
        assert_eq!(state.selected_index, Some(0));

        // Clear selection
        state.clear_selection();
        assert_eq!(state.selected_index, None);
    }

    #[test]
    fn test_get_selected_process() {
        let mut state = ProcessPanelState::new();
        let processes = vec![
            create_test_process("p1", 100),
            create_test_process("p2", 200),
        ];
        state.update_processes(processes);

        assert!(state.get_selected_process().is_none());

        state.select_next();
        let selected = state.get_selected_process();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "p1");
    }
}

//! Checkpoint tracking for workflow state persistence.

use std::time::SystemTime;

/// Checkpoint information
#[derive(Debug, Clone)]
pub struct CheckpointInfo {
    /// Checkpoint ID
    pub id: String,
    /// Checkpoint name/description
    pub name: String,
    /// Timestamp when checkpoint was created
    pub timestamp: SystemTime,
    /// Step number when checkpoint was created
    pub step_number: usize,
    /// Git commit hash if using git-based checkpointing
    pub commit_hash: Option<String>,
    /// Whether this checkpoint can be restored
    pub restorable: bool,
}

impl CheckpointInfo {
    /// Creates a new checkpoint info.
    pub fn new(id: String, name: String, step_number: usize) -> Self {
        Self {
            id,
            name,
            timestamp: SystemTime::now(),
            step_number,
            commit_hash: None,
            restorable: true,
        }
    }

    /// Sets the commit hash.
    pub fn with_commit_hash(mut self, hash: String) -> Self {
        self.commit_hash = Some(hash);
        self
    }

    /// Marks the checkpoint as not restorable.
    pub fn mark_not_restorable(&mut self) {
        self.restorable = false;
    }

    /// Formats the checkpoint as a string.
    pub fn format(&self) -> String {
        let time_str = format_system_time(&self.timestamp);
        let commit_str = if let Some(ref hash) = self.commit_hash {
            format!(" ({})", &hash[..8])
        } else {
            String::new()
        };

        format!("[Step {}] {} - {}{}", self.step_number, self.name, time_str, commit_str)
    }
}

/// Checkpoint state tracking
#[derive(Debug, Clone)]
pub struct CheckpointState {
    /// All checkpoints created during workflow execution
    pub checkpoints: Vec<CheckpointInfo>,
    /// Current checkpoint (if any)
    pub current_checkpoint: Option<String>,
    /// Whether checkpointing is enabled
    pub enabled: bool,
    /// Auto-checkpoint frequency (steps between auto-checkpoints)
    pub auto_checkpoint_frequency: Option<usize>,
    /// Loop iteration tracking
    pub loop_iterations: Vec<LoopIteration>,
    /// Filter/search text for checkpoint list
    pub filter_text: String,
    /// Whether diff preview is visible
    pub show_diff: bool,
    /// Current page for pagination (0-indexed)
    pub current_page: usize,
    /// Items per page for pagination
    pub items_per_page: usize,
}

/// Loop iteration tracking
#[derive(Debug, Clone)]
pub struct LoopIteration {
    /// Iteration number
    pub iteration: usize,
    /// Step number where loop started
    pub start_step: usize,
    /// Step number where loop ended (if completed)
    pub end_step: Option<usize>,
    /// Timestamp when iteration started
    pub start_time: SystemTime,
    /// Timestamp when iteration ended (if completed)
    pub end_time: Option<SystemTime>,
}

impl LoopIteration {
    /// Creates a new loop iteration.
    pub fn new(iteration: usize, start_step: usize) -> Self {
        Self {
            iteration,
            start_step,
            end_step: None,
            start_time: SystemTime::now(),
            end_time: None,
        }
    }

    /// Completes the loop iteration.
    pub fn complete(&mut self, end_step: usize) {
        self.end_step = Some(end_step);
        self.end_time = Some(SystemTime::now());
    }

    /// Returns whether the iteration is active.
    pub fn is_active(&self) -> bool {
        self.end_step.is_none()
    }
}

impl CheckpointState {
    /// Creates a new checkpoint state.
    pub fn new() -> Self {
        Self {
            checkpoints: Vec::new(),
            current_checkpoint: None,
            enabled: true,
            auto_checkpoint_frequency: Some(5),
            loop_iterations: Vec::new(),
            filter_text: String::new(),
            show_diff: false,
            current_page: 0,
            items_per_page: 15,
        }
    }

    /// Creates a new checkpoint.
    pub fn create_checkpoint(&mut self, name: String, step_number: usize) -> String {
        let id = format!("checkpoint-{}", self.checkpoints.len() + 1);
        let checkpoint = CheckpointInfo::new(id.clone(), name, step_number);
        self.checkpoints.push(checkpoint);
        self.current_checkpoint = Some(id.clone());
        id
    }

    /// Creates a checkpoint with a git commit hash.
    pub fn create_checkpoint_with_commit(
        &mut self,
        name: String,
        step_number: usize,
        commit_hash: String,
    ) -> String {
        let id = format!("checkpoint-{}", self.checkpoints.len() + 1);
        let checkpoint =
            CheckpointInfo::new(id.clone(), name, step_number).with_commit_hash(commit_hash);
        self.checkpoints.push(checkpoint);
        self.current_checkpoint = Some(id.clone());
        id
    }

    /// Gets a checkpoint by ID.
    pub fn get_checkpoint(&self, id: &str) -> Option<&CheckpointInfo> {
        self.checkpoints.iter().find(|c| c.id == id)
    }

    /// Gets the most recent checkpoint.
    pub fn get_latest_checkpoint(&self) -> Option<&CheckpointInfo> {
        self.checkpoints.last()
    }

    /// Starts a new loop iteration.
    pub fn start_loop_iteration(&mut self, iteration: usize, start_step: usize) {
        let loop_iter = LoopIteration::new(iteration, start_step);
        self.loop_iterations.push(loop_iter);
    }

    /// Completes the current loop iteration.
    pub fn complete_loop_iteration(&mut self, end_step: usize) {
        if let Some(loop_iter) = self.loop_iterations.last_mut() {
            if loop_iter.is_active() {
                loop_iter.complete(end_step);
            }
        }
    }

    /// Gets the current active loop iteration.
    pub fn get_current_loop_iteration(&self) -> Option<&LoopIteration> {
        self.loop_iterations.iter().rev().find(|l| l.is_active())
    }

    /// Returns the total number of loop iterations.
    pub fn total_loop_iterations(&self) -> usize {
        self.loop_iterations.len()
    }

    /// Enables checkpointing.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disables checkpointing.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Sets auto-checkpoint frequency.
    pub fn set_auto_checkpoint_frequency(&mut self, frequency: Option<usize>) {
        self.auto_checkpoint_frequency = frequency;
    }

    /// Checks if an auto-checkpoint should be created at the given step.
    pub fn should_auto_checkpoint(&self, step_number: usize) -> bool {
        if !self.enabled {
            return false;
        }

        if let Some(frequency) = self.auto_checkpoint_frequency {
            return step_number % frequency == 0;
        }

        false
    }

    /// Returns a summary of checkpoint state.
    pub fn summary(&self) -> String {
        let checkpoint_count = self.checkpoints.len();
        let loop_count = self.total_loop_iterations();

        if loop_count > 0 {
            format!("Checkpoints: {} | Loop iterations: {}", checkpoint_count, loop_count)
        } else {
            format!("Checkpoints: {}", checkpoint_count)
        }
    }

    /// Filters checkpoints based on filter text.
    pub fn filtered_checkpoints(&self) -> Vec<&CheckpointInfo> {
        if self.filter_text.is_empty() {
            return self.checkpoints.iter().collect();
        }

        let filter_lower = self.filter_text.to_lowercase();
        self.checkpoints
            .iter()
            .filter(|cp| {
                cp.id.to_lowercase().contains(&filter_lower)
                    || cp.name.to_lowercase().contains(&filter_lower)
            })
            .collect()
    }

    /// Gets the total number of pages for pagination.
    pub fn total_pages(&self) -> usize {
        let filtered_count = self.filtered_checkpoints().len();
        if filtered_count == 0 {
            return 1;
        }
        (filtered_count + self.items_per_page - 1) / self.items_per_page
    }

    /// Gets checkpoints for the current page.
    pub fn paginated_checkpoints(&self) -> Vec<&CheckpointInfo> {
        let filtered = self.filtered_checkpoints();
        let start = self.current_page * self.items_per_page;
        let end = std::cmp::min(start + self.items_per_page, filtered.len());
        filtered[start..end].to_vec()
    }

    /// Sets the filter text.
    pub fn set_filter(&mut self, filter: String) {
        self.filter_text = filter;
        self.current_page = 0; // Reset to first page when filtering
    }

    /// Clears the filter.
    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.current_page = 0;
    }

    /// Toggles diff preview.
    pub fn toggle_diff(&mut self) {
        self.show_diff = !self.show_diff;
    }

    /// Goes to the next page.
    pub fn next_page(&mut self) {
        let total = self.total_pages();
        if self.current_page < total.saturating_sub(1) {
            self.current_page += 1;
        }
    }

    /// Goes to the previous page.
    pub fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
        }
    }
}

impl Default for CheckpointState {
    fn default() -> Self {
        Self::new()
    }
}

/// Formats a SystemTime as a readable string.
fn format_system_time(time: &SystemTime) -> String {
    use std::time::UNIX_EPOCH;

    if let Ok(duration) = time.duration_since(UNIX_EPOCH) {
        let secs = duration.as_secs();
        let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now());
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "Unknown time".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_info() {
        let checkpoint =
            CheckpointInfo::new("cp-1".to_string(), "Before file modification".to_string(), 5);

        assert_eq!(checkpoint.id, "cp-1");
        assert_eq!(checkpoint.name, "Before file modification");
        assert_eq!(checkpoint.step_number, 5);
        assert!(checkpoint.restorable);
        assert!(checkpoint.commit_hash.is_none());

        let formatted = checkpoint.format();
        assert!(formatted.contains("[Step 5]"));
        assert!(formatted.contains("Before file modification"));
    }

    #[test]
    fn test_checkpoint_with_commit() {
        let checkpoint = CheckpointInfo::new("cp-1".to_string(), "Test checkpoint".to_string(), 3)
            .with_commit_hash("abc123def456".to_string());

        assert_eq!(checkpoint.commit_hash, Some("abc123def456".to_string()));

        let formatted = checkpoint.format();
        assert!(formatted.contains("(abc123de)"));
    }

    #[test]
    fn test_checkpoint_state() {
        let mut state = CheckpointState::new();

        assert!(state.enabled);
        assert_eq!(state.checkpoints.len(), 0);

        let id1 = state.create_checkpoint("Checkpoint 1".to_string(), 1);
        assert_eq!(state.checkpoints.len(), 1);
        assert_eq!(state.current_checkpoint, Some(id1.clone()));

        let id2 = state.create_checkpoint_with_commit(
            "Checkpoint 2".to_string(),
            5,
            "abc123".to_string(),
        );
        assert_eq!(state.checkpoints.len(), 2);
        assert_eq!(state.current_checkpoint, Some(id2));

        let checkpoint = state.get_checkpoint(&id1).unwrap();
        assert_eq!(checkpoint.name, "Checkpoint 1");

        let latest = state.get_latest_checkpoint().unwrap();
        assert_eq!(latest.name, "Checkpoint 2");
    }

    #[test]
    fn test_loop_iterations() {
        let mut state = CheckpointState::new();

        state.start_loop_iteration(1, 3);
        assert_eq!(state.total_loop_iterations(), 1);

        let current = state.get_current_loop_iteration().unwrap();
        assert_eq!(current.iteration, 1);
        assert_eq!(current.start_step, 3);
        assert!(current.is_active());

        state.complete_loop_iteration(7);
        let completed = state.loop_iterations.last().unwrap();
        assert_eq!(completed.end_step, Some(7));
        assert!(!completed.is_active());

        assert!(state.get_current_loop_iteration().is_none());
    }

    #[test]
    fn test_auto_checkpoint() {
        let mut state = CheckpointState::new();
        state.set_auto_checkpoint_frequency(Some(5));

        assert!(!state.should_auto_checkpoint(1));
        assert!(!state.should_auto_checkpoint(4));
        assert!(state.should_auto_checkpoint(5));
        assert!(state.should_auto_checkpoint(10));
        assert!(!state.should_auto_checkpoint(11));

        state.disable();
        assert!(!state.should_auto_checkpoint(5));

        state.enable();
        assert!(state.should_auto_checkpoint(5));

        state.set_auto_checkpoint_frequency(None);
        assert!(!state.should_auto_checkpoint(5));
    }
}

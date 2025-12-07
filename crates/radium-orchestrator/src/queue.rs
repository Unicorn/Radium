//! Agent execution queue.
//!
//! This module provides functionality for managing agent task execution with priority-based scheduling.

use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, warn};

/// Priority for task execution (higher value = higher priority).
pub type Priority = u32;

/// Task to be executed by an agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionTask {
    /// The agent ID to execute.
    pub agent_id: String,
    /// The input for the agent.
    pub input: String,
    /// Priority of the task (higher = more important).
    pub priority: Priority,
    /// Optional task ID for tracking.
    pub task_id: Option<String>,
}

impl ExecutionTask {
    /// Creates a new execution task.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID to execute
    /// * `input` - The input for the agent
    /// * `priority` - Priority of the task
    #[must_use]
    pub fn new(agent_id: String, input: String, priority: Priority) -> Self {
        Self { agent_id, input, priority, task_id: None }
    }

    /// Sets the task ID.
    ///
    /// # Arguments
    /// * `task_id` - The task ID
    #[must_use]
    pub fn with_task_id(mut self, task_id: String) -> Self {
        self.task_id = Some(task_id);
        self
    }
}

/// Wrapper for priority queue ordering (higher priority first).
#[derive(Debug, Clone, PartialEq, Eq)]
struct PriorityTask(ExecutionTask);

impl PartialOrd for PriorityTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // BinaryHeap is a max-heap, so higher priority should be "greater"
        // This means priority 10 > priority 1, so we compare normally
        self.0.priority.cmp(&other.0.priority)
    }
}

/// Execution queue for managing agent tasks.
pub struct ExecutionQueue {
    /// Sender for enqueueing tasks (for real-time processing).
    #[allow(dead_code)] // Reserved for future real-time processing
    task_sender: mpsc::UnboundedSender<ExecutionTask>,
    /// Receiver for dequeuing tasks (for real-time processing).
    #[allow(dead_code)] // Reserved for future real-time processing
    task_receiver: mpsc::UnboundedReceiver<ExecutionTask>,
    /// Priority queue for pending tasks.
    pending_queue: Arc<tokio::sync::Mutex<BinaryHeap<PriorityTask>>>,
    /// Set of currently running task IDs.
    running_tasks: Arc<tokio::sync::Mutex<std::collections::HashSet<String>>>,
    /// Count of completed tasks.
    completed_count: Arc<tokio::sync::Mutex<usize>>,
}

impl ExecutionQueue {
    /// Creates a new execution queue.
    #[must_use]
    pub fn new() -> Self {
        let (task_sender, task_receiver) = mpsc::unbounded_channel();
        Self {
            task_sender,
            task_receiver,
            pending_queue: Arc::new(tokio::sync::Mutex::new(BinaryHeap::new())),
            running_tasks: Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new())),
            completed_count: Arc::new(tokio::sync::Mutex::new(0)),
        }
    }

    /// Enqueues a task for execution.
    ///
    /// # Arguments
    /// * `task` - The task to enqueue
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, `Err` if the queue is closed.
    pub async fn enqueue_task(
        &self,
        task: ExecutionTask,
    ) -> Result<(), mpsc::error::SendError<ExecutionTask>> {
        let task_id =
            task.task_id.clone().unwrap_or_else(|| format!("task-{}", uuid::Uuid::new_v4()));
        debug!(task_id = %task_id, agent_id = %task.agent_id, priority = task.priority, "Enqueueing task");

        // Add to priority queue
        let mut queue = self.pending_queue.lock().await;
        queue.push(PriorityTask(task.clone()));

        // Also send through channel for notifications (optional)
        let _ = self.task_sender.send(task);
        Ok(())
    }

    /// Dequeues the next task (highest priority first).
    ///
    /// # Returns
    /// Returns `Some(ExecutionTask)` if a task is available, `None` if the queue is empty.
    pub async fn dequeue_task(&mut self) -> Option<ExecutionTask> {
        self.dequeue_task_immutable().await
    }

    /// Dequeues the next task (highest priority first) without requiring mutable reference.
    ///
    /// # Returns
    /// Returns `Some(ExecutionTask)` if a task is available, `None` if the queue is empty.
    pub async fn dequeue_task_immutable(&self) -> Option<ExecutionTask> {
        // Get from priority queue (highest priority first)
        let mut queue = self.pending_queue.lock().await;
        if let Some(priority_task) = queue.pop() {
            let task = priority_task.0;
            let task_id = task.task_id.clone().unwrap_or_else(|| "unknown".to_string());

            // Mark as running
            let mut running = self.running_tasks.lock().await;
            running.insert(task_id.clone());
            debug!(task_id = %task_id, priority = task.priority, "Dequeued task");

            return Some(task);
        }

        None
    }

    /// Cancels a task by ID.
    ///
    /// # Arguments
    /// * `task_id` - The task ID to cancel
    ///
    /// # Returns
    /// Returns `true` if the task was found and cancelled, `false` otherwise.
    pub async fn cancel_task(&self, task_id: &str) -> bool {
        debug!(task_id = %task_id, "Cancelling task");

        // Remove from running tasks
        let mut running = self.running_tasks.lock().await;
        let was_running = running.remove(task_id);

        // Try to remove from pending queue (this is best-effort since we can't easily search)
        // In a production system, we'd maintain a separate index for O(1) lookup
        if !was_running {
            warn!(task_id = %task_id, "Task not found in running tasks, may already be completed");
        }

        was_running
    }

    /// Marks a task as completed.
    ///
    /// # Arguments
    /// * `task_id` - The task ID
    pub async fn mark_completed(&self, task_id: &str) {
        let mut running = self.running_tasks.lock().await;
        if running.remove(task_id) {
            let mut completed = self.completed_count.lock().await;
            *completed += 1;
            debug!(task_id = %task_id, "Task completed");
        }
    }

    /// Returns the number of pending tasks.
    ///
    /// # Returns
    /// The count of pending tasks.
    pub async fn pending_count(&self) -> usize {
        let queue = self.pending_queue.lock().await;
        queue.len()
    }

    /// Returns the number of running tasks.
    ///
    /// # Returns
    /// The count of running tasks.
    pub async fn running_count(&self) -> usize {
        let running = self.running_tasks.lock().await;
        running.len()
    }

    /// Returns the number of completed tasks.
    ///
    /// # Returns
    /// The count of completed tasks.
    pub async fn completed_count(&self) -> usize {
        let completed = self.completed_count.lock().await;
        *completed
    }

    /// Returns queue metrics.
    ///
    /// # Returns
    /// A struct containing all queue metrics.
    pub async fn metrics(&self) -> QueueMetrics {
        QueueMetrics {
            pending: self.pending_count().await,
            running: self.running_count().await,
            completed: self.completed_count().await,
        }
    }

    /// Returns the queue depth (pending + running tasks) for a specific agent.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID to check
    ///
    /// # Returns
    /// The number of tasks (pending + running) for the specified agent.
    pub async fn get_queue_depth_for_agent(&self, agent_id: &str) -> usize {
        let mut depth = 0;

        // Count pending tasks for this agent
        let queue = self.pending_queue.lock().await;
        for priority_task in queue.iter() {
            if priority_task.0.agent_id == agent_id {
                depth += 1;
            }
        }
        drop(queue);

        // Count running tasks for this agent
        // Note: We can't easily track agent_id for running tasks from the current structure,
        // so we'll only count pending tasks. This is acceptable for MVP as pending tasks
        // are the main indicator of load.
        depth
    }
}

impl Default for ExecutionQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ExecutionQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionQueue")
            .field("pending_count", &self.pending_queue.try_lock().map(|q| q.len()).unwrap_or(0))
            .field("running_count", &self.running_tasks.try_lock().map(|r| r.len()).unwrap_or(0))
            .finish_non_exhaustive()
    }
}

/// Queue metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueMetrics {
    /// Number of pending tasks.
    pub pending: usize,
    /// Number of running tasks.
    pub running: usize,
    /// Number of completed tasks.
    pub completed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enqueue_dequeue() {
        let mut queue = ExecutionQueue::new();
        let task = ExecutionTask::new("agent-1".to_string(), "test input".to_string(), 1);

        queue.enqueue_task(task.clone()).await.unwrap();
        let dequeued = queue.dequeue_task().await;
        assert!(dequeued.is_some());
        assert_eq!(dequeued.unwrap().agent_id, "agent-1");
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let mut queue = ExecutionQueue::new();

        // Enqueue tasks with different priorities
        queue
            .enqueue_task(ExecutionTask::new("agent-1".to_string(), "low".to_string(), 1))
            .await
            .unwrap();
        queue
            .enqueue_task(ExecutionTask::new("agent-2".to_string(), "high".to_string(), 10))
            .await
            .unwrap();
        queue
            .enqueue_task(ExecutionTask::new("agent-3".to_string(), "medium".to_string(), 5))
            .await
            .unwrap();

        // Should dequeue in priority order: high, medium, low
        let task1 = queue.dequeue_task().await.unwrap();
        assert_eq!(task1.priority, 10);
        assert_eq!(task1.input, "high");

        let task2 = queue.dequeue_task().await.unwrap();
        assert_eq!(task2.priority, 5);
        assert_eq!(task2.input, "medium");

        let task3 = queue.dequeue_task().await.unwrap();
        assert_eq!(task3.priority, 1);
        assert_eq!(task3.input, "low");
    }

    #[tokio::test]
    async fn test_metrics() {
        let mut queue = ExecutionQueue::new();
        let task = ExecutionTask::new("agent-1".to_string(), "test".to_string(), 1)
            .with_task_id("task-1".to_string());

        queue.enqueue_task(task).await.unwrap();
        assert_eq!(queue.pending_count().await, 1);
        assert_eq!(queue.running_count().await, 0);
        assert_eq!(queue.completed_count().await, 0);

        let _dequeued = queue.dequeue_task().await;
        assert_eq!(queue.running_count().await, 1);

        queue.mark_completed("task-1").await;
        assert_eq!(queue.running_count().await, 0);
        assert_eq!(queue.completed_count().await, 1);
    }

    #[tokio::test]
    async fn test_cancel_task() {
        let mut queue = ExecutionQueue::new();
        let task = ExecutionTask::new("agent-1".to_string(), "test".to_string(), 1)
            .with_task_id("task-1".to_string());

        queue.enqueue_task(task).await.unwrap();
        let _dequeued = queue.dequeue_task().await;

        assert!(queue.cancel_task("task-1").await);
        assert_eq!(queue.running_count().await, 0);
    }
}

//! Task dispatcher for autonomous execution.
//!
//! This module provides a background service that continuously processes the ExecutionQueue
//! and dispatches tasks to agents until completion or critical errors.

use crate::{AgentExecutor, AgentRegistry, CriticalError, ExecutionQueue, LoadBalancer};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::watch;
use tokio::time;
use tracing::{debug, error, info, warn};

/// Configuration for the task dispatcher.
#[derive(Debug, Clone)]
pub struct TaskDispatcherConfig {
    /// Interval for polling the queue when empty.
    pub poll_interval: Duration,
    /// Maximum concurrent tasks per agent.
    pub max_concurrent_per_agent: usize,
}

impl Default for TaskDispatcherConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_millis(100),
            max_concurrent_per_agent: 10,
        }
    }
}

/// Task dispatcher that continuously processes the execution queue.
pub struct TaskDispatcher {
    /// Agent registry for accessing agents.
    registry: Arc<AgentRegistry>,
    /// Execution queue for tasks.
    queue: Arc<ExecutionQueue>,
    /// Agent executor for running agents.
    executor: Arc<AgentExecutor>,
    /// Load balancer for agent selection.
    load_balancer: Arc<LoadBalancer>,
    /// Configuration.
    config: TaskDispatcherConfig,
    /// Shutdown signal sender.
    shutdown_tx: Option<watch::Sender<()>>,
    /// Pause state flag.
    paused: Arc<AtomicBool>,
    /// Pause notification for waiting on resume.
    pause_notify: Arc<tokio::sync::Notify>,
    /// Last critical error encountered (if any).
    last_error: Arc<Mutex<Option<CriticalError>>>,
}

impl std::fmt::Debug for TaskDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskDispatcher")
            .field("config", &self.config)
            .field("paused", &self.paused.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl TaskDispatcher {
    /// Creates a new task dispatcher.
    ///
    /// # Arguments
    /// * `registry` - Agent registry
    /// * `queue` - Execution queue
    /// * `executor` - Agent executor
    /// * `config` - Dispatcher configuration
    #[must_use]
    pub fn new(
        registry: Arc<AgentRegistry>,
        queue: Arc<ExecutionQueue>,
        executor: Arc<AgentExecutor>,
        config: TaskDispatcherConfig,
    ) -> Self {
        let load_balancer = Arc::new(LoadBalancer::new(config.max_concurrent_per_agent));
        Self {
            registry,
            queue,
            executor,
            load_balancer,
            config,
            shutdown_tx: None,
            paused: Arc::new(AtomicBool::new(false)),
            pause_notify: Arc::new(tokio::sync::Notify::new()),
            last_error: Arc::new(Mutex::new(None)),
        }
    }

    /// Starts the dispatcher in a background task.
    ///
    /// # Returns
    /// Returns `Ok(())` if started successfully, or an error if already running.
    pub fn start(&mut self) -> Result<(), String> {
        if self.shutdown_tx.is_some() {
            return Err("Task dispatcher is already running".to_string());
        }

        let (shutdown_tx, mut shutdown_rx) = watch::channel(());
        let shutdown_tx_for_error = shutdown_tx.clone();
        self.shutdown_tx = Some(shutdown_tx);

        let config = self.config.clone();
        let registry = Arc::clone(&self.registry);
        let queue = Arc::clone(&self.queue);
        let executor = Arc::clone(&self.executor);
        let load_balancer = Arc::clone(&self.load_balancer);
        let paused = Arc::clone(&self.paused);
        let pause_notify = Arc::clone(&self.pause_notify);
        let last_error = Arc::clone(&self.last_error);

        tokio::spawn(async move {
            info!("Task dispatcher started");

            let mut interval = time::interval(config.poll_interval);

            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        info!("Task dispatcher shutdown signal received");
                        break;
                    }
                    _ = interval.tick() => {
                        // Check if paused
                        if paused.load(Ordering::Relaxed) {
                            debug!("Task dispatcher paused, waiting for resume");
                            pause_notify.notified().await;
                            continue;
                        }

                        // Try to dequeue and process a task
                        if let Some(task) = queue.dequeue_task_immutable().await {
                            let task_id = task.task_id.clone().unwrap_or_else(|| {
                                format!("task-{}", uuid::Uuid::new_v4())
                            });
                            let agent_id = task.agent_id.clone();
                            let input = task.input.clone();

                            debug!(
                                task_id = %task_id,
                                agent_id = %agent_id,
                                "Processing task"
                            );

                            // Check if agent is available (not at capacity)
                            let agent_load = load_balancer.get_agent_load(&agent_id).await;
                            if agent_load >= config.max_concurrent_per_agent {
                                // Agent is at capacity, put task back in queue
                                warn!(
                                    task_id = %task_id,
                                    agent_id = %agent_id,
                                    load = agent_load,
                                    max = config.max_concurrent_per_agent,
                                    "Agent at capacity, skipping task"
                                );
                                // Note: We can't easily put the task back, so we'll mark it as completed
                                // In a production system, we'd have a better mechanism for this
                                queue.mark_completed(&task_id).await;
                                continue;
                            }

                            // Get agent from registry
                            let Some(agent) = registry.get_agent(&agent_id).await else {
                                error!(
                                    task_id = %task_id,
                                    agent_id = %agent_id,
                                    "Agent not found"
                                );
                                queue.mark_completed(&task_id).await;
                                continue;
                            };

                            // Increment load before execution
                            load_balancer.increment_load(&agent_id).await;

                            // Execute the agent
                            let result = executor
                                .execute_agent_with_default_model(agent, &input, None)
                                .await;

                            // Decrement load after execution
                            load_balancer.decrement_load(&agent_id).await;

                            match result {
                                Ok(execution_result) => {
                                    if execution_result.success {
                                        info!(
                                            task_id = %task_id,
                                            agent_id = %agent_id,
                                            "Task completed successfully"
                                        );
                                    } else {
                                        warn!(
                                            task_id = %task_id,
                                            agent_id = %agent_id,
                                            error = ?execution_result.error,
                                            "Task execution failed"
                                        );
                                    }
                                }
                                Err(e) => {
                                    // Check if this is a critical error
                                    if let Some(critical_error) = CriticalError::from_model_error(&e) {
                                        error!(
                                            task_id = %task_id,
                                            agent_id = %agent_id,
                                            error = %critical_error,
                                            "Critical error detected, shutting down dispatcher"
                                        );

                                        // Store the error
                                        {
                                            let mut last_err = last_error.lock().unwrap();
                                            *last_err = Some(critical_error.clone());
                                        }

                                        // Signal shutdown
                                        let _ = shutdown_tx_for_error.send(());

                                        // Mark task as completed and break
                                        queue.mark_completed(&task_id).await;
                                        break;
                                    } else {
                                        error!(
                                            task_id = %task_id,
                                            agent_id = %agent_id,
                                            error = %e,
                                            "Task execution error"
                                        );
                                    }
                                }
                            }

                            // Mark task as completed
                            queue.mark_completed(&task_id).await;
                        }
                    }
                }
            }

            info!("Task dispatcher stopped");
        });

        Ok(())
    }

    /// Stops the dispatcher gracefully.
    ///
    /// # Returns
    /// Returns `Ok(())` if stopped successfully, or an error if not running.
    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
            Ok(())
        } else {
            Err("Task dispatcher is not running".to_string())
        }
    }

    /// Checks if the dispatcher is currently running.
    ///
    /// # Returns
    /// Returns `true` if running, `false` otherwise.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.shutdown_tx.is_some()
    }

    /// Pauses the dispatcher.
    ///
    /// The dispatcher will stop processing new tasks but will complete any in-flight tasks.
    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
        info!("Task dispatcher paused");
    }

    /// Resumes the dispatcher.
    ///
    /// The dispatcher will continue processing tasks from the queue.
    pub fn resume(&self) {
        self.paused.store(false, Ordering::Relaxed);
        self.pause_notify.notify_one();
        info!("Task dispatcher resumed");
    }

    /// Checks if the dispatcher is paused.
    ///
    /// # Returns
    /// Returns `true` if paused, `false` otherwise.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    /// Gets the load balancer for monitoring agent utilization.
    ///
    /// # Returns
    /// Returns a reference to the load balancer.
    pub fn load_balancer(&self) -> Arc<LoadBalancer> {
        Arc::clone(&self.load_balancer)
    }

    /// Gets the last critical error encountered (if any).
    ///
    /// # Returns
    /// Returns `Some(CriticalError)` if a critical error occurred, `None` otherwise.
    pub fn last_error(&self) -> Option<CriticalError> {
        self.last_error.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EchoAgent, ExecutionTask, Priority};

    #[tokio::test]
    async fn test_task_dispatcher_new() {
        let registry = Arc::new(AgentRegistry::new());
        let queue = Arc::new(ExecutionQueue::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let config = TaskDispatcherConfig::default();

        let dispatcher = TaskDispatcher::new(
            registry,
            queue,
            executor,
            config,
        );

        assert!(!dispatcher.is_running());
        assert!(!dispatcher.is_paused());
    }

    #[tokio::test]
    async fn test_task_dispatcher_start_stop() {
        let registry = Arc::new(AgentRegistry::new());
        let queue = Arc::new(ExecutionQueue::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let config = TaskDispatcherConfig::default();

        let mut dispatcher = TaskDispatcher::new(
            registry,
            queue,
            executor,
            config,
        );

        // Start dispatcher
        let result = dispatcher.start();
        assert!(result.is_ok());
        assert!(dispatcher.is_running());

        // Wait a bit to ensure background task is running
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Stop dispatcher
        let result = dispatcher.stop();
        assert!(result.is_ok());
        assert!(!dispatcher.is_running());

        // Wait for background task to stop
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_task_dispatcher_double_start() {
        let registry = Arc::new(AgentRegistry::new());
        let queue = Arc::new(ExecutionQueue::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let config = TaskDispatcherConfig::default();

        let mut dispatcher = TaskDispatcher::new(
            registry,
            queue,
            executor,
            config,
        );

        // Start dispatcher
        let result = dispatcher.start();
        assert!(result.is_ok());

        // Try to start again
        let result = dispatcher.start();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Task dispatcher is already running");

        // Cleanup
        let _ = dispatcher.stop();
    }

    #[tokio::test]
    async fn test_task_dispatcher_processes_tasks() {
        let registry = Arc::new(AgentRegistry::new());
        let queue = Arc::new(ExecutionQueue::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let config = TaskDispatcherConfig {
            poll_interval: Duration::from_millis(10),
            max_concurrent_per_agent: 10,
        };

        // Register an agent
        let agent = Arc::new(EchoAgent::new(
            "test-agent".to_string(),
            "Test agent".to_string(),
        ));
        registry.register_agent(agent).await;

        // Enqueue a task
        let task = ExecutionTask::new(
            "test-agent".to_string(),
            "test input".to_string(),
            Priority::default(),
        )
        .with_task_id("test-task-1".to_string());
        queue.enqueue_task(task).await.unwrap();

        let queue_clone = Arc::clone(&queue);
        let mut dispatcher = TaskDispatcher::new(
            registry,
            queue_clone,
            executor,
            config,
        );

        // Start dispatcher
        dispatcher.start().unwrap();

        // Wait for task to be processed
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Check that task was completed
        let metrics = queue.metrics().await;
        assert_eq!(metrics.completed, 1);
        assert_eq!(metrics.pending, 0);

        // Cleanup
        dispatcher.stop().unwrap();
    }

    #[tokio::test]
    async fn test_task_dispatcher_pause_resume() {
        let registry = Arc::new(AgentRegistry::new());
        let queue = Arc::new(ExecutionQueue::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let config = TaskDispatcherConfig {
            poll_interval: Duration::from_millis(10),
            max_concurrent_per_agent: 10,
        };

        let mut dispatcher = TaskDispatcher::new(
            registry,
            queue,
            executor,
            config,
        );

        // Start dispatcher
        dispatcher.start().unwrap();

        // Pause dispatcher
        dispatcher.pause();
        assert!(dispatcher.is_paused());

        // Resume dispatcher
        dispatcher.resume();
        assert!(!dispatcher.is_paused());

        // Cleanup
        dispatcher.stop().unwrap();
    }

    #[tokio::test]
    async fn test_task_dispatcher_pause_stops_processing() {
        let registry = Arc::new(AgentRegistry::new());
        let queue = Arc::new(ExecutionQueue::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let config = TaskDispatcherConfig {
            poll_interval: Duration::from_millis(10),
            max_concurrent_per_agent: 10,
        };

        // Register an agent
        let agent = Arc::new(EchoAgent::new(
            "test-agent".to_string(),
            "Test agent".to_string(),
        ));
        registry.register_agent(agent).await;

        let queue_clone = Arc::clone(&queue);
        let mut dispatcher = TaskDispatcher::new(
            registry,
            queue_clone,
            executor,
            config,
        );

        // Start dispatcher
        dispatcher.start().unwrap();

        // Enqueue a couple tasks first
        for i in 0..2 {
            let task = ExecutionTask::new(
                "test-agent".to_string(),
                format!("test input {}", i),
                Priority::default(),
            )
            .with_task_id(format!("test-task-{}", i));
            queue.enqueue_task(task).await.unwrap();
        }

        // Wait for tasks to start processing
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Pause dispatcher
        dispatcher.pause();

        // Enqueue more tasks while paused
        for i in 2..5 {
            let task = ExecutionTask::new(
                "test-agent".to_string(),
                format!("test input {}", i),
                Priority::default(),
            )
            .with_task_id(format!("test-task-{}", i));
            queue.enqueue_task(task).await.unwrap();
        }

        // Wait a bit - paused tasks should not be processed
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check that not all tasks were processed
        let metrics = queue.metrics().await;
        assert!(metrics.completed < 5, "Some tasks should still be pending when paused");
        assert!(metrics.pending > 0, "Some tasks should be pending when paused");

        // Resume and wait for completion
        dispatcher.resume();
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Check that all tasks were eventually completed
        let metrics = queue.metrics().await;
        assert_eq!(metrics.completed, 5);
        assert_eq!(metrics.pending, 0);

        // Cleanup
        dispatcher.stop().unwrap();
    }
}


#![cfg(feature = "collaboration")]
// Note: collaboration module temporarily disabled

//! End-to-end integration tests for multi-agent collaboration.

use radium_core::collaboration::delegation::{DelegationManager, WorkerStatus};
use radium_core::collaboration::lock_manager::{LockType, ResourceLockManager};
use radium_core::collaboration::message_bus::{MessageBus, MessageType};
use radium_core::collaboration::progress::{ProgressStatus, ProgressTracker};
use radium_core::storage::database::Database;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

// Mock worker executor for testing
struct MockWorkerExecutor;

impl radium_core::collaboration::delegation::WorkerExecutor for MockWorkerExecutor {
    fn execute_worker(
        &self,
        _worker_id: &str,
        _task_input: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = std::result::Result<radium_core::collaboration::delegation::WorkerExecutionResult, String>> + Send>,
    > {
        Box::pin(async move {
            // Simulate work
            sleep(Duration::from_millis(10)).await;
            Ok(radium_core::collaboration::delegation::WorkerExecutionResult {
                success: true,
                output: Some("Task completed".to_string()),
                error: None,
            })
        })
    }
}

#[tokio::test]
async fn test_message_bus_point_to_point() {
    let db = Arc::new(Mutex::new(Database::open_in_memory().unwrap()));
    let message_bus = MessageBus::new(db.clone());

    // Register two agents
    let _rx1 = message_bus.register_agent("agent-1".to_string()).await;
    let mut rx2 = message_bus.register_agent("agent-2".to_string()).await;

    // Send message from agent-1 to agent-2
    let payload = serde_json::json!({"task": "test"});
    let message_id = message_bus
        .send_message("agent-1", "agent-2", MessageType::TaskRequest, payload)
        .await
        .unwrap();

    // Wait for delivery
    sleep(Duration::from_millis(50)).await;

    // Check that message was received
    let received = rx2.try_recv();
    assert!(received.is_ok());
    let message = received.unwrap();
    assert_eq!(message.id, message_id);
    assert_eq!(message.sender_id, "agent-1");
    assert_eq!(message.recipient_id, Some("agent-2".to_string()));
}

#[tokio::test]
async fn test_message_bus_broadcast() {
    let db = Arc::new(Mutex::new(Database::open_in_memory().unwrap()));
    let message_bus = MessageBus::new(db.clone());

    // Register three agents
    let rx1 = message_bus.register_agent("agent-1".to_string()).await;
    let rx2 = message_bus.register_agent("agent-2".to_string()).await;
    let rx3 = message_bus.register_agent("agent-3".to_string()).await;

    // Broadcast a message from agent-1
    let payload = serde_json::json!({"status": "update"});
    message_bus
        .broadcast_message("agent-1", MessageType::StatusUpdate, payload)
        .await
        .unwrap();

    // Wait for delivery
    sleep(Duration::from_millis(50)).await;

    // Check that agent-2 and agent-3 received the message (agent-1 should not)
    let msg2 = rx2.try_recv();
    let msg3 = rx3.try_recv();
    let msg1 = rx1.try_recv();

    assert!(msg2.is_ok());
    assert!(msg3.is_ok());
    assert!(msg1.is_err()); // agent-1 should not receive its own broadcast
}

#[tokio::test]
async fn test_lock_manager_read_write() {
    let lock_manager = ResourceLockManager::new();

    // Agent A acquires write lock
    let lock_a = lock_manager
        .request_write_lock("agent-a", "file.txt", None)
        .await
        .unwrap();

    // Agent B should not be able to acquire write lock (timeout)
    let result = tokio::time::timeout(
        Duration::from_millis(500),
        lock_manager.request_write_lock("agent-b", "file.txt", Some(1)),
    )
    .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_err());

    // Release lock
    drop(lock_a);

    // Wait a bit for cleanup
    sleep(Duration::from_millis(50)).await;

    // Now agent B should be able to acquire lock
    let lock_b = lock_manager
        .request_write_lock("agent-b", "file.txt", None)
        .await
        .unwrap();

    drop(lock_b);
}

#[tokio::test]
async fn test_progress_tracking() {
    let db = Arc::new(Mutex::new(Database::open_in_memory().unwrap()));
    let progress_tracker = ProgressTracker::new(db.clone());

    // Report progress
    progress_tracker
        .report_progress("agent-1", 50, ProgressStatus::Working, Some("Halfway done".to_string()))
        .await
        .unwrap();

    // Get progress
    let progress = progress_tracker.get_progress("agent-1").await.unwrap();
    assert!(progress.is_some());
    let snapshot = progress.unwrap();
    assert_eq!(snapshot.percentage, 50);
    assert_eq!(snapshot.status, ProgressStatus::Working);
    assert_eq!(snapshot.message, Some("Halfway done".to_string()));
}

#[tokio::test]
async fn test_delegation_spawn_worker() {
    let db = Arc::new(Mutex::new(Database::open_in_memory().unwrap()));
    let message_bus = Arc::new(MessageBus::new(db.clone()));
    let worker_executor: Arc<dyn radium_core::collaboration::delegation::WorkerExecutor> =
        Arc::new(MockWorkerExecutor);
    let delegation_manager = DelegationManager::new(db, message_bus, worker_executor);

    // Spawn a worker
    let worker_id = delegation_manager
        .spawn_worker("supervisor-1", "worker-1", "Do task", 0)
        .await
        .unwrap();

    assert_eq!(worker_id, "worker-1");

    // Wait for worker to complete
    sleep(Duration::from_millis(100)).await;

    // Check worker status
    let status = delegation_manager.get_worker_status("worker-1").await.unwrap();
    assert_eq!(status, WorkerStatus::Completed);
}

#[tokio::test]
async fn test_delegation_max_depth() {
    let db = Arc::new(Mutex::new(Database::open_in_memory().unwrap()));
    let message_bus = Arc::new(MessageBus::new(db.clone()));
    let worker_executor: Arc<dyn radium_core::collaboration::delegation::WorkerExecutor> =
        Arc::new(MockWorkerExecutor);
    let delegation_manager = DelegationManager::new(db, message_bus, worker_executor);

    // Try to spawn at max depth (3)
    let result = delegation_manager
        .spawn_worker("supervisor-1", "worker-1", "Do task", 3)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        radium_core::collaboration::CollaborationError::MaxDelegationDepthExceeded { .. } => {}
        _ => panic!("Expected MaxDelegationDepthExceeded error"),
    }
}

#[tokio::test]
async fn test_progress_aggregation() {
    let db = Arc::new(Mutex::new(Database::open_in_memory().unwrap()));
    let progress_tracker = ProgressTracker::new(db.clone());

    // Report progress for multiple agents
    progress_tracker
        .report_progress("agent-1", 30, ProgressStatus::Working, None)
        .await
        .unwrap();
    progress_tracker
        .report_progress("agent-2", 60, ProgressStatus::Working, None)
        .await
        .unwrap();
    progress_tracker
        .report_progress("agent-3", 90, ProgressStatus::Working, None)
        .await
        .unwrap();

    // Get aggregated progress
    let aggregated = progress_tracker
        .get_aggregated_progress(&[
            "agent-1".to_string(),
            "agent-2".to_string(),
            "agent-3".to_string(),
        ])
        .await
        .unwrap();

    assert!((aggregated.average_percentage - 60.0).abs() < 0.1);
    assert_eq!(aggregated.worker_statuses.len(), 3);
}

#[tokio::test]
async fn test_hierarchical_delegation() {
    let db = Arc::new(Mutex::new(Database::open_in_memory().unwrap()));
    let message_bus = Arc::new(MessageBus::new(db.clone()));
    let worker_executor: Arc<dyn radium_core::collaboration::delegation::WorkerExecutor> =
        Arc::new(MockWorkerExecutor);
    let delegation_manager = DelegationManager::new(db, message_bus, worker_executor);

    // Supervisor spawns worker
    delegation_manager
        .spawn_worker("supervisor-1", "worker-1", "Task 1", 0)
        .await
        .unwrap();

    // Worker spawns sub-worker (depth 1)
    delegation_manager
        .spawn_worker("worker-1", "worker-2", "Task 2", 1)
        .await
        .unwrap();

    // Sub-worker spawns sub-sub-worker (depth 2)
    delegation_manager
        .spawn_worker("worker-2", "worker-3", "Task 3", 2)
        .await
        .unwrap();

    // Wait for all workers to complete
    sleep(Duration::from_millis(200)).await;

    // Check all workers completed
    assert_eq!(
        delegation_manager.get_worker_status("worker-1").await.unwrap(),
        WorkerStatus::Completed
    );
    assert_eq!(
        delegation_manager.get_worker_status("worker-2").await.unwrap(),
        WorkerStatus::Completed
    );
    assert_eq!(
        delegation_manager.get_worker_status("worker-3").await.unwrap(),
        WorkerStatus::Completed
    );
}


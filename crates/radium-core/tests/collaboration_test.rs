//! Tests for agent collaboration features.

use radium_core::collaboration::message_bus::{AgentMessage, MessageBus, MessageType};
use radium_core::storage::database::Database;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_message_bus_send_message() {
    let db = Arc::new(Mutex::new(Database::open_in_memory().unwrap()));
    let message_bus = MessageBus::new(db.clone());

    // Register two agents
    let _rx1 = message_bus.register_agent("agent-1".to_string()).await;
    let rx2 = message_bus.register_agent("agent-2".to_string()).await;

    // Send a message from agent-1 to agent-2
    let payload = serde_json::json!({"task": "test"});
    let message_id = message_bus
        .send_message("agent-1", "agent-2", MessageType::TaskRequest, payload)
        .await
        .unwrap();

    // Wait a bit for delivery
    sleep(Duration::from_millis(10)).await;

    // Check that message was received
    let received = rx2.try_recv();
    assert!(received.is_ok());
    let message = received.unwrap();
    assert_eq!(message.id, message_id);
    assert_eq!(message.sender_id, "agent-1");
    assert_eq!(message.recipient_id, Some("agent-2".to_string()));
    assert_eq!(message.message_type, MessageType::TaskRequest);
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

    // Wait a bit for delivery
    sleep(Duration::from_millis(10)).await;

    // Check that agent-2 and agent-3 received the message (agent-1 should not)
    let msg2 = rx2.try_recv();
    let msg3 = rx3.try_recv();
    let msg1 = rx1.try_recv();

    assert!(msg2.is_ok());
    assert!(msg3.is_ok());
    assert!(msg1.is_err()); // agent-1 should not receive its own broadcast
}

#[tokio::test]
async fn test_message_bus_get_messages() {
    let db = Arc::new(Mutex::new(Database::open_in_memory().unwrap()));
    let message_bus = MessageBus::new(db.clone());

    // Register an agent
    message_bus.register_agent("agent-1".to_string()).await;

    // Send a message to agent-1 (before it's registered, so it will be stored)
    let payload = serde_json::json!({"task": "test"});
    message_bus
        .send_message("agent-2", "agent-1", MessageType::TaskRequest, payload)
        .await
        .unwrap();

    // Get messages for agent-1
    let messages = message_bus.get_messages("agent-1", false).await.unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].sender_id, "agent-2");
}

#[tokio::test]
async fn test_message_bus_nonexistent_recipient() {
    let db = Arc::new(Mutex::new(Database::open_in_memory().unwrap()));
    let message_bus = MessageBus::new(db.clone());

    // Try to send message to non-existent agent
    let payload = serde_json::json!({"task": "test"});
    let result = message_bus
        .send_message("agent-1", "nonexistent", MessageType::TaskRequest, payload)
        .await;

    // Should succeed (message stored for later delivery)
    assert!(result.is_ok());

    // Message should be stored but not delivered
    let messages = message_bus.get_messages("nonexistent", true).await.unwrap();
    assert_eq!(messages.len(), 1);
    assert!(!messages[0].delivered);
}


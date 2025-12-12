//! Tests for session resume functionality.

use radium_core::session::{SessionManager, SessionState};
use radium_core::session::state::{Message, ToolCall};
use tempfile::TempDir;
use uuid::Uuid;

fn create_test_manager() -> (SessionManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path()).unwrap();
    (manager, temp_dir)
}

#[tokio::test]
async fn test_session_resume_after_clear_cache() {
    let (manager, _temp_dir) = create_test_manager();

    // Create session and add data
    let session = manager.create_session(None, None, None).await.unwrap();
    let session_id = session.id.clone();

    // Add messages
    for i in 0..3 {
        let message = Message {
            id: Uuid::new_v4().to_string(),
            content: format!("Message {}", i),
            role: "user".to_string(),
            timestamp: chrono::Utc::now(),
        };
        manager.append_message(&session_id, message).await.unwrap();
    }

    // Add tool calls
    for i in 0..2 {
        let tool_call = ToolCall {
            id: Uuid::new_v4().to_string(),
            tool_name: format!("tool_{}", i),
            arguments_json: format!(r#"{{"arg": {}}}"#, i),
            result_json: Some(format!(r#"{{"result": {}}}"#, i)),
            success: true,
            error: None,
            duration_ms: 100 * (i + 1) as u64,
            timestamp: chrono::Utc::now(),
        };
        manager.append_tool_call(&session_id, tool_call).await.unwrap();
    }

    // Clear in-memory cache by creating a new manager (simulating restart)
    let manager2 = SessionManager::new(_temp_dir.path()).unwrap();

    // Resume session - should load full history from disk
    let resumed = manager2.attach_session(&session_id).await.unwrap();

    assert_eq!(resumed.id, session_id);
    assert_eq!(resumed.messages.len(), 3);
    assert_eq!(resumed.tool_calls.len(), 2);

    // Verify message content
    assert_eq!(resumed.messages[0].content, "Message 0");
    assert_eq!(resumed.messages[1].content, "Message 1");
    assert_eq!(resumed.messages[2].content, "Message 2");

    // Verify tool call content
    assert_eq!(resumed.tool_calls[0].tool_name, "tool_0");
    assert_eq!(resumed.tool_calls[1].tool_name, "tool_1");
}

#[tokio::test]
async fn test_session_resume_preserves_state() {
    let (manager, _temp_dir) = create_test_manager();

    let session = manager.create_session(None, None, None).await.unwrap();
    let session_id = session.id.clone();

    // Update state
    manager
        .update_session_state(&session_id, SessionState::Paused)
        .await
        .unwrap();

    // Clear cache
    let manager2 = SessionManager::new(_temp_dir.path()).unwrap();

    // Resume
    let resumed = manager2.attach_session(&session_id).await.unwrap();
    assert_eq!(resumed.state, SessionState::Paused);
}

#[tokio::test]
async fn test_session_resume_with_artifacts() {
    let (manager, _temp_dir) = create_test_manager();

    let session = manager.create_session(None, None, None).await.unwrap();
    let session_id = session.id.clone();

    // Save artifacts
    manager
        .save_artifact(&session_id, "file1.txt", b"content1")
        .await
        .unwrap();
    manager
        .save_artifact(&session_id, "file2.txt", b"content2")
        .await
        .unwrap();

    // Clear cache
    let manager2 = SessionManager::new(_temp_dir.path()).unwrap();

    // Resume
    let resumed = manager2.attach_session(&session_id).await.unwrap();
    assert_eq!(resumed.artifacts.len(), 2);
    assert_eq!(resumed.artifacts[0].id, "file1.txt");
    assert_eq!(resumed.artifacts[1].id, "file2.txt");
}

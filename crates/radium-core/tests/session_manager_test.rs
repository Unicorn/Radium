//! Tests for session manager.

use radium_core::session::{SessionManager, SessionState};
use radium_core::session::state::{Approval, Message, ToolCall};
use std::path::PathBuf;
use tempfile::TempDir;
use uuid::Uuid;

fn create_test_manager() -> (SessionManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path()).unwrap();
    (manager, temp_dir)
}

#[tokio::test]
async fn test_create_session() {
    let (manager, _temp_dir) = create_test_manager();

    let session = manager
        .create_session(Some("agent-1".to_string()), Some("/workspace".to_string()), Some("Test Session".to_string()))
        .await
        .unwrap();

    assert!(!session.id.is_empty());
    assert_eq!(session.agent_id, Some("agent-1".to_string()));
    assert_eq!(session.workspace_root, Some("/workspace".to_string()));
    assert_eq!(session.name, Some("Test Session".to_string()));
    assert_eq!(session.state, SessionState::Active);
    assert!(session.messages.is_empty());
    assert!(session.tool_calls.is_empty());
}

#[tokio::test]
async fn test_get_session() {
    let (manager, _temp_dir) = create_test_manager();

    let created = manager
        .create_session(None, None, None)
        .await
        .unwrap();
    let session_id = created.id.clone();

    let retrieved = manager.get_session(&session_id).await.unwrap();
    assert_eq!(retrieved.id, session_id);
    assert_eq!(retrieved.state, SessionState::Active);
}

#[tokio::test]
async fn test_list_sessions() {
    let (manager, _temp_dir) = create_test_manager();

    // Create multiple sessions
    let _session1 = manager.create_session(None, None, None).await.unwrap();
    let _session2 = manager.create_session(None, None, None).await.unwrap();
    let _session3 = manager.create_session(None, None, None).await.unwrap();

    let (sessions, total, page, page_size) = manager.list_sessions(None, None, None, None).await.unwrap();
    assert_eq!(total, 3);
    assert_eq!(sessions.len(), 3);
    assert_eq!(page, 1);
    assert_eq!(page_size, 50);
}

#[tokio::test]
async fn test_list_sessions_pagination() {
    let (manager, _temp_dir) = create_test_manager();

    // Create 5 sessions
    for _ in 0..5 {
        manager.create_session(None, None, None).await.unwrap();
    }

    // First page
    let (sessions, total, page, page_size) = manager.list_sessions(Some(1), Some(2), None, None).await.unwrap();
    assert_eq!(total, 5);
    assert_eq!(sessions.len(), 2);
    assert_eq!(page, 1);
    assert_eq!(page_size, 2);

    // Second page
    let (sessions, _, _, _) = manager.list_sessions(Some(2), Some(2), None, None).await.unwrap();
    assert_eq!(sessions.len(), 2);
}

#[tokio::test]
async fn test_list_sessions_filter_state() {
    let (manager, _temp_dir) = create_test_manager();

    let session1 = manager.create_session(None, None, None).await.unwrap();
    let session2 = manager.create_session(None, None, None).await.unwrap();

    // Set one session to completed
    manager
        .update_session_state(&session1.id, SessionState::Completed)
        .await
        .unwrap();

    // Filter by active
    let (sessions, _, _, _) = manager
        .list_sessions(None, None, Some(SessionState::Active), None)
        .await
        .unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, session2.id);

    // Filter by completed
    let (sessions, _, _, _) = manager
        .list_sessions(None, None, Some(SessionState::Completed), None)
        .await
        .unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, session1.id);
}

#[tokio::test]
async fn test_attach_session() {
    let (manager, _temp_dir) = create_test_manager();

    let created = manager.create_session(None, None, None).await.unwrap();
    let session_id = created.id.clone();

    let attached = manager.attach_session(&session_id).await.unwrap();
    assert_eq!(attached.id, session_id);
    // last_active should be updated
    assert!(attached.last_active >= created.last_active);
}

#[tokio::test]
async fn test_append_message() {
    let (manager, _temp_dir) = create_test_manager();

    let session = manager.create_session(None, None, None).await.unwrap();
    let session_id = session.id.clone();

    let message = Message {
        id: Uuid::new_v4().to_string(),
        content: "Hello, world!".to_string(),
        role: "user".to_string(),
        timestamp: chrono::Utc::now(),
    };

    manager.append_message(&session_id, message.clone()).await.unwrap();

    let retrieved = manager.get_session(&session_id).await.unwrap();
    assert_eq!(retrieved.messages.len(), 1);
    assert_eq!(retrieved.messages[0].content, "Hello, world!");
}

#[tokio::test]
async fn test_append_tool_call() {
    let (manager, _temp_dir) = create_test_manager();

    let session = manager.create_session(None, None, None).await.unwrap();
    let session_id = session.id.clone();

    let tool_call = ToolCall {
        id: Uuid::new_v4().to_string(),
        tool_name: "read_file".to_string(),
        arguments_json: r#"{"path": "test.txt"}"#.to_string(),
        result_json: Some(r#"{"content": "test"}"#.to_string()),
        success: true,
        error: None,
        duration_ms: 100,
        timestamp: chrono::Utc::now(),
    };

    manager.append_tool_call(&session_id, tool_call.clone()).await.unwrap();

    let retrieved = manager.get_session(&session_id).await.unwrap();
    assert_eq!(retrieved.tool_calls.len(), 1);
    assert_eq!(retrieved.tool_calls[0].tool_name, "read_file");
}

#[tokio::test]
async fn test_append_approval() {
    let (manager, _temp_dir) = create_test_manager();

    let session = manager.create_session(None, None, None).await.unwrap();
    let session_id = session.id.clone();

    let approval = Approval {
        id: Uuid::new_v4().to_string(),
        tool_name: "write_file".to_string(),
        arguments_json: r#"{"path": "test.txt", "content": "test"}"#.to_string(),
        policy_rule: "require_approval_for_file_writes".to_string(),
        approved: true,
        reason: Some("User approved".to_string()),
        timestamp: chrono::Utc::now(),
    };

    manager.append_approval(&session_id, approval.clone()).await.unwrap();

    let retrieved = manager.get_session(&session_id).await.unwrap();
    assert_eq!(retrieved.approvals.len(), 1);
    assert_eq!(retrieved.approvals[0].tool_name, "write_file");
    assert!(retrieved.approvals[0].approved);
}

#[tokio::test]
async fn test_save_artifact() {
    let (manager, _temp_dir) = create_test_manager();

    let session = manager.create_session(None, None, None).await.unwrap();
    let session_id = session.id.clone();

    let content = b"artifact content";
    let artifact_path = manager
        .save_artifact(&session_id, "test.txt", content)
        .await
        .unwrap();

    assert!(artifact_path.exists());
    let retrieved = manager.get_session(&session_id).await.unwrap();
    assert_eq!(retrieved.artifacts.len(), 1);
    assert_eq!(retrieved.artifacts[0].id, "test.txt");
}

#[tokio::test]
async fn test_update_session_state() {
    let (manager, _temp_dir) = create_test_manager();

    let session = manager.create_session(None, None, None).await.unwrap();
    let session_id = session.id.clone();

    manager
        .update_session_state(&session_id, SessionState::Completed)
        .await
        .unwrap();

    let retrieved = manager.get_session(&session_id).await.unwrap();
    assert_eq!(retrieved.state, SessionState::Completed);
}

#[tokio::test]
async fn test_delete_session() {
    let (manager, _temp_dir) = create_test_manager();

    let session = manager.create_session(None, None, None).await.unwrap();
    let session_id = session.id.clone();

    manager.delete_session(&session_id).await.unwrap();

    // Session should not be found
    let result = manager.get_session(&session_id).await;
    assert!(result.is_err());
}

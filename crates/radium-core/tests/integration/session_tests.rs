//! Integration tests for session management.

use radium_core::session::SessionManager;
use radium_core::session::state::{Message, ToolCall, SessionState};
use radium_core::server::RadiumService;
use radium_core::storage::Database;
use tempfile::TempDir;
use uuid::Uuid;

/// Create a test service with session manager.
fn create_test_service() -> (RadiumService, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db = Database::open_in_memory().unwrap();
    let service = RadiumService::new(db);
    (service, temp_dir)
}

#[tokio::test]
async fn test_session_resume_after_disconnect() {
    let (service, _temp_dir) = create_test_service();

    // Create session
    let create_request = tonic::Request::new(radium_core::proto::CreateSessionRequest {
        agent_id: None,
        workspace_root: None,
        session_name: None,
    });
    let create_response = service.create_session(create_request).await.unwrap();
    let session_id = create_response.into_inner().session_id;

    // Add messages
    for i in 0..3 {
        let message_request = tonic::Request::new(radium_core::proto::SendSessionMessageRequest {
            session_id: session_id.clone(),
            message: format!("Message {}", i),
            role: Some("user".to_string()),
        });
        service.send_session_message(message_request).await.unwrap();
    }

    // Simulate disconnect by creating new service (simulating restart)
    let (service2, _temp_dir2) = create_test_service();

    // Attach to session - should load full history
    let attach_request = tonic::Request::new(radium_core::proto::AttachSessionRequest {
        session_id: session_id.clone(),
    });
    let attach_response = service2.attach_session(attach_request).await.unwrap();
    let inner = attach_response.into_inner();

    assert_eq!(inner.messages.len(), 3);
    assert_eq!(inner.messages[0].content, "Message 0");
    assert_eq!(inner.messages[1].content, "Message 1");
    assert_eq!(inner.messages[2].content, "Message 2");
}

#[tokio::test]
async fn test_concurrent_sessions() {
    let (service, _temp_dir) = create_test_service();

    // Create multiple sessions
    let mut session_ids = Vec::new();
    for i in 0..5 {
        let create_request = tonic::Request::new(radium_core::proto::CreateSessionRequest {
            agent_id: Some(format!("agent-{}", i)),
            workspace_root: None,
            session_name: Some(format!("Session {}", i)),
        });
        let create_response = service.create_session(create_request).await.unwrap();
        session_ids.push(create_response.into_inner().session_id);
    }

    // Verify all sessions are independent
    let list_request = tonic::Request::new(radium_core::proto::ListSessionsRequest {
        page: Some(1),
        page_size: Some(10),
        filter_state: None,
        filter_agent_id: None,
    });
    let list_response = service.list_sessions(list_request).await.unwrap();
    let inner = list_response.into_inner();

    assert_eq!(inner.total, 5);
    assert_eq!(inner.sessions.len(), 5);

    // Verify each session can be accessed independently
    for session_id in &session_ids {
        let attach_request = tonic::Request::new(radium_core::proto::AttachSessionRequest {
            session_id: session_id.clone(),
        });
        let attach_response = service.attach_session(attach_request).await.unwrap();
        assert!(attach_response.into_inner().session.is_some());
    }
}

#[tokio::test]
async fn test_event_ordering() {
    // This test would verify event ordering in SessionEventsStream
    // For now, it's a placeholder as full event streaming integration
    // requires more complex setup with agent execution
    // TODO: Implement when agent execution is integrated with session events
}

#[tokio::test]
async fn test_approval_handshake() {
    // This test would verify approval request/response handshake
    // For now, it's a placeholder as approval integration requires
    // policy engine and tool execution integration
    // TODO: Implement when policy engine is integrated with session events
}

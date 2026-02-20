//! Tests for session management gRPC API.

use radium_core::session::SessionManager;
use radium_core::server::RadiumService;
use radium_core::storage::Database;
use tempfile::TempDir;
use tonic::Request;

fn create_test_service() -> (RadiumService, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db = Database::open_in_memory().unwrap();
    let service = RadiumService::new(db);
    (service, temp_dir)
}

#[tokio::test]
async fn test_create_session_rpc() {
    let (service, _temp_dir) = create_test_service();

    let request = Request::new(radium_core::proto::CreateSessionRequest {
        agent_id: Some("agent-1".to_string()),
        workspace_root: Some("/workspace".to_string()),
        session_name: Some("Test Session".to_string()),
    });

    let response = service.create_session(request).await.unwrap();
    let inner = response.into_inner();

    assert!(!inner.session_id.is_empty());
    assert!(inner.session.is_some());
    assert_eq!(inner.error, None);
}

#[tokio::test]
async fn test_list_sessions_rpc() {
    let (service, _temp_dir) = create_test_service();

    // Create a session first
    let create_request = Request::new(radium_core::proto::CreateSessionRequest {
        agent_id: None,
        workspace_root: None,
        session_name: None,
    });
    let _ = service.create_session(create_request).await.unwrap();

    // List sessions
    let list_request = Request::new(radium_core::proto::ListSessionsRequest {
        page: Some(1),
        page_size: Some(10),
        filter_state: None,
        filter_agent_id: None,
    });

    let response = service.list_sessions(list_request).await.unwrap();
    let inner = response.into_inner();

    assert!(inner.total >= 1);
    assert!(!inner.sessions.is_empty());
    assert_eq!(inner.error, None);
}

#[tokio::test]
async fn test_attach_session_rpc() {
    let (service, _temp_dir) = create_test_service();

    // Create a session
    let create_request = Request::new(radium_core::proto::CreateSessionRequest {
        agent_id: None,
        workspace_root: None,
        session_name: None,
    });
    let create_response = service.create_session(create_request).await.unwrap();
    let session_id = create_response.into_inner().session_id;

    // Attach to session
    let attach_request = Request::new(radium_core::proto::AttachSessionRequest {
        session_id: session_id.clone(),
    });

    let response = service.attach_session(attach_request).await.unwrap();
    let inner = response.into_inner();

    assert!(inner.session.is_some());
    assert_eq!(inner.session.as_ref().unwrap().id, session_id);
    assert_eq!(inner.error, None);
}

#[tokio::test]
async fn test_send_session_message_rpc() {
    let (service, _temp_dir) = create_test_service();

    // Create a session
    let create_request = Request::new(radium_core::proto::CreateSessionRequest {
        agent_id: None,
        workspace_root: None,
        session_name: None,
    });
    let create_response = service.create_session(create_request).await.unwrap();
    let session_id = create_response.into_inner().session_id;

    // Send a message
    let message_request = Request::new(radium_core::proto::SendSessionMessageRequest {
        session_id: session_id.clone(),
        message: "Hello, world!".to_string(),
        role: Some("user".to_string()),
    });

    let response = service.send_session_message(message_request).await.unwrap();
    let inner = response.into_inner();

    assert!(inner.success);
    assert_eq!(inner.error, None);
}

//! Integration tests for session management.

use radium_tui::session_manager::{ChatSession, SessionManager};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_session_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
    assert!(manager.sessions_dir.exists());
}

#[test]
fn test_session_creation() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
    
    let session = ChatSession {
        session_id: "test-session-1".to_string(),
        agent_id: "test-agent".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        message_count: 0,
        last_message: None,
    };
    
    manager.save_session(&session).unwrap();
    
    // Verify session file exists
    let session_file = manager.sessions_dir.join("test-session-1.json");
    assert!(session_file.exists());
}

#[test]
fn test_session_loading() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
    
    // Create a session
    let session = ChatSession {
        session_id: "test-session-1".to_string(),
        agent_id: "test-agent".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        message_count: 5,
        last_message: Some("Test message".to_string()),
    };
    
    manager.save_session(&session).unwrap();
    
    // Load sessions
    let sessions = manager.load_sessions().unwrap();
    assert!(!sessions.is_empty());
    
    // Find our session
    let mut found = false;
    for sessions_list in sessions.values() {
        for s in sessions_list {
            if s.session_id == "test-session-1" {
                assert_eq!(s.agent_id, "test-agent");
                assert_eq!(s.message_count, 5);
                found = true;
            }
        }
    }
    assert!(found);
}

#[test]
fn test_session_update() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
    
    // Update a non-existent session (creates it)
    manager.update_session("new-session", "agent-1", "Hello").unwrap();
    
    // Load and verify
    let sessions = manager.load_sessions().unwrap();
    let mut found = false;
    for sessions_list in sessions.values() {
        for s in sessions_list {
            if s.session_id == "new-session" {
                assert_eq!(s.agent_id, "agent-1");
                assert_eq!(s.message_count, 1);
                assert_eq!(s.last_message, Some("Hello".to_string()));
                found = true;
            }
        }
    }
    assert!(found);
    
    // Update again
    manager.update_session("new-session", "agent-1", "World").unwrap();
    
    // Verify message count increased
    let sessions = manager.load_sessions().unwrap();
    for sessions_list in sessions.values() {
        for s in sessions_list {
            if s.session_id == "new-session" {
                assert_eq!(s.message_count, 2);
                assert_eq!(s.last_message, Some("World".to_string()));
            }
        }
    }
}

#[test]
fn test_session_deletion() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
    
    // Create a session
    let session = ChatSession {
        session_id: "delete-me".to_string(),
        agent_id: "test-agent".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        message_count: 0,
        last_message: None,
    };
    
    manager.save_session(&session).unwrap();
    
    // Verify it exists
    let sessions = manager.load_sessions().unwrap();
    let mut count_before = 0;
    for sessions_list in sessions.values() {
        count_before += sessions_list.len();
    }
    assert!(count_before > 0);
    
    // Delete it
    manager.delete_session("delete-me").unwrap();
    
    // Verify it's gone
    let sessions = manager.load_sessions().unwrap();
    let mut count_after = 0;
    for sessions_list in sessions.values() {
        count_after += sessions_list.len();
    }
    assert_eq!(count_after, count_before - 1);
}

#[test]
fn test_session_grouping_by_date() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
    
    // Create sessions with different dates
    let today = chrono::Utc::now();
    let yesterday = today - chrono::Duration::days(1);
    
    let session1 = ChatSession {
        session_id: "session-today".to_string(),
        agent_id: "agent-1".to_string(),
        created_at: today,
        updated_at: today,
        message_count: 0,
        last_message: None,
    };
    
    let session2 = ChatSession {
        session_id: "session-yesterday".to_string(),
        agent_id: "agent-1".to_string(),
        created_at: yesterday,
        updated_at: yesterday,
        message_count: 0,
        last_message: None,
    };
    
    manager.save_session(&session1).unwrap();
    manager.save_session(&session2).unwrap();
    
    // Load and verify grouping
    let sessions = manager.load_sessions().unwrap();
    assert!(sessions.len() >= 1); // At least one date group
    
    // Verify sessions are grouped by date
    for (date_key, sessions_list) in &sessions {
        assert!(!sessions_list.is_empty());
        // All sessions in a group should have the same date
        for session in sessions_list {
            let session_date = session.created_at.format("%Y-%m-%d").to_string();
            assert_eq!(*date_key, session_date);
        }
    }
}


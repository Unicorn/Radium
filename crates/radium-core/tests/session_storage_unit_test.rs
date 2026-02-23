//! Unit tests for `radium_core::session::SessionStorage`.
//!
//! Covers save_session_metadata, append_message, load_session, and list_session_ids
//! using real filesystem operations on a temp directory.

use chrono::Utc;
use radium_core::session::{Session, SessionStorage};
use radium_core::session::state::Message;
use tempfile::TempDir;
use uuid::Uuid;

fn tmp_storage() -> (SessionStorage, TempDir) {
    let dir = TempDir::new().unwrap();
    let storage = SessionStorage::new(dir.path()).unwrap();
    (storage, dir)
}

fn make_message(role: &str, content: &str) -> Message {
    Message {
        id: Uuid::new_v4().to_string(),
        role: role.to_string(),
        content: content.to_string(),
        timestamp: Utc::now(),
    }
}

// ── new ────────────────────────────────────────────────────────────────────

#[test]
fn test_storage_new_creates_sessions_dir() {
    let dir = TempDir::new().unwrap();
    let _storage = SessionStorage::new(dir.path()).unwrap();
    let sessions_dir = dir.path().join(".radium/_internals/sessions");
    assert!(sessions_dir.exists(), "sessions directory should be created");
}

// ── save_session_metadata ──────────────────────────────────────────────────

#[test]
fn test_save_session_metadata_creates_session_json() {
    let (storage, _dir) = tmp_storage();

    let session = Session::new("test-session".to_string(), Some("agent-1".to_string()), None);
    storage.save_session_metadata(&session).unwrap();

    let session_json = _dir
        .path()
        .join(".radium/_internals/sessions/test-session/session.json");
    assert!(session_json.exists(), "session.json should be written");

    let content = std::fs::read_to_string(&session_json).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["id"], "test-session");
    assert_eq!(parsed["agent_id"], "agent-1");
}

#[test]
fn test_save_session_metadata_overwrites_on_second_call() {
    let (storage, _dir) = tmp_storage();

    let mut session = Session::new("overwrite-test".to_string(), Some("agent-a".to_string()), None);
    storage.save_session_metadata(&session).unwrap();

    session.agent_id = Some("agent-b".to_string());
    storage.save_session_metadata(&session).unwrap();

    let path = _dir
        .path()
        .join(".radium/_internals/sessions/overwrite-test/session.json");
    let content = std::fs::read_to_string(&path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["agent_id"], "agent-b");
}

// ── append_message ─────────────────────────────────────────────────────────

#[test]
fn test_append_message_creates_messages_jsonl() {
    let (storage, _dir) = tmp_storage();

    let session = Session::new("msg-test".to_string(), None, None);
    storage.save_session_metadata(&session).unwrap();

    let msg = make_message("user", "hello world");
    storage.append_message("msg-test", &msg).unwrap();

    let jsonl_path = _dir
        .path()
        .join(".radium/_internals/sessions/msg-test/messages.jsonl");
    assert!(jsonl_path.exists(), "messages.jsonl should be created");

    let content = std::fs::read_to_string(&jsonl_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 1, "one line per message");

    let parsed: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(parsed["role"], "user");
    assert_eq!(parsed["content"], "hello world");
}

#[test]
fn test_append_message_accumulates_multiple_messages() {
    let (storage, _dir) = tmp_storage();

    let session = Session::new("multi-msg".to_string(), None, None);
    storage.save_session_metadata(&session).unwrap();

    storage.append_message("multi-msg", &make_message("user", "turn 1")).unwrap();
    storage.append_message("multi-msg", &make_message("assistant", "response 1")).unwrap();
    storage.append_message("multi-msg", &make_message("user", "turn 2")).unwrap();

    let jsonl_path = _dir
        .path()
        .join(".radium/_internals/sessions/multi-msg/messages.jsonl");
    let content = std::fs::read_to_string(&jsonl_path).unwrap();
    let line_count = content.lines().count();
    assert_eq!(line_count, 3, "should have one line per appended message");
}

// ── load_session ───────────────────────────────────────────────────────────

#[test]
fn test_load_session_roundtrip() {
    let (storage, _dir) = tmp_storage();

    let session = Session::new(
        "roundtrip".to_string(),
        Some("my-agent".to_string()),
        Some("/workspace".to_string()),
    );
    storage.save_session_metadata(&session).unwrap();
    storage.append_message("roundtrip", &make_message("user", "ping")).unwrap();
    storage.append_message("roundtrip", &make_message("assistant", "pong")).unwrap();

    let loaded = storage.load_session("roundtrip").unwrap();
    assert_eq!(loaded.id, "roundtrip");
    assert_eq!(loaded.agent_id.as_deref(), Some("my-agent"));
    assert_eq!(loaded.messages.len(), 2);
    assert_eq!(loaded.messages[0].role, "user");
    assert_eq!(loaded.messages[0].content, "ping");
    assert_eq!(loaded.messages[1].role, "assistant");
    assert_eq!(loaded.messages[1].content, "pong");
}

#[test]
fn test_load_session_not_found_returns_error() {
    let (storage, _dir) = tmp_storage();
    let result = storage.load_session("does-not-exist");
    assert!(result.is_err(), "loading missing session should return Err");
}

#[test]
fn test_load_session_no_messages_file_gives_empty_messages() {
    let (storage, _dir) = tmp_storage();

    let session = Session::new("no-msgs".to_string(), None, None);
    storage.save_session_metadata(&session).unwrap();
    // Deliberately do NOT append any messages

    let loaded = storage.load_session("no-msgs").unwrap();
    assert!(loaded.messages.is_empty(), "messages should be empty when no file exists");
}

// ── list_session_ids ───────────────────────────────────────────────────────

#[test]
fn test_list_session_ids_empty() {
    let (storage, _dir) = tmp_storage();
    let ids = storage.list_session_ids().unwrap();
    assert!(ids.is_empty());
}

#[test]
fn test_list_session_ids_returns_saved_sessions() {
    let (storage, _dir) = tmp_storage();

    storage.save_session_metadata(&Session::new("alpha".to_string(), None, None)).unwrap();
    storage.save_session_metadata(&Session::new("beta".to_string(), None, None)).unwrap();

    let mut ids = storage.list_session_ids().unwrap();
    ids.sort();
    assert_eq!(ids, vec!["alpha", "beta"]);
}

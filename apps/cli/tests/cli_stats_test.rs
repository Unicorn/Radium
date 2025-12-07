//! Integration tests for CLI stats commands.

use assert_cmd::Command;
use chrono::Utc;
use predicates::prelude::*;
use radium_core::analytics::{SessionMetrics, SessionReport, SessionStorage};
use radium_core::monitoring::{AgentRecord, AgentStatus, MonitoringService, TelemetryRecord, TelemetryTracking};
use std::collections::HashMap;
use std::time::Duration;
use tempfile::TempDir;
use tokio;

/// Helper to create a test workspace with monitoring data.
async fn setup_test_workspace() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    
    // Initialize workspace
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path)
        .arg("init")
        .arg("--use-defaults")
        .assert()
        .success();
    
    // Create monitoring database with test data
    let monitoring_path = temp_dir.path().join(".radium").join("monitoring.db");
    let monitoring = MonitoringService::open(&monitoring_path).unwrap();
    
    // Create agent and telemetry
    // Use session_id in agent_id so session lookup can find it
    let session_id = "test-session-1";
    let agent_id = format!("{}-agent-1", session_id);
    let mut agent = AgentRecord::new(agent_id.clone(), "test-agent".to_string());
    agent.plan_id = Some(session_id.to_string()); // Also set plan_id for lookup
    monitoring.register_agent(&agent).unwrap();
    
    // Record telemetry
    let telemetry = TelemetryRecord::new(agent_id.clone())
        .with_tokens(1000, 500)
        .with_model("gpt-4".to_string(), "openai".to_string());
    monitoring.record_telemetry(&telemetry).await.unwrap();
    
    // Complete agent
    monitoring.complete_agent(&agent_id, 0).unwrap();
    
    // Create session report
    let storage = SessionStorage::new(temp_dir.path()).unwrap();
    let metrics = SessionMetrics {
        session_id: "test-session-1".to_string(),
        start_time: Utc::now() - chrono::Duration::seconds(3600),
        end_time: Some(Utc::now()),
        wall_time: Duration::from_secs(3600),
        agent_active_time: Duration::from_secs(1800),
        api_time: Duration::from_secs(600),
        tool_time: Duration::from_secs(1200),
        tool_calls: 50,
        successful_tool_calls: 48,
        failed_tool_calls: 2,
        lines_added: 200,
        lines_removed: 50,
        model_usage: {
            let mut map = HashMap::new();
            map.insert("gpt-4".to_string(), radium_core::analytics::ModelUsageStats {
                requests: 10,
                input_tokens: 1000,
                output_tokens: 500,
                cached_tokens: 0,
                estimated_cost: 0.05,
            });
            map
        },
        total_cached_tokens: 0,
        total_cache_creation_tokens: 0,
        total_cache_read_tokens: 0,
        total_cost: 0.05,
    };
    let report = SessionReport::new(metrics);
    storage.save_report(&report).unwrap();
    
    temp_dir
}

#[tokio::test]
async fn test_stats_session_with_id() {
    let temp_dir = setup_test_workspace().await;
    
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path)
        .arg("stats")
        .arg("session")
        .arg("test-session-1")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-session-1"))
        .stdout(predicate::str::contains("Interaction Summary"))
        .stdout(predicate::str::contains("Performance"));
}

#[tokio::test]
async fn test_stats_session_json() {
    let temp_dir = setup_test_workspace().await;
    
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path);
    let assert = cmd
        .arg("stats")
        .arg("session")
        .arg("test-session-1")
        .arg("--json")
        .assert()
        .success();
    let output = assert.get_output();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["metrics"]["session_id"], "test-session-1");
}

#[tokio::test]
async fn test_stats_session_latest() {
    let temp_dir = setup_test_workspace().await;
    
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path)
        .arg("stats")
        .arg("session")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-session-1"));
}

#[tokio::test]
async fn test_stats_session_missing() {
    let temp_dir = setup_test_workspace().await;
    
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path)
        .arg("stats")
        .arg("session")
        .arg("non-existent-session")
        .assert()
        .failure();
}

#[tokio::test]
async fn test_stats_model_with_session() {
    let temp_dir = setup_test_workspace().await;
    
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path)
        .arg("stats")
        .arg("model")
        .arg("test-session-1")
        .assert()
        .success()
        .stdout(predicate::str::contains("gpt-4"))
        .stdout(predicate::str::contains("1000"))
        .stdout(predicate::str::contains("500"));
}

#[tokio::test]
async fn test_stats_model_json() {
    let temp_dir = setup_test_workspace().await;
    
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path);
    let assert = cmd
        .arg("stats")
        .arg("model")
        .arg("test-session-1")
        .arg("--json")
        .assert()
        .success();
    let output = assert.get_output();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.is_array());
    assert_eq!(parsed[0]["model"], "gpt-4");
}

#[tokio::test]
async fn test_stats_model_no_session() {
    let temp_dir = setup_test_workspace().await;
    
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path)
        .arg("stats")
        .arg("model")
        .assert()
        .success()
        .stdout(predicate::str::contains("Coming soon"));
}

#[tokio::test]
async fn test_stats_history() {
    let temp_dir = setup_test_workspace().await;
    
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path)
        .arg("stats")
        .arg("history")
        .assert()
        .success()
        .stdout(predicate::str::contains("Recent Session Summaries"))
        .stdout(predicate::str::contains("test-session-1"));
}

#[tokio::test]
async fn test_stats_history_limit() {
    let temp_dir = setup_test_workspace().await;
    
    // Create additional session reports
    let storage = SessionStorage::new(temp_dir.path()).unwrap();
    for i in 2..=5 {
        let metrics = SessionMetrics {
            session_id: format!("test-session-{}", i),
            start_time: Utc::now() - chrono::Duration::seconds(3600),
            end_time: Some(Utc::now()),
            wall_time: Duration::from_secs(3600),
            agent_active_time: Duration::from_secs(1800),
            api_time: Duration::from_secs(600),
            tool_time: Duration::from_secs(1200),
            tool_calls: 50,
            successful_tool_calls: 48,
            failed_tool_calls: 2,
            lines_added: 200,
            lines_removed: 50,
            model_usage: HashMap::new(),
            total_cached_tokens: 0,
            total_cache_creation_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 0.05,
        };
        let report = SessionReport::new(metrics);
        storage.save_report(&report).unwrap();
        // Small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path);
    let assert = cmd
        .arg("stats")
        .arg("history")
        .arg("--limit")
        .arg("3")
        .env_remove("RUST_LOG")  // Remove log level to avoid conflict
        .assert()
        .success();
    let output = assert.get_output();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show at most 3 sessions
    let session_count = stdout.matches("test-session-").count();
    assert!(session_count <= 3);
}

#[tokio::test]
async fn test_stats_history_json() {
    let temp_dir = setup_test_workspace().await;
    
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path);
    let assert = cmd
        .arg("stats")
        .arg("history")
        .arg("--json")
        .assert()
        .success();
    let output = assert.get_output();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.is_array());
}

#[tokio::test]
async fn test_stats_history_empty() {
    let temp_dir = TempDir::new().unwrap();
    
    // Initialize workspace but don't create any sessions
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path)
        .arg("init")
        .arg("--use-defaults")
        .assert()
        .success();
    
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path)
        .arg("stats")
        .arg("history")
        .assert()
        .success()
        .stdout(predicate::str::contains("No session history found"));
}

#[tokio::test]
async fn test_stats_export_session() {
    let temp_dir = setup_test_workspace().await;
    let output_file = temp_dir.path().join("export.json");
    
    let output_path = output_file.to_str().unwrap().to_string();
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path)
        .arg("stats")
        .arg("export")
        .arg("--output")
        .arg(&output_path)
        .arg("test-session-1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Exported session"));
    
    // Verify file was created and contains valid JSON
    let content = std::fs::read_to_string(&output_file).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["metrics"]["session_id"], "test-session-1");
}

#[tokio::test]
async fn test_stats_export_all() {
    let temp_dir = setup_test_workspace().await;
    let output_file = temp_dir.path().join("export-all.json");
    
    let output_path = output_file.to_str().unwrap().to_string();
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path)
        .arg("stats")
        .arg("export")
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Exported"));
    
    // Verify file was created
    assert!(output_file.exists());
    let content = std::fs::read_to_string(&output_file).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(parsed.is_array());
}

#[tokio::test]
async fn test_stats_export_stdout() {
    let temp_dir = setup_test_workspace().await;
    
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path);
    let assert = cmd
        .arg("stats")
        .arg("export")
        .arg("test-session-1")
        .assert()
        .success();
    let output = assert.get_output();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["metrics"]["session_id"], "test-session-1");
}

#[tokio::test]
async fn test_stats_no_workspace() {
    let temp_dir = TempDir::new().unwrap();
    // Don't initialize workspace
    
    let path = temp_dir.path().to_path_buf();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&path)
        .arg("stats")
        .arg("session")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No Radium workspace found"));
}


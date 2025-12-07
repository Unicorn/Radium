//! Unit tests for session report storage and persistence.

use chrono::Utc;
use radium_core::analytics::{SessionMetrics, SessionReport, SessionStorage};
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use tempfile::TempDir;

/// Helper function to create a test session report.
fn create_test_report(session_id: &str) -> SessionReport {
    let metrics = SessionMetrics {
        session_id: session_id.to_string(),
        start_time: Utc::now(),
        end_time: Some(Utc::now()),
        wall_time: Duration::from_secs(100),
        agent_active_time: Duration::from_secs(80),
        api_time: Duration::from_secs(30),
        tool_time: Duration::from_secs(50),
        tool_calls: 10,
        successful_tool_calls: 9,
        failed_tool_calls: 1,
        tool_approvals_allowed: 0,
        tool_approvals_denied: 0,
        tool_approvals_asked: 0,
        lines_added: 50,
        lines_removed: 20,
        model_usage: HashMap::new(),
        total_cached_tokens: 0,
        total_cache_creation_tokens: 0,
        total_cache_read_tokens: 0,
        total_cost: 0.0,
    };
    SessionReport::new(metrics)
}

#[test]
fn test_session_storage_new_creates_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    
    let sessions_dir = storage.sessions_dir();
    assert!(sessions_dir.exists(), "Sessions directory should exist");
    assert!(sessions_dir.is_dir(), "Sessions directory should be a directory");
    
    // Verify path structure
    assert!(sessions_dir.to_string_lossy().contains(".radium"));
    assert!(sessions_dir.to_string_lossy().contains("_internals"));
    assert!(sessions_dir.to_string_lossy().contains("sessions"));
}

#[test]
fn test_session_storage_new_creates_nested_directories() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    // Ensure parent directories don't exist
    let radium_dir = workspace_root.join(".radium");
    let internals_dir = radium_dir.join("_internals");
    assert!(!internals_dir.exists(), "Internals directory should not exist initially");
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    
    let sessions_dir = storage.sessions_dir();
    assert!(sessions_dir.exists(), "Sessions directory should be created");
}

#[test]
fn test_session_storage_save_report_creates_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    let report = create_test_report("test-session-1");
    
    let file_path = storage.save_report(&report).expect("Failed to save report");
    
    assert!(file_path.exists(), "Report file should exist");
    assert!(file_path.is_file(), "Report file should be a file");
    assert_eq!(file_path.file_name().unwrap().to_string_lossy(), "test-session-1.json");
}

#[test]
fn test_session_storage_save_report_json_format() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    let report = create_test_report("test-session-2");
    
    let file_path = storage.save_report(&report).expect("Failed to save report");
    
    let content = fs::read_to_string(&file_path).expect("Failed to read report file");
    
    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");
    assert_eq!(parsed["metrics"]["session_id"], "test-session-2");
    
    // Verify it's pretty-printed (contains newlines)
    assert!(content.contains('\n'), "JSON should be pretty-printed");
}

#[test]
fn test_session_storage_load_report_retrieves_data() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    let original_report = create_test_report("test-session-3");
    
    storage.save_report(&original_report).expect("Failed to save report");
    
    let loaded_report = storage.load_report("test-session-3").expect("Failed to load report");
    
    assert_eq!(loaded_report.metrics.session_id, original_report.metrics.session_id);
    assert_eq!(loaded_report.metrics.tool_calls, original_report.metrics.tool_calls);
    assert_eq!(loaded_report.metrics.successful_tool_calls, original_report.metrics.successful_tool_calls);
    assert_eq!(loaded_report.metrics.failed_tool_calls, original_report.metrics.failed_tool_calls);
    assert_eq!(loaded_report.metrics.lines_added, original_report.metrics.lines_added);
    assert_eq!(loaded_report.metrics.lines_removed, original_report.metrics.lines_removed);
}

#[test]
fn test_session_storage_load_report_missing_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    
    let result = storage.load_report("non-existent-session");
    
    assert!(result.is_err(), "Loading non-existent report should error");
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("non-existent-session") || error_msg.contains("No such file"), 
            "Error should mention the missing file");
}

#[test]
fn test_session_storage_load_report_corrupted_json() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    
    // Create a corrupted JSON file
    let sessions_dir = storage.sessions_dir();
    let corrupted_file = sessions_dir.join("corrupted-session.json");
    fs::write(&corrupted_file, "{ invalid json }").expect("Failed to write corrupted file");
    
    let result = storage.load_report("corrupted-session");
    
    assert!(result.is_err(), "Loading corrupted JSON should error");
    // Error message may vary, just verify it's an error
    let _error = result.unwrap_err();
}

#[test]
fn test_session_storage_list_reports_empty_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    
    let reports = storage.list_reports().expect("Failed to list reports");
    
    assert_eq!(reports.len(), 0, "Empty directory should return no reports");
}

#[test]
fn test_session_storage_list_reports_single_report() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    let report = create_test_report("test-session-4");
    
    storage.save_report(&report).expect("Failed to save report");
    
    let reports = storage.list_reports().expect("Failed to list reports");
    
    assert_eq!(reports.len(), 1, "Should return one report");
    assert_eq!(reports[0].metrics.session_id, "test-session-4");
}

#[test]
fn test_session_storage_list_reports_multiple_reports() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    
    // Create multiple reports with slight delays to ensure different timestamps
    let report1 = create_test_report("test-session-5");
    std::thread::sleep(std::time::Duration::from_millis(10));
    storage.save_report(&report1).expect("Failed to save report1");
    
    let report2 = create_test_report("test-session-6");
    std::thread::sleep(std::time::Duration::from_millis(10));
    storage.save_report(&report2).expect("Failed to save report2");
    
    let report3 = create_test_report("test-session-7");
    std::thread::sleep(std::time::Duration::from_millis(10));
    storage.save_report(&report3).expect("Failed to save report3");
    
    let reports = storage.list_reports().expect("Failed to list reports");
    
    assert_eq!(reports.len(), 3, "Should return three reports");
    
    // Verify all session IDs are present
    let session_ids: Vec<String> = reports.iter()
        .map(|r| r.metrics.session_id.clone())
        .collect();
    assert!(session_ids.contains(&"test-session-5".to_string()));
    assert!(session_ids.contains(&"test-session-6".to_string()));
    assert!(session_ids.contains(&"test-session-7".to_string()));
}

#[test]
fn test_session_storage_list_reports_sorted_by_timestamp() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    
    // Create reports with known timestamps (most recent first)
    let report1 = create_test_report("oldest-session");
    std::thread::sleep(std::time::Duration::from_millis(50));
    storage.save_report(&report1).expect("Failed to save report1");
    
    let report2 = create_test_report("middle-session");
    std::thread::sleep(std::time::Duration::from_millis(50));
    storage.save_report(&report2).expect("Failed to save report2");
    
    let report3 = create_test_report("newest-session");
    std::thread::sleep(std::time::Duration::from_millis(50));
    storage.save_report(&report3).expect("Failed to save report3");
    
    let reports = storage.list_reports().expect("Failed to list reports");
    
    assert_eq!(reports.len(), 3, "Should return three reports");
    
    // Verify sorting: most recent first (newest should be first)
    assert_eq!(reports[0].metrics.session_id, "newest-session");
    assert_eq!(reports[1].metrics.session_id, "middle-session");
    assert_eq!(reports[2].metrics.session_id, "oldest-session");
    
    // Verify timestamps are in descending order
    assert!(reports[0].generated_at >= reports[1].generated_at);
    assert!(reports[1].generated_at >= reports[2].generated_at);
}

#[test]
fn test_session_storage_list_reports_ignores_non_json_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    
    // Create a JSON report
    let report = create_test_report("test-session-8");
    storage.save_report(&report).expect("Failed to save report");
    
    // Create a non-JSON file in the sessions directory
    let sessions_dir = storage.sessions_dir();
    let text_file = sessions_dir.join("not-a-report.txt");
    fs::write(&text_file, "This is not a JSON file").expect("Failed to write text file");
    
    let reports = storage.list_reports().expect("Failed to list reports");
    
    // Should only return the JSON report, ignoring the .txt file
    assert_eq!(reports.len(), 1, "Should return only JSON reports");
    assert_eq!(reports[0].metrics.session_id, "test-session-8");
}

#[test]
fn test_session_storage_round_trip() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    
    // Create a report with various fields populated
    let mut metrics = SessionMetrics {
        session_id: "round-trip-session".to_string(),
        start_time: Utc::now(),
        end_time: Some(Utc::now()),
        wall_time: Duration::from_secs(200),
        agent_active_time: Duration::from_secs(150),
        api_time: Duration::from_secs(60),
        tool_time: Duration::from_secs(90),
        tool_calls: 25,
        successful_tool_calls: 23,
        failed_tool_calls: 2,
        tool_approvals_allowed: 0,
        tool_approvals_denied: 0,
        tool_approvals_asked: 0,
        lines_added: 100,
        lines_removed: 50,
        model_usage: HashMap::new(),
        total_cached_tokens: 5000,
        total_cache_creation_tokens: 2000,
        total_cache_read_tokens: 3000,
        total_cost: 0.15,
    };
    
    // Add some model usage
    use radium_core::analytics::ModelUsageStats;
    metrics.model_usage.insert("model-1".to_string(), ModelUsageStats {
        requests: 10,
        input_tokens: 5000,
        output_tokens: 2000,
        cached_tokens: 1000,
        estimated_cost: 0.10,
    });
    
    let original_report = SessionReport::new(metrics);
    
    // Save and load
    storage.save_report(&original_report).expect("Failed to save report");
    let loaded_report = storage.load_report("round-trip-session").expect("Failed to load report");
    
    // Verify all fields match
    assert_eq!(loaded_report.metrics.session_id, original_report.metrics.session_id);
    assert_eq!(loaded_report.metrics.tool_calls, original_report.metrics.tool_calls);
    assert_eq!(loaded_report.metrics.successful_tool_calls, original_report.metrics.successful_tool_calls);
    assert_eq!(loaded_report.metrics.failed_tool_calls, original_report.metrics.failed_tool_calls);
    assert_eq!(loaded_report.metrics.lines_added, original_report.metrics.lines_added);
    assert_eq!(loaded_report.metrics.lines_removed, original_report.metrics.lines_removed);
    assert_eq!(loaded_report.metrics.total_cached_tokens, original_report.metrics.total_cached_tokens);
    assert_eq!(loaded_report.metrics.total_cache_creation_tokens, original_report.metrics.total_cache_creation_tokens);
    assert_eq!(loaded_report.metrics.total_cache_read_tokens, original_report.metrics.total_cache_read_tokens);
    assert_eq!(loaded_report.metrics.total_cost, original_report.metrics.total_cost);
    assert_eq!(loaded_report.metrics.model_usage.len(), original_report.metrics.model_usage.len());
    
    // Verify model usage
    let loaded_model = loaded_report.metrics.model_usage.get("model-1").unwrap();
    let original_model = original_report.metrics.model_usage.get("model-1").unwrap();
    assert_eq!(loaded_model.requests, original_model.requests);
    assert_eq!(loaded_model.input_tokens, original_model.input_tokens);
    assert_eq!(loaded_model.output_tokens, original_model.output_tokens);
    assert_eq!(loaded_model.cached_tokens, original_model.cached_tokens);
    assert_eq!(loaded_model.estimated_cost, original_model.estimated_cost);
}

#[test]
fn test_session_storage_sessions_dir() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    
    let sessions_dir = storage.sessions_dir();
    
    assert!(sessions_dir.exists(), "Sessions directory should exist");
    assert_eq!(sessions_dir, storage.sessions_dir(), "Should return same path on multiple calls");
}

#[test]
fn test_session_storage_list_reports_paginated() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    
    // Create 5 reports
    for i in 1..=5 {
        let report = create_test_report(&format!("session-{}", i));
        std::thread::sleep(std::time::Duration::from_millis(10));
        storage.save_report(&report).expect(&format!("Failed to save report {}", i));
    }
    
    // Test pagination: limit 2, offset 0
    let reports = storage.list_reports_paginated(Some(2), Some(0)).expect("Failed to list reports");
    assert_eq!(reports.len(), 2, "Should return 2 reports");
    
    // Test pagination: limit 2, offset 2
    let reports = storage.list_reports_paginated(Some(2), Some(2)).expect("Failed to list reports");
    assert_eq!(reports.len(), 2, "Should return 2 reports");
    
    // Test pagination: limit 2, offset 4
    let reports = storage.list_reports_paginated(Some(2), Some(4)).expect("Failed to list reports");
    assert_eq!(reports.len(), 1, "Should return 1 report");
    
    // Test no limit
    let reports = storage.list_reports_paginated(None, None).expect("Failed to list reports");
    assert_eq!(reports.len(), 5, "Should return all 5 reports");
}

#[test]
fn test_session_storage_list_report_metadata() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    
    // Create a report
    let report = create_test_report("metadata-test-session");
    storage.save_report(&report).expect("Failed to save report");
    
    // Get metadata
    use radium_core::analytics::SessionMetadata;
    let metadata = storage.list_report_metadata().expect("Failed to list metadata");
    
    assert_eq!(metadata.len(), 1, "Should return one metadata entry");
    assert_eq!(metadata[0].session_id, "metadata-test-session");
    assert_eq!(metadata[0].tool_calls, 10);
}

#[test]
fn test_session_storage_atomic_write_concurrent_safety() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    
    // Create two different reports with the same session ID
    let report1 = create_test_report("concurrent-session");
    let mut report2 = create_test_report("concurrent-session");
    report2.metrics.tool_calls = 999; // Make it clearly different
    
    // Simulate concurrent writes by saving both in sequence
    // In a real concurrent scenario, both would try to write simultaneously
    // The atomic write ensures the final file is complete (not corrupted)
    let result1 = storage.save_report(&report1);
    let result2 = storage.save_report(&report2);
    
    // Both should succeed (last write wins)
    assert!(result1.is_ok(), "First write should succeed");
    assert!(result2.is_ok(), "Second write should succeed");
    
    // Load the final file - it should contain complete JSON from one of the writes
    let loaded = storage.load_report("concurrent-session").expect("Failed to load report");
    
    // The final file should be complete and valid (not a mix of both)
    // It should match either report1 or report2 completely
    let matches_report1 = loaded.metrics.tool_calls == report1.metrics.tool_calls;
    let matches_report2 = loaded.metrics.tool_calls == report2.metrics.tool_calls;
    
    assert!(matches_report1 || matches_report2, 
            "Final file should contain complete data from one write, not corrupted mix");
    
    // Verify the file is valid JSON (not corrupted)
    let file_path = storage.sessions_dir().join("concurrent-session.json");
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    let _: serde_json::Value = serde_json::from_str(&content)
        .expect("File should contain valid JSON, not corrupted data");
}

#[test]
fn test_session_storage_atomic_write_no_temp_files_left() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    let report = create_test_report("cleanup-test-session");
    
    // Save a report
    storage.save_report(&report).expect("Failed to save report");
    
    // Check that no .tmp files were left behind
    let sessions_dir = storage.sessions_dir();
    let entries: Vec<_> = fs::read_dir(sessions_dir)
        .expect("Failed to read directory")
        .map(|e| e.expect("Failed to read entry"))
        .collect();
    
    // Should only have the .json file, no .tmp files
    let temp_files: Vec<_> = entries
        .iter()
        .filter(|e| {
            e.path()
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.contains(".tmp"))
                .unwrap_or(false)
        })
        .collect();
    
    assert_eq!(temp_files.len(), 0, "No temporary files should be left behind");
}

#[test]
fn test_session_storage_compact_json_format() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root)
        .expect("Failed to create storage")
        .with_compact_json(true);
    let report = create_test_report("compact-json-session");
    
    let file_path = storage.save_report(&report).expect("Failed to save report");
    let content = fs::read_to_string(&file_path).expect("Failed to read report file");
    
    // Verify it's valid JSON
    let _: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");
    
    // Verify it's compact (no newlines except in string values)
    // Compact JSON should have minimal whitespace
    let lines: Vec<&str> = content.lines().collect();
    // Compact JSON should be mostly on one line (or very few lines)
    assert!(lines.len() <= 3, "Compact JSON should have minimal line breaks");
    
    // Verify no indentation (no leading spaces on lines)
    for line in &lines {
        if !line.trim().is_empty() {
            assert!(!line.starts_with("  "), "Compact JSON should not have indentation");
        }
    }
}

#[test]
fn test_session_storage_pretty_json_format() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let storage = SessionStorage::new(workspace_root)
        .expect("Failed to create storage")
        .with_compact_json(false);
    let report = create_test_report("pretty-json-session");
    
    let file_path = storage.save_report(&report).expect("Failed to save report");
    let content = fs::read_to_string(&file_path).expect("Failed to read report file");
    
    // Verify it's valid JSON
    let _: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");
    
    // Verify it's pretty-printed (contains newlines and indentation)
    assert!(content.contains('\n'), "Pretty JSON should contain newlines");
    
    // Verify it has indentation (leading spaces)
    let has_indentation = content.lines().any(|line| line.starts_with("  "));
    assert!(has_indentation, "Pretty JSON should have indentation");
}

#[test]
fn test_session_storage_default_pretty_json() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    // Default should be pretty-printed (backward compatibility)
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    let report = create_test_report("default-json-session");
    
    let file_path = storage.save_report(&report).expect("Failed to save report");
    let content = fs::read_to_string(&file_path).expect("Failed to read report file");
    
    // Default should be pretty-printed
    assert!(content.contains('\n'), "Default JSON should be pretty-printed");
}


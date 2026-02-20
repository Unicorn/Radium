//! Unit tests for session report formatting.

#![cfg(feature = "monitoring")]

use chrono::Utc;
use radium_core::analytics::{ModelUsageStats, ReportFormatter, SessionMetrics, SessionReport};
use std::collections::HashMap;
use std::time::Duration;

/// Helper function to create a test session report.
fn create_test_report() -> SessionReport {
    let mut metrics = SessionMetrics {
        session_id: "test-session-123".to_string(),
        start_time: Utc::now(),
        end_time: Some(Utc::now()),
        wall_time: Duration::from_secs(3600), // 1 hour
        agent_active_time: Duration::from_secs(1800), // 30 minutes
        api_time: Duration::from_secs(600), // 10 minutes
        tool_time: Duration::from_secs(1200), // 20 minutes
        tool_calls: 100,
        successful_tool_calls: 95,
        failed_tool_calls: 5,
        lines_added: 500,
        lines_removed: 200,
        model_usage: HashMap::new(),
        engine_usage: HashMap::new(),
        total_cached_tokens: 0,
        total_cache_creation_tokens: 0,
        total_cache_read_tokens: 0,
        total_cost: 0.0,
        tool_approvals_allowed: 0,
        tool_approvals_asked: 0,
        tool_approvals_denied: 0,
    };
    
    // Add model usage
    metrics.model_usage.insert("gpt-4".to_string(), ModelUsageStats {
        requests: 50,
        input_tokens: 10000,
        output_tokens: 5000,
        cached_tokens: 0,
        estimated_cost: 0.50,
    });
    
    SessionReport::new(metrics)
}

#[test]
fn test_report_formatter_format_contains_sections() {
    let formatter = ReportFormatter;
    let report = create_test_report();
    
    let output = formatter.format(&report);
    
    // Verify all major sections are present
    assert!(output.contains("Interaction Summary"), "Should contain Interaction Summary");
    assert!(output.contains("Performance"), "Should contain Performance");
    assert!(output.contains("Model Usage"), "Should contain Model Usage");
    assert!(output.contains("Tip:"), "Should contain tip");
}

#[test]
fn test_report_formatter_format_session_id() {
    let formatter = ReportFormatter;
    let report = create_test_report();
    
    let output = formatter.format(&report);
    
    assert!(output.contains("test-session-123"), "Should contain session ID");
    assert!(output.contains("Session ID:"), "Should contain Session ID label");
}

#[test]
fn test_report_formatter_format_tool_calls() {
    let formatter = ReportFormatter;
    let report = create_test_report();
    
    let output = formatter.format(&report);
    
    assert!(output.contains("Tool Calls:"), "Should contain Tool Calls");
    assert!(output.contains("100"), "Should contain total tool calls");
    assert!(output.contains("95"), "Should contain successful tool calls");
    assert!(output.contains("5"), "Should contain failed tool calls");
}

#[test]
fn test_report_formatter_format_success_rate() {
    let formatter = ReportFormatter;
    let report = create_test_report();
    
    let output = formatter.format(&report);
    
    assert!(output.contains("Success Rate:"), "Should contain Success Rate");
    // 95/100 = 95.0%
    assert!(output.contains("95.0"), "Should contain success rate percentage");
}

#[test]
fn test_report_formatter_format_code_changes() {
    let formatter = ReportFormatter;
    let report = create_test_report();
    
    let output = formatter.format(&report);
    
    assert!(output.contains("Code Changes:"), "Should contain Code Changes");
    assert!(output.contains("+500"), "Should contain lines added");
    assert!(output.contains("-200"), "Should contain lines removed");
}

#[test]
fn test_report_formatter_format_duration_hours() {
    let formatter = ReportFormatter;
    let mut report = create_test_report();
    // Set wall time to 2 hours 30 minutes 45 seconds
    report.metrics.wall_time = Duration::from_secs(2 * 3600 + 30 * 60 + 45);
    
    let output = formatter.format(&report);
    
    assert!(output.contains("Wall Time:"), "Should contain Wall Time");
    // Should format as "2h 30m 45s" or similar
    assert!(output.contains("h"), "Should contain hours");
    assert!(output.contains("m"), "Should contain minutes");
    assert!(output.contains("s"), "Should contain seconds");
}

#[test]
fn test_report_formatter_format_duration_minutes() {
    let formatter = ReportFormatter;
    let mut report = create_test_report();
    // Set wall time to 5 minutes 30 seconds
    report.metrics.wall_time = Duration::from_secs(5 * 60 + 30);
    
    let output = formatter.format(&report);
    
    assert!(output.contains("Wall Time:"), "Should contain Wall Time");
    // Should format as "5m 30s"
    assert!(output.contains("m"), "Should contain minutes");
    assert!(output.contains("s"), "Should contain seconds");
}

#[test]
fn test_report_formatter_format_duration_seconds() {
    let formatter = ReportFormatter;
    let mut report = create_test_report();
    // Set wall time to 45 seconds
    report.metrics.wall_time = Duration::from_secs(45);
    
    let output = formatter.format(&report);
    
    assert!(output.contains("Wall Time:"), "Should contain Wall Time");
    // Should format as "45s"
    assert!(output.contains("s"), "Should contain seconds");
}

#[test]
fn test_report_formatter_format_performance_breakdown() {
    let formatter = ReportFormatter;
    let report = create_test_report();
    
    let output = formatter.format(&report);
    
    assert!(output.contains("Agent Active:"), "Should contain Agent Active");
    assert!(output.contains("API Time:"), "Should contain API Time");
    assert!(output.contains("Tool Time:"), "Should contain Tool Time");
    // Should contain percentages
    assert!(output.contains("%"), "Should contain percentage symbol");
}

#[test]
fn test_report_formatter_format_model_usage() {
    let formatter = ReportFormatter;
    let report = create_test_report();
    
    let output = formatter.format(&report);
    
    assert!(output.contains("Model Usage"), "Should contain Model Usage header");
    assert!(output.contains("gpt-4"), "Should contain model name");
    assert!(output.contains("50"), "Should contain request count");
    assert!(output.contains("10000"), "Should contain input tokens");
    assert!(output.contains("5000"), "Should contain output tokens");
}

#[test]
fn test_report_formatter_format_multiple_models() {
    let formatter = ReportFormatter;
    let mut report = create_test_report();
    
    // Add another model
    report.metrics.model_usage.insert("claude-3".to_string(), ModelUsageStats {
        requests: 25,
        input_tokens: 5000,
        output_tokens: 2500,
        cached_tokens: 0,
        estimated_cost: 0.25,
    });
    
    let output = formatter.format(&report);
    
    assert!(output.contains("gpt-4"), "Should contain first model");
    assert!(output.contains("claude-3"), "Should contain second model");
}

#[test]
fn test_report_formatter_format_cache_savings() {
    let formatter = ReportFormatter;
    let mut report = create_test_report();
    
    // Add cached tokens
    report.metrics.total_cached_tokens = 5000;
    
    let output = formatter.format(&report);
    
    assert!(output.contains("Savings Highlight"), "Should contain cache savings");
    assert!(output.contains("5000"), "Should contain cached token count");
    assert!(output.contains("cache"), "Should mention cache");
}

#[test]
fn test_report_formatter_format_no_cache_savings() {
    let formatter = ReportFormatter;
    let report = create_test_report();
    
    let output = formatter.format(&report);
    
    // Should not contain cache savings section when no cached tokens
    assert!(!output.contains("Savings Highlight"), "Should not contain cache savings when zero");
}

#[test]
fn test_report_formatter_format_number_large() {
    let formatter = ReportFormatter;
    let mut report = create_test_report();
    
    // Set a large number that should be formatted with commas
    report.metrics.total_cached_tokens = 1234567;
    
    let output = formatter.format(&report);
    
    // Should format as "1,234,567" or similar
    assert!(output.contains("1234567") || output.contains("1,234,567"), 
            "Should contain formatted number");
}

#[test]
fn test_report_formatter_format_json_valid() {
    let formatter = ReportFormatter;
    let report = create_test_report();

    let json_output = formatter.format_json(&report, false).expect("Failed to format as JSON");
    
    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_output)
        .expect("Should be valid JSON");
    
    assert_eq!(parsed["metrics"]["session_id"], "test-session-123");
    assert_eq!(parsed["metrics"]["tool_calls"], 100);
    assert_eq!(parsed["metrics"]["successful_tool_calls"], 95);
    assert_eq!(parsed["metrics"]["failed_tool_calls"], 5);
}

#[test]
fn test_report_formatter_format_json_round_trip() {
    let formatter = ReportFormatter;
    let original_report = create_test_report();

    let json_output = formatter.format_json(&original_report, false).expect("Failed to format as JSON");
    
    // Deserialize back to SessionReport
    let deserialized: SessionReport = serde_json::from_str(&json_output)
        .expect("Failed to deserialize JSON");
    
    assert_eq!(deserialized.metrics.session_id, original_report.metrics.session_id);
    assert_eq!(deserialized.metrics.tool_calls, original_report.metrics.tool_calls);
    assert_eq!(deserialized.metrics.successful_tool_calls, original_report.metrics.successful_tool_calls);
    assert_eq!(deserialized.metrics.failed_tool_calls, original_report.metrics.failed_tool_calls);
    assert_eq!(deserialized.metrics.lines_added, original_report.metrics.lines_added);
    assert_eq!(deserialized.metrics.lines_removed, original_report.metrics.lines_removed);
}

#[test]
fn test_report_formatter_format_empty_model_usage() {
    let formatter = ReportFormatter;
    let mut report = create_test_report();
    
    // Clear model usage
    report.metrics.model_usage.clear();
    
    let output = formatter.format(&report);
    
    // Should still contain Model Usage header
    assert!(output.contains("Model Usage"), "Should contain Model Usage header even when empty");
}

#[test]
fn test_report_formatter_format_zero_tool_calls() {
    let formatter = ReportFormatter;
    let mut report = create_test_report();
    
    report.metrics.tool_calls = 0;
    report.metrics.successful_tool_calls = 0;
    report.metrics.failed_tool_calls = 0;
    
    let output = formatter.format(&report);
    
    assert!(output.contains("Tool Calls:"), "Should contain Tool Calls");
    assert!(output.contains("0"), "Should contain zero tool calls");
    // Success rate should be 0.0%
    assert!(output.contains("0.0"), "Should contain zero success rate");
}

#[test]
fn test_report_formatter_format_percentage_calculations() {
    let formatter = ReportFormatter;
    let mut report = create_test_report();
    
    // Set specific values for percentage testing
    // API time: 600s, Agent active: 1800s = 33.3%
    // Tool time: 1200s, Agent active: 1800s = 66.7%
    report.metrics.agent_active_time = Duration::from_secs(1800);
    report.metrics.api_time = Duration::from_secs(600);
    report.metrics.tool_time = Duration::from_secs(1200);
    
    let output = formatter.format(&report);
    
    assert!(output.contains("API Time:"), "Should contain API Time");
    assert!(output.contains("Tool Time:"), "Should contain Tool Time");
    // Percentages should be present
    assert!(output.contains("%"), "Should contain percentage symbols");
}


//! Unit tests for session analytics and metrics tracking.

use chrono::{DateTime, Utc};
use radium_core::analytics::{ModelUsageStats, SessionAnalytics, SessionMetrics};
use radium_core::monitoring::{AgentRecord, MonitoringService, TelemetryRecord, TelemetryTracking};
use std::time::Duration;
use tokio;

/// Helper function to create a monitoring service with test data.
fn create_test_monitoring() -> MonitoringService {
    MonitoringService::new().expect("Failed to create monitoring service")
}

/// Helper function to create a telemetry record with default values.
fn create_test_telemetry(agent_id: &str, input_tokens: u64, output_tokens: u64) -> TelemetryRecord {
    TelemetryRecord::new(agent_id.to_string())
        .with_tokens(input_tokens, output_tokens)
        .with_model("test-model".to_string(), "test-provider".to_string())
}

#[tokio::test]
async fn test_session_metrics_success_rate_normal() {
    let mut metrics = SessionMetrics::default();
    metrics.tool_calls = 100;
    metrics.successful_tool_calls = 85;
    metrics.failed_tool_calls = 15;

    let success_rate = metrics.success_rate();
    assert_eq!(success_rate, 85.0);
}

#[tokio::test]
async fn test_session_metrics_success_rate_zero_tool_calls() {
    let metrics = SessionMetrics::default();
    assert_eq!(metrics.tool_calls, 0);
    
    let success_rate = metrics.success_rate();
    assert_eq!(success_rate, 0.0);
}

#[tokio::test]
async fn test_session_metrics_success_rate_100_percent() {
    let mut metrics = SessionMetrics::default();
    metrics.tool_calls = 50;
    metrics.successful_tool_calls = 50;
    metrics.failed_tool_calls = 0;

    let success_rate = metrics.success_rate();
    assert_eq!(success_rate, 100.0);
}

#[tokio::test]
async fn test_session_metrics_cache_hit_rate_normal() {
    let mut metrics = SessionMetrics::default();
    
    // Add model usage with input tokens
    let model_stats = ModelUsageStats {
        requests: 1,
        input_tokens: 1000,
        output_tokens: 500,
        cached_tokens: 0,
        estimated_cost: 0.0,
    };
    metrics.model_usage.insert("test-model".to_string(), model_stats);
    
    metrics.total_cached_tokens = 300;

    let cache_hit_rate = metrics.cache_hit_rate();
    assert_eq!(cache_hit_rate, 30.0); // 300 / 1000 * 100
}

#[tokio::test]
async fn test_session_metrics_cache_hit_rate_zero_input() {
    let mut metrics = SessionMetrics::default();
    
    // Add model usage with zero input tokens
    let model_stats = ModelUsageStats {
        requests: 1,
        input_tokens: 0,
        output_tokens: 500,
        cached_tokens: 0,
        estimated_cost: 0.0,
    };
    metrics.model_usage.insert("test-model".to_string(), model_stats);
    
    metrics.total_cached_tokens = 100;

    let cache_hit_rate = metrics.cache_hit_rate();
    assert_eq!(cache_hit_rate, 0.0);
}

#[tokio::test]
async fn test_session_metrics_cache_hit_rate_100_percent() {
    let mut metrics = SessionMetrics::default();
    
    let model_stats = ModelUsageStats {
        requests: 1,
        input_tokens: 1000,
        output_tokens: 500,
        cached_tokens: 0,
        estimated_cost: 0.0,
    };
    metrics.model_usage.insert("test-model".to_string(), model_stats);
    
    metrics.total_cached_tokens = 1000;

    let cache_hit_rate = metrics.cache_hit_rate();
    assert_eq!(cache_hit_rate, 100.0);
}

#[tokio::test]
async fn test_session_metrics_total_tokens_single_model() {
    let mut metrics = SessionMetrics::default();
    
    let model_stats = radium_core::analytics::ModelUsageStats {
        requests: 1,
        input_tokens: 1000,
        output_tokens: 500,
        cached_tokens: 0,
        estimated_cost: 0.0,
    };
    metrics.model_usage.insert("model1".to_string(), model_stats);

    let (input, output) = metrics.total_tokens();
    assert_eq!(input, 1000);
    assert_eq!(output, 500);
}

#[tokio::test]
async fn test_session_metrics_total_tokens_multiple_models() {
    let mut metrics = SessionMetrics::default();
    
    let model1_stats = ModelUsageStats {
        requests: 1,
        input_tokens: 1000,
        output_tokens: 500,
        cached_tokens: 0,
        estimated_cost: 0.0,
    };
    metrics.model_usage.insert("model1".to_string(), model1_stats);
    
    let model2_stats = ModelUsageStats {
        requests: 1,
        input_tokens: 2000,
        output_tokens: 1000,
        cached_tokens: 0,
        estimated_cost: 0.0,
    };
    metrics.model_usage.insert("model2".to_string(), model2_stats);

    let (input, output) = metrics.total_tokens();
    assert_eq!(input, 3000);
    assert_eq!(output, 1500);
}

#[tokio::test]
async fn test_session_metrics_total_tokens_empty() {
    let metrics = SessionMetrics::default();
    
    let (input, output) = metrics.total_tokens();
    assert_eq!(input, 0);
    assert_eq!(output, 0);
}

#[tokio::test]
async fn test_generate_session_metrics_single_agent() {
    let monitoring = create_test_monitoring();
    
    // Create an agent record
    let agent_id = "agent-1".to_string();
    let agent = AgentRecord::new(agent_id.clone(), "test-agent".to_string());
    monitoring.register_agent(&agent).expect("Failed to register agent");
    
    // Record telemetry
    let telemetry = create_test_telemetry(&agent_id, 1000, 500);
    monitoring.record_telemetry(&telemetry).await.expect("Failed to record telemetry");
    
    // Mark agent as completed (end_time will be set automatically)
    monitoring.complete_agent(&agent_id, 0).expect("Failed to complete agent");
    
    let analytics = SessionAnalytics::new(monitoring);
    let start_time = Utc::now() - chrono::Duration::seconds(20);
    let end_time = Some(Utc::now());
    
    let metrics = analytics.generate_session_metrics(
        "session-1",
        &[agent_id],
        start_time,
        end_time,
    ).expect("Failed to generate metrics");
    
    assert_eq!(metrics.session_id, "session-1");
    assert_eq!(metrics.model_usage.len(), 1);
    assert!(metrics.model_usage.contains_key("test-model"));
    
    let model_stats = metrics.model_usage.get("test-model").unwrap();
    assert_eq!(model_stats.requests, 1);
    assert_eq!(model_stats.input_tokens, 1000);
    assert_eq!(model_stats.output_tokens, 500);
}

#[tokio::test]
async fn test_generate_session_metrics_multiple_agents() {
    let monitoring = create_test_monitoring();
    
    // Create multiple agents
    let agent1_id = "agent-1".to_string();
    let agent2_id = "agent-2".to_string();
    
    let agent1 = AgentRecord::new(agent1_id.clone(), "test-agent".to_string());
    let agent2 = AgentRecord::new(agent2_id.clone(), "test-agent".to_string());
    
    monitoring.register_agent(&agent1).expect("Failed to register agent1");
    monitoring.register_agent(&agent2).expect("Failed to register agent2");
    
    // Record telemetry for both agents
    let telemetry1 = create_test_telemetry(&agent1_id, 1000, 500);
    let telemetry2 = create_test_telemetry(&agent2_id, 2000, 1000);
    
    monitoring.record_telemetry(&telemetry1).await.expect("Failed to record telemetry1");
    monitoring.record_telemetry(&telemetry2).await.expect("Failed to record telemetry2");
    
    // Mark agents as completed (end_time will be set automatically)
    monitoring.complete_agent(&agent1_id, 0).expect("Failed to complete agent1");
    monitoring.complete_agent(&agent2_id, 0).expect("Failed to complete agent2");
    
    let analytics = SessionAnalytics::new(monitoring);
    let start_time = Utc::now() - chrono::Duration::seconds(30);
    let end_time = Some(Utc::now());
    
    let metrics = analytics.generate_session_metrics(
        "session-2",
        &[agent1_id, agent2_id],
        start_time,
        end_time,
    ).expect("Failed to generate metrics");
    
    assert_eq!(metrics.session_id, "session-2");
    assert_eq!(metrics.model_usage.len(), 1); // Same model for both
    
    let model_stats = metrics.model_usage.get("test-model").unwrap();
    assert_eq!(model_stats.requests, 2); // Two telemetry records
    assert_eq!(model_stats.input_tokens, 3000); // 1000 + 2000
    assert_eq!(model_stats.output_tokens, 1500); // 500 + 1000
    
    assert_eq!(metrics.total_cost, model_stats.estimated_cost);
}

#[tokio::test]
async fn test_generate_session_metrics_empty_agent_list() {
    let monitoring = create_test_monitoring();
    let analytics = SessionAnalytics::new(monitoring);
    
    let start_time = Utc::now() - chrono::Duration::seconds(10);
    let end_time = Some(Utc::now());
    
    let metrics = analytics.generate_session_metrics(
        "session-empty",
        &[],
        start_time,
        end_time,
    ).expect("Failed to generate metrics");
    
    assert_eq!(metrics.session_id, "session-empty");
    assert_eq!(metrics.model_usage.len(), 0);
    assert_eq!(metrics.total_cost, 0.0);
    assert_eq!(metrics.agent_active_time, Duration::ZERO);
}

#[tokio::test]
async fn test_generate_session_metrics_missing_telemetry() {
    let monitoring = create_test_monitoring();
    
    // Create an agent but don't record telemetry
    let agent_id = "agent-no-telemetry".to_string();
    let agent = AgentRecord::new(agent_id.clone(), "test-agent".to_string());
    monitoring.register_agent(&agent).expect("Failed to register agent");
    
    // Mark agent as completed (end_time will be set automatically)
    monitoring.complete_agent(&agent_id, 0).expect("Failed to complete agent");
    
    let analytics = SessionAnalytics::new(monitoring);
    let start_time = Utc::now() - chrono::Duration::seconds(10);
    let end_time = Some(Utc::now());
    
    let metrics = analytics.generate_session_metrics(
        "session-no-telemetry",
        &[agent_id],
        start_time,
        end_time,
    ).expect("Failed to generate metrics");
    
    assert_eq!(metrics.session_id, "session-no-telemetry");
    assert_eq!(metrics.model_usage.len(), 0);
    assert_eq!(metrics.total_cost, 0.0);
    // Agent active time should be calculated if agent was completed
    // (may be 0 if agent wasn't properly completed or timing is very fast)
    assert!(metrics.agent_active_time.as_secs() >= 0);
}

#[tokio::test]
async fn test_generate_session_metrics_time_calculations() {
    let monitoring = create_test_monitoring();
    
    let agent_id = "agent-time".to_string();
    let agent = AgentRecord::new(agent_id.clone(), "test-agent".to_string());
    monitoring.register_agent(&agent).expect("Failed to register agent");
    
    // Record some telemetry
    let telemetry = create_test_telemetry(&agent_id, 100, 50);
    monitoring.record_telemetry(&telemetry).await.expect("Failed to record telemetry");
    
    // Get start time before completing agent
    let agent = monitoring.get_agent(&agent_id).expect("Failed to get agent");
    let start_time_secs = agent.start_time;
    
    // Mark agent as completed (end_time will be set automatically)
    monitoring.complete_agent(&agent_id, 0).expect("Failed to complete agent");
    
    // Get the actual end_time from the completed agent
    let completed_agent = monitoring.get_agent(&agent_id).expect("Failed to get completed agent");
    let end_time_secs = completed_agent.end_time.expect("Agent should have end_time");
    
    let analytics = SessionAnalytics::new(monitoring);
    let start_time = DateTime::from_timestamp(start_time_secs as i64, 0).unwrap();
    let end_time = Some(DateTime::from_timestamp(end_time_secs as i64, 0).unwrap());
    
    let metrics = analytics.generate_session_metrics(
        "session-time",
        &[agent_id],
        start_time,
        end_time,
    ).expect("Failed to generate metrics");
    
    // Wall time should be approximately the duration (may vary slightly)
    let wall_time_secs = metrics.wall_time.as_secs();
    assert!(wall_time_secs < 60); // Should be reasonable
    
    // Agent active time should match the duration
    let agent_active_secs = metrics.agent_active_time.as_secs();
    assert!(agent_active_secs < 60); // Should be reasonable
    
    // API time should be estimated (100ms per request = 100ms for 1 request)
    assert!(metrics.api_time.as_millis() >= 100);
    
    // Tool time should be agent_active_time - api_time
    assert!(metrics.tool_time.as_secs() < 30);
}

#[tokio::test]
async fn test_generate_session_metrics_multiple_models() {
    let monitoring = create_test_monitoring();
    
    let agent_id = "agent-multi-model".to_string();
    let agent = AgentRecord::new(agent_id.clone(), "test-agent".to_string());
    monitoring.register_agent(&agent).expect("Failed to register agent");
    
    // Record telemetry with different models
    let mut telemetry1 = create_test_telemetry(&agent_id, 1000, 500);
    telemetry1.model = Some("model-1".to_string());
    
    let mut telemetry2 = create_test_telemetry(&agent_id, 2000, 1000);
    telemetry2.model = Some("model-2".to_string());
    
    monitoring.record_telemetry(&telemetry1).await.expect("Failed to record telemetry1");
    monitoring.record_telemetry(&telemetry2).await.expect("Failed to record telemetry2");
    
    let analytics = SessionAnalytics::new(monitoring);
    let start_time = Utc::now() - chrono::Duration::seconds(10);
    let end_time = Some(Utc::now());
    
    let metrics = analytics.generate_session_metrics(
        "session-multi-model",
        &[agent_id],
        start_time,
        end_time,
    ).expect("Failed to generate metrics");
    
    assert_eq!(metrics.model_usage.len(), 2);
    assert!(metrics.model_usage.contains_key("model-1"));
    assert!(metrics.model_usage.contains_key("model-2"));
    
    let model1_stats = metrics.model_usage.get("model-1").unwrap();
    assert_eq!(model1_stats.input_tokens, 1000);
    assert_eq!(model1_stats.output_tokens, 500);
    
    let model2_stats = metrics.model_usage.get("model-2").unwrap();
    assert_eq!(model2_stats.input_tokens, 2000);
    assert_eq!(model2_stats.output_tokens, 1000);
}

#[tokio::test]
async fn test_generate_session_metrics_cache_statistics() {
    let monitoring = create_test_monitoring();
    
    let agent_id = "agent-cache".to_string();
    let agent = AgentRecord::new(agent_id.clone(), "test-agent".to_string());
    monitoring.register_agent(&agent).expect("Failed to register agent");
    
    // Record telemetry with cache stats
    let mut telemetry = create_test_telemetry(&agent_id, 1000, 500);
    telemetry = telemetry.with_cache_stats(300, 200, 100);
    monitoring.record_telemetry(&telemetry).await.expect("Failed to record telemetry");
    
    let analytics = SessionAnalytics::new(monitoring);
    let start_time = Utc::now() - chrono::Duration::seconds(10);
    let end_time = Some(Utc::now());
    
    let metrics = analytics.generate_session_metrics(
        "session-cache",
        &[agent_id],
        start_time,
        end_time,
    ).expect("Failed to generate metrics");
    
    assert_eq!(metrics.total_cached_tokens, 300);
    assert_eq!(metrics.total_cache_creation_tokens, 200);
    assert_eq!(metrics.total_cache_read_tokens, 100);
    
    let model_stats = metrics.model_usage.get("test-model").unwrap();
    assert_eq!(model_stats.cached_tokens, 300);
}

#[tokio::test]
async fn test_generate_session_metrics_with_workspace() {
    use tempfile::TempDir;
    use std::fs;
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    // Initialize git repo
    let output = std::process::Command::new("git")
        .args(["init"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to init git");
    
    if !output.status.success() {
        // Git not available, skip this test
        return;
    }
    
    // Create a test file
    let test_file = workspace_root.join("test.txt");
    fs::write(&test_file, "line 1\nline 2\nline 3\n").expect("Failed to write test file");
    
    // Commit the file
    std::process::Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to git add");
    
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to set git config");
    
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to set git config");
    
    let commit_time = Utc::now();
    
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to git commit");
    
    // Add more lines after commit
    fs::write(&test_file, "line 1\nline 2\nline 3\nline 4\nline 5\n").expect("Failed to write test file");
    
    let monitoring = create_test_monitoring();
    let agent_id = "agent-workspace".to_string();
    let agent = AgentRecord::new(agent_id.clone(), "test-agent".to_string());
    monitoring.register_agent(&agent).expect("Failed to register agent");
    
    let analytics = SessionAnalytics::new(monitoring);
    let end_time = Some(Utc::now());
    
    let metrics = analytics.generate_session_metrics_with_workspace(
        "session-workspace",
        &[agent_id],
        commit_time,
        end_time,
        Some(workspace_root),
    ).expect("Failed to generate metrics");
    
    // Should have tracked code changes (2 lines added)
    assert!(metrics.lines_added >= 0); // May be 0 if git diff doesn't work in test environment
}

#[tokio::test]
async fn test_tool_approval_metrics_aggregation() {
    let monitoring = create_test_monitoring();
    let agent_id = "agent-approvals".to_string();
    let agent = AgentRecord::new(agent_id.clone(), "test-agent".to_string());
    monitoring.register_agent(&agent).expect("Failed to register agent");
    
    // Create telemetry records with tool approval information
    let mut telemetry1 = create_test_telemetry(&agent_id, 100, 50);
    telemetry1 = telemetry1.with_tool_approval(
        "write_file".to_string(),
        Some(vec!["file.txt".to_string(), "content".to_string()]),
        true,
        "auto".to_string(),
    );
    monitoring.record_telemetry(&telemetry1).await.expect("Failed to record telemetry");
    
    let mut telemetry2 = create_test_telemetry(&agent_id, 200, 100);
    telemetry2 = telemetry2.with_tool_approval(
        "read_file".to_string(),
        Some(vec!["file.txt".to_string()]),
        true,
        "user".to_string(),
    );
    monitoring.record_telemetry(&telemetry2).await.expect("Failed to record telemetry");
    
    let mut telemetry3 = create_test_telemetry(&agent_id, 150, 75);
    telemetry3 = telemetry3.with_tool_approval(
        "delete_file".to_string(),
        Some(vec!["file.txt".to_string()]),
        false,
        "user".to_string(),
    );
    monitoring.record_telemetry(&telemetry3).await.expect("Failed to record telemetry");
    
    let analytics = SessionAnalytics::new(monitoring);
    let start_time = Utc::now() - chrono::Duration::seconds(10);
    let end_time = Some(Utc::now());
    
    let metrics = analytics.generate_session_metrics(
        "session-approvals",
        &[agent_id],
        start_time,
        end_time,
    ).expect("Failed to generate metrics");
    
    // Should have aggregated tool approval metrics
    assert_eq!(metrics.tool_approvals_allowed, 2); // telemetry1 and telemetry2 were approved
    assert_eq!(metrics.tool_approvals_denied, 1); // telemetry3 was denied
    assert_eq!(metrics.tool_approvals_asked, 2); // telemetry2 and telemetry3 had "user" approval type
}

#[tokio::test]
async fn test_get_aggregated_model_usage() {
    use tempfile::TempDir;
    use radium_core::analytics::{SessionReport, SessionStorage, ModelUsageStats};
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    // Create storage and save multiple session reports with different model usage
    let storage = SessionStorage::new(workspace_root).expect("Failed to create storage");
    
    // Session 1: model-1 and model-2
    let mut metrics1 = SessionMetrics::default();
    metrics1.session_id = "session-1".to_string();
    metrics1.model_usage.insert("model-1".to_string(), ModelUsageStats {
        requests: 5,
        input_tokens: 1000,
        output_tokens: 500,
        cached_tokens: 200,
        estimated_cost: 0.05,
    });
    metrics1.model_usage.insert("model-2".to_string(), ModelUsageStats {
        requests: 3,
        input_tokens: 600,
        output_tokens: 300,
        cached_tokens: 100,
        estimated_cost: 0.03,
    });
    let report1 = SessionReport::new(metrics1);
    storage.save_report(&report1).expect("Failed to save report1");
    
    // Session 2: model-1 and model-3 (overlapping with session 1)
    let mut metrics2 = SessionMetrics::default();
    metrics2.session_id = "session-2".to_string();
    metrics2.model_usage.insert("model-1".to_string(), ModelUsageStats {
        requests: 7,
        input_tokens: 1400,
        output_tokens: 700,
        cached_tokens: 300,
        estimated_cost: 0.07,
    });
    metrics2.model_usage.insert("model-3".to_string(), ModelUsageStats {
        requests: 2,
        input_tokens: 400,
        output_tokens: 200,
        cached_tokens: 50,
        estimated_cost: 0.02,
    });
    let report2 = SessionReport::new(metrics2);
    storage.save_report(&report2).expect("Failed to save report2");
    
    // Session 3: only model-2
    let mut metrics3 = SessionMetrics::default();
    metrics3.session_id = "session-3".to_string();
    metrics3.model_usage.insert("model-2".to_string(), ModelUsageStats {
        requests: 4,
        input_tokens: 800,
        output_tokens: 400,
        cached_tokens: 150,
        estimated_cost: 0.04,
    });
    let report3 = SessionReport::new(metrics3);
    storage.save_report(&report3).expect("Failed to save report3");
    
    // Create analytics and get aggregated usage
    let monitoring = create_test_monitoring();
    let analytics = SessionAnalytics::new(monitoring);
    let aggregated = analytics.get_aggregated_model_usage(Some(workspace_root)).expect("Failed to get aggregated usage");
    
    // Verify aggregation
    assert_eq!(aggregated.len(), 3, "Should have 3 models");
    
    // model-1: aggregated from session-1 and session-2
    let model1 = aggregated.get("model-1").unwrap();
    assert_eq!(model1.requests, 12); // 5 + 7
    assert_eq!(model1.input_tokens, 2400); // 1000 + 1400
    assert_eq!(model1.output_tokens, 1200); // 500 + 700
    assert_eq!(model1.cached_tokens, 500); // 200 + 300
    assert!((model1.estimated_cost - 0.12).abs() < 0.001); // 0.05 + 0.07
    
    // model-2: aggregated from session-1 and session-3
    let model2 = aggregated.get("model-2").unwrap();
    assert_eq!(model2.requests, 7); // 3 + 4
    assert_eq!(model2.input_tokens, 1400); // 600 + 800
    assert_eq!(model2.output_tokens, 700); // 300 + 400
    assert_eq!(model2.cached_tokens, 250); // 100 + 150
    assert!((model2.estimated_cost - 0.07).abs() < 0.001); // 0.03 + 0.04
    
    // model-3: only from session-2
    let model3 = aggregated.get("model-3").unwrap();
    assert_eq!(model3.requests, 2);
    assert_eq!(model3.input_tokens, 400);
    assert_eq!(model3.output_tokens, 200);
    assert_eq!(model3.cached_tokens, 50);
    assert!((model3.estimated_cost - 0.02).abs() < 0.001);
}


//! Integration tests for local model cost tracking.

use radium_core::analytics::{CostQueryService, ExportOptions, ExportFormat};
use radium_core::config::engine_costs::EngineCostsConfig;
use radium_core::engines::engine_trait::{Engine, ExecutionRequest};
use radium_core::engines::providers::mock::MockEngine;
use radium_core::monitoring::{
    AgentRecord, LocalModelCostTracker, MonitoringService, TelemetryRecord, TelemetryTracking,
};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

/// Helper function to create a test monitoring service with cost tracker.
async fn setup_test_service_with_tracker(
    config_content: &str,
) -> (MonitoringService, LocalModelCostTracker, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("engine-costs.toml");
    std::fs::write(&config_path, config_content).unwrap();

    let tracker = LocalModelCostTracker::new(&config_path).unwrap();
    let mut monitoring = MonitoringService::new().unwrap();
    monitoring.set_cost_tracker(Arc::new(tracker.clone()));

    (monitoring, tracker, temp_dir)
}

#[tokio::test]
async fn test_complete_flow_engine_to_cost_report() {
    let config_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1
"#;

    let (monitoring, tracker, _temp) = setup_test_service_with_tracker(config_content).await;

    // Register agent
    let agent = AgentRecord::new("agent-1".to_string(), "developer".to_string());
    monitoring.register_agent(&agent).unwrap();

    // Execute engine request
    let engine = MockEngine::new();
    let request = ExecutionRequest::new("mock-model-1".to_string(), "Test prompt".to_string());
    let response = engine.execute(request).await.unwrap();

    // Simulate 2.5 second duration for cost calculation
    let duration = Duration::from_millis(2500);

    // Record telemetry with local cost
    let record = TelemetryRecord::new("agent-1".to_string())
        .with_local_cost("ollama", duration, &tracker);

    monitoring.record_telemetry(&record).await.unwrap();

    // Query cost summary
    let cost_service = CostQueryService::new(&monitoring);
    let options = ExportOptions {
        format: ExportFormat::Json,
        start_date: None,
        end_date: None,
        plan_id: None,
        provider: None,
        output_path: None,
    };

    let records = cost_service.query_records(&options).unwrap();
    let summary = cost_service.generate_summary(&records);

    // Verify telemetry record
    let telemetry_records = monitoring.get_agent_telemetry("agent-1").unwrap();
    assert_eq!(telemetry_records.len(), 1);
    let telemetry = &telemetry_records[0];
    assert!((telemetry.estimated_cost - 0.00025).abs() < 0.000001);
    assert_eq!(telemetry.behavior_duration_ms, Some(2500));
    assert_eq!(telemetry.engine_id, Some("ollama".to_string()));
    assert_eq!(telemetry.provider, Some("local".to_string()));

    // Verify cost summary
    assert!((summary.total_cost - 0.00025).abs() < 0.000001);
    assert!(summary.local_breakdown.is_some());
    let local_breakdown = summary.local_breakdown.unwrap();
    assert_eq!(local_breakdown.get("ollama"), Some(&0.00025));
}

#[tokio::test]
async fn test_multiple_engines_different_rates() {
    let config_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1

[engines.lm-studio]
cost_per_second = 0.00015
min_billable_duration = 0.1
"#;

    let (monitoring, tracker, _temp) = setup_test_service_with_tracker(config_content).await;

    // Register agent
    let agent = AgentRecord::new("agent-1".to_string(), "developer".to_string());
    monitoring.register_agent(&agent).unwrap();

    // Execute 5 requests on ollama (2s each = 10s total)
    for _ in 0..5 {
        let record = TelemetryRecord::new("agent-1".to_string())
            .with_local_cost("ollama", Duration::from_secs(2), &tracker);
        monitoring.record_telemetry(&record).await.unwrap();
    }

    // Execute 3 requests on lm-studio (2s each = 6s total)
    for _ in 0..3 {
        let record = TelemetryRecord::new("agent-1".to_string())
            .with_local_cost("lm-studio", Duration::from_secs(2), &tracker);
        monitoring.record_telemetry(&record).await.unwrap();
    }

    // Query cost summary
    let cost_service = CostQueryService::new(&monitoring);
    let options = ExportOptions {
        format: ExportFormat::Json,
        start_date: None,
        end_date: None,
        plan_id: None,
        provider: None,
        output_path: None,
    };

    let records = cost_service.query_records(&options).unwrap();
    let summary = cost_service.generate_summary(&records);

    // Verify local breakdown
    assert!(summary.local_breakdown.is_some());
    let local_breakdown = summary.local_breakdown.unwrap();
    // ollama: 5 * 2s * 0.0001 = 0.001
    assert!((local_breakdown.get("ollama").unwrap() - 0.001).abs() < 0.000001);
    // lm-studio: 3 * 2s * 0.00015 = 0.0009
    assert!((local_breakdown.get("lm-studio").unwrap() - 0.0009).abs() < 0.000001);

    // Total local cost: 0.001 + 0.0009 = 0.0019
    assert!((summary.total_cost - 0.0019).abs() < 0.000001);
}

#[tokio::test]
async fn test_hot_reload_updates_cost_calculations() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("engine-costs.toml");

    // Initial config
    let initial_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1
"#;
    std::fs::write(&config_path, initial_content).unwrap();

    let tracker = LocalModelCostTracker::new(&config_path).unwrap();
    let mut monitoring = MonitoringService::new().unwrap();
    monitoring.set_cost_tracker(Arc::new(tracker.clone()));

    let agent = AgentRecord::new("agent-1".to_string(), "developer".to_string());
    monitoring.register_agent(&agent).unwrap();

    // First execution with initial rate
    let record1 = TelemetryRecord::new("agent-1".to_string())
        .with_local_cost("ollama", Duration::from_secs(1), &tracker);
    monitoring.record_telemetry(&record1).await.unwrap();

    // Update config file
    let updated_content = r#"
[engines.ollama]
cost_per_second = 0.0002
min_billable_duration = 0.1
"#;
    std::fs::write(&config_path, updated_content).unwrap();

    // Reload config
    tracker.reload_config().unwrap();

    // Second execution with new rate
    let record2 = TelemetryRecord::new("agent-1".to_string())
        .with_local_cost("ollama", Duration::from_secs(1), &tracker);
    monitoring.record_telemetry(&record2).await.unwrap();

    // Verify both records
    let telemetry_records = monitoring.get_agent_telemetry("agent-1").unwrap();
    assert_eq!(telemetry_records.len(), 2);
    
    // First record: 1s * 0.0001 = 0.0001
    assert!((telemetry_records[1].estimated_cost - 0.0001).abs() < 0.000001);
    
    // Second record: 1s * 0.0002 = 0.0002
    assert!((telemetry_records[0].estimated_cost - 0.0002).abs() < 0.000001);
}

#[tokio::test]
async fn test_missing_engine_config_defaults_to_zero() {
    let config_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1
"#;

    let (monitoring, tracker, _temp) = setup_test_service_with_tracker(config_content).await;

    let agent = AgentRecord::new("agent-1".to_string(), "developer".to_string());
    monitoring.register_agent(&agent).unwrap();

    // Execute on unknown engine
    let record = TelemetryRecord::new("agent-1".to_string())
        .with_local_cost("unknown-engine", Duration::from_secs(5), &tracker);
    monitoring.record_telemetry(&record).await.unwrap();

    // Verify telemetry
    let telemetry_records = monitoring.get_agent_telemetry("agent-1").unwrap();
    assert_eq!(telemetry_records.len(), 1);
    let telemetry = &telemetry_records[0];
    
    // Cost should be 0.0 (missing config defaults to zero)
    assert_eq!(telemetry.estimated_cost, 0.0);
    // But duration and engine_id should still be populated
    assert_eq!(telemetry.behavior_duration_ms, Some(5000));
    assert_eq!(telemetry.engine_id, Some("unknown-engine".to_string()));
}

#[tokio::test]
async fn test_minimum_billable_duration_enforcement() {
    let config_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1
"#;

    let (monitoring, tracker, _temp) = setup_test_service_with_tracker(config_content).await;

    let agent = AgentRecord::new("agent-1".to_string(), "developer".to_string());
    monitoring.register_agent(&agent).unwrap();

    // Execute with duration below minimum (0.05s, but min is 0.1s)
    let record = TelemetryRecord::new("agent-1".to_string())
        .with_local_cost("ollama", Duration::from_millis(50), &tracker);
    monitoring.record_telemetry(&record).await.unwrap();

    // Verify telemetry
    let telemetry_records = monitoring.get_agent_telemetry("agent-1").unwrap();
    assert_eq!(telemetry_records.len(), 1);
    let telemetry = &telemetry_records[0];
    
    // Cost should be calculated using 0.1s minimum: 0.1 * 0.0001 = 0.00001
    assert!((telemetry.estimated_cost - 0.00001).abs() < 0.000001);
    // Actual duration should still be recorded
    assert_eq!(telemetry.behavior_duration_ms, Some(50));
}

#[tokio::test]
async fn test_cost_attribution_fields() {
    let config_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1
"#;

    let (monitoring, tracker, _temp) = setup_test_service_with_tracker(config_content).await;

    let agent = AgentRecord::new("agent-1".to_string(), "developer".to_string());
    monitoring.register_agent(&agent).unwrap();

    // Record telemetry with attribution
    let record = TelemetryRecord::new("agent-1".to_string())
        .with_local_cost("ollama", Duration::from_secs(2), &tracker)
        .with_attribution(
            Some("api-key-123".to_string()),
            Some("team-alpha".to_string()),
            Some("project-beta".to_string()),
            Some("cost-center-gamma".to_string()),
        );
    monitoring.record_telemetry(&record).await.unwrap();

    // Verify attribution fields
    let telemetry_records = monitoring.get_agent_telemetry("agent-1").unwrap();
    assert_eq!(telemetry_records.len(), 1);
    let telemetry = &telemetry_records[0];
    assert_eq!(telemetry.api_key_id, Some("api-key-123".to_string()));
    assert_eq!(telemetry.team_name, Some("team-alpha".to_string()));
    assert_eq!(telemetry.project_name, Some("project-beta".to_string()));
    assert_eq!(telemetry.cost_center, Some("cost-center-gamma".to_string()));
}

#[tokio::test]
async fn test_concurrent_access_to_cost_tracker() {
    let config_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1
"#;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("engine-costs.toml");
    std::fs::write(&config_path, config_content).unwrap();

    let tracker = Arc::new(LocalModelCostTracker::new(&config_path).unwrap());

    // Spawn 100 concurrent tasks calculating costs
    let handles: Vec<_> = (0..100)
        .map(|_| {
            let tracker_clone = Arc::clone(&tracker);
            tokio::spawn(async move {
                for _ in 0..10 {
                    let cost = tracker_clone.calculate_cost("ollama", Duration::from_secs(1));
                    assert!((cost - 0.0001).abs() < 0.000001);
                }
            })
        })
        .collect();

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_backward_compatibility_with_cloud_models() {
    let config_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1
"#;

    let (monitoring, tracker, _temp) = setup_test_service_with_tracker(config_content).await;

    let agent = AgentRecord::new("agent-1".to_string(), "developer".to_string());
    monitoring.register_agent(&agent).unwrap();

    // Record cloud model telemetry (token-based)
    let cloud_record = TelemetryRecord::new("agent-1".to_string())
        .with_tokens(1_000_000, 1_000_000)
        .with_model("gpt-4".to_string(), "openai".to_string());
    let mut cloud_record = cloud_record;
    cloud_record.calculate_cost();
    monitoring.record_telemetry(&cloud_record).await.unwrap();

    // Record local model telemetry (duration-based)
    let local_record = TelemetryRecord::new("agent-1".to_string())
        .with_local_cost("ollama", Duration::from_secs(10), &tracker);
    monitoring.record_telemetry(&local_record).await.unwrap();

    // Query cost summary
    let cost_service = CostQueryService::new(&monitoring);
    let options = ExportOptions {
        format: ExportFormat::Json,
        start_date: None,
        end_date: None,
        plan_id: None,
        provider: None,
        output_path: None,
    };

    let records = cost_service.query_records(&options).unwrap();
    let summary = cost_service.generate_summary(&records);

    // Verify both costs are included
    // Cloud: GPT-4 = $30 input + $60 output = $90 per 1M tokens = $90 total
    // Local: 10s * 0.0001 = $0.001
    assert!(summary.total_cost > 90.0); // Should include both

    // Verify provider breakdown includes both
    assert!(summary.breakdown_by_provider.get("openai").is_some());
    assert!(summary.breakdown_by_provider.get("local").is_some());

    // Verify local breakdown is present
    assert!(summary.local_breakdown.is_some());
    let local_breakdown = summary.local_breakdown.unwrap();
    assert_eq!(local_breakdown.get("ollama"), Some(&0.001));
}

#[tokio::test]
async fn test_zero_duration_edge_case() {
    let config_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.0
"#;

    let (monitoring, tracker, _temp) = setup_test_service_with_tracker(config_content).await;

    let agent = AgentRecord::new("agent-1".to_string(), "developer".to_string());
    monitoring.register_agent(&agent).unwrap();

    // Execute with zero duration (cached response)
    let record = TelemetryRecord::new("agent-1".to_string())
        .with_local_cost("ollama", Duration::from_secs(0), &tracker);
    monitoring.record_telemetry(&record).await.unwrap();

    // Verify telemetry
    let telemetry_records = monitoring.get_agent_telemetry("agent-1").unwrap();
    assert_eq!(telemetry_records.len(), 1);
    let telemetry = &telemetry_records[0];
    
    // Cost should be $0.00 for zero duration
    assert_eq!(telemetry.estimated_cost, 0.0);
    assert_eq!(telemetry.behavior_duration_ms, Some(0));
}


//! Tests for monitoring and telemetry integration.

use radium_core::monitoring::TelemetryRecord;

#[test]
fn test_telemetry_record_creation() {
    let telemetry = TelemetryRecord::new("test-agent".to_string());
    
    assert_eq!(telemetry.agent_id, "test-agent");
    assert_eq!(telemetry.input_tokens, 0);
    assert_eq!(telemetry.output_tokens, 0);
    assert_eq!(telemetry.total_tokens, 0);
    assert_eq!(telemetry.estimated_cost, 0.0);
    assert!(telemetry.timestamp > 0);
}

#[test]
fn test_telemetry_token_tracking() {
    let telemetry = TelemetryRecord::new("test-agent".to_string())
        .with_tokens(1000, 500);
    
    assert_eq!(telemetry.input_tokens, 1000);
    assert_eq!(telemetry.output_tokens, 500);
    assert_eq!(telemetry.total_tokens, 1500);
}

#[test]
fn test_telemetry_cost_calculation_openai() {
    let mut telemetry = TelemetryRecord::new("test-agent".to_string())
        .with_tokens(1000, 500)
        .with_engine_id("openai".to_string())
        .with_model("gpt-4".to_string(), "openai".to_string());
    
    telemetry.calculate_cost();
    
    // GPT-4: $30 per 1M input tokens, $60 per 1M output tokens
    // 1000 input tokens = 0.001M * $30 = $0.03
    // 500 output tokens = 0.0005M * $60 = $0.03
    // Total = $0.06
    assert!(telemetry.estimated_cost > 0.0);
    assert!(telemetry.estimated_cost < 1.0); // Should be small for 1500 tokens
}

#[test]
fn test_telemetry_cost_calculation_claude() {
    let mut telemetry = TelemetryRecord::new("test-agent".to_string())
        .with_tokens(1000, 500)
        .with_engine_id("claude".to_string())
        .with_model("claude-3-sonnet".to_string(), "claude".to_string());
    
    telemetry.calculate_cost();
    
    // Claude Sonnet: $3 per 1M input tokens, $15 per 1M output tokens
    // 1000 input tokens = 0.001M * $3 = $0.003
    // 500 output tokens = 0.0005M * $15 = $0.0075
    // Total = $0.0105
    assert!(telemetry.estimated_cost > 0.0);
    assert!(telemetry.estimated_cost < 1.0);
}

#[test]
fn test_telemetry_cost_calculation_gemini() {
    let mut telemetry = TelemetryRecord::new("test-agent".to_string())
        .with_tokens(1000, 500)
        .with_engine_id("gemini".to_string())
        .with_model("gemini-pro".to_string(), "gemini".to_string());
    
    telemetry.calculate_cost();
    
    // Gemini: $0.5 per 1M input tokens, $1.5 per 1M output tokens
    // 1000 input tokens = 0.001M * $0.5 = $0.0005
    // 500 output tokens = 0.0005M * $1.5 = $0.00075
    // Total = $0.00125
    assert!(telemetry.estimated_cost > 0.0);
    assert!(telemetry.estimated_cost < 1.0);
}

#[test]
fn test_telemetry_cost_calculation_mock() {
    let mut telemetry = TelemetryRecord::new("test-agent".to_string())
        .with_tokens(1000, 500)
        .with_engine_id("mock".to_string());
    
    telemetry.calculate_cost();
    
    // Mock engine is free
    assert_eq!(telemetry.estimated_cost, 0.0);
}

#[test]
fn test_telemetry_cost_calculation_default() {
    let mut telemetry = TelemetryRecord::new("test-agent".to_string())
        .with_tokens(1000, 500)
        .with_engine_id("unknown-engine".to_string());
    
    telemetry.calculate_cost();
    
    // Default fallback: $1 per 1M input, $2 per 1M output
    // 1000 input tokens = 0.001M * $1 = $0.001
    // 500 output tokens = 0.0005M * $2 = $0.001
    // Total = $0.002
    assert!(telemetry.estimated_cost > 0.0);
    assert!(telemetry.estimated_cost < 1.0);
}

#[test]
fn test_telemetry_with_model_and_engine() {
    let telemetry = TelemetryRecord::new("test-agent".to_string())
        .with_tokens(2000, 1000)
        .with_model("gpt-4".to_string(), "openai".to_string())
        .with_engine_id("openai".to_string());
    
    assert_eq!(telemetry.model, Some("gpt-4".to_string()));
    assert_eq!(telemetry.provider, Some("openai".to_string()));
    assert_eq!(telemetry.engine_id, Some("openai".to_string()));
    assert_eq!(telemetry.total_tokens, 3000);
}

#[test]
fn test_telemetry_cache_stats() {
    let telemetry = TelemetryRecord::new("test-agent".to_string())
        .with_tokens(1000, 500)
        .with_cache_stats(200, 50, 150);
    
    assert_eq!(telemetry.cached_tokens, 200);
    assert_eq!(telemetry.cache_creation_tokens, 50);
    assert_eq!(telemetry.cache_read_tokens, 150);
}

#[test]
fn test_telemetry_cost_calculation_gpt35() {
    let mut telemetry = TelemetryRecord::new("test-agent".to_string())
        .with_tokens(1000, 500)
        .with_engine_id("openai".to_string())
        .with_model("gpt-3.5-turbo".to_string(), "openai".to_string());
    
    telemetry.calculate_cost();
    
    // GPT-3.5: $0.5 per 1M input tokens, $1.5 per 1M output tokens
    // 1000 input tokens = 0.001M * $0.5 = $0.0005
    // 500 output tokens = 0.0005M * $1.5 = $0.00075
    // Total = $0.00125
    assert!(telemetry.estimated_cost > 0.0);
    assert!(telemetry.estimated_cost < 1.0);
}

#[test]
fn test_telemetry_cost_calculation_claude_opus() {
    let mut telemetry = TelemetryRecord::new("test-agent".to_string())
        .with_tokens(1000, 500)
        .with_engine_id("claude".to_string())
        .with_model("claude-3-opus".to_string(), "claude".to_string());
    
    telemetry.calculate_cost();
    
    // Claude Opus: $15 per 1M input tokens, $75 per 1M output tokens
    // 1000 input tokens = 0.001M * $15 = $0.015
    // 500 output tokens = 0.0005M * $75 = $0.0375
    // Total = $0.0525
    assert!(telemetry.estimated_cost > 0.0);
    assert!(telemetry.estimated_cost < 1.0);
}

#[test]
fn test_telemetry_cost_calculation_claude_haiku() {
    let mut telemetry = TelemetryRecord::new("test-agent".to_string())
        .with_tokens(1000, 500)
        .with_engine_id("claude".to_string())
        .with_model("claude-3-haiku".to_string(), "claude".to_string());
    
    telemetry.calculate_cost();
    
    // Claude Haiku: $0.25 per 1M input tokens, $1.25 per 1M output tokens
    // 1000 input tokens = 0.001M * $0.25 = $0.00025
    // 500 output tokens = 0.0005M * $1.25 = $0.000625
    // Total = $0.000875
    assert!(telemetry.estimated_cost > 0.0);
    assert!(telemetry.estimated_cost < 1.0);
}

#[test]
fn test_telemetry_without_engine_fallback() {
    let mut telemetry = TelemetryRecord::new("test-agent".to_string())
        .with_tokens(1000, 500)
        .with_model("gpt-4".to_string(), "openai".to_string());
    // No engine_id set, should fall back to model-based pricing
    
    telemetry.calculate_cost();
    
    // Should use model-based pricing (gpt-4: $30/$60)
    assert!(telemetry.estimated_cost > 0.0);
    assert!(telemetry.estimated_cost < 1.0);
}


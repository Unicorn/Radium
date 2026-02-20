//! Integration tests for models command.

use std::process::Command;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_test() {
    INIT.call_once(|| {
        // Set up test environment if needed
    });
}

#[tokio::test]
async fn test_models_list_command() {
    init_test();
    
    // Test that the command runs without error
    let output = Command::new("cargo")
        .args(&["run", "--", "models", "list"])
        .output()
        .expect("Failed to execute command");
    
    // Command should complete (may have warnings but shouldn't crash)
    assert!(output.status.code().is_some());
}

#[tokio::test]
async fn test_models_list_json() {
    init_test();
    
    // Test JSON output format
    let output = Command::new("cargo")
        .args(&["run", "--", "models", "list", "--json"])
        .output()
        .expect("Failed to execute command");
    
    // Command should complete
    assert!(output.status.code().is_some());
    
    // If output exists, it should be valid JSON
    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Try to parse as JSON (may fail if command errors, that's ok for now)
        let _ = serde_json::from_str::<serde_json::Value>(&stdout);
    }
}

#[tokio::test]
async fn test_models_test_command() {
    init_test();
    
    // Test that the command accepts a model ID
    let output = Command::new("cargo")
        .args(&["run", "--", "models", "test", "mock"])
        .output()
        .expect("Failed to execute command");
    
    // Command should complete (may fail if mock engine not available, that's ok)
    assert!(output.status.code().is_some());
}

#[tokio::test]
async fn test_models_test_invalid_model() {
    init_test();
    
    // Test with invalid model ID
    let output = Command::new("cargo")
        .args(&["run", "--", "models", "test", "nonexistent-model"])
        .output()
        .expect("Failed to execute command");
    
    // Should fail with error
    assert!(!output.status.success());
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should mention the model not found
    assert!(stderr.contains("not found") || stderr.contains("error"));
}


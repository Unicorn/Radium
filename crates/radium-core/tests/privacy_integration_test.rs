//! Comprehensive integration tests for privacy mode functionality.

use radium_core::config::Config;
use radium_core::context::ContextManager;
use radium_core::monitoring::logs::LogManager;
use radium_core::monitoring::service::MonitoringService;
use radium_core::security::{PatternLibrary, PrivacyFilter, RedactionStyle};
use radium_core::workspace::Workspace;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_pattern_library_detection() {
    let library = PatternLibrary::default();
    
    // Test IPv4 detection
    let text = "Connect to 192.168.1.100";
    let matches = library.find_matches(text);
    assert!(matches.contains_key("ipv4"));
    assert!(matches["ipv4"].contains(&"192.168.1.100".to_string()));
    
    // Test email detection
    let text = "Contact user@example.com";
    let matches = library.find_matches(text);
    assert!(matches.contains_key("email"));
    assert!(matches["email"].contains(&"user@example.com".to_string()));
    
    // Test credit card with Luhn validation
    let text = "Card: 4532015112830366"; // Valid Luhn
    let matches = library.find_matches(text);
    assert!(matches.contains_key("credit_card"));
    
    // Invalid Luhn should not match
    let text = "Card: 4532015112830367"; // Invalid Luhn
    let matches = library.find_matches(text);
    assert!(!matches.contains_key("credit_card"));
}

#[test]
fn test_privacy_filter_full_redaction() {
    let library = PatternLibrary::default();
    let filter = PrivacyFilter::new(RedactionStyle::Full, library);
    
    let text = "Connect to 192.168.1.100 and email user@example.com";
    let (redacted, stats) = filter.redact(text).unwrap();
    
    assert!(redacted.contains("***"));
    assert!(!redacted.contains("192.168.1.100"));
    assert!(!redacted.contains("user@example.com"));
    assert_eq!(stats.count, 2);
}

#[test]
fn test_privacy_filter_partial_redaction() {
    let library = PatternLibrary::default();
    let filter = PrivacyFilter::new(RedactionStyle::Partial, library);
    
    let text = "Connect to 192.168.1.100";
    let (redacted, stats) = filter.redact(text).unwrap();
    
    // Should show parts of the IP
    assert!(redacted.contains("192"));
    assert!(redacted.contains("100"));
    assert!(redacted.contains("*"));
    assert_eq!(stats.count, 1);
}

#[test]
fn test_privacy_filter_hash_redaction() {
    let library = PatternLibrary::default();
    let filter = PrivacyFilter::new(RedactionStyle::Hash, library);
    
    let text = "API key: sk_live_abc123";
    let (redacted, stats) = filter.redact(text).unwrap();
    
    assert!(redacted.contains("[REDACTED:sha256:"));
    assert!(!redacted.contains("sk_live_abc123"));
    assert_eq!(stats.count, 1);
}

#[test]
fn test_privacy_filter_allowlist() {
    let library = PatternLibrary::default();
    let filter = PrivacyFilter::new(RedactionStyle::Full, library);
    
    filter.add_to_allowlist("192.168.1.1".to_string());
    
    let text = "Connect to 192.168.1.1 and 192.168.1.100";
    let (redacted, stats) = filter.redact(text).unwrap();
    
    // Allowed IP should not be redacted
    assert!(redacted.contains("192.168.1.1"));
    // Other IP should be redacted
    assert!(!redacted.contains("192.168.1.100"));
    assert_eq!(stats.count, 1);
}

#[test]
fn test_context_manager_privacy_integration() {
    let temp_dir = TempDir::new().unwrap();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    
    // Create config with privacy enabled
    let mut config = Config::default();
    config.security.privacy.enable = true;
    config.security.privacy.redaction_style = "full".to_string();
    
    let mut manager = ContextManager::new_with_config(&workspace, Some(&config));
    
    // Create a context file with sensitive data
    let context_file = temp_dir.path().join(".radium").join("GEMINI.md");
    fs::create_dir_all(context_file.parent().unwrap()).unwrap();
    fs::write(&context_file, "Connect to 192.168.1.100 for testing").unwrap();
    
    // Build context
    let context = manager.build_context("test", None).unwrap();
    
    // Context should be redacted
    assert!(context.contains("***"));
    assert!(!context.contains("192.168.1.100"));
}

#[test]
fn test_context_manager_privacy_disabled() {
    let temp_dir = TempDir::new().unwrap();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    
    // Create config with privacy disabled
    let mut config = Config::default();
    config.security.privacy.enable = false;
    
    let mut manager = ContextManager::new_with_config(&workspace, Some(&config));
    
    // Create a context file with sensitive data
    let context_file = temp_dir.path().join(".radium").join("GEMINI.md");
    fs::create_dir_all(context_file.parent().unwrap()).unwrap();
    fs::write(&context_file, "Connect to 192.168.1.100 for testing").unwrap();
    
    // Build context
    let context = manager.build_context("test", None).unwrap();
    
    // Context should NOT be redacted
    assert!(context.contains("192.168.1.100"));
}

#[test]
fn test_log_manager_privacy_integration() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create config with privacy enabled
    let mut config = Config::default();
    config.security.privacy.enable = true;
    config.security.privacy.redaction_style = "full".to_string();
    
    let manager = LogManager::new_with_config(temp_dir.path(), Some(&config)).unwrap();
    
    // Append log with sensitive data
    manager.append_log("agent-1", "Connecting to 192.168.1.100").unwrap();
    
    // Read log back
    let content = manager.read_log("agent-1").unwrap();
    
    // Log should be redacted
    assert!(content.contains("***"));
    assert!(!content.contains("192.168.1.100"));
}

#[test]
fn test_log_manager_ansi_stripping_and_redaction() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create config with privacy enabled
    let mut config = Config::default();
    config.security.privacy.enable = true;
    config.security.privacy.redaction_style = "full".to_string();
    
    let manager = LogManager::new_with_config(temp_dir.path(), Some(&config)).unwrap();
    
    // Append log with ANSI codes and sensitive data
    let colored_line = "\x1B[32mConnecting to 192.168.1.100\x1B[0m";
    manager.append_log("agent-1", colored_line).unwrap();
    
    // Read log back
    let content = manager.read_log("agent-1").unwrap();
    
    // Should have no ANSI codes and be redacted
    assert!(!content.contains("\x1B"));
    assert!(content.contains("***"));
    assert!(!content.contains("192.168.1.100"));
}

#[test]
fn test_monitoring_service_privacy_integration() {
    let service = MonitoringService::new().unwrap();
    
    // Create agent record with sensitive data in log_file
    let mut record = radium_core::monitoring::service::AgentRecord::new(
        "agent-1".to_string(),
        "test".to_string(),
    );
    record.log_file = Some("/path/to/logs/192.168.1.100.log".to_string());
    
    // Register agent (should not redact without privacy enabled)
    service.register_agent(&record).unwrap();
    
    // Get agent back
    let retrieved = service.get_agent("agent-1").unwrap();
    assert_eq!(retrieved.log_file, record.log_file);
}

#[test]
fn test_monitoring_service_error_message_redaction() {
    // Create config with privacy enabled
    let mut config = Config::default();
    config.security.privacy.enable = true;
    config.security.privacy.redaction_style = "full".to_string();
    
    let service = MonitoringService::new_with_config(Some(&config)).unwrap();
    
    // Register agent first
    let record = radium_core::monitoring::service::AgentRecord::new(
        "agent-1".to_string(),
        "test".to_string(),
    );
    service.register_agent(&record).unwrap();
    
    // Fail agent with error message containing sensitive data
    service.fail_agent("agent-1", "Connection failed to 192.168.1.100").unwrap();
    
    // Get agent back
    let retrieved = service.get_agent("agent-1").unwrap();
    
    // Error message should be redacted
    assert!(retrieved.error_message.is_some());
    let error = retrieved.error_message.unwrap();
    assert!(error.contains("***"));
    assert!(!error.contains("192.168.1.100"));
}

#[test]
fn test_false_positive_rate() {
    let library = PatternLibrary::default();
    
    // Test corpus of non-sensitive data that might trigger false positives
    let test_cases = vec![
        "Version 1.2.3.4 released",
        "Phone number format: XXX-XXX-XXXX",
        "Account ID format: 12 digits",
        "Email format: user@domain.com",
        "IP address format: XXX.XXX.XXX.XXX",
    ];
    
    let mut false_positives = 0;
    let mut total_tests = 0;
    
    for test_case in test_cases {
        let matches = library.find_matches(test_case);
        total_tests += 1;
        if !matches.is_empty() {
            false_positives += 1;
        }
    }
    
    // False positive rate should be < 5%
    let false_positive_rate = (false_positives as f64 / total_tests as f64) * 100.0;
    assert!(false_positive_rate < 5.0, "False positive rate: {}%", false_positive_rate);
}

#[test]
fn test_thread_safety() {
    use std::sync::Arc;
    use std::thread;
    
    let library = PatternLibrary::default();
    let filter = Arc::new(PrivacyFilter::new(RedactionStyle::Full, library));
    
    let mut handles = vec![];
    for i in 0..10 {
        let filter_clone = Arc::clone(&filter);
        let handle = thread::spawn(move || {
            let text = format!("Connect to 192.168.1.{}", i);
            let (redacted, stats) = filter_clone.redact(&text).unwrap();
            assert!(redacted.contains("***"));
            assert_eq!(stats.count, 1);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_multiple_pattern_types() {
    let library = PatternLibrary::default();
    let filter = PrivacyFilter::new(RedactionStyle::Full, library);
    
    let text = "Contact user@example.com at 192.168.1.100 or call 555-123-4567. AWS account: 123456789012";
    let (redacted, stats) = filter.redact(text).unwrap();
    
    // Should detect multiple pattern types
    assert!(stats.patterns.len() >= 3); // email, ipv4, phone, aws_account_id
    assert!(stats.count >= 4);
    
    // All should be redacted
    assert!(!redacted.contains("user@example.com"));
    assert!(!redacted.contains("192.168.1.100"));
    assert!(!redacted.contains("555-123-4567"));
    assert!(!redacted.contains("123456789012"));
}


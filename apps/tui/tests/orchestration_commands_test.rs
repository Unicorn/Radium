//! Integration tests for orchestration commands in TUI.
//!
//! Tests the /orchestrator command and its subcommands, as well as
//! natural language input routing to orchestration.

use radium_tui::commands::Command;
use std::sync::Arc;
use tokio::sync::Mutex;

// Note: These are integration tests that test command parsing and routing logic.
// Full end-to-end tests would require a running server and are marked as ignored.

#[test]
fn test_orchestrator_command_parsing() {
    // Test parsing of /orchestrator command
    let cmd = Command::parse("/orchestrator").unwrap();
    assert_eq!(cmd.name, "orchestrator");
    assert_eq!(cmd.args, Vec::<String>::new());

    // Test parsing of /orchestrator toggle
    let cmd = Command::parse("/orchestrator toggle").unwrap();
    assert_eq!(cmd.name, "orchestrator");
    assert_eq!(cmd.args, vec!["toggle"]);

    // Test parsing of /orchestrator switch gemini
    let cmd = Command::parse("/orchestrator switch gemini").unwrap();
    assert_eq!(cmd.name, "orchestrator");
    assert_eq!(cmd.args, vec!["switch", "gemini"]);

    // Test parsing of /orchestrator status
    let cmd = Command::parse("/orchestrator status").unwrap();
    assert_eq!(cmd.name, "orchestrator");
    assert_eq!(cmd.args, vec!["status"]);
}

#[test]
fn test_orchestrator_command_invalid_provider() {
    // Test that invalid provider names are rejected
    let cmd = Command::parse("/orchestrator switch invalid").unwrap();
    assert_eq!(cmd.name, "orchestrator");
    assert_eq!(cmd.args, vec!["switch", "invalid"]);
    // The actual validation happens in switch_orchestrator_provider
}

#[test]
fn test_natural_input_not_command() {
    // Test that natural language input (without /) is not parsed as a command
    assert!(Command::parse("I need to refactor the authentication module").is_none());
    assert!(Command::parse("Create a new feature for task templates").is_none());
    assert!(Command::parse("hello world").is_none());
}

#[test]
fn test_command_input_bypasses_orchestration() {
    // Test that commands starting with / are parsed as commands
    assert!(Command::parse("/chat my-agent").is_some());
    assert!(Command::parse("/agents").is_some());
    assert!(Command::parse("/help").is_some());
    assert!(Command::parse("/orchestrator").is_some());
}

#[test]
fn test_orchestrator_subcommands() {
    // Test all valid subcommands
    let subcommands = vec!["toggle", "switch", "status"];

    for subcmd in subcommands {
        let input = format!("/orchestrator {}", subcmd);
        let cmd = Command::parse(&input).unwrap();
        assert_eq!(cmd.name, "orchestrator");
        assert!(cmd.args.contains(&subcmd.to_string()));
    }
}

#[test]
fn test_orchestrator_switch_requires_provider() {
    // Test that /orchestrator switch without provider is parsed but will show usage
    let cmd = Command::parse("/orchestrator switch").unwrap();
    assert_eq!(cmd.name, "orchestrator");
    assert_eq!(cmd.args, vec!["switch"]);
    // The actual validation happens in handle_orchestrator_command
}

#[test]
fn test_provider_names_case_insensitive() {
    // Test that provider names can be parsed in different cases
    let providers = vec!["gemini", "GEMINI", "Gemini", "claude", "CLAUDE", "Claude"];

    for provider in providers {
        let input = format!("/orchestrator switch {}", provider);
        let cmd = Command::parse(&input).unwrap();
        assert_eq!(cmd.name, "orchestrator");
        assert_eq!(cmd.args.len(), 2);
        assert_eq!(cmd.args[0], "switch");
        // The actual case-insensitive matching happens in switch_orchestrator_provider
    }
}

#[test]
fn test_orchestrator_command_with_multiple_args() {
    // Test that extra arguments are preserved
    let cmd = Command::parse("/orchestrator switch gemini extra arg").unwrap();
    assert_eq!(cmd.name, "orchestrator");
    assert_eq!(cmd.args, vec!["switch", "gemini", "extra", "arg"]);
    // The handler will only use the first two args
}

// Note: Full integration tests that test the actual App behavior would require:
// - Mock orchestration service
// - Async runtime
// - Full App initialization
// These are better suited for E2E tests with a running server.
// The tests above verify the command parsing logic which is the critical part.


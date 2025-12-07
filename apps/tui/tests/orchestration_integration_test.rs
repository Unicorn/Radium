//! Integration tests for orchestration functionality in TUI.
//!
//! Tests orchestration integration, error handling, and configuration persistence.

use radium_tui::commands::Command;

#[test]
fn test_orchestration_routing_logic() {
    // Test that natural language input routes to orchestration
    // (not parsed as command)
    let natural_inputs = vec![
        "I need help with refactoring",
        "Create a new feature",
        "What agents can help me?",
    ];

    for input in natural_inputs {
        let cmd = Command::parse(input);
        assert!(
            cmd.is_none(),
            "Natural input '{}' should route to orchestration, not be parsed as command",
            input
        );
    }
}

#[test]
fn test_orchestrator_config_commands() {
    // Test all configuration-related commands
    let config_commands = vec![
        "/orchestrator",
        "/orchestrator status",
        "/orchestrator config",
        "/orchestrator toggle",
        "/orchestrator refresh",
    ];

    for cmd_str in config_commands {
        let cmd = Command::parse(cmd_str);
        assert!(
            cmd.is_some(),
            "Config command '{}' should be parsed",
            cmd_str
        );
        if let Some(cmd) = cmd {
            assert_eq!(cmd.name, "orchestrator");
        }
    }
}

#[test]
fn test_orchestrator_provider_switching() {
    // Test provider switching commands
    let providers = vec!["gemini", "claude", "openai", "prompt-based", "prompt_based"];

    for provider in providers {
        let cmd_str = format!("/orchestrator switch {}", provider);
        let cmd = Command::parse(&cmd_str);
        assert!(
            cmd.is_some(),
            "Provider switch command '{}' should be parsed",
            cmd_str
        );
        if let Some(cmd) = cmd {
            assert_eq!(cmd.name, "orchestrator");
            assert_eq!(cmd.args[0], "switch");
            assert_eq!(cmd.args[1], provider);
        }
    }
}

#[test]
fn test_orchestrator_error_handling() {
    // Test error cases in command parsing
    // Invalid provider should still parse (validation happens in handler)
    let invalid_cmd = Command::parse("/orchestrator switch invalid_provider");
    assert!(invalid_cmd.is_some()); // Parses successfully
    if let Some(cmd) = invalid_cmd {
        assert_eq!(cmd.name, "orchestrator");
        assert_eq!(cmd.args[0], "switch");
    }

    // Missing provider for switch should parse but be invalid
    let missing_provider = Command::parse("/orchestrator switch");
    assert!(missing_provider.is_some()); // Parses, validation in handler
}

#[test]
fn test_orchestrator_command_completeness() {
    // Test that all documented orchestrator commands are parseable
    let documented_commands = vec![
        "/orchestrator",
        "/orchestrator status",
        "/orchestrator toggle",
        "/orchestrator switch gemini",
        "/orchestrator switch claude",
        "/orchestrator switch openai",
        "/orchestrator switch prompt-based",
        "/orchestrator config",
        "/orchestrator refresh",
    ];

    for cmd_str in documented_commands {
        let cmd = Command::parse(cmd_str);
        assert!(
            cmd.is_some(),
            "Documented command '{}' should be parseable",
            cmd_str
        );
        if let Some(cmd) = cmd {
            assert_eq!(cmd.name, "orchestrator");
        }
    }
}

#[test]
fn test_natural_input_vs_command_distinction() {
    // Test clear distinction - commands must start with /
    let commands = vec![
        "/orchestrator",
        "/chat agent",
        "/help",
    ];

    let natural = vec![
        "orchestrator status", // Missing /
        "chat with agent",
        "help me",
    ];

    for cmd_str in commands {
        assert!(
            Command::parse(cmd_str).is_some(),
            "Command '{}' should be parsed",
            cmd_str
        );
    }

    for natural_str in natural {
        assert!(
            Command::parse(natural_str).is_none(),
            "Natural input '{}' should NOT be parsed as command",
            natural_str
        );
    }
}

#[test]
fn test_orchestrator_subcommand_validation() {
    // Test that subcommands are properly identified
    let test_cases = vec![
        ("/orchestrator", None),
        ("/orchestrator status", Some("status")),
        ("/orchestrator toggle", Some("toggle")),
        ("/orchestrator switch", Some("switch")),
        ("/orchestrator config", Some("config")),
        ("/orchestrator refresh", Some("refresh")),
    ];

    for (input, expected_subcmd) in test_cases {
        let cmd = Command::parse(input).unwrap();
        assert_eq!(cmd.name, "orchestrator");
        if let Some(expected) = expected_subcmd {
            assert!(
                cmd.args.contains(&expected.to_string()),
                "Command '{}' should have subcommand '{}'",
                input,
                expected
            );
        }
    }
}


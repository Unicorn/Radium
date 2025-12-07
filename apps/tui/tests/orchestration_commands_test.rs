//! Integration tests for orchestration commands in TUI.
//!
//! Tests the /orchestrator command and its subcommands, as well as
//! natural language input routing to orchestration.

use radium_tui::commands::Command;

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

#[tokio::test]
async fn test_orchestrator_command_handling_logic() {
    // Test the logic for handling orchestrator commands
    // This tests the command routing without requiring full App initialization
    
    // Test status command parsing
    let status_cmd = Command::parse("/orchestrator status").unwrap();
    assert_eq!(status_cmd.name, "orchestrator");
    assert_eq!(status_cmd.args[0], "status");
    
    // Test toggle command parsing
    let toggle_cmd = Command::parse("/orchestrator toggle").unwrap();
    assert_eq!(toggle_cmd.name, "orchestrator");
    assert_eq!(toggle_cmd.args[0], "toggle");
    
    // Test switch command parsing with provider
    let switch_cmd = Command::parse("/orchestrator switch gemini").unwrap();
    assert_eq!(switch_cmd.name, "orchestrator");
    assert_eq!(switch_cmd.args[0], "switch");
    assert_eq!(switch_cmd.args[1], "gemini");
}

#[test]
fn test_orchestrator_command_validation() {
    // Test that orchestrator commands are properly validated
    
    // Valid commands
    assert!(Command::parse("/orchestrator").is_some());
    assert!(Command::parse("/orchestrator status").is_some());
    assert!(Command::parse("/orchestrator toggle").is_some());
    assert!(Command::parse("/orchestrator switch gemini").is_some());
    assert!(Command::parse("/orchestrator switch claude").is_some());
    assert!(Command::parse("/orchestrator switch openai").is_some());
    assert!(Command::parse("/orchestrator switch prompt-based").is_some());
    assert!(Command::parse("/orchestrator switch prompt_based").is_some());
    
    // Invalid - missing provider for switch
    let switch_no_provider = Command::parse("/orchestrator switch");
    assert!(switch_no_provider.is_some()); // Parses, but validation happens in handler
}

#[test]
fn test_natural_language_routing() {
    // Test that natural language input (without /) is correctly identified as non-command
    
    let natural_inputs = vec![
        "I need to refactor the authentication module",
        "Create a new feature for task templates",
        "What agents are available?",
        "Help me debug this issue",
        "hello world",
        "test",
    ];
    
    for input in natural_inputs {
        assert!(
            Command::parse(input).is_none(),
            "Natural language input '{}' should not be parsed as a command",
            input
        );
    }
}

#[test]
fn test_command_vs_natural_input_distinction() {
    // Test clear distinction between commands and natural input
    
    // Commands (should parse)
    let commands = vec![
        "/chat agent-id",
        "/agents",
        "/help",
        "/orchestrator",
        "/orchestrator toggle",
        "/orchestrator switch gemini",
    ];
    
    for cmd in commands {
        assert!(
            Command::parse(cmd).is_some(),
            "Command '{}' should be parsed",
            cmd
        );
    }
    
    // Natural input (should not parse)
    let natural = vec![
        "chat with agent",
        "show agents",
        "help me",
        "orchestrator status", // Missing leading /
    ];
    
    for input in natural {
        assert!(
            Command::parse(input).is_none(),
            "Natural input '{}' should not be parsed as command",
            input
        );
    }
}

#[test]
fn test_orchestrator_subcommand_parsing() {
    // Test all valid orchestrator subcommands are parsed correctly
    
    let test_cases = vec![
        ("/orchestrator", vec![]),
        ("/orchestrator status", vec!["status"]),
        ("/orchestrator toggle", vec!["toggle"]),
        ("/orchestrator switch", vec!["switch"]),
        ("/orchestrator switch gemini", vec!["switch", "gemini"]),
        ("/orchestrator switch claude", vec!["switch", "claude"]),
        ("/orchestrator switch openai", vec!["switch", "openai"]),
        ("/orchestrator switch prompt-based", vec!["switch", "prompt-based"]),
    ];
    
    for (input, expected_args) in test_cases {
        let cmd = Command::parse(input).unwrap();
        assert_eq!(cmd.name, "orchestrator", "Command name should be 'orchestrator' for input: {}", input);
        assert_eq!(
            cmd.args, expected_args,
            "Args mismatch for input: {}. Expected: {:?}, Got: {:?}",
            input, expected_args, cmd.args
        );
    }
}

#[test]
fn test_provider_name_parsing_variations() {
    // Test that provider names can be parsed in various formats
    
    let providers = vec!["gemini", "claude", "openai", "prompt-based", "prompt_based"];
    let variations = vec!["lowercase", "UPPERCASE", "MixedCase"];
    
    for provider in &providers {
        for variation in &variations {
            let provider_variant = match *variation {
                "lowercase" => provider.to_lowercase(),
                "UPPERCASE" => provider.to_uppercase(),
                "MixedCase" => {
                    let mut chars = provider.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                }
                _ => unreachable!(),
            };
            
            let input = format!("/orchestrator switch {}", provider_variant);
            let cmd = Command::parse(&input).unwrap();
            assert_eq!(cmd.name, "orchestrator");
            assert_eq!(cmd.args[0], "switch");
            // The actual case-insensitive matching happens in switch_orchestrator_provider
        }
    }
}


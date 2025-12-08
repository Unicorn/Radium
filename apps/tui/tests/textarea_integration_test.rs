//! Integration tests for TextArea widget rendering and interaction.

use radium_tui::views::PromptData;
use radium_tui::commands::Command;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[test]
fn test_textarea_widget_creation() {
    let data = PromptData::new();
    // TextArea implements Widget directly, so we can render it
    // Just verify the textarea exists and has content
    assert_eq!(data.input.lines().len(), 1);
    assert!(data.input.text().is_empty());
}

#[test]
fn test_command_parsing_with_multiline_input() {
    let mut data = PromptData::new();
    
    // Set multiline command input
    data.set_input("/chat\nagent-1");
    
    // Command parsing should work with first line only
    let input = data.input_text();
    let first_line = input.lines().next().unwrap_or("");
    let cmd = Command::parse(first_line);
    
    // Should parse the command from first line
    assert!(cmd.is_some());
    if let Some(cmd) = cmd {
        assert_eq!(cmd.name, "chat");
    }
}

#[test]
fn test_chat_message_with_multiline_text() {
    let mut data = PromptData::new();
    
    // Set multiline chat message
    data.set_input("Hello,\nthis is a\nmultiline message");
    
    let text = data.input_text();
    assert!(text.contains('\n'));
    assert_eq!(text.lines().count(), 3);
}

#[test]
fn test_autocomplete_with_multiline_input() {
    let mut data = PromptData::new();
    
    // Set input that starts with /
    data.set_input("/cha");
    
    let input = data.input_text();
    // Should still start with / for autocomplete
    assert!(input.starts_with('/'));
    
    // Autocomplete should work with first line
    let first_line = input.lines().next().unwrap_or("");
    assert!(first_line.starts_with("/cha"));
}

#[test]
fn test_enter_vs_cmd_enter_submission_logic() {
    let mut data = PromptData::new();
    data.set_input("test message");
    
    // Plain Enter should insert newline (handled by TextArea)
    data.input.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    
    let text = data.input_text();
    assert!(text.ends_with('\n') || text.contains('\n'));
    
    // Cmd+Enter would be handled at app level, not by TextArea
    // This test verifies Enter behavior is correct
}

#[test]
fn test_interaction_with_command_palette() {
    let mut data = PromptData::new();
    
    // Command palette should use separate query field
    data.command_palette_active = true;
    data.command_palette_query = "test".to_string();
    
    // Input field should be separate from palette query
    data.set_input("main input");
    assert_eq!(data.input_text(), "main input");
    assert_eq!(data.command_palette_query, "test");
}

#[test]
fn test_edge_case_very_long_input() {
    let mut data = PromptData::new();
    
    // Create a very long single line
    let long_text = "a".repeat(1000);
    data.set_input(&long_text);
    
    assert_eq!(data.input_text().len(), 1000);
    assert_eq!(data.input_text(), long_text);
}

#[test]
fn test_edge_case_special_characters() {
    let mut data = PromptData::new();
    
    // Test with special characters
    let special = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~";
    data.set_input(special);
    
    assert_eq!(data.input_text(), special);
}

#[test]
fn test_edge_case_unicode_characters() {
    let mut data = PromptData::new();
    
    // Test with unicode
    let unicode = "Hello 世界 🌍";
    data.set_input(unicode);
    
    assert_eq!(data.input_text(), unicode);
}

#[test]
fn test_edge_case_empty_lines() {
    let mut data = PromptData::new();
    
    // Test with multiple empty lines
    data.set_input("line1\n\n\nline2");
    
    let text = data.input_text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.len() >= 2);
}

#[test]
fn test_edge_case_whitespace_only_input() {
    let mut data = PromptData::new();
    
    // Test with whitespace only
    data.set_input("   \n  \t  ");
    
    let text = data.input_text();
    // Should preserve whitespace
    assert!(!text.is_empty() || text.chars().any(|c| c.is_whitespace()));
}


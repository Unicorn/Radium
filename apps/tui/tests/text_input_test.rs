//! Unit tests for TextArea input handling and PromptData integration.

use radium_tui::views::PromptData;
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn test_textarea_initialization() {
    let data = PromptData::new();
    assert_eq!(data.input_text(), "");
    assert_eq!(data.input.lines().len(), 1); // Default has one empty line
}

#[test]
fn test_textarea_text_insertion() {
    let mut data = PromptData::new();
    
    // Insert characters
    for c in "hello".chars() {
        data.input.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    
    assert_eq!(data.input_text(), "hello");
}

#[test]
fn test_textarea_text_deletion() {
    let mut data = PromptData::new();
    
    // Insert text
    data.set_input("test");
    assert_eq!(data.input_text(), "test");
    
    // Delete with backspace
    data.input.handle_key(KeyCode::Backspace, KeyModifiers::NONE);
    
    assert_eq!(data.input_text(), "tes");
}

#[test]
fn test_textarea_multiline_handling() {
    let mut data = PromptData::new();
    
    // Type first line
    for c in "line1".chars() {
        data.input.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    
    // Press Enter to create newline
    data.input.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    
    // Type second line
    for c in "line2".chars() {
        data.input.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    
    assert_eq!(data.input_text(), "line1\nline2");
}

#[test]
fn test_textarea_cursor_movement() {
    let mut data = PromptData::new();
    data.set_input("hello");
    
    // Move cursor left
    data.input.handle_key(KeyCode::Left, KeyModifiers::NONE);
    
    // Insert character at cursor position
    data.input.handle_key(KeyCode::Char('x'), KeyModifiers::NONE);
    
    // Should insert 'x' before the last 'o'
    let text = data.input_text();
    assert!(text.contains('x'));
}

#[test]
fn test_enter_vs_cmd_enter_behavior() {
    let mut data = PromptData::new();
    data.set_input("test");
    
    // Plain Enter should insert newline
    data.input.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    
    assert_eq!(data.input_text(), "test\n");
    
    // Cmd+Enter would be handled at a higher level (in handle_key)
    // This test just verifies Enter inserts newline
}

#[test]
fn test_input_clearing() {
    let mut data = PromptData::new();
    data.set_input("some text");
    assert_eq!(data.input_text(), "some text");
    
    data.clear_input();
    assert_eq!(data.input_text(), "");
}

#[test]
fn test_text_retrieval_methods() {
    let mut data = PromptData::new();
    data.set_input("multiline\ntext\ninput");
    
    let text = data.input_text();
    assert_eq!(text, "multiline\ntext\ninput");
    
    // Verify lines are preserved
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "multiline");
    assert_eq!(lines[1], "text");
    assert_eq!(lines[2], "input");
}

#[test]
fn test_set_input_with_empty_string() {
    let mut data = PromptData::new();
    data.set_input("initial");
    data.set_input("");
    assert_eq!(data.input_text(), "");
}

#[test]
fn test_set_input_preserves_multiline() {
    let mut data = PromptData::new();
    let multiline = "line1\nline2\nline3";
    data.set_input(multiline);
    assert_eq!(data.input_text(), multiline);
}


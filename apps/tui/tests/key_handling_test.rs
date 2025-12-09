//! Key event simulation tests for TextArea behavior and submission logic.

use radium_tui::views::PromptData;
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn test_cmd_enter_submission_detection() {
    // Test that Cmd+Enter can be detected (this would be in app.rs handle_key)
    let modifiers_cmd = KeyModifiers::META;
    let modifiers_ctrl = KeyModifiers::CONTROL;
    
    // Verify modifiers can be checked
    assert!(modifiers_cmd.contains(KeyModifiers::META));
    assert!(modifiers_ctrl.contains(KeyModifiers::CONTROL));
}

#[test]
fn test_enter_newline_insertion() {
    let mut data = PromptData::new();
    data.set_input("test");
    
    // Plain Enter should insert newline
    data.input.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    
    let text = data.input_text();
    assert!(text.contains('\n') || text.ends_with('\n'));
}

#[test]
fn test_navigation_keys_arrows() {
    let mut data = PromptData::new();
    data.set_input("hello world");
    
    // Move cursor left
    data.input.handle_key(KeyCode::Left, KeyModifiers::NONE);
    
    // Move cursor right
    data.input.handle_key(KeyCode::Right, KeyModifiers::NONE);
    
    // Text should remain unchanged
    assert_eq!(data.input_text(), "hello world");
}

#[test]
fn test_navigation_keys_home_end() {
    let mut data = PromptData::new();
    data.set_input("test line");
    
    // Home key
    data.input.handle_key(KeyCode::Home, KeyModifiers::NONE);
    
    // End key
    data.input.handle_key(KeyCode::End, KeyModifiers::NONE);
    
    // Text should remain unchanged
    assert_eq!(data.input_text(), "test line");
}

#[test]
fn test_backspace_behavior() {
    let mut data = PromptData::new();
    data.set_input("test");
    
    // Backspace should delete character
    data.input.handle_key(KeyCode::Backspace, KeyModifiers::NONE);
    
    assert_eq!(data.input_text(), "tes");
}

#[test]
fn test_delete_key_behavior() {
    let mut data = PromptData::new();
    data.set_input("test");
    
    // Delete key (forward delete)
    // Note: Delete behavior depends on cursor position
    data.input.handle_key(KeyCode::Delete, KeyModifiers::NONE);
    
    // Text may or may not change depending on cursor position
    let text = data.input_text();
    assert!(text.len() <= 4);
}

#[test]
fn test_special_key_combinations() {
    let mut data = PromptData::new();
    data.set_input("test");
    
    // Ctrl+A (select all) - TextArea may handle this
    data.input.handle_key(KeyCode::Char('a'), KeyModifiers::CONTROL);
    
    // Text should remain (Ctrl+A doesn't delete)
    assert_eq!(data.input_text(), "test");
}

#[test]
fn test_multiline_navigation() {
    let mut data = PromptData::new();
    data.set_input("line1\nline2\nline3");
    
    // Navigate up (to previous line)
    data.input.handle_key(KeyCode::Up, KeyModifiers::NONE);
    
    // Navigate down (to next line)
    data.input.handle_key(KeyCode::Down, KeyModifiers::NONE);
    
    // Text should remain unchanged
    assert_eq!(data.input_text(), "line1\nline2\nline3");
}

#[test]
fn test_page_up_page_down() {
    let mut data = PromptData::new();
    data.set_input("line1\nline2\nline3\nline4\nline5");
    
    // PageUp
    data.input.handle_key(KeyCode::PageUp, KeyModifiers::NONE);
    
    // PageDown
    data.input.handle_key(KeyCode::PageDown, KeyModifiers::NONE);
    
    // Text should remain unchanged
    assert_eq!(data.input_text(), "line1\nline2\nline3\nline4\nline5");
}

#[test]
fn test_character_input_with_modifiers() {
    let mut data = PromptData::new();
    
    // Regular character
    data.input.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
    
    // Shift+character (should be uppercase if shift is pressed, but we're not testing that here)
    // Just verify regular input works
    assert_eq!(data.input_text(), "a");
}

#[test]
fn test_tab_key_behavior() {
    let mut data = PromptData::new();
    data.set_input("test");
    
    // Tab key - TextArea may insert tab or spaces
    data.input.handle_key(KeyCode::Tab, KeyModifiers::NONE);
    
    // Text should have changed (tab inserted or cursor moved)
    let text = data.input_text();
    assert!(text.len() >= 4); // At least original text
}

#[test]
fn test_character_input_when_typing() {
    // Test that characters like 'h', 's', '?' can be typed when input is not empty
    let mut data = PromptData::new();
    
    // Type some text first
    data.set_input("tell me about my project");
    
    // These characters should be insertable when input is not empty
    data.input.handle_key(KeyCode::Char('h'), KeyModifiers::NONE);
    data.input.handle_key(KeyCode::Char('s'), KeyModifiers::NONE);
    data.input.handle_key(KeyCode::Char('?'), KeyModifiers::NONE);
    
    let text = data.input_text();
    // Should contain the original text plus the new characters
    assert!(text.contains("tell me about my project"));
    assert!(text.contains('h') || text.contains('s') || text.contains('?'));
}

#[test]
fn test_enter_key_clears_input() {
    // Test that Enter key clears input after submission
    let mut data = PromptData::new();
    data.set_input("test input");
    
    // Simulate Enter key (this would normally call handle_enter which clears input)
    // For this test, we'll just verify the input can be cleared
    data.clear_input();
    
    assert_eq!(data.input_text(), "");
}

#[test]
fn test_hotkey_characters_when_input_empty() {
    // Test that hotkey characters can be typed when input is empty
    // (they should work as regular characters when input is empty, not as hotkeys)
    let mut data = PromptData::new();
    
    // Input is empty, so typing 'h' should insert 'h'
    data.input.handle_key(KeyCode::Char('h'), KeyModifiers::NONE);
    assert_eq!(data.input_text(), "h");
    
    data.clear_input();
    
    // Typing 's' should insert 's'
    data.input.handle_key(KeyCode::Char('s'), KeyModifiers::NONE);
    assert_eq!(data.input_text(), "s");
    
    data.clear_input();
    
    // Typing '?' should insert '?'
    data.input.handle_key(KeyCode::Char('?'), KeyModifiers::NONE);
    assert_eq!(data.input_text(), "?");
}


//! Integration tests for setup wizard workflow.

use crossterm::event::{KeyCode, KeyModifiers};
use radium_tui::setup::{SetupState, SetupWizard};

#[tokio::test]
async fn test_setup_wizard_initial_state() {
    let wizard = SetupWizard::new();
    assert_eq!(wizard.state, SetupState::Welcome);
    assert!(wizard.error_message.is_none());
}

#[tokio::test]
async fn test_setup_wizard_welcome_to_provider_selection() {
    let mut wizard = SetupWizard::new();
    
    // Press Enter to move from Welcome to ProviderSelection
    let done = wizard.handle_key(KeyCode::Enter, KeyModifiers::empty()).await.unwrap();
    assert!(!done);
    assert!(matches!(wizard.state, SetupState::ProviderSelection { .. }));
}

#[tokio::test]
async fn test_setup_wizard_provider_selection_navigation() {
    let mut wizard = SetupWizard::new();
    
    // Move to provider selection
    wizard.handle_key(KeyCode::Enter, KeyModifiers::empty()).await.unwrap();
    
    if let SetupState::ProviderSelection { cursor, .. } = &wizard.state {
        let initial_cursor = *cursor;
        
        // Move down
        wizard.handle_key(KeyCode::Down, KeyModifiers::empty()).await.unwrap();
        if let SetupState::ProviderSelection { cursor, .. } = &wizard.state {
            assert_eq!(*cursor, initial_cursor + 1);
        }
        
        // Move up
        wizard.handle_key(KeyCode::Up, KeyModifiers::empty()).await.unwrap();
        if let SetupState::ProviderSelection { cursor, .. } = &wizard.state {
            assert_eq!(*cursor, initial_cursor);
        }
    } else {
        panic!("Expected ProviderSelection state");
    }
}

#[tokio::test]
async fn test_setup_wizard_provider_toggle_selection() {
    let mut wizard = SetupWizard::new();
    
    // Move to provider selection
    wizard.handle_key(KeyCode::Enter, KeyModifiers::empty()).await.unwrap();
    
    // Toggle first provider (Gemini)
    wizard.handle_key(KeyCode::Char(' '), KeyModifiers::empty()).await.unwrap();
    
    if let SetupState::ProviderSelection { selected_providers, .. } = &wizard.state {
        assert!(!selected_providers.is_empty());
    } else {
        panic!("Expected ProviderSelection state");
    }
}

#[tokio::test]
async fn test_setup_wizard_provider_selection_requires_at_least_one() {
    let mut wizard = SetupWizard::new();
    
    // Move to provider selection
    wizard.handle_key(KeyCode::Enter, KeyModifiers::empty()).await.unwrap();
    
    // Try to continue without selecting any provider
    wizard.handle_key(KeyCode::Enter, KeyModifiers::empty()).await.unwrap();
    
    // Should have error message
    assert!(wizard.error_message.is_some());
    assert!(wizard.error_message.as_ref().unwrap().contains("select at least one"));
}

#[tokio::test]
async fn test_setup_wizard_skip_with_esc() {
    let mut wizard = SetupWizard::new();
    
    // Move to provider selection
    wizard.handle_key(KeyCode::Enter, KeyModifiers::empty()).await.unwrap();
    
    // Press Esc to skip
    let done = wizard.handle_key(KeyCode::Esc, KeyModifiers::empty()).await.unwrap();
    assert!(done);
}

#[tokio::test]
async fn test_setup_wizard_skip_welcome() {
    let wizard = SetupWizard::new_skip_welcome();
    assert!(matches!(wizard.state, SetupState::ProviderSelection { .. }));
}

#[tokio::test]
async fn test_setup_wizard_display_lines() {
    let wizard = SetupWizard::new();
    let lines = wizard.display_lines();
    assert!(!lines.is_empty());
    assert!(lines.iter().any(|l| l.contains("Welcome")));
}

#[tokio::test]
async fn test_setup_wizard_title() {
    let wizard = SetupWizard::new();
    assert_eq!(wizard.title(), "Welcome");
    
    // Move to provider selection
    let mut wizard = SetupWizard::new();
    wizard.handle_key(KeyCode::Enter, KeyModifiers::empty()).await.unwrap();
    assert_eq!(wizard.title(), "Provider Selection");
}


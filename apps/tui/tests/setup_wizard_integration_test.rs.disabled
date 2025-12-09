//! Integration tests for setup wizard validation flow.
//!
//! Tests the complete authentication flow including credential validation,
//! error handling, and multi-provider sequential flow.

use crossterm::event::{KeyCode, KeyModifiers};
use radium_core::auth::{CredentialStore, ProviderType};
use radium_tui::setup::{ProviderStatus, SetupState, SetupWizard, ValidationProgress};
use tempfile::TempDir;

#[tokio::test]
async fn test_provider_selection_status_indicators() {
    let temp_dir = TempDir::new().unwrap();
    let creds_path = temp_dir.path().join("credentials.json");
    let store = CredentialStore::with_path(creds_path);
    
    // Configure Gemini
    store.store(ProviderType::Gemini, "test-key".to_string()).unwrap();
    
    let mut wizard = SetupWizard::new_skip_welcome();
    
    if let SetupState::ProviderSelection { provider_status, .. } = &wizard.state {
        assert_eq!(
            provider_status.get(&ProviderType::Gemini),
            Some(&ProviderStatus::Configured)
        );
        assert_eq!(
            provider_status.get(&ProviderType::OpenAI),
            Some(&ProviderStatus::NotConfigured)
        );
    } else {
        panic!("Expected ProviderSelection state");
    }
}

#[tokio::test]
async fn test_credential_input_show_hide_toggle() {
    let mut wizard = SetupWizard::new_skip_welcome();
    
    // Navigate to ApiKeyInput
    wizard.handle_key(KeyCode::Char(' '), KeyModifiers::empty()).await.unwrap(); // Select Gemini
    wizard.handle_key(KeyCode::Enter, KeyModifiers::empty()).await.unwrap();
    
    if let SetupState::ApiKeyInput { show_input, .. } = &mut wizard.state {
        assert!(!*show_input);
        
        // Toggle show
        wizard.handle_key(KeyCode::Char('h'), KeyModifiers::CONTROL).await.unwrap();
        assert!(*show_input);
        
        // Toggle hide
        wizard.handle_key(KeyCode::Char('h'), KeyModifiers::CONTROL).await.unwrap();
        assert!(!*show_input);
    } else {
        panic!("Expected ApiKeyInput state");
    }
}

#[tokio::test]
async fn test_credential_input_clear() {
    let mut wizard = SetupWizard::new_skip_welcome();
    
    // Navigate to ApiKeyInput
    wizard.handle_key(KeyCode::Char(' '), KeyModifiers::empty()).await.unwrap(); // Select Gemini
    wizard.handle_key(KeyCode::Enter, KeyModifiers::empty()).await.unwrap();
    
    if let SetupState::ApiKeyInput { input, .. } = &mut wizard.state {
        // Type some input
        wizard.handle_key(KeyCode::Char('t'), KeyModifiers::empty()).await.unwrap();
        wizard.handle_key(KeyCode::Char('e'), KeyModifiers::empty()).await.unwrap();
        wizard.handle_key(KeyCode::Char('s'), KeyModifiers::empty()).await.unwrap();
        assert_eq!(input.len(), 3);
        
        // Clear input
        wizard.handle_key(KeyCode::Char('u'), KeyModifiers::CONTROL).await.unwrap();
        assert_eq!(input.len(), 0);
    } else {
        panic!("Expected ApiKeyInput state");
    }
}

#[tokio::test]
async fn test_empty_api_key_validation() {
    let mut wizard = SetupWizard::new_skip_welcome();
    
    // Navigate to ApiKeyInput
    wizard.handle_key(KeyCode::Char(' '), KeyModifiers::empty()).await.unwrap(); // Select Gemini
    wizard.handle_key(KeyCode::Enter, KeyModifiers::empty()).await.unwrap();
    
    // Try to validate empty key
    wizard.handle_key(KeyCode::Enter, KeyModifiers::empty()).await.unwrap();
    
    assert!(wizard.error_message.is_some());
    assert!(wizard.error_message.as_ref().unwrap().contains("cannot be empty"));
}

#[tokio::test]
async fn test_validation_result_retry() {
    let mut wizard = SetupWizard::new_skip_welcome();
    
    // This test would require mocking the validation service
    // For now, we test the state transition logic
    let state = SetupState::ValidationResult {
        provider: ProviderType::Gemini,
        api_key: "test-key".to_string(),
        result: Err(radium_core::auth::AuthError::Unauthorized {
            provider: "Gemini (Google)".to_string(),
        }),
        remaining_providers: vec![],
    };
    wizard.state = state;
    
    // Press R to retry
    wizard.handle_key(KeyCode::Char('r'), KeyModifiers::empty()).await.unwrap();
    
    // Should transition to Validating state
    assert!(matches!(wizard.state, SetupState::Validating { .. }));
}

#[tokio::test]
async fn test_validation_result_edit() {
    let mut wizard = SetupWizard::new_skip_welcome();
    
    let state = SetupState::ValidationResult {
        provider: ProviderType::Gemini,
        api_key: "old-key".to_string(),
        result: Err(radium_core::auth::AuthError::Unauthorized {
            provider: "Gemini (Google)".to_string(),
        }),
        remaining_providers: vec![],
    };
    wizard.state = state;
    
    // Press E to edit
    wizard.handle_key(KeyCode::Char('e'), KeyModifiers::empty()).await.unwrap();
    
    // Should transition back to ApiKeyInput with previous input
    if let SetupState::ApiKeyInput { input, .. } = &wizard.state {
        assert_eq!(input, "old-key");
    } else {
        panic!("Expected ApiKeyInput state");
    }
}

#[tokio::test]
async fn test_multi_provider_sequential_flow() {
    let temp_dir = TempDir::new().unwrap();
    let creds_path = temp_dir.path().join("credentials.json");
    let store = CredentialStore::with_path(creds_path);
    
    let mut wizard = SetupWizard::new_skip_welcome();
    
    // Select both Gemini and OpenAI
    wizard.handle_key(KeyCode::Char(' '), KeyModifiers::empty()).await.unwrap(); // Select Gemini
    wizard.handle_key(KeyCode::Down, KeyModifiers::empty()).await.unwrap();
    wizard.handle_key(KeyCode::Char(' '), KeyModifiers::empty()).await.unwrap(); // Select OpenAI
    wizard.handle_key(KeyCode::Enter, KeyModifiers::empty()).await.unwrap();
    
    // Should start with Gemini
    if let SetupState::ApiKeyInput { provider, remaining_providers, .. } = &wizard.state {
        assert_eq!(*provider, ProviderType::Gemini);
        assert_eq!(remaining_providers.len(), 1);
        assert_eq!(remaining_providers[0], ProviderType::OpenAI);
    } else {
        panic!("Expected ApiKeyInput state for Gemini");
    }
}

#[tokio::test]
async fn test_complete_state_shows_configured_providers() {
    let temp_dir = TempDir::new().unwrap();
    let creds_path = temp_dir.path().join("credentials.json");
    let store = CredentialStore::with_path(creds_path);
    
    // Configure both Gemini and OpenAI
    store.store(ProviderType::Gemini, "gemini-key".to_string()).unwrap();
    store.store(ProviderType::OpenAI, "openai-key".to_string()).unwrap();
    
    let state = SetupState::Complete {
        configured_providers: vec![ProviderType::Gemini, ProviderType::OpenAI],
    };
    let wizard = SetupWizard {
        state,
        error_message: None,
        validation_service: radium_tui::setup::ValidationService::new(),
    };
    
    let lines = wizard.display_lines();
    let display_text = lines.join("\n");
    
    assert!(display_text.contains("Gemini (Google)"));
    assert!(display_text.contains("OpenAI (GPT)"));
}

#[tokio::test]
async fn test_provider_selection_cursor_navigation() {
    let mut wizard = SetupWizard::new_skip_welcome();
    
    if let SetupState::ProviderSelection { cursor, .. } = &wizard.state {
        let initial_cursor = *cursor;
        
        // Move down twice (should handle 3 providers)
        wizard.handle_key(KeyCode::Down, KeyModifiers::empty()).await.unwrap();
        if let SetupState::ProviderSelection { cursor, .. } = &wizard.state {
            assert_eq!(*cursor, (initial_cursor + 1).min(2));
        }
        
        wizard.handle_key(KeyCode::Down, KeyModifiers::empty()).await.unwrap();
        if let SetupState::ProviderSelection { cursor, .. } = &wizard.state {
            assert_eq!(*cursor, 2); // Max is 2 for 3 providers (0, 1, 2)
        }
        
        // Move up
        wizard.handle_key(KeyCode::Up, KeyModifiers::empty()).await.unwrap();
        if let SetupState::ProviderSelection { cursor, .. } = &wizard.state {
            assert_eq!(*cursor, 1);
        }
    } else {
        panic!("Expected ProviderSelection state");
    }
}

#[tokio::test]
async fn test_provider_selection_requires_at_least_one() {
    let mut wizard = SetupWizard::new_skip_welcome();
    
    // Try to proceed without selecting any provider
    wizard.handle_key(KeyCode::Enter, KeyModifiers::empty()).await.unwrap();
    
    assert!(wizard.error_message.is_some());
    assert!(wizard.error_message.as_ref().unwrap().contains("select at least one"));
}

#[tokio::test]
async fn test_validation_state_progress() {
    let state = SetupState::Validating {
        provider: ProviderType::Gemini,
        progress: ValidationProgress::Testing,
    };
    let wizard = SetupWizard {
        state,
        error_message: None,
        validation_service: radium_tui::setup::ValidationService::new(),
    };
    
    let lines = wizard.display_lines();
    let display_text = lines.join("\n");
    
    assert!(display_text.contains("Validating"));
    assert!(display_text.contains("Testing connection"));
}


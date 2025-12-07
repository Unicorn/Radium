//! Interactive setup wizard for first-time users.
//!
//! Guides users through authentication and configuration.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use radium_core::auth::{CredentialStore, ProviderType};

use crate::icons::Icons;
use crate::theme::THEME;

/// Setup wizard state.
#[derive(Debug, Clone, PartialEq)]
pub enum SetupState {
    /// Welcome screen
    Welcome,
    /// Provider selection
    ProviderSelection {
        selected_providers: Vec<ProviderType>,
        cursor: usize,
    },
    /// API key input for a specific provider
    ApiKeyInput {
        provider: ProviderType,
        input: String,
    },
    /// Validating API key
    Validating {
        provider: ProviderType,
    },
    /// Setup complete
    Complete,
}

/// Setup wizard manager.
pub struct SetupWizard {
    pub state: SetupState,
    pub error_message: Option<String>,
}

impl SetupWizard {
    /// Creates a new setup wizard.
    pub fn new() -> Self {
        Self {
            state: SetupState::Welcome,
            error_message: None,
        }
    }

    /// Checks if setup is needed.
    pub fn is_needed() -> bool {
        if let Ok(store) = CredentialStore::new() {
            !store.is_configured(ProviderType::Gemini)
                && !store.is_configured(ProviderType::OpenAI)
        } else {
            true
        }
    }

    /// Handles key input.
    pub async fn handle_key(&mut self, key: KeyCode, _modifiers: KeyModifiers) -> Result<bool> {
        match &mut self.state {
            SetupState::Welcome => {
                if matches!(key, KeyCode::Enter | KeyCode::Char(' ')) {
                    // Move to provider selection
                    self.state = SetupState::ProviderSelection {
                        selected_providers: vec![],
                        cursor: 0,
                    };
                }
            }
            SetupState::ProviderSelection { selected_providers, cursor } => {
                match key {
                    KeyCode::Up => {
                        *cursor = cursor.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        let max = 1; // 2 providers (0 and 1)
                        *cursor = (*cursor + 1).min(max);
                    }
                    KeyCode::Char(' ') => {
                        // Toggle selection
                        let provider = match *cursor {
                            0 => ProviderType::Gemini,
                            1 => ProviderType::OpenAI,
                            _ => return Ok(false),
                        };

                        if let Some(pos) = selected_providers.iter().position(|p| *p == provider) {
                            selected_providers.remove(pos);
                        } else {
                            selected_providers.push(provider);
                        }
                    }
                    KeyCode::Enter => {
                        if !selected_providers.is_empty() {
                            // Start with first selected provider
                            let provider = selected_providers[0];
                            self.state = SetupState::ApiKeyInput {
                                provider,
                                input: String::new(),
                            };
                        } else {
                            self.error_message = Some("Please select at least one provider".to_string());
                        }
                    }
                    KeyCode::Esc => {
                        // Skip setup for now
                        return Ok(true);
                    }
                    _ => {}
                }
            }
            SetupState::ApiKeyInput { provider, input } => {
                match key {
                    KeyCode::Char(c) => {
                        input.push(c);
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Enter => {
                        if input.is_empty() {
                            self.error_message = Some("API key cannot be empty".to_string());
                        } else {
                            // Save the key
                            if let Ok(store) = CredentialStore::new() {
                                if let Err(e) = store.store(*provider, input.clone()) {
                                    self.error_message = Some(format!("Failed to save: {}", e));
                                } else {
                                    // Success! Move to complete
                                    self.state = SetupState::Complete;
                                    self.error_message = None;
                                }
                            } else {
                                self.error_message = Some("Failed to access credential store".to_string());
                            }
                        }
                    }
                    KeyCode::Esc => {
                        // Go back to provider selection
                        self.state = SetupState::ProviderSelection {
                            selected_providers: vec![],
                            cursor: 0,
                        };
                    }
                    _ => {}
                }
            }
            SetupState::Complete => {
                // Any key exits
                return Ok(true);
            }
            _ => {}
        }

        Ok(false)
    }

    /// Returns the display lines for the current state.
    pub fn display_lines(&self) -> Vec<String> {
        let mut lines = vec![];

        match &self.state {
            SetupState::Welcome => {
                lines.push(format!("{} Welcome to Radium!", Icons::ROCKET));
                lines.push("".to_string());
                lines.push("Transform your terminal into an AI-powered workspace.".to_string());
                lines.push("".to_string());
                lines.push("Let's get you set up with AI providers...".to_string());
                lines.push("".to_string());
                lines.push("Press Enter to continue".to_string());
            }
            SetupState::ProviderSelection { selected_providers, cursor } => {
                lines.push("Select AI providers to configure:".to_string());
                lines.push("".to_string());

                let providers = vec![
                    (ProviderType::Gemini, "Gemini (Google)", "Fast, free tier available"),
                    (ProviderType::OpenAI, "OpenAI (GPT)", "Most capable models"),
                ];

                for (i, (provider_type, name, desc)) in providers.iter().enumerate() {
                    let is_selected = selected_providers.contains(provider_type);
                    let is_cursor = i == *cursor;

                    let checkbox = if is_selected { "[x]" } else { "[ ]" };
                    let cursor_mark = if is_cursor { ">" } else { " " };

                    lines.push(format!("{} {} {} - {}", cursor_mark, checkbox, name, desc));
                }

                lines.push("".to_string());
                lines.push("Use ↑↓ to navigate, Space to select, Enter to continue".to_string());
                lines.push("Press Esc to skip for now".to_string());
            }
            SetupState::ApiKeyInput { provider, input } => {
                let provider_name = match provider {
                    ProviderType::Gemini => "Gemini",
                    ProviderType::OpenAI => "OpenAI",
                };

                lines.push(format!("{} Configure {}", Icons::AUTH, provider_name));
                lines.push("".to_string());
                lines.push(format!("Enter your {} API key:", provider_name));
                lines.push("".to_string());

                // Show masked input
                let masked = "*".repeat(input.len());
                lines.push(format!("> {}_", masked));
                lines.push("".to_string());

                // Show where to get the key
                match provider {
                    ProviderType::Gemini => {
                        lines.push("Get your API key at:".to_string());
                        lines.push("https://aistudio.google.com/app/apikey".to_string());
                    }
                    ProviderType::OpenAI => {
                        lines.push("Get your API key at:".to_string());
                        lines.push("https://platform.openai.com/api-keys".to_string());
                    }
                }

                lines.push("".to_string());
                lines.push("Press Enter to save, Esc to go back".to_string());
            }
            SetupState::Complete => {
                lines.push(format!("{} Setup Complete!", Icons::SUCCESS));
                lines.push("".to_string());
                lines.push("Your API keys have been securely stored in:".to_string());
                lines.push("~/.radium/auth/credentials.json".to_string());
                lines.push("".to_string());
                lines.push("You're ready to start chatting with AI agents!".to_string());
                lines.push("".to_string());
                lines.push("Press any key to continue...".to_string());
            }
            _ => {}
        }

        // Add error message if present
        if let Some(error) = &self.error_message {
            lines.push("".to_string());
            lines.push(format!("{} {}", Icons::ERROR, error));
        }

        lines
    }

    /// Returns the title for the current state.
    pub fn title(&self) -> String {
        match &self.state {
            SetupState::Welcome => "Welcome".to_string(),
            SetupState::ProviderSelection { .. } => "Provider Selection".to_string(),
            SetupState::ApiKeyInput { provider, .. } => {
                format!("Configure {}", match provider {
                    ProviderType::Gemini => "Gemini",
                    ProviderType::OpenAI => "OpenAI",
                })
            }
            SetupState::Complete => "Setup Complete".to_string(),
            _ => "Setup".to_string(),
        }
    }
}

impl Default for SetupWizard {
    fn default() -> Self {
        Self::new()
    }
}

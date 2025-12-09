//! Interactive setup wizard for first-time users.
//!
//! Guides users through authentication and configuration.

use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use radium_core::auth::{AuthError, CredentialStore, ProviderType};

use crate::icons::Icons;

/// Validation service for testing API credentials.
pub struct ValidationService {
    client: reqwest::Client,
}

impl ValidationService {
    /// Creates a new validation service with a configured HTTP client.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        Self { client }
    }

    /// Validates credentials for a provider by making a test API call.
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider type to validate
    /// * `api_key` - The API key to test
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if validation succeeds, or an `AuthError` if it fails.
    pub async fn validate_credential(
        &self,
        provider: ProviderType,
        api_key: &str,
    ) -> Result<(), AuthError> {
        let provider_name = provider_display_name(provider).to_string();

        // Build request based on provider
        let request = match provider {
            ProviderType::Gemini => {
                let url = format!(
                    "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                    api_key
                );
                self.client.get(&url).build()
            }
            ProviderType::OpenAI => {
                self.client
                    .get("https://api.openai.com/v1/models")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .build()
            }
            ProviderType::Claude => {
                self.client
                    .get("https://api.anthropic.com/v1/models")
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .build()
            }
        };

        let request = match request {
            Ok(req) => req,
            Err(e) => {
                return Err(AuthError::ConnectionFailed {
                    provider: provider_name,
                    reason: format!("Failed to build request: {}", e),
                });
            }
        };

        // Execute request with timeout
        match tokio::time::timeout(Duration::from_secs(10), self.client.execute(request)).await {
            Ok(Ok(response)) => {
                let status = response.status();
                if status.is_success() {
                    Ok(())
                } else if status == reqwest::StatusCode::UNAUTHORIZED
                    || status == reqwest::StatusCode::FORBIDDEN
                {
                    Err(AuthError::Unauthorized { provider: provider_name })
                } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    Err(AuthError::RateLimited {
                        provider: provider_name,
                        retry_after: None, // Could parse Retry-After header if needed
                    })
                } else if status.is_server_error() {
                    Err(AuthError::ServiceUnavailable {
                        provider: provider_name,
                    })
                } else {
                    Err(AuthError::ConnectionFailed {
                        provider: provider_name,
                        reason: format!("HTTP {}", status),
                    })
                }
            }
            Ok(Err(e)) => {
                if e.is_timeout() {
                    Err(AuthError::Timeout {
                        provider: provider_name,
                    })
                } else if e.is_connect() {
                    Err(AuthError::ConnectionFailed {
                        provider: provider_name,
                        reason: "Connection failed - check your internet connection".to_string(),
                    })
                } else {
                    Err(AuthError::ConnectionFailed {
                        provider: provider_name,
                        reason: e.to_string(),
                    })
                }
            }
            Err(_) => Err(AuthError::Timeout {
                provider: provider_name,
            }),
        }
    }
}

impl Default for ValidationService {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to get provider display name.
fn provider_display_name(provider: ProviderType) -> &'static str {
    match provider {
        ProviderType::Gemini => "Gemini (Google)",
        ProviderType::OpenAI => "OpenAI (GPT)",
        ProviderType::Claude => "Claude (Anthropic)",
    }
}

/// Formats validation error messages with detailed troubleshooting information.
fn format_validation_error(error: &AuthError, provider: ProviderType) -> Vec<String> {
    let mut lines = vec![];
    let provider_name = provider_display_name(provider);

    match error {
        AuthError::Unauthorized { .. } => {
            lines.push("✗ Authentication Failed".to_string());
            lines.push("".to_string());
            lines.push(format!(
                "The API key was rejected by {}'s API.",
                provider_name
            ));
            lines.push("".to_string());
            lines.push("Possible reasons:".to_string());
            lines.push("  • API key is incorrect or has been revoked".to_string());
            lines.push("  • API key doesn't have required permissions".to_string());
            lines.push("  • Account may be suspended".to_string());
            lines.push("".to_string());
            lines.push("Next steps:".to_string());
            match provider {
                ProviderType::Gemini => {
                    lines.push("  1. Double-check your API key at aistudio.google.com/app/apikey".to_string());
                }
                ProviderType::OpenAI => {
                    lines.push("  1. Double-check your API key at platform.openai.com/api-keys".to_string());
                }
                ProviderType::Claude => {
                    lines.push("  1. Double-check your API key at console.anthropic.com/settings/keys".to_string());
                }
            }
            lines.push("  2. Ensure your account is active and in good standing".to_string());
            lines.push("  3. Generate a new API key if needed".to_string());
        }
        AuthError::ConnectionFailed { reason, .. } => {
            lines.push("✗ Connection Failed".to_string());
            lines.push("".to_string());
            lines.push(format!("Couldn't connect to {}'s API.", provider_name));
            lines.push("".to_string());
            lines.push("This might be due to:".to_string());
            lines.push("  • Network connectivity issues".to_string());
            lines.push("  • Firewall blocking the connection".to_string());
            lines.push(format!("  • {} service temporarily unavailable", provider_name));
            lines.push("".to_string());
            lines.push("Troubleshooting:".to_string());
            lines.push("  1. Check your internet connection".to_string());
            lines.push("  2. Try again in a few moments".to_string());
            match provider {
                ProviderType::Gemini => {
                    lines.push("  3. Visit status.cloud.google.com for service status".to_string());
                }
                ProviderType::OpenAI => {
                    lines.push("  3. Visit status.openai.com for service status".to_string());
                }
                ProviderType::Claude => {
                    lines.push("  3. Visit status.anthropic.com for service status".to_string());
                }
            }
            if !reason.is_empty() {
                lines.push("".to_string());
                lines.push(format!("Error details: {}", reason));
            }
        }
        AuthError::RateLimited { retry_after, .. } => {
            lines.push("✗ Rate Limited".to_string());
            lines.push("".to_string());
            lines.push(format!("Too many requests to {}'s API.", provider_name));
            lines.push("".to_string());
            if let Some(duration) = retry_after {
                lines.push(format!(
                    "Please try again after {} seconds.",
                    duration.as_secs()
                ));
            } else {
                lines.push("Try again in a few moments.".to_string());
            }
        }
        AuthError::Timeout { .. } => {
            lines.push("✗ Connection Timeout".to_string());
            lines.push("".to_string());
            lines.push(format!("Request to {}'s API timed out.", provider_name));
            lines.push("".to_string());
            lines.push("Troubleshooting:".to_string());
            lines.push("  1. Check your network connection".to_string());
            lines.push("  2. Try again - the service may be slow to respond".to_string());
            lines.push("  3. Check if your firewall is blocking the connection".to_string());
        }
        AuthError::ServiceUnavailable { .. } => {
            lines.push("✗ Service Unavailable".to_string());
            lines.push("".to_string());
            lines.push(format!("{}'s API appears to be down.", provider_name));
            lines.push("".to_string());
            lines.push("Please:".to_string());
            match provider {
                ProviderType::Gemini => {
                    lines.push("  • Check status at status.cloud.google.com".to_string());
                }
                ProviderType::OpenAI => {
                    lines.push("  • Check status at status.openai.com".to_string());
                }
                ProviderType::Claude => {
                    lines.push("  • Check status at status.anthropic.com".to_string());
                }
            }
            lines.push("  • Try again later".to_string());
        }
        AuthError::InvalidFormat => {
            lines.push("✗ Invalid API Key Format".to_string());
            lines.push("".to_string());
            match provider {
                ProviderType::Gemini => {
                    lines.push("Gemini API keys should start with 'AIza' and be 39 characters long.".to_string());
                    lines.push("".to_string());
                    lines.push("Example: AIzaSyD1234567890abcdefghijklmnopqrstuv".to_string());
                }
                ProviderType::OpenAI => {
                    lines.push("OpenAI API keys should start with 'sk-proj-' or 'sk-'.".to_string());
                    lines.push("".to_string());
                    lines.push("Example: sk-proj-1234567890abcdefghijklmnopqrstuvwxyz".to_string());
                }
                ProviderType::Claude => {
                    lines.push("Claude API keys should start with 'sk-ant-'.".to_string());
                    lines.push("".to_string());
                    lines.push("Example: sk-ant-api03-1234567890abcdefghijklmnopqrstuvwxyz".to_string());
                }
            }
        }
        _ => {
            // Generic error message for other error types
            lines.push("✗ Validation Failed".to_string());
            lines.push("".to_string());
            lines.push(format!("{}", error));
        }
    }

    lines
}

/// Provider status for tracking configuration state.
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderStatus {
    /// Provider is not configured.
    NotConfigured,
    /// Provider is configured but not validated.
    Configured,
    /// Provider is configured and validated.
    ValidatedAndActive,
}

/// Validation progress indicator.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationProgress {
    /// Validation is in progress.
    Testing,
    /// Validation succeeded.
    Success,
    /// Validation failed with error message.
    Failed(String),
}

/// Setup wizard state.
#[derive(Debug, Clone, PartialEq)]
pub enum SetupState {
    /// Welcome screen
    Welcome,
    /// Provider selection
    ProviderSelection {
        selected_providers: Vec<ProviderType>,
        cursor: usize,
        provider_status: HashMap<ProviderType, ProviderStatus>,
    },
    /// API key input for a specific provider
    ApiKeyInput {
        provider: ProviderType,
        input: String,
        show_input: bool,
        remaining_providers: Vec<ProviderType>,
    },
    /// Validating API key
    Validating {
        provider: ProviderType,
        progress: ValidationProgress,
    },
    /// Validation result for a provider
    ValidationResult {
        provider: ProviderType,
        api_key: String,
        result: Result<(), AuthError>,
        remaining_providers: Vec<ProviderType>,
    },
    /// Setup complete
    Complete {
        configured_providers: Vec<ProviderType>,
    },
}

/// Setup wizard manager.
pub struct SetupWizard {
    pub state: SetupState,
    pub error_message: Option<String>,
    validation_service: ValidationService,
}

impl SetupWizard {
    /// Creates a new setup wizard.
    pub fn new() -> Self {
        Self {
            state: SetupState::Welcome,
            error_message: None,
            validation_service: ValidationService::new(),
        }
    }

    /// Creates a new setup wizard, skipping the welcome screen.
    ///
    /// Used when user explicitly triggers authentication via /auth command.
    pub fn new_skip_welcome() -> Self {
        // Initialize provider status from credential store
        let mut provider_status = HashMap::new();
        if let Ok(store) = CredentialStore::new() {
            for provider in [ProviderType::Gemini, ProviderType::OpenAI, ProviderType::Claude] {
                let status = if store.is_configured(provider) {
                    ProviderStatus::Configured
                } else {
                    ProviderStatus::NotConfigured
                };
                provider_status.insert(provider, status);
            }
        } else {
            // If we can't access the store, assume all are not configured
            for provider in [ProviderType::Gemini, ProviderType::OpenAI, ProviderType::Claude] {
                provider_status.insert(provider, ProviderStatus::NotConfigured);
            }
        }

        Self {
            state: SetupState::ProviderSelection {
                selected_providers: vec![],
                cursor: 0,
                provider_status,
            },
            error_message: None,
            validation_service: ValidationService::new(),
        }
    }

    /// Checks if setup is needed.
    pub fn is_needed() -> bool {
        if let Ok(store) = CredentialStore::new() {
            !store.is_configured(ProviderType::Gemini) && !store.is_configured(ProviderType::OpenAI)
        } else {
            true
        }
    }

    /// Handles key input.
    pub async fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<bool> {
        match &mut self.state {
            SetupState::Welcome => {
                if matches!(key, KeyCode::Enter | KeyCode::Char(' ')) {
                    // Move to provider selection
                    // Initialize provider status from credential store
                    let mut provider_status = HashMap::new();
                    if let Ok(store) = CredentialStore::new() {
                        for provider in [ProviderType::Gemini, ProviderType::OpenAI, ProviderType::Claude] {
                            let status = if store.is_configured(provider) {
                                ProviderStatus::Configured
                            } else {
                                ProviderStatus::NotConfigured
                            };
                            provider_status.insert(provider, status);
                        }
                    } else {
                        for provider in [ProviderType::Gemini, ProviderType::OpenAI, ProviderType::Claude] {
                            provider_status.insert(provider, ProviderStatus::NotConfigured);
                        }
                    }
                    self.state = SetupState::ProviderSelection {
                        selected_providers: vec![],
                        cursor: 0,
                        provider_status,
                    };
                }
            }
            SetupState::ProviderSelection { selected_providers, cursor, .. } => {
                match key {
                    KeyCode::Up => {
                        *cursor = cursor.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        let max = 2; // 3 providers (0, 1, 2)
                        *cursor = (*cursor + 1).min(max);
                    }
                    KeyCode::Char(' ') => {
                        // Toggle selection
                        let provider = match *cursor {
                            0 => ProviderType::Gemini,
                            1 => ProviderType::OpenAI,
                            2 => ProviderType::Claude,
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
                            let remaining = selected_providers[1..].to_vec();
                            self.state = SetupState::ApiKeyInput {
                                provider,
                                input: String::new(),
                                show_input: false,
                                remaining_providers: remaining,
                            };
                        } else {
                            self.error_message =
                                Some("Please select at least one provider".to_string());
                        }
                    }
                    KeyCode::Esc => {
                        // Skip setup for now
                        return Ok(true);
                    }
                    _ => {}
                }
            }
            SetupState::ApiKeyInput { provider, input, show_input, remaining_providers } => {
                match key {
                    KeyCode::Char('h') | KeyCode::Char('H') if modifiers.contains(KeyModifiers::CONTROL) => {
                        // Toggle show/hide
                        *show_input = !*show_input;
                    }
                    KeyCode::Char('u') | KeyCode::Char('U') if modifiers.contains(KeyModifiers::CONTROL) => {
                        // Clear input
                        input.clear();
                    }
                    KeyCode::Char(c) if !modifiers.contains(KeyModifiers::CONTROL) => {
                        input.push(c);
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Enter => {
                        if input.is_empty() {
                            self.error_message = Some("API key cannot be empty".to_string());
                        } else {
                            // Start validation
                            let api_key = input.clone();
                            let provider = *provider;
                            self.state = SetupState::Validating {
                                provider,
                                progress: ValidationProgress::Testing,
                            };

                            // Spawn async validation task
                            let remaining = remaining_providers.clone();
                            let validation_result = self.validation_service.validate_credential(provider, &api_key).await;

                            // Update state based on validation result
                            self.state = SetupState::ValidationResult {
                                provider,
                                api_key,
                                result: validation_result.map_err(|e| e),
                                remaining_providers: remaining,
                            };
                        }
                    }
                    KeyCode::Esc => {
                        // Go back to provider selection - need to reconstruct with status
                        let mut provider_status = HashMap::new();
                        if let Ok(store) = CredentialStore::new() {
                            for p in [ProviderType::Gemini, ProviderType::OpenAI, ProviderType::Claude] {
                                let status = if store.is_configured(p) {
                                    ProviderStatus::Configured
                                } else {
                                    ProviderStatus::NotConfigured
                                };
                                provider_status.insert(p, status);
                            }
                        } else {
                            for p in [ProviderType::Gemini, ProviderType::OpenAI, ProviderType::Claude] {
                                provider_status.insert(p, ProviderStatus::NotConfigured);
                            }
                        }
                        // Reconstruct selected providers list (current + remaining)
                        let mut selected = vec![*provider];
                        selected.extend(remaining_providers.iter().cloned());
                        self.state = SetupState::ProviderSelection {
                            selected_providers: selected,
                            cursor: 0,
                            provider_status,
                        };
                    }
                    _ => {}
                }
            }
            SetupState::Validating { .. } => {
                // Validation is in progress - no keyboard handling during validation
                // State will be updated by the validation task
            }
            SetupState::ValidationResult { provider, api_key, result, remaining_providers } => {
                match key {
                    KeyCode::Enter => {
                        match result {
                            Ok(()) => {
                                // Save the credential
                                if let Ok(store) = CredentialStore::new() {
                                    if let Err(e) = store.store(*provider, api_key.clone()) {
                                        self.error_message = Some(format!("Failed to save: {}", e));
                                    } else {
                                        // Success! Check if there are more providers to configure
                                        if !remaining_providers.is_empty() {
                                            // Move to next provider
                                            let next_provider = remaining_providers[0];
                                            let next_remaining = remaining_providers[1..].to_vec();
                                            self.state = SetupState::ApiKeyInput {
                                                provider: next_provider,
                                                input: String::new(),
                                                show_input: false,
                                                remaining_providers: next_remaining,
                                            };
                                            self.error_message = None;
                                        } else {
                                            // All providers configured - collect all configured providers from store
                                            let configured_providers = if let Ok(store) = CredentialStore::new() {
                                                let mut configured = Vec::new();
                                                for provider in [ProviderType::Gemini, ProviderType::OpenAI, ProviderType::Claude] {
                                                    if store.is_configured(provider) {
                                                        configured.push(provider);
                                                    }
                                                }
                                                configured
                                            } else {
                                                vec![*provider]
                                            };
                                            self.state = SetupState::Complete {
                                                configured_providers,
                                            };
                                            self.error_message = None;
                                        }
                                    }
                                } else {
                                    self.error_message =
                                        Some("Failed to access credential store".to_string());
                                }
                            }
                            Err(_) => {
                                // Don't do anything on Enter if validation failed
                            }
                        }
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        // Retry validation with same credentials
                        let key_to_retry = api_key.clone();
                        let provider_to_retry = *provider;
                        let remaining = remaining_providers.clone();
                        self.state = SetupState::Validating {
                            provider: provider_to_retry,
                            progress: ValidationProgress::Testing,
                        };

                        // Spawn async validation task
                        let validation_result = self.validation_service.validate_credential(provider_to_retry, &key_to_retry).await;

                        // Update state based on validation result
                        self.state = SetupState::ValidationResult {
                            provider: provider_to_retry,
                            api_key: key_to_retry,
                            result: validation_result.map_err(|e| e),
                            remaining_providers: remaining,
                        };
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        // Edit credentials - go back to input screen with current input
                        let remaining = remaining_providers.clone();
                        self.state = SetupState::ApiKeyInput {
                            provider: *provider,
                            input: api_key.clone(),
                            show_input: false,
                            remaining_providers: remaining,
                        };
                    }
                    KeyCode::Esc => {
                        // Skip this provider and continue
                        // Go back to provider selection
                        let mut provider_status = HashMap::new();
                        if let Ok(store) = CredentialStore::new() {
                            for p in [ProviderType::Gemini, ProviderType::OpenAI, ProviderType::Claude] {
                                let status = if store.is_configured(p) {
                                    ProviderStatus::Configured
                                } else {
                                    ProviderStatus::NotConfigured
                                };
                                provider_status.insert(p, status);
                            }
                        } else {
                            for p in [ProviderType::Gemini, ProviderType::OpenAI, ProviderType::Claude] {
                                provider_status.insert(p, ProviderStatus::NotConfigured);
                            }
                        }
                        self.state = SetupState::ProviderSelection {
                            selected_providers: vec![],
                            cursor: 0,
                            provider_status,
                        };
                    }
                    _ => {}
                }
            }
            SetupState::Complete { .. } => {
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
            SetupState::ProviderSelection { selected_providers, cursor, provider_status } => {
                lines.push("Select AI providers to configure:".to_string());
                lines.push("".to_string());

                let providers = vec![
                    (ProviderType::Gemini, "Gemini (Google)", "Fast, free tier available"),
                    (ProviderType::OpenAI, "OpenAI (GPT)", "Most capable models"),
                    (ProviderType::Claude, "Claude (Anthropic)", "Advanced reasoning"),
                ];

                for (i, (provider_type, name, desc)) in providers.iter().enumerate() {
                    let is_selected = selected_providers.contains(provider_type);
                    let is_cursor = i == *cursor;
                    let status = provider_status
                        .get(provider_type)
                        .map(|s| match s {
                            ProviderStatus::ValidatedAndActive => " ✓ Connected",
                            ProviderStatus::Configured => " ⚠ Configured",
                            ProviderStatus::NotConfigured => " ✗ Not configured",
                        })
                        .unwrap_or(" ✗ Not configured");

                    let checkbox = if is_selected { "[x]" } else { "[ ]" };
                    let cursor_mark = if is_cursor { ">" } else { " " };

                    lines.push(format!(
                        "{} {} {}{} - {}",
                        cursor_mark, checkbox, name, status, desc
                    ));
                }

                lines.push("".to_string());
                lines.push("Use ↑↓ to navigate, Space to select, Enter to continue".to_string());
                lines.push("Press Esc to skip for now".to_string());
            }
            SetupState::ApiKeyInput { provider, input, show_input, .. } => {
                let provider_name = match provider {
                    ProviderType::Gemini => "Gemini",
                    ProviderType::OpenAI => "OpenAI",
                    ProviderType::Claude => "Claude",
                };

                lines.push(format!("{} Configure {}", Icons::AUTH, provider_name));
                lines.push("".to_string());
                lines.push(format!("Enter your {} API key:", provider_name));
                lines.push("".to_string());

                // Show masked or visible input
                let display = if *show_input {
                    input.clone()
                } else {
                    "*".repeat(input.len())
                };
                lines.push(format!("> {}_", display));
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
                    ProviderType::Claude => {
                        lines.push("Get your API key at:".to_string());
                        lines.push("https://console.anthropic.com/settings/keys".to_string());
                    }
                }

                lines.push("".to_string());
                lines.push("Press Ctrl+H to show/hide, Ctrl+U to clear".to_string());
                lines.push("Press Enter to validate, Esc to go back".to_string());
            }
            SetupState::Validating { provider, progress } => {
                let provider_name = match provider {
                    ProviderType::Gemini => "Gemini",
                    ProviderType::OpenAI => "OpenAI",
                    ProviderType::Claude => "Claude",
                };
                lines.push(format!("{} Validating {}", Icons::LOADING, provider_name));
                lines.push("".to_string());
                match progress {
                    ValidationProgress::Testing => {
                        lines.push("⏳ Testing connection...".to_string());
                    }
                    ValidationProgress::Success => {
                        lines.push("✓ Credentials validated successfully".to_string());
                    }
                    ValidationProgress::Failed(msg) => {
                        lines.push(format!("✗ Validation failed: {}", msg));
                    }
                }
            }
            SetupState::ValidationResult { provider, result, .. } => {
                match result {
                    Ok(()) => {
                        let provider_name = match provider {
                            ProviderType::Gemini => "Gemini",
                            ProviderType::OpenAI => "OpenAI",
                            ProviderType::Claude => "Claude",
                        };
                        lines.push(format!("{} Validation Successful!", Icons::SUCCESS));
                        lines.push("".to_string());
                        lines.push(format!("✓ {} credentials validated successfully", provider_name));
                        lines.push("".to_string());
                        lines.push("Press Enter to continue".to_string());
                    }
                    Err(e) => {
                        // Use detailed error formatter
                        let error_lines = format_validation_error(e, *provider);
                        lines.extend(error_lines);
                        lines.push("".to_string());
                        lines.push("Press R to retry, E to edit, or Esc to skip".to_string());
                    }
                }
            }
            SetupState::Complete { configured_providers } => {
                lines.push(format!("{} Setup Complete!", Icons::SUCCESS));
                lines.push("".to_string());
                lines.push("Your API keys have been securely stored in:".to_string());
                lines.push("~/.radium/auth/credentials.json".to_string());
                lines.push("".to_string());
                if !configured_providers.is_empty() {
                    lines.push("Configured providers:".to_string());
                    for provider in configured_providers {
                        let name = match provider {
                            ProviderType::Gemini => "Gemini (Google)",
                            ProviderType::OpenAI => "OpenAI (GPT)",
                            ProviderType::Claude => "Claude (Anthropic)",
                        };
                        lines.push(format!("  ✓ {}", name));
                    }
                    lines.push("".to_string());
                }
                lines.push("You're ready to start chatting with AI agents!".to_string());
                lines.push("".to_string());
                lines.push("Press any key to continue...".to_string());
            }
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
                format!(
                    "Configure {}",
                    match provider {
                        ProviderType::Gemini => "Gemini",
                        ProviderType::OpenAI => "OpenAI",
                        ProviderType::Claude => "Claude",
                    }
                )
            }
            SetupState::Validating { provider, .. } => {
                format!(
                    "Validating {}",
                    match provider {
                        ProviderType::Gemini => "Gemini",
                        ProviderType::OpenAI => "OpenAI",
                        ProviderType::Claude => "Claude",
                    }
                )
            }
            SetupState::ValidationResult { .. } => "Validation Result".to_string(),
            SetupState::Complete { .. } => "Setup Complete".to_string(),
            _ => "Setup".to_string(),
        }
    }
}

impl Default for SetupWizard {
    fn default() -> Self {
        Self::new()
    }
}

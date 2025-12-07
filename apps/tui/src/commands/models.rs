//! Model selection command implementation.

use anyhow::Result;
use radium_core::auth::{CredentialStore, ProviderType};

/// Get available models from all providers.
pub fn get_available_models() -> Result<Vec<crate::views::model_selector::ModelInfo>> {
    let mut models = Vec::new();
    let store = CredentialStore::new().ok();

    // Get Gemini models (if configured)
    let gemini_configured = store.as_ref()
        .map(|s| s.is_configured(ProviderType::Gemini))
        .unwrap_or(false);

    if gemini_configured {
        models.push(crate::views::model_selector::ModelInfo {
            id: "gemini-2.0-flash-thinking".to_string(),
            name: "gemini-2.0-flash-thinking".to_string(),
            provider: "Gemini".to_string(),
            description: Some("Default - Reasoning optimized".to_string()),
            is_selected: true, // TODO: Load from config
        });
        models.push(crate::views::model_selector::ModelInfo {
            id: "gemini-2.0-flash-exp".to_string(),
            name: "gemini-2.0-flash-exp".to_string(),
            provider: "Gemini".to_string(),
            description: Some("Fast, experimental".to_string()),
            is_selected: false,
        });
        models.push(crate::views::model_selector::ModelInfo {
            id: "gemini-1.5-pro".to_string(),
            name: "gemini-1.5-pro".to_string(),
            provider: "Gemini".to_string(),
            description: Some("Most capable".to_string()),
            is_selected: false,
        });
    }

    // Get OpenAI models (if configured)
    let openai_configured = store.as_ref()
        .map(|s| s.is_configured(ProviderType::OpenAI))
        .unwrap_or(false);

    if openai_configured {
        models.push(crate::views::model_selector::ModelInfo {
            id: "gpt-4o".to_string(),
            name: "gpt-4o".to_string(),
            provider: "OpenAI".to_string(),
            description: Some("Multimodal".to_string()),
            is_selected: false,
        });
        models.push(crate::views::model_selector::ModelInfo {
            id: "gpt-4o-mini".to_string(),
            name: "gpt-4o-mini".to_string(),
            provider: "OpenAI".to_string(),
            description: Some("Fast, efficient".to_string()),
            is_selected: false,
        });
    }

    Ok(models)
}


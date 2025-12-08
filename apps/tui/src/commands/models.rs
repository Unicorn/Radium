//! Model selection command implementation.

use anyhow::Result;
use radium_core::auth::{CredentialStore, ProviderType};

/// Get available models from all providers.
pub fn get_available_models() -> Result<Vec<crate::views::model_selector::ModelInfo>> {
    let mut models = Vec::new();
    let store = CredentialStore::new().ok();

    // Load config to get default model
    let config = crate::config::TuiConfig::load().unwrap_or_default();
    let default_model_id = config.model.default_model_id;

    // Get Gemini models (if configured)
    let gemini_configured =
        store.as_ref().map(|s| s.is_configured(ProviderType::Gemini)).unwrap_or(false);

    if gemini_configured {
        models.push(crate::views::model_selector::ModelInfo {
            id: "gemini-2.0-flash-thinking".to_string(),
            name: "gemini-2.0-flash-thinking".to_string(),
            provider: "Gemini".to_string(),
            description: Some("Default - Reasoning optimized".to_string()),
            is_selected: "gemini-2.0-flash-thinking" == default_model_id,
        });
        models.push(crate::views::model_selector::ModelInfo {
            id: "gemini-2.0-flash-exp".to_string(),
            name: "gemini-2.0-flash-exp".to_string(),
            provider: "Gemini".to_string(),
            description: Some("Fast, experimental".to_string()),
            is_selected: "gemini-2.0-flash-exp" == default_model_id,
        });
        models.push(crate::views::model_selector::ModelInfo {
            id: "gemini-1.5-pro".to_string(),
            name: "gemini-1.5-pro".to_string(),
            provider: "Gemini".to_string(),
            description: Some("Most capable".to_string()),
            is_selected: "gemini-1.5-pro" == default_model_id,
        });
    }

    // Get OpenAI models (if configured)
    let openai_configured =
        store.as_ref().map(|s| s.is_configured(ProviderType::OpenAI)).unwrap_or(false);

    if openai_configured {
        models.push(crate::views::model_selector::ModelInfo {
            id: "gpt-4o".to_string(),
            name: "gpt-4o".to_string(),
            provider: "OpenAI".to_string(),
            description: Some("Multimodal".to_string()),
            is_selected: "gpt-4o" == default_model_id,
        });
        models.push(crate::views::model_selector::ModelInfo {
            id: "gpt-4o-mini".to_string(),
            name: "gpt-4o-mini".to_string(),
            provider: "OpenAI".to_string(),
            description: Some("Fast, efficient".to_string()),
            is_selected: "gpt-4o-mini" == default_model_id,
        });
    }

    Ok(models)
}

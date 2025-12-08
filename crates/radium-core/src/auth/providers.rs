//! Provider type definitions and metadata.

use serde::{Deserialize, Serialize};

/// Supported AI provider types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    /// Google Gemini
    Gemini,
    /// OpenAI
    OpenAI,
    /// Anthropic Claude
    Claude,
}

impl ProviderType {
    /// Returns the string representation of the provider.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gemini => "gemini",
            Self::OpenAI => "openai",
            Self::Claude => "claude",
        }
    }

    /// Parses a provider type from a string.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to parse (case-insensitive)
    ///
    /// # Returns
    ///
    /// `Some(ProviderType)` if the string matches a known provider, `None` otherwise.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "gemini" => Some(Self::Gemini),
            "openai" => Some(Self::OpenAI),
            "claude" => Some(Self::Claude),
            _ => None,
        }
    }

    /// Returns all supported provider types.
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![Self::Gemini, Self::OpenAI, Self::Claude]
    }

    /// Returns the environment variable names that can be used for this provider.
    ///
    /// Credentials are checked in the order returned.
    #[must_use]
    pub fn env_var_names(self) -> Vec<&'static str> {
        match self {
            Self::Gemini => vec!["GOOGLE_API_KEY", "GEMINI_API_KEY"],
            Self::OpenAI => vec!["OPENAI_API_KEY"],
            Self::Claude => vec!["ANTHROPIC_API_KEY"],
        }
    }
}

/// Provider credential information stored in the credentials file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    /// The provider type.
    #[serde(rename = "type")]
    pub kind: ProviderType,
    /// The API key for this provider.
    pub api_key: String,
    /// Whether this provider is enabled.
    pub enabled: bool,
    /// Last update timestamp in RFC 3339 format.
    #[serde(with = "time::serde::rfc3339")]
    pub last_updated: time::OffsetDateTime,
    /// Team name for cost attribution (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
    /// Project name for cost attribution (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    /// Cost center for chargeback (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_center: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_as_str() {
        assert_eq!(ProviderType::Gemini.as_str(), "gemini");
        assert_eq!(ProviderType::OpenAI.as_str(), "openai");
        assert_eq!(ProviderType::Claude.as_str(), "claude");
    }

    #[test]
    fn test_provider_type_parse() {
        assert_eq!(ProviderType::parse("gemini"), Some(ProviderType::Gemini));
        assert_eq!(ProviderType::parse("GEMINI"), Some(ProviderType::Gemini));
        assert_eq!(ProviderType::parse("openai"), Some(ProviderType::OpenAI));
        assert_eq!(ProviderType::parse("OpenAI"), Some(ProviderType::OpenAI));
        assert_eq!(ProviderType::parse("claude"), Some(ProviderType::Claude));
        assert_eq!(ProviderType::parse("Claude"), Some(ProviderType::Claude));
        assert_eq!(ProviderType::parse("unknown"), None);
    }

    #[test]
    fn test_provider_type_all() {
        let all = ProviderType::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&ProviderType::Gemini));
        assert!(all.contains(&ProviderType::OpenAI));
        assert!(all.contains(&ProviderType::Claude));
    }

    #[test]
    fn test_provider_type_env_var_names() {
        let gemini_vars = ProviderType::Gemini.env_var_names();
        assert_eq!(gemini_vars, vec!["GOOGLE_API_KEY", "GEMINI_API_KEY"]);

        let openai_vars = ProviderType::OpenAI.env_var_names();
        assert_eq!(openai_vars, vec!["OPENAI_API_KEY"]);

        let claude_vars = ProviderType::Claude.env_var_names();
        assert_eq!(claude_vars, vec!["ANTHROPIC_API_KEY"]);
    }

    #[test]
    fn test_provider_serialization() {
        let provider = Provider {
            kind: ProviderType::Gemini,
            api_key: "test-key".to_string(),
            enabled: true,
            last_updated: time::OffsetDateTime::now_utc(),
        };

        let json = serde_json::to_string(&provider).unwrap();
        assert!(json.contains("gemini"));
        assert!(json.contains("test-key"));
        assert!(json.contains("\"enabled\":true"));

        let deserialized: Provider = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.kind, ProviderType::Gemini);
        assert_eq!(deserialized.api_key, "test-key");
        assert!(deserialized.enabled);
    }
}

//! Agent configuration file format.
//!
//! Defines the TOML configuration format for agents.

use crate::sandbox::SandboxConfig;
use radium_abstraction::{ModelParameters, ResponseFormat, SafetyBlockBehavior};
use radium_models::{GeminiSafetySetting, SafetyCategory, SafetyThreshold};
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::PathBuf;
use thiserror::Error;

// Note: We use a type alias to avoid circular dependencies.
// The actual types are in crate::workflow::behaviors, but we'll
// define them here as optional TOML fields that can be deserialized.
// For now, we'll use a simplified representation that can be
// converted to the full types when needed.

/// Simplified loop behavior configuration for TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLoopBehavior {
    /// Number of steps to go back when looping.
    pub steps: usize,
    /// Maximum number of iterations before stopping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_iterations: Option<usize>,
    /// List of step IDs to skip during loop.
    #[serde(default)]
    pub skip: Vec<String>,
}

/// Simplified trigger behavior configuration for TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTriggerBehavior {
    /// Default agent ID to trigger (can be overridden in behavior.json).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_agent_id: Option<String>,
}

/// Routing configuration for agent model selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRoutingConfig {
    /// Routing strategy to use.
    ///
    /// Options: "complexity_based", "cost_optimized", "latency_optimized", "quality_optimized"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
    
    /// Maximum cost per request in USD (optional constraint).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cost_per_request: Option<f64>,
}

/// Agent configuration errors.
#[derive(Debug, Error)]
pub enum AgentConfigError {
    /// Invalid configuration.
    #[error("invalid configuration: {0}")]
    Invalid(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML deserialization error.
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Result type for agent configuration operations.
pub type Result<T> = std::result::Result<T, AgentConfigError>;

/// Reasoning effort levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ReasoningEffort {
    /// Minimal reasoning effort.
    Low,

    /// Moderate reasoning effort.
    #[default]
    Medium,

    /// Maximum reasoning effort.
    High,
}

impl std::fmt::Display for ReasoningEffort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
        }
    }
}

/// Model class categories for agent selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelClass {
    /// Fast models (e.g., Flash, Mini) - optimized for speed.
    Fast,

    /// Balanced models (e.g., Pro, 4o) - balanced speed and quality.
    Balanced,

    /// Reasoning models (e.g., o1, Thinking) - optimized for deep reasoning.
    Reasoning,
}

impl std::fmt::Display for ModelClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fast => write!(f, "fast"),
            Self::Balanced => write!(f, "balanced"),
            Self::Reasoning => write!(f, "reasoning"),
        }
    }
}

/// Cost tier for agent model selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CostTier {
    /// Low cost tier.
    Low,

    /// Medium cost tier.
    Medium,

    /// High cost tier.
    High,
}

impl std::fmt::Display for CostTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
        }
    }
}

/// Agent capabilities for dynamic selection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentCapabilities {
    /// Model class category for this agent.
    pub model_class: ModelClass,

    /// Cost tier for this agent's models.
    pub cost_tier: CostTier,

    /// Maximum number of concurrent tasks this agent can handle.
    pub max_concurrent_tasks: usize,
}

impl Default for AgentCapabilities {
    fn default() -> Self {
        Self {
            model_class: ModelClass::Balanced,
            cost_tier: CostTier::Medium,
            max_concurrent_tasks: 5,
        }
    }
}

/// Persona configuration for TOML deserialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaConfigToml {
    /// Model recommendations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<PersonaModelsToml>,
    /// Performance configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<PersonaPerformanceToml>,
}

/// Model recommendations for TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaModelsToml {
    /// Primary model.
    pub primary: String,
    /// Fallback model (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
    /// Premium model (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub premium: Option<String>,
}

/// Performance configuration for TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaPerformanceToml {
    /// Performance profile.
    pub profile: String,
    /// Estimated tokens (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_tokens: Option<u64>,
}

/// Model configuration for TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfigToml {
    /// Top-k sampling parameter (1-100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    /// Frequency penalty (-2.0 to 2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    /// Presence penalty (-2.0 to 2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    /// Response format: "text", "json", or JSON schema string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
}

/// Agent configuration file (TOML format).
///
/// This is the structure of an agent configuration file, typically stored at
/// `agents/<category>/<agent-id>.toml`.
///
/// # Example TOML
///
/// ```toml
/// [agent]
/// id = "arch-agent"
/// name = "Architecture Agent"
/// description = "Defines system architecture and technical design decisions"
/// prompt_path = "prompts/agents/my-agents/arch-agent.md"
/// engine = "gemini"
/// model = "gemini-2.0-flash-exp"
/// reasoning_effort = "medium"
///
/// [agent.persona]
/// [agent.persona.models]
/// primary = "gemini-2.0-flash-exp"
/// fallback = "gemini-1.5-flash"
/// premium = "gemini-1.5-pro"
///
/// [agent.persona.performance]
/// profile = "thinking"
/// estimated_tokens = 2000
/// ```
/// Gemini-specific safety configuration for TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeminiSafetyConfigToml {
    /// Threshold for hate speech: "BLOCK_NONE", "BLOCK_LOW_AND_ABOVE", "BLOCK_MEDIUM_AND_ABOVE", or "BLOCK_ONLY_HIGH"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hate_speech: Option<String>,
    /// Threshold for harassment: "BLOCK_NONE", "BLOCK_LOW_AND_ABOVE", "BLOCK_MEDIUM_AND_ABOVE", or "BLOCK_ONLY_HIGH"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harassment: Option<String>,
    /// Threshold for sexually explicit content: "BLOCK_NONE", "BLOCK_LOW_AND_ABOVE", "BLOCK_MEDIUM_AND_ABOVE", or "BLOCK_ONLY_HIGH"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sexually_explicit: Option<String>,
    /// Threshold for dangerous content: "BLOCK_NONE", "BLOCK_LOW_AND_ABOVE", "BLOCK_MEDIUM_AND_ABOVE", or "BLOCK_ONLY_HIGH"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dangerous_content: Option<String>,
    /// Threshold for civic integrity: "BLOCK_NONE", "BLOCK_LOW_AND_ABOVE", "BLOCK_MEDIUM_AND_ABOVE", or "BLOCK_ONLY_HIGH"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub civic_integrity: Option<String>,
    /// Default threshold for categories not explicitly specified: "BLOCK_NONE", "BLOCK_LOW_AND_ABOVE", "BLOCK_MEDIUM_AND_ABOVE", or "BLOCK_ONLY_HIGH"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

impl Default for GeminiSafetyConfigToml {
    fn default() -> Self {
        Self {
            hate_speech: None,
            harassment: None,
            sexually_explicit: None,
            dangerous_content: None,
            civic_integrity: None,
            default: None,
        }
    }
}

impl GeminiSafetyConfigToml {
    /// Parse a threshold string into a SafetyThreshold enum.
    ///
    /// # Arguments
    /// * `s` - Threshold string (e.g., "BLOCK_MEDIUM_AND_ABOVE")
    ///
    /// # Errors
    /// Returns `AgentConfigError::Invalid` if the threshold string is not recognized.
    pub(crate) fn parse_threshold(s: &str) -> Result<SafetyThreshold> {
        match s {
            "BLOCK_NONE" => Ok(SafetyThreshold::BlockNone),
            "BLOCK_LOW_AND_ABOVE" => Ok(SafetyThreshold::BlockLowAndAbove),
            "BLOCK_MEDIUM_AND_ABOVE" => Ok(SafetyThreshold::BlockMediumAndAbove),
            "BLOCK_ONLY_HIGH" => Ok(SafetyThreshold::BlockOnlyHigh),
            _ => Err(AgentConfigError::Invalid(format!(
                "Invalid safety threshold: '{}'. Must be one of: BLOCK_NONE, BLOCK_LOW_AND_ABOVE, BLOCK_MEDIUM_AND_ABOVE, BLOCK_ONLY_HIGH",
                s
            ))),
        }
    }

    /// Convert this TOML configuration to a vector of GeminiSafetySetting.
    ///
    /// Applies default threshold to categories not explicitly configured.
    /// Returns an empty vector if no categories are configured and no default is set.
    ///
    /// # Errors
    /// Returns `AgentConfigError::Invalid` if any threshold string is invalid.
    pub fn to_safety_settings(&self) -> Result<Vec<GeminiSafetySetting>> {
        let mut settings = Vec::new();

        // Helper to get threshold for a category
        let get_threshold = |category_value: &Option<String>| -> Result<Option<SafetyThreshold>> {
            match category_value {
                Some(s) => Ok(Some(Self::parse_threshold(s)?)),
                None => Ok(None),
            }
        };

        // Get default threshold if available
        let default_threshold = match &self.default {
            Some(s) => Some(Self::parse_threshold(s)?),
            None => None,
        };

        // Process each category
        // Hate Speech
        let threshold = get_threshold(&self.hate_speech)?
            .or(default_threshold);
        if let Some(threshold) = threshold {
            settings.push(GeminiSafetySetting {
                category: SafetyCategory::HateSpeech,
                threshold,
            });
        }

        // Harassment
        let threshold = get_threshold(&self.harassment)?
            .or(default_threshold);
        if let Some(threshold) = threshold {
            settings.push(GeminiSafetySetting {
                category: SafetyCategory::Harassment,
                threshold,
            });
        }

        // Sexually Explicit
        let threshold = get_threshold(&self.sexually_explicit)?
            .or(default_threshold);
        if let Some(threshold) = threshold {
            settings.push(GeminiSafetySetting {
                category: SafetyCategory::SexuallyExplicit,
                threshold,
            });
        }

        // Dangerous Content
        let threshold = get_threshold(&self.dangerous_content)?
            .or(default_threshold);
        if let Some(threshold) = threshold {
            settings.push(GeminiSafetySetting {
                category: SafetyCategory::DangerousContent,
                threshold,
            });
        }

        // Civic Integrity
        let threshold = get_threshold(&self.civic_integrity)?
            .or(default_threshold);
        if let Some(threshold) = threshold {
            settings.push(GeminiSafetySetting {
                category: SafetyCategory::CivicIntegrity,
                threshold,
            });
        }

        Ok(settings)
    }
}

/// Safety configuration for TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfigToml {
    /// Safety block behavior: "return-partial", "error", or "log".
    #[serde(default = "default_safety_behavior")]
    pub behavior: String,
    /// Optional Gemini-specific safety settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gemini: Option<GeminiSafetyConfigToml>,
}

fn default_safety_behavior() -> String {
    "return-partial".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfigFile {
    /// Agent configuration.
    pub agent: AgentConfig,
    /// Optional persona configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona: Option<PersonaConfigToml>,
    /// Optional model configuration.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub model: Option<ModelConfigToml>,
    /// Optional safety configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety: Option<SafetyConfigToml>,
}

impl AgentConfigFile {
    /// Extract Gemini safety settings from this agent configuration.
    ///
    /// Returns `Ok(None)` if no Gemini safety settings are configured.
    /// Returns `Ok(Some(settings))` if safety settings are successfully converted.
    /// Returns `Err` if safety settings are configured but conversion fails.
    pub fn gemini_safety_settings(&self) -> Result<Option<Vec<radium_models::GeminiSafetySetting>>> {
        if let Some(ref safety) = self.safety {
            if let Some(ref gemini) = safety.gemini {
                let settings = gemini.to_safety_settings()?;
                if settings.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(settings))
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Load agent configuration from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or parsed.
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        let mut config: Self = toml::from_str(&content)?;
        
        // Set file_path before validation so validate_prompt_path can resolve relative paths
        config.agent.file_path = Some(path.to_path_buf());
        
        config.validate()?;
        
        // Convert persona TOML to PersonaConfig if present
        if let Some(ref persona_toml) = config.persona {
            config.agent.persona_config = Some(config.parse_persona_config(persona_toml)?);
        }
        
        // Parse safety configuration if present
        if let Some(ref safety_toml) = config.safety {
            config.agent.safety_behavior = Some(match safety_toml.behavior.as_str() {
                "return-partial" => SafetyBlockBehavior::ReturnPartial,
                "error" => SafetyBlockBehavior::ThrowError,
                "log" => SafetyBlockBehavior::LogWarning,
                _ => {
                    return Err(AgentConfigError::Invalid(format!(
                        "Invalid safety.behavior '{}'. Must be 'return-partial', 'error', or 'log'",
                        safety_toml.behavior
                    )));
                }
            });
        }
        
        Ok(config)
    }

    /// Parses persona configuration from TOML format.
    fn parse_persona_config(&self, persona_toml: &PersonaConfigToml) -> Result<crate::agents::persona::PersonaConfig> {
        use crate::agents::persona::{PerformanceConfig, PerformanceProfile, RecommendedModels, SimpleModelRecommendation};

        let models = if let Some(ref models_toml) = persona_toml.models {
            // Parse primary model (required) - can be "engine:model" or just "model" (uses agent's engine)
            let primary = if models_toml.primary.contains(':') {
                let parts: Vec<&str> = models_toml.primary.split(':').collect();
                if parts.len() != 2 {
                    return Err(AgentConfigError::Invalid(
                        "persona.models.primary must be in format 'engine:model' or 'model'".to_string(),
                    ));
                }
                SimpleModelRecommendation {
                    engine: parts[0].to_string(),
                    model: parts[1].to_string(),
                }
            } else {
                // Use agent's engine if available, otherwise default to gemini
                let engine = self.agent.engine.as_ref()
                    .map(|e| e.clone())
                    .unwrap_or_else(|| "gemini".to_string());
                SimpleModelRecommendation {
                    engine,
                    model: models_toml.primary.clone(),
                }
            };

            // Parse fallback (optional)
            let fallback = models_toml.fallback.as_ref().map(|f| {
                if f.contains(':') {
                    let parts: Vec<&str> = f.split(':').collect();
                    if parts.len() == 2 {
                        SimpleModelRecommendation {
                            engine: parts[0].to_string(),
                            model: parts[1].to_string(),
                        }
                    } else {
                        SimpleModelRecommendation {
                            engine: primary.engine.clone(),
                            model: f.clone(),
                        }
                    }
                } else {
                    SimpleModelRecommendation {
                        engine: primary.engine.clone(),
                        model: f.clone(),
                    }
                }
            });

            // Parse premium (optional)
            let premium = models_toml.premium.as_ref().map(|p| {
                if p.contains(':') {
                    let parts: Vec<&str> = p.split(':').collect();
                    if parts.len() == 2 {
                        SimpleModelRecommendation {
                            engine: parts[0].to_string(),
                            model: parts[1].to_string(),
                        }
                    } else {
                        SimpleModelRecommendation {
                            engine: primary.engine.clone(),
                            model: p.clone(),
                        }
                    }
                } else {
                    SimpleModelRecommendation {
                        engine: primary.engine.clone(),
                        model: p.clone(),
                    }
                }
            });

            RecommendedModels {
                primary,
                fallback,
                premium,
            }
        } else {
            return Err(AgentConfigError::Invalid(
                "persona.models is required when persona section is present".to_string(),
            ));
        };

        let performance = if let Some(ref perf_toml) = persona_toml.performance {
            let profile = match perf_toml.profile.to_lowercase().as_str() {
                "speed" => PerformanceProfile::Speed,
                "balanced" => PerformanceProfile::Balanced,
                "thinking" => PerformanceProfile::Thinking,
                "expert" => PerformanceProfile::Expert,
                _ => {
                    return Err(AgentConfigError::Invalid(format!(
                        "invalid performance profile: {} (must be speed, balanced, thinking, or expert)",
                        perf_toml.profile
                    )));
                }
            };
            PerformanceConfig {
                profile,
                estimated_tokens: perf_toml.estimated_tokens,
            }
        } else {
            PerformanceConfig {
                profile: PerformanceProfile::Balanced,
                estimated_tokens: None,
            }
        };

        Ok(crate::agents::persona::PersonaConfig {
            models,
            performance,
        })
    }

    /// Save agent configuration to a TOML file.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be written.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let content =
            toml::to_string_pretty(self).map_err(|e| AgentConfigError::Invalid(e.to_string()))?;
        std::fs::write(path.as_ref(), content)?;
        Ok(())
    }

    /// Validate configuration.
    fn validate(&self) -> Result<()> {
        // Validate required fields
        if self.agent.id.is_empty() {
            return Err(AgentConfigError::Invalid("agent ID cannot be empty".to_string()));
        }

        // Validate agent ID format (kebab-case: lowercase letters, numbers, hyphens)
        if !self.agent.id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err(AgentConfigError::Invalid(format!(
                "agent ID must be in kebab-case (lowercase letters, numbers, hyphens): '{}'",
                self.agent.id
            )));
        }

        // Agent ID cannot start or end with hyphen
        if self.agent.id.starts_with('-') || self.agent.id.ends_with('-') {
            return Err(AgentConfigError::Invalid(format!(
                "agent ID cannot start or end with hyphen: '{}'",
                self.agent.id
            )));
        }

        if self.agent.name.is_empty() {
            return Err(AgentConfigError::Invalid("agent name cannot be empty".to_string()));
        }

        if self.agent.prompt_path.as_os_str().is_empty() {
            return Err(AgentConfigError::Invalid("prompt path cannot be empty".to_string()));
        }

        // Validate prompt file existence
        self.validate_prompt_path()?;

        // Validate engine if present
        if let Some(engine) = &self.agent.engine {
            self.validate_engine(engine)?;
        }

        // Validate loop behavior if present
        if let Some(loop_behavior) = &self.agent.loop_behavior {
            self.validate_loop_behavior(loop_behavior)?;
        }

        // Validate trigger behavior if present
        if let Some(trigger_behavior) = &self.agent.trigger_behavior {
            self.validate_trigger_behavior(trigger_behavior)?;
        }

        // Validate model configuration if present
        if let Some(model_config) = &self.model {
            self.validate_model_config(model_config)?;
        }

        Ok(())
    }

    /// Validate prompt file path exists and is readable.
    fn validate_prompt_path(&self) -> Result<()> {
        let prompt_path = &self.agent.prompt_path;

        // Check if path is absolute
        if prompt_path.is_absolute() {
            if !prompt_path.exists() {
                return Err(AgentConfigError::Invalid(format!(
                    "prompt file not found: {}",
                    prompt_path.display()
                )));
            }
            if !prompt_path.is_file() {
                return Err(AgentConfigError::Invalid(format!(
                    "prompt path is not a file: {}",
                    prompt_path.display()
                )));
            }
            return Ok(());
        }

        // For relative paths, try to resolve from config file directory
        if let Some(config_dir) = self.agent.file_path.as_ref().and_then(|p| p.parent()) {
            let full_path = config_dir.join(prompt_path);
            if full_path.exists() && full_path.is_file() {
                return Ok(());
            }
        }

        // Try relative to current working directory (workspace root)
        if let Ok(cwd) = std::env::current_dir() {
            let full_path = cwd.join(prompt_path);
            if full_path.exists() && full_path.is_file() {
                return Ok(());
            }
        }

        Err(AgentConfigError::Invalid(format!(
            "prompt file not found: {} (checked relative to config file and workspace root)",
            prompt_path.display()
        )))
    }

    /// Validate engine value.
    fn validate_engine(&self, engine: &str) -> Result<()> {
        // Include "mock" for local/testing agents and CI scenarios.
        const SUPPORTED_ENGINES: &[&str] = &["gemini", "openai", "claude", "codex", "mock"];

        let engine_lower = engine.to_lowercase();
        if !SUPPORTED_ENGINES.contains(&engine_lower.as_str()) {
            return Err(AgentConfigError::Invalid(format!(
                "unsupported engine: '{}'. Supported engines: {}",
                engine,
                SUPPORTED_ENGINES.join(", ")
            )));
        }

        Ok(())
    }

    /// Validate loop behavior configuration.
    fn validate_loop_behavior(&self, loop_behavior: &AgentLoopBehavior) -> Result<()> {
        if loop_behavior.steps == 0 {
            return Err(AgentConfigError::Invalid(
                "loop_behavior.steps must be greater than 0".to_string(),
            ));
        }

        if let Some(max_iterations) = loop_behavior.max_iterations {
            if max_iterations == 0 {
                return Err(AgentConfigError::Invalid(
                    "loop_behavior.max_iterations must be greater than 0 if specified".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate model configuration.
    fn validate_model_config(&self, model_config: &ModelConfigToml) -> Result<()> {
        // Validate top_k range
        if let Some(k) = model_config.top_k {
            if !(1..=100).contains(&k) {
                return Err(AgentConfigError::Invalid(format!(
                    "top_k must be between 1 and 100, got {}",
                    k
                )));
            }
        }

        // Validate frequency_penalty range
        if let Some(p) = model_config.frequency_penalty {
            if !(-2.0..=2.0).contains(&p) {
                return Err(AgentConfigError::Invalid(format!(
                    "frequency_penalty must be between -2.0 and 2.0, got {}",
                    p
                )));
            }
        }

        // Validate presence_penalty range
        if let Some(p) = model_config.presence_penalty {
            if !(-2.0..=2.0).contains(&p) {
                return Err(AgentConfigError::Invalid(format!(
                    "presence_penalty must be between -2.0 and 2.0, got {}",
                    p
                )));
            }
        }

        // Validate response_format if provided
        if let Some(ref format_str) = model_config.response_format {
            Self::parse_response_format(format_str)?;
        }

        Ok(())
    }

    /// Parse response format string into ResponseFormat enum.
    fn parse_response_format(value: &str) -> Result<ResponseFormat> {
        match value.to_lowercase().as_str() {
            "text" => Ok(ResponseFormat::Text),
            "json" => Ok(ResponseFormat::Json),
            schema if schema.starts_with('{') => {
                // Validate JSON schema
                serde_json::from_str::<serde_json::Value>(schema).map_err(|e| {
                    AgentConfigError::Invalid(format!(
                        "Invalid JSON schema in response_format: {}",
                        e
                    ))
                })?;
                Ok(ResponseFormat::JsonSchema(schema.to_string()))
            }
            _ => Err(AgentConfigError::Invalid(format!(
                "Invalid response_format '{}'. Must be 'text', 'json', or a JSON schema",
                value
            ))),
        }
    }

    /// Convert ModelConfigToml to ModelParameters.
    pub fn to_model_parameters(&self) -> Option<ModelParameters> {
        self.model.as_ref().map(|model_config| {
            let response_format = model_config.response_format.as_ref().and_then(|s| {
                Self::parse_response_format(s).ok()
            });

            ModelParameters {
                temperature: None,
                top_p: None,
                max_tokens: None,
                top_k: model_config.top_k,
                frequency_penalty: model_config.frequency_penalty,
                presence_penalty: model_config.presence_penalty,
                response_format,
                stop_sequences: None,
                enable_grounding: None,
                grounding_threshold: None,
                reasoning_effort: None,
            }
        })
    }

    /// Validate trigger behavior configuration.
    fn validate_trigger_behavior(&self, trigger_behavior: &AgentTriggerBehavior) -> Result<()> {
        if let Some(trigger_agent_id) = &trigger_behavior.trigger_agent_id {
            if trigger_agent_id.is_empty() {
                return Err(AgentConfigError::Invalid(
                    "trigger_behavior.trigger_agent_id cannot be empty if specified".to_string(),
                ));
            }

            // Validate agent ID format (kebab-case: lowercase letters, numbers, hyphens)
            if !trigger_agent_id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            {
                return Err(AgentConfigError::Invalid(format!(
                    "trigger_behavior.trigger_agent_id must be a valid agent ID (kebab-case): '{}'",
                    trigger_agent_id
                )));
            }

            // Agent ID cannot start or end with hyphen
            if trigger_agent_id.starts_with('-') || trigger_agent_id.ends_with('-') {
                return Err(AgentConfigError::Invalid(format!(
                    "trigger_behavior.trigger_agent_id cannot start or end with hyphen: '{}'",
                    trigger_agent_id
                )));
            }
        }

        Ok(())
    }
}

/// Agent configuration section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique agent identifier (e.g., "arch-agent", "plan-agent").
    pub id: String,

    /// Human-readable agent name (e.g., "Architecture Agent").
    pub name: String,

    /// Agent description.
    pub description: String,

    /// Path to the prompt template file (markdown).
    ///
    /// Can be absolute or relative to the workspace root.
    pub prompt_path: PathBuf,

    /// Optional mirror path for RAD-agents.
    ///
    /// This is used when agents are mirrored from another location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_path: Option<PathBuf>,

    /// Default engine for this agent (optional).
    ///
    /// Examples: "gemini", "openai", "claude", "codex"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,

    /// Default model for this agent (optional).
    ///
    /// Examples: "gemini-2.0-flash-exp", "gpt-4", "claude-3-opus-20240229"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Default reasoning effort level (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,

    /// Optional loop behavior configuration.
    ///
    /// When set, this agent can request looping back to previous steps
    /// during workflow execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loop_behavior: Option<AgentLoopBehavior>,

    /// Optional trigger behavior configuration.
    ///
    /// When set, this agent can dynamically trigger other agents
    /// during workflow execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_behavior: Option<AgentTriggerBehavior>,

    /// Agent category (e.g., "my-agents", "rad-agents/design").
    ///
    /// This is typically derived from the file path, not stored in the TOML.
    #[serde(skip)]
    pub category: Option<String>,

    /// File path where this config was loaded from.
    ///
    /// This is not stored in the TOML, but set during loading.
    #[serde(skip)]
    pub file_path: Option<PathBuf>,

    /// Agent capabilities for dynamic selection.
    ///
    /// Defines the agent's model class, cost tier, and concurrency limits.
    /// If not specified, defaults to Balanced/Medium/5.
    #[serde(default)]
    pub capabilities: AgentCapabilities,

    /// Optional sandbox configuration for safe command execution.
    ///
    /// When set, agent commands will execute in the specified sandbox environment.
    /// If not set, commands execute directly without sandboxing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<SandboxConfig>,

    /// Optional persona configuration (loaded from TOML if present).
    ///
    /// This is set when loading from a config file that includes persona settings.
    #[serde(skip)]
    pub persona_config: Option<crate::agents::persona::PersonaConfig>,
    
    /// Optional routing configuration for model selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<AgentRoutingConfig>,
    
    /// Safety block behavior configuration.
    ///
    /// Determines how to handle content that is filtered/blocked by safety systems.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_behavior: Option<radium_abstraction::SafetyBlockBehavior>,
    
    /// Enable code execution for this agent.
    ///
    /// When `None`, uses model config or provider default (true for Gemini, false for others).
    /// When `Some(true)`, enables code execution.
    /// When `Some(false)`, disables code execution.
    ///
    /// Configuration precedence: Agent config > Model config > Provider default
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_execution_enabled: Option<bool>,
}

impl AgentConfig {
    /// Create a new agent configuration.
    pub fn new(id: impl Into<String>, name: impl Into<String>, prompt_path: PathBuf) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            prompt_path,
            mirror_path: None,
            engine: None,
            model: None,
            
            reasoning_effort: None,
            loop_behavior: None,
            trigger_behavior: None,
            category: None,
            file_path: None,
            capabilities: AgentCapabilities::default(),
            sandbox: None,
            persona_config: None,
            routing: None,
            safety_behavior: None,
            code_execution_enabled: None,
        }
    }

    /// Set the description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the default engine.
    #[must_use]
    pub fn with_engine(mut self, engine: impl Into<String>) -> Self {
        self.engine = Some(engine.into());
        self
    }

    /// Set the default model.
    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the reasoning effort level.
    #[must_use]
    pub fn with_reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(effort);
        self
    }

    /// Set the category.
    #[must_use]
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Set the file path.
    #[must_use]
    pub fn with_file_path(mut self, path: PathBuf) -> Self {
        self.file_path = Some(path);
        self
    }

    /// Set the loop behavior configuration.
    #[must_use]
    pub fn with_loop_behavior(mut self, config: AgentLoopBehavior) -> Self {
        self.loop_behavior = Some(config);
        self
    }

    /// Set the trigger behavior configuration.
    #[must_use]
    pub fn with_trigger_behavior(mut self, config: AgentTriggerBehavior) -> Self {
        self.trigger_behavior = Some(config);
        self
    }

    /// Set the agent capabilities.
    #[must_use]
    pub fn with_capabilities(mut self, capabilities: AgentCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Set the sandbox configuration.
    #[must_use]
    pub fn with_sandbox(mut self, sandbox: SandboxConfig) -> Self {
        self.sandbox = Some(sandbox);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_new() {
        let config = AgentConfig::new("test-agent", "Test Agent", PathBuf::from("prompts/test.md"));

        assert_eq!(config.id, "test-agent");
        assert_eq!(config.name, "Test Agent");
        assert_eq!(config.prompt_path, PathBuf::from("prompts/test.md"));
    }

    #[test]
    fn test_agent_config_builder() {
        let config = AgentConfig::new("test-agent", "Test Agent", PathBuf::from("prompts/test.md"))
            .with_description("A test agent")
            .with_engine("gemini")
            .with_model("gemini-2.0-flash-exp")
            .with_reasoning_effort(ReasoningEffort::High)
            .with_category("test");

        assert_eq!(config.description, "A test agent");
        assert_eq!(config.engine, Some("gemini".to_string()));
        assert_eq!(config.model, Some("gemini-2.0-flash-exp".to_string()));
        assert_eq!(config.reasoning_effort, Some(ReasoningEffort::High));
        assert_eq!(config.category, Some("test".to_string()));
    }

    #[test]
    fn test_agent_config_load() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("arch-agent.toml");
        let prompts_dir = temp_dir.path().join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        let prompt_path = prompts_dir.join("arch-agent.md");
        fs::write(&prompt_path, "# Architecture Agent").unwrap();

        let toml_content = r#"
[agent]
id = "arch-agent"
name = "Architecture Agent"
description = "Defines system architecture"
prompt_path = "prompts/arch-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"
reasoning_effort = "medium"
"#;

        fs::write(&config_path, toml_content).unwrap();

        let config = AgentConfigFile::load(&config_path).unwrap();
        assert_eq!(config.agent.id, "arch-agent");
        assert_eq!(config.agent.name, "Architecture Agent");
        assert_eq!(config.agent.engine, Some("gemini".to_string()));
        assert_eq!(config.agent.reasoning_effort, Some(ReasoningEffort::Medium));
    }

    #[test]
    fn test_agent_config_save() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test-agent.toml");
        let prompts_dir = temp_dir.path().join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        let prompt_path = prompts_dir.join("test.md");
        fs::write(&prompt_path, "# Test Agent").unwrap();

        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test-agent", "Test Agent", PathBuf::from("prompts/test.md"))
                .with_description("A test agent")
                .with_engine("gemini")
                .with_file_path(config_path.clone()),
            persona: None,
        };

        config.save(&config_path).unwrap();

        let loaded = AgentConfigFile::load(&config_path).unwrap();
        assert_eq!(loaded.agent.id, config.agent.id);
        assert_eq!(loaded.agent.name, config.agent.name);
        assert_eq!(loaded.agent.engine, config.agent.engine);
    }

    #[test]
    fn test_agent_config_minimal() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("minimal.toml");
        let prompts_dir = temp_dir.path().join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        let prompt_path = prompts_dir.join("minimal.md");
        fs::write(&prompt_path, "# Minimal Agent").unwrap();

        let toml_content = r#"
[agent]
id = "minimal"
name = "Minimal Agent"
description = "Minimal config"
prompt_path = "prompts/minimal.md"
"#;

        fs::write(&config_path, toml_content).unwrap();

        let config = AgentConfigFile::load(&config_path).unwrap();
        assert_eq!(config.agent.id, "minimal");
        assert_eq!(config.agent.engine, None);
        assert_eq!(config.agent.model, None);
        assert_eq!(config.agent.reasoning_effort, None);
    }

    #[test]
    fn test_reasoning_effort_display() {
        assert_eq!(ReasoningEffort::Low.to_string(), "low");
        assert_eq!(ReasoningEffort::Medium.to_string(), "medium");
        assert_eq!(ReasoningEffort::High.to_string(), "high");
    }

    #[test]
    fn test_reasoning_effort_default() {
        assert_eq!(ReasoningEffort::default(), ReasoningEffort::Medium);
    }

    #[test]
    fn test_model_class_display() {
        assert_eq!(ModelClass::Fast.to_string(), "fast");
        assert_eq!(ModelClass::Balanced.to_string(), "balanced");
        assert_eq!(ModelClass::Reasoning.to_string(), "reasoning");
    }

    #[test]
    fn test_cost_tier_display() {
        assert_eq!(CostTier::Low.to_string(), "low");
        assert_eq!(CostTier::Medium.to_string(), "medium");
        assert_eq!(CostTier::High.to_string(), "high");
    }

    #[test]
    fn test_agent_capabilities_default() {
        let capabilities = AgentCapabilities::default();
        assert_eq!(capabilities.model_class, ModelClass::Balanced);
        assert_eq!(capabilities.cost_tier, CostTier::Medium);
        assert_eq!(capabilities.max_concurrent_tasks, 5);
    }

    #[test]
    fn test_agent_config_with_capabilities() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test-agent.toml");
        let prompts_dir = temp_dir.path().join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        let prompt_path = prompts_dir.join("test.md");
        fs::write(&prompt_path, "# Test Agent").unwrap();

        let toml_content = r#"
[agent]
id = "test-agent"
name = "Test Agent"
description = "Test agent with capabilities"
prompt_path = "prompts/test.md"

[agent.capabilities]
model_class = "fast"
cost_tier = "low"
max_concurrent_tasks = 10
"#;

        fs::write(&config_path, toml_content).unwrap();

        let config = AgentConfigFile::load(&config_path).unwrap();
        assert_eq!(config.agent.id, "test-agent");
        assert_eq!(config.agent.capabilities.model_class, ModelClass::Fast);
        assert_eq!(config.agent.capabilities.cost_tier, CostTier::Low);
        assert_eq!(config.agent.capabilities.max_concurrent_tasks, 10);
    }

    #[test]
    fn test_agent_config_capabilities_defaults() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test-agent.toml");
        let prompts_dir = temp_dir.path().join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        let prompt_path = prompts_dir.join("test.md");
        fs::write(&prompt_path, "# Test Agent").unwrap();

        let toml_content = r#"
[agent]
id = "test-agent"
name = "Test Agent"
description = "Test agent without capabilities"
prompt_path = "prompts/test.md"
"#;

        fs::write(&config_path, toml_content).unwrap();

        let config = AgentConfigFile::load(&config_path).unwrap();
        assert_eq!(config.agent.id, "test-agent");
        // Should use defaults
        assert_eq!(config.agent.capabilities.model_class, ModelClass::Balanced);
        assert_eq!(config.agent.capabilities.cost_tier, CostTier::Medium);
        assert_eq!(config.agent.capabilities.max_concurrent_tasks, 5);
    }

    #[test]
    fn test_agent_config_with_loop_behavior() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test-agent.toml");
        let prompts_dir = temp_dir.path().join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        let prompt_path = prompts_dir.join("test.md");
        fs::write(&prompt_path, "# Test Agent").unwrap();

        let toml_content = r#"
[agent]
id = "test-agent"
name = "Test Agent"
description = "Test agent with loop behavior"
prompt_path = "prompts/test.md"

[agent.loop_behavior]
steps = 2
max_iterations = 5
skip = ["step-1", "step-3"]
"#;

        fs::write(&config_path, toml_content).unwrap();

        let config = AgentConfigFile::load(&config_path).unwrap();
        assert_eq!(config.agent.id, "test-agent");
        assert!(config.agent.loop_behavior.is_some());

        let loop_behavior = config.agent.loop_behavior.unwrap();
        assert_eq!(loop_behavior.steps, 2);
        assert_eq!(loop_behavior.max_iterations, Some(5));
        assert_eq!(loop_behavior.skip, vec!["step-1", "step-3"]);
    }

    #[test]
    fn test_agent_config_with_trigger_behavior() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test-agent.toml");
        let prompts_dir = temp_dir.path().join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        let prompt_path = prompts_dir.join("test.md");
        fs::write(&prompt_path, "# Test Agent").unwrap();

        let toml_content = r#"
[agent]
id = "test-agent"
name = "Test Agent"
description = "Test agent with trigger behavior"
prompt_path = "prompts/test.md"

[agent.trigger_behavior]
trigger_agent_id = "fallback-agent"
"#;

        fs::write(&config_path, toml_content).unwrap();

        let config = AgentConfigFile::load(&config_path).unwrap();
        assert_eq!(config.agent.id, "test-agent");
        assert!(config.agent.trigger_behavior.is_some());

        let trigger_behavior = config.agent.trigger_behavior.unwrap();
        assert_eq!(trigger_behavior.trigger_agent_id, Some("fallback-agent".to_string()));
    }

    #[test]
    fn test_agent_config_with_both_behaviors() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test-agent.toml");
        let prompts_dir = temp_dir.path().join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        let prompt_path = prompts_dir.join("test.md");
        fs::write(&prompt_path, "# Test Agent").unwrap();

        let toml_content = r#"
[agent]
id = "test-agent"
name = "Test Agent"
description = "Test agent with both behaviors"
prompt_path = "prompts/test.md"

[agent.loop_behavior]
steps = 3
max_iterations = 10

[agent.trigger_behavior]
trigger_agent_id = "helper-agent"
"#;

        fs::write(&config_path, toml_content).unwrap();

        let config = AgentConfigFile::load(&config_path).unwrap();
        assert!(config.agent.loop_behavior.is_some());
        assert!(config.agent.trigger_behavior.is_some());
    }

    #[test]
    fn test_validate_agent_id_format() {
        use std::fs;

        // Valid IDs
        let valid_ids = vec!["arch-agent", "test-agent-123", "my-agent"];
        for id in valid_ids {
            let temp_dir = tempfile::tempdir().unwrap();
            let prompt_path = temp_dir.path().join("test.md");
            fs::write(&prompt_path, "# Test").unwrap();
            let config_path = temp_dir.path().join("test.toml");

            let config = AgentConfigFile {
            model: None,
            safety: None,
                agent: AgentConfig::new(id, "Test", prompt_path.clone())
                    .with_file_path(config_path),
            persona: None,
            };
            assert!(config.validate().is_ok(), "ID '{}' should be valid", id);
        }

        // Invalid IDs
        let invalid_ids = vec![
            ("agent with spaces", "spaces"),
            ("AgentWithCaps", "uppercase"),
            ("agent-with-", "trailing hyphen"),
            ("-agent", "leading hyphen"),
            ("agent_with_underscore", "underscore"),
        ];
        for (id, reason) in invalid_ids {
            let config = AgentConfigFile {
            model: None,
            safety: None,
                agent: AgentConfig::new(id, "Test", PathBuf::from("prompts/test.md")),
            persona: None,
            };
            assert!(
                config.validate().is_err(),
                "ID '{}' should be invalid ({})",
                id,
                reason
            );
        }
    }

    #[test]
    fn test_validate_engine() {
        use std::fs;

        // Valid engines
        let valid_engines = vec!["gemini", "openai", "claude", "codex"];
        for engine in valid_engines {
            let temp_dir = tempfile::tempdir().unwrap();
            let prompt_path = temp_dir.path().join("test.md");
            fs::write(&prompt_path, "# Test").unwrap();
            let config_path = temp_dir.path().join("test.toml");

        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test-agent", "Test", prompt_path.clone())
                .with_engine(engine)
                .with_file_path(config_path),
            persona: None,
        };
            assert!(
                config.validate().is_ok(),
                "Engine '{}' should be valid",
                engine
            );
        }

        // Invalid engines (note: case-insensitive, so "GEMINI" should be valid)
        let invalid_engines = vec!["invalid", "unknown", "gpt-4"];
        for engine in invalid_engines {
            let temp_dir = tempfile::tempdir().unwrap();
            let prompt_path = temp_dir.path().join("test.md");
            fs::write(&prompt_path, "# Test").unwrap();
            let config_path = temp_dir.path().join("test.toml");

        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test-agent", "Test", prompt_path.clone())
                .with_engine(engine)
                .with_file_path(config_path),
            persona: None,
        };
            assert!(
                config.validate().is_err(),
                "Engine '{}' should be invalid",
                engine
            );
        }
    }

    #[test]
    fn test_validate_loop_behavior() {
        use std::fs;

        // Valid loop behavior
        let temp_dir = tempfile::tempdir().unwrap();
        let prompt_path = temp_dir.path().join("test.md");
        fs::write(&prompt_path, "# Test").unwrap();
        let config_path = temp_dir.path().join("test.toml");

        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test-agent", "Test", prompt_path.clone())
                .with_loop_behavior(AgentLoopBehavior {
                    steps: 2,
                    max_iterations: Some(5),
                    skip: vec![],
                })
                .with_file_path(config_path.clone()),
            persona: None,
        };
        assert!(config.validate().is_ok());

        // Invalid: steps = 0
        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test-agent", "Test", prompt_path.clone())
                .with_loop_behavior(AgentLoopBehavior {
                    steps: 0,
                    max_iterations: None,
                    skip: vec![],
                })
                .with_file_path(config_path.clone()),
            persona: None,
        };
        assert!(config.validate().is_err());

        // Invalid: max_iterations = 0
        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test-agent", "Test", prompt_path.clone())
                .with_loop_behavior(AgentLoopBehavior {
                    steps: 2,
                    max_iterations: Some(0),
                    skip: vec![],
                })
                .with_file_path(config_path),
            persona: None,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_trigger_behavior() {
        use std::fs;

        // Valid trigger behavior
        let temp_dir = tempfile::tempdir().unwrap();
        let prompt_path = temp_dir.path().join("test.md");
        fs::write(&prompt_path, "# Test").unwrap();
        let config_path = temp_dir.path().join("test.toml");

        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test-agent", "Test", prompt_path.clone())
                .with_trigger_behavior(AgentTriggerBehavior {
                    trigger_agent_id: Some("fallback-agent".to_string()),
                })
                .with_file_path(config_path.clone()),
            persona: None,
        };
        assert!(config.validate().is_ok());

        // Invalid: empty trigger_agent_id
        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test-agent", "Test", prompt_path.clone())
                .with_trigger_behavior(AgentTriggerBehavior {
                    trigger_agent_id: Some("".to_string()),
                })
                .with_file_path(config_path.clone()),
            persona: None,
        };
        assert!(config.validate().is_err());

        // Invalid: trigger_agent_id with invalid format
        let invalid_ids = vec!["agent with spaces", "AgentWithCaps", "-agent", "agent-"];
        for invalid_id in invalid_ids {
            let test_config_path = config_path.clone();
            let config = AgentConfigFile {
            model: None,
            safety: None,
                agent: AgentConfig::new("test-agent", "Test", prompt_path.clone())
                    .with_trigger_behavior(AgentTriggerBehavior {
                        trigger_agent_id: Some(invalid_id.to_string()),
                    })
                    .with_file_path(test_config_path),
                persona: None,
            };
            assert!(
                config.validate().is_err(),
                "Trigger agent ID '{}' should be invalid",
                invalid_id
            );
        }
    }

    #[test]
    fn test_validate_prompt_path() {
        use std::fs;

        // Create a temporary directory structure
        let temp_dir = tempfile::tempdir().unwrap();
        let config_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&config_dir).unwrap();

        let prompt_dir = temp_dir.path().join("prompts");
        fs::create_dir_all(&prompt_dir).unwrap();
        let prompt_file = prompt_dir.join("test.md");
        fs::write(&prompt_file, "# Test").unwrap();

        // Valid: relative path from config directory
        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test-agent", "Test", PathBuf::from("../prompts/test.md"))
                .with_file_path(config_dir.join("test-agent.toml")),
        persona: None,
        };
        assert!(config.validate().is_ok());

        // Valid: absolute path
        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test-agent", "Test", prompt_file.clone())
                .with_file_path(config_dir.join("test-agent.toml")),
        persona: None,
        };
        assert!(config.validate().is_ok());

        // Invalid: non-existent file
        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test-agent", "Test", PathBuf::from("nonexistent.md"))
                .with_file_path(config_dir.join("test-agent.toml")),
        persona: None,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_reject_config_with_missing_required_fields() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let config_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&config_dir).unwrap();

        // Test: Empty agent ID
        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig {
                id: String::new(),
                name: "Test Agent".to_string(),
                safety_behavior: None,
                prompt_path: PathBuf::from("test.md"),
                description: String::new(),
                mirror_path: None,
                engine: None,
                model: None,
            
                reasoning_effort: None,
                loop_behavior: None,
                trigger_behavior: None,
                category: None,
                file_path: None,
                capabilities: AgentCapabilities::default(),
                sandbox: None,
                persona_config: None,
                routing: None,
                code_execution_enabled: None,
            }
            .with_file_path(config_dir.join("empty-id.toml")),
            persona: None,
        };
        assert!(config.validate().is_err());

        // Test: Empty agent name
        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig {
                id: "test-agent".to_string(),
                name: String::new(),
                prompt_path: PathBuf::from("test.md"),
                description: String::new(),
                mirror_path: None,
                engine: None,
                model: None,
            
                reasoning_effort: None,
                loop_behavior: None,
                trigger_behavior: None,
                category: None,
                file_path: None,
                capabilities: AgentCapabilities::default(),
                sandbox: None,
                persona_config: None,
                routing: None,
                safety_behavior: None,
                code_execution_enabled: None,
            }
            .with_file_path(config_dir.join("empty-name.toml")),
            persona: None,
        };
        assert!(config.validate().is_err());

        // Test: Empty prompt path
        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig {
                id: "test-agent".to_string(),
                name: "Test Agent".to_string(),
                prompt_path: PathBuf::new(),
                description: String::new(),
                mirror_path: None,
                engine: None,
                model: None,
            
                reasoning_effort: None,
                loop_behavior: None,
                trigger_behavior: None,
                category: None,
                file_path: None,
                capabilities: AgentCapabilities::default(),
                sandbox: None,
                persona_config: None,
                routing: None,
                safety_behavior: None,
                code_execution_enabled: None,
            }
            .with_file_path(config_dir.join("empty-prompt.toml")),
            persona: None,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_reasoning_effort_enum() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test.toml");
        let prompts_dir = temp_dir.path().join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        let prompt_path = prompts_dir.join("test.md");
        fs::write(&prompt_path, "# Test").unwrap();

        // Invalid reasoning_effort value
        let toml_content = r#"
[agent]
id = "test-agent"
name = "Test Agent"
description = "Test"
prompt_path = "prompts/test.md"
reasoning_effort = "invalid"
"#;

        fs::write(&config_path, toml_content).unwrap();
        assert!(AgentConfigFile::load(&config_path).is_err());
    }

    #[test]
    fn test_agent_config_load_file_not_found() {
        let result = AgentConfigFile::load("/nonexistent/path/agent.toml");
        assert!(result.is_err());
        match result.unwrap_err() {
            AgentConfigError::Io(_) => {}
            _ => panic!("Expected I/O error for missing file"),
        }
    }

    #[test]
    fn test_agent_config_load_invalid_toml() {
        use std::fs;
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("invalid.toml");
        
        // Write invalid TOML
        fs::write(&config_path, "invalid toml content {").unwrap();
        
        let result = AgentConfigFile::load(&config_path);
        assert!(result.is_err());
        match result.unwrap_err() {
            AgentConfigError::Toml(_) => {}
            _ => panic!("Expected TOML parse error"),
        }
    }

    #[test]
    fn test_agent_config_save_permission_error() {
        use std::fs;
        #[cfg(unix)]
        use std::os::unix::fs::PermissionsExt;
        
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("readonly.toml");
        
        // Create a read-only file
        fs::write(&config_path, "[agent]\nid = \"test\"").unwrap();
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&config_path).unwrap().permissions();
            perms.set_mode(0o444); // Read-only
            fs::set_permissions(&config_path, perms).unwrap();
        }
        
        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test", "Test", PathBuf::from("test.md")),
            persona: None,
        };
        
        // On Unix, this should fail with permission error
        #[cfg(unix)]
        {
            let result = config.save(&config_path);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_agent_config_load_corrupted_file() {
        use std::fs;
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("corrupted.toml");
        
        // Write file with null bytes (corrupted)
        fs::write(&config_path, b"[\x00agent]\nid = \"test\"").unwrap();
        
        let result = AgentConfigFile::load(&config_path);
        // Should fail to parse
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_config_save_to_nonexistent_directory() {
        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test", "Test", PathBuf::from("test.md")),
            persona: None,
        };
        
        // Try to save to a path in a nonexistent directory
        let result = config.save("/nonexistent/dir/agent.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_model_class_enum() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test.toml");
        let prompts_dir = temp_dir.path().join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        let prompt_path = prompts_dir.join("test.md");
        fs::write(&prompt_path, "# Test").unwrap();

        // Invalid model_class value
        let toml_content = r#"
[agent]
id = "test-agent"
name = "Test Agent"
description = "Test"
prompt_path = "prompts/test.md"

[agent.capabilities]
model_class = "invalid"
cost_tier = "low"
"#;

        fs::write(&config_path, toml_content).unwrap();
        assert!(AgentConfigFile::load(&config_path).is_err());
    }

    #[test]
    fn test_invalid_cost_tier_enum() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test.toml");
        let prompts_dir = temp_dir.path().join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        let prompt_path = prompts_dir.join("test.md");
        fs::write(&prompt_path, "# Test").unwrap();

        // Invalid cost_tier value
        let toml_content = r#"
[agent]
id = "test-agent"
name = "Test Agent"
description = "Test"
prompt_path = "prompts/test.md"

[agent.capabilities]
model_class = "fast"
cost_tier = "invalid"
"#;

        fs::write(&config_path, toml_content).unwrap();
        assert!(AgentConfigFile::load(&config_path).is_err());
    }

    #[test]
    fn test_capabilities_validation() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let prompt_path = temp_dir.path().join("test.md");
        fs::write(&prompt_path, "# Test").unwrap();
        let config_path = temp_dir.path().join("test.toml");

        // Valid capabilities
        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test-agent", "Test", prompt_path.clone())
                .with_file_path(config_path.clone()),
        persona: None,
        };
        // Default capabilities should be valid
        assert!(config.validate().is_ok());

        // Test with explicit valid capabilities
        let mut agent = AgentConfig::new("test-agent", "Test", prompt_path.clone());
        agent.capabilities = AgentCapabilities {
            model_class: ModelClass::Fast,
            cost_tier: CostTier::Low,
            max_concurrent_tasks: 10,
        };
        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: agent.with_file_path(config_path),
        persona: None,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_agent_id_special_characters() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let prompt_path = temp_dir.path().join("test.md");
        fs::write(&prompt_path, "# Test").unwrap();
        let config_path = temp_dir.path().join("test.toml");

        // Test various invalid characters
        let invalid_ids = vec![
            "agent@123",
            "agent#test",
            "agent$test",
            "agent%test",
            "agent&test",
            "agent*test",
            "agent+test",
            "agent=test",
            "agent.test",
            "agent/test",
            "agent\\test",
        ];

        for invalid_id in invalid_ids {
            let config = AgentConfigFile {
            model: None,
            safety: None,
                agent: AgentConfig::new(invalid_id, "Test", prompt_path.clone())
                    .with_file_path(config_path.clone()),
            persona: None,
            };
            assert!(
                config.validate().is_err(),
                "ID '{}' should be invalid",
                invalid_id
            );
        }
    }

    #[test]
    fn test_empty_skip_list_in_loop_behavior() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let prompt_path = temp_dir.path().join("test.md");
        fs::write(&prompt_path, "# Test").unwrap();
        let config_path = temp_dir.path().join("test.toml");

        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test-agent", "Test", prompt_path.clone())
                .with_loop_behavior(AgentLoopBehavior {
                    steps: 2,
                    max_iterations: Some(5),
                    skip: vec![], // Empty skip list should be valid
                })
                .with_file_path(config_path),
            persona: None,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_parse_response_format() {
        use super::AgentConfigFile;

        // Test text format
        let result = AgentConfigFile::parse_response_format("text").unwrap();
        assert_eq!(result, ResponseFormat::Text);

        // Test json format
        let result = AgentConfigFile::parse_response_format("json").unwrap();
        assert_eq!(result, ResponseFormat::Json);

        // Test JSON schema format
        let valid_schema = "{\"type\":\"object\"}";
        let result = AgentConfigFile::parse_response_format(valid_schema).unwrap();
        assert!(matches!(result, ResponseFormat::JsonSchema(_)));

        // Test invalid format
        assert!(AgentConfigFile::parse_response_format("xml").is_err());
        assert!(AgentConfigFile::parse_response_format("invalid").is_err());

        // Test invalid JSON schema
        assert!(AgentConfigFile::parse_response_format("{invalid json}").is_err());
    }

    #[test]
    fn test_model_config_validation() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let prompt_path = temp_dir.path().join("test.md");
        fs::write(&prompt_path, "# Test").unwrap();
        let config_path = temp_dir.path().join("test.toml");

        // Valid top_k
        let mut config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test-agent", "Test", prompt_path.clone())
                .with_file_path(config_path.clone()),
            persona: None,
        };
        config.model = Some(ModelConfigToml {
            top_k: Some(50),
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
        });
        assert!(config.validate().is_ok());

        // Invalid top_k (out of range)
        config.model = Some(ModelConfigToml {
            top_k: Some(101),
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
        });
        assert!(config.validate().is_err());

        // Invalid frequency_penalty (out of range)
        config.model = Some(ModelConfigToml {
            top_k: None,
            frequency_penalty: Some(3.0),
            presence_penalty: None,
            response_format: None,
        });
        assert!(config.validate().is_err());

        // Valid penalties
        config.model = Some(ModelConfigToml {
            top_k: Some(40),
            frequency_penalty: Some(0.5),
            presence_penalty: Some(0.3),
            response_format: Some("json".to_string()),
        });
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_to_model_parameters() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let prompt_path = temp_dir.path().join("test.md");
        fs::write(&prompt_path, "# Test").unwrap();
        let config_path = temp_dir.path().join("test.toml");

        let mut config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test-agent", "Test", prompt_path)
                .with_file_path(config_path),
            persona: None,
        };

        // Test without model config
        assert!(config.to_model_parameters().is_none());

        // Test with model config
        config.model = Some(ModelConfigToml {
            top_k: Some(40),
            frequency_penalty: Some(0.5),
            presence_penalty: Some(0.3),
            response_format: Some("json".to_string()),
        });

        let params = config.to_model_parameters().unwrap();
        assert_eq!(params.top_k, Some(40));
        assert_eq!(params.frequency_penalty, Some(0.5));
        assert_eq!(params.presence_penalty, Some(0.3));
        assert_eq!(params.response_format, Some(ResponseFormat::Json));
    }

    #[test]
    fn test_parse_threshold_valid() {
        use radium_models::{SafetyThreshold};

        // Test all valid threshold values
        assert_eq!(
            GeminiSafetyConfigToml::parse_threshold("BLOCK_NONE").unwrap(),
            SafetyThreshold::BlockNone
        );
        assert_eq!(
            GeminiSafetyConfigToml::parse_threshold("BLOCK_LOW_AND_ABOVE").unwrap(),
            SafetyThreshold::BlockLowAndAbove
        );
        assert_eq!(
            GeminiSafetyConfigToml::parse_threshold("BLOCK_MEDIUM_AND_ABOVE").unwrap(),
            SafetyThreshold::BlockMediumAndAbove
        );
        assert_eq!(
            GeminiSafetyConfigToml::parse_threshold("BLOCK_ONLY_HIGH").unwrap(),
            SafetyThreshold::BlockOnlyHigh
        );
    }

    #[test]
    fn test_parse_threshold_invalid() {
        // Test invalid threshold values
        assert!(GeminiSafetyConfigToml::parse_threshold("INVALID").is_err());
        assert!(GeminiSafetyConfigToml::parse_threshold("").is_err());
        assert!(GeminiSafetyConfigToml::parse_threshold("block_none").is_err()); // case sensitive
        assert!(GeminiSafetyConfigToml::parse_threshold("BLOCK_LOW").is_err()); // partial match
    }

    #[test]
    fn test_to_safety_settings_empty() {
        // Test empty configuration (all None)
        let config = GeminiSafetyConfigToml::default();
        let settings = config.to_safety_settings().unwrap();
        assert!(settings.is_empty());
    }

    #[test]
    fn test_to_safety_settings_with_default() {
        use radium_models::{SafetyCategory, SafetyThreshold};

        // Test default threshold applies to all categories
        let config = GeminiSafetyConfigToml {
            hate_speech: None,
            harassment: None,
            sexually_explicit: None,
            dangerous_content: None,
            civic_integrity: None,
            default: Some("BLOCK_MEDIUM_AND_ABOVE".to_string()),
        };

        let settings = config.to_safety_settings().unwrap();
        assert_eq!(settings.len(), 5); // All 5 categories should use default

        // Verify all categories are present with default threshold
        for setting in &settings {
            assert_eq!(setting.threshold, SafetyThreshold::BlockMediumAndAbove);
        }

        // Verify all categories are included
        let categories: Vec<SafetyCategory> = settings.iter().map(|s| s.category).collect();
        assert!(categories.contains(&SafetyCategory::HateSpeech));
        assert!(categories.contains(&SafetyCategory::Harassment));
        assert!(categories.contains(&SafetyCategory::SexuallyExplicit));
        assert!(categories.contains(&SafetyCategory::DangerousContent));
        assert!(categories.contains(&SafetyCategory::CivicIntegrity));
    }

    #[test]
    fn test_to_safety_settings_partial() {
        use radium_models::{SafetyCategory, SafetyThreshold};

        // Test partial configuration with default fallback
        let config = GeminiSafetyConfigToml {
            hate_speech: Some("INVALID_THRESHOLD".to_string()), // Invalid - should error
            harassment: Some("BLOCK_LOW_AND_ABOVE".to_string()),
            sexually_explicit: None,
            dangerous_content: None,
            civic_integrity: None,
            default: Some("BLOCK_MEDIUM_AND_ABOVE".to_string()),
        };

        // First test should error due to invalid threshold
        assert!(config.to_safety_settings().is_err());

        // Fix the invalid threshold
        let config = GeminiSafetyConfigToml {
            hate_speech: Some("BLOCK_ONLY_HIGH".to_string()),
            harassment: Some("BLOCK_LOW_AND_ABOVE".to_string()),
            sexually_explicit: None,
            dangerous_content: None,
            civic_integrity: None,
            default: Some("BLOCK_MEDIUM_AND_ABOVE".to_string()),
        };

        let settings = config.to_safety_settings().unwrap();
        assert_eq!(settings.len(), 5); // All 5 categories

        // Verify explicit values override default
        let hate_speech_setting = settings
            .iter()
            .find(|s| s.category == SafetyCategory::HateSpeech)
            .unwrap();
        assert_eq!(hate_speech_setting.threshold, SafetyThreshold::BlockOnlyHigh);

        let harassment_setting = settings
            .iter()
            .find(|s| s.category == SafetyCategory::Harassment)
            .unwrap();
        assert_eq!(harassment_setting.threshold, SafetyThreshold::BlockLowAndAbove);

        // Verify default applies to unspecified categories
        let sexually_explicit_setting = settings
            .iter()
            .find(|s| s.category == SafetyCategory::SexuallyExplicit)
            .unwrap();
        assert_eq!(sexually_explicit_setting.threshold, SafetyThreshold::BlockMediumAndAbove);
    }

    #[test]
    fn test_to_safety_settings_all_categories() {
        use radium_models::{SafetyCategory, SafetyThreshold};

        // Test all categories explicitly configured
        let config = GeminiSafetyConfigToml {
            hate_speech: Some("BLOCK_MEDIUM_AND_ABOVE".to_string()),
            harassment: Some("BLOCK_LOW_AND_ABOVE".to_string()),
            sexually_explicit: Some("BLOCK_MEDIUM_AND_ABOVE".to_string()),
            dangerous_content: Some("BLOCK_ONLY_HIGH".to_string()),
            civic_integrity: Some("BLOCK_NONE".to_string()),
            default: None,
        };

        let settings = config.to_safety_settings().unwrap();
        assert_eq!(settings.len(), 5);

        // Verify each category has correct threshold
        for setting in &settings {
            match setting.category {
                SafetyCategory::HateSpeech => {
                    assert_eq!(setting.threshold, SafetyThreshold::BlockMediumAndAbove);
                }
                SafetyCategory::Harassment => {
                    assert_eq!(setting.threshold, SafetyThreshold::BlockLowAndAbove);
                }
                SafetyCategory::SexuallyExplicit => {
                    assert_eq!(setting.threshold, SafetyThreshold::BlockMediumAndAbove);
                }
                SafetyCategory::DangerousContent => {
                    assert_eq!(setting.threshold, SafetyThreshold::BlockOnlyHigh);
                }
                SafetyCategory::CivicIntegrity => {
                    assert_eq!(setting.threshold, SafetyThreshold::BlockNone);
                }
            }
        }
    }

    #[test]
    fn test_to_safety_settings_selective() {
        use radium_models::{SafetyCategory, SafetyThreshold};

        // Test selective configuration (some categories, no default)
        let config = GeminiSafetyConfigToml {
            hate_speech: Some("BLOCK_MEDIUM_AND_ABOVE".to_string()),
            harassment: Some("BLOCK_LOW_AND_ABOVE".to_string()),
            sexually_explicit: None,
            dangerous_content: None,
            civic_integrity: None,
            default: None,
        };

        let settings = config.to_safety_settings().unwrap();
        assert_eq!(settings.len(), 2); // Only configured categories

        let categories: Vec<SafetyCategory> = settings.iter().map(|s| s.category).collect();
        assert!(categories.contains(&SafetyCategory::HateSpeech));
        assert!(categories.contains(&SafetyCategory::Harassment));
        assert!(!categories.contains(&SafetyCategory::SexuallyExplicit));
    }

    #[test]
    fn test_toml_deserialization() {
        // Test TOML deserialization of GeminiSafetyConfigToml
        let toml_str = r#"
hate_speech = "BLOCK_MEDIUM_AND_ABOVE"
harassment = "BLOCK_LOW_AND_ABOVE"
default = "BLOCK_MEDIUM_AND_ABOVE"
"#;

        let config: GeminiSafetyConfigToml = toml::from_str(toml_str).unwrap();
        assert_eq!(config.hate_speech, Some("BLOCK_MEDIUM_AND_ABOVE".to_string()));
        assert_eq!(config.harassment, Some("BLOCK_LOW_AND_ABOVE".to_string()));
        assert_eq!(config.default, Some("BLOCK_MEDIUM_AND_ABOVE".to_string()));
        assert_eq!(config.sexually_explicit, None);
        assert_eq!(config.dangerous_content, None);
        assert_eq!(config.civic_integrity, None);
    }

    #[test]
    fn test_toml_deserialization_partial() {
        // Test TOML deserialization with missing fields (should use defaults)
        let toml_str = r#"
default = "BLOCK_NONE"
"#;

        let config: GeminiSafetyConfigToml = toml::from_str(toml_str).unwrap();
        assert_eq!(config.default, Some("BLOCK_NONE".to_string()));
        assert_eq!(config.hate_speech, None);
        assert_eq!(config.harassment, None);
    }

    #[test]
    fn test_safety_config_toml_with_gemini() {
        // Test SafetyConfigToml with gemini section
        let toml_str = r#"
behavior = "return-partial"

[gemini]
hate_speech = "BLOCK_MEDIUM_AND_ABOVE"
default = "BLOCK_LOW_AND_ABOVE"
"#;

        let config: SafetyConfigToml = toml::from_str(toml_str).unwrap();
        assert_eq!(config.behavior, "return-partial");
        assert!(config.gemini.is_some());

        let gemini_config = config.gemini.unwrap();
        assert_eq!(gemini_config.hate_speech, Some("BLOCK_MEDIUM_AND_ABOVE".to_string()));
        assert_eq!(gemini_config.default, Some("BLOCK_LOW_AND_ABOVE".to_string()));
    }
}

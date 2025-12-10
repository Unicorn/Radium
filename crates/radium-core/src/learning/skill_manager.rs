//! SkillManager for generating skillbook updates from oversight feedback.
//!
//! The SkillManager analyzes oversight responses (from MetacognitiveService)
//! and generates update operations to improve the skillbook over time.

use std::fmt::Write;
use std::sync::Arc;

use radium_abstraction::{ChatMessage, Model, ModelError, ModelParameters};
use thiserror::Error;

use crate::learning::store::LearningStore;
use crate::learning::{UpdateBatch, UpdateOperation, UpdateOperationType};
use crate::oversight::OversightResponse;

/// Errors that can occur during skill management.
#[derive(Error, Debug)]
pub enum SkillManagerError {
    /// Model error during skill curation.
    #[error("Model error: {0}")]
    ModelError(#[from] ModelError),

    /// Failed to parse skill manager response.
    #[error("Failed to parse skill manager response: {0}")]
    ParseError(String),
}

/// Result type for skill manager operations.
pub type Result<T> = std::result::Result<T, SkillManagerError>;

/// SkillManager generates skillbook updates from oversight feedback.
///
/// Analyzes helpful/harmful patterns from OversightResponse and generates
/// UpdateBatch operations to improve the skillbook.
pub struct SkillManager {
    /// The model to use for skill curation.
    model: Arc<dyn Model>,
}

impl SkillManager {
    /// Creates a new skill manager.
    ///
    /// # Arguments
    /// * `model` - The model to use for generating skill updates
    pub fn new(model: Arc<dyn Model>) -> Self {
        Self { model }
    }

    /// Generates skillbook updates from oversight feedback.
    ///
    /// # Arguments
    /// * `oversight_response` - The oversight response containing helpful/harmful patterns
    /// * `learning_store` - The current learning store (for context)
    /// * `question_context` - Description of the task or question
    /// * `progress` - Current progress summary
    ///
    /// # Returns
    /// UpdateBatch with operations to apply to the skillbook
    ///
    /// # Errors
    /// Returns error if model call fails or response cannot be parsed
    pub async fn generate_updates(
        &self,
        oversight_response: &OversightResponse,
        learning_store: &LearningStore,
        question_context: &str,
        progress: &str,
    ) -> Result<UpdateBatch> {
        // Build prompt for skill curation
        let prompt =
            Self::build_prompt(oversight_response, learning_store, question_context, progress);

        // Create messages for chat completion
        let messages = vec![
            ChatMessage { role: "system".to_string(), content: Self::build_system_prompt().into() },
            ChatMessage { role: "user".to_string(), content: prompt.into() },
        ];

        // Call model with structured output request
        let parameters = ModelParameters {
            temperature: Some(0.3),
            top_p: Some(0.9),
            max_tokens: Some(2048),
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
            stop_sequences: None,
            enable_grounding: None,
            grounding_threshold: None,
            reasoning_effort: None,
        };

        let response = self
            .model
            .generate_chat_completion(&messages, Some(parameters))
            .await
            .map_err(SkillManagerError::ModelError)?;

        // Parse response into UpdateBatch
        Self::parse_response(&response.content)
    }

    /// Builds the system prompt for skill curation.
    fn build_system_prompt() -> String {
        "You are a SkillManager that analyzes oversight feedback and generates skillbook updates.

Your role is to:
1. Extract learnable patterns from oversight feedback
2. Identify helpful strategies that should be added or reinforced
3. Identify harmful patterns that should be removed or discouraged
4. Generate structured update operations (ADD, UPDATE, TAG, REMOVE)

Return your response as JSON with this structure:
{
  \"reasoning\": \"Brief explanation of the updates\",
  \"operations\": [
    {
      \"type\": \"ADD\",
      \"section\": \"task_guidance\",
      \"content\": \"Strategy description\",
      \"skill_id\": null
    },
    {
      \"type\": \"TAG\",
      \"section\": null,
      \"content\": null,
      \"skill_id\": \"skill-00001\",
      \"metadata\": {\"helpful\": 1}
    }
  ]
}

Available sections: task_guidance, tool_usage, error_handling, code_patterns, communication, general
Operation types: ADD (new skill), UPDATE (modify skill), TAG (mark helpful/harmful), REMOVE (soft-delete skill)
For TAG operations, metadata should contain \"helpful\", \"harmful\", or \"neutral\" with increment values."
            .to_string()
    }

    /// Builds the user prompt with context.
    fn build_prompt(
        oversight_response: &OversightResponse,
        learning_store: &LearningStore,
        question_context: &str,
        progress: &str,
    ) -> String {
        let mut prompt = String::new();

        writeln!(prompt, "Question Context: {}", question_context).unwrap();
        writeln!(prompt, "Progress: {}", progress).unwrap();
        writeln!(prompt, "Oversight Advice: {}", oversight_response.advice).unwrap();
        writeln!(prompt, "Risk Score: {:.2}", oversight_response.risk_score).unwrap();

        if !oversight_response.traits.is_empty() {
            writeln!(prompt, "Detected Traits: {}", oversight_response.traits.join(", ")).unwrap();
        }

        if !oversight_response.uncertainties.is_empty() {
            writeln!(prompt, "Uncertainties: {}", oversight_response.uncertainties.join(", "))
                .unwrap();
        }

        // Include current skillbook context (limited to avoid token bloat)
        let skillbook_context = learning_store.as_context(3);
        if !skillbook_context.is_empty() {
            prompt.push_str("\nCurrent Skillbook:\n");
            prompt.push_str(&skillbook_context);
        }

        prompt.push_str(
            "\n\nAnalyze the oversight feedback and generate update operations to improve the skillbook.
Focus on extracting actionable strategies that can help future tasks succeed.",
        );

        prompt
    }

    /// Parses model response into UpdateBatch.
    ///
    /// For now, uses simple JSON parsing. In the future, could use structured output
    /// if the model supports it.
    fn parse_response(content: &str) -> Result<UpdateBatch> {
        // Try to extract JSON from the response
        let json_start = content.find('{');
        let json_end = content.rfind('}');

        let json_str = if let (Some(start), Some(end)) = (json_start, json_end) {
            &content[start..=end]
        } else {
            return Err(SkillManagerError::ParseError("No JSON found in response".to_string()));
        };

        // Parse JSON
        let json: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| SkillManagerError::ParseError(format!("JSON parse error: {}", e)))?;

        let reasoning = json
            .get("reasoning")
            .and_then(|v| v.as_str())
            .unwrap_or("Generated from oversight feedback")
            .to_string();

        let mut batch = UpdateBatch::new(reasoning);

        // Parse operations
        if let Some(ops_array) = json.get("operations").and_then(|v| v.as_array()) {
            for op_json in ops_array {
                if let Ok(operation) = Self::parse_operation(op_json) {
                    batch.add_operation(operation);
                }
            }
        }

        Ok(batch)
    }

    /// Parses a single operation from JSON.
    fn parse_operation(op_json: &serde_json::Value) -> Result<UpdateOperation> {
        let op_type_str = op_json
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillManagerError::ParseError("Missing 'type' field".to_string()))?;

        let op_type = match op_type_str {
            "ADD" => UpdateOperationType::Add,
            "UPDATE" => UpdateOperationType::Update,
            "TAG" => UpdateOperationType::Tag,
            "REMOVE" => UpdateOperationType::Remove,
            _ => {
                return Err(SkillManagerError::ParseError(format!(
                    "Invalid operation type: {}",
                    op_type_str
                )));
            }
        };

        let section =
            op_json.get("section").and_then(|v| v.as_str()).map(std::string::ToString::to_string);
        let content =
            op_json.get("content").and_then(|v| v.as_str()).map(std::string::ToString::to_string);
        let skill_id =
            op_json.get("skill_id").and_then(|v| v.as_str()).map(std::string::ToString::to_string);

        let mut metadata = std::collections::HashMap::new();
        if let Some(meta_obj) = op_json.get("metadata").and_then(|v| v.as_object()) {
            for (key, value) in meta_obj {
                if let Some(num) = value.as_u64() {
                    metadata.insert(key.clone(), num as u32);
                }
            }
        }

        Ok(UpdateOperation { op_type, section, content, skill_id, metadata })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::store::LearningStore;
    use crate::oversight::OversightResponse;
    use tempfile::TempDir;

    #[test]
    fn test_parse_operation_add() {
        let json = serde_json::json!({
            "type": "ADD",
            "section": "task_guidance",
            "content": "Test skill",
            "skill_id": null
        });

        let op = SkillManager::parse_operation(&json).unwrap();
        assert_eq!(op.op_type, UpdateOperationType::Add);
        assert_eq!(op.section.as_deref(), Some("task_guidance"));
        assert_eq!(op.content.as_deref(), Some("Test skill"));
    }

    #[test]
    fn test_parse_operation_tag() {
        let json = serde_json::json!({
            "type": "TAG",
            "section": null,
            "content": null,
            "skill_id": "skill-00001",
            "metadata": {"helpful": 1}
        });

        let op = SkillManager::parse_operation(&json).unwrap();
        assert_eq!(op.op_type, UpdateOperationType::Tag);
        assert_eq!(op.skill_id.as_deref(), Some("skill-00001"));
        assert_eq!(op.metadata.get("helpful"), Some(&1));
    }

    #[test]
    fn test_parse_response() {
        let json_str = r#"{
            "reasoning": "Test reasoning",
            "operations": [
                {
                    "type": "ADD",
                    "section": "general",
                    "content": "Test skill",
                    "skill_id": null
                }
            ]
        }"#;

        let batch = SkillManager::parse_response(json_str).unwrap();
        assert_eq!(batch.reasoning, "Test reasoning");
        assert_eq!(batch.operations.len(), 1);
    }

    #[test]
    fn test_build_prompt() {
        let temp_dir = TempDir::new().unwrap();
        let learning_store = LearningStore::new(temp_dir.path()).unwrap();

        let oversight = OversightResponse::new("Test advice".to_string(), 0.5)
            .with_trait("Complex Solution Bias");

        let prompt = SkillManager::build_prompt(&oversight, &learning_store, "Test context", "50%");
        assert!(prompt.contains("Test context"));
        assert!(prompt.contains("Test advice"));
        assert!(prompt.contains("Complex Solution Bias"));
    }
}

//! AI evaluator for grading orchestrator responses.
//!
//! Uses AI models (Gemini Flash 2.0 by default) to provide structured
//! evaluation of orchestrator quality.

use anyhow::{Context, Result};
use radium_abstraction::{ChatMessage, MessageContent, Model, ModelParameters};
use radium_models::{ModelConfig, ModelFactory, ModelType};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;

/// Evaluation aspect being tested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationAspect {
    /// How well the orchestrator selects and uses tools.
    ToolAppropriateness,
    /// Quality of response (clarity, accuracy, helpfulness).
    ResponseQuality,
    /// Ability to maintain context across multiple turns.
    MultiTurnCoherence,
    /// Graceful handling of errors and failures.
    ErrorRecovery,
    /// Accuracy of file:line references in responses.
    FileReferenceAccuracy,
}

impl EvaluationAspect {
    /// Get a description of this aspect for prompts.
    pub fn description(&self) -> &'static str {
        match self {
            Self::ToolAppropriateness => "Tool Selection and Usage",
            Self::ResponseQuality => "Response Quality",
            Self::MultiTurnCoherence => "Multi-Turn Context Tracking",
            Self::ErrorRecovery => "Error Handling and Recovery",
            Self::FileReferenceAccuracy => "File Reference Accuracy",
        }
    }
}

/// Criteria for evaluating a response.
#[derive(Debug, Clone)]
pub struct EvaluationCriteria {
    /// The aspect being evaluated.
    pub aspect: EvaluationAspect,
    /// Elements that must be present or demonstrated.
    pub required_elements: Vec<String>,
    /// Minimum score required to pass (0-100).
    pub min_score: u8,
}

/// Result of an evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Whether the evaluation passed.
    pub passed: bool,
    /// Numeric score (0-100), if applicable.
    pub score: Option<u8>,
    /// Brief summary of the evaluation.
    pub feedback: String,
    /// Detailed reasoning for the score.
    pub reasoning: String,
}

/// AI evaluator for grading responses.
pub struct AiEvaluator {
    /// The AI model used for evaluation.
    model: Arc<dyn Model + Send + Sync>,
}

impl AiEvaluator {
    /// Create a new evaluator using Gemini Flash 2.0 (cost-effective default).
    ///
    /// # Errors
    /// Returns an error if GEMINI_API_KEY is not set.
    pub fn new_gemini_flash() -> Result<Self> {
        let config = ModelConfig::new(
            ModelType::Gemini,
            "gemini-2.0-flash-exp".to_string(),
        );

        let model = ModelFactory::create(config)
            .context("Failed to create Gemini model - ensure GEMINI_API_KEY is set")?;

        Ok(Self { model })
    }

    /// Create a new evaluator using Claude Sonnet 4.5 (higher quality, more expensive).
    ///
    /// # Errors
    /// Returns an error if ANTHROPIC_API_KEY is not set.
    pub fn new_claude() -> Result<Self> {
        let config = ModelConfig::new(
            ModelType::Claude,
            "claude-sonnet-4-5-20250929".to_string(),
        );

        let model = ModelFactory::create(config)
            .context("Failed to create Claude model - ensure ANTHROPIC_API_KEY is set")?;

        Ok(Self { model })
    }

    /// Create an evaluator based on environment variable.
    ///
    /// Checks EVALUATOR_MODEL environment variable:
    /// - "claude" -> Claude Sonnet 4.5
    /// - anything else (default) -> Gemini Flash 2.0
    ///
    /// # Errors
    /// Returns an error if the required API key is not set.
    pub fn from_env() -> Result<Self> {
        let evaluator_model = env::var("EVALUATOR_MODEL")
            .unwrap_or_else(|_| "gemini".to_string());

        match evaluator_model.as_str() {
            "claude" => Self::new_claude(),
            _ => Self::new_gemini_flash(),
        }
    }

    /// Evaluate a response against criteria.
    ///
    /// # Arguments
    /// * `scenario_description` - Description of the test scenario
    /// * `user_input` - The original user request
    /// * `expected_behavior` - What the orchestrator should do
    /// * `actual_response` - The orchestrator's actual response
    /// * `criteria` - Evaluation criteria
    ///
    /// # Errors
    /// Returns an error if the model call fails or response parsing fails.
    pub async fn evaluate(
        &self,
        scenario_description: &str,
        user_input: &str,
        expected_behavior: &str,
        actual_response: &str,
        criteria: &EvaluationCriteria,
    ) -> Result<EvaluationResult> {
        let prompt = build_evaluation_prompt(
            scenario_description,
            user_input,
            expected_behavior,
            actual_response,
            criteria,
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: MessageContent::text(prompt),
        }];

        // Use low temperature for consistency
        let parameters = Some(ModelParameters {
            temperature: Some(0.2),
            ..Default::default()
        });

        let response = self.model.generate_chat_completion(&messages, parameters)
            .await
            .context("Failed to get evaluation from AI model")?;

        parse_evaluation_response(&response.content)
    }
}

/// Build the evaluation prompt.
fn build_evaluation_prompt(
    scenario_description: &str,
    user_input: &str,
    expected_behavior: &str,
    actual_response: &str,
    criteria: &EvaluationCriteria,
) -> String {
    let required_elements_text = criteria
        .required_elements
        .iter()
        .map(|e| format!("  - {}", e))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"You are an expert evaluator for an AI orchestration system. Your task is to objectively grade the quality of an AI orchestrator's response.

# Evaluation Context

**Scenario**: {}

**User Input**: "{}"

**Expected Behavior**: {}

**Actual Response**:
```
{}
```

# Evaluation Aspect: {}

# Evaluation Criteria

Required Elements:
{}

Minimum Score: {}/100

# Instructions

1. Carefully analyze the actual response against the expected behavior
2. Check if all required elements are present or demonstrated
3. Assign a numeric score from 0-100 based on:
   - Completeness (are all required elements present?)
   - Correctness (is the behavior appropriate?)
   - Quality (is it well-executed?)
4. Determine pass/fail based on minimum score threshold
5. Provide clear, actionable feedback

# Output Format

Respond with ONLY a JSON object (no markdown code blocks, no additional text):

{{
  "passed": true or false,
  "score": 0-100,
  "feedback": "Brief 1-2 sentence summary",
  "reasoning": "Detailed explanation of score, what was done well and what was missing"
}}

Ensure your response is valid JSON that can be parsed directly.
"#,
        scenario_description,
        user_input,
        expected_behavior,
        actual_response,
        criteria.aspect.description(),
        required_elements_text,
        criteria.min_score
    )
}

/// Parse the evaluation response JSON.
fn parse_evaluation_response(response_text: &str) -> Result<EvaluationResult> {
    // Try to extract JSON from the response (in case model wraps it in markdown)
    let json_text = if response_text.trim().starts_with("```") {
        // Extract content between ```json and ``` or ``` and ```
        let lines: Vec<&str> = response_text.lines().collect();
        let start = lines.iter().position(|l| l.trim().starts_with("```")).unwrap_or(0);
        let end = lines.iter().rposition(|l| l.trim() == "```").unwrap_or(lines.len());
        lines[start + 1..end].join("\n")
    } else {
        response_text.to_string()
    };

    serde_json::from_str(&json_text)
        .context("Failed to parse evaluation JSON response")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluation_aspect_description() {
        assert_eq!(
            EvaluationAspect::ToolAppropriateness.description(),
            "Tool Selection and Usage"
        );
    }

    #[test]
    fn test_build_evaluation_prompt() {
        let criteria = EvaluationCriteria {
            aspect: EvaluationAspect::ToolAppropriateness,
            required_elements: vec!["Uses read_file tool".to_string()],
            min_score: 70,
        };

        let prompt = build_evaluation_prompt(
            "Test scenario",
            "Read config.json",
            "Should use read_file tool",
            "I'll read the file...",
            &criteria,
        );

        assert!(prompt.contains("Test scenario"));
        assert!(prompt.contains("Read config.json"));
        assert!(prompt.contains("Uses read_file tool"));
        assert!(prompt.contains("70/100"));
    }

    #[test]
    fn test_parse_evaluation_response() {
        let json = r#"{"passed": true, "score": 85, "feedback": "Good job", "reasoning": "All elements present"}"#;
        let result = parse_evaluation_response(json).unwrap();

        assert!(result.passed);
        assert_eq!(result.score, Some(85));
        assert_eq!(result.feedback, "Good job");
    }

    #[test]
    fn test_parse_evaluation_response_with_markdown() {
        let response = r#"```json
{"passed": false, "score": 45, "feedback": "Missing elements", "reasoning": "Tool not used"}
```"#;
        let result = parse_evaluation_response(response).unwrap();

        assert!(!result.passed);
        assert_eq!(result.score, Some(45));
    }
}

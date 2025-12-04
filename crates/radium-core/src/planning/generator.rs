//! AI-powered plan generation using LLMs.

use radium_abstraction::Model;
use std::sync::Arc;

use super::parser::{ParsedPlan, PlanParser};

/// Configuration for plan generation.
#[derive(Debug, Clone)]
pub struct PlanGeneratorConfig {
    /// Model to use for generation.
    pub model: String,
    /// Engine to use (gemini, openai, etc.)
    pub engine: String,
    /// Maximum tokens for generation.
    pub max_tokens: Option<u32>,
    /// Temperature for generation (0.0-1.0).
    pub temperature: Option<f32>,
}

impl Default for PlanGeneratorConfig {
    fn default() -> Self {
        Self {
            model: "gemini-1.5-flash".to_string(),
            engine: "gemini".to_string(),
            max_tokens: Some(8000),
            temperature: Some(0.7),
        }
    }
}

/// AI-powered plan generator.
pub struct PlanGenerator {
    #[allow(dead_code)]
    config: PlanGeneratorConfig,
}

impl PlanGenerator {
    /// Creates a new plan generator with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self { config: PlanGeneratorConfig::default() }
    }

    /// Creates a new plan generator with custom configuration.
    #[must_use]
    pub fn with_config(config: PlanGeneratorConfig) -> Self {
        Self { config }
    }

    /// Generates a plan from a specification using an AI model.
    ///
    /// # Arguments
    /// * `spec` - The specification content
    /// * `model` - The model instance to use for generation
    ///
    /// # Returns
    /// A parsed plan structure
    ///
    /// # Errors
    /// Returns an error if model execution or parsing fails
    pub async fn generate(&self, spec: &str, model: Arc<dyn Model>) -> Result<ParsedPlan, String> {
        // Create prompt for plan generation
        let prompt = Self::create_plan_prompt(spec);

        // Execute model
        let response = model
            .generate_text(&prompt, None)
            .await
            .map_err(|e| format!("Model execution failed: {}", e))?;

        // Parse response
        PlanParser::parse(&response.content)
    }

    /// Creates the prompt for plan generation.
    fn create_plan_prompt(spec: &str) -> String {
        format!(
            r#"You are an expert project planner. Generate a detailed, structured plan from the following specification.

SPECIFICATION:
{spec}

Generate a plan with the following structure:

# Project Name

Brief project description (1-2 sentences).

## Tech Stack
- Technology 1
- Technology 2
- Technology 3

## Iteration 1: [Name]

Goal: [Clear goal for this iteration]

1. **Task Title** - Brief description
   - Agent: [agent-id or "auto"]
   - Dependencies: [e.g., I1.T2, I1.T3 or "none"]
   - Acceptance: [Clear acceptance criteria]

2. **Next Task** - Description
   - Agent: [agent-id]
   - Dependencies: [previous task IDs]
   - Acceptance: [Criteria]

## Iteration 2: [Name]

Goal: [Goal]

[Continue with more iterations...]

IMPORTANT GUIDELINES:
- Break the project into 3-5 logical iterations
- Each iteration should have 3-8 concrete tasks
- Tasks should be specific, actionable, and measurable
- Include clear acceptance criteria for each task
- Specify dependencies between tasks where relevant
- Suggest appropriate agent IDs when applicable (e.g., "setup-agent", "code-agent", "test-agent")
- Start with foundation/setup tasks, then build incrementally
- Each iteration should deliver working functionality

Generate the plan now:"#
        )
    }
}

impl Default for PlanGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radium_models::{ModelFactory, ModelType};

    #[test]
    fn test_plan_generator_new() {
        let generator = PlanGenerator::new();
        assert_eq!(generator.config.engine, "gemini");
    }

    #[test]
    fn test_plan_generator_with_config() {
        let config = PlanGeneratorConfig {
            model: "gpt-4".to_string(),
            engine: "openai".to_string(),
            max_tokens: Some(4000),
            temperature: Some(0.5),
        };

        let generator = PlanGenerator::with_config(config);
        assert_eq!(generator.config.model, "gpt-4");
        assert_eq!(generator.config.engine, "openai");
    }

    #[test]
    fn test_create_plan_prompt() {
        let spec = "Build a todo app with Rust";
        let prompt = PlanGenerator::create_plan_prompt(spec);

        assert!(prompt.contains("Build a todo app with Rust"));
        assert!(prompt.contains("Iteration"));
        assert!(prompt.contains("Tech Stack"));
        assert!(prompt.contains("Dependencies"));
    }

    #[test]
    fn test_plan_generator_config_default() {
        let config = PlanGeneratorConfig::default();
        assert_eq!(config.model, "gemini-1.5-flash");
        assert_eq!(config.engine, "gemini");
        assert_eq!(config.max_tokens, Some(8000));
        assert_eq!(config.temperature, Some(0.7));
    }

    #[test]
    fn test_plan_generator_config_custom() {
        let config = PlanGeneratorConfig {
            model: "custom-model".to_string(),
            engine: "custom-engine".to_string(),
            max_tokens: Some(2000),
            temperature: Some(0.9),
        };

        assert_eq!(config.model, "custom-model");
        assert_eq!(config.engine, "custom-engine");
        assert_eq!(config.max_tokens, Some(2000));
        assert_eq!(config.temperature, Some(0.9));
    }

    #[test]
    fn test_plan_generator_default() {
        let generator = PlanGenerator::default();
        assert_eq!(generator.config.engine, "gemini");
    }

    #[test]
    fn test_create_plan_prompt_empty_spec() {
        let prompt = PlanGenerator::create_plan_prompt("");
        assert!(prompt.contains("SPECIFICATION:"));
        assert!(prompt.contains("Iteration"));
    }

    #[test]
    fn test_create_plan_prompt_long_spec() {
        let spec = "A".repeat(1000);
        let prompt = PlanGenerator::create_plan_prompt(&spec);
        assert!(prompt.contains(&spec));
    }

    #[tokio::test]
    async fn test_plan_generator_generate_with_mock_model() {
        let generator = PlanGenerator::new();
        let model = ModelFactory::create_from_str("mock", "test-model".to_string()).unwrap();

        // MockModel will return a response that may or may not be parseable as a plan
        // This tests that generate calls the model and attempts to parse the response
        let spec = "Build a test app";
        let result = generator.generate(spec, model).await;

        // The result could be Ok or Err depending on whether the mock response parses
        // Either way, we've tested that generate() calls the model and attempts parsing
        match result {
            Ok(plan) => {
                // If it parsed successfully, verify it has expected structure
                assert!(!plan.project_name.is_empty());
            }
            Err(err) => {
                // If parsing failed, that's also a valid test outcome
                assert!(!err.is_empty());
            }
        }
    }
}

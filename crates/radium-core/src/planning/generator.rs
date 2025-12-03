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
}

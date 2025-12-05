//! Metacognitive LLM service for agent oversight.
//!
//! Provides phase-aware oversight feedback using a second LLM to prevent
//! reasoning lock-in and improve alignment.

use radium_abstraction::{ChatMessage, Model, ModelError, ModelParameters};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

use crate::workflow::behaviors::vibe_check::WorkflowPhase;

/// Errors that can occur during metacognitive oversight.
#[derive(Error, Debug)]
pub enum MetacognitiveError {
    /// Model error during oversight call.
    #[error("Model error: {0}")]
    ModelError(#[from] ModelError),

    /// Missing required context for oversight.
    #[error("Missing required context: {0}")]
    MissingContext(String),

    /// Failed to parse oversight response.
    #[error("Failed to parse oversight response: {0}")]
    ParseError(String),
}

/// Result type for metacognitive operations.
pub type Result<T> = std::result::Result<T, MetacognitiveError>;

/// Request for metacognitive oversight.
#[derive(Debug, Clone)]
pub struct OversightRequest {
    /// Current workflow phase.
    pub phase: WorkflowPhase,
    /// Goal or objective being pursued.
    pub goal: String,
    /// Current plan or approach.
    pub plan: String,
    /// Progress made so far.
    pub progress: Option<String>,
    /// User's original prompt.
    pub user_prompt: Option<String>,
    /// Task context or recent actions.
    pub task_context: Option<String>,
    /// Uncertainties identified.
    pub uncertainties: Vec<String>,
    /// Session ID for history continuity.
    pub session_id: Option<String>,
    /// History summary from previous interactions.
    pub history_summary: Option<String>,
    /// Learning context from past mistakes.
    pub learning_context: Option<String>,
    /// Constitution rules for this session.
    pub constitution_rules: Vec<String>,
}

impl OversightRequest {
    /// Creates a new oversight request.
    pub fn new(phase: WorkflowPhase, goal: String, plan: String) -> Self {
        Self {
            phase,
            goal,
            plan,
            progress: None,
            user_prompt: None,
            task_context: None,
            uncertainties: vec![],
            session_id: None,
            history_summary: None,
            learning_context: None,
            constitution_rules: vec![],
        }
    }

    /// Sets the progress.
    #[must_use]
    pub fn with_progress(mut self, progress: impl Into<String>) -> Self {
        self.progress = Some(progress.into());
        self
    }

    /// Sets the user prompt.
    #[must_use]
    pub fn with_user_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.user_prompt = Some(prompt.into());
        self
    }

    /// Sets the task context.
    #[must_use]
    pub fn with_task_context(mut self, context: impl Into<String>) -> Self {
        self.task_context = Some(context.into());
        self
    }

    /// Adds an uncertainty.
    #[must_use]
    pub fn with_uncertainty(mut self, uncertainty: impl Into<String>) -> Self {
        self.uncertainties.push(uncertainty.into());
        self
    }

    /// Sets the session ID.
    #[must_use]
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Sets the history summary.
    #[must_use]
    pub fn with_history_summary(mut self, summary: impl Into<String>) -> Self {
        self.history_summary = Some(summary.into());
        self
    }

    /// Sets the learning context.
    #[must_use]
    pub fn with_learning_context(mut self, context: impl Into<String>) -> Self {
        self.learning_context = Some(context.into());
        self
    }

    /// Sets the constitution rules.
    #[must_use]
    pub fn with_constitution_rules(mut self, rules: Vec<String>) -> Self {
        self.constitution_rules = rules;
        self
    }
}

/// Response from metacognitive oversight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OversightResponse {
    /// Human-readable advice from oversight LLM.
    pub advice: String,
    /// Risk score (0.0 to 1.0) indicating potential issues.
    pub risk_score: f64,
    /// Detected traits or patterns.
    pub traits: Vec<String>,
    /// Uncertainties identified.
    pub uncertainties: Vec<String>,
}

impl OversightResponse {
    /// Creates a new oversight response.
    pub fn new(advice: String, risk_score: f64) -> Self {
        Self { advice, risk_score, traits: vec![], uncertainties: vec![] }
    }

    /// Adds a trait.
    #[must_use]
    pub fn with_trait(mut self, trait_name: impl Into<String>) -> Self {
        self.traits.push(trait_name.into());
        self
    }

    /// Adds an uncertainty.
    #[must_use]
    pub fn with_uncertainty(mut self, uncertainty: impl Into<String>) -> Self {
        self.uncertainties.push(uncertainty.into());
        self
    }
}

/// Metacognitive service for agent oversight.
pub struct MetacognitiveService {
    /// The model to use for oversight.
    model: Arc<dyn Model>,
}

impl MetacognitiveService {
    /// Creates a new metacognitive service.
    ///
    /// # Arguments
    /// * `model` - The model to use for oversight feedback
    pub fn new(model: Arc<dyn Model>) -> Self {
        Self { model }
    }

    /// Generates phase-aware oversight feedback.
    ///
    /// # Arguments
    /// * `request` - The oversight request with context
    ///
    /// # Returns
    /// `OversightResponse` with advice, risk score, and detected patterns
    ///
    /// # Errors
    /// Returns error if model call fails or response cannot be parsed
    pub async fn generate_oversight(&self, request: &OversightRequest) -> Result<OversightResponse> {
        // Build system prompt based on phase
        let system_prompt = Self::build_system_prompt(&request.phase);

        // Build context section
        let context_section = Self::build_context_section(request);

        // Create messages for chat completion
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: context_section,
            },
        ];

        // Call model with lower temperature for more focused feedback
        let parameters = ModelParameters {
            temperature: Some(0.2),
            top_p: Some(0.9),
            max_tokens: Some(1024),
            stop_sequences: None,
        };

        let response = self
            .model
            .generate_chat_completion(&messages, Some(parameters))
            .await
            .map_err(MetacognitiveError::ModelError)?;

        // Parse response and extract structured data
        Self::parse_response(&response.content)
    }

    /// Builds phase-aware system prompt.
    fn build_system_prompt(phase: &WorkflowPhase) -> String {
        let base_prompt = "You are a meta-mentor. You're an experienced feedback provider that specializes in understanding intent, dysfunctional patterns in AI agents, and in responding in ways that further the goal. You need to carefully reason and process the information provided, to determine your output.\n\nYour tone needs to always be a mix of these traits based on the context of which pushes the message in the most appropriate affect: Gentle & Validating, Unafraid to push many questions but humble enough to step back, Sharp about problems and eager to help about problem-solving & giving tips and/or advice, stern and straightforward when spotting patterns & the agent being stuck in something that could derail things.\n\nHere's what you need to think about (Do not output the full thought process, only what is explicitly requested):\n1. What's going on here? What's the nature of the problem is the agent tackling? What's the approach, situation and goal? Is there any prior context that clarifies context further? \n2. What does the agent need to hear right now: Are there any clear patterns, loops, or unspoken assumptions being missed here? Or is the agent doing fine - in which case should I interrupt it or provide soft encouragement and a few questions? What is the best response I can give right now?\n3. In case the issue is technical - I need to provide guidance and help. In case I spot something that's clearly not accounted for/ assumed/ looping/ or otherwise could be out of alignment with the user or agent stated goals - I need to point out what I see gently and ask questions on if the agent agrees. If I don't see/ can't interpret an explicit issue - what intervention would provide valuable feedback here - questions, guidance, validation, or giving a soft go-ahead with reminders of best practices?\n4. In case the plan looks to be accurate - based on the context, can I remind the agent of how to continue, what not to forget, or should I soften and step back for the agent to continue its work? What's the most helpful thing I can do right now?";

        let phase_guidance = match phase {
            WorkflowPhase::Planning => {
                "\n\nCurrent Phase: PLANNING\nFocus on: alignment with user intent, exploration of alternatives, questioning fundamental assumptions."
            }
            WorkflowPhase::Implementation => {
                "\n\nCurrent Phase: IMPLEMENTATION\nFocus on: consistency with the plan, appropriateness of methods, technical alignment."
            }
            WorkflowPhase::Review => {
                "\n\nCurrent Phase: REVIEW\nFocus on: comprehensiveness, edge cases, verification of outcomes."
            }
        };

        format!("{}{}", base_prompt, phase_guidance)
    }

    /// Builds context section from request.
    fn build_context_section(request: &OversightRequest) -> String {
        let mut context = String::new();

        if let Some(ref history) = request.history_summary {
            context.push_str(&format!("History Context: {}\n", history));
        }

        if let Some(ref learning) = request.learning_context {
            context.push_str(&format!("Learning Context:\n{}\n", learning));
        }

        context.push_str(&format!("Goal: {}\n", request.goal));
        context.push_str(&format!("Plan: {}\n", request.plan));

        if let Some(ref progress) = request.progress {
            context.push_str(&format!("Progress: {}\n", progress));
        } else {
            context.push_str("Progress: None\n");
        }

        if !request.uncertainties.is_empty() {
            context.push_str(&format!("Uncertainties: {}\n", request.uncertainties.join(", ")));
        } else {
            context.push_str("Uncertainties: None\n");
        }

        if let Some(ref task_context) = request.task_context {
            context.push_str(&format!("Task Context: {}\n", task_context));
        } else {
            context.push_str("Task Context: None\n");
        }

        if let Some(ref user_prompt) = request.user_prompt {
            context.push_str(&format!("User Prompt: {}\n", user_prompt));
        } else {
            context.push_str("User Prompt: None\n");
        }

        if !request.constitution_rules.is_empty() {
            context.push_str("\nConstitution:\n");
            for rule in &request.constitution_rules {
                context.push_str(&format!("- {}\n", rule));
            }
        }

        context
    }

    /// Parses model response into structured oversight data.
    ///
    /// For now, extracts advice and estimates risk score from content.
    /// In the future, this could parse structured JSON if the model supports it.
    fn parse_response(content: &str) -> Result<OversightResponse> {
        // Extract advice (full content for now)
        let advice = content.trim().to_string();

        // Estimate risk score based on keywords
        let risk_score = Self::estimate_risk_score(&advice);

        // Extract traits (simple keyword detection for now)
        let traits = Self::extract_traits(&advice);

        // Extract uncertainties (look for question marks and uncertainty phrases)
        let uncertainties = Self::extract_uncertainties(&advice);

        Ok(OversightResponse::new(advice, risk_score)
            .with_trait(traits.join(", "))
            .with_uncertainty(uncertainties.join(", ")))
    }

    /// Estimates risk score from advice content.
    fn estimate_risk_score(advice: &str) -> f64 {
        let lower = advice.to_lowercase();
        let mut score: f64 = 0.3; // Base risk

        // High risk indicators
        if lower.contains("wrong") || lower.contains("incorrect") || lower.contains("misaligned") {
            score += 0.3;
        }
        if lower.contains("problem") || lower.contains("issue") || lower.contains("concern") {
            score += 0.2;
        }
        if lower.contains("complex") || lower.contains("over-engineered") {
            score += 0.15;
        }
        if lower.contains("assumption") || lower.contains("unclear") {
            score += 0.1;
        }

        // Low risk indicators
        if lower.contains("good") || lower.contains("correct") || lower.contains("aligned") {
            score -= 0.1;
        }
        if lower.contains("continue") || lower.contains("proceed") {
            score -= 0.05;
        }

        score.min(1.0_f64).max(0.0_f64)
    }

    /// Extracts traits from advice content.
    fn extract_traits(advice: &str) -> Vec<String> {
        let lower = advice.to_lowercase();
        let mut traits = Vec::new();

        if lower.contains("complex") || lower.contains("over-engineered") {
            traits.push("Complex Solution Bias".to_string());
        }
        if lower.contains("feature") && lower.contains("creep") {
            traits.push("Feature Creep".to_string());
        }
        if lower.contains("premature") || lower.contains("too quick") {
            traits.push("Premature Implementation".to_string());
        }
        if lower.contains("misaligned") || lower.contains("wrong direction") {
            traits.push("Misalignment".to_string());
        }
        if lower.contains("too many tools") || lower.contains("unnecessary") {
            traits.push("Overtooling".to_string());
        }

        traits
    }

    /// Extracts uncertainties from advice content.
    fn extract_uncertainties(advice: &str) -> Vec<String> {
        // Simple extraction: look for questions
        advice
            .split(|c: char| c == '?' || c == '.')
            .filter(|s| s.trim().starts_with(char::is_uppercase))
            .take(3)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radium_abstraction::ModelResponse;

    #[test]
    fn test_oversight_request_new() {
        let request = OversightRequest::new(
            WorkflowPhase::Planning,
            "Build a web app".to_string(),
            "Use React and Node.js".to_string(),
        );

        assert_eq!(request.phase, WorkflowPhase::Planning);
        assert_eq!(request.goal, "Build a web app");
        assert_eq!(request.plan, "Use React and Node.js");
        assert!(request.progress.is_none());
    }

    #[test]
    fn test_oversight_request_builder() {
        let request = OversightRequest::new(
            WorkflowPhase::Implementation,
            "Fix bug".to_string(),
            "Debug the issue".to_string(),
        )
        .with_progress("50% complete")
        .with_user_prompt("Fix the login bug")
        .with_uncertainty("Not sure about the root cause");

        assert_eq!(request.progress.as_deref(), Some("50% complete"));
        assert_eq!(request.user_prompt.as_deref(), Some("Fix the login bug"));
        assert_eq!(request.uncertainties.len(), 1);
    }

    #[test]
    fn test_estimate_risk_score() {
        assert!(MetacognitiveService::estimate_risk_score("This looks wrong") > 0.5);
        assert!(MetacognitiveService::estimate_risk_score("This looks good, continue") < 0.5);
        assert!(MetacognitiveService::estimate_risk_score("") >= 0.0);
        assert!(MetacognitiveService::estimate_risk_score("") <= 1.0);
    }

    #[test]
    fn test_extract_traits() {
        let advice = "This solution is too complex and over-engineered";
        let traits = MetacognitiveService::extract_traits(advice);
        assert!(traits.contains(&"Complex Solution Bias".to_string()));
    }

    #[test]
    fn test_build_system_prompt() {
        let prompt = MetacognitiveService::build_system_prompt(&WorkflowPhase::Planning);
        assert!(prompt.contains("PLANNING"));
        assert!(prompt.contains("alignment"));
    }

    #[test]
    fn test_build_context_section() {
        let request = OversightRequest::new(
            WorkflowPhase::Implementation,
            "Build feature".to_string(),
            "Use React".to_string(),
        )
        .with_progress("Halfway done")
        .with_user_prompt("Add login");

        let context = MetacognitiveService::build_context_section(&request);
        assert!(context.contains("Goal: Build feature"));
        assert!(context.contains("Plan: Use React"));
        assert!(context.contains("Progress: Halfway done"));
        assert!(context.contains("User Prompt: Add login"));
    }
}


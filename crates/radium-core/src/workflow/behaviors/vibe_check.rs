//! VibeCheck behavior implementation.
//!
//! Implements Chain-Pattern Interrupt (CPI) for metacognitive oversight.
//! Allows agents to request oversight feedback to prevent reasoning lock-in
//! and improve alignment with user intent.

use serde::{Deserialize, Serialize};
use std::path::Path;

use super::types::{BehaviorAction, BehaviorActionType, BehaviorError, BehaviorEvaluator};
use crate::context::ContextManager;
use crate::oversight::{MetacognitiveService, OversightRequest};
use crate::policy::ConstitutionManager;

/// Decision result from vibe check evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct VibeCheckDecision {
    /// Whether to trigger oversight.
    pub should_trigger: bool,
    /// Risk score (0.0 to 1.0) indicating potential issues.
    pub risk_score: f64,
    /// Human-readable advice from oversight LLM.
    pub advice: String,
    /// Detected traits or patterns.
    pub traits: Vec<String>,
    /// Uncertainties identified.
    pub uncertainties: Vec<String>,
    /// Human-readable reason for the vibe check.
    pub reason: Option<String>,
}

impl VibeCheckDecision {
    /// Creates a new vibe check decision.
    pub fn new(
        should_trigger: bool,
        risk_score: f64,
        advice: String,
        traits: Vec<String>,
        uncertainties: Vec<String>,
    ) -> Self {
        Self { should_trigger, risk_score, advice, traits, uncertainties, reason: None }
    }

    /// Adds a reason to the decision.
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

/// Context for vibe check evaluation.
#[derive(Debug, Clone)]
pub struct VibeCheckContext {
    /// Current workflow phase (planning, implementation, review).
    pub phase: WorkflowPhase,
    /// Goal or objective being pursued.
    pub goal: Option<String>,
    /// Current plan or approach.
    pub plan: Option<String>,
    /// Progress made so far.
    pub progress: Option<String>,
    /// User's original prompt.
    pub user_prompt: Option<String>,
    /// Task context or recent actions.
    pub task_context: Option<String>,
}

impl VibeCheckContext {
    /// Creates a new vibe check context.
    pub fn new(phase: WorkflowPhase) -> Self {
        Self {
            phase,
            goal: None,
            plan: None,
            progress: None,
            user_prompt: None,
            task_context: None,
        }
    }

    /// Sets the goal.
    #[must_use]
    pub fn with_goal(mut self, goal: impl Into<String>) -> Self {
        self.goal = Some(goal.into());
        self
    }

    /// Sets the plan.
    #[must_use]
    pub fn with_plan(mut self, plan: impl Into<String>) -> Self {
        self.plan = Some(plan.into());
        self
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
}

/// Workflow phase for phase-aware oversight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowPhase {
    /// Planning phase - focus on alignment and assumptions.
    Planning,
    /// Implementation phase - focus on consistency and methods.
    Implementation,
    /// Review phase - focus on completeness and verification.
    Review,
}

impl std::fmt::Display for WorkflowPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Planning => write!(f, "planning"),
            Self::Implementation => write!(f, "implementation"),
            Self::Review => write!(f, "review"),
        }
    }
}

/// Evaluates vibe check behavior based on behavior.json.
///
/// VibeCheck can be triggered by agents writing a vibecheck action,
/// or automatically at workflow checkpoints based on risk assessment.
pub struct VibeCheckEvaluator;

impl VibeCheckEvaluator {
    /// Creates a new vibe check evaluator.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates vibe check behavior (synchronous, basic detection only).
    ///
    /// # Arguments
    /// * `behavior_file` - Path to behavior.json
    /// * `output` - Output from agent execution
    /// * `context` - VibeCheckContext for phase-aware evaluation
    ///
    /// # Returns
    /// `Ok(Some(VibeCheckDecision))` if vibe check should be triggered,
    /// `Ok(None)` if no vibe check behavior,
    /// `Err(BehaviorError)` on evaluation error.
    ///
    /// Note: This method only detects if a vibecheck should be triggered.
    /// For full oversight with MetacognitiveService, use `evaluate_with_oversight`.
    pub fn evaluate_vibe_check(
        &self,
        behavior_file: &Path,
        _output: &str,
        _context: &VibeCheckContext,
    ) -> Result<Option<VibeCheckDecision>, BehaviorError> {
        // Check for explicit vibe check action
        let Some(action) = BehaviorAction::read_from_file(behavior_file)? else {
            return Ok(None);
        };

        // Only handle vibe check actions
        if action.action != BehaviorActionType::VibeCheck {
            return Ok(None);
        }

        // For now, create a basic decision that triggers oversight
        // The actual oversight LLM call will be handled by evaluate_with_oversight
        let mut decision = VibeCheckDecision::new(
            true,
            0.5, // Default risk score - will be updated by oversight service
            "Oversight requested".to_string(),
            vec![],
            vec![],
        );

        if let Some(reason) = action.reason {
            decision = decision.with_reason(reason);
        }

        Ok(Some(decision))
    }

    /// Evaluates vibe check behavior with full metacognitive oversight.
    ///
    /// This method:
    /// 1. Detects if vibecheck should be triggered
    /// 2. Gathers context from ContextManager, LearningStore, and ConstitutionManager
    /// 3. Calls MetacognitiveService for oversight feedback
    /// 4. Returns VibeCheckDecision with oversight response
    ///
    /// # Arguments
    /// * `behavior_file` - Path to behavior.json
    /// * `output` - Output from agent execution
    /// * `vibe_context` - VibeCheckContext for phase-aware evaluation
    /// * `metacognitive` - MetacognitiveService for oversight
    /// * `context_manager` - ContextManager for gathering history and learning context
    /// * `constitution_manager` - ConstitutionManager for session rules
    /// * `session_id` - Session ID for constitution and history
    ///
    /// # Returns
    /// `Ok(Some(VibeCheckDecision))` with oversight feedback if vibe check triggered,
    /// `Ok(None)` if no vibe check behavior,
    /// `Err(BehaviorError)` on evaluation error.
    pub async fn evaluate_with_oversight(
        &self,
        behavior_file: &Path,
        output: &str,
        vibe_context: &VibeCheckContext,
        metacognitive: &MetacognitiveService,
        context_manager: &ContextManager,
        constitution_manager: &ConstitutionManager,
        session_id: Option<&str>,
    ) -> Result<Option<VibeCheckDecision>, BehaviorError> {
        // First check if vibecheck should be triggered
        let Some(mut decision) = self.evaluate_vibe_check(behavior_file, output, vibe_context)? else {
            return Ok(None);
        };

        // Gather context for oversight request
        let goal = vibe_context.goal.clone().unwrap_or_else(|| "Unknown goal".to_string());
        let plan = vibe_context.plan.clone().unwrap_or_else(|| "No plan specified".to_string());

        // Build oversight request
        let mut oversight_request = OversightRequest::new(vibe_context.phase, goal, plan)
            .with_progress(vibe_context.progress.clone().unwrap_or_default())
            .with_task_context(vibe_context.task_context.clone().unwrap_or_default());
        
        if let Some(user_prompt) = vibe_context.user_prompt.clone() {
            oversight_request = oversight_request.with_user_prompt(user_prompt);
        }

        // Add learning context if available
        if let Some(learning_context) = context_manager.gather_learning_context(3) {
            // Remove the markdown header that gather_learning_context adds
            let context = learning_context
                .strip_prefix("# Learning Context\n\n")
                .and_then(|s| s.strip_suffix("\n"))
                .unwrap_or(&learning_context)
                .to_string();
            oversight_request = oversight_request.with_learning_context(context);
        }

        // Add history summary if available (from memory context)
        if let Ok(Some(history)) = context_manager.gather_memory_context("agent") {
            oversight_request = oversight_request.with_history_summary(history);
        }

        // Add constitution rules if session ID provided
        if let Some(sid) = session_id {
            let rules = constitution_manager.get_constitution(sid);
            oversight_request = oversight_request.with_constitution_rules(rules);
        }

        // Add session ID if provided
        if let Some(sid) = session_id {
            oversight_request = oversight_request.with_session_id(sid);
        }

        // Call metacognitive service for oversight
        match metacognitive.generate_oversight(&oversight_request).await {
            Ok(oversight_response) => {
                // Update decision with oversight response
                decision.risk_score = oversight_response.risk_score;
                decision.advice = oversight_response.advice;
                decision.traits = oversight_response.traits;
                decision.uncertainties = oversight_response.uncertainties;
                Ok(Some(decision))
            }
            Err(e) => {
                // On error, return decision with error message in advice
                decision.advice = format!("Oversight service error: {}", e);
                decision.risk_score = 0.8; // High risk due to oversight failure
                Ok(Some(decision))
            }
        }
    }
}

impl Default for VibeCheckEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl BehaviorEvaluator for VibeCheckEvaluator {
    type Decision = VibeCheckDecision;

    fn evaluate(
        &self,
        behavior_file: &Path,
        output: &str,
        context: &dyn std::any::Any,
    ) -> Result<Option<Self::Decision>, BehaviorError> {
        // Try to downcast context to VibeCheckContext
        let default_context = VibeCheckContext::new(WorkflowPhase::Implementation);
        let vibe_context = context.downcast_ref::<VibeCheckContext>().unwrap_or(&default_context);

        self.evaluate_vibe_check(behavior_file, output, vibe_context)
    }
}

/// State for managing vibe check UI and workflow oversight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeCheckState {
    /// Whether a vibe check is currently active.
    pub active: bool,
    /// Current risk score.
    pub risk_score: f64,
    /// Advice from oversight LLM.
    pub advice: Option<String>,
    /// When the vibe check was triggered.
    pub triggered_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl VibeCheckState {
    /// Creates a new inactive vibe check state.
    pub fn new() -> Self {
        Self { active: false, risk_score: 0.0, advice: None, triggered_at: None }
    }

    /// Activates a vibe check with decision data.
    pub fn activate(&mut self, decision: &VibeCheckDecision) {
        self.active = true;
        self.risk_score = decision.risk_score;
        self.advice = Some(decision.advice.clone());
        self.triggered_at = Some(chrono::Utc::now());
    }

    /// Deactivates the vibe check.
    pub fn deactivate(&mut self) {
        self.active = false;
        self.risk_score = 0.0;
        self.advice = None;
        self.triggered_at = None;
    }

    /// Checks if a vibe check is currently active.
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Default for VibeCheckState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_vibe_check_evaluator_no_behavior_file() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        let evaluator = VibeCheckEvaluator::new();
        let context = VibeCheckContext::new(WorkflowPhase::Implementation);
        let result = evaluator.evaluate_vibe_check(&behavior_file, "", &context);

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_vibe_check_evaluator_vibe_check_action() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        // Write vibe check action
        let action = BehaviorAction::new(BehaviorActionType::VibeCheck)
            .with_reason("Need to verify approach");
        action.write_to_file(&behavior_file).unwrap();

        let evaluator = VibeCheckEvaluator::new();
        let context = VibeCheckContext::new(WorkflowPhase::Planning);
        let result = evaluator.evaluate_vibe_check(&behavior_file, "", &context).unwrap();

        assert!(result.is_some());
        let decision = result.unwrap();
        assert!(decision.should_trigger);
        assert_eq!(decision.reason.as_deref(), Some("Need to verify approach"));
    }

    #[test]
    fn test_vibe_check_evaluator_non_vibe_check_action() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        // Write loop action (should not trigger vibe check)
        let action = BehaviorAction::new(BehaviorActionType::Loop);
        action.write_to_file(&behavior_file).unwrap();

        let evaluator = VibeCheckEvaluator::new();
        let context = VibeCheckContext::new(WorkflowPhase::Implementation);
        let result = evaluator.evaluate_vibe_check(&behavior_file, "", &context).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_vibe_check_state_new() {
        let state = VibeCheckState::new();
        assert!(!state.is_active());
        assert_eq!(state.risk_score, 0.0);
        assert!(state.advice.is_none());
        assert!(state.triggered_at.is_none());
    }

    #[test]
    fn test_vibe_check_state_activate() {
        let mut state = VibeCheckState::new();
        let decision = VibeCheckDecision::new(
            true,
            0.7,
            "Consider simpler approach".to_string(),
            vec!["complexity".to_string()],
            vec![],
        );

        state.activate(&decision);

        assert!(state.is_active());
        assert_eq!(state.risk_score, 0.7);
        assert_eq!(state.advice.as_deref(), Some("Consider simpler approach"));
        assert!(state.triggered_at.is_some());
    }

    #[test]
    fn test_vibe_check_state_deactivate() {
        let mut state = VibeCheckState::new();
        let decision = VibeCheckDecision::new(true, 0.5, "Test".to_string(), vec![], vec![]);
        state.activate(&decision);
        assert!(state.is_active());

        state.deactivate();
        assert!(!state.is_active());
        assert_eq!(state.risk_score, 0.0);
        assert!(state.advice.is_none());
        assert!(state.triggered_at.is_none());
    }

    #[test]
    fn test_workflow_phase_display() {
        assert_eq!(WorkflowPhase::Planning.to_string(), "planning");
        assert_eq!(WorkflowPhase::Implementation.to_string(), "implementation");
        assert_eq!(WorkflowPhase::Review.to_string(), "review");
    }
}

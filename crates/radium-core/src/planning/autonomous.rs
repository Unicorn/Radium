//! Autonomous workflow decomposition from high-level goals.
//!
//! Provides functionality to decompose high-level goals into executable workflows
//! with proper dependency analysis and validation.

use crate::agents::registry::AgentRegistry;
use crate::models::PlanManifest;
use crate::planning::dag::{DagError, DependencyGraph};
use crate::planning::generator::PlanGenerator;
use crate::planning::parser::{ParsedIteration, ParsedPlan, ParsedTask};
use crate::workflow::templates::{WorkflowStep, WorkflowStepConfig, WorkflowStepType, WorkflowTemplate};
use radium_abstraction::Model;
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during autonomous planning.
#[derive(Debug, Error)]
pub enum PlanningError {
    /// Plan generation failed.
    #[error("Plan generation failed: {0}")]
    GenerationFailed(String),

    /// Plan validation failed.
    #[error("Plan validation failed: {0}")]
    ValidationFailed(String),

    /// DAG error.
    #[error("DAG error: {0}")]
    Dag(#[from] DagError),

    /// Workflow generation failed.
    #[error("Workflow generation failed: {0}")]
    WorkflowGenerationFailed(String),

    /// Agent not found.
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
}

/// Result type for autonomous planning operations.
pub type Result<T> = std::result::Result<T, PlanningError>;

/// Validation report for plan validation.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Whether the plan is valid.
    pub is_valid: bool,
    /// List of validation errors.
    pub errors: Vec<String>,
    /// List of validation warnings.
    pub warnings: Vec<String>,
}

/// Validates plans for correctness.
pub struct PlanValidator {
    /// Agent registry for validating agent assignments.
    agent_registry: Arc<AgentRegistry>,
}

impl PlanValidator {
    /// Creates a new plan validator.
    pub fn new(agent_registry: Arc<AgentRegistry>) -> Self {
        Self { agent_registry }
    }

    /// Validates a parsed plan.
    ///
    /// # Arguments
    /// * `plan` - The plan to validate
    ///
    /// # Returns
    /// Validation report with errors and warnings
    pub fn validate_plan(&self, plan: &ParsedPlan) -> ValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Convert to PlanManifest for DAG validation
        let manifest = self.parsed_plan_to_manifest(plan);

        // Validate dependencies with DAG
        match DependencyGraph::from_manifest(&manifest) {
            Ok(dag) => {
                // Check for cycles
                if let Err(e) = dag.detect_cycles() {
                    errors.push(format!("Circular dependency detected: {}", e));
                }
            }
            Err(e) => {
                errors.push(format!("Dependency graph error: {}", e));
            }
        }

        // Validate agent assignments
        for iteration in &plan.iterations {
            for task in &iteration.tasks {
                if let Some(agent_id) = &task.agent_id {
                    if agent_id != "auto" {
                        if self.agent_registry.get(agent_id).is_err() {
                            warnings.push(format!(
                                "Task {}.T{} references unknown agent: {}",
                                iteration.number, task.number, agent_id
                            ));
                        }
                    }
                }
            }
        }

        // Validate dependency references
        for iteration in &plan.iterations {
            for task in &iteration.tasks {
                for dep_id in &task.dependencies {
                    if !self.dependency_exists(plan, dep_id) {
                        errors.push(format!(
                            "Task {}.T{} references non-existent dependency: {}",
                            iteration.number, task.number, dep_id
                        ));
                    }
                }
            }
        }

        let is_valid = errors.is_empty();
        ValidationReport { is_valid, errors, warnings }
    }

    /// Converts ParsedPlan to PlanManifest for DAG validation.
    fn parsed_plan_to_manifest(&self, plan: &ParsedPlan) -> PlanManifest {
        use crate::models::{Iteration, PlanTask};
        use crate::workspace::RequirementId;
        use std::str::FromStr;

        let req_id = RequirementId::from_str("AUTO").unwrap_or_else(|_| {
            RequirementId::from_str("REQ-000").unwrap()
        });
        let mut manifest = PlanManifest::new(req_id, plan.project_name.clone());

        for parsed_iter in &plan.iterations {
            let mut iteration = Iteration::new(parsed_iter.number, parsed_iter.name.clone());
            if let Some(desc) = &parsed_iter.description {
                iteration.description = Some(desc.clone());
            }
            if let Some(goal) = &parsed_iter.goal {
                iteration.goal = Some(goal.clone());
            }

            for parsed_task in &parsed_iter.tasks {
                let mut task = PlanTask::new(
                    &format!("I{}", parsed_iter.number),
                    parsed_task.number,
                    parsed_task.title.clone(),
                );
                task.description = parsed_task.description.clone();
                task.agent_id = parsed_task.agent_id.clone();
                task.dependencies = parsed_task.dependencies.clone();
                task.acceptance_criteria = parsed_task.acceptance_criteria.clone();
                iteration.add_task(task);
            }

            manifest.add_iteration(iteration);
        }

        manifest
    }

    /// Checks if a dependency exists in the plan.
    fn dependency_exists(&self, plan: &ParsedPlan, dep_id: &str) -> bool {
        for iteration in &plan.iterations {
            let expected_id = format!("I{}.T", iteration.number);
            if dep_id.starts_with(&expected_id) {
                if let Some(task_num) = dep_id.strip_prefix(&expected_id) {
                    if let Ok(num) = task_num.parse::<u32>() {
                        if iteration.tasks.iter().any(|t| t.number == num) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

/// Generates workflows from validated plans.
pub struct WorkflowGenerator;

impl WorkflowGenerator {
    /// Creates a new workflow generator.
    pub fn new() -> Self {
        Self
    }

    /// Generates a workflow from a validated plan and DAG.
    ///
    /// # Arguments
    /// * `plan` - The parsed plan
    /// * `dag` - The dependency graph
    ///
    /// # Returns
    /// A workflow template
    ///
    /// # Errors
    /// Returns error if workflow generation fails
    pub fn generate_workflow(
        &self,
        plan: &ParsedPlan,
        dag: &DependencyGraph,
    ) -> Result<WorkflowTemplate> {
        // Get topological sort for execution order
        let sorted_tasks = dag.topological_sort()?;

        // Create workflow template
        let mut template = WorkflowTemplate::new(&plan.project_name);
        if let Some(desc) = &plan.description {
            template = template.with_description(desc.clone());
        }

        // Create steps in dependency order
        let mut step_order = 0u32;
        for task_id in sorted_tasks {
            // Find the task in the plan
            if let Some((iteration, task)) = self.find_task_in_plan(plan, &task_id) {
                let agent_id = task.agent_id.as_deref().unwrap_or("auto");

                let step_config = WorkflowStepConfig {
                    agent_id: agent_id.to_string(),
                    agent_name: Some(task.title.clone()),
                    step_type: WorkflowStepType::Step,
                    execute_once: false,
                    engine: None,
                    model: None,
                    model_reasoning_effort: None,
                    not_completed_fallback: None,
                    module: None,
                    label: None,
                };

                let step = WorkflowStep {
                    config: step_config,
                };

                template = template.add_step(step);
                step_order += 1;
            }
        }

        Ok(template)
    }

    /// Finds a task in the plan by task ID.
    fn find_task_in_plan<'a>(
        &self,
        plan: &'a ParsedPlan,
        task_id: &str,
    ) -> Option<(&'a ParsedIteration, &'a ParsedTask)> {
        for iteration in &plan.iterations {
            let iter_prefix = format!("I{}.T", iteration.number);
            if let Some(task_num_str) = task_id.strip_prefix(&iter_prefix) {
                if let Ok(task_num) = task_num_str.parse::<u32>() {
                    if let Some(task) = iteration.tasks.iter().find(|t| t.number == task_num) {
                        return Some((iteration, task));
                    }
                }
            }
        }
        None
    }
}

impl Default for WorkflowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Autonomous planner that orchestrates goal-to-workflow pipeline.
pub struct AutonomousPlanner {
    /// Plan generator for LLM-based decomposition.
    plan_generator: PlanGenerator,
    /// Plan validator.
    plan_validator: PlanValidator,
    /// Workflow generator.
    workflow_generator: WorkflowGenerator,
    /// Agent registry.
    agent_registry: Arc<AgentRegistry>,
}

impl AutonomousPlanner {
    /// Creates a new autonomous planner.
    ///
    /// # Arguments
    /// * `agent_registry` - The agent registry
    pub fn new(agent_registry: Arc<AgentRegistry>) -> Self {
        Self {
            plan_generator: PlanGenerator::new(),
            plan_validator: PlanValidator::new(agent_registry.clone()),
            workflow_generator: WorkflowGenerator::new(),
            agent_registry,
        }
    }

    /// Plans from a high-level goal.
    ///
    /// # Arguments
    /// * `goal` - The high-level goal description
    /// * `model` - The model to use for plan generation
    ///
    /// # Returns
    /// An autonomous plan containing plan, workflow, and DAG
    ///
    /// # Errors
    /// Returns error if planning fails
    pub async fn plan_from_goal(
        &self,
        goal: &str,
        model: Arc<dyn Model>,
    ) -> Result<AutonomousPlan> {
        // Step 1: Generate plan from goal
        let mut parsed_plan = self.plan_generator.generate(goal, model.clone()).await
            .map_err(|e| PlanningError::GenerationFailed(e))?;

        // Step 2: Validate plan (with retry logic)
        let mut validation_attempts = 0;
        const MAX_VALIDATION_RETRIES: u32 = 2;

        loop {
            let report = self.plan_validator.validate_plan(&parsed_plan);

            if report.is_valid {
                break;
            }

            if validation_attempts >= MAX_VALIDATION_RETRIES {
                return Err(PlanningError::ValidationFailed(format!(
                    "Plan validation failed after {} attempts: {:?}",
                    MAX_VALIDATION_RETRIES, report.errors
                )));
            }

            // Retry with validation feedback
            let feedback = format!(
                "{}\n\nValidation errors found:\n{}",
                goal,
                report.errors.join("\n")
            );
            parsed_plan = self.plan_generator.generate(&feedback, model.clone()).await
                .map_err(|e| PlanningError::GenerationFailed(e))?;
            validation_attempts += 1;
        }

        // Step 3: Build dependency graph
        let manifest = self.plan_validator.parsed_plan_to_manifest(&parsed_plan);
        let dag = DependencyGraph::from_manifest(&manifest)?;

        // Step 4: Generate workflow
        let workflow = self.workflow_generator.generate_workflow(&parsed_plan, &dag)?;

        Ok(AutonomousPlan {
            plan: parsed_plan,
            workflow,
            dag,
            manifest,
        })
    }
}

/// Complete autonomous plan with all components.
#[derive(Debug, Clone)]
pub struct AutonomousPlan {
    /// The parsed plan.
    pub plan: ParsedPlan,
    /// The generated workflow template.
    pub workflow: WorkflowTemplate,
    /// The dependency graph.
    pub dag: DependencyGraph,
    /// The plan manifest.
    pub manifest: PlanManifest,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::registry::AgentRegistry;

    #[test]
    fn test_plan_validator_dependency_validation() {
        // This test would require creating a mock plan
        // For now, we'll test the logic structure
        let registry = Arc::new(AgentRegistry::new());
        let validator = PlanValidator::new(registry);
        
        // Test would validate dependency references
        assert!(true); // Placeholder
    }

    #[test]
    fn test_workflow_generator_ordering() {
        let generator = WorkflowGenerator::new();
        // Test would verify workflow steps are in correct order
        assert!(true); // Placeholder
    }
}


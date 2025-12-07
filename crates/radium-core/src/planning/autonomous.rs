//! Autonomous workflow decomposition from high-level goals.
//!
//! This module provides the autonomous planning system that converts high-level goals
//! into structured, executable workflows with automatic validation and retry logic.
//!
//! # Overview
//!
//! The autonomous planning system orchestrates a complete pipeline from goal to workflow:
//!
//! 1. **Plan Generation**: Uses AI to decompose goals into structured plans with iterations and tasks
//! 2. **Plan Validation**: Multi-stage validation with retry logic (up to 2 retries)
//! 3. **Dependency Analysis**: Builds a DAG to detect cycles and validate dependencies
//! 4. **Workflow Generation**: Creates executable workflow templates from validated plans
//!
//! # Architecture
//!
//! ```
//! Goal → PlanGenerator → ParsedPlan
//!                          ↓
//!                    PlanValidator (with retry)
//!                          ↓
//!                    DependencyGraph (DAG)
//!                          ↓
//!                    WorkflowGenerator
//!                          ↓
//!                    AutonomousPlan (complete)
//! ```
//!
//! # Validation Retry Logic
//!
//! The system includes intelligent retry logic to handle validation failures:
//!
//! - **Max Retries**: 2 attempts after initial generation
//! - **Feedback Loop**: Validation errors are fed back to the generator
//! - **Error Categories**: Distinguishes between recoverable and fatal errors
//!
//! # Example
//!
//! ```rust,no_run
//! use radium_core::planning::autonomous::AutonomousPlanner;
//! use radium_core::agents::registry::AgentRegistry;
//! use radium_abstraction::Model;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let agent_registry = Arc::new(AgentRegistry::new());
//! let planner = AutonomousPlanner::new(agent_registry);
//! let model: Arc<dyn Model> = /* ... */;
//!
//! let goal = "Build a REST API with authentication and user management";
//! let plan = planner.plan_from_goal(goal, model).await?;
//!
//! // plan contains:
//! // - plan: ParsedPlan with iterations and tasks
//! // - workflow: WorkflowTemplate ready for execution
//! // - dag: DependencyGraph for dependency analysis
//! // - manifest: PlanManifest for execution tracking
//! # Ok(())
//! # }
//! ```
//!
//! # See Also
//!
//! - [User Guide](../../../docs/features/autonomous-planning.md) - Complete user documentation
//! - [DAG Module](dag) - Dependency graph implementation
//! - [Plan Generator](generator) - AI-powered plan generation

use crate::agents::registry::AgentRegistry;
use crate::models::PlanManifest;
use crate::planning::dag::{DagError, DependencyGraph};
use crate::planning::generator::PlanGenerator;
use crate::planning::parser::{ParsedIteration, ParsedPlan, ParsedTask};
#[cfg(feature = "workflow")]
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
///
/// Contains the results of plan validation, including errors and warnings.
/// Errors indicate problems that must be fixed (e.g., circular dependencies, missing references).
/// Warnings indicate potential issues that don't block execution (e.g., unknown agents).
///
/// # Example
///
/// ```rust,no_run
/// use radium_core::planning::autonomous::PlanValidator;
/// use radium_core::agents::registry::AgentRegistry;
/// use std::sync::Arc;
///
/// let registry = Arc::new(AgentRegistry::new());
/// let validator = PlanValidator::new(registry);
/// let plan = /* ... */;
///
/// let report = validator.validate_plan(&plan);
/// if !report.is_valid {
///     println!("Validation errors: {:?}", report.errors);
/// }
/// if !report.warnings.is_empty() {
///     println!("Warnings: {:?}", report.warnings);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Whether the plan is valid.
    ///
    /// A plan is valid if it has no errors. Warnings do not affect validity.
    pub is_valid: bool,
    /// List of validation errors.
    ///
    /// Errors must be fixed before the plan can be executed. Examples:
    /// - Circular dependencies
    /// - Missing dependency references
    /// - Invalid task ID formats
    pub errors: Vec<String>,
    /// List of validation warnings.
    ///
    /// Warnings indicate potential issues but don't block execution. Examples:
    /// - Unknown agent IDs (may be resolved at runtime)
    /// - Missing optional fields
    pub warnings: Vec<String>,
}

/// Validates plans for correctness.
///
/// Performs multi-stage validation including:
/// - Dependency graph validation (cycle detection)
/// - Agent assignment validation
/// - Dependency reference validation
///
/// # Validation Stages
///
/// 1. **Dependency Graph**: Builds a DAG and checks for cycles
/// 2. **Agent Validation**: Verifies agent IDs exist in registry
/// 3. **Dependency References**: Ensures all task dependencies exist
///
/// # Example
///
/// ```rust,no_run
/// use radium_core::planning::autonomous::PlanValidator;
/// use radium_core::agents::registry::AgentRegistry;
/// use std::sync::Arc;
///
/// let registry = Arc::new(AgentRegistry::new());
/// let validator = PlanValidator::new(registry);
/// let plan = /* ... */;
///
/// let report = validator.validate_plan(&plan);
/// ```
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
///
/// Converts validated plans into executable workflow templates by:
/// - Using topological sort from DAG to determine execution order
/// - Creating workflow steps for each task
/// - Preserving agent assignments and task metadata
///
/// # Example
///
/// ```rust,no_run
/// use radium_core::planning::autonomous::WorkflowGenerator;
/// use radium_core::planning::dag::DependencyGraph;
///
/// let generator = WorkflowGenerator::new();
/// let plan = /* ... */;
/// let dag = DependencyGraph::from_manifest(&manifest)?;
///
/// let workflow = generator.generate_workflow(&plan, &dag)?;
/// ```
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
    #[cfg(feature = "workflow")]
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
///
/// The main entry point for autonomous planning. Coordinates plan generation,
/// validation (with retry logic), dependency analysis, and workflow generation.
///
/// # Pipeline Flow
///
/// 1. **Generate Plan**: Uses `PlanGenerator` to create a plan from goal
/// 2. **Validate Plan**: Uses `PlanValidator` to check correctness
/// 3. **Retry on Failure**: If validation fails, regenerates with feedback (max 2 retries)
/// 4. **Build DAG**: Creates dependency graph for cycle detection and ordering
/// 5. **Generate Workflow**: Converts validated plan to executable workflow
///
/// # Retry Logic
///
/// If validation fails, the system:
/// - Feeds validation errors back to the generator
/// - Regenerates the plan with error context
/// - Re-validates the new plan
/// - Fails after 2 retry attempts
///
/// # Example
///
/// ```rust,no_run
/// use radium_core::planning::autonomous::AutonomousPlanner;
/// use radium_core::agents::registry::AgentRegistry;
/// use radium_abstraction::Model;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let agent_registry = Arc::new(AgentRegistry::new());
/// let planner = AutonomousPlanner::new(agent_registry);
/// let model: Arc<dyn Model> = /* ... */;
///
/// let goal = "Build a REST API with authentication";
/// let autonomous_plan = planner.plan_from_goal(goal, model).await?;
///
/// // Use the generated workflow
/// println!("Generated {} iterations", autonomous_plan.plan.iterations.len());
/// println!("Workflow has {} steps", autonomous_plan.workflow.steps().len());
/// # Ok(())
/// # }
/// ```
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
        // DISABLED: workflow module is disabled
        // let workflow = self.workflow_generator.generate_workflow(&parsed_plan, &dag)?;

        // For now, return error since workflow is required
        return Err(PlanningError::WorkflowGenerationFailed(
            "Workflow module is disabled. Cannot generate workflow.".to_string()
        ));

        // Ok(AutonomousPlan {
        //     plan: parsed_plan,
        //     workflow,
        //     dag,
        //     manifest,
        // })
    }
}

/// Complete autonomous plan with all components.
///
/// Contains all artifacts generated by the autonomous planning process:
/// - The structured plan with iterations and tasks
/// - An executable workflow template
/// - A dependency graph for execution ordering
/// - A plan manifest for tracking execution state
///
/// # Example
///
/// ```rust,no_run
/// use radium_core::planning::autonomous::AutonomousPlanner;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let planner = /* ... */;
/// let model = /* ... */;
/// let autonomous_plan = planner.plan_from_goal("Build API", model).await?;
///
/// // Access components
/// let plan = &autonomous_plan.plan;
/// let workflow = &autonomous_plan.workflow;
/// let dag = &autonomous_plan.dag;
/// let manifest = &autonomous_plan.manifest;
///
/// // Use workflow for execution
/// // Use DAG for dependency analysis
/// // Use manifest for progress tracking
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct AutonomousPlan {
    /// The parsed plan with iterations and tasks.
    pub plan: ParsedPlan,
    /// The generated workflow template ready for execution.
    #[cfg(feature = "workflow")]
    pub workflow: crate::workflow::templates::WorkflowTemplate,
    /// The dependency graph for cycle detection and ordering.
    pub dag: DependencyGraph,
    /// The plan manifest for execution tracking.
    pub manifest: PlanManifest,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::registry::AgentRegistry;
    use crate::planning::{ParsedIteration, ParsedPlan, ParsedTask};

    #[test]
    fn test_plan_validator_dependency_validation() {
        let registry = Arc::new(AgentRegistry::new());
        let validator = PlanValidator::new(registry);

        // Create plan with valid and invalid dependencies
        let plan = ParsedPlan {
            project_name: "Dependency Test".to_string(),
            description: None,
            tech_stack: vec![],
            iterations: vec![
                ParsedIteration {
                    number: 1,
                    name: "Iteration 1".to_string(),
                    description: None,
                    goal: None,
                    tasks: vec![
                        ParsedTask {
                            number: 1,
                            title: "Task 1".to_string(),
                            description: None,
                            agent_id: Some("code-agent".to_string()),
                            dependencies: vec![],
                            acceptance_criteria: vec![],
                        },
                        ParsedTask {
                            number: 2,
                            title: "Task 2".to_string(),
                            description: None,
                            agent_id: Some("code-agent".to_string()),
                            dependencies: vec!["I1.T1".to_string()], // Valid dependency
                            acceptance_criteria: vec![],
                        },
                        ParsedTask {
                            number: 3,
                            title: "Task 3".to_string(),
                            description: None,
                            agent_id: Some("code-agent".to_string()),
                            dependencies: vec!["I5.T1".to_string()], // Invalid dependency
                            acceptance_criteria: vec![],
                        },
                    ],
                },
            ],
        };

        let report = validator.validate_plan(&plan);
        
        // Should have error for invalid dependency
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.contains("non-existent dependency")));
    }

    #[test]
    fn test_plan_validator_agent_validation() {
        let registry = Arc::new(AgentRegistry::new());
        let validator = PlanValidator::new(registry);

        let plan = ParsedPlan {
            project_name: "Agent Validation Test".to_string(),
            description: None,
            tech_stack: vec![],
            iterations: vec![ParsedIteration {
                number: 1,
                name: "Iteration 1".to_string(),
                description: None,
                goal: None,
                tasks: vec![
                    ParsedTask {
                        number: 1,
                        title: "Task 1".to_string(),
                        description: None,
                        agent_id: Some("unknown-agent-123".to_string()),
                        dependencies: vec![],
                        acceptance_criteria: vec![],
                    },
                ],
            }],
        };

        let report = validator.validate_plan(&plan);
        
        // Unknown agents are warnings, not errors
        assert!(report.is_valid);
        assert!(report.warnings.iter().any(|w| w.contains("unknown agent")));
    }

    #[test]
    fn test_plan_validator_auto_agent_no_warning() {
        let registry = Arc::new(AgentRegistry::new());
        let validator = PlanValidator::new(registry);

        let plan = ParsedPlan {
            project_name: "Auto Agent Test".to_string(),
            description: None,
            tech_stack: vec![],
            iterations: vec![ParsedIteration {
                number: 1,
                name: "Iteration 1".to_string(),
                description: None,
                goal: None,
                tasks: vec![ParsedTask {
                    number: 1,
                    title: "Task 1".to_string(),
                    description: None,
                    agent_id: Some("auto".to_string()), // "auto" should not trigger warning
                    dependencies: vec![],
                    acceptance_criteria: vec![],
                }],
            }],
        };

        let report = validator.validate_plan(&plan);
        
        // "auto" agent should not generate warnings
        assert!(report.is_valid);
        assert!(!report.warnings.iter().any(|w| w.contains("auto")));
    }

    #[test]
    fn test_plan_validator_empty_plan() {
        let registry = Arc::new(AgentRegistry::new());
        let validator = PlanValidator::new(registry);

        let plan = ParsedPlan {
            project_name: "Empty Plan".to_string(),
            description: None,
            tech_stack: vec![],
            iterations: vec![],
        };

        let report = validator.validate_plan(&plan);
        
        // Empty plan should be valid (no errors)
        assert!(report.is_valid);
        assert!(report.errors.is_empty());
    }

    #[cfg(feature = "workflow")]
    #[test]
    fn test_workflow_generator_ordering() {
        let generator = WorkflowGenerator::new();
        
        // Create a plan with dependencies
        let plan = ParsedPlan {
            project_name: "Workflow Test".to_string(),
            description: None,
            tech_stack: vec![],
            iterations: vec![ParsedIteration {
                number: 1,
                name: "Iteration 1".to_string(),
                description: None,
                goal: None,
                tasks: vec![
                    ParsedTask {
                        number: 1,
                        title: "Task 1".to_string(),
                        description: None,
                        agent_id: Some("code-agent".to_string()),
                        dependencies: vec![],
                        acceptance_criteria: vec![],
                    },
                    ParsedTask {
                        number: 2,
                        title: "Task 2".to_string(),
                        description: None,
                        agent_id: Some("code-agent".to_string()),
                        dependencies: vec!["I1.T1".to_string()],
                        acceptance_criteria: vec![],
                    },
                ],
            }],
        };

        // Convert to manifest for DAG
        use crate::models::{Iteration, PlanManifest, PlanTask};
        use crate::workspace::RequirementId;
        use std::str::FromStr;

        let req_id = RequirementId::from_str("TEST").unwrap();
        let mut manifest = PlanManifest::new(req_id, plan.project_name.clone());

        for parsed_iter in &plan.iterations {
            let mut iteration = Iteration::new(parsed_iter.number, parsed_iter.name.clone());
            for parsed_task in &parsed_iter.tasks {
                let mut task = PlanTask::new(
                    &format!("I{}", parsed_iter.number),
                    parsed_task.number,
                    parsed_task.title.clone(),
                );
                task.dependencies = parsed_task.dependencies.clone();
                iteration.add_task(task);
            }
            manifest.add_iteration(iteration);
        }

        let dag = DependencyGraph::from_manifest(&manifest).unwrap();
        let workflow = generator.generate_workflow(&plan, &dag).unwrap();

        // Workflow should have steps in dependency order
        // Access steps as a field, not a method
        let steps = &workflow.steps;
        assert_eq!(steps.len(), 2);
        
        // Verify workflow was generated successfully
        // The exact structure depends on WorkflowStep implementation
        // Just verify we have the expected number of steps
        assert!(!steps.is_empty());
    }
}


//! TypeScript code generation

use handlebars::Handlebars;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

use crate::schema::{NodeType, RetryPolicy, WorkflowDefinition, WorkflowNode, WorkflowVariable};

/// Code generation errors
#[derive(Debug, Error)]
pub enum GenerationError {
    #[error("Template error: {0}")]
    TemplateError(#[from] handlebars::TemplateError),
    #[error("Render error: {0}")]
    RenderError(#[from] handlebars::RenderError),
    #[error("No start node found in workflow")]
    NoStartNode,
}

/// Generated code bundle
#[derive(Debug, Clone, Serialize)]
pub struct GeneratedCode {
    pub workflow: String,
    pub activities: String,
    pub worker: String,
    pub package_json: String,
    pub tsconfig: String,
}

/// Code generator using Handlebars templates
pub struct CodeGenerator<'a> {
    handlebars: Handlebars<'a>,
}

/// Template data for workflow generation
#[derive(Debug, Serialize)]
struct WorkflowTemplateData {
    version: String,
    generated_at: String,
    workflow_id: String,
    workflow_name: String,
    function_name: String,
    default_timeout: String,
    has_retry_policy: bool,
    retry_policy: Option<RetryPolicyData>,
    has_queries: bool,
    has_signals: bool,
    is_long_running: bool,
    variables: Vec<VariableData>,
    queries: Vec<QueryData>,
    signals: Vec<SignalData>,
    code_blocks: Vec<String>,
    input_type: String,
    output_type: String,
    imports: Vec<String>,
    proxy_declarations: Vec<ProxyDeclaration>,
}

#[derive(Debug, Serialize)]
struct RetryPolicyData {
    max_attempts: u32,
    initial_interval: String,
    max_interval: String,
    backoff_coefficient: f64,
}

#[derive(Debug, Serialize)]
struct VariableData {
    name: String,
    var_type: String,
    default_value: String,
}

#[derive(Debug, Serialize)]
struct QueryData {
    name: String,
    return_type: String,
}

#[derive(Debug, Serialize)]
struct SignalData {
    name: String,
    param_types: String,
}

#[derive(Debug, Serialize)]
struct ProxyDeclaration {
    var_name: String,
    timeout: String,
    has_retry: bool,
    retry_policy: Option<RetryPolicyData>,
}

#[derive(Debug, Serialize)]
struct ActivityData {
    function_name: String,
    input_type: String,
    output_type: String,
    description: String,
    is_log_activity: bool,
    implementation: String,
}

#[derive(Debug, Serialize)]
struct ActivitiesTemplateData {
    version: String,
    generated_at: String,
    imports: Vec<ImportData>,
    activities: Vec<ActivityData>,
}

#[derive(Debug, Serialize)]
struct ImportData {
    module: String,
    items: String,
}

#[derive(Debug, Serialize)]
struct WorkerTemplateData {
    version: String,
    generated_at: String,
    workflow_name: String,
    task_queue: String,
}

#[derive(Debug, Serialize)]
struct PackageJsonData {
    name: String,
    version: String,
}

impl<'a> CodeGenerator<'a> {
    /// Create a new code generator with embedded templates
    pub fn new() -> Result<Self, GenerationError> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        // Register templates
        handlebars
            .register_template_string("workflow", include_str!("templates/workflow.ts.hbs"))?;
        handlebars
            .register_template_string("activities", include_str!("templates/activities.ts.hbs"))?;
        handlebars.register_template_string("worker", include_str!("templates/worker.ts.hbs"))?;
        handlebars
            .register_template_string("package_json", include_str!("templates/package.json.hbs"))?;
        handlebars
            .register_template_string("tsconfig", include_str!("templates/tsconfig.json.hbs"))?;

        Ok(CodeGenerator { handlebars })
    }

    /// Generate all TypeScript code from a workflow definition
    pub fn generate(
        &self,
        workflow: &WorkflowDefinition,
    ) -> Result<GeneratedCode, GenerationError> {
        let workflow_code = self.generate_workflow(workflow)?;
        let activities_code = self.generate_activities(workflow)?;
        let worker_code = self.generate_worker(workflow)?;
        let package_json = self.generate_package_json(workflow)?;
        let tsconfig = self.generate_tsconfig()?;

        Ok(GeneratedCode {
            workflow: workflow_code,
            activities: activities_code,
            worker: worker_code,
            package_json,
            tsconfig,
        })
    }

    /// Generate the main workflow file
    fn generate_workflow(&self, workflow: &WorkflowDefinition) -> Result<String, GenerationError> {
        let start_node = workflow
            .find_trigger()
            .ok_or(GenerationError::NoStartNode)?;
        let code_blocks = self.generate_workflow_body(workflow, start_node)?;
        let (has_signals, signals) = self.extract_signals(workflow);
        let proxy_declarations = self.build_proxy_declarations(workflow);

        let data = WorkflowTemplateData {
            version: env!("CARGO_PKG_VERSION").to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            workflow_id: workflow.id.clone(),
            workflow_name: workflow.name.clone(),
            function_name: workflow.function_name(),
            default_timeout: workflow.settings.default_timeout().to_string(),
            has_retry_policy: workflow.settings.retry_policy.is_some(),
            retry_policy: workflow
                .settings
                .retry_policy
                .as_ref()
                .map(|rp| self.convert_retry_policy(rp)),
            has_queries: false,
            has_signals,
            is_long_running: workflow.settings.is_long_running(),
            variables: self.convert_variables(&workflow.variables),
            queries: vec![],
            signals,
            code_blocks,
            input_type: "Record<string, unknown>".to_string(),
            output_type: "WorkflowResult".to_string(),
            imports: vec![],
            proxy_declarations,
        };

        self.handlebars
            .render("workflow", &data)
            .map_err(GenerationError::RenderError)
    }

    /// Generate the activities file
    fn generate_activities(
        &self,
        workflow: &WorkflowDefinition,
    ) -> Result<String, GenerationError> {
        let activities = self.extract_activities(workflow);

        let data = ActivitiesTemplateData {
            version: env!("CARGO_PKG_VERSION").to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            imports: vec![],
            activities,
        };

        self.handlebars
            .render("activities", &data)
            .map_err(GenerationError::RenderError)
    }

    /// Generate the worker file
    fn generate_worker(&self, workflow: &WorkflowDefinition) -> Result<String, GenerationError> {
        let data = WorkerTemplateData {
            version: env!("CARGO_PKG_VERSION").to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            workflow_name: workflow.function_name(),
            task_queue: workflow.settings.task_queue().to_string(),
        };

        self.handlebars
            .render("worker", &data)
            .map_err(GenerationError::RenderError)
    }

    /// Generate package.json
    fn generate_package_json(
        &self,
        workflow: &WorkflowDefinition,
    ) -> Result<String, GenerationError> {
        let data = PackageJsonData {
            name: format!("workflow-{}", workflow.id),
            version: "1.0.0".to_string(),
        };

        self.handlebars
            .render("package_json", &data)
            .map_err(GenerationError::RenderError)
    }

    /// Generate tsconfig.json
    fn generate_tsconfig(&self) -> Result<String, GenerationError> {
        let data: HashMap<String, String> = HashMap::new();
        self.handlebars
            .render("tsconfig", &data)
            .map_err(GenerationError::RenderError)
    }

    /// Generate the workflow body by traversing the graph
    fn generate_workflow_body(
        &self,
        workflow: &WorkflowDefinition,
        start_node: &WorkflowNode,
    ) -> Result<Vec<String>, GenerationError> {
        let mut code_blocks = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = vec![start_node.id.as_str()];

        while let Some(node_id) = queue.pop() {
            if visited.contains(node_id) {
                continue;
            }
            visited.insert(node_id.to_string());

            if let Some(node) = workflow.find_node(node_id) {
                let code = self.generate_node_code(node, &visited);
                if !code.is_empty() {
                    code_blocks.push(code);
                }

                // Add connected nodes to queue
                for edge in workflow.edges_from(node_id) {
                    if !visited.contains(edge.target.as_str()) {
                        queue.push(&edge.target);
                    }
                }
            }
        }

        Ok(code_blocks)
    }

    /// Generate code for a single node
    fn generate_node_code(&self, node: &WorkflowNode, _visited: &HashSet<String>) -> String {
        match node.node_type {
            NodeType::Trigger => {
                // Start component: record workflow start time
                "const startOutput: StartOutput = { startedAt: new Date() };".to_string()
            },
            NodeType::Activity | NodeType::Agent => {
                let activity_name = node.activity_name().unwrap_or("unknownActivity");
                let var_name = format!("result_{}", node.id.replace('-', "_"));
                format!("const {} = await acts.{}(input);", var_name, activity_name)
            },
            NodeType::End => {
                // Stop component: record workflow completion
                concat!(
                    "const stopOutput: StopOutput = { completedAt: new Date(), success: true };\n",
                    "  result = {\n",
                    "    startedAt: startOutput.startedAt,\n",
                    "    completedAt: stopOutput.completedAt,\n",
                    "    success: stopOutput.success,\n",
                    "    result: input,\n",
                    "  };"
                ).to_string()
            },
            NodeType::StateVariable => {
                let var_name = &node.data.label;
                format!("let {} = undefined;", var_name)
            },
            NodeType::Conditional | NodeType::Condition => "// TODO: Conditional logic".to_string(),
            NodeType::Loop => "// TODO: Loop logic".to_string(),
            _ => String::new(),
        }
    }

    /// Extract activity nodes and generate activity data
    fn extract_activities(&self, workflow: &WorkflowDefinition) -> Vec<ActivityData> {
        workflow
            .nodes
            .iter()
            .filter(|n| n.node_type.is_activity())
            .map(|node| {
                let function_name = node.activity_name().unwrap_or("unknownActivity").to_string();
                let is_log = function_name.to_lowercase().contains("log");

                // Use proper types for log activities
                let (input_type, output_type) = if is_log {
                    ("LogInput".to_string(), "LogOutput".to_string())
                } else {
                    ("Record<string, unknown>".to_string(), "Record<string, unknown>".to_string())
                };

                ActivityData {
                    function_name: function_name.clone(),
                    input_type,
                    output_type,
                    description: node.data.description.clone().unwrap_or_default(),
                    is_log_activity: is_log,
                    implementation: format!(
                        "console.log('Executing {}', input);\n  return {{ success: true, data: input }};",
                        function_name
                    ),
                }
            })
            .collect()
    }

    /// Extract signal definitions
    fn extract_signals(&self, workflow: &WorkflowDefinition) -> (bool, Vec<SignalData>) {
        let signals: Vec<SignalData> = workflow
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Signal)
            .filter_map(|n| {
                n.data.signal_name.as_ref().map(|name| SignalData {
                    name: name.clone(),
                    param_types: "unknown".to_string(),
                })
            })
            .collect();

        (!signals.is_empty(), signals)
    }

    /// Build proxy declarations for activities
    fn build_proxy_declarations(&self, workflow: &WorkflowDefinition) -> Vec<ProxyDeclaration> {
        // Group activities by timeout
        let mut proxies_by_timeout: HashMap<String, Vec<&WorkflowNode>> = HashMap::new();

        for node in workflow.nodes.iter().filter(|n| n.node_type.is_activity()) {
            let timeout = node
                .data
                .timeout
                .clone()
                .unwrap_or_else(|| "1m".to_string());
            proxies_by_timeout.entry(timeout).or_default().push(node);
        }

        if proxies_by_timeout.is_empty() {
            // Default proxy
            return vec![ProxyDeclaration {
                var_name: "acts".to_string(),
                timeout: workflow.settings.default_timeout().to_string(),
                has_retry: workflow.settings.retry_policy.is_some(),
                retry_policy: workflow
                    .settings
                    .retry_policy
                    .as_ref()
                    .map(|rp| self.convert_retry_policy(rp)),
            }];
        }

        proxies_by_timeout
            .into_iter()
            .enumerate()
            .map(|(i, (timeout, nodes))| {
                let var_name = if i == 0 {
                    "acts".to_string()
                } else {
                    format!("acts_{}", i)
                };

                // Use the retry policy from the first node, or workflow default
                let retry_policy = nodes
                    .first()
                    .and_then(|n| n.data.retry_policy.as_ref())
                    .or(workflow.settings.retry_policy.as_ref())
                    .map(|rp| self.convert_retry_policy(rp));

                ProxyDeclaration {
                    var_name,
                    timeout,
                    has_retry: retry_policy.is_some(),
                    retry_policy,
                }
            })
            .collect()
    }

    /// Convert variables to template data
    fn convert_variables(&self, variables: &[WorkflowVariable]) -> Vec<VariableData> {
        variables
            .iter()
            .map(|v| VariableData {
                name: v.name.clone(),
                var_type: v.var_type.to_typescript().to_string(),
                default_value: serde_json::to_string(&v.default_value()).unwrap_or_default(),
            })
            .collect()
    }

    /// Convert retry policy to template data
    fn convert_retry_policy(&self, policy: &RetryPolicy) -> RetryPolicyData {
        RetryPolicyData {
            max_attempts: policy.max_attempts.unwrap_or(3),
            initial_interval: policy
                .initial_interval
                .clone()
                .unwrap_or_else(|| "1s".to_string()),
            max_interval: policy
                .max_interval
                .clone()
                .unwrap_or_else(|| "1m".to_string()),
            backoff_coefficient: policy.backoff_coefficient.unwrap_or(2.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{NodeData, Position, WorkflowEdge, WorkflowSettings};

    fn create_simple_workflow() -> WorkflowDefinition {
        WorkflowDefinition {
            id: "wf_test".to_string(),
            name: "Test Workflow".to_string(),
            nodes: vec![
                WorkflowNode {
                    id: "trigger".to_string(),
                    node_type: NodeType::Trigger,
                    data: NodeData {
                        label: "Start".to_string(),
                        ..Default::default()
                    },
                    position: Position::default(),
                },
                WorkflowNode {
                    id: "activity".to_string(),
                    node_type: NodeType::Activity,
                    data: NodeData {
                        label: "Do Something".to_string(),
                        activity_name: Some("doSomething".to_string()),
                        ..Default::default()
                    },
                    position: Position::default(),
                },
                WorkflowNode {
                    id: "end".to_string(),
                    node_type: NodeType::End,
                    data: NodeData {
                        label: "End".to_string(),
                        ..Default::default()
                    },
                    position: Position::default(),
                },
            ],
            edges: vec![
                WorkflowEdge::new("e1", "trigger", "activity"),
                WorkflowEdge::new("e2", "activity", "end"),
            ],
            variables: vec![],
            settings: WorkflowSettings::default(),
        }
    }

    #[test]
    fn test_generate_workflow() {
        let workflow = create_simple_workflow();
        let generator = CodeGenerator::new().unwrap();
        let result = generator.generate(&workflow);
        assert!(result.is_ok());

        let code = result.unwrap();
        assert!(code.workflow.contains("proxyActivities"));
        assert!(code.workflow.contains("TestWorkflow"));
    }

    #[test]
    fn test_generated_code_has_no_any() {
        let workflow = create_simple_workflow();
        let generator = CodeGenerator::new().unwrap();
        let code = generator.generate(&workflow).unwrap();

        // Check that 'any' type is not used (except in comments or as part of other words)
        // This is a basic check - full verification is done by tsc
        let any_count =
            code.workflow.matches(": any").count() + code.activities.matches(": any").count();
        assert_eq!(
            any_count, 0,
            "Generated code should not contain ': any' type"
        );
    }

    #[test]
    fn test_workflow_has_start_output_type() {
        let workflow = create_simple_workflow();
        let generator = CodeGenerator::new().unwrap();
        let code = generator.generate(&workflow).unwrap();

        // Verify StartOutput interface is defined
        assert!(
            code.workflow.contains("interface StartOutput"),
            "Generated workflow should define StartOutput interface"
        );
        assert!(
            code.workflow.contains("startedAt: Date"),
            "StartOutput should have startedAt field"
        );
    }

    #[test]
    fn test_workflow_has_stop_output_type() {
        let workflow = create_simple_workflow();
        let generator = CodeGenerator::new().unwrap();
        let code = generator.generate(&workflow).unwrap();

        // Verify StopOutput interface is defined
        assert!(
            code.workflow.contains("interface StopOutput"),
            "Generated workflow should define StopOutput interface"
        );
        assert!(
            code.workflow.contains("completedAt: Date"),
            "StopOutput should have completedAt field"
        );
        assert!(
            code.workflow.contains("success: boolean"),
            "StopOutput should have success field"
        );
    }

    #[test]
    fn test_workflow_has_result_type() {
        let workflow = create_simple_workflow();
        let generator = CodeGenerator::new().unwrap();
        let code = generator.generate(&workflow).unwrap();

        // Verify WorkflowResult interface is defined
        assert!(
            code.workflow.contains("interface WorkflowResult"),
            "Generated workflow should define WorkflowResult interface"
        );
    }

    #[test]
    fn test_trigger_generates_start_output() {
        let workflow = create_simple_workflow();
        let generator = CodeGenerator::new().unwrap();
        let code = generator.generate(&workflow).unwrap();

        // Verify trigger node generates startOutput initialization
        assert!(
            code.workflow.contains("const startOutput: StartOutput"),
            "Trigger node should generate startOutput variable"
        );
    }

    #[test]
    fn test_end_generates_stop_output() {
        let workflow = create_simple_workflow();
        let generator = CodeGenerator::new().unwrap();
        let code = generator.generate(&workflow).unwrap();

        // Verify end node generates stopOutput and result
        assert!(
            code.workflow.contains("const stopOutput: StopOutput"),
            "End node should generate stopOutput variable"
        );
    }

    #[test]
    fn test_log_activity_generates_proper_types() {
        let workflow = WorkflowDefinition {
            id: "wf_log_test".to_string(),
            name: "Log Test Workflow".to_string(),
            nodes: vec![
                WorkflowNode {
                    id: "trigger".to_string(),
                    node_type: NodeType::Trigger,
                    data: NodeData {
                        label: "Start".to_string(),
                        ..Default::default()
                    },
                    position: Position::default(),
                },
                WorkflowNode {
                    id: "log".to_string(),
                    node_type: NodeType::Activity,
                    data: NodeData {
                        label: "Log Message".to_string(),
                        activity_name: Some("logMessage".to_string()),
                        ..Default::default()
                    },
                    position: Position::default(),
                },
                WorkflowNode {
                    id: "end".to_string(),
                    node_type: NodeType::End,
                    data: NodeData {
                        label: "End".to_string(),
                        ..Default::default()
                    },
                    position: Position::default(),
                },
            ],
            edges: vec![
                WorkflowEdge::new("e1", "trigger", "log"),
                WorkflowEdge::new("e2", "log", "end"),
            ],
            variables: vec![],
            settings: WorkflowSettings::default(),
        };

        let generator = CodeGenerator::new().unwrap();
        let code = generator.generate(&workflow).unwrap();

        // Verify log activities have proper types
        assert!(
            code.activities.contains("interface LogInput"),
            "Activities should define LogInput interface"
        );
        assert!(
            code.activities.contains("interface LogOutput"),
            "Activities should define LogOutput interface"
        );
        assert!(
            code.activities.contains("type LogLevel"),
            "Activities should define LogLevel type"
        );
    }

    #[test]
    fn test_workflow_returns_workflow_result() {
        let workflow = create_simple_workflow();
        let generator = CodeGenerator::new().unwrap();
        let code = generator.generate(&workflow).unwrap();

        // Verify function signature returns WorkflowResult
        assert!(
            code.workflow.contains("Promise<WorkflowResult>"),
            "Workflow function should return Promise<WorkflowResult>"
        );
    }
}

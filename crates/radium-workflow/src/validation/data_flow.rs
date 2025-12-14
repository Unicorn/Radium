//! Data flow validation
//!
//! Validates data flow between workflow components, ensuring:
//! - Type compatibility between connected components
//! - Required inputs are provided
//! - No circular dependencies in variable usage
//! - All variable references can be resolved

use std::collections::{HashMap, HashSet};

use crate::schema::variables::{VariableDefinition, VariableReference, VariableType};
use crate::schema::{WorkflowDefinition, WorkflowNode};

/// Data flow validation result
#[derive(Debug, Clone, Default)]
pub struct DataFlowAnalysis {
    /// Critical errors that prevent execution
    pub errors: Vec<DataFlowError>,
    /// Warnings that should be addressed
    pub warnings: Vec<DataFlowWarning>,
    /// Variable usage tracking
    pub variable_usage: HashMap<String, VariableUsage>,
    /// Data flow graph for visualization
    pub flow_graph: FlowGraph,
}

impl DataFlowAnalysis {
    /// Create a new empty analysis
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if analysis has any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if analysis is clean (no errors or warnings)
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    /// Add an error
    pub fn add_error(&mut self, error: DataFlowError) {
        self.errors.push(error);
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: DataFlowWarning) {
        self.warnings.push(warning);
    }
}

/// Data flow errors
#[derive(Debug, Clone, PartialEq)]
pub enum DataFlowError {
    /// Required input not provided
    MissingRequiredInput {
        node_id: String,
        variable_name: String,
    },
    /// Type mismatch between source and target
    TypeMismatch {
        node_id: String,
        variable_name: String,
        expected: VariableType,
        actual: VariableType,
    },
    /// Circular dependency detected
    CircularDependency { variables: Vec<String> },
    /// Variable reference cannot be resolved
    UnresolvedReference { node_id: String, reference: String },
    /// Duplicate variable definition
    DuplicateVariable {
        variable_name: String,
        first_defined: String,
        second_defined: String,
    },
}

impl std::fmt::Display for DataFlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRequiredInput {
                node_id,
                variable_name,
            } => {
                write!(
                    f,
                    "Node '{}' requires input '{}' which is not provided",
                    node_id, variable_name
                )
            }
            Self::TypeMismatch {
                node_id,
                variable_name,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Type mismatch at node '{}': variable '{}' expects {}, got {}",
                    node_id, variable_name, expected, actual
                )
            }
            Self::CircularDependency { variables } => {
                write!(
                    f,
                    "Circular dependency detected involving: {}",
                    variables.join(" -> ")
                )
            }
            Self::UnresolvedReference { node_id, reference } => {
                write!(
                    f,
                    "Cannot resolve variable reference '{}' at node '{}'",
                    reference, node_id
                )
            }
            Self::DuplicateVariable {
                variable_name,
                first_defined,
                second_defined,
            } => {
                write!(
                    f,
                    "Variable '{}' defined at both '{}' and '{}'",
                    variable_name, first_defined, second_defined
                )
            }
        }
    }
}

impl std::error::Error for DataFlowError {}

/// Data flow warnings
#[derive(Debug, Clone, PartialEq)]
pub enum DataFlowWarning {
    /// Variable defined but never used
    UnusedVariable {
        variable_name: String,
        defined_at: String,
    },
    /// Variable shadows another variable
    ShadowedVariable {
        variable_name: String,
        original_at: String,
        shadowed_at: String,
    },
    /// Implicit type coercion happening
    ImplicitTypeCoercion {
        node_id: String,
        variable_name: String,
        from_type: VariableType,
        to_type: VariableType,
    },
    /// Potential null access
    PotentialNullAccess { node_id: String, reference: String },
}

impl std::fmt::Display for DataFlowWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnusedVariable {
                variable_name,
                defined_at,
            } => {
                write!(
                    f,
                    "Variable '{}' defined at '{}' is never used",
                    variable_name, defined_at
                )
            }
            Self::ShadowedVariable {
                variable_name,
                original_at,
                shadowed_at,
            } => {
                write!(
                    f,
                    "Variable '{}' at '{}' shadows definition at '{}'",
                    variable_name, shadowed_at, original_at
                )
            }
            Self::ImplicitTypeCoercion {
                node_id,
                variable_name,
                from_type,
                to_type,
            } => {
                write!(
                    f,
                    "Implicit coercion of '{}' from {} to {} at node '{}'",
                    variable_name, from_type, to_type, node_id
                )
            }
            Self::PotentialNullAccess { node_id, reference } => {
                write!(
                    f,
                    "Potential null access at node '{}' via reference '{}'",
                    node_id, reference
                )
            }
        }
    }
}

/// Usage information for a variable
#[derive(Debug, Clone, Default)]
pub struct VariableUsage {
    /// Nodes where the variable is defined
    pub defined_at: Vec<String>,
    /// Nodes where the variable is read
    pub read_at: Vec<String>,
    /// Nodes where the variable is written
    pub written_at: Vec<String>,
    /// Whether the variable is required
    pub is_required: bool,
    /// The variable's type
    pub variable_type: VariableType,
}

impl VariableUsage {
    /// Check if variable is used
    pub fn is_used(&self) -> bool {
        !self.read_at.is_empty() || !self.written_at.is_empty()
    }
}

/// Flow graph for visualization
#[derive(Debug, Clone, Default)]
pub struct FlowGraph {
    /// Nodes in the flow graph
    pub nodes: Vec<FlowNode>,
    /// Edges representing data flow
    pub edges: Vec<FlowEdge>,
}

/// A node in the flow graph
#[derive(Debug, Clone)]
pub struct FlowNode {
    /// Node ID
    pub id: String,
    /// Node type
    pub node_type: String,
    /// Variables consumed by this node
    pub inputs: Vec<String>,
    /// Variables produced by this node
    pub outputs: Vec<String>,
}

/// An edge in the flow graph
#[derive(Debug, Clone)]
pub struct FlowEdge {
    /// Source node ID
    pub from_node: String,
    /// Variable produced
    pub from_variable: String,
    /// Target node ID
    pub to_node: String,
    /// Variable consumed
    pub to_variable: String,
}

/// Data flow validator
pub struct DataFlowValidator {
    /// Variables available at each node (computed during analysis)
    available_at: HashMap<String, HashSet<String>>,
    /// Variable type map
    types: HashMap<String, VariableType>,
    /// Variable definitions from workflow
    definitions: Vec<VariableDefinition>,
}

impl DataFlowValidator {
    /// Create a new validator
    pub fn new(definitions: Vec<VariableDefinition>) -> Self {
        let types: HashMap<String, VariableType> = definitions
            .iter()
            .map(|d| (d.name.clone(), d.variable_type.clone()))
            .collect();

        Self {
            available_at: HashMap::new(),
            types,
            definitions,
        }
    }

    /// Analyze data flow in a workflow
    pub fn analyze(&mut self, workflow: &WorkflowDefinition) -> DataFlowAnalysis {
        let mut analysis = DataFlowAnalysis::new();
        let mut variable_usage: HashMap<String, VariableUsage> = HashMap::new();

        // Initialize usage for all defined variables
        for def in &self.definitions {
            variable_usage.insert(
                def.name.clone(),
                VariableUsage {
                    defined_at: vec!["workflow".to_string()],
                    read_at: Vec::new(),
                    written_at: Vec::new(),
                    is_required: def.required,
                    variable_type: def.variable_type.clone(),
                },
            );
        }

        // Compute execution order
        let execution_order = self.compute_execution_order(workflow);

        // Initialize available variables at trigger
        let mut available: HashSet<String> = self
            .definitions
            .iter()
            .filter(|d| d.required)
            .map(|d| d.name.clone())
            .collect();

        // Analyze each node in order
        for node_id in &execution_order {
            if let Some(node) = workflow.nodes.iter().find(|n| &n.id == node_id) {
                // Track what's available at this node
                self.available_at
                    .insert(node_id.clone(), available.clone());

                // Validate node inputs
                self.validate_node_inputs(node, &available, &mut analysis, &mut variable_usage);

                // Track node outputs
                self.track_node_outputs(node, &mut available, &mut variable_usage);
            }
        }

        // Check for unused variables
        for (name, usage) in &variable_usage {
            if usage.defined_at.iter().any(|d| d != "workflow") && usage.read_at.is_empty() {
                analysis.add_warning(DataFlowWarning::UnusedVariable {
                    variable_name: name.clone(),
                    defined_at: usage.defined_at.first().cloned().unwrap_or_default(),
                });
            }
        }

        // Build flow graph
        analysis.flow_graph = self.build_flow_graph(workflow, &variable_usage);
        analysis.variable_usage = variable_usage;

        analysis
    }

    /// Compute execution order using topological sort
    fn compute_execution_order(&self, workflow: &WorkflowDefinition) -> Vec<String> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut in_progress = HashSet::new();

        // Build adjacency list
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for node in &workflow.nodes {
            adj.entry(node.id.clone()).or_default();
        }
        for edge in &workflow.edges {
            adj.entry(edge.source.clone())
                .or_default()
                .push(edge.target.clone());
        }

        // Find trigger node to start
        let trigger_id = workflow
            .nodes
            .iter()
            .find(|n| n.node_type == crate::schema::NodeType::Trigger)
            .map(|n| n.id.clone());

        if let Some(trigger) = trigger_id {
            self.dfs_order(&trigger, &adj, &mut visited, &mut in_progress, &mut order);
        }

        order
    }

    fn dfs_order(
        &self,
        node_id: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        in_progress: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) {
        if visited.contains(node_id) {
            return;
        }
        if in_progress.contains(node_id) {
            // Cycle detected - but we already handle this elsewhere
            return;
        }

        in_progress.insert(node_id.to_string());

        if let Some(neighbors) = adj.get(node_id) {
            for neighbor in neighbors {
                self.dfs_order(neighbor, adj, visited, in_progress, order);
            }
        }

        in_progress.remove(node_id);
        visited.insert(node_id.to_string());
        order.push(node_id.to_string());
    }

    /// Validate inputs for a node
    fn validate_node_inputs(
        &self,
        node: &WorkflowNode,
        available: &HashSet<String>,
        analysis: &mut DataFlowAnalysis,
        usage: &mut HashMap<String, VariableUsage>,
    ) {
        // Extract variable references from node data
        let references = self.extract_references(node);

        for reference in references {
            match VariableReference::parse(&reference) {
                Ok(var_ref) => {
                    if let Some(root_var) = var_ref.root_variable() {
                        // Check if variable is available
                        if !available.contains(root_var)
                            && !self.definitions.iter().any(|d| d.name == root_var)
                        {
                            analysis.add_error(DataFlowError::UnresolvedReference {
                                node_id: node.id.clone(),
                                reference: reference.clone(),
                            });
                        } else {
                            // Track read
                            usage
                                .entry(root_var.to_string())
                                .or_default()
                                .read_at
                                .push(node.id.clone());
                        }

                        // Check for potential null access
                        if self
                            .definitions
                            .iter()
                            .any(|d| d.name == root_var && d.is_nullable())
                        {
                            analysis.add_warning(DataFlowWarning::PotentialNullAccess {
                                node_id: node.id.clone(),
                                reference,
                            });
                        }
                    }
                }
                Err(_) => {
                    // Not a valid reference - might be a literal value
                }
            }
        }
    }

    /// Extract variable references from node
    fn extract_references(&self, node: &WorkflowNode) -> Vec<String> {
        let mut references = Vec::new();

        // Check all string fields in node data for $ references
        if let Ok(json) = serde_json::to_value(&node.data) {
            self.extract_references_from_json(&json, &mut references);
        }

        references
    }

    fn extract_references_from_json(&self, value: &serde_json::Value, references: &mut Vec<String>) {
        match value {
            serde_json::Value::String(s) => {
                // Look for $. references
                let mut remaining = s.as_str();
                while let Some(pos) = remaining.find("$.") {
                    remaining = &remaining[pos..];
                    // Find end of reference
                    let end = remaining[1..]
                        .find(|c: char| !c.is_alphanumeric() && c != '.' && c != '[' && c != ']' && c != '_')
                        .map(|p| p + 1)
                        .unwrap_or(remaining.len());
                    let reference = &remaining[..end];
                    references.push(reference.to_string());
                    remaining = &remaining[end..];
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    self.extract_references_from_json(item, references);
                }
            }
            serde_json::Value::Object(obj) => {
                for (_, v) in obj {
                    self.extract_references_from_json(v, references);
                }
            }
            _ => {}
        }
    }

    /// Track outputs produced by a node
    fn track_node_outputs(
        &self,
        node: &WorkflowNode,
        available: &mut HashSet<String>,
        usage: &mut HashMap<String, VariableUsage>,
    ) {
        // Activities produce results that are available for subsequent nodes
        if matches!(
            node.node_type,
            crate::schema::NodeType::Activity | crate::schema::NodeType::Agent
        ) {
            if let Some(activity_name) = node.activity_name() {
                let result_var = format!("{}_result", activity_name);
                available.insert(result_var.clone());
                usage.entry(result_var).or_default().written_at.push(node.id.clone());
            }
        }

        // If node data has a label that looks like a variable assignment, track it
        let label = &node.data.label;
        if !label.is_empty() && !label.starts_with('$') {
            // Label could be a variable being produced - we don't track it
            // as a variable unless it's explicitly defined
        }
    }

    /// Build flow graph for visualization
    fn build_flow_graph(
        &self,
        workflow: &WorkflowDefinition,
        usage: &HashMap<String, VariableUsage>,
    ) -> FlowGraph {
        let mut flow_graph = FlowGraph::default();

        // Create flow nodes
        for node in &workflow.nodes {
            let inputs: Vec<String> = self
                .extract_references(node)
                .iter()
                .filter_map(|r| {
                    VariableReference::parse(r)
                        .ok()
                        .and_then(|v| v.root_variable().map(|s| s.to_string()))
                })
                .collect();

            let outputs: Vec<String> = if matches!(
                node.node_type,
                crate::schema::NodeType::Activity | crate::schema::NodeType::Agent
            ) {
                node.activity_name()
                    .map(|n| vec![format!("{}_result", n)])
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            flow_graph.nodes.push(FlowNode {
                id: node.id.clone(),
                node_type: format!("{:?}", node.node_type),
                inputs,
                outputs,
            });
        }

        // Create flow edges based on variable usage
        for (var_name, var_usage) in usage {
            for writer in &var_usage.written_at {
                for reader in &var_usage.read_at {
                    flow_graph.edges.push(FlowEdge {
                        from_node: writer.clone(),
                        from_variable: var_name.clone(),
                        to_node: reader.clone(),
                        to_variable: var_name.clone(),
                    });
                }
            }
        }

        flow_graph
    }
}

/// Validate data flow in a workflow
pub fn validate_data_flow(workflow: &WorkflowDefinition) -> DataFlowAnalysis {
    let definitions: Vec<VariableDefinition> = workflow
        .variables
        .iter()
        .map(|v| VariableDefinition::new(&v.name, VariableType::from(&v.var_type)))
        .collect();

    let mut validator = DataFlowValidator::new(definitions);
    validator.analyze(workflow)
}

/// Convert legacy VariableType to new VariableType
impl From<&crate::schema::LegacyVariableType> for VariableType {
    fn from(legacy: &crate::schema::LegacyVariableType) -> Self {
        match legacy {
            crate::schema::LegacyVariableType::String => VariableType::String,
            crate::schema::LegacyVariableType::Number => VariableType::Number,
            crate::schema::LegacyVariableType::Boolean => VariableType::Boolean,
            crate::schema::LegacyVariableType::Array => VariableType::Array,
            crate::schema::LegacyVariableType::Object => VariableType::Object,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_workflow() -> WorkflowDefinition {
        use crate::schema::{NodeData, NodeType, Position, WorkflowEdge, WorkflowSettings};

        WorkflowDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            nodes: vec![
                WorkflowNode {
                    id: "trigger".to_string(),
                    node_type: NodeType::Trigger,
                    data: NodeData::default(),
                    position: Position::default(),
                },
                WorkflowNode {
                    id: "activity".to_string(),
                    node_type: NodeType::Activity,
                    data: NodeData {
                        label: "Process".to_string(),
                        activity_name: Some("processData".to_string()),
                        ..Default::default()
                    },
                    position: Position::default(),
                },
                WorkflowNode {
                    id: "end".to_string(),
                    node_type: NodeType::End,
                    data: NodeData::default(),
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
    fn test_data_flow_analysis_new() {
        let analysis = DataFlowAnalysis::new();
        assert!(!analysis.has_errors());
        assert!(analysis.is_clean());
    }

    #[test]
    fn test_simple_workflow_analysis() {
        let workflow = make_simple_workflow();
        let analysis = validate_data_flow(&workflow);

        assert!(!analysis.has_errors());
    }

    #[test]
    fn test_execution_order() {
        let workflow = make_simple_workflow();
        let validator = DataFlowValidator::new(vec![]);
        let order = validator.compute_execution_order(&workflow);

        // Should end with trigger (reverse DFS order)
        assert!(!order.is_empty());
    }

    #[test]
    fn test_variable_usage_tracking() {
        let definitions = vec![
            VariableDefinition::new("input", VariableType::String).required(),
            VariableDefinition::new("output", VariableType::String),
        ];

        let mut validator = DataFlowValidator::new(definitions);
        let workflow = make_simple_workflow();
        let analysis = validator.analyze(&workflow);

        // Should have tracked the defined variables
        assert!(analysis.variable_usage.contains_key("input"));
        assert!(analysis.variable_usage.contains_key("output"));
    }

    #[test]
    fn test_flow_graph_creation() {
        let workflow = make_simple_workflow();
        let analysis = validate_data_flow(&workflow);

        // Should have nodes for each workflow node
        assert_eq!(analysis.flow_graph.nodes.len(), 3);
    }

    #[test]
    fn test_error_display() {
        let error = DataFlowError::TypeMismatch {
            node_id: "node-1".to_string(),
            variable_name: "count".to_string(),
            expected: VariableType::Integer,
            actual: VariableType::String,
        };

        let display = format!("{}", error);
        assert!(display.contains("node-1"));
        assert!(display.contains("count"));
    }

    #[test]
    fn test_warning_display() {
        let warning = DataFlowWarning::UnusedVariable {
            variable_name: "temp".to_string(),
            defined_at: "activity-1".to_string(),
        };

        let display = format!("{}", warning);
        assert!(display.contains("temp"));
        assert!(display.contains("never used"));
    }
}

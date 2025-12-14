//! Query Handlers
//!
//! Implement query handlers for workflow state inspection:
//! - Query definitions with typed inputs/outputs
//! - State projections
//! - Computed queries
//! - Standard queries (getState, getProgress)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// Schema definition for query inputs/outputs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QuerySchema {
    /// Schema fields
    #[serde(default)]
    pub fields: Vec<QuerySchemaField>,
}

/// A field in a query schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySchemaField {
    /// Field name
    pub name: String,
    /// TypeScript type
    pub typescript_type: String,
    /// Whether the field is required
    #[serde(default = "default_true")]
    pub required: bool,
    /// Description of the field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn default_true() -> bool {
    true
}

impl QuerySchema {
    /// Create a schema that accepts any value
    pub fn any() -> Self {
        Self { fields: vec![] }
    }

    /// Create a schema with specific fields
    pub fn with_fields(fields: Vec<QuerySchemaField>) -> Self {
        Self { fields }
    }

    /// Create a simple object schema from field tuples
    pub fn object(fields: Vec<(&str, &str)>) -> Self {
        Self {
            fields: fields
                .into_iter()
                .map(|(name, ts_type)| QuerySchemaField {
                    name: name.to_string(),
                    typescript_type: ts_type.to_string(),
                    required: true,
                    description: None,
                })
                .collect(),
        }
    }

    /// Generate TypeScript interface
    pub fn to_typescript_interface(&self, name: &str) -> String {
        if self.fields.is_empty() {
            return format!("type {} = unknown;", name);
        }

        let mut code = format!("interface {} {{\n", name);
        for field in &self.fields {
            if let Some(desc) = &field.description {
                code.push_str(&format!("  /** {} */\n", desc));
            }
            let optional = if field.required { "" } else { "?" };
            code.push_str(&format!(
                "  {}{}: {};\n",
                field.name, optional, field.typescript_type
            ));
        }
        code.push_str("}\n");
        code
    }

    /// Check if this is an empty/void schema
    pub fn is_void(&self) -> bool {
        self.fields.is_empty()
    }
}

/// Logic for query handlers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum QueryHandlerLogic {
    /// Return specific state variables
    StateProjection {
        /// Variable names to return (* for all)
        variables: Vec<String>,
    },
    /// Return a computed value from an expression
    Computed {
        /// TypeScript expression to evaluate
        expression: String,
    },
    /// Custom handler code
    Custom {
        /// TypeScript code for the handler
        code: String,
    },
}

impl QueryHandlerLogic {
    /// Create a state projection
    pub fn project(variables: Vec<impl Into<String>>) -> Self {
        QueryHandlerLogic::StateProjection {
            variables: variables.into_iter().map(|v| v.into()).collect(),
        }
    }

    /// Create a computed query
    pub fn computed(expression: impl Into<String>) -> Self {
        QueryHandlerLogic::Computed {
            expression: expression.into(),
        }
    }

    /// Create a custom handler
    pub fn custom(code: impl Into<String>) -> Self {
        QueryHandlerLogic::Custom { code: code.into() }
    }

    /// Convert to TypeScript handler body
    pub fn to_typescript(&self) -> String {
        match self {
            QueryHandlerLogic::StateProjection { variables } => {
                if variables.len() == 1 && variables[0] == "*" {
                    "return { ...state.variables };".to_string()
                } else {
                    let fields: Vec<_> = variables
                        .iter()
                        .map(|v| format!("{}: state.variables.{}", v, v))
                        .collect();
                    format!("return {{ {} }};", fields.join(", "))
                }
            }
            QueryHandlerLogic::Computed { expression } => {
                format!("return {};", expression)
            }
            QueryHandlerLogic::Custom { code } => code.clone(),
        }
    }
}

/// Query definition for workflow state inspection
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct QueryDefinition {
    /// Query name
    #[validate(length(min = 1, message = "Query name is required"))]
    pub name: String,

    /// Description of the query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Input schema (query parameters)
    #[serde(default)]
    pub input_schema: QuerySchema,

    /// Output schema (response type)
    pub output_schema: QuerySchema,

    /// Handler logic
    pub handler: QueryHandlerLogic,
}

impl QueryDefinition {
    /// Create a new query definition
    pub fn new(name: impl Into<String>, output_schema: QuerySchema, handler: QueryHandlerLogic) -> Self {
        Self {
            name: name.into(),
            description: None,
            input_schema: QuerySchema::any(),
            output_schema,
            handler,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set input schema
    pub fn with_input_schema(mut self, schema: QuerySchema) -> Self {
        self.input_schema = schema;
        self
    }

    /// Generate TypeScript input type name
    pub fn typescript_input_type(&self) -> String {
        if self.input_schema.is_void() {
            "void".to_string()
        } else {
            format!("{}QueryInput", to_pascal_case(&self.name))
        }
    }

    /// Generate TypeScript output type name
    pub fn typescript_output_type(&self) -> String {
        if self.output_schema.is_void() {
            "unknown".to_string()
        } else {
            format!("{}QueryOutput", to_pascal_case(&self.name))
        }
    }

    /// Generate TypeScript query definition and handler
    pub fn to_typescript(&self) -> String {
        let mut code = String::new();

        // Add description
        if let Some(desc) = &self.description {
            code.push_str(&format!("/** {} */\n", desc));
        }

        // Generate input interface if needed
        if !self.input_schema.is_void() {
            code.push_str(&self.input_schema.to_typescript_interface(&self.typescript_input_type()));
            code.push('\n');
        }

        // Generate output interface
        if !self.output_schema.is_void() {
            code.push_str(&self.output_schema.to_typescript_interface(&self.typescript_output_type()));
            code.push('\n');
        }

        // Generate query definition
        let input_type = self.typescript_input_type();
        let output_type = self.typescript_output_type();

        if input_type == "void" {
            code.push_str(&format!(
                "export const {}Query = defineQuery<{}>('{}');\n\n",
                to_camel_case(&self.name),
                output_type,
                self.name
            ));
        } else {
            code.push_str(&format!(
                "export const {}Query = defineQuery<{}, [{}]>('{}');\n\n",
                to_camel_case(&self.name),
                output_type,
                input_type,
                self.name
            ));
        }

        // Generate handler
        let input_param = if input_type == "void" {
            ""
        } else {
            "input"
        };

        code.push_str(&format!(
            "setHandler({}Query, ({}): {} => {{\n  {}\n}});\n",
            to_camel_case(&self.name),
            input_param,
            output_type,
            self.handler.to_typescript()
        ));

        code
    }
}

/// Collection of queries for a workflow
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowQueries {
    /// All query definitions
    #[serde(default)]
    pub queries: HashMap<String, QueryDefinition>,
}

impl WorkflowQueries {
    /// Create empty query collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with standard queries
    pub fn with_standard_queries() -> Self {
        let mut queries = Self::new();

        for query in standard_queries() {
            queries.add(query);
        }

        queries
    }

    /// Add a query
    pub fn add(&mut self, query: QueryDefinition) {
        self.queries.insert(query.name.clone(), query);
    }

    /// Get a query by name
    pub fn get(&self, name: &str) -> Option<&QueryDefinition> {
        self.queries.get(name)
    }

    /// Generate TypeScript for all queries
    pub fn to_typescript(&self) -> String {
        let mut code = String::new();

        code.push_str("// Query definitions and handlers\n");
        code.push_str("import { defineQuery, setHandler } from '@temporalio/workflow';\n\n");

        for query in self.queries.values() {
            code.push_str(&query.to_typescript());
            code.push('\n');
        }

        code
    }
}

/// Standard queries available on all workflows
pub fn standard_queries() -> Vec<QueryDefinition> {
    vec![
        QueryDefinition::new(
            "getState",
            QuerySchema::any(),
            QueryHandlerLogic::project(vec!["*"]),
        )
        .with_description("Get the current workflow state"),

        QueryDefinition::new(
            "getProgress",
            QuerySchema::object(vec![
                ("completedSteps", "string[]"),
                ("currentStep", "string | null"),
                ("percentComplete", "number"),
            ]),
            QueryHandlerLogic::custom(
                r#"return {
    completedSteps: state.progress?.completedSteps ?? [],
    currentStep: state.progress?.currentStep ?? null,
    percentComplete: calculateProgress(state),
  };"#,
            ),
        )
        .with_description("Get workflow progress information"),

        QueryDefinition::new(
            "getStatus",
            QuerySchema::object(vec![
                ("status", "string"),
                ("startedAt", "string"),
                ("lastActivityAt", "string | null"),
            ]),
            QueryHandlerLogic::custom(
                r#"return {
    status: state.status ?? 'unknown',
    startedAt: state.startedAt ?? new Date().toISOString(),
    lastActivityAt: state.lastActivityAt ?? null,
  };"#,
            ),
        )
        .with_description("Get workflow status information"),
    ]
}

// Helper functions
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, c) in s.chars().enumerate() {
        if c == '-' || c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else if i == 0 {
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

fn to_pascal_case(s: &str) -> String {
    let camel = to_camel_case(s);
    let mut chars = camel.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_schema_interface() {
        let schema = QuerySchema::object(vec![
            ("name", "string"),
            ("count", "number"),
        ]);

        let ts = schema.to_typescript_interface("TestOutput");
        assert!(ts.contains("interface TestOutput"));
        assert!(ts.contains("name: string"));
        assert!(ts.contains("count: number"));
    }

    #[test]
    fn test_query_handler_logic_projection() {
        let logic = QueryHandlerLogic::project(vec!["name", "status"]);
        let ts = logic.to_typescript();
        assert!(ts.contains("name: state.variables.name"));
        assert!(ts.contains("status: state.variables.status"));
    }

    #[test]
    fn test_query_handler_logic_all_projection() {
        let logic = QueryHandlerLogic::project(vec!["*"]);
        let ts = logic.to_typescript();
        assert!(ts.contains("...state.variables"));
    }

    #[test]
    fn test_query_handler_logic_computed() {
        let logic = QueryHandlerLogic::computed("state.variables.items.length");
        let ts = logic.to_typescript();
        assert!(ts.contains("return state.variables.items.length"));
    }

    #[test]
    fn test_query_definition_basic() {
        let query = QueryDefinition::new(
            "getCount",
            QuerySchema::object(vec![("count", "number")]),
            QueryHandlerLogic::computed("state.variables.count"),
        )
        .with_description("Get the current count");

        let ts = query.to_typescript();
        assert!(ts.contains("defineQuery<GetCountQueryOutput>('getCount')"));
        assert!(ts.contains("setHandler(getCountQuery"));
        assert!(ts.contains("interface GetCountQueryOutput"));
    }

    #[test]
    fn test_query_definition_with_input() {
        let query = QueryDefinition::new(
            "getItem",
            QuerySchema::object(vec![("item", "unknown")]),
            QueryHandlerLogic::computed("state.variables.items[input.index]"),
        )
        .with_input_schema(QuerySchema::object(vec![("index", "number")]));

        let ts = query.to_typescript();
        assert!(ts.contains("GetItemQueryInput"));
        assert!(ts.contains("[GetItemQueryInput]"));
    }

    #[test]
    fn test_standard_queries() {
        let queries = standard_queries();
        assert!(!queries.is_empty());

        let names: Vec<_> = queries.iter().map(|q| q.name.as_str()).collect();
        assert!(names.contains(&"getState"));
        assert!(names.contains(&"getProgress"));
        assert!(names.contains(&"getStatus"));
    }

    #[test]
    fn test_workflow_queries_collection() {
        let mut queries = WorkflowQueries::new();

        queries.add(QueryDefinition::new(
            "customQuery",
            QuerySchema::any(),
            QueryHandlerLogic::project(vec!["*"]),
        ));

        assert_eq!(queries.queries.len(), 1);
        assert!(queries.get("customQuery").is_some());
    }

    #[test]
    fn test_workflow_queries_with_standard() {
        let queries = WorkflowQueries::with_standard_queries();
        assert!(queries.get("getState").is_some());
        assert!(queries.get("getProgress").is_some());
    }

    #[test]
    fn test_workflow_queries_to_typescript() {
        let queries = WorkflowQueries::with_standard_queries();
        let ts = queries.to_typescript();

        assert!(ts.contains("defineQuery"));
        assert!(ts.contains("setHandler"));
        assert!(ts.contains("@temporalio/workflow"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let query = QueryDefinition::new(
            "test",
            QuerySchema::object(vec![("value", "string")]),
            QueryHandlerLogic::project(vec!["value"]),
        );

        let json = serde_json::to_string(&query).unwrap();
        let restored: QueryDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(query.name, restored.name);
    }
}

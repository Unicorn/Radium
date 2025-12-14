//! Signal Handlers
//!
//! Implement signal handling for workflow communication:
//! - Signal definitions with typed payloads
//! - Signal handlers with state updates
//! - Signal buffering strategies
//! - External signal support

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// Signal buffering behavior
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SignalBuffering {
    /// Buffer signals and process in order received
    #[default]
    Ordered,
    /// Keep only the most recent signal, drop older ones
    Latest,
    /// Process signals immediately without buffering
    Immediate,
}

impl SignalBuffering {
    /// Convert to TypeScript comment/documentation
    pub fn to_typescript_comment(&self) -> &'static str {
        match self {
            SignalBuffering::Ordered => "// Signals are processed in order received",
            SignalBuffering::Latest => "// Only the most recent signal is processed",
            SignalBuffering::Immediate => "// Signals are processed immediately",
        }
    }
}

/// Schema definition for signal payloads
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SignalSchema {
    /// Schema fields
    #[serde(default)]
    pub fields: Vec<SignalSchemaField>,
    /// Whether the schema is strict (no extra fields allowed)
    #[serde(default)]
    pub strict: bool,
}

/// A field in a signal schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalSchemaField {
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
    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
}

fn default_true() -> bool {
    true
}

impl SignalSchema {
    /// Create an empty schema (accepts any payload)
    pub fn any() -> Self {
        Self {
            fields: vec![],
            strict: false,
        }
    }

    /// Create a schema with fields
    pub fn with_fields(fields: Vec<SignalSchemaField>) -> Self {
        Self {
            fields,
            strict: true,
        }
    }

    /// Generate TypeScript interface
    pub fn to_typescript_interface(&self, name: &str) -> String {
        if self.fields.is_empty() {
            return format!("type {} = void;", name);
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
}

/// Signal definition for workflow communication
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SignalDefinition {
    /// Signal name (must be a valid identifier)
    #[validate(length(min = 1, message = "Signal name is required"))]
    pub name: String,

    /// Description of the signal's purpose
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Input schema for the signal payload
    #[serde(default)]
    pub input_schema: SignalSchema,

    /// Whether this signal can be sent from outside the workflow
    #[serde(default = "default_true")]
    pub external: bool,

    /// Signal buffering behavior
    #[serde(default)]
    pub buffering: SignalBuffering,
}

impl SignalDefinition {
    /// Create a new signal definition
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            input_schema: SignalSchema::any(),
            external: true,
            buffering: SignalBuffering::default(),
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set input schema
    pub fn with_input_schema(mut self, schema: SignalSchema) -> Self {
        self.input_schema = schema;
        self
    }

    /// Set buffering behavior
    pub fn with_buffering(mut self, buffering: SignalBuffering) -> Self {
        self.buffering = buffering;
        self
    }

    /// Make this an internal-only signal
    pub fn internal_only(mut self) -> Self {
        self.external = false;
        self
    }

    /// Generate TypeScript type name
    pub fn typescript_type_name(&self) -> String {
        format!("{}Signal", to_pascal_case(&self.name))
    }

    /// Generate TypeScript payload type name
    pub fn typescript_payload_type(&self) -> String {
        if self.input_schema.fields.is_empty() {
            "void".to_string()
        } else {
            format!("{}Payload", to_pascal_case(&self.name))
        }
    }

    /// Generate TypeScript signal definition
    pub fn to_typescript_definition(&self) -> String {
        let mut code = String::new();

        // Add description comment
        if let Some(desc) = &self.description {
            code.push_str(&format!("/** {} */\n", desc));
        }

        // Generate payload interface if needed
        if !self.input_schema.fields.is_empty() {
            code.push_str(&self.input_schema.to_typescript_interface(&self.typescript_payload_type()));
            code.push('\n');
        }

        // Generate signal definition
        let payload_type = self.typescript_payload_type();
        code.push_str(&format!(
            "export const {} = defineSignal<{}>('{}');\n",
            to_camel_case(&self.name),
            payload_type,
            self.name
        ));

        code
    }
}

/// Source for variable updates in signal handlers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum VariableSource {
    /// Value from signal payload
    SignalPayload {
        /// JSON path in the payload
        path: String,
    },
    /// Constant value
    Constant {
        /// The constant value
        value: serde_json::Value,
    },
    /// Computed expression
    Expression {
        /// TypeScript expression
        expression: String,
    },
}

impl VariableSource {
    /// Create from signal payload path
    pub fn from_payload(path: impl Into<String>) -> Self {
        VariableSource::SignalPayload { path: path.into() }
    }

    /// Create from constant value
    pub fn from_constant(value: serde_json::Value) -> Self {
        VariableSource::Constant { value }
    }

    /// Create from expression
    pub fn from_expression(expression: impl Into<String>) -> Self {
        VariableSource::Expression {
            expression: expression.into(),
        }
    }

    /// Convert to TypeScript expression
    pub fn to_typescript(&self) -> String {
        match self {
            VariableSource::SignalPayload { path } => format!("payload.{}", path),
            VariableSource::Constant { value } => match value {
                serde_json::Value::String(s) => format!("'{}'", s),
                serde_json::Value::Null => "null".to_string(),
                _ => value.to_string(),
            },
            VariableSource::Expression { expression } => expression.clone(),
        }
    }
}

/// A variable update in a signal handler
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableUpdate {
    /// Name of the variable to update
    pub variable_name: String,
    /// Source of the new value
    pub source: VariableSource,
}

impl VariableUpdate {
    /// Create a new variable update
    pub fn new(variable_name: impl Into<String>, source: VariableSource) -> Self {
        Self {
            variable_name: variable_name.into(),
            source,
        }
    }

    /// Convert to TypeScript assignment
    pub fn to_typescript(&self) -> String {
        format!(
            "state.variables.{} = {};",
            self.variable_name,
            self.source.to_typescript()
        )
    }
}

/// Handler logic for signals
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SignalHandlerLogic {
    /// Reference to a workflow node to execute
    NodeReference {
        /// Node ID to execute
        node_id: String,
    },
    /// Only update state, no other logic
    StateUpdate,
    /// Custom TypeScript code
    Custom {
        /// TypeScript code to execute
        code: String,
    },
}

/// Signal handler implementation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SignalHandler {
    /// Name of the signal being handled
    #[validate(length(min = 1))]
    pub signal_name: String,

    /// Handler logic
    pub handler: SignalHandlerLogic,

    /// Variables to update when signal is received
    #[serde(default)]
    pub updates: Vec<VariableUpdate>,

    /// Whether to validate input against schema
    #[serde(default = "default_true")]
    pub validate_input: bool,
}

impl SignalHandler {
    /// Create a new signal handler
    pub fn new(signal_name: impl Into<String>) -> Self {
        Self {
            signal_name: signal_name.into(),
            handler: SignalHandlerLogic::StateUpdate,
            updates: vec![],
            validate_input: true,
        }
    }

    /// Set handler logic to node reference
    pub fn with_node(mut self, node_id: impl Into<String>) -> Self {
        self.handler = SignalHandlerLogic::NodeReference {
            node_id: node_id.into(),
        };
        self
    }

    /// Set handler logic to custom code
    pub fn with_custom_code(mut self, code: impl Into<String>) -> Self {
        self.handler = SignalHandlerLogic::Custom { code: code.into() };
        self
    }

    /// Add a variable update
    pub fn with_update(mut self, update: VariableUpdate) -> Self {
        self.updates.push(update);
        self
    }

    /// Disable input validation
    pub fn without_validation(mut self) -> Self {
        self.validate_input = false;
        self
    }

    /// Generate TypeScript handler code
    pub fn to_typescript(&self, signal_def: &SignalDefinition) -> String {
        let mut code = String::new();

        // Add buffering comment
        code.push_str(signal_def.buffering.to_typescript_comment());
        code.push('\n');

        // Generate handler
        let payload_type = signal_def.typescript_payload_type();
        let payload_param = if payload_type == "void" {
            ""
        } else {
            &format!("payload: {}", payload_type)
        };

        code.push_str(&format!(
            "setHandler({}, async ({}) => {{\n",
            to_camel_case(&self.signal_name),
            payload_param
        ));

        // Add validation if enabled
        if self.validate_input && !signal_def.input_schema.fields.is_empty() {
            code.push_str(&format!(
                "  validate{}Payload(payload);\n",
                to_pascal_case(&self.signal_name)
            ));
        }

        // Add variable updates
        for update in &self.updates {
            code.push_str(&format!("  {}\n", update.to_typescript()));
        }

        // Add handler logic
        match &self.handler {
            SignalHandlerLogic::NodeReference { node_id } => {
                code.push_str(&format!("  await executeNode('{}');\n", node_id));
            }
            SignalHandlerLogic::StateUpdate => {
                // Updates already added above
            }
            SignalHandlerLogic::Custom { code: custom_code } => {
                code.push_str(&format!("  {}\n", custom_code));
            }
        }

        code.push_str("});\n");

        code
    }
}

/// A complete signal definition with handler
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalWithHandler {
    /// The signal definition
    pub definition: SignalDefinition,
    /// The handler for this signal
    pub handler: SignalHandler,
}

impl SignalWithHandler {
    /// Create a new signal with handler
    pub fn new(definition: SignalDefinition, handler: SignalHandler) -> Self {
        Self {
            definition,
            handler,
        }
    }

    /// Generate complete TypeScript code
    pub fn to_typescript(&self) -> String {
        let mut code = String::new();

        // Generate signal definition
        code.push_str(&self.definition.to_typescript_definition());
        code.push('\n');

        // Generate handler
        code.push_str(&self.handler.to_typescript(&self.definition));

        code
    }
}

/// Collection of signals for a workflow
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSignals {
    /// All signals with their handlers
    #[serde(default)]
    pub signals: HashMap<String, SignalWithHandler>,
}

impl WorkflowSignals {
    /// Create empty signal collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a signal with handler
    pub fn add(&mut self, signal: SignalWithHandler) {
        self.signals
            .insert(signal.definition.name.clone(), signal);
    }

    /// Get a signal by name
    pub fn get(&self, name: &str) -> Option<&SignalWithHandler> {
        self.signals.get(name)
    }

    /// Generate TypeScript for all signals
    pub fn to_typescript(&self) -> String {
        let mut code = String::new();

        code.push_str("// Signal definitions and handlers\n");
        code.push_str("import { defineSignal, setHandler } from '@temporalio/workflow';\n\n");

        for signal in self.signals.values() {
            code.push_str(&signal.to_typescript());
            code.push('\n');
        }

        code
    }
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
    fn test_signal_buffering_serialization() {
        assert_eq!(
            serde_json::to_string(&SignalBuffering::Ordered).unwrap(),
            "\"ordered\""
        );
        assert_eq!(
            serde_json::to_string(&SignalBuffering::Latest).unwrap(),
            "\"latest\""
        );
    }

    #[test]
    fn test_signal_schema_typescript_interface() {
        let schema = SignalSchema::with_fields(vec![
            SignalSchemaField {
                name: "approved".to_string(),
                typescript_type: "boolean".to_string(),
                required: true,
                description: Some("Whether approved".to_string()),
                default: None,
            },
            SignalSchemaField {
                name: "approver".to_string(),
                typescript_type: "string".to_string(),
                required: false,
                description: None,
                default: None,
            },
        ]);

        let ts = schema.to_typescript_interface("ApprovalPayload");
        assert!(ts.contains("interface ApprovalPayload"));
        assert!(ts.contains("approved: boolean"));
        assert!(ts.contains("approver?: string"));
    }

    #[test]
    fn test_signal_definition_builder() {
        let signal = SignalDefinition::new("approveOrder")
            .with_description("Approve a pending order")
            .with_buffering(SignalBuffering::Latest)
            .internal_only();

        assert_eq!(signal.name, "approveOrder");
        assert!(!signal.external);
        assert_eq!(signal.buffering, SignalBuffering::Latest);
    }

    #[test]
    fn test_signal_definition_to_typescript() {
        let signal = SignalDefinition::new("updateStatus")
            .with_description("Update workflow status");

        let ts = signal.to_typescript_definition();
        assert!(ts.contains("defineSignal<void>('updateStatus')"));
    }

    #[test]
    fn test_variable_source_typescript() {
        assert_eq!(
            VariableSource::from_payload("approved").to_typescript(),
            "payload.approved"
        );
        assert_eq!(
            VariableSource::from_constant(serde_json::json!("test")).to_typescript(),
            "'test'"
        );
        assert_eq!(
            VariableSource::from_expression("Date.now()").to_typescript(),
            "Date.now()"
        );
    }

    #[test]
    fn test_variable_update_typescript() {
        let update = VariableUpdate::new("isApproved", VariableSource::from_payload("approved"));

        assert_eq!(
            update.to_typescript(),
            "state.variables.isApproved = payload.approved;"
        );
    }

    #[test]
    fn test_signal_handler_builder() {
        let handler = SignalHandler::new("approveOrder")
            .with_update(VariableUpdate::new(
                "isApproved",
                VariableSource::from_payload("approved"),
            ))
            .with_node("process-approval");

        assert_eq!(handler.signal_name, "approveOrder");
        assert!(matches!(
            handler.handler,
            SignalHandlerLogic::NodeReference { .. }
        ));
        assert_eq!(handler.updates.len(), 1);
    }

    #[test]
    fn test_signal_handler_to_typescript() {
        let signal = SignalDefinition::new("updateCount");
        let handler = SignalHandler::new("updateCount")
            .with_update(VariableUpdate::new(
                "count",
                VariableSource::from_expression("state.variables.count + 1"),
            ));

        let ts = handler.to_typescript(&signal);
        assert!(ts.contains("setHandler(updateCount"));
        assert!(ts.contains("state.variables.count = state.variables.count + 1"));
    }

    #[test]
    fn test_signal_with_handler() {
        let definition = SignalDefinition::new("notify")
            .with_description("Send notification");
        let handler = SignalHandler::new("notify")
            .with_custom_code("console.log('Notified!');");

        let signal = SignalWithHandler::new(definition, handler);
        let ts = signal.to_typescript();

        assert!(ts.contains("defineSignal"));
        assert!(ts.contains("setHandler"));
        assert!(ts.contains("console.log"));
    }

    #[test]
    fn test_workflow_signals_collection() {
        let mut signals = WorkflowSignals::new();

        signals.add(SignalWithHandler::new(
            SignalDefinition::new("start"),
            SignalHandler::new("start"),
        ));
        signals.add(SignalWithHandler::new(
            SignalDefinition::new("stop"),
            SignalHandler::new("stop"),
        ));

        assert_eq!(signals.signals.len(), 2);
        assert!(signals.get("start").is_some());
        assert!(signals.get("stop").is_some());
    }

    #[test]
    fn test_camel_and_pascal_case() {
        assert_eq!(to_camel_case("approve-order"), "approveOrder");
        assert_eq!(to_camel_case("update_status"), "updateStatus");
        assert_eq!(to_pascal_case("approve-order"), "ApproveOrder");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let signal = SignalDefinition::new("testSignal")
            .with_description("Test signal")
            .with_buffering(SignalBuffering::Latest);

        let json = serde_json::to_string(&signal).unwrap();
        let restored: SignalDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(signal.name, restored.name);
        assert_eq!(signal.buffering, restored.buffering);
    }
}

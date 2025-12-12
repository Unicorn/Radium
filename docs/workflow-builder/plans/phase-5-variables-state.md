# Phase 5: Variables & State Management

**Status**: Not Started
**Duration**: 2-3 weeks
**Prerequisites**: Phase 4 (Radium Migration)
**Blocks**: Phase 6

## Overview

Implement Rust validation and code generation for workflow variables, state management, and data flow between components. This phase establishes the patterns for type-safe data handling throughout workflow execution.

## Goals

1. Define Rust schemas for all variable types
2. Implement type-safe variable validation
3. Generate TypeScript code for state management
4. Support variable scoping (workflow, activity, local)
5. Enable type-checked data flow between components

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Variable System Architecture                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐    │
│  │  Workflow    │     │  Activity    │     │   Local      │    │
│  │  Variables   │     │  Variables   │     │  Variables   │    │
│  │              │     │              │     │              │    │
│  │ - input      │     │ - params     │     │ - temp       │    │
│  │ - output     │     │ - result     │     │ - computed   │    │
│  │ - state      │     │ - error      │     │ - iteration  │    │
│  └──────┬───────┘     └──────┬───────┘     └──────┬───────┘    │
│         │                    │                    │             │
│         └────────────────────┼────────────────────┘             │
│                              │                                   │
│                              ▼                                   │
│                    ┌─────────────────┐                          │
│                    │ State Manager   │                          │
│                    │                 │                          │
│                    │ - get/set       │                          │
│                    │ - validate      │                          │
│                    │ - transform     │                          │
│                    └─────────────────┘                          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Variable Types

| Type | Description | Rust Type | TypeScript Type |
|------|-------------|-----------|-----------------|
| `string` | Text values | `String` | `string` |
| `number` | Numeric values | `f64` | `number` |
| `integer` | Whole numbers | `i64` | `number` |
| `boolean` | True/false | `bool` | `boolean` |
| `object` | JSON objects | `serde_json::Value` | `Record<string, unknown>` |
| `array` | Lists | `Vec<serde_json::Value>` | `unknown[]` |
| `datetime` | Timestamps | `chrono::DateTime<Utc>` | `Date` |
| `duration` | Time spans | `chrono::Duration` | `number` (ms) |
| `null` | Null value | `Option<()>` | `null` |

---

## Tasks

### 4.1 Variable Type Schema

**Description**: Define Rust enums and structs for all variable types

**Subtasks**:
- [ ] 4.1.1 Create VariableType enum with all supported types
- [ ] 4.1.2 Create VariableValue enum for runtime values
- [ ] 4.1.3 Create VariableDefinition struct
- [ ] 4.1.4 Create VariableReference struct for variable lookups
- [ ] 4.1.5 Implement Default trait for all types
- [ ] 4.1.6 Implement Display trait for debugging
- [ ] 4.1.7 Add serde serialization with proper naming
- [ ] 4.1.8 Write unit tests for type conversions

**Files to Create**:
```rust
// src/schemas/variables/mod.rs
mod types;
mod definition;
mod reference;
mod value;

pub use types::*;
pub use definition::*;
pub use reference::*;
pub use value::*;
```

```rust
// src/schemas/variables/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum VariableType {
    String,
    Number,
    Integer,
    Boolean,
    Object,
    Array,
    Datetime,
    Duration,
    Null,
    #[serde(rename = "any")]
    Any, // Only for migration compatibility, generates warning
}

impl VariableType {
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            Self::String | Self::Number | Self::Integer | Self::Boolean | Self::Null
        )
    }

    pub fn is_complex(&self) -> bool {
        matches!(self, Self::Object | Self::Array)
    }

    pub fn typescript_type(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Number | Self::Integer => "number",
            Self::Boolean => "boolean",
            Self::Object => "Record<string, unknown>",
            Self::Array => "unknown[]",
            Self::Datetime => "Date",
            Self::Duration => "number",
            Self::Null => "null",
            Self::Any => "unknown", // Never use 'any'
        }
    }

    pub fn default_value(&self) -> &'static str {
        match self {
            Self::String => "''",
            Self::Number | Self::Integer | Self::Duration => "0",
            Self::Boolean => "false",
            Self::Object => "{}",
            Self::Array => "[]",
            Self::Datetime => "new Date()",
            Self::Null => "null",
            Self::Any => "undefined",
        }
    }
}

impl Default for VariableType {
    fn default() -> Self {
        Self::String
    }
}
```

```rust
// src/schemas/variables/value.rs
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VariableValue {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Object(JsonValue),
    Array(Vec<JsonValue>),
    Datetime(DateTime<Utc>),
    Duration(Duration),
    Null,
}

impl VariableValue {
    pub fn type_of(&self) -> super::VariableType {
        match self {
            Self::String(_) => super::VariableType::String,
            Self::Number(_) => super::VariableType::Number,
            Self::Integer(_) => super::VariableType::Integer,
            Self::Boolean(_) => super::VariableType::Boolean,
            Self::Object(_) => super::VariableType::Object,
            Self::Array(_) => super::VariableType::Array,
            Self::Datetime(_) => super::VariableType::Datetime,
            Self::Duration(_) => super::VariableType::Duration,
            Self::Null => super::VariableType::Null,
        }
    }

    pub fn is_compatible_with(&self, expected: &super::VariableType) -> bool {
        use super::VariableType;
        match (self, expected) {
            (_, VariableType::Any) => true,
            (Self::String(_), VariableType::String) => true,
            (Self::Number(_), VariableType::Number) => true,
            (Self::Integer(_), VariableType::Integer) => true,
            (Self::Integer(_), VariableType::Number) => true, // int -> number ok
            (Self::Boolean(_), VariableType::Boolean) => true,
            (Self::Object(_), VariableType::Object) => true,
            (Self::Array(_), VariableType::Array) => true,
            (Self::Datetime(_), VariableType::Datetime) => true,
            (Self::Duration(_), VariableType::Duration) => true,
            (Self::Null, VariableType::Null) => true,
            _ => false,
        }
    }
}
```

**Acceptance Criteria**:
- [ ] All variable types defined
- [ ] Serialization round-trips correctly
- [ ] TypeScript type mapping is accurate
- [ ] Default values are sensible

---

### 4.2 Variable Definition Schema

**Description**: Create comprehensive variable definition structure

**Subtasks**:
- [ ] 4.2.1 Create VariableDefinition struct with all metadata
- [ ] 4.2.2 Create VariableScope enum (workflow, activity, local)
- [ ] 4.2.3 Create VariableConstraints for validation rules
- [ ] 4.2.4 Implement validation for definitions
- [ ] 4.2.5 Add computed variable support
- [ ] 4.2.6 Add transformation/mapping support
- [ ] 4.2.7 Write tests for definition validation

**Files to Create**:
```rust
// src/schemas/variables/definition.rs
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VariableScope {
    Workflow,  // Available throughout workflow
    Activity,  // Available within single activity
    Local,     // Available within current block
}

impl Default for VariableScope {
    fn default() -> Self {
        Self::Local
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VariableConstraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,

    #[serde(default)]
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VariableDefinition {
    #[validate(length(min = 1, message = "Variable name is required"))]
    #[validate(regex(
        path = "crate::validation::IDENTIFIER_REGEX",
        message = "Variable name must be a valid identifier"
    ))]
    pub name: String,

    #[serde(rename = "type")]
    pub variable_type: super::VariableType,

    #[serde(default)]
    pub scope: VariableScope,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<super::VariableValue>,

    #[serde(default)]
    pub required: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<VariableConstraints>,

    /// Expression for computed variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computed: Option<String>,

    /// Source variable for transformations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Transformation to apply (e.g., "uppercase", "trim", "parseInt")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<String>,
}

impl VariableDefinition {
    pub fn new(name: impl Into<String>, variable_type: super::VariableType) -> Self {
        Self {
            name: name.into(),
            variable_type,
            scope: VariableScope::default(),
            description: None,
            default_value: None,
            required: false,
            constraints: None,
            computed: None,
            source: None,
            transform: None,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn with_default(mut self, value: super::VariableValue) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn workflow_scope(mut self) -> Self {
        self.scope = VariableScope::Workflow;
        self
    }

    pub fn is_computed(&self) -> bool {
        self.computed.is_some()
    }

    pub fn is_transformed(&self) -> bool {
        self.source.is_some() && self.transform.is_some()
    }
}
```

**Acceptance Criteria**:
- [ ] All variable metadata captured
- [ ] Scoping rules enforced
- [ ] Constraints are validatable
- [ ] Computed variables supported

---

### 4.3 Variable Reference System

**Description**: Implement variable reference and lookup system

**Subtasks**:
- [ ] 4.3.1 Create VariableReference struct
- [ ] 4.3.2 Implement reference parsing from strings (e.g., `$.workflow.input.name`)
- [ ] 4.3.3 Create reference path validation
- [ ] 4.3.4 Implement nested property access
- [ ] 4.3.5 Add array index support
- [ ] 4.3.6 Create TypeScript code generator for references
- [ ] 4.3.7 Write tests for reference resolution

**Files to Create**:
```rust
// src/schemas/variables/reference.rs
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VariableReference {
    /// Full path string (e.g., "$.workflow.input.userId")
    pub path: String,

    /// Parsed segments
    pub segments: Vec<PathSegment>,

    /// Expected type (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_type: Option<super::VariableType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PathSegment {
    Root,                    // $
    Property(String),        // .name
    Index(usize),           // [0]
    Wildcard,               // [*]
    Filter(String),         // [?(@.active)]
}

impl VariableReference {
    pub fn parse(path: &str) -> Result<Self, ReferenceParseError> {
        if !path.starts_with('$') {
            return Err(ReferenceParseError::MissingRoot);
        }

        let mut segments = vec![PathSegment::Root];
        let mut current = &path[1..];

        while !current.is_empty() {
            if current.starts_with('.') {
                // Property access
                current = &current[1..];
                let end = current
                    .find(|c: char| c == '.' || c == '[')
                    .unwrap_or(current.len());
                let prop = &current[..end];
                if prop.is_empty() {
                    return Err(ReferenceParseError::EmptyProperty);
                }
                segments.push(PathSegment::Property(prop.to_string()));
                current = &current[end..];
            } else if current.starts_with('[') {
                // Index or filter
                let end = current.find(']').ok_or(ReferenceParseError::UnclosedBracket)?;
                let inner = &current[1..end];

                if inner == "*" {
                    segments.push(PathSegment::Wildcard);
                } else if inner.starts_with("?(") {
                    segments.push(PathSegment::Filter(inner[2..inner.len()-1].to_string()));
                } else {
                    let idx: usize = inner.parse()
                        .map_err(|_| ReferenceParseError::InvalidIndex(inner.to_string()))?;
                    segments.push(PathSegment::Index(idx));
                }
                current = &current[end + 1..];
            } else {
                return Err(ReferenceParseError::UnexpectedCharacter(current.chars().next().unwrap()));
            }
        }

        Ok(Self {
            path: path.to_string(),
            segments,
            expected_type: None,
        })
    }

    pub fn to_typescript(&self) -> String {
        let mut result = String::new();

        for segment in &self.segments {
            match segment {
                PathSegment::Root => result.push_str("state"),
                PathSegment::Property(name) => {
                    result.push('.');
                    result.push_str(name);
                }
                PathSegment::Index(idx) => {
                    result.push('[');
                    result.push_str(&idx.to_string());
                    result.push(']');
                }
                PathSegment::Wildcard => {
                    // For wildcards, we need to map
                    result = format!("{}.map(item => item", result);
                }
                PathSegment::Filter(expr) => {
                    // For filters, we need to filter
                    result = format!("{}.filter(item => {})", result, expr);
                }
            }
        }

        result
    }

    pub fn root_variable(&self) -> Option<&str> {
        self.segments.get(1).and_then(|s| {
            if let PathSegment::Property(name) = s {
                Some(name.as_str())
            } else {
                None
            }
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReferenceParseError {
    #[error("Variable reference must start with $")]
    MissingRoot,

    #[error("Empty property name")]
    EmptyProperty,

    #[error("Unclosed bracket in reference")]
    UnclosedBracket,

    #[error("Invalid array index: {0}")]
    InvalidIndex(String),

    #[error("Unexpected character: {0}")]
    UnexpectedCharacter(char),
}

impl FromStr for VariableReference {
    type Err = ReferenceParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}
```

**Acceptance Criteria**:
- [ ] JSONPath-style references parsed correctly
- [ ] TypeScript code generation works
- [ ] Nested access supported
- [ ] Array indices handled

---

### 4.4 State Container Schema

**Description**: Define the state container structure for workflow execution

**Subtasks**:
- [ ] 4.4.1 Create WorkflowState struct
- [ ] 4.4.2 Create ActivityState struct
- [ ] 4.4.3 Create StateSnapshot for continue-as-new
- [ ] 4.4.4 Implement state versioning
- [ ] 4.4.5 Add state validation
- [ ] 4.4.6 Create state serialization format
- [ ] 4.4.7 Write tests for state operations

**Files to Create**:
```rust
// src/schemas/state/mod.rs
mod workflow_state;
mod activity_state;
mod snapshot;

pub use workflow_state::*;
pub use activity_state::*;
pub use snapshot::*;
```

```rust
// src/schemas/state/workflow_state.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::schemas::variables::{VariableDefinition, VariableValue};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Workflow instance ID
    pub workflow_id: String,

    /// State version for optimistic concurrency
    pub version: u64,

    /// Input variables (immutable after start)
    pub input: HashMap<String, VariableValue>,

    /// Output variables (set at completion)
    pub output: HashMap<String, VariableValue>,

    /// Workflow-scoped variables
    pub variables: HashMap<String, VariableValue>,

    /// Variable definitions for type checking
    pub definitions: Vec<VariableDefinition>,

    /// Current execution state
    pub execution_state: ExecutionState,

    /// Error information if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<StateError>,

    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    /// Metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionState {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub occurred_at: DateTime<Utc>,
}

impl WorkflowState {
    pub fn new(workflow_id: String, definitions: Vec<VariableDefinition>) -> Self {
        let now = Utc::now();
        Self {
            workflow_id,
            version: 1,
            input: HashMap::new(),
            output: HashMap::new(),
            variables: HashMap::new(),
            definitions,
            execution_state: ExecutionState::Pending,
            error: None,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&VariableValue> {
        self.variables.get(name)
            .or_else(|| self.input.get(name))
    }

    pub fn set(&mut self, name: String, value: VariableValue) -> Result<(), StateError> {
        // Validate type if definition exists
        if let Some(def) = self.definitions.iter().find(|d| d.name == name) {
            if !value.is_compatible_with(&def.variable_type) {
                return Err(StateError {
                    code: "TYPE_MISMATCH".to_string(),
                    message: format!(
                        "Variable '{}' expects type {:?}, got {:?}",
                        name, def.variable_type, value.type_of()
                    ),
                    details: None,
                    occurred_at: Utc::now(),
                });
            }
        }

        self.variables.insert(name, value);
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        matches!(
            self.execution_state,
            ExecutionState::Completed | ExecutionState::Failed | ExecutionState::Cancelled
        )
    }
}
```

```rust
// src/schemas/state/snapshot.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::schemas::variables::VariableValue;

/// Minimal state for continue-as-new
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Variables to carry forward
    pub variables: HashMap<String, VariableValue>,

    /// Current progress marker
    pub progress: ProgressMarker,

    /// Continuation metadata
    pub continuation: ContinuationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressMarker {
    /// Last completed activity/node ID
    pub last_completed: Option<String>,

    /// Iteration counters for loops
    pub iterations: HashMap<String, u64>,

    /// Batch progress
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_progress: Option<BatchProgress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    pub total_items: u64,
    pub processed_items: u64,
    pub current_batch: u64,
    pub batch_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuationInfo {
    /// Original workflow ID
    pub original_workflow_id: String,

    /// Continuation count
    pub continuation_count: u32,

    /// First started at
    pub started_at: DateTime<Utc>,

    /// This continuation started at
    pub continued_at: DateTime<Utc>,
}

impl StateSnapshot {
    pub fn from_workflow_state(
        state: &super::WorkflowState,
        last_completed: Option<String>,
    ) -> Self {
        Self {
            variables: state.variables.clone(),
            progress: ProgressMarker {
                last_completed,
                iterations: HashMap::new(),
                batch_progress: None,
            },
            continuation: ContinuationInfo {
                original_workflow_id: state.workflow_id.clone(),
                continuation_count: 0,
                started_at: state.created_at,
                continued_at: Utc::now(),
            },
        }
    }
}
```

**Acceptance Criteria**:
- [ ] State contains all necessary data
- [ ] Version tracking works
- [ ] Snapshots minimize data for continue-as-new
- [ ] Type validation on set

---

### 4.5 Data Flow Validation

**Description**: Validate data flow between components

**Subtasks**:
- [ ] 4.5.1 Create DataFlowValidator struct
- [ ] 4.5.2 Implement input/output type matching
- [ ] 4.5.3 Detect unreachable variables
- [ ] 4.5.4 Detect missing required inputs
- [ ] 4.5.5 Validate transformation chains
- [ ] 4.5.6 Generate data flow warnings
- [ ] 4.5.7 Create visualization data for UI
- [ ] 4.5.8 Write comprehensive tests

**Files to Create**:
```rust
// src/validation/data_flow.rs
use std::collections::{HashMap, HashSet};

use crate::schemas::{
    components::ComponentDefinition,
    variables::{VariableDefinition, VariableReference, VariableType},
    workflow::WorkflowDefinition,
};

pub struct DataFlowValidator {
    workflow: WorkflowDefinition,
    /// Variables available at each node
    available_at: HashMap<String, HashSet<String>>,
    /// Variable types
    types: HashMap<String, VariableType>,
}

#[derive(Debug, Clone)]
pub struct DataFlowAnalysis {
    pub errors: Vec<DataFlowError>,
    pub warnings: Vec<DataFlowWarning>,
    pub variable_usage: HashMap<String, VariableUsage>,
    pub flow_graph: FlowGraph,
}

#[derive(Debug, Clone)]
pub enum DataFlowError {
    MissingRequiredInput {
        node_id: String,
        variable_name: String,
    },
    TypeMismatch {
        node_id: String,
        variable_name: String,
        expected: VariableType,
        actual: VariableType,
    },
    CircularDependency {
        variables: Vec<String>,
    },
    UnresolvedReference {
        node_id: String,
        reference: String,
    },
}

#[derive(Debug, Clone)]
pub enum DataFlowWarning {
    UnusedVariable {
        variable_name: String,
        defined_at: String,
    },
    ShadowedVariable {
        variable_name: String,
        original_at: String,
        shadowed_at: String,
    },
    ImplicitTypeCoercion {
        node_id: String,
        variable_name: String,
        from_type: VariableType,
        to_type: VariableType,
    },
    PotentialNullAccess {
        node_id: String,
        reference: String,
    },
}

#[derive(Debug, Clone)]
pub struct VariableUsage {
    pub defined_at: Vec<String>,
    pub read_at: Vec<String>,
    pub written_at: Vec<String>,
    pub is_required: bool,
    pub variable_type: VariableType,
}

#[derive(Debug, Clone)]
pub struct FlowGraph {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
}

#[derive(Debug, Clone)]
pub struct FlowNode {
    pub id: String,
    pub node_type: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FlowEdge {
    pub from_node: String,
    pub from_variable: String,
    pub to_node: String,
    pub to_variable: String,
}

impl DataFlowValidator {
    pub fn new(workflow: WorkflowDefinition) -> Self {
        Self {
            workflow,
            available_at: HashMap::new(),
            types: HashMap::new(),
        }
    }

    pub fn analyze(&mut self) -> DataFlowAnalysis {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut variable_usage = HashMap::new();
        let mut flow_edges = Vec::new();

        // Build type map from workflow variables
        for var in &self.workflow.variables {
            self.types.insert(var.name.clone(), var.variable_type.clone());
        }

        // Initialize with workflow inputs
        let workflow_inputs: HashSet<String> = self.workflow.variables
            .iter()
            .filter(|v| v.required)
            .map(|v| v.name.clone())
            .collect();

        // Traverse workflow in execution order
        let execution_order = self.compute_execution_order();

        for node_id in &execution_order {
            let node = self.workflow.nodes.iter().find(|n| &n.id == node_id);
            if let Some(node) = node {
                // Check required inputs are available
                self.validate_node_inputs(node, &mut errors);

                // Track outputs
                self.track_node_outputs(node, &mut variable_usage);

                // Build flow edges
                self.build_flow_edges(node, &mut flow_edges);
            }
        }

        // Check for unused variables
        for (name, usage) in &variable_usage {
            if usage.read_at.is_empty() && !usage.defined_at.is_empty() {
                warnings.push(DataFlowWarning::UnusedVariable {
                    variable_name: name.clone(),
                    defined_at: usage.defined_at.first().cloned().unwrap_or_default(),
                });
            }
        }

        DataFlowAnalysis {
            errors,
            warnings,
            variable_usage,
            flow_graph: FlowGraph {
                nodes: self.build_flow_nodes(),
                edges: flow_edges,
            },
        }
    }

    fn compute_execution_order(&self) -> Vec<String> {
        // Topological sort based on edges
        // ... implementation
        Vec::new()
    }

    fn validate_node_inputs(
        &self,
        node: &crate::schemas::workflow::WorkflowNode,
        errors: &mut Vec<DataFlowError>,
    ) {
        // Check each input reference can be resolved
        // Check types match
        // ... implementation
    }

    fn track_node_outputs(
        &self,
        node: &crate::schemas::workflow::WorkflowNode,
        usage: &mut HashMap<String, VariableUsage>,
    ) {
        // Record what variables this node produces
        // ... implementation
    }

    fn build_flow_edges(
        &self,
        node: &crate::schemas::workflow::WorkflowNode,
        edges: &mut Vec<FlowEdge>,
    ) {
        // Create edges from source variables to this node's inputs
        // ... implementation
    }

    fn build_flow_nodes(&self) -> Vec<FlowNode> {
        // Convert workflow nodes to flow nodes
        // ... implementation
        Vec::new()
    }
}
```

**Acceptance Criteria**:
- [ ] Type mismatches detected
- [ ] Missing inputs detected
- [ ] Unused variables warned
- [ ] Circular dependencies detected

---

### 4.6 TypeScript State Generation

**Description**: Generate TypeScript code for state management

**Subtasks**:
- [ ] 4.6.1 Create StateGenerator struct
- [ ] 4.6.2 Generate state interface definitions
- [ ] 4.6.3 Generate state initialization code
- [ ] 4.6.4 Generate getter/setter with type guards
- [ ] 4.6.5 Generate variable reference resolution
- [ ] 4.6.6 Generate snapshot creation code
- [ ] 4.6.7 Generate snapshot restoration code
- [ ] 4.6.8 Write tests with tsc verification

**Files to Create**:
```rust
// src/codegen/state_generator.rs
use handlebars::Handlebars;
use serde::Serialize;

use crate::schemas::{
    state::WorkflowState,
    variables::{VariableDefinition, VariableType},
};

pub struct StateGenerator<'a> {
    handlebars: &'a Handlebars<'a>,
}

#[derive(Serialize)]
struct StateTemplateContext {
    workflow_name: String,
    variables: Vec<VariableTemplateData>,
    input_variables: Vec<VariableTemplateData>,
    output_variables: Vec<VariableTemplateData>,
}

#[derive(Serialize)]
struct VariableTemplateData {
    name: String,
    typescript_type: String,
    default_value: String,
    is_required: bool,
    is_nullable: bool,
    description: Option<String>,
}

impl<'a> StateGenerator<'a> {
    pub fn new(handlebars: &'a Handlebars<'a>) -> Self {
        Self { handlebars }
    }

    pub fn generate(&self, context: StateTemplateContext) -> Result<String, StateGenError> {
        self.handlebars
            .render("state", &context)
            .map_err(|e| StateGenError::TemplateError(e.to_string()))
    }

    pub fn build_context(
        workflow_name: &str,
        definitions: &[VariableDefinition],
    ) -> StateTemplateContext {
        let variables: Vec<_> = definitions
            .iter()
            .map(|d| VariableTemplateData {
                name: d.name.clone(),
                typescript_type: d.variable_type.typescript_type().to_string(),
                default_value: d.variable_type.default_value().to_string(),
                is_required: d.required,
                is_nullable: d.constraints
                    .as_ref()
                    .map(|c| c.nullable)
                    .unwrap_or(false),
                description: d.description.clone(),
            })
            .collect();

        StateTemplateContext {
            workflow_name: workflow_name.to_string(),
            variables: variables.clone(),
            input_variables: variables.iter().filter(|v| v.is_required).cloned().collect(),
            output_variables: Vec::new(), // Populate based on workflow outputs
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StateGenError {
    #[error("Template error: {0}")]
    TemplateError(String),
}
```

**Template File** (`templates/state.ts.hbs`):
```handlebars
// Generated by Rust Workflow Compiler - DO NOT EDIT
// Workflow: {{workflow_name}}
// Generated at: {{generated_at}}

/**
 * Input type for {{workflow_name}}
 */
export interface {{workflow_name}}Input {
{{#each input_variables}}
  {{#if description}}
  /** {{description}} */
  {{/if}}
  {{name}}{{#unless is_required}}?{{/unless}}: {{typescript_type}}{{#if is_nullable}} | null{{/if}};
{{/each}}
}

/**
 * Output type for {{workflow_name}}
 */
export interface {{workflow_name}}Output {
{{#each output_variables}}
  {{#if description}}
  /** {{description}} */
  {{/if}}
  {{name}}: {{typescript_type}}{{#if is_nullable}} | null{{/if}};
{{/each}}
}

/**
 * State container for {{workflow_name}}
 */
export interface {{workflow_name}}State {
  input: {{workflow_name}}Input;
  output: Partial<{{workflow_name}}Output>;
  variables: {
{{#each variables}}
    {{name}}: {{typescript_type}}{{#if is_nullable}} | null{{/if}};
{{/each}}
  };
  metadata: {
    workflowId: string;
    version: number;
    createdAt: Date;
    updatedAt: Date;
  };
}

/**
 * Create initial state for {{workflow_name}}
 */
export function create{{workflow_name}}State(
  workflowId: string,
  input: {{workflow_name}}Input
): {{workflow_name}}State {
  const now = new Date();
  return {
    input,
    output: {},
    variables: {
{{#each variables}}
      {{name}}: {{#if is_required}}input.{{name}}{{else}}{{default_value}}{{/if}},
{{/each}}
    },
    metadata: {
      workflowId,
      version: 1,
      createdAt: now,
      updatedAt: now,
    },
  };
}

/**
 * Type-safe variable getter
 */
export function getVariable<K extends keyof {{workflow_name}}State['variables']>(
  state: {{workflow_name}}State,
  key: K
): {{workflow_name}}State['variables'][K] {
  return state.variables[key];
}

/**
 * Type-safe variable setter with immutable update
 */
export function setVariable<K extends keyof {{workflow_name}}State['variables']>(
  state: {{workflow_name}}State,
  key: K,
  value: {{workflow_name}}State['variables'][K]
): {{workflow_name}}State {
  return {
    ...state,
    variables: {
      ...state.variables,
      [key]: value,
    },
    metadata: {
      ...state.metadata,
      version: state.metadata.version + 1,
      updatedAt: new Date(),
    },
  };
}

/**
 * Create snapshot for continue-as-new
 */
export function createSnapshot(state: {{workflow_name}}State): {{workflow_name}}Snapshot {
  return {
    variables: { ...state.variables },
    progress: {
      lastCompleted: undefined,
      iterations: {},
    },
    continuation: {
      originalWorkflowId: state.metadata.workflowId,
      continuationCount: 0,
      startedAt: state.metadata.createdAt,
      continuedAt: new Date(),
    },
  };
}

/**
 * Restore state from snapshot
 */
export function restoreFromSnapshot(
  workflowId: string,
  snapshot: {{workflow_name}}Snapshot,
  input: {{workflow_name}}Input
): {{workflow_name}}State {
  return {
    input,
    output: {},
    variables: snapshot.variables,
    metadata: {
      workflowId,
      version: 1,
      createdAt: snapshot.continuation.startedAt,
      updatedAt: new Date(),
    },
  };
}

/**
 * Snapshot type for continue-as-new
 */
export interface {{workflow_name}}Snapshot {
  variables: {{workflow_name}}State['variables'];
  progress: {
    lastCompleted: string | undefined;
    iterations: Record<string, number>;
  };
  continuation: {
    originalWorkflowId: string;
    continuationCount: number;
    startedAt: Date;
    continuedAt: Date;
  };
}
```

**Acceptance Criteria**:
- [ ] State interfaces are fully typed
- [ ] No `any` in generated code
- [ ] Getters/setters type-safe
- [ ] Snapshot round-trips work

---

### 4.7 Expression Evaluation

**Description**: Implement expression parsing and evaluation for computed variables

**Subtasks**:
- [ ] 4.7.1 Create ExpressionParser for simple expressions
- [ ] 4.7.2 Support arithmetic operations (+, -, *, /, %)
- [ ] 4.7.3 Support comparison operations (==, !=, <, >, <=, >=)
- [ ] 4.7.4 Support logical operations (&&, ||, !)
- [ ] 4.7.5 Support string operations (concat, includes, startsWith)
- [ ] 4.7.6 Support array operations (length, includes, find)
- [ ] 4.7.7 Generate TypeScript expressions
- [ ] 4.7.8 Write tests for all expression types

**Files to Create**:
```rust
// src/expressions/mod.rs
mod parser;
mod evaluator;
mod typescript_gen;

pub use parser::*;
pub use evaluator::*;
pub use typescript_gen::*;
```

```rust
// src/expressions/parser.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    // Literals
    String(String),
    Number(f64),
    Boolean(bool),
    Null,

    // References
    Variable(String),

    // Binary operations
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
    Modulo(Box<Expression>, Box<Expression>),

    // Comparison
    Equal(Box<Expression>, Box<Expression>),
    NotEqual(Box<Expression>, Box<Expression>),
    LessThan(Box<Expression>, Box<Expression>),
    GreaterThan(Box<Expression>, Box<Expression>),
    LessOrEqual(Box<Expression>, Box<Expression>),
    GreaterOrEqual(Box<Expression>, Box<Expression>),

    // Logical
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),

    // Ternary
    Conditional {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>,
    },

    // String operations
    Concat(Vec<Expression>),

    // Array operations
    ArrayLength(Box<Expression>),
    ArrayIncludes(Box<Expression>, Box<Expression>),

    // Property access
    Property(Box<Expression>, String),
    Index(Box<Expression>, Box<Expression>),

    // Function calls (limited set)
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
}

impl Expression {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        // Simple recursive descent parser
        // ... implementation
        todo!("Implement expression parser")
    }

    pub fn to_typescript(&self) -> String {
        match self {
            Expression::String(s) => format!("'{}'", s.replace('\'', "\\'")),
            Expression::Number(n) => n.to_string(),
            Expression::Boolean(b) => b.to_string(),
            Expression::Null => "null".to_string(),
            Expression::Variable(name) => format!("state.variables.{}", name),

            Expression::Add(l, r) => format!("({} + {})", l.to_typescript(), r.to_typescript()),
            Expression::Subtract(l, r) => format!("({} - {})", l.to_typescript(), r.to_typescript()),
            Expression::Multiply(l, r) => format!("({} * {})", l.to_typescript(), r.to_typescript()),
            Expression::Divide(l, r) => format!("({} / {})", l.to_typescript(), r.to_typescript()),
            Expression::Modulo(l, r) => format!("({} % {})", l.to_typescript(), r.to_typescript()),

            Expression::Equal(l, r) => format!("({} === {})", l.to_typescript(), r.to_typescript()),
            Expression::NotEqual(l, r) => format!("({} !== {})", l.to_typescript(), r.to_typescript()),
            Expression::LessThan(l, r) => format!("({} < {})", l.to_typescript(), r.to_typescript()),
            Expression::GreaterThan(l, r) => format!("({} > {})", l.to_typescript(), r.to_typescript()),
            Expression::LessOrEqual(l, r) => format!("({} <= {})", l.to_typescript(), r.to_typescript()),
            Expression::GreaterOrEqual(l, r) => format!("({} >= {})", l.to_typescript(), r.to_typescript()),

            Expression::And(l, r) => format!("({} && {})", l.to_typescript(), r.to_typescript()),
            Expression::Or(l, r) => format!("({} || {})", l.to_typescript(), r.to_typescript()),
            Expression::Not(e) => format!("(!{})", e.to_typescript()),

            Expression::Conditional { condition, then_branch, else_branch } => {
                format!("({} ? {} : {})",
                    condition.to_typescript(),
                    then_branch.to_typescript(),
                    else_branch.to_typescript()
                )
            }

            Expression::Concat(parts) => {
                let ts_parts: Vec<_> = parts.iter().map(|p| p.to_typescript()).collect();
                format!("[{}].join('')", ts_parts.join(", "))
            }

            Expression::ArrayLength(arr) => format!("{}.length", arr.to_typescript()),
            Expression::ArrayIncludes(arr, item) => {
                format!("{}.includes({})", arr.to_typescript(), item.to_typescript())
            }

            Expression::Property(obj, prop) => format!("{}.{}", obj.to_typescript(), prop),
            Expression::Index(arr, idx) => format!("{}[{}]", arr.to_typescript(), idx.to_typescript()),

            Expression::FunctionCall { name, args } => {
                let ts_args: Vec<_> = args.iter().map(|a| a.to_typescript()).collect();
                format!("{}({})", name, ts_args.join(", "))
            }
        }
    }

    pub fn referenced_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_variables(&mut vars);
        vars
    }

    fn collect_variables(&self, vars: &mut Vec<String>) {
        match self {
            Expression::Variable(name) => vars.push(name.clone()),
            Expression::Add(l, r) | Expression::Subtract(l, r) |
            Expression::Multiply(l, r) | Expression::Divide(l, r) |
            Expression::Modulo(l, r) | Expression::Equal(l, r) |
            Expression::NotEqual(l, r) | Expression::LessThan(l, r) |
            Expression::GreaterThan(l, r) | Expression::LessOrEqual(l, r) |
            Expression::GreaterOrEqual(l, r) | Expression::And(l, r) |
            Expression::Or(l, r) | Expression::ArrayIncludes(l, r) |
            Expression::Index(l, r) => {
                l.collect_variables(vars);
                r.collect_variables(vars);
            }
            Expression::Not(e) | Expression::ArrayLength(e) | Expression::Property(e, _) => {
                e.collect_variables(vars);
            }
            Expression::Conditional { condition, then_branch, else_branch } => {
                condition.collect_variables(vars);
                then_branch.collect_variables(vars);
                else_branch.collect_variables(vars);
            }
            Expression::Concat(parts) => {
                for p in parts {
                    p.collect_variables(vars);
                }
            }
            Expression::FunctionCall { args, .. } => {
                for a in args {
                    a.collect_variables(vars);
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),

    #[error("Unexpected end of expression")]
    UnexpectedEnd,

    #[error("Invalid number: {0}")]
    InvalidNumber(String),

    #[error("Unknown function: {0}")]
    UnknownFunction(String),
}
```

**Acceptance Criteria**:
- [ ] Simple expressions parse correctly
- [ ] TypeScript generation is valid
- [ ] Variables are extracted for dependency tracking
- [ ] All operators supported

---

### 4.8 Integration Testing

**Description**: Create comprehensive tests for the variable and state system

**Subtasks**:
- [ ] 4.8.1 Create test fixtures with various variable types
- [ ] 4.8.2 Test variable definition validation
- [ ] 4.8.3 Test reference parsing and resolution
- [ ] 4.8.4 Test state container operations
- [ ] 4.8.5 Test data flow analysis
- [ ] 4.8.6 Test TypeScript generation with tsc
- [ ] 4.8.7 Test expression evaluation
- [ ] 4.8.8 Create end-to-end variable flow test

**Test File**:
```rust
// tests/integration/variables_test.rs

use workflow_compiler::{
    schemas::{
        variables::{VariableDefinition, VariableType, VariableValue, VariableReference},
        state::WorkflowState,
    },
    validation::DataFlowValidator,
    codegen::StateGenerator,
};

#[test]
fn test_variable_type_compatibility() {
    let int_value = VariableValue::Integer(42);

    assert!(int_value.is_compatible_with(&VariableType::Integer));
    assert!(int_value.is_compatible_with(&VariableType::Number)); // int -> number ok
    assert!(!int_value.is_compatible_with(&VariableType::String));
    assert!(int_value.is_compatible_with(&VariableType::Any)); // any accepts all
}

#[test]
fn test_variable_reference_parsing() {
    let ref1 = VariableReference::parse("$.workflow.input.userId").unwrap();
    assert_eq!(ref1.segments.len(), 4);
    assert_eq!(ref1.root_variable(), Some("workflow"));

    let ref2 = VariableReference::parse("$.items[0].name").unwrap();
    assert_eq!(ref2.to_typescript(), "state.items[0].name");

    // Invalid references
    assert!(VariableReference::parse("workflow.input").is_err()); // missing $
    assert!(VariableReference::parse("$.[invalid]").is_err()); // empty property
}

#[test]
fn test_state_operations() {
    let definitions = vec![
        VariableDefinition::new("counter", VariableType::Integer).required(),
        VariableDefinition::new("name", VariableType::String),
    ];

    let mut state = WorkflowState::new("wf-123".to_string(), definitions);

    // Set valid value
    state.set("counter".to_string(), VariableValue::Integer(10)).unwrap();
    assert_eq!(state.get("counter"), Some(&VariableValue::Integer(10)));

    // Type mismatch should error
    let result = state.set("counter".to_string(), VariableValue::String("invalid".to_string()));
    assert!(result.is_err());
}

#[test]
fn test_data_flow_analysis() {
    // Create a simple workflow with variable flow
    let workflow = create_test_workflow();
    let mut validator = DataFlowValidator::new(workflow);
    let analysis = validator.analyze();

    // Check for expected errors/warnings
    assert!(analysis.errors.is_empty(), "Expected no errors: {:?}", analysis.errors);

    // Verify variable usage tracking
    assert!(analysis.variable_usage.contains_key("input_data"));
}

#[test]
fn test_typescript_state_generation() {
    let definitions = vec![
        VariableDefinition::new("userId", VariableType::String).required(),
        VariableDefinition::new("count", VariableType::Integer),
        VariableDefinition::new("items", VariableType::Array),
    ];

    let handlebars = setup_handlebars();
    let generator = StateGenerator::new(&handlebars);
    let context = StateGenerator::build_context("TestWorkflow", &definitions);

    let typescript = generator.generate(context).unwrap();

    // Verify no 'any' type
    assert!(!typescript.contains(": any"), "Generated code contains 'any' type");

    // Verify expected interfaces
    assert!(typescript.contains("interface TestWorkflowInput"));
    assert!(typescript.contains("interface TestWorkflowState"));

    // Compile with tsc
    verify_typescript_compiles(&typescript);
}

fn verify_typescript_compiles(code: &str) {
    use std::process::Command;
    use std::fs;

    let temp_file = "/tmp/test_state.ts";
    fs::write(temp_file, code).unwrap();

    let output = Command::new("npx")
        .args(["tsc", "--noEmit", "--strict", temp_file])
        .output()
        .expect("Failed to run tsc");

    assert!(
        output.status.success(),
        "TypeScript compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_file(temp_file).ok();
}
```

**Acceptance Criteria**:
- [ ] All variable types tested
- [ ] State operations are type-safe
- [ ] Data flow analysis catches issues
- [ ] Generated TypeScript compiles

---

## Rollback Plan

If Phase 4 causes issues:

1. **Keep existing TypeScript state management**: Don't remove until fully verified
2. **Feature flag**: Use environment variable to toggle between old/new
3. **Gradual migration**: Migrate one workflow at a time

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Type coverage | 100% | All variables typed |
| Validation accuracy | 100% | No false positives |
| TypeScript compilation | 100% pass | tsc --strict |
| Data flow analysis | < 100ms | Benchmark |

---

## Files to Create

```
packages/workflow-builder/
  rust-compiler/
    src/
      schemas/
        variables/
          mod.rs
          types.rs
          definition.rs
          reference.rs
          value.rs
        state/
          mod.rs
          workflow_state.rs
          activity_state.rs
          snapshot.rs
      validation/
        data_flow.rs
      expressions/
        mod.rs
        parser.rs
        evaluator.rs
        typescript_gen.rs
      codegen/
        state_generator.rs
    templates/
      state.ts.hbs
    tests/
      integration/
        variables_test.rs
```

---

## Dependencies

**Rust Crates**:
- `chrono` - Date/time handling
- `serde_json` - JSON value handling
- `thiserror` - Error definitions
- `handlebars` - Template engine

**TypeScript**:
- Generated code has no external dependencies
- Uses standard TypeScript types

---

## Checklist

Before marking Phase 5 complete:

- [ ] All variable types defined in Rust
- [ ] Variable definitions validated
- [ ] Reference parsing works
- [ ] State container operations type-safe
- [ ] Data flow validation catches issues
- [ ] TypeScript generation produces valid code
- [ ] Expression evaluation working
- [ ] All tests pass
- [ ] Documentation updated

---

## CI/CD Requirements (moved from Phase 2)

These CI/CD items should be implemented once the project is in the radium repo:

### 4.9 CI/CD Pipeline Setup

**Description**: Set up automated build and test pipeline for Rust compiler

**Subtasks**:
- [ ] 4.9.1 Create GitHub Actions workflow for Rust builds
- [ ] 4.9.2 Configure cargo build and test in CI
- [ ] 4.9.3 Add cargo clippy linting to CI
- [ ] 4.9.4 Add cargo fmt --check to CI
- [ ] 4.9.5 Set up test coverage reporting
- [ ] 4.9.6 Configure artifact caching for faster builds
- [ ] 4.9.7 Add Docker image build to CI
- [ ] 4.9.8 Set up deployment pipeline (staging/production)

**Files to Create**:
```yaml
# .github/workflows/rust-compiler.yml
name: Rust Compiler CI

on:
  push:
    paths:
      - 'packages/workflow-builder/workflow-compiler-rs/**'
  pull_request:
    paths:
      - 'packages/workflow-builder/workflow-compiler-rs/**'

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: packages/workflow-builder/workflow-compiler-rs
      - name: Build
        run: cargo build --release
        working-directory: packages/workflow-builder/workflow-compiler-rs
      - name: Test
        run: cargo test
        working-directory: packages/workflow-builder/workflow-compiler-rs
      - name: Clippy
        run: cargo clippy -- -D warnings
        working-directory: packages/workflow-builder/workflow-compiler-rs
      - name: Format Check
        run: cargo fmt --check
        working-directory: packages/workflow-builder/workflow-compiler-rs
```

**Acceptance Criteria**:
- [ ] CI runs on all PRs touching Rust code
- [ ] Build, test, lint, and format checks all pass
- [ ] Docker images built and pushed on merge to main
- [ ] Deployment to staging automated

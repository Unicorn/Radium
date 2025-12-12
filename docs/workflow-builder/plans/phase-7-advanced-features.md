# Phase 7: Advanced Features

**Status**: Not Started
**Duration**: 2-3 weeks
**Prerequisites**: Phase 6 (Component Migration)
**Blocks**: Phase 8

## Overview

Implement advanced workflow features including sub-workflows, signals, queries, cancellation handling, and complex control flow patterns. This phase adds the sophisticated capabilities needed for production-grade workflow orchestration.

## Goals

1. Enable workflow composition (sub-workflows)
2. Implement signal-based communication
3. Add query support for workflow state inspection
4. Handle cancellation gracefully
5. Support complex control flow patterns

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Advanced Features Architecture                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                     Parent Workflow                              │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │    │
│  │  │ Activity │──│ Signal   │──│ Child    │──│ Query    │        │    │
│  │  │          │  │ Handler  │  │ Workflow │  │ Handler  │        │    │
│  │  └──────────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘        │    │
│  │                     │             │             │               │    │
│  └─────────────────────┼─────────────┼─────────────┼───────────────┘    │
│                        │             │             │                     │
│                        ▼             ▼             ▼                     │
│  ┌──────────────┐  ┌──────────────┐  │  ┌──────────────────┐           │
│  │ External     │  │ Child        │  │  │ External Query   │           │
│  │ Signal       │  │ Workflow     │  │  │ Client           │           │
│  │ Sender       │  │ Instance     │  │  │                  │           │
│  └──────────────┘  └──────────────┘  │  └──────────────────┘           │
│                                       │                                  │
│                     ┌─────────────────┘                                 │
│                     ▼                                                    │
│            ┌──────────────────┐                                         │
│            │ Cancellation     │                                         │
│            │ Scope            │                                         │
│            └──────────────────┘                                         │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## Feature Matrix

| Feature | Rust Schema | TypeScript Gen | Tests | Docs |
|---------|-------------|----------------|-------|------|
| Child Workflows | 6.1 | 6.2 | 6.8 | 6.9 |
| Signal Handlers | 6.3 | 6.3 | 6.8 | 6.9 |
| Query Handlers | 6.4 | 6.4 | 6.8 | 6.9 |
| Cancellation | 6.5 | 6.5 | 6.8 | 6.9 |
| Search Attributes | 6.6 | 6.6 | 6.8 | 6.9 |
| Workflow Versioning | 6.7 | 6.7 | 6.8 | 6.9 |

---

## Tasks

### 6.1 Child Workflow Orchestration

**Description**: Implement comprehensive child workflow support

**Subtasks**:
- [ ] 6.1.1 Create ChildWorkflowConfig schema
- [ ] 6.1.2 Implement parent-child relationship tracking
- [ ] 6.1.3 Handle parent close policies
- [ ] 6.1.4 Support workflow ID generation strategies
- [ ] 6.1.5 Implement result handling and error propagation
- [ ] 6.1.6 Add cancellation propagation
- [ ] 6.1.7 Support workflow execution options
- [ ] 6.1.8 Create child workflow TypeScript template

**Files to Create**:
```rust
// src/schemas/advanced/child_workflow.rs
use serde::{Deserialize, Serialize};
use validator::Validate;
use std::collections::HashMap;

/// Parent close policy determines what happens to child when parent terminates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ParentClosePolicy {
    /// Child is terminated when parent completes
    Terminate,
    /// Child is abandoned (continues running orphaned)
    Abandon,
    /// Send cancellation request to child
    RequestCancel,
}

impl Default for ParentClosePolicy {
    fn default() -> Self {
        Self::Terminate
    }
}

/// Strategy for generating child workflow IDs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowIdStrategy {
    /// Use provided ID
    Explicit,
    /// Generate UUID
    Uuid,
    /// Use parent ID with suffix
    ParentSuffix,
    /// Custom pattern
    Pattern,
}

impl Default for WorkflowIdStrategy {
    fn default() -> Self {
        Self::Uuid
    }
}

/// ID conflict resolution policy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowIdReusePolicy {
    /// Allow if previous run completed
    AllowDuplicate,
    /// Allow if previous run failed
    AllowDuplicateFailedOnly,
    /// Never reuse
    RejectDuplicate,
    /// Terminate existing and start new
    TerminateIfRunning,
}

impl Default for WorkflowIdReusePolicy {
    fn default() -> Self {
        Self::AllowDuplicate
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ChildWorkflowConfig {
    /// Workflow type/name
    #[validate(length(min = 1, message = "Workflow type is required"))]
    pub workflow_type: String,

    /// Task queue (defaults to parent's)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_queue: Option<String>,

    /// Workflow ID strategy
    #[serde(default)]
    pub id_strategy: WorkflowIdStrategy,

    /// Explicit workflow ID (when strategy is Explicit)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,

    /// ID pattern for Pattern strategy (e.g., "child-{parent_id}-{index}")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_pattern: Option<String>,

    /// ID reuse policy
    #[serde(default)]
    pub id_reuse_policy: WorkflowIdReusePolicy,

    /// Input to pass to child workflow
    #[serde(default)]
    pub input: HashMap<String, serde_json::Value>,

    /// Parent close policy
    #[serde(default)]
    pub parent_close_policy: ParentClosePolicy,

    /// Execution timeout (total time for all runs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_timeout_ms: Option<u64>,

    /// Run timeout (time for single run)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_timeout_ms: Option<u64>,

    /// Task timeout (time to start execution)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_timeout_ms: Option<u64>,

    /// Retry policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<super::RetryPolicy>,

    /// Cancellation type (wait for cancellation to complete vs fire-and-forget)
    #[serde(default)]
    pub cancellation_type: CancellationType,

    /// Wait for result (false = fire and forget)
    #[serde(default = "default_true")]
    pub await_result: bool,

    /// Search attributes to set on child
    #[serde(default)]
    pub search_attributes: HashMap<String, SearchAttributeValue>,

    /// Memo fields
    #[serde(default)]
    pub memo: HashMap<String, String>,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CancellationType {
    /// Wait for child to acknowledge cancellation
    WaitCancellationCompleted,
    /// Try to cancel, don't wait
    TryCancel,
    /// Abandon (don't send cancellation)
    Abandon,
}

impl Default for CancellationType {
    fn default() -> Self {
        Self::WaitCancellationCompleted
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SearchAttributeValue {
    String(String),
    Int(i64),
    Double(f64),
    Bool(bool),
    Datetime(chrono::DateTime<chrono::Utc>),
    StringArray(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildWorkflowHandle {
    pub workflow_id: String,
    pub run_id: String,
    pub workflow_type: String,
    pub parent_workflow_id: String,
    pub parent_run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildWorkflowResult {
    pub handle: ChildWorkflowHandle,
    pub status: WorkflowExecutionStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<WorkflowError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowExecutionStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
    Terminated,
    ContinuedAsNew,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowError {
    pub error_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<Box<WorkflowError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<String>,
}
```

**TypeScript Template** (`templates/child_workflow.ts.hbs`):
```handlebars
// Generated Child Workflow Invocation
import {
  executeChild,
  ChildWorkflowHandle,
  ParentClosePolicy,
  WorkflowIdReusePolicy,
} from '@temporalio/workflow';

import type { {{workflow_type}}Input, {{workflow_type}}Output } from './{{workflow_type_snake}}';

/**
 * Execute child workflow: {{workflow_type}}
 */
export async function execute{{workflow_type}}Child(
  input: {{workflow_type}}Input,
  options?: {
    workflowId?: string;
    taskQueue?: string;
  }
): Promise<{{workflow_type}}Output> {
  const handle = await executeChild<typeof {{workflow_type}}>(
    '{{workflow_type}}',
    {
      args: [input],
      workflowId: options?.workflowId ?? `{{id_pattern}}`,
      taskQueue: options?.taskQueue ?? '{{default_task_queue}}',
      parentClosePolicy: ParentClosePolicy.{{parent_close_policy}},
      workflowIdReusePolicy: WorkflowIdReusePolicy.{{id_reuse_policy}},
      {{#if execution_timeout_ms}}
      workflowExecutionTimeout: {{execution_timeout_ms}},
      {{/if}}
      {{#if run_timeout_ms}}
      workflowRunTimeout: {{run_timeout_ms}},
      {{/if}}
      {{#if retry_policy}}
      retry: {
        maximumAttempts: {{retry_policy.max_attempts}},
        initialInterval: '{{retry_policy.initial_interval}}',
        maximumInterval: '{{retry_policy.max_interval}}',
        backoffCoefficient: {{retry_policy.backoff_coefficient}},
      },
      {{/if}}
    }
  );

  {{#if await_result}}
  return handle.result();
  {{else}}
  return { handle };
  {{/if}}
}

/**
 * Start child workflow without waiting
 */
export async function start{{workflow_type}}Child(
  input: {{workflow_type}}Input,
  workflowId: string
): Promise<ChildWorkflowHandle<{{workflow_type}}Output>> {
  return startChild<typeof {{workflow_type}}>('{{workflow_type}}', {
    args: [input],
    workflowId,
    taskQueue: '{{default_task_queue}}',
    parentClosePolicy: ParentClosePolicy.{{parent_close_policy}},
  });
}
```

**Acceptance Criteria**:
- [ ] Child workflows can be started
- [ ] Parent close policies work correctly
- [ ] ID strategies generate valid IDs
- [ ] Results propagate to parent

---

### 6.2 Workflow Composition Patterns

**Description**: Implement common workflow composition patterns

**Subtasks**:
- [ ] 6.2.1 Create scatter-gather pattern
- [ ] 6.2.2 Create pipeline pattern
- [ ] 6.2.3 Create saga pattern with compensation
- [ ] 6.2.4 Create map-reduce pattern
- [ ] 6.2.5 Generate TypeScript for each pattern
- [ ] 6.2.6 Test all patterns
- [ ] 6.2.7 Document pattern usage
- [ ] 6.2.8 Create pattern selection guide

**Pattern Schemas**:
```rust
// src/schemas/patterns/mod.rs
mod scatter_gather;
mod pipeline;
mod saga;
mod map_reduce;

pub use scatter_gather::*;
pub use pipeline::*;
pub use saga::*;
pub use map_reduce::*;

/// Common trait for workflow patterns
pub trait WorkflowPattern {
    fn pattern_name(&self) -> &'static str;
    fn validate(&self) -> Result<(), Vec<String>>;
    fn to_typescript(&self) -> String;
}
```

```rust
// src/schemas/patterns/saga.rs
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Saga pattern for distributed transactions with compensation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SagaDefinition {
    /// Saga name
    #[validate(length(min = 1))]
    pub name: String,

    /// Saga steps in order
    #[validate(length(min = 1, message = "Saga requires at least one step"))]
    pub steps: Vec<SagaStep>,

    /// Compensation behavior
    #[serde(default)]
    pub compensation_behavior: CompensationBehavior,

    /// Parallel compensation (run compensations concurrently)
    #[serde(default)]
    pub parallel_compensation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SagaStep {
    /// Step name
    #[validate(length(min = 1))]
    pub name: String,

    /// Forward action (activity or child workflow)
    pub action: SagaAction,

    /// Compensation action (rollback)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compensation: Option<SagaAction>,

    /// Skip compensation on specific errors
    #[serde(default)]
    pub skip_compensation_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SagaAction {
    Activity {
        activity_name: String,
        input: serde_json::Value,
    },
    ChildWorkflow {
        workflow_type: String,
        input: serde_json::Value,
    },
    LocalActivity {
        function_name: String,
        input: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CompensationBehavior {
    /// Compensate all completed steps on failure
    CompensateAll,
    /// Compensate only the failed step
    CompensateFailed,
    /// Custom compensation logic
    Custom,
}

impl Default for CompensationBehavior {
    fn default() -> Self {
        Self::CompensateAll
    }
}

impl SagaDefinition {
    pub fn to_typescript(&self) -> String {
        let mut code = String::new();

        code.push_str(&format!(
            r#"
// Saga: {}
// Generated by Rust Workflow Compiler

import {{ proxyActivities, ApplicationFailure }} from '@temporalio/workflow';

interface SagaContext {{
  completedSteps: string[];
  results: Record<string, unknown>;
}}

export async function {}Saga(input: unknown): Promise<unknown> {{
  const context: SagaContext = {{
    completedSteps: [],
    results: {{}},
  }};

  try {{
"#,
            self.name, self.name
        ));

        // Generate forward steps
        for step in &self.steps {
            code.push_str(&format!(
                r#"
    // Step: {}
    const {}Result = await execute{}(input);
    context.completedSteps.push('{}');
    context.results['{}'] = {}Result;
"#,
                step.name,
                to_camel_case(&step.name),
                to_pascal_case(&step.name),
                step.name,
                step.name,
                to_camel_case(&step.name)
            ));
        }

        code.push_str(
            r#"
    return context.results;
  } catch (error) {
    // Compensate in reverse order
    for (const stepName of context.completedSteps.reverse()) {
      try {
        await compensateStep(stepName, context);
      } catch (compensationError) {
        // Log compensation failure but continue
        console.error(`Compensation failed for ${stepName}:`, compensationError);
      }
    }
    throw error;
  }
}
"#,
        );

        code
    }
}

fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, c) in s.chars().enumerate() {
        if c == '-' || c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else if i == 0 {
            result.push(c.to_lowercase().next().unwrap());
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
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
```

**Acceptance Criteria**:
- [ ] Saga pattern handles compensation
- [ ] Scatter-gather parallelizes correctly
- [ ] Pipeline chains activities
- [ ] Map-reduce aggregates results

---

### 6.3 Signal Handlers

**Description**: Implement signal handling for workflow communication

**Subtasks**:
- [ ] 6.3.1 Create SignalDefinition schema
- [ ] 6.3.2 Create SignalHandler schema
- [ ] 6.3.3 Implement signal validation
- [ ] 6.3.4 Support typed signal payloads
- [ ] 6.3.5 Generate TypeScript signal handlers
- [ ] 6.3.6 Test signal send/receive
- [ ] 6.3.7 Document signal patterns
- [ ] 6.3.8 Handle signal ordering

**Files to Create**:
```rust
// src/schemas/advanced/signals.rs
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Signal definition for workflow communication
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SignalDefinition {
    /// Signal name
    #[validate(length(min = 1, message = "Signal name is required"))]
    #[validate(regex(
        path = "crate::validation::IDENTIFIER_REGEX",
        message = "Signal name must be a valid identifier"
    ))]
    pub name: String,

    /// Description of the signal's purpose
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Input schema for the signal payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<super::SchemaDefinition>,

    /// Whether this signal can be sent externally
    #[serde(default = "default_true")]
    pub external: bool,

    /// Signal buffering behavior
    #[serde(default)]
    pub buffering: SignalBuffering,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SignalBuffering {
    /// Buffer signals and process in order
    Ordered,
    /// Process most recent, drop older
    Latest,
    /// No buffering, process immediately
    Immediate,
}

impl Default for SignalBuffering {
    fn default() -> Self {
        Self::Ordered
    }
}

/// Signal handler implementation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SignalHandler {
    /// Signal being handled
    #[validate(length(min = 1))]
    pub signal_name: String,

    /// Handler logic (component node ID or inline)
    pub handler: SignalHandlerLogic,

    /// Variables to update on signal
    #[serde(default)]
    pub updates: Vec<VariableUpdate>,

    /// Whether to validate input against schema
    #[serde(default = "default_true")]
    pub validate_input: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SignalHandlerLogic {
    /// Reference to a component node
    NodeReference { node_id: String },
    /// Inline state update only
    StateUpdate,
    /// Custom TypeScript code
    Custom { code: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableUpdate {
    pub variable_name: String,
    pub source: VariableSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum VariableSource {
    SignalPayload { path: String },
    Constant { value: serde_json::Value },
    Expression { expression: String },
}

impl SignalDefinition {
    pub fn to_typescript_type(&self) -> String {
        match &self.input_schema {
            Some(schema) => schema.to_typescript_interface(&format!("{}Input", self.name)),
            None => "void".to_string(),
        }
    }

    pub fn to_typescript_handler(&self, handler: &SignalHandler) -> String {
        format!(
            r#"
// Signal: {name}
// {description}
export const {name}Signal = defineSignal<{input_type}>('{name}');

setHandler({name}Signal, async (payload: {input_type}) => {{
  {validation}
  {updates}
  {handler_code}
}});
"#,
            name = self.name,
            description = self.description.as_deref().unwrap_or("No description"),
            input_type = self.to_typescript_type(),
            validation = if handler.validate_input {
                format!("validate{}Input(payload);", self.name)
            } else {
                String::new()
            },
            updates = handler.updates.iter()
                .map(|u| format!("state.variables.{} = {};",
                    u.variable_name,
                    match &u.source {
                        VariableSource::SignalPayload { path } => format!("payload.{}", path),
                        VariableSource::Constant { value } => value.to_string(),
                        VariableSource::Expression { expression } => expression.clone(),
                    }
                ))
                .collect::<Vec<_>>()
                .join("\n  "),
            handler_code = match &handler.handler {
                SignalHandlerLogic::NodeReference { node_id } => format!("await executeNode('{}');", node_id),
                SignalHandlerLogic::StateUpdate => String::new(),
                SignalHandlerLogic::Custom { code } => code.clone(),
            }
        )
    }
}
```

**Acceptance Criteria**:
- [ ] Signals can be defined with schemas
- [ ] Handlers update state correctly
- [ ] External signals work
- [ ] Signal buffering behaves as expected

---

### 6.4 Query Handlers

**Description**: Implement query handlers for workflow state inspection

**Subtasks**:
- [ ] 6.4.1 Create QueryDefinition schema
- [ ] 6.4.2 Create QueryHandler schema
- [ ] 6.4.3 Implement query response types
- [ ] 6.4.4 Support filtered queries
- [ ] 6.4.5 Generate TypeScript query handlers
- [ ] 6.4.6 Test query execution
- [ ] 6.4.7 Document query patterns
- [ ] 6.4.8 Handle query timeouts

**Files to Create**:
```rust
// src/schemas/advanced/queries.rs
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Query definition for workflow state inspection
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct QueryDefinition {
    /// Query name
    #[validate(length(min = 1, message = "Query name is required"))]
    pub name: String,

    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Input schema (query parameters)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<super::SchemaDefinition>,

    /// Output schema (response type)
    pub output_schema: super::SchemaDefinition,

    /// Handler logic
    pub handler: QueryHandlerLogic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum QueryHandlerLogic {
    /// Return specific state variables
    StateProjection {
        variables: Vec<String>,
    },
    /// Return computed value
    Computed {
        expression: String,
    },
    /// Custom handler code
    Custom {
        code: String,
    },
}

impl QueryDefinition {
    pub fn to_typescript(&self) -> String {
        let input_type = self.input_schema.as_ref()
            .map(|s| s.to_typescript_interface(&format!("{}QueryInput", self.name)))
            .unwrap_or_else(|| "void".to_string());

        let output_type = self.output_schema.to_typescript_interface(&format!("{}QueryOutput", self.name));

        let handler_body = match &self.handler {
            QueryHandlerLogic::StateProjection { variables } => {
                let fields: Vec<_> = variables.iter()
                    .map(|v| format!("{}: state.variables.{}", v, v))
                    .collect();
                format!("return {{ {} }};", fields.join(", "))
            }
            QueryHandlerLogic::Computed { expression } => {
                format!("return {};", expression)
            }
            QueryHandlerLogic::Custom { code } => code.clone(),
        };

        format!(
            r#"
// Query: {name}
// {description}
{input_interface}

{output_interface}

export const {name}Query = defineQuery<{output_type}, [{input_type}]>('{name}');

setHandler({name}Query, ({input_param}): {output_type} => {{
  {handler_body}
}});
"#,
            name = self.name,
            description = self.description.as_deref().unwrap_or("No description"),
            input_interface = if self.input_schema.is_some() { &input_type } else { "" },
            output_interface = output_type,
            output_type = format!("{}QueryOutput", self.name),
            input_type = if self.input_schema.is_some() {
                format!("{}QueryInput", self.name)
            } else {
                "void".to_string()
            },
            input_param = if self.input_schema.is_some() { "input" } else { "" },
            handler_body = handler_body
        )
    }
}

/// Standard queries available on all workflows
pub fn standard_queries() -> Vec<QueryDefinition> {
    vec![
        QueryDefinition {
            name: "getState".to_string(),
            description: Some("Get current workflow state".to_string()),
            input_schema: None,
            output_schema: super::SchemaDefinition::any(),
            handler: QueryHandlerLogic::StateProjection {
                variables: vec!["*".to_string()], // All variables
            },
        },
        QueryDefinition {
            name: "getProgress".to_string(),
            description: Some("Get workflow progress".to_string()),
            input_schema: None,
            output_schema: super::SchemaDefinition::object(vec![
                ("completedSteps".to_string(), "string[]".to_string()),
                ("currentStep".to_string(), "string | null".to_string()),
                ("percentComplete".to_string(), "number".to_string()),
            ]),
            handler: QueryHandlerLogic::Custom {
                code: r#"
                    return {
                        completedSteps: state.progress.completedSteps,
                        currentStep: state.progress.currentStep,
                        percentComplete: calculateProgress(state),
                    };
                "#.to_string(),
            },
        },
    ]
}
```

**Acceptance Criteria**:
- [ ] Queries return typed responses
- [ ] State projections work
- [ ] Computed queries execute
- [ ] Standard queries available

---

### 6.5 Cancellation Handling

**Description**: Implement graceful cancellation support

**Subtasks**:
- [ ] 6.5.1 Create CancellationScope schema
- [ ] 6.5.2 Implement cancellation request handling
- [ ] 6.5.3 Support cleanup logic
- [ ] 6.5.4 Handle activity cancellation
- [ ] 6.5.5 Handle child workflow cancellation
- [ ] 6.5.6 Generate TypeScript cancellation handling
- [ ] 6.5.7 Test cancellation scenarios
- [ ] 6.5.8 Document cancellation patterns

**Files to Create**:
```rust
// src/schemas/advanced/cancellation.rs
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Cancellation scope for grouping cancellable operations
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CancellationScope {
    /// Scope name
    #[validate(length(min = 1))]
    pub name: String,

    /// Whether cancellation is shielded (not cancelled when parent is)
    #[serde(default)]
    pub shielded: bool,

    /// Cleanup logic to run on cancellation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup: Option<CleanupConfig>,

    /// Timeout for cleanup (after which force terminate)
    #[serde(default = "default_cleanup_timeout")]
    pub cleanup_timeout_ms: u64,
}

fn default_cleanup_timeout() -> u64 { 30000 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    /// Activities to run for cleanup
    #[serde(default)]
    pub activities: Vec<CleanupActivity>,

    /// State updates on cleanup
    #[serde(default)]
    pub state_updates: Vec<StateUpdate>,

    /// Custom cleanup code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupActivity {
    pub activity_name: String,
    pub input: serde_json::Value,
    /// Maximum attempts for cleanup activity
    #[serde(default = "default_one")]
    pub max_attempts: u32,
}

fn default_one() -> u32 { 1 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateUpdate {
    pub variable: String,
    pub value: serde_json::Value,
}

impl CancellationScope {
    pub fn to_typescript(&self) -> String {
        let cleanup_code = self.cleanup.as_ref().map(|c| {
            let mut code = String::new();

            for activity in &c.activities {
                code.push_str(&format!(
                    "await proxyActivities.{}({});\n",
                    activity.activity_name,
                    serde_json::to_string(&activity.input).unwrap_or_default()
                ));
            }

            for update in &c.state_updates {
                code.push_str(&format!(
                    "state.variables.{} = {};\n",
                    update.variable,
                    serde_json::to_string(&update.value).unwrap_or_default()
                ));
            }

            if let Some(custom) = &c.custom_code {
                code.push_str(custom);
            }

            code
        }).unwrap_or_default();

        format!(
            r#"
// Cancellation Scope: {}
const {}Scope = CancellationScope.{};

try {{
  await {}Scope.run(async () => {{
    // Protected operations here
  }});
}} catch (err) {{
  if (isCancellation(err)) {{
    // Run cleanup
    {}
    throw err; // Re-throw to propagate cancellation
  }}
  throw err;
}}
"#,
            self.name,
            self.name,
            if self.shielded { "nonCancellable" } else { "cancellable" },
            self.name,
            cleanup_code
        )
    }
}
```

**Acceptance Criteria**:
- [ ] Cancellation scopes work
- [ ] Cleanup runs on cancellation
- [ ] Shielded scopes protect operations
- [ ] Child cancellations propagate

---

### 6.6 Search Attributes

**Description**: Implement search attributes for workflow discovery

**Subtasks**:
- [ ] 6.6.1 Create SearchAttribute schema
- [ ] 6.6.2 Support standard attribute types
- [ ] 6.6.3 Implement attribute updates
- [ ] 6.6.4 Generate TypeScript search attribute code
- [ ] 6.6.5 Test search queries
- [ ] 6.6.6 Document attribute usage
- [ ] 6.6.7 Create attribute indexing guide
- [ ] 6.6.8 Add attribute validation

**Files to Create**:
```rust
// src/schemas/advanced/search_attributes.rs
use serde::{Deserialize, Serialize};
use validator::Validate;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum SearchAttributeType {
    Bool,
    Datetime,
    Double,
    Int,
    Keyword,
    KeywordList,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SearchAttributeDefinition {
    #[validate(length(min = 1))]
    pub name: String,

    pub attribute_type: SearchAttributeType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    /// Whether to index for search
    #[serde(default = "default_true")]
    pub indexed: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAttributeUpdate {
    pub name: String,
    pub value: SearchAttributeValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SearchAttributeValue {
    Bool(bool),
    Datetime(DateTime<Utc>),
    Double(f64),
    Int(i64),
    Keyword(String),
    KeywordList(Vec<String>),
    Text(String),
}

impl SearchAttributeDefinition {
    pub fn typescript_type(&self) -> &'static str {
        match self.attribute_type {
            SearchAttributeType::Bool => "boolean",
            SearchAttributeType::Datetime => "Date",
            SearchAttributeType::Double => "number",
            SearchAttributeType::Int => "number",
            SearchAttributeType::Keyword => "string",
            SearchAttributeType::KeywordList => "string[]",
            SearchAttributeType::Text => "string",
        }
    }
}

/// Standard search attributes
pub fn standard_search_attributes() -> Vec<SearchAttributeDefinition> {
    vec![
        SearchAttributeDefinition {
            name: "WorkflowType".to_string(),
            attribute_type: SearchAttributeType::Keyword,
            description: Some("Type of workflow".to_string()),
            default: None,
            indexed: true,
        },
        SearchAttributeDefinition {
            name: "CustomStatus".to_string(),
            attribute_type: SearchAttributeType::Keyword,
            description: Some("Custom workflow status".to_string()),
            default: Some(serde_json::json!("pending")),
            indexed: true,
        },
        SearchAttributeDefinition {
            name: "OwnerId".to_string(),
            attribute_type: SearchAttributeType::Keyword,
            description: Some("Owner/creator of workflow".to_string()),
            default: None,
            indexed: true,
        },
    ]
}
```

**Acceptance Criteria**:
- [ ] Search attributes can be defined
- [ ] Attributes can be updated at runtime
- [ ] Queries filter by attributes
- [ ] Standard attributes available

---

### 6.7 Workflow Versioning

**Description**: Implement workflow versioning for safe deployments

**Subtasks**:
- [ ] 6.7.1 Create VersioningConfig schema
- [ ] 6.7.2 Implement version change detection
- [ ] 6.7.3 Support branching based on version
- [ ] 6.7.4 Generate TypeScript versioning code
- [ ] 6.7.5 Test version migrations
- [ ] 6.7.6 Document versioning strategies
- [ ] 6.7.7 Create migration guide
- [ ] 6.7.8 Handle incompatible changes

**Files to Create**:
```rust
// src/schemas/advanced/versioning.rs
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VersioningConfig {
    /// Current version identifier
    #[validate(length(min = 1))]
    pub current_version: String,

    /// Version history for migration
    #[serde(default)]
    pub version_history: Vec<VersionInfo>,

    /// Version change points
    #[serde(default)]
    pub change_points: Vec<VersionChangePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub released_at: chrono::DateTime<chrono::Utc>,
    pub description: String,
    pub breaking_changes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VersionChangePoint {
    /// Unique identifier for this change point
    #[validate(length(min = 1))]
    pub change_id: String,

    /// Version where this change was introduced
    pub introduced_version: String,

    /// Description of the change
    pub description: String,

    /// Branches for different versions
    pub branches: Vec<VersionBranch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionBranch {
    /// Minimum version for this branch
    pub min_version: Option<String>,
    /// Maximum version for this branch
    pub max_version: Option<String>,
    /// Code to execute for this version range
    pub code: String,
}

impl VersionChangePoint {
    pub fn to_typescript(&self) -> String {
        let mut code = String::new();

        code.push_str(&format!(
            r#"
// Version change point: {}
// Introduced in version: {}
// {}
const {}Version = patched('{}'_change, () => {{
"#,
            self.change_id,
            self.introduced_version,
            self.description,
            to_camel_case(&self.change_id),
            self.change_id
        ));

        for (i, branch) in self.branches.iter().enumerate() {
            if i == 0 {
                code.push_str(&format!(
                    r#"
  if (version >= '{}') {{
    {}
  }}"#,
                    branch.min_version.as_deref().unwrap_or("0.0.0"),
                    branch.code
                ));
            } else {
                code.push_str(&format!(
                    r#" else {{
    {}
  }}"#,
                    branch.code
                ));
            }
        }

        code.push_str("\n});\n");
        code
    }
}

fn to_camel_case(s: &str) -> String {
    s.split('-')
        .enumerate()
        .map(|(i, part)| {
            if i == 0 {
                part.to_lowercase()
            } else {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().collect::<String>() + &chars.collect::<String>().to_lowercase(),
                }
            }
        })
        .collect()
}
```

**Acceptance Criteria**:
- [ ] Version change points work
- [ ] Old workflows continue functioning
- [ ] New workflows use latest code
- [ ] Migration paths documented

---

### 6.8 Integration Testing

**Description**: Comprehensive testing for all advanced features

**Subtasks**:
- [ ] 6.8.1 Test child workflow execution
- [ ] 6.8.2 Test signal send/receive
- [ ] 6.8.3 Test query handlers
- [ ] 6.8.4 Test cancellation handling
- [ ] 6.8.5 Test search attributes
- [ ] 6.8.6 Test versioning
- [ ] 6.8.7 Test composition patterns
- [ ] 6.8.8 Performance benchmarks

**Test File**:
```rust
// tests/advanced_features_test.rs
use workflow_compiler::schemas::advanced::*;
use tokio::test;

#[test]
async fn test_child_workflow_generation() {
    let config = ChildWorkflowConfig {
        workflow_type: "ProcessOrder".to_string(),
        task_queue: Some("orders".to_string()),
        id_strategy: WorkflowIdStrategy::ParentSuffix,
        parent_close_policy: ParentClosePolicy::Terminate,
        await_result: true,
        ..Default::default()
    };

    let typescript = config.to_typescript();

    assert!(typescript.contains("executeChild"));
    assert!(typescript.contains("ProcessOrder"));
    assert!(typescript.contains("ParentClosePolicy.TERMINATE"));

    verify_typescript_compiles(&typescript);
}

#[test]
async fn test_signal_handler_generation() {
    let signal = SignalDefinition {
        name: "approveOrder".to_string(),
        description: Some("Approve pending order".to_string()),
        input_schema: Some(SchemaDefinition::object(vec![
            ("approved".to_string(), "boolean".to_string()),
            ("approver".to_string(), "string".to_string()),
        ])),
        external: true,
        buffering: SignalBuffering::Ordered,
    };

    let handler = SignalHandler {
        signal_name: "approveOrder".to_string(),
        handler: SignalHandlerLogic::StateUpdate,
        updates: vec![
            VariableUpdate {
                variable_name: "isApproved".to_string(),
                source: VariableSource::SignalPayload {
                    path: "approved".to_string(),
                },
            },
        ],
        validate_input: true,
    };

    let typescript = signal.to_typescript_handler(&handler);

    assert!(typescript.contains("defineSignal"));
    assert!(typescript.contains("setHandler"));
    assert!(!typescript.contains("any")); // No any types

    verify_typescript_compiles(&typescript);
}

#[test]
async fn test_saga_compensation() {
    let saga = SagaDefinition {
        name: "OrderSaga".to_string(),
        steps: vec![
            SagaStep {
                name: "reserveInventory".to_string(),
                action: SagaAction::Activity {
                    activity_name: "reserveInventory".to_string(),
                    input: serde_json::json!({}),
                },
                compensation: Some(SagaAction::Activity {
                    activity_name: "releaseInventory".to_string(),
                    input: serde_json::json!({}),
                }),
                skip_compensation_errors: vec![],
            },
            SagaStep {
                name: "chargePayment".to_string(),
                action: SagaAction::Activity {
                    activity_name: "chargePayment".to_string(),
                    input: serde_json::json!({}),
                },
                compensation: Some(SagaAction::Activity {
                    activity_name: "refundPayment".to_string(),
                    input: serde_json::json!({}),
                }),
                skip_compensation_errors: vec![],
            },
        ],
        compensation_behavior: CompensationBehavior::CompensateAll,
        parallel_compensation: false,
    };

    let typescript = saga.to_typescript();

    assert!(typescript.contains("OrderSaga"));
    assert!(typescript.contains("compensateStep"));
    assert!(typescript.contains("reverse()")); // Reverse compensation order

    verify_typescript_compiles(&typescript);
}
```

**Acceptance Criteria**:
- [ ] All advanced features tested
- [ ] Generated code compiles
- [ ] Integration with Temporal verified
- [ ] No regressions

---

### 6.9 Documentation

**Description**: Complete documentation for advanced features

**Subtasks**:
- [ ] 6.9.1 Document child workflow patterns
- [ ] 6.9.2 Document signal usage
- [ ] 6.9.3 Document query patterns
- [ ] 6.9.4 Document cancellation handling
- [ ] 6.9.5 Document search attributes
- [ ] 6.9.6 Document versioning strategy
- [ ] 6.9.7 Create API reference
- [ ] 6.9.8 Create examples for each feature

**Acceptance Criteria**:
- [ ] All features documented
- [ ] Examples provided
- [ ] API reference complete
- [ ] Troubleshooting guide available

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Feature coverage | 100% | All 6 major features |
| TypeScript compilation | 100% pass | tsc --strict |
| Test coverage | > 80% | Code coverage |
| Documentation | 100% | All features documented |

---

## Files to Create

```
packages/workflow-builder/
  rust-compiler/
    src/
      schemas/
        advanced/
          mod.rs
          child_workflow.rs
          signals.rs
          queries.rs
          cancellation.rs
          search_attributes.rs
          versioning.rs
        patterns/
          mod.rs
          scatter_gather.rs
          pipeline.rs
          saga.rs
          map_reduce.rs
    templates/
      child_workflow.ts.hbs
      signal.ts.hbs
      query.ts.hbs
      cancellation.ts.hbs
    tests/
      advanced_features_test.rs
      patterns_test.rs
```

---

## Checklist

Before marking Phase 7 complete:

- [ ] Child workflow orchestration working
- [ ] All composition patterns implemented
- [ ] Signal handlers functional
- [ ] Query handlers functional
- [ ] Cancellation handling working
- [ ] Search attributes working
- [ ] Versioning support added
- [ ] All tests passing
- [ ] Documentation complete
- [ ] Phase 8 dependencies identified

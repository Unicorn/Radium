# ADR-004: Component Schema Structure

## Status

Accepted

## Date

2024 (Phase 1 of Rust migration)

## Context

Each workflow component type (activity, condition, child-workflow, etc.) needs:
1. A schema defining valid configurations
2. Validation rules
3. Code generation logic
4. UI representation

We needed to decide how to structure these schemas in Rust.

## Decision

Use a hierarchical schema structure with:
1. `WorkflowSchema` as the root type
2. `NodeSchema` for individual nodes with type-specific configs
3. `EdgeSchema` for connections between nodes
4. Separate config structs for each component type

## Schema Structure

### Root Schema
```rust
pub struct WorkflowSchema {
    pub id: String,
    pub name: String,
    pub nodes: Vec<NodeSchema>,
    pub edges: Vec<EdgeSchema>,
    pub metadata: WorkflowMetadata,
}
```

### Node Schema
```rust
pub struct NodeSchema {
    pub id: String,
    pub node_type: NodeType,
    pub position: Position,
    pub data: NodeData,
}

pub enum NodeType {
    Trigger,
    Activity,
    Condition,
    ChildWorkflow,
    Signal,
    Phase,
    StateVariable,
    KongLogging,
    KongCache,
    KongCors,
    End,
    // ... more types
}
```

### Component-Specific Configs
```rust
pub struct ActivityConfig {
    pub component_name: String,
    pub timeout: Option<Duration>,
    pub retry_policy: Option<RetryPolicy>,
}

pub struct ConditionConfig {
    pub expression: String,
    pub label: Option<String>,
}

pub struct ChildWorkflowConfig {
    pub workflow_type: String,
    pub execution_type: ExecutionType,
    pub task_queue: Option<String>,
    pub input_mapping: Option<HashMap<String, String>>,
}
```

## Validation Approach

### Structural Validation (serde)
- Required fields
- Type correctness
- Enum variants

### Semantic Validation (validator)
- Expression syntax
- Reference integrity (node IDs exist)
- Cycle detection
- Start/end node presence

### Cross-Component Validation
- Signal handlers have matching signal definitions
- Child workflows reference valid workflow types
- State variables are declared before use

## Rationale

### Enum for Node Types
Using an enum ensures:
- Exhaustive handling in code generation
- Compile-time checking for new types
- Clear documentation of supported types

### Separate Config Structs
Each component type has its own config struct because:
- Different components have different fields
- Type-safe access without casting
- Clear per-component documentation
- Easier to add new component types

### Flat Node Array
Nodes are stored in a flat array (not hierarchical) because:
- Matches React Flow data structure
- Simpler graph algorithms
- Edges define relationships explicitly

## Alternatives Considered

### Single Generic Config
```rust
pub struct NodeConfig {
    pub config: HashMap<String, Value>,
}
```
**Rejected because:**
- No compile-time type checking
- Validation becomes complex
- Documentation is unclear

### Nested Node Hierarchy
```rust
pub struct TriggerNode {
    pub children: Vec<ActivityNode>,
}
```
**Rejected because:**
- Doesn't match UI model
- Complex to serialize
- Hard to handle arbitrary connections

### JSON Schema
**Rejected because:**
- Runtime validation only
- Verbose for complex schemas
- Less idiomatic in Rust

## Consequences

### Positive
- Strong compile-time type checking
- Clear documentation through types
- Easy to add new component types
- IDE support for schema authoring

### Negative
- More Rust code to maintain
- Adding a component requires schema + validation + codegen updates
- Some duplication between component configs

## Implementation Files

- `src/schema/workflow.rs` - Root schema types
- `src/schema/node.rs` - Node and component schemas
- `src/schema/edge.rs` - Edge schema
- `src/validation/mod.rs` - Validation rules
- `src/codegen/typescript.rs` - Code generation per type

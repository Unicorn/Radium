# ADR-001: Use Rust for Schema Validation and Code Generation

## Status

Accepted

## Date

2024 (Phase 1-3 of Rust migration)

## Context

The workflow builder needs to:
1. Validate workflow definitions (nodes, edges, configurations)
2. Generate TypeScript code for Temporal workflows
3. Ensure type safety between UI and generated code
4. Handle complex component schemas with nested structures
5. Integrate with the existing Radium Rust ecosystem

We needed to choose a language and approach for the schema validation and code generation system.

## Decision

Use Rust with:
- **serde** for JSON/YAML serialization and schema definition
- **Handlebars** for TypeScript code generation templates
- **Axum** for the HTTP API server
- **validator** crate for validation rules

## Rationale

### Type Safety at Compile Time
Rust's type system catches schema errors during compilation rather than at runtime. This is critical for a code generator where invalid schemas could produce broken TypeScript.

### serde Excellence
The serde ecosystem provides:
- Zero-copy deserialization
- Excellent error messages for invalid input
- Custom serialization/deserialization
- Support for complex nested structures

### Pattern Matching
Rust's exhaustive pattern matching ensures all component types are handled in code generation. Adding a new component type without updating the code generator causes a compile error.

### Performance
Large workflows with many nodes need fast compilation. Rust's performance handles complex workflows efficiently.

### Radium Integration
The Radium project is built on Rust. Using Rust for the workflow compiler allows:
- Shared type definitions (radium-models)
- Common utilities (radium-core)
- Consistent tooling and CI/CD
- Future integration as a library

### Handlebars for Templates
Handlebars provides:
- Logic-less templates (keeps complex logic in Rust)
- Easy-to-read template syntax
- Good TypeScript support
- Custom helpers for type transformations

## Alternatives Considered

### TypeScript with Zod
**Pros:**
- Same language as output
- Familiar to frontend developers
- Zod provides runtime validation

**Cons:**
- Type errors at runtime, not compile time
- Less performant for large schemas
- No integration with Radium ecosystem

### Go with JSON Schema
**Pros:**
- Good performance
- Simple deployment (single binary)
- JSON Schema is standard

**Cons:**
- Less type safety than Rust
- JSON Schema is verbose for complex schemas
- No Radium ecosystem integration

### Python with Pydantic
**Pros:**
- Easy to write
- Pydantic is powerful
- Good for prototyping

**Cons:**
- Slow for large schemas
- Deployment complexity
- Dynamic typing

## Consequences

### Positive
- Compile-time guarantees for schema correctness
- Excellent error messages for invalid workflows
- Easy integration with Radium crates
- High performance for large workflows
- Strong type safety throughout

### Negative
- Steeper learning curve for TypeScript-only developers
- Requires Rust toolchain for development
- Templates are separated from Rust code (some context switching)

## Implementation Notes

The implementation is structured as:
```
radium-workflow/
  src/
    schema/           # Workflow and component schemas
    codegen/          # TypeScript code generation
    validation/       # Schema validation rules
    api/              # HTTP API server
    templates/        # Handlebars templates
```

Key files:
- `schema/workflow.rs` - Core workflow schema types
- `codegen/typescript.rs` - TypeScript generator
- `templates/workflow.ts.hbs` - Main workflow template

## References

- Phase 1: Schema definition
- Phase 2: Code generation
- Phase 3: Basic workflow verification
- Phase 4: Radium migration (this document)

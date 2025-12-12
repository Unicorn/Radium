# ADR-005: Code Generation Approach

## Status

Accepted

## Date

2024 (Phase 2 of Rust migration)

## Context

We need to generate TypeScript code from workflow schemas. Key considerations:
1. Generated code must be valid TypeScript
2. Generated code must be valid Temporal workflow code
3. Output should be readable and debuggable
4. Generation should be fast and reliable
5. Templates should be maintainable

## Decision

Use Handlebars templates with:
1. Main workflow template for structure
2. Partials for reusable sections
3. Custom helpers for type transformations
4. Rust logic for complex decisions

## Template Structure

```
templates/
  workflow.ts.hbs       # Main workflow template
  partials/
    imports.hbs         # Import statements
    activity.hbs        # Activity invocation
    condition.hbs       # Condition handling
    child-workflow.hbs  # Child workflow calls
    retry.hbs           # Retry loop
    state.hbs           # State variable ops
```

## Custom Helpers

### Type Conversion
```handlebars
{{temporal_type field_type}}
```
Converts schema types to TypeScript types.

### Name Formatting
```handlebars
{{camelCase name}}
{{pascalCase name}}
{{snakeCase name}}
```
Consistent naming transformations.

### Code Generation
```handlebars
{{#if has_timeout}}
  startToCloseTimeout: '{{timeout}}',
{{/if}}
```
Conditional code generation.

## Generation Flow

1. **Parse** - Validate workflow schema
2. **Analyze** - Determine imports, components used
3. **Order** - Topological sort of nodes
4. **Generate** - Apply templates with context
5. **Format** - Clean up whitespace, ensure valid syntax

## Generated Code Structure

```typescript
// Imports (determined by component analysis)
import { proxyActivities, ... } from '@temporalio/workflow';

// Type imports if needed
import type * as activities from './activities';

// Signal definitions if any
const updateSignal = defineSignal<[string]>('update');

// Main workflow function
export async function WorkflowName(
  input: Record<string, unknown> | undefined
): Promise<unknown> {

  // State variables
  let counter = 0;

  // Workflow logic (from nodes in order)
  // ...

}
```

## Rationale

### Handlebars Over String Concatenation
- Templates are readable
- Separation of structure and logic
- Easy to modify output format
- Supports partials and helpers

### Logic in Rust, Not Templates
Complex decisions (node ordering, import analysis, validation) happen in Rust because:
- Better error handling
- Type safety
- Testable logic
- Templates stay simple

### Readable Output
Generated code should look hand-written because:
- Debugging production issues
- Code review
- Learning resource
- Confidence in correctness

### Single File Output
Each workflow generates one TypeScript file because:
- Matches Temporal patterns
- Simple deployment
- Clear boundaries

## HTML Escaping

Important: Handlebars escapes HTML by default. For code generation:
- Use `{{{triple_braces}}}` for unescaped output
- Or configure Handlebars to disable escaping
- Critical for expressions like `result.amount > 100`

## Alternatives Considered

### String Templates (format!)
**Rejected because:**
- Hard to read for complex output
- No separation of concerns
- Difficult to maintain

### AST-based Generation
**Rejected because:**
- Over-engineering for current needs
- More complex implementation
- Templates are sufficient

### Embedded DSL
**Rejected because:**
- Custom syntax to learn
- Less flexible
- Harder to debug

## Consequences

### Positive
- Clean separation of structure and logic
- Easy to modify output format
- Readable templates
- Fast generation

### Negative
- Two languages (Rust + Handlebars)
- Template debugging can be tricky
- Need to handle HTML escaping

## Implementation Notes

Handlebars registration:
```rust
let mut handlebars = Handlebars::new();
handlebars.register_template_file("workflow", "templates/workflow.ts.hbs")?;
handlebars.register_helper("camelCase", Box::new(camel_case_helper));
```

Template rendering:
```rust
let context = WorkflowContext::from_schema(&workflow);
let output = handlebars.render("workflow", &context)?;
```

# ADR-003: Temporal Integration Pattern

## Status

Accepted

## Date

2024 (Phase 1-3 of Rust migration)

## Context

Temporal is the workflow orchestration engine. We needed to decide how to:
1. Map visual workflow nodes to Temporal constructs
2. Handle Temporal-specific features (signals, queries, child workflows)
3. Generate idiomatic Temporal TypeScript code
4. Ensure determinism requirements are met

## Decision

Generate idiomatic Temporal TypeScript SDK code that:
1. Uses standard Temporal APIs (proxyActivities, startChild, etc.)
2. Preserves Temporal's determinism requirements
3. Maps UI concepts directly to Temporal concepts
4. Generates human-readable, debuggable code

## Mapping Rules

### Workflow Structure
| UI Concept | Temporal Concept |
|------------|------------------|
| Workflow | Workflow function |
| Trigger node | Workflow entry point |
| End node | Workflow completion |
| Activity node | Activity invocation |
| Agent node | Activity invocation (AI) |

### Control Flow
| UI Concept | Temporal Concept |
|------------|------------------|
| Condition | In-line expression |
| Phase | Logical grouping |
| Retry | Retry policy OR custom loop |

### Advanced Features
| UI Concept | Temporal Concept |
|------------|------------------|
| Signal | defineSignal + setHandler |
| Query | defineQuery + setHandler |
| Child Workflow | startChild / executeChild |
| State Variable | Workflow-local variable |

## Code Generation Patterns

### Activity Invocation
```typescript
const result = await activityName(input, {
  startToCloseTimeout: '30s',
  retry: {
    maximumAttempts: 3,
  },
});
```

### Child Workflow
```typescript
const result = await executeChild(ChildWorkflow, {
  workflowId: 'child-' + uuid4(),
  args: [input],
});
```

### Signal Handler (Future)
```typescript
const updateSignal = defineSignal<[string]>('update');
setHandler(updateSignal, (value) => {
  state = value;
});
```

## Determinism Considerations

Temporal workflows must be deterministic. Our code generation ensures:

1. **No random values** - uuid4() comes from Temporal's deterministic source
2. **No current time** - Time-based logic uses Temporal's workflow.now()
3. **No external calls** - All external work goes through activities
4. **Idempotent replays** - Generated code produces same results on replay

## Rationale

### Standard APIs
Using standard Temporal APIs rather than custom wrappers:
- Works with existing Temporal tooling
- Documentation is readily available
- No learning curve for Temporal developers
- Easier debugging and troubleshooting

### Readable Output
Generated code should be readable because:
- Developers need to debug production issues
- Code review of generated workflows
- Understanding workflow behavior
- Learning how Temporal works

### Direct Mapping
UI concepts map directly to Temporal concepts to:
- Reduce cognitive overhead
- Enable accurate Temporal documentation in UI
- Allow power users to understand generated code

## Alternatives Considered

### Custom Runtime Layer
**Rejected because:**
- Adds complexity
- Hides Temporal semantics
- Makes debugging harder
- No ecosystem benefit

### JSON-based Workflow Definition
**Rejected because:**
- Temporal SDK is TypeScript, not JSON interpreter
- Would require custom runtime
- Loses TypeScript type checking

### GraphQL/REST API Wrapper
**Rejected because:**
- Adds network hop
- Loses determinism guarantees
- Complicates deployment

## Consequences

### Positive
- Idiomatic Temporal code
- Works with all Temporal tooling
- Human-readable output
- Standard debugging approaches

### Negative
- Tied to Temporal TypeScript SDK patterns
- Must update when SDK changes
- Some advanced Temporal features need explicit support

## Implementation Notes

Key Temporal imports used:
```typescript
import {
  proxyActivities,
  startChild,
  executeChild,
  sleep,
  condition,
  defineSignal,
  setHandler,
  uuid4
} from '@temporalio/workflow';
```

Activity proxy pattern:
```typescript
const activities = proxyActivities<typeof import('./activities')>({
  startToCloseTimeout: '30s',
});
```

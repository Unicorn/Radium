# Workflow Compiler Pattern Library

## Overview

The pattern-based workflow compiler uses a modular, extensible architecture where each workflow node type is handled by a specific **Pattern**. Patterns are responsible for detecting when they should be applied and generating the appropriate TypeScript code.

## Architecture

```
┌─────────────────────┐
│ WorkflowDefinition  │
│   (JSON/Graph)      │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  WorkflowCompiler   │
│   - Validates       │
│   - Applies Patterns│
│   - Generates Code  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│   Pattern Library   │
│ ┌─────────────────┐ │
│ │ Activity Proxy  │ │
│ │ State Mgmt      │ │
│ │ Signal Handler  │ │
│ │ Child Workflow  │ │
│ └─────────────────┘ │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  TypeScript Code    │
│   - workflow.ts     │
│   - activities.ts   │
│   - worker.ts       │
└─────────────────────┘
```

## Pattern Interface

Every pattern must implement the `Pattern` interface:

```typescript
interface Pattern {
  name: string;                    // Unique pattern identifier
  priority?: number;               // Higher priority runs first (default: 0)
  detect: (node, context) => boolean;    // Should this pattern apply?
  generate: (node, context) => CodeBlock; // Generate code for this node
  dependencies?: string[];         // NPM package dependencies
}
```

## Built-in Patterns

### 1. Activity Proxy Pattern

**Purpose**: Generates `proxyActivities()` code for activity and agent nodes.

**Priority**: 100 (high)

**Detects**: Nodes with type `activity` or `agent`

**Generates**:
- Activity proxy setup (first activity only)
- Activity invocation with proper input parameters
- Result variable capture

**Example Output**:
```typescript
// Activity proxies
const { processData, sendEmail } = proxyActivities<typeof activities>({
  startToCloseTimeout: '5 minutes',
  retry: {
    initialInterval: '1s',
    backoffCoefficient: 2,
    maximumAttempts: 3,
  },
});

// Execute activity: Process Data
const result_activity_1 = await processData(input);
```

**Usage**:
```typescript
import { ActivityProxyPattern } from './patterns';

compiler.registerPattern(ActivityProxyPattern);
```

### 2. State Management Pattern

**Purpose**: Handles state variable declarations and operations.

**Priority**: 90

**Detects**: Nodes with type `state-variable`

**Generates**:
- Variable declarations (first occurrence)
- State operations (set, append, increment, decrement, get)

**Example Output**:
```typescript
// State variable declaration
let counter = 0;

// State variable operation
counter++;
```

**Supported Operations**:
- `set`: Assign a value
- `append`: Push to array
- `increment`: Add 1
- `decrement`: Subtract 1
- `get`: Read current value

## Creating Custom Patterns

### Step 1: Define the Pattern

Create a new file in `/patterns/`:

```typescript
// patterns/custom-pattern.ts
import type { Pattern, WorkflowNode, GeneratorContext, CodeBlock } from '../types';
import { indent } from '../utils/ast-helpers';

export const CustomPattern: Pattern = {
  name: 'custom-pattern',
  priority: 50,

  detect: (node: WorkflowNode, context: GeneratorContext): boolean => {
    // Return true if this pattern should handle this node
    return node.type === 'custom-type';
  },

  generate: (node: WorkflowNode, context: GeneratorContext): CodeBlock => {
    const indentStr = indent(context.currentIndent);

    return {
      code: `${indentStr}// Custom code here`,
      imports: [`import { customFunction } from 'custom-package';`],
      declarations: [`const customVar = 'value';`],
      resultVar: `result_${node.id}`,
    };
  },

  dependencies: ['custom-package'],
};
```

### Step 2: Register the Pattern

```typescript
import { WorkflowCompiler } from './compiler';
import { CustomPattern } from './patterns/custom-pattern';

const compiler = new WorkflowCompiler();
compiler.registerPattern(CustomPattern);
```

### Step 3: Use in Workflow

The pattern will automatically apply when a matching node is encountered during compilation.

## Pattern Priority

Patterns are executed in priority order (highest first). This ensures that critical setup code (like activity proxies) runs before dependent code.

**Priority Levels**:
- **100+**: Critical infrastructure (activity proxies, imports)
- **50-99**: Business logic (signal handlers, queries)
- **0-49**: Utilities and helpers
- **Negative**: Cleanup and finalization

## Generator Context

The `GeneratorContext` provides access to the complete workflow state:

```typescript
interface GeneratorContext {
  nodes: WorkflowNode[];           // All workflow nodes
  edges: WorkflowEdge[];           // All edges
  variables: WorkflowVariable[];   // Workflow variables
  settings: WorkflowSettings;      // Global settings
  nodeMap: Map<string, WorkflowNode>;    // Quick node lookup
  edgeMap: Map<string, WorkflowEdge[]>;  // Outgoing edges by source
  visitedNodes: Set<string>;       // Already processed nodes
  resultVars: Map<string, string>; // Node ID -> result variable name
  currentIndent: number;           // Current indentation level
}
```

## Code Block Structure

Patterns return a `CodeBlock` containing the generated code and metadata:

```typescript
interface CodeBlock {
  code: string;              // The main code to insert
  imports?: string[];        // Import statements needed
  declarations?: string[];   // Top-level declarations
  resultVar?: string;        // Variable name for this node's result
}
```

The compiler automatically:
- Deduplicates imports
- Places declarations at the top of the workflow function
- Tracks result variables for data flow
- Manages indentation

## Best Practices

### 1. Single Responsibility
Each pattern should handle one specific node type or behavior.

### 2. Idempotency
Patterns should produce the same output for the same input.

### 3. Error Handling
Validate node configuration and provide helpful error messages:

```typescript
generate: (node, context) => {
  if (!node.data.config?.requiredField) {
    throw new Error(`Node ${node.id} missing requiredField`);
  }
  // ... generate code
}
```

### 4. Import Management
Only include necessary imports:

```typescript
const imports: string[] = [];

if (needsProxyActivities) {
  imports.push(`import { proxyActivities } from '@temporalio/workflow';`);
}

return { code, imports };
```

### 5. Context Usage
Use the context to understand relationships between nodes:

```typescript
// Find incoming edges
const incomingEdges = context.edges.filter(e => e.target === node.id);

// Get previous result
const sourceNode = context.nodeMap.get(incomingEdges[0].source);
const previousResult = context.resultVars.get(sourceNode.id);
```

## Testing Patterns

Create unit tests for your patterns:

```typescript
import { describe, it, expect } from 'vitest';
import { CustomPattern } from '../custom-pattern';

describe('CustomPattern', () => {
  it('should detect custom nodes', () => {
    const node = {
      id: 'test-1',
      type: 'custom-type',
      data: {},
      position: { x: 0, y: 0 },
    };

    const context = createMockContext();

    expect(CustomPattern.detect(node, context)).toBe(true);
  });

  it('should generate correct code', () => {
    const node = createCustomNode();
    const context = createMockContext();

    const result = CustomPattern.generate(node, context);

    expect(result.code).toContain('expected code');
    expect(result.imports).toContain('expected import');
  });
});
```

## Advanced Examples

### Conditional Branching Pattern

```typescript
export const ConditionalPattern: Pattern = {
  name: 'conditional',
  priority: 60,

  detect: (node) => node.type === 'condition',

  generate: (node, context) => {
    const expression = node.data.config?.expression || 'true';
    const indentStr = indent(context.currentIndent);

    // Find true/false branches
    const outgoingEdges = context.edgeMap.get(node.id) || [];
    const trueBranch = outgoingEdges.find(e => e.sourceHandle === 'true');
    const falseBranch = outgoingEdges.find(e => e.sourceHandle === 'false');

    const code = `${indentStr}if (${expression}) {
${indentStr}  // True branch
${indentStr}} else {
${indentStr}  // False branch
${indentStr}}`;

    return { code };
  },
};
```

### Loop Pattern

```typescript
export const LoopPattern: Pattern = {
  name: 'loop',
  priority: 55,

  detect: (node) => node.type === 'loop',

  generate: (node, context) => {
    const config = node.data.config || {};
    const maxIterations = config.maxIterations || 100;
    const indentStr = indent(context.currentIndent);

    const code = `${indentStr}let iteration = 0;
${indentStr}while (iteration < ${maxIterations}) {
${indentStr}  // Loop body
${indentStr}  iteration++;
${indentStr}}`;

    return { code };
  },
};
```

## Debugging

Enable detailed logging during development:

```typescript
const compiler = new WorkflowCompiler({
  includeComments: true,  // Add helpful comments
  strictMode: true,       // Strict TypeScript checking
});

const result = compiler.compile(workflow);

if (!result.success) {
  console.error('Compilation errors:', result.errors);
  console.warn('Warnings:', result.warnings);
}

console.log('Patterns applied:', result.metadata?.patternsApplied);
console.log('Compilation time:', result.metadata?.compilationTime, 'ms');
```

## Future Patterns

Planned patterns for future releases:

- **Signal Handler Pattern**: Generate signal definitions and handlers
- **Query Pattern**: Generate query definitions and handlers
- **Child Workflow Pattern**: Handle child workflow execution
- **Timer Pattern**: Sleep and timer operations
- **Saga Pattern**: Compensation logic
- **Parallel Execution Pattern**: Promise.all coordination

## Contributing

To contribute a new pattern:

1. Create the pattern file in `/patterns/`
2. Add comprehensive tests in `/__tests__/`
3. Document in this README
4. Update the pattern index (`/patterns/index.ts`)
5. Submit a pull request

## License

MIT

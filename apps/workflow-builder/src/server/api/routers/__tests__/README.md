# Compiler Router API Documentation

This document describes the tRPC API endpoints for the workflow compiler.

## Overview

The compiler router provides type-safe API endpoints for compiling workflow definitions into executable TypeScript code using the pattern-based compiler.

## Endpoints

### 1. `compiler.compile`

Compile a workflow by ID (fetches from database and saves compiled code).

**Type:** Mutation
**Auth:** Protected (requires authentication)

**Input:**
```typescript
{
  workflowId: string;           // ID of workflow to compile
  includeComments?: boolean;    // Include comments in generated code (default: true)
  strictMode?: boolean;         // Enable TypeScript strict mode (default: true)
  optimizationLevel?: 'none' | 'basic' | 'aggressive'; // Optimization level (default: 'basic')
  saveToDatabase?: boolean;     // Save compiled code to database (default: true)
}
```

**Output:**
```typescript
{
  success: boolean;
  workflowCode?: string;        // Generated workflow TypeScript code
  activitiesCode?: string;      // Generated activities TypeScript code
  workerCode?: string;          // Generated worker TypeScript code
  packageJson?: string;         // Generated package.json
  tsConfig?: string;            // Generated tsconfig.json
  errors: CompilerError[];      // Compilation errors
  warnings: CompilerWarning[];  // Compilation warnings
  metadata?: CompilerMetadata;  // Compilation metadata
}
```

**Example:**
```typescript
const result = await trpc.compiler.compile.mutate({
  workflowId: 'workflow-123',
  includeComments: true,
  strictMode: true,
  optimizationLevel: 'basic',
});

if (result.success) {
  console.log('Compiled code:', result.workflowCode);
}
```

---

### 2. `compiler.compileDefinition`

Compile a workflow definition directly (without database interaction).

**Type:** Mutation
**Auth:** Protected (requires authentication)

**Input:**
```typescript
{
  workflow: WorkflowDefinition;  // Workflow definition to compile
  includeComments?: boolean;     // Include comments (default: true)
  strictMode?: boolean;          // TypeScript strict mode (default: true)
  optimizationLevel?: 'none' | 'basic' | 'aggressive'; // Optimization (default: 'basic')
}
```

**Output:** Same as `compiler.compile`

**Example:**
```typescript
const workflowDef = {
  id: 'my-workflow',
  name: 'My Workflow',
  nodes: [
    { id: 'node-1', type: 'trigger', data: { label: 'Start' }, position: { x: 0, y: 0 } },
    { id: 'node-2', type: 'activity', data: { label: 'Process', componentName: 'process' }, position: { x: 100, y: 0 } },
  ],
  edges: [
    { id: 'edge-1', source: 'node-1', target: 'node-2' },
  ],
  variables: [],
  settings: {},
};

const result = await trpc.compiler.compileDefinition.mutate({
  workflow: workflowDef,
});
```

---

### 3. `compiler.getCompiledCode`

Retrieve previously compiled code for a workflow.

**Type:** Query
**Auth:** Protected (requires authentication)

**Input:**
```typescript
{
  workflowId: string;  // ID of workflow
}
```

**Output:**
```typescript
{
  id: string;           // Workflow ID
  name: string;         // Workflow name
  compiledCode: string; // Previously compiled TypeScript code
  compiledAt: string;   // Timestamp of compilation
}
```

**Example:**
```typescript
const compiled = await trpc.compiler.getCompiledCode.query({
  workflowId: 'workflow-123',
});

console.log('Compiled code:', compiled.compiledCode);
```

---

### 4. `compiler.validate`

Validate a workflow without compiling it.

**Type:** Mutation
**Auth:** Protected (requires authentication)

**Input:**
```typescript
{
  workflowId?: string;          // ID of workflow to validate
  workflow?: WorkflowDefinition; // Or provide definition directly
}
```

**Output:**
```typescript
{
  valid: boolean;               // Whether workflow is valid
  errors: CompilerError[];      // Validation errors
  warnings: CompilerWarning[];  // Validation warnings
  metadata?: CompilerMetadata;  // Validation metadata
}
```

**Example:**
```typescript
// Validate by ID
const result = await trpc.compiler.validate.mutate({
  workflowId: 'workflow-123',
});

// Validate definition
const result = await trpc.compiler.validate.mutate({
  workflow: workflowDef,
});

if (!result.valid) {
  console.error('Validation errors:', result.errors);
}
```

---

### 5. `compiler.preview`

Preview compiled code without saving to database.

**Type:** Query
**Auth:** Protected (requires authentication)

**Input:**
```typescript
{
  workflowId: string;  // ID of workflow to preview
}
```

**Output:**
```typescript
{
  workflowCode: string;    // Generated workflow code
  activitiesCode: string;  // Generated activities code
  workerCode: string;      // Generated worker code
  packageJson: string;     // Generated package.json
  tsConfig: string;        // Generated tsconfig.json
}
```

**Example:**
```typescript
const preview = await trpc.compiler.preview.query({
  workflowId: 'workflow-123',
});

console.log('Preview:', preview.workflowCode);
```

---

### 6. `compiler.getMetadata`

Get compiler capabilities and metadata.

**Type:** Query
**Auth:** Protected (requires authentication)

**Input:** None

**Output:**
```typescript
{
  version: string;              // Compiler version
  supportedNodeTypes: string[]; // Supported workflow node types
  optimizationLevels: string[]; // Available optimization levels
  features: string[];           // Compiler features
}
```

**Example:**
```typescript
const metadata = await trpc.compiler.getMetadata.query();

console.log('Supported node types:', metadata.supportedNodeTypes);
// Output: ['trigger', 'activity', 'agent', 'conditional', 'loop', ...]
```

---

## Types

### WorkflowDefinition

```typescript
interface WorkflowDefinition {
  id: string;
  name: string;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  variables: WorkflowVariable[];
  settings: WorkflowSettings;
}
```

### WorkflowNode

```typescript
interface WorkflowNode {
  id: string;
  type: 'trigger' | 'activity' | 'agent' | 'conditional' | 'loop' | 'child-workflow' | 'signal' | 'phase' | 'retry' | 'state-variable' | 'api-endpoint' | 'condition' | 'end';
  data: NodeData;
  position: { x: number; y: number };
}
```

### CompilerError

```typescript
interface CompilerError {
  message: string;
  nodeId?: string;
  type: 'validation' | 'generation' | 'pattern' | 'dependency';
  severity: 'error' | 'fatal';
}
```

### CompilerWarning

```typescript
interface CompilerWarning {
  message: string;
  nodeId?: string;
  type: 'optimization' | 'deprecation' | 'best-practice';
}
```

---

## Error Handling

All endpoints use standard tRPC error handling:

```typescript
try {
  const result = await trpc.compiler.compile.mutate({ workflowId });
} catch (error) {
  if (error.code === 'NOT_FOUND') {
    console.error('Workflow not found');
  } else if (error.code === 'UNAUTHORIZED') {
    console.error('Not authenticated');
  }
}
```

Common error codes:
- `NOT_FOUND` - Workflow not found or not authorized
- `UNAUTHORIZED` - User not authenticated
- `BAD_REQUEST` - Invalid input parameters
- `INTERNAL_SERVER_ERROR` - Compilation failed

---

## Testing

See `compiler.test.ts` for comprehensive integration tests covering:

- Basic compilation
- Validation
- Error handling
- Multiple node types
- Retry policies
- Performance with large workflows

---

## Database Schema

The compiler uses the `compiled_typescript` field in the `workflows` table:

```sql
ALTER TABLE workflows
ADD COLUMN compiled_typescript TEXT;
```

This field stores the generated workflow code when `saveToDatabase: true` is used in the `compile` endpoint.

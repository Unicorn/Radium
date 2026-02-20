# Component Builder Agent Architecture

This document describes the internal architecture of the Component Builder Agent, the AI-powered system that guides users through designing and generating new workflow components.

## Overview

The Component Builder Agent is a conversational AI system built on Claude that:
- Gathers requirements through natural conversation
- Finds similar existing components for reference
- Designs type-safe schemas for Rust and TypeScript
- Generates production-ready code artifacts
- Validates and saves components to storage

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       Component Builder Agent                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                         State Machine                                  │  │
│  │                                                                        │  │
│  │  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐           │  │
│  │  │gathering │──►│designing │──►│refining  │──►│generating│           │  │
│  │  └──────────┘   └──────────┘   └──────────┘   └──────────┘           │  │
│  │       │                              ▲              │                  │  │
│  │       │              feedback        │              │                  │  │
│  │       │              loop ───────────┘              ▼                  │  │
│  │       │                                       ┌──────────┐            │  │
│  │       │                                       │reviewing │            │  │
│  │       │                                       └──────────┘            │  │
│  │       │                                             │                 │  │
│  │       │                                             ▼                 │  │
│  │       │                                       ┌──────────┐            │  │
│  │       └───────────────────────────────────────│ complete │            │  │
│  │                                               └──────────┘            │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                        Core Components                                 │  │
│  │                                                                        │  │
│  │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐    │  │
│  │  │  Claude API      │  │ Knowledge        │  │  Component       │    │  │
│  │  │  Integration     │  │ Retrieval        │  │  Storage         │    │  │
│  │  └──────────────────┘  └──────────────────┘  └──────────────────┘    │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## State Machine

The agent operates as a finite state machine with six phases:

### 1. Gathering Phase

**Purpose**: Collect requirements from the user through conversation.

**Behavior**:
- Accepts natural language descriptions
- Queries knowledge base for similar components
- Asks clarifying questions about inputs, outputs, validation
- Transitions to `designing` when requirements are sufficient

**Transition Triggers**:
- Agent response contains phrases like "enough information", "ready to design"

### 2. Designing Phase

**Purpose**: Create the initial component schema based on gathered requirements.

**Behavior**:
- Retrieves detailed knowledge from similar components
- Uses Claude to design input/output schemas
- Applies patterns from existing components
- Generates Rust and TypeScript type names

**Output**:
- `ComponentDesign` object with schemas, validation rules, connections
- Automatically transitions to `refining`

### 3. Refining Phase

**Purpose**: Iterate on the design based on user feedback.

**Behavior**:
- Accepts change requests from user
- Updates design incrementally
- Loops back on itself for multiple iterations

**Transition Triggers**:
- User says "looks good", "approve", "generate"
- Transitions to `generating`

### 4. Generating Phase

**Purpose**: Generate all code artifacts from the approved design.

**Behavior**:
- Generates Rust schema with serde/validator derives
- Generates TypeScript interfaces and activity code
- Creates test cases
- Produces migration record YAML
- Validates generated artifacts

**Output**:
- `GeneratedArtifacts` object with all code
- Automatically transitions to `reviewing`

### 5. Reviewing Phase

**Purpose**: Allow user to review generated code and request changes.

**Behavior**:
- Presents artifact summaries
- Answers questions about generated code
- Can regenerate if changes requested

**Transition Triggers**:
- User says "finalize", "save", "done"
- Transitions to `complete`

### 6. Complete Phase

**Purpose**: Save the component and conclude the session.

**Behavior**:
- Saves component to storage (filesystem or database)
- Provides summary of created files
- Session is done; requires reset for new component

## Core Components

### Claude API Integration

The agent uses the Anthropic Claude API for all AI interactions:

```typescript
interface BuilderAgentOptions {
  apiKey?: string;          // Anthropic API key
  model?: string;           // Default: claude-sonnet-4-20250514
  maxTokens?: number;       // Default: 2048
  temperature?: number;     // Default: 0.7
  debug?: boolean;          // Enable debug logging
}
```

Each phase has a specialized system prompt:
- `SYSTEM_PROMPTS.gathering`: Focus on requirements and clarification
- `SYSTEM_PROMPTS.designing`: Focus on schema design patterns
- `SYSTEM_PROMPTS.refining`: Focus on incorporating feedback
- `SYSTEM_PROMPTS.generating`: Focus on code quality
- `SYSTEM_PROMPTS.reviewing`: Focus on explaining code

### Knowledge Retrieval

The agent integrates with the Knowledge Retrieval system to:
1. Find similar components using semantic search
2. Extract applicable patterns from matches
3. Retrieve schema decisions for reference

```typescript
// During gathering phase
const similar = await this.knowledge.findSimilar(userMessage, 3);
this.state.requirement.similarComponents = similar.map(s => s.componentId);

// During designing phase
const similarKnowledge = this.state.requirement.similarComponents
  .map(id => this.knowledge.getComponent(id))
  .filter(Boolean);
```

### Component Storage

Generated components are saved via the storage system:

```typescript
const storedComponent: StoredComponent = {
  id: design.name,
  name: design.displayName,
  description: requirement.description,
  category: design.category,
  temporalType: design.temporalType,
  version: '1.0.0',
  artifacts: {
    rustSchema,
    typescriptCode,
    testCases,
    migrationRecord,
  },
  schema: { inputs, outputs, validationRules, connectionRules },
  metadata: { tags, usageCount: 0, isMarketplace: false },
  status: 'published',
};

await storage.save(storedComponent);
```

## State Structure

### BuilderState

```typescript
interface BuilderState {
  conversationId: string;           // Unique session ID
  phase: BuilderPhase;              // Current state machine phase
  requirement: ComponentRequirement; // Gathered requirements
  designDraft: ComponentDesign | null; // Current design
  generatedArtifacts: GeneratedArtifacts | null; // Generated code
  messages: Message[];              // Conversation history
  createdAt: Date;
  updatedAt: Date;
}
```

### ComponentRequirement

```typescript
interface ComponentRequirement {
  description: string;      // Natural language description
  category: string;         // Component category
  temporalType: string;     // activity, workflow, signal, etc.
  inputs: FieldRequirement[];
  outputs: FieldRequirement[];
  validationRules: string[];
  similarComponents: string[]; // IDs of similar components found
  additionalContext: string;
  constraints: string[];
}
```

### ComponentDesign

```typescript
interface ComponentDesign {
  name: string;             // snake_case identifier
  displayName: string;      // Human-readable name
  category: ComponentCategory;
  temporalType: TemporalType;
  description: string;
  inputSchema: SchemaDesign;
  outputSchema: SchemaDesign;
  validationRules: ValidationRule[];
  connections: ComponentConnectionRules;
  decisions: SchemaDecision[];  // Design decisions made
  appliedPatterns: string[];    // Patterns used from knowledge base
}
```

### GeneratedArtifacts

```typescript
interface GeneratedArtifacts {
  rustSchema: string;       // Complete Rust code
  typescriptCode: string;   // Complete TypeScript code
  testCases: string;        // Rust test code
  migrationRecord: string;  // YAML migration record
  validationStatus: {
    rustValid: boolean;
    rustErrors: string[];
    typescriptValid: boolean;
    typescriptErrors: string[];
    isValid: boolean;
  };
}
```

## Code Generation

### Rust Schema Generation

The agent generates Rust code following these patterns:

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ComponentNameInput {
    #[validate(length(min = 1))]
    pub required_field: String,

    #[serde(default)]
    pub optional_field: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentNameOutput {
    pub success: bool,
    pub result: Option<String>,
}
```

### TypeScript Generation

Generated TypeScript follows strict typing:

```typescript
export interface ComponentNameInput {
  requiredField: string;
  optionalField?: string;
}

export interface ComponentNameOutput {
  success: boolean;
  result?: string;
}

export async function executeComponentName(
  input: ComponentNameInput
): Promise<ComponentNameOutput> {
  // Implementation
}
```

## Artifact Validation

Generated code is validated before saving:

| Check | Rust | TypeScript |
|-------|------|------------|
| Struct/interface present | `struct` keyword | `interface` keyword |
| Serde derives | `Serialize`, `Deserialize` | N/A |
| No `any` types | N/A | No `: any` |
| Proper naming | snake_case | camelCase |

## Error Handling

### API Errors

```typescript
try {
  const response = await this.anthropic.messages.create({...});
} catch (error) {
  throw new Error('Failed to communicate with AI service');
}
```

### Storage Errors

If storage fails, the component is still available in the session:

```typescript
catch (error) {
  return `Component was generated but could not be saved.
Error: ${errorMessage}
Generated code is still available in the session.`;
}
```

## Thread Safety

The agent maintains conversation state internally. For multi-tenant usage:
- Create separate `ComponentBuilderAgent` instances per user session
- Use the `conversationId` to track sessions
- Call `reset()` to start a new conversation

## Configuration

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `ANTHROPIC_API_KEY` | Yes | API key for Claude |

### Default Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `model` | claude-sonnet-4-20250514 | Claude model to use |
| `maxTokens` | 2048 | Max response tokens |
| `temperature` | 0.7 | Generation temperature |

## Extension Points

### Custom Storage Backend

```typescript
const agent = new ComponentBuilderAgent(
  knowledge,
  { apiKey: process.env.ANTHROPIC_API_KEY },
  customStorageInstance
);
```

### Custom System Prompts

The system prompts are defined in `SYSTEM_PROMPTS` constant and can be modified for different behaviors.

## See Also

- [Knowledge Base Documentation](./knowledge-base.md)
- [API Reference](./api-reference.md)
- [User Guide](./user-guide.md)

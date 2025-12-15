# Component Builder

AI-powered system for creating new workflow components through conversational interaction.

## Overview

The Component Builder enables admins to create new workflow components by describing them in natural language. The system uses AI to:

1. Understand component requirements
2. Reference similar existing components
3. Design appropriate schemas
4. Generate production-ready Rust and TypeScript code
5. Create test cases and documentation

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Component Builder                         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────┐   ┌─────────────────┐                 │
│  │   Admin UI      │   │  API Routes     │                 │
│  │  /admin/        │   │  /api/component-│                 │
│  │  component-     │◄──┤  builder/chat   │                 │
│  │  builder        │   │                 │                 │
│  └────────┬────────┘   └────────┬────────┘                 │
│           │                     │                           │
│           └──────────┬──────────┘                           │
│                      │                                      │
│           ┌──────────▼──────────┐                          │
│           │  Builder Agent      │                          │
│           │  - Conversation     │                          │
│           │  - Schema Design    │                          │
│           │  - Code Generation  │                          │
│           └──────────┬──────────┘                          │
│                      │                                      │
│           ┌──────────▼──────────┐                          │
│           │  Knowledge Base     │                          │
│           │  - Migration Records│                          │
│           │  - Pattern Library  │                          │
│           │  - Templates        │                          │
│           └─────────────────────┘                          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Modules

### Knowledge Base (`knowledge-base/`)

Processes migration records from the `radium-workflow` crate and provides semantic search for finding similar components.

```typescript
import { initializeKnowledgeBase } from '@/lib/component-builder';

const { retrieval } = await initializeKnowledgeBase();

// Find similar components
const similar = await retrieval.findSimilar('I need a component that sends emails');

// Get component details
const httpRequest = retrieval.getComponent('http_request');
```

### Builder Agent (`agent/`)

AI-powered agent that guides users through component creation via conversation.

```typescript
import { ComponentBuilderAgent } from '@/lib/component-builder';

const agent = new ComponentBuilderAgent(knowledge);

// Start conversation
const response1 = await agent.chat('I need a component that sends webhooks');
// Agent asks clarifying questions...

const response2 = await agent.chat('It should support retry and authentication');
// Agent designs schema...

const response3 = await agent.chat('Looks good, generate the code');
// Agent generates Rust schema, TypeScript, and tests
```

### Templates (`templates/`)

Reusable component templates as starting points.

```typescript
import { getTemplateLibrary } from '@/lib/component-builder';

const library = getTemplateLibrary();

// Search templates
const results = library.search('email', { category: 'communication' });

// Get template by ID
const emailTemplate = library.get('email-sender');

// Customize template
const custom = library.applyCustomization({
  templateId: 'webhook',
  componentName: 'Slack Webhook',
  fieldCustomizations: [
    { originalName: 'url', newDescription: 'Slack webhook URL' },
  ],
  additionalFields: [],
  removedFields: [],
  customValidation: [],
});
```

## API Routes

### POST `/api/component-builder/chat`

Send a message to the Component Builder Agent.

**Request:**
```json
{
  "message": "I need a component that sends emails",
  "conversationId": "optional-existing-session-id"
}
```

**Response:**
```json
{
  "response": "I can help you create an email component...",
  "conversationId": "uuid",
  "phase": "gathering",
  "phaseChanged": false,
  "suggestedActions": ["Describe input fields", "Specify output fields"]
}
```

### GET `/api/component-builder/chat`

Get session status.

**Query params:**
- `conversationId` - Optional session ID

### DELETE `/api/component-builder/chat`

Delete a session.

**Query params:**
- `conversationId` - Required session ID

## Builder Phases

1. **Gathering** - Initial requirement collection
2. **Designing** - Schema design based on requirements
3. **Refining** - Iterative design refinement
4. **Generating** - Code artifact generation
5. **Reviewing** - Final review and approval
6. **Complete** - Component creation finished

## Built-in Templates

| Template | Category | Complexity | Description |
|----------|----------|------------|-------------|
| `email-sender` | communication | moderate | Send emails via SMTP |
| `webhook` | integration | simple | Send HTTP webhooks |
| `database-query` | data | moderate | Execute SQL queries |

## Usage Example

```typescript
// Initialize the Component Builder
const sessionManager = await initializeComponentBuilder();

// Create a new session
const agent = sessionManager.createSession();

// Start building a component
const res1 = await agent.chat('Create a Slack notification component');
// "I can help you create that. What information should the component receive?"

const res2 = await agent.chat('Channel, message, and optional attachments');
// "Got it. Let me design the schema..."

const res3 = await agent.chat('Looks good, generate');
// Generates Rust schema, TypeScript interfaces, and tests

// Access generated artifacts
const state = agent.getState();
console.log(state.generatedArtifacts?.rustSchema);
console.log(state.generatedArtifacts?.typescriptCode);
```

## Configuration

The Component Builder uses environment variables:

- `ANTHROPIC_API_KEY` - Required for AI functionality

## Testing

```bash
# Run all Component Builder tests
npx vitest run src/lib/component-builder/__tests__

# Run specific test file
npx vitest run src/lib/component-builder/__tests__/templates.test.ts
```

## Files Created

When a component is generated, the following files are created:

- `src/schema/components/{name}.rs` - Rust schema
- `templates/{name}.ts.hbs` - TypeScript template (if applicable)
- `tests/{name}_test.rs` - Test cases
- `component-records/{name}.yaml` - Migration record

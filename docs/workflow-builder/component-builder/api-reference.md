# Component Builder API Reference

This document describes the programmatic interfaces for the Component Builder system.

## REST API

### Chat Endpoint

Send messages to the component builder agent.

**POST** `/api/component-builder/chat`

#### Request Body

```json
{
  "message": "I need a component that sends emails",
  "sessionId": "optional-session-id"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `message` | string | Yes | User message to the agent |
| `sessionId` | string | No | Resume existing session |

#### Response

```json
{
  "response": "I can help you create an email sender component...",
  "phase": "gathering",
  "sessionId": "uuid-session-id",
  "suggestedActions": [
    "Describe the email inputs",
    "Specify output format",
    "Use email template"
  ],
  "artifacts": null
}
```

| Field | Type | Description |
|-------|------|-------------|
| `response` | string | Agent's response message |
| `phase` | string | Current builder phase |
| `sessionId` | string | Session identifier for continuation |
| `suggestedActions` | string[] | Suggested next actions |
| `artifacts` | object | Generated artifacts (when available) |

#### Phases

| Phase | Description |
|-------|-------------|
| `gathering` | Collecting requirements |
| `designing` | Creating schema design |
| `refining` | Refining based on feedback |
| `generating` | Generating code artifacts |
| `reviewing` | Final review before save |
| `complete` | Component creation complete |

### Session Info

Get information about a builder session.

**GET** `/api/component-builder/chat?sessionId={sessionId}`

#### Response

```json
{
  "sessionId": "uuid-session-id",
  "phase": "designing",
  "messageCount": 5,
  "hasDesign": true,
  "hasArtifacts": false,
  "createdAt": "2025-01-01T12:00:00Z",
  "lastActivity": "2025-01-01T12:05:00Z"
}
```

### Delete Session

End and cleanup a builder session.

**DELETE** `/api/component-builder/chat?sessionId={sessionId}`

#### Response

```json
{
  "success": true,
  "message": "Session deleted"
}
```

---

## TypeScript API

### ComponentBuilderAgent

Main class for conversational component building.

```typescript
import { ComponentBuilderAgent } from '@/lib/component-builder/agent/builder-agent';
import { KnowledgeRetrieval } from '@/lib/component-builder/knowledge-base/retrieval';

const knowledge = new KnowledgeRetrieval();
await knowledge.loadKnowledgeBase(records);

const agent = new ComponentBuilderAgent(knowledge);
```

#### Methods

##### `chat(message: string): Promise<ChatResponse>`

Send a message to the agent.

```typescript
const response = await agent.chat('Create a webhook sender component');

console.log(response.response);  // Agent's reply
console.log(response.phase);     // Current phase
console.log(response.artifacts); // Generated code (if available)
```

##### `getState(): BuilderState`

Get the current builder state.

```typescript
const state = agent.getState();

console.log(state.phase);           // Current phase
console.log(state.messages);        // Conversation history
console.log(state.designDraft);     // Schema design
console.log(state.generatedArtifacts); // Generated code
```

##### `reset(): void`

Reset the agent to start a new conversation.

```typescript
agent.reset();
// Agent is now ready for a new component
```

### BuilderState

State object returned by `getState()`.

```typescript
interface BuilderState {
  conversationId: string;
  phase: BuilderPhase;
  requirement: ComponentRequirement;
  designDraft: ComponentDesign | null;
  generatedArtifacts: GeneratedArtifacts | null;
  messages: Message[];
}
```

### GeneratedArtifacts

Generated code artifacts.

```typescript
interface GeneratedArtifacts {
  rustSchema: string;      // Rust struct definitions
  typescriptCode: string;  // TypeScript interfaces and code
  testCases: string;       // Rust test code
  migrationRecord: string; // YAML migration record
}
```

---

## Storage API

### ComponentStorage

Main class for component storage operations.

```typescript
import { ComponentStorage } from '@/lib/component-builder/storage';

const storage = new ComponentStorage({
  type: 'filesystem',
  config: { baseDir: '/path/to/components' },
});

await storage.initialize();
```

#### Methods

##### `save(component: StoredComponent): Promise<StoredComponent>`

Save a component to storage.

```typescript
const saved = await storage.save({
  id: 'my-component',
  name: 'My Component',
  description: 'Does something useful',
  category: 'utilities',
  temporalType: 'activity',
  version: '1.0.0',
  artifacts: {
    rustSchema: '...',
    typescriptCode: '...',
    testCases: '...',
    migrationRecord: '...',
  },
  schema: { inputs: [], outputs: [], validationRules: [], connectionRules: {} },
  metadata: { tags: [], usageCount: 0, isMarketplace: false },
  createdBy: 'admin',
  status: 'draft',
});
```

##### `get(id: string): Promise<StoredComponent | null>`

Retrieve a component by ID.

```typescript
const component = await storage.get('my-component');
if (component) {
  console.log(component.name);
}
```

##### `list(options?: ListOptions): Promise<ListResult>`

List components with filtering and pagination.

```typescript
const result = await storage.list({
  category: 'utilities',
  status: 'published',
  search: 'email',
  offset: 0,
  limit: 20,
  sortBy: 'name',
  sortOrder: 'asc',
});

console.log(result.total);      // Total matching components
console.log(result.components); // Array of components
```

##### `update(id: string, updates: Partial<StoredComponent>): Promise<StoredComponent | null>`

Update a component.

```typescript
const updated = await storage.update('my-component', {
  description: 'Updated description',
  status: 'published',
});
```

##### `delete(id: string): Promise<boolean>`

Delete a component.

```typescript
const deleted = await storage.delete('my-component');
console.log(deleted); // true if deleted
```

##### `search(query: string, limit?: number): Promise<StoredComponent[]>`

Search components by text.

```typescript
const results = await storage.search('email sender', 10);
```

##### `incrementUsage(id: string): Promise<void>`

Increment the usage counter.

```typescript
await storage.incrementUsage('my-component');
```

##### `exists(id: string): Promise<boolean>`

Check if a component exists.

```typescript
const exists = await storage.exists('my-component');
```

### StoredComponent

Component data structure.

```typescript
interface StoredComponent {
  id: string;
  name: string;
  description: string;
  category: string;
  temporalType: 'activity' | 'workflow' | 'signal' | 'query';
  version: string;
  artifacts: {
    rustSchema: string;
    typescriptCode: string;
    testCases: string;
    migrationRecord: string;
  };
  schema: {
    inputs: FieldDefinition[];
    outputs: FieldDefinition[];
    validationRules: ValidationRule[];
    connectionRules: ConnectionRules;
  };
  metadata: {
    tags: string[];
    usageCount: number;
    isMarketplace: boolean;
  };
  createdAt: Date;
  updatedAt: Date;
  createdBy: string;
  status: 'draft' | 'published' | 'deprecated';
}
```

---

## Template API

### ComponentTemplateLibrary

Access and manage component templates.

```typescript
import { ComponentTemplateLibrary } from '@/lib/component-builder/templates/library';

const library = new ComponentTemplateLibrary();
```

#### Methods

##### `getAll(): ComponentTemplate[]`

Get all available templates.

```typescript
const templates = library.getAll();
```

##### `get(id: string): ComponentTemplate | undefined`

Get a template by ID.

```typescript
const template = library.get('email-sender');
```

##### `search(query: string, filters?: SearchFilters): SearchResult`

Search templates.

```typescript
const result = library.search('webhook', {
  category: 'communication',
  complexity: 'simple',
});
```

##### `getByCategory(category: string): ComponentTemplate[]`

Get templates by category.

```typescript
const templates = library.getByCategory('communication');
```

##### `applyCustomization(customization: TemplateCustomization): ComponentTemplate`

Customize a template.

```typescript
const customized = library.applyCustomization({
  templateId: 'email-sender',
  componentName: 'custom-email',
  fieldCustomizations: [
    { originalName: 'subject', newDescription: 'Custom subject' },
  ],
  additionalFields: [
    { name: 'priority', rustType: 'String', ... },
  ],
  removedFields: ['attachments'],
  customValidation: [],
});
```

##### `addCustomTemplate(template: ComponentTemplate): void`

Add a custom template.

```typescript
library.addCustomTemplate({
  id: 'my-template',
  name: 'My Template',
  // ... template definition
});
```

---

## Knowledge Base API

### KnowledgeRetrieval

Query the component knowledge base.

```typescript
import { KnowledgeRetrieval } from '@/lib/component-builder/knowledge-base/retrieval';

const knowledge = new KnowledgeRetrieval();
await knowledge.loadKnowledgeBase(records);
```

#### Methods

##### `loadKnowledgeBase(records: ProcessedRecord[]): Promise<void>`

Load processed migration records into the knowledge base.

```typescript
await knowledge.loadKnowledgeBase(records);
```

##### `findSimilar(query: string, limit?: number): Promise<SimilarComponent[]>`

Find components similar to a description.

```typescript
const similar = await knowledge.findSimilar('send HTTP requests', 5);

for (const comp of similar) {
  console.log(comp.componentId, comp.similarity);
}
```

##### `getComponent(id: string): ProcessedRecord | undefined`

Get a specific component's knowledge.

```typescript
const knowledge = knowledge.getComponent('http_request');
```

### MigrationRecordProcessor

Process migration records for the knowledge base.

```typescript
import { MigrationRecordProcessor } from '@/lib/component-builder/knowledge-base/processor';

const processor = new MigrationRecordProcessor({
  recordsDir: '/path/to/component-records',
});

const records = await processor.processAll();
```

---

## Error Handling

All API methods may throw errors. Handle appropriately:

```typescript
try {
  const response = await agent.chat('...');
} catch (error) {
  if (error.message === 'Failed to communicate with AI service') {
    // Handle API error
  }
  throw error;
}
```

Common errors:

| Error | Cause | Solution |
|-------|-------|----------|
| `Failed to communicate with AI service` | API key invalid or network issue | Check API key and connectivity |
| `Component not found` | Invalid component ID | Verify component exists |
| `Session not found` | Session expired or invalid | Start new session |
| `Storage error` | Backend configuration issue | Check storage configuration |

---

## Rate Limits

The Component Builder uses the Anthropic API, which has rate limits:
- Requests per minute: Varies by plan
- Tokens per minute: Varies by plan

Monitor usage and implement appropriate backoff strategies for production use.

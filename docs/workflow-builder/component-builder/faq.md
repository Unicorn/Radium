# Component Builder FAQ

Frequently asked questions about the Component Builder system.

## General Questions

### What is the Component Builder?

The Component Builder is an AI-powered tool that helps you create new workflow components through natural conversation. Instead of manually writing Rust schemas and TypeScript code, you describe what you need and the AI generates production-ready code.

### Do I need coding experience to use it?

No. The conversational interface guides you through the process. However, understanding basic concepts like inputs, outputs, and data types will help you describe your requirements more precisely.

### What AI model does it use?

The Component Builder uses Claude (Anthropic's AI) for:
- Understanding your requirements
- Finding similar existing components
- Generating code artifacts

### Is my data sent to external services?

Yes, conversations are processed by the Anthropic Claude API. Your component descriptions and requirements are sent to Claude for processing. Generated code is stored locally in your configured storage backend.

---

## Component Design

### What types of components can I create?

- **Activities**: Single operations (API calls, database queries, etc.)
- **Workflows**: Multi-step processes with child workflows
- **Signals**: Event handlers for workflow communication
- **Queries**: Read-only state queries

### What input/output types are supported?

| Type | Rust | TypeScript | Use Case |
|------|------|------------|----------|
| Text | `String` | `string` | Names, URLs, messages |
| Number | `i32`, `i64`, `f64` | `number` | Counts, amounts |
| Boolean | `bool` | `boolean` | Flags, toggles |
| Optional | `Option<T>` | `T \| undefined` | Optional fields |
| List | `Vec<T>` | `T[]` | Collections |
| Map | `HashMap<K, V>` | `Record<K, V>` | Key-value pairs |
| JSON | `serde_json::Value` | `unknown` | Dynamic data |
| DateTime | `chrono::DateTime<Utc>` | `string` | Timestamps |

### How do I specify required vs optional fields?

During the conversation, indicate which fields are required:

```
User: I need these inputs:
- url (required)
- headers (optional)
- timeout (optional, default 30 seconds)
```

The agent will generate appropriate types:
- Required: `pub url: String`
- Optional: `pub headers: Option<HashMap<String, String>>`
- With default: `#[serde(default = "default_timeout")]`

### Can I add validation rules?

Yes. Specify validation during design:

```
User: The email field should be validated as a proper email address.
      The retry_count should be between 1 and 10.
```

Generated Rust:
```rust
#[validate(email)]
pub email: String,

#[validate(range(min = 1, max = 10))]
pub retry_count: i32,
```

---

## Troubleshooting

### "Failed to communicate with AI service"

**Cause**: API key missing or invalid, or network issues.

**Solution**:
1. Check `ANTHROPIC_API_KEY` environment variable is set
2. Verify the API key is valid at console.anthropic.com
3. Check network connectivity
4. Check Anthropic API status

### "Component already exists"

**Cause**: A component with that ID already exists in storage.

**Solution**:
- Choose a different name
- Or update the existing component instead of creating new

### "Invalid schema"

**Cause**: Generated schema has issues.

**Solution**:
- Simplify your requirements
- Be more specific about field types
- Check for circular references in your design

### Generated code has compilation errors

**Cause**: AI-generated code may have issues.

**Solution**:
1. Review the validation status in the artifacts
2. Request refinements: "The Rust code needs to import HashMap"
3. Manually edit after generation if needed

### Agent seems stuck or confused

**Cause**: Ambiguous or contradictory requirements.

**Solution**:
1. Reset the conversation: `agent.reset()`
2. Start fresh with clearer requirements
3. Answer one question at a time

### Knowledge base is empty

**Cause**: No migration records loaded.

**Solution**:
1. Check that migration records exist in `component-records/`
2. Verify YAML syntax is valid
3. Check logs for processing errors

---

## Storage

### Where are components saved?

Depends on your configured storage backend:

**Filesystem Backend**:
```
components/
  metadata/      → Component JSON files
  rust/          → Rust schema files
  typescript/    → TypeScript code
  tests/         → Test files
  records/       → Migration records
```

**Database Backend**:
- Stored in configured database
- Uses Prisma schema (when configured)

### How do I change storage backends?

```typescript
// Filesystem
const storage = new ComponentStorage({
  type: 'filesystem',
  config: { baseDir: '/path/to/components' },
});

// Database
const storage = new ComponentStorage({
  type: 'database',
  config: { connectionString: process.env.DATABASE_URL },
});
```

### Can I export/import components?

Yes. Components are stored as JSON with all artifacts:

```typescript
// Export
const component = await storage.get('my-component');
const exported = JSON.stringify(component, null, 2);

// Import
const imported = JSON.parse(exportedData);
await storage.save(imported);
```

---

## Templates

### What templates are available?

| Template | Category | Description |
|----------|----------|-------------|
| Email Sender | Communication | Send emails via SMTP |
| Webhook | Communication | Send HTTP webhooks |
| Database Query | Data | Execute SQL queries |

### How do I customize a template?

```typescript
const customized = library.applyCustomization({
  templateId: 'email-sender',
  componentName: 'marketing-email',
  fieldCustomizations: [
    { originalName: 'to', newDescription: 'Marketing list recipient' },
  ],
  additionalFields: [
    { name: 'campaign_id', rustType: 'String', ... },
  ],
  removedFields: ['cc', 'bcc'],
});
```

### Can I create my own templates?

Yes. Register custom templates:

```typescript
library.addCustomTemplate({
  id: 'my-template',
  name: 'My Template',
  description: 'Description',
  category: 'custom',
  // ... schema definitions
});
```

---

## API Usage

### How do I use the Component Builder programmatically?

```typescript
import { ComponentBuilderAgent } from '@/lib/component-builder/agent/builder-agent';
import { KnowledgeRetrieval } from '@/lib/component-builder/knowledge-base/retrieval';

// Initialize knowledge base
const knowledge = new KnowledgeRetrieval();
await knowledge.loadKnowledgeBase(records);

// Create agent
const agent = new ComponentBuilderAgent(knowledge);

// Chat
const response = await agent.chat('I need a component that...');
console.log(response.response);
console.log(response.phase);
```

### How do I handle session state?

The agent maintains state internally. For multiple users:

```typescript
// Create per-user agents
const userAgents = new Map<string, ComponentBuilderAgent>();

function getAgentForUser(userId: string): ComponentBuilderAgent {
  if (!userAgents.has(userId)) {
    userAgents.set(userId, new ComponentBuilderAgent(knowledge));
  }
  return userAgents.get(userId)!;
}
```

### How do I get the generated artifacts?

```typescript
const state = agent.getState();

if (state.generatedArtifacts) {
  console.log(state.generatedArtifacts.rustSchema);
  console.log(state.generatedArtifacts.typescriptCode);
  console.log(state.generatedArtifacts.testCases);
}
```

---

## Best Practices

### Tips for better results

1. **Be specific**: "URL field that must be HTTPS" vs "URL field"
2. **Give examples**: "Like the http_request component but for GraphQL"
3. **Iterate**: Request refinements rather than starting over
4. **Review carefully**: AI-generated code should be reviewed before use

### Naming conventions

| Element | Convention | Example |
|---------|------------|---------|
| Component ID | snake_case | `email_sender` |
| Display name | Title Case | `Email Sender` |
| Rust fields | snake_case | `retry_count` |
| TypeScript fields | camelCase | `retryCount` |
| Rust structs | PascalCase | `EmailSenderInput` |
| TypeScript interfaces | PascalCase | `EmailSenderInput` |

### When NOT to use the Component Builder

- Simple components with 1-2 fields (faster to write manually)
- Highly specialized components requiring custom Rust code
- Components that need specific third-party crate integrations

---

## Performance

### How fast is component generation?

| Phase | Typical Time |
|-------|--------------|
| Gathering | 2-5 messages |
| Designing | 1-2 seconds |
| Generating | 3-5 seconds |
| Saving | < 100ms |

### Rate limits

The Component Builder uses the Anthropic API which has rate limits. Monitor:
- Requests per minute
- Tokens per minute

### Memory usage

- Knowledge base: ~5 KB per component
- Agent state: ~10 KB per session
- Generated artifacts: ~20 KB per component

---

## Security

### Is generated code safe?

Generated code follows security best practices:
- No hardcoded secrets
- Input validation by default
- Proper error handling

However, always review generated code before deploying to production.

### API key security

- Never commit API keys to version control
- Use environment variables
- Rotate keys regularly
- Use separate keys for dev/prod

---

## See Also

- [User Guide](./user-guide.md)
- [Admin Guide](./admin-guide.md)
- [API Reference](./api-reference.md)
- [Agent Architecture](./agent-architecture.md)
- [Knowledge Base](./knowledge-base.md)

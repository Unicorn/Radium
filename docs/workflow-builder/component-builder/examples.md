# Component Builder Examples

This document provides practical examples of creating components with the Component Builder.

## Example 1: Simple Logging Component

Create a basic logging component with message and level inputs.

### Conversation Flow

```
User: I need a component that logs messages with different severity levels.

Agent: I can help you create a logging component. Let me ask a few questions:
- What log levels do you need? (e.g., info, warn, error, debug)
- Should the message be required or optional?
- What output do you need - just success, or also timestamp?

User: Levels should be: debug, info, warn, error. Message is required.
      Output should be success boolean and the timestamp when logged.

Agent: Great! Here's the proposed schema:

Input Schema:
- message: String (required) - The log message
- level: LogLevel enum (debug, info, warn, error) - defaults to "info"

Output Schema:
- success: bool - Whether logging succeeded
- timestamp: String - ISO 8601 timestamp of the log

Does this look correct?

User: Yes, looks good. Generate the code.
```

### Generated Rust Schema

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SimpleLoggerInput {
    #[validate(length(min = 1, message = "Message cannot be empty"))]
    pub message: String,

    #[serde(default)]
    pub level: LogLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleLoggerOutput {
    pub success: bool,
    pub timestamp: String,
}
```

### Generated TypeScript

```typescript
export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

export interface SimpleLoggerInput {
  message: string;
  level?: LogLevel;
}

export interface SimpleLoggerOutput {
  success: boolean;
  timestamp: string;
}

export function isSimpleLoggerInput(value: unknown): value is SimpleLoggerInput {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;

  if (typeof obj.message !== 'string') return false;
  if (obj.level !== undefined) {
    if (!['debug', 'info', 'warn', 'error'].includes(obj.level as string)) {
      return false;
    }
  }

  return true;
}

export async function executeSimpleLogger(
  input: SimpleLoggerInput
): Promise<SimpleLoggerOutput> {
  const level = input.level || 'info';
  const timestamp = new Date().toISOString();

  console[level === 'warn' ? 'warn' : level === 'error' ? 'error' : 'log'](
    `[${level.toUpperCase()}] ${timestamp}: ${input.message}`
  );

  return {
    success: true,
    timestamp,
  };
}
```

---

## Example 2: HTTP Webhook Sender

Create a webhook component with configurable HTTP methods.

### Conversation Flow

```
User: Create a webhook sender that can POST or PUT to a URL with JSON payload.

Agent: I'll create a webhook sender component. Some questions:
- Should the URL be validated?
- What headers should be configurable?
- Do you need retry logic?
- What should the output contain?

User: URL should be valid HTTPS. Headers should be configurable as key-value pairs.
      No retry for now. Output should include status code and response body.

Agent: Here's the design:

Input Schema:
- url: String (required, URL validation, must be HTTPS)
- method: HttpMethod enum (POST, PUT) - defaults to POST
- payload: JSON object (required)
- headers: Map<String, String> (optional)

Output Schema:
- statusCode: i32
- body: String
- success: bool

Shall I proceed with generation?
```

### Generated Rust Schema

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Post,
    Put,
}

impl Default for HttpMethod {
    fn default() -> Self {
        HttpMethod::Post
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct WebhookSenderInput {
    #[validate(url(message = "Must be a valid URL"))]
    #[validate(custom(function = "validate_https"))]
    pub url: String,

    #[serde(default)]
    pub method: HttpMethod,

    pub payload: Value,

    #[serde(default)]
    pub headers: HashMap<String, String>,
}

fn validate_https(url: &str) -> Result<(), validator::ValidationError> {
    if url.starts_with("https://") {
        Ok(())
    } else {
        Err(validator::ValidationError::new("url_must_be_https"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookSenderOutput {
    pub status_code: i32,
    pub body: String,
    pub success: bool,
}
```

### Generated TypeScript

```typescript
export type HttpMethod = 'POST' | 'PUT';

export interface WebhookSenderInput {
  url: string;
  method?: HttpMethod;
  payload: Record<string, unknown>;
  headers?: Record<string, string>;
}

export interface WebhookSenderOutput {
  statusCode: number;
  body: string;
  success: boolean;
}

export async function executeWebhookSender(
  input: WebhookSenderInput
): Promise<WebhookSenderOutput> {
  const method = input.method || 'POST';

  try {
    const response = await fetch(input.url, {
      method,
      headers: {
        'Content-Type': 'application/json',
        ...input.headers,
      },
      body: JSON.stringify(input.payload),
    });

    const body = await response.text();

    return {
      statusCode: response.status,
      body,
      success: response.ok,
    };
  } catch (error) {
    return {
      statusCode: 0,
      body: error instanceof Error ? error.message : 'Unknown error',
      success: false,
    };
  }
}
```

---

## Example 3: Using a Template

Start from the email-sender template and customize it.

### Template Selection

```typescript
import { ComponentTemplateLibrary } from '@/lib/component-builder/templates/library';

const library = new ComponentTemplateLibrary();

// Search for email templates
const results = library.search('email');
console.log(results.templates.map(t => t.name));
// ['Email Sender']

// Get the template
const template = library.get('email-sender');
```

### Customization

```typescript
const customized = library.applyCustomization({
  templateId: 'email-sender',
  componentName: 'marketing-email',

  // Modify existing fields
  fieldCustomizations: [
    {
      originalName: 'to',
      newDescription: 'Marketing list recipient email',
    },
    {
      originalName: 'body',
      newDescription: 'Marketing email HTML content',
    },
  ],

  // Add new fields
  additionalFields: [
    {
      name: 'campaign_id',
      rustType: 'String',
      typescriptType: 'string',
      required: true,
      description: 'Marketing campaign identifier',
    },
    {
      name: 'tracking_enabled',
      rustType: 'bool',
      typescriptType: 'boolean',
      required: false,
      default: 'true',
      description: 'Enable open/click tracking',
    },
  ],

  // Remove fields we don't need
  removedFields: ['cc', 'bcc'],

  // Add custom validation
  customValidation: [
    {
      field: 'campaign_id',
      rule: 'pattern',
      params: { pattern: '^CAMP-[0-9]+$' },
    },
  ],
});
```

### Save Customized Component

```typescript
import { ComponentStorage, DatabaseStorageBackend } from '@/lib/component-builder/storage';

const storage = new ComponentStorage({
  type: 'custom',
  customBackend: new DatabaseStorageBackend(),
});

await storage.initialize();

await storage.save({
  id: customized.id,
  name: customized.name,
  description: 'Send marketing emails with campaign tracking',
  category: 'communication',
  temporalType: 'activity',
  version: '1.0.0',
  artifacts: {
    rustSchema: generateRustFromTemplate(customized),
    typescriptCode: generateTypeScriptFromTemplate(customized),
    testCases: generateTestsFromTemplate(customized),
    migrationRecord: generateMigrationRecord(customized),
  },
  schema: {
    inputs: customized.inputSchema.fields,
    outputs: customized.outputSchema.fields,
    validationRules: customized.validationRules,
    connectionRules: { allowedSources: ['*'], allowedTargets: ['*'] },
  },
  metadata: {
    tags: ['email', 'marketing', 'campaign'],
    usageCount: 0,
    isMarketplace: false,
  },
  createdBy: 'admin',
  status: 'draft',
});
```

---

## Example 4: Programmatic Component Creation

Create a component entirely through the API.

```typescript
import { ComponentBuilderAgent } from '@/lib/component-builder/agent/builder-agent';
import { KnowledgeRetrieval } from '@/lib/component-builder/knowledge-base/retrieval';
import { MigrationRecordProcessor } from '@/lib/component-builder/knowledge-base/processor';

async function createSlackNotifier() {
  // Initialize knowledge base
  const processor = new MigrationRecordProcessor({
    recordsDir: './component-records',
  });
  const records = await processor.processAll();

  const knowledge = new KnowledgeRetrieval();
  await knowledge.loadKnowledgeBase(records);

  // Create agent
  const agent = new ComponentBuilderAgent(knowledge);

  // Step 1: Describe the component
  let response = await agent.chat(
    'Create a Slack notification component with channel, message, and optional emoji inputs. ' +
    'Output should be success boolean and message_ts (Slack message timestamp).'
  );
  console.log('Phase:', response.phase);

  // Step 2: Confirm requirements
  response = await agent.chat('Yes, that matches my requirements. Please design it.');
  console.log('Phase:', response.phase);

  // Step 3: Generate code
  response = await agent.chat('Looks good, generate the code.');
  console.log('Phase:', response.phase);

  // Step 4: Get artifacts
  const state = agent.getState();
  if (state.generatedArtifacts) {
    console.log('Rust Schema:', state.generatedArtifacts.rustSchema);
    console.log('TypeScript:', state.generatedArtifacts.typescriptCode);
    console.log('Tests:', state.generatedArtifacts.testCases);

    // Save to storage
    const storage = new ComponentStorage({
      type: 'filesystem',
      config: { baseDir: './components' },
    });
    await storage.initialize();

    await storage.save({
      id: 'slack-notifier',
      name: 'Slack Notifier',
      description: 'Send notifications to Slack channels',
      category: 'communication',
      temporalType: 'activity',
      version: '1.0.0',
      artifacts: state.generatedArtifacts,
      schema: {
        inputs: state.designDraft?.inputSchema.fields || [],
        outputs: state.designDraft?.outputSchema.fields || [],
        validationRules: state.designDraft?.validationRules || [],
        connectionRules: { allowedSources: ['*'], allowedTargets: ['*'] },
      },
      metadata: {
        tags: ['slack', 'notification', 'messaging'],
        usageCount: 0,
        isMarketplace: false,
      },
      createdBy: 'api',
      status: 'draft',
    });

    console.log('Component saved!');
  }
}

createSlackNotifier().catch(console.error);
```

---

## Example 5: Batch Component Management

Manage multiple components programmatically.

```typescript
import { ComponentStorage, DatabaseStorageBackend } from '@/lib/component-builder/storage';

async function batchOperations() {
  const storage = new ComponentStorage({
    type: 'custom',
    customBackend: new DatabaseStorageBackend(),
  });
  await storage.initialize();

  // List all draft components
  const drafts = await storage.list({
    status: 'draft',
    sortBy: 'updatedAt',
    sortOrder: 'desc',
  });
  console.log(`Found ${drafts.total} draft components`);

  // Publish all drafts older than 7 days
  const oneWeekAgo = new Date();
  oneWeekAgo.setDate(oneWeekAgo.getDate() - 7);

  for (const component of drafts.components) {
    if (component.updatedAt < oneWeekAgo) {
      await storage.update(component.id, { status: 'published' });
      console.log(`Published: ${component.name}`);
    }
  }

  // Search for unused components
  const allComponents = await storage.list({ status: 'published' });
  const unused = allComponents.components.filter(c => c.metadata.usageCount === 0);
  console.log(`${unused.length} components have never been used`);

  // Export component data
  const exported = allComponents.components.map(c => ({
    id: c.id,
    name: c.name,
    category: c.category,
    version: c.version,
    usageCount: c.metadata.usageCount,
    createdAt: c.createdAt.toISOString(),
  }));
  console.log(JSON.stringify(exported, null, 2));
}

batchOperations().catch(console.error);
```

---

## Common Patterns

### Input Validation Pattern

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MyInput {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(url(message = "Invalid URL"))]
    pub webhook_url: String,

    #[validate(length(min = 1, max = 1000))]
    pub message: String,

    #[validate(range(min = 1, max = 10))]
    pub retry_count: i32,
}
```

### Optional Fields Pattern

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyInput {
    pub required_field: String,

    #[serde(default)]
    pub optional_with_default: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub truly_optional: Option<String>,
}
```

### Enum Pattern

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Medium
    }
}
```

### Error Output Pattern

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyOutput {
    pub success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,

    pub result: Option<ResultData>,
}
```

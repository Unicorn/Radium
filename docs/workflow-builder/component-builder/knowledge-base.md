# Component Builder Knowledge Base

This document describes the knowledge base system that powers the Component Builder's AI-assisted component generation.

## Overview

The Knowledge Base is a semantic search system that:
- Stores processed migration records from existing components
- Finds similar components using Claude AI
- Extracts patterns for schema design
- Provides decisions and rationale for reference

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          Knowledge Base System                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                    Migration Record Processor                          │  │
│  │                                                                        │  │
│  │  YAML Files  ──►  Parse  ──►  Extract Patterns  ──►  ProcessedRecord  │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                        │                                     │
│                                        ▼                                     │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                       Knowledge Retrieval                              │  │
│  │                                                                        │  │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐       │  │
│  │  │  In-Memory      │  │  Claude AI      │  │  Pattern        │       │  │
│  │  │  Storage        │  │  Semantic Search│  │  Aggregation    │       │  │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘       │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Data Sources

### Migration Records

The knowledge base is built from YAML migration records located in:
```
crates/radium-workflow/component-records/
```

Each record documents a component's:
- Schema design decisions
- Input/output structure
- Validation patterns
- Lessons learned

### Record Structure

```yaml
component:
  name: http_request
  category: activities
  version: 1.0.0
  description: Execute HTTP requests
  temporal_type: activity

migration:
  migrated_by: Developer Name
  migration_date: 2025-01-01
  difficulty: medium
  breaking_changes: false

schema_decisions:
  - field: url
    decision: Use String with URL validation
    rationale: URLs can be dynamic expressions
    alternatives_considered:
      - approach: Use url::Url type
        pros: [Type safety]
        cons: [Serialization complexity]
        why_rejected: Expressions need string type

input_schema:
  rust_struct: HttpRequestInput
  typescript_interface: HttpRequestInput
  fields:
    - name: url
      rust_type: String
      typescript_type: string
      required: true
      description: Target URL

output_schema:
  rust_struct: HttpRequestOutput
  typescript_interface: HttpRequestOutput
  fields:
    - name: status_code
      rust_type: u16
      typescript_type: number
      required: true

lessons_learned:
  what_worked_well:
    - Using enum for HTTP methods
  challenges:
    - challenge: Header handling
      solution: Used HashMap<String, String>
  recommendations:
    - Always validate URLs at runtime
```

## Processing Pipeline

### MigrationRecordProcessor

Transforms raw YAML into knowledge base entries:

```typescript
import { MigrationRecordProcessor } from '@/lib/component-builder/knowledge-base/processor';

const processor = new MigrationRecordProcessor({
  recordsDir: './component-records',
});

const records = await processor.processAll();
// Returns ProcessedRecord[]
```

### ProcessedRecord Structure

```typescript
interface ProcessedRecord {
  id: string;                    // Component identifier
  content: string;               // Searchable text content
  metadata: ComponentMetadata;   // Category, type, complexity
  patterns: ComponentPatterns;   // Extracted patterns
  inputSchema: InputSchema;      // Input field definitions
  outputSchema: OutputSchema;    // Output field definitions
  decisions: SchemaDecision[];   // Design decisions
  lessonsLearned: LessonsLearned;
  relatedComponents: RelatedComponent[];
}
```

## Knowledge Retrieval

### Initialization

```typescript
import { KnowledgeRetrieval } from '@/lib/component-builder/knowledge-base/retrieval';

const knowledge = new KnowledgeRetrieval({
  apiKey: process.env.ANTHROPIC_API_KEY,
  model: 'claude-sonnet-4-20250514',
  maxResults: 5,
});

await knowledge.loadKnowledgeBase(records);
```

### Finding Similar Components

The `findSimilar` method uses Claude to semantically match queries:

```typescript
const similar = await knowledge.findSimilar(
  'I need a component that sends emails',
  5  // limit
);

// Returns:
[
  {
    componentId: 'smtp_sender',
    similarity: 0.92,
    reason: 'Both handle email sending functionality',
    relevantDecisions: [...],
    applicablePatterns: ['retry_pattern', 'auth_config']
  },
  {
    componentId: 'http_request',
    similarity: 0.75,
    reason: 'Similar outbound communication pattern',
    relevantDecisions: [...],
    applicablePatterns: ['timeout_handling']
  }
]
```

### Structured Queries

```typescript
const result = await knowledge.query('database query component');

// Returns KnowledgeQueryResult:
{
  query: 'database query component',
  similarComponents: [...],
  extractedPatterns: {
    inputValidation: [...],
    outputSchema: [...],
    errorHandling: [...],
    typescriptPatterns: [...],
    rustPatterns: [...]
  },
  suggestedDecisions: [...]
}
```

## Pattern Types

### Validation Patterns

```typescript
interface ValidationPattern {
  type: string;           // 'email', 'url', 'range', etc.
  field: string;          // Target field name
  rule: string;           // Validation rule expression
  rustImplementation: string; // Rust validator code
  rationale: string;      // Why this validation
}
```

Example patterns:
- Email validation: `#[validate(email)]`
- URL validation: `#[validate(url)]`
- Length constraints: `#[validate(length(min = 1, max = 255))]`
- Range validation: `#[validate(range(min = 0, max = 100))]`

### Schema Patterns

```typescript
interface SchemaPattern {
  fieldName: string;
  fieldType: string;      // Logical type
  rustType: string;       // Rust type mapping
  typescriptType: string; // TypeScript type mapping
  required: boolean;
  defaultValue?: string;
  serdeAnnotations: string[];
  description: string;
}
```

Common patterns:
| Field Type | Rust | TypeScript | Serde |
|------------|------|------------|-------|
| Text | `String` | `string` | `rename_all = "camelCase"` |
| Optional text | `Option<String>` | `string \| undefined` | `skip_serializing_if = "Option::is_none"` |
| Number | `i64` | `number` | `default` |
| Boolean | `bool` | `boolean` | `default` |
| List | `Vec<T>` | `T[]` | `default` |
| Map | `HashMap<String, T>` | `Record<string, T>` | `default` |
| JSON | `serde_json::Value` | `unknown` | - |

### Rust Patterns

```typescript
interface RustPattern {
  patternName: string;
  derives: string[];       // #[derive(...)]
  structs: string[];       // Struct names
  enums: string[];         // Enum names
  implementation: string;  // Code example
}
```

Standard derives by component type:
- **Activities**: `Debug, Clone, Serialize, Deserialize, Validate`
- **Signals**: `Debug, Clone, Serialize, Deserialize`
- **Queries**: `Debug, Clone, Serialize, Deserialize`

### TypeScript Patterns

```typescript
interface CodePattern {
  patternName: string;
  template: string;        // Handlebars template
  usage: string;           // When to use
  example: string;         // Code example
}
```

## Pattern Retrieval Methods

### Get Patterns by Field Type

```typescript
const patterns = knowledge.getPatternsByFieldType('email');
// Returns SchemaPattern[] matching email fields
```

### Get Validation Patterns

```typescript
const patterns = knowledge.getValidationPatterns('url');
// Returns ValidationPattern[] for URL validation
```

## Knowledge Base Statistics

```typescript
const stats = knowledge.getStats();

// Returns:
{
  totalComponents: 15,
  byCategory: {
    activities: 8,
    'control-flow': 4,
    triggers: 2,
    signals: 1
  },
  byTemporalType: {
    activity: 10,
    workflow: 3,
    signal: 1,
    query: 1
  },
  totalDecisions: 45,
  totalPatterns: 120
}
```

## Semantic Search Details

### How Similarity Works

1. **Query Preparation**: User's natural language description is sent to Claude
2. **Component Summaries**: All components are summarized with:
   - ID, category, temporal type
   - Description
   - Input/output field names
3. **Claude Analysis**: Claude ranks components by:
   - Similar functionality or purpose
   - Similar input/output patterns
   - Same category or temporal type
   - Shared design patterns
4. **Enrichment**: Results are enriched with:
   - Relevant schema decisions
   - Applicable patterns extracted

### Similarity Scoring

| Score | Meaning |
|-------|---------|
| 0.9 - 1.0 | Nearly identical purpose |
| 0.7 - 0.9 | Strongly related |
| 0.5 - 0.7 | Moderately similar |
| 0.3 - 0.5 | Loosely related |
| 0.0 - 0.3 | Minimal relevance |

## Pattern Aggregation

When multiple similar components are found, patterns are aggregated:

```typescript
private aggregatePatterns(similarComponents: SimilarComponent[]): ComponentPatterns {
  const inputValidation: ValidationPattern[] = [];

  for (const similar of similarComponents) {
    const record = this.knowledgeBase.get(similar.componentId);

    // Aggregate validation patterns (deduplicated)
    for (const pattern of record.patterns.inputValidation) {
      if (!inputValidation.find(p =>
        p.field === pattern.field && p.type === pattern.type
      )) {
        inputValidation.push(pattern);
      }
    }
  }

  return { inputValidation, outputSchema, errorHandling, ... };
}
```

## Adding New Knowledge

### Creating Migration Records

1. Create a YAML file in `component-records/`:

```yaml
component:
  name: my_new_component
  category: activities
  version: 1.0.0
  description: What this component does
  temporal_type: activity

# ... full record structure
```

2. Restart the application to reload the knowledge base

### Programmatic Addition

```typescript
const newRecord: ProcessedRecord = {
  id: 'my_component',
  content: 'Searchable description...',
  metadata: { ... },
  patterns: { ... },
  // ...
};

// Add to existing loaded knowledge base
knowledge.loadKnowledgeBase([...existingRecords, newRecord]);
```

## Performance Considerations

### Memory Usage

- Each ProcessedRecord is stored in-memory
- Typical record size: 2-5 KB
- 100 components ≈ 200-500 KB

### API Calls

- `findSimilar()`: 1 Claude API call per invocation
- `query()`: 1 Claude API call per invocation
- `getComponent()`: No API call (local lookup)

### Caching

The knowledge base is loaded once at startup. For production:
- Consider preloading during build
- Use persistent storage for large knowledge bases
- Cache frequently accessed components

## See Also

- [Agent Architecture](./agent-architecture.md)
- [API Reference](./api-reference.md)
- [Examples](./examples.md)

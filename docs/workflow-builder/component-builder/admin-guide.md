# Component Builder Admin Guide

This guide covers administrative tasks for managing the Component Builder system, including component library management, storage configuration, and monitoring.

## Component Library Management

### Viewing Components

Access the component library from **Admin > Components**.

The library displays:
- Component name and description
- Category (utilities, communication, data, etc.)
- Status (draft, published, deprecated)
- Usage count
- Last updated date

### Component Statuses

| Status | Description | Visible to Users |
|--------|-------------|-----------------|
| `draft` | Work in progress | No |
| `published` | Ready for use | Yes |
| `deprecated` | Scheduled for removal | Yes (with warning) |

### Publishing a Component

1. Navigate to the component detail page
2. Review all artifacts (Rust, TypeScript, tests)
3. Click **Publish**
4. Confirm the action

Published components:
- Appear in the component palette
- Can be used in workflows
- Usage is tracked

### Updating Components

When updating a published component:

1. Create a new version (keeps history)
2. Or update in place (for minor fixes)

Version updates are recommended for:
- Breaking schema changes
- New required fields
- Type changes

In-place updates are acceptable for:
- Documentation improvements
- Bug fixes that don't change the interface

### Deprecating Components

To deprecate a component:

1. Navigate to component details
2. Click **Deprecate**
3. Optionally specify a replacement component
4. Set a deprecation message

Deprecated components:
- Show a warning when used
- Don't appear in new workflow creation
- Existing workflows continue to work

### Deleting Components

**Warning**: Deleting components is destructive.

Before deleting:
1. Check usage count is 0
2. Verify no workflows reference it
3. Consider deprecating instead

To delete:
1. Navigate to component details
2. Click **Delete**
3. Confirm by typing the component name

## Storage Configuration

### Storage Backends

The Component Builder supports two storage backends:

#### Filesystem Backend

Best for:
- Open-source deployments
- Version-controlled components
- Git-based workflows

Configuration:
```typescript
const storage = new ComponentStorage({
  type: 'filesystem',
  config: {
    baseDir: '/path/to/components',
    createDirs: true,
  },
});
```

Directory structure:
```
components/
  metadata/           # Component JSON files
  rust/              # Rust schema files
  typescript/        # TypeScript code
  tests/             # Test files
  records/           # Migration records
```

#### Database Backend

Best for:
- Multi-tenant deployments
- Cloud-hosted environments
- High-availability requirements

Configuration:
```typescript
const storage = new ComponentStorage({
  type: 'database',
  config: {
    connectionString: process.env.DATABASE_URL,
  },
});
```

**Note**: The database backend currently uses in-memory storage. Prisma schema integration is planned.

### Switching Backends

To migrate between backends:

1. Export all components from current backend
2. Configure new backend
3. Import components to new backend
4. Update environment configuration
5. Restart the application

## Template Management

### Adding Templates

To add a new template:

1. Create template definition:
```typescript
const myTemplate: ComponentTemplate = {
  id: 'my-template',
  name: 'My Template',
  description: 'Description of what this template does',
  category: 'utilities',
  complexity: 'simple',
  // ... schema definitions
};
```

2. Register in template library:
```typescript
library.addCustomTemplate(myTemplate);
```

### Template Categories

| Category | Description |
|----------|-------------|
| `communication` | Email, SMS, Slack, etc. |
| `data` | Database, API, transforms |
| `integration` | Third-party services |
| `control` | Flow control, conditions |
| `ai` | AI/ML components |
| `custom` | User-created templates |

### Template Versioning

Templates include version numbers for tracking changes:

```typescript
{
  version: '1.0.0',
  // ... template content
}
```

Follow semantic versioning:
- MAJOR: Breaking changes
- MINOR: New features, backward compatible
- PATCH: Bug fixes

## Knowledge Base

### Understanding the Knowledge Base

The Component Builder uses a knowledge base built from migration records to inform AI decisions.

Knowledge sources:
- Component migration records (YAML)
- Schema patterns
- Validation patterns
- Error handling patterns

### Refreshing the Knowledge Base

The knowledge base refreshes on application startup. To manually refresh:

```typescript
const processor = new MigrationRecordProcessor({ recordsDir });
const records = await processor.processAll();
await knowledgeRetrieval.loadKnowledgeBase(records);
```

### Adding Knowledge

To expand the knowledge base:

1. Create migration records for new patterns
2. Place in `component-records/` directory
3. Restart or refresh the knowledge base

Migration record format:
```yaml
component:
  name: my_component
  category: utilities
  version: 1.0.0
  description: What this component does
  temporal_type: activity

migration:
  migrated_by: Admin Name
  migration_date: 2025-01-01
  difficulty: low

schema_decisions:
  - field: input_field
    decision: Use String instead of i32
    rationale: Better compatibility with external systems
```

## Monitoring

### Component Usage Metrics

Track component usage through:
- Usage count per component
- Workflows using each component
- Error rates

Access metrics at **Admin > Analytics > Components**.

### Builder Sessions

Monitor active builder sessions:
- Active conversations
- Session duration
- Completion rate

Sessions auto-cleanup after 30 minutes of inactivity.

### Performance Monitoring

Key metrics to monitor:
- Storage operation latency
- Knowledge base query time
- Code generation duration

Performance thresholds:
| Operation | Target |
|-----------|--------|
| Storage save | < 50ms |
| Storage get | < 10ms |
| Storage list | < 100ms |
| Template search | < 20ms |
| Knowledge processing | < 500ms |

## Security Considerations

### API Key Management

The Component Builder requires an Anthropic API key:
- Store in environment variables only
- Never commit to version control
- Rotate regularly
- Use separate keys for dev/prod

### Input Validation

All user inputs are validated:
- Component names: alphanumeric and underscores
- Field names: valid Rust identifiers
- Types: from allowed list only

### Generated Code Review

Before publishing components:
1. Review generated Rust schema
2. Check TypeScript code for issues
3. Run generated tests
4. Verify no sensitive data in outputs

## Troubleshooting

### Agent Not Responding

1. Check API key is set and valid
2. Verify network connectivity
3. Check rate limits
4. Review application logs

### Storage Errors

1. Check backend configuration
2. Verify permissions (filesystem) or connection (database)
3. Review storage directory structure
4. Check available disk space

### Knowledge Base Issues

1. Verify migration records exist
2. Check YAML syntax in records
3. Review processor output for errors
4. Ensure records directory is accessible

## Backup and Recovery

### Backing Up Components

For filesystem backend:
```bash
tar -czf components-backup.tar.gz /path/to/components/
```

For database backend:
```bash
pg_dump -t components > components-backup.sql
```

### Restoring Components

For filesystem backend:
```bash
tar -xzf components-backup.tar.gz -C /path/to/components/
```

For database backend:
```bash
psql < components-backup.sql
```

## Next Steps

- Review [API Reference](./api-reference.md) for automation
- See [Examples](./examples.md) for common patterns
- Read [User Guide](./user-guide.md) for end-user documentation

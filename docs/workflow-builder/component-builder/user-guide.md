# Component Builder User Guide

The Component Builder is an AI-powered tool that helps you create custom workflow components through natural conversation. This guide walks you through creating your first component.

## Overview

The Component Builder enables you to:
- Create new workflow components by describing what you need
- Use templates as starting points
- Generate production-ready Rust schemas and TypeScript code
- Save components to your component library

## Getting Started

### Accessing the Component Builder

Navigate to **Admin > Component Builder** in the workflow-builder interface.

You'll see two main panels:
1. **Conversation Panel** (left): Chat with the AI agent
2. **Preview Panel** (right): View generated code and artifacts

### Creating Your First Component

1. **Describe what you need**

   Start by describing the component you want to create in natural language:

   ```
   I need a component that sends notifications to Slack channels
   ```

2. **Answer clarifying questions**

   The agent will ask questions to understand your requirements:
   - What inputs does it need?
   - What should the output be?
   - Are there any validation rules?

3. **Review the design**

   Once requirements are gathered, the agent shows you the proposed schema:

   ```
   Input Schema:
   - channel: String (required)
   - message: String (required)
   - webhook_url: Option<String>

   Output Schema:
   - success: bool
   - error_message: Option<String>
   ```

4. **Refine or approve**

   - Say "looks good" or "approve" to proceed
   - Or request changes: "Add a 'priority' field"

5. **Generate code**

   The agent generates:
   - Rust schema with serde annotations
   - TypeScript interfaces and activity code
   - Test cases
   - Migration record

6. **Save your component**

   Review the generated code in the preview panel, then save to your component library.

## Using Templates

For common use cases, start with a template:

1. Click **Templates** in the component builder
2. Browse by category:
   - Communication (email, Slack, webhooks)
   - Data (database, API, transforms)
   - Integration (third-party services)
3. Select a template
4. Customize fields, validation, and naming
5. Generate and save

### Available Built-in Templates

| Template | Description |
|----------|-------------|
| Email Sender | Send emails via SMTP with HTML support |
| Webhook | Send HTTP webhooks with configurable methods |
| Database Query | Execute SQL queries with parameterized inputs |

## Visual Builder

The visual builder provides a drag-and-drop interface for schema design:

### Adding Fields

1. Open the **Schema Editor** tab
2. Drag field types from the palette:
   - **String**: Text values
   - **Number**: Integer or decimal values
   - **Boolean**: True/false flags
   - **Array**: Lists of items
   - **Object**: Nested structures
   - **DateTime**: Timestamps

3. Configure each field:
   - Name (snake_case for Rust)
   - Required vs optional
   - Default value
   - Description

### Adding Validation Rules

1. Open the **Validation** tab
2. Add rules for your fields:
   - **Required**: Field must be provided
   - **Email**: Must be valid email format
   - **URL**: Must be valid URL
   - **Min/Max Length**: String length limits
   - **Min/Max Value**: Numeric bounds
   - **Pattern**: Regex validation

### Configuring Connections

Define how your component connects in workflows:

1. Open the **Connections** tab
2. Set **Allowed Sources**: Components that can connect to this
3. Set **Allowed Targets**: Components this can connect to
4. Use `*` to allow any connection

## Tips for Good Components

### Naming Conventions

- Use `snake_case` for field names
- Use descriptive names (e.g., `retry_count` not `rc`)
- Component names should be verb-noun (e.g., `send_email`)

### Input Design

- Required fields first, optional fields last
- Provide sensible defaults where possible
- Use enums for fixed options (e.g., `log_level: info|warn|error`)

### Output Design

- Always include a success indicator
- Return error details for debugging
- Include relevant metadata (timestamps, IDs)

### Validation

- Validate at boundaries (inputs from external sources)
- Use appropriate types (don't validate URLs on strings that must be URLs)
- Provide clear error messages

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl + Enter` | Send message |
| `Cmd/Ctrl + R` | Reset conversation |
| `Cmd/Ctrl + S` | Save component |
| `Cmd/Ctrl + P` | Toggle preview |

## Troubleshooting

### "Component already exists"

A component with that name is already in your library. Either:
- Choose a different name
- Update the existing component instead

### "Invalid schema"

The generated schema has issues. Check:
- Field names are valid identifiers
- Types are supported
- No circular references

### "Generation failed"

The AI couldn't generate valid code. Try:
- Simplifying your requirements
- Breaking into smaller components
- Being more specific about types

## Next Steps

- Read the [Admin Guide](./admin-guide.md) for managing components
- See the [API Reference](./api-reference.md) for programmatic access
- Review [Examples](./examples.md) for common patterns

# JSON Schema Guide

Radium supports structured output with JSON schema enforcement, allowing you to get reliably formatted responses from AI models. This guide covers how to use JSON schemas with Radium's CLI and explains provider-specific differences.

## Quick Start

### Basic Usage

```bash
# JSON mode (no schema validation)
rad step agent-id "Generate a user profile" --response-format json

# JSON schema mode (with validation)
rad step agent-id "Extract user data" --response-format json-schema --response-schema user-schema.json

# Inline schema
rad step agent-id "Extract data" --response-format json-schema --response-schema '{"type":"object","properties":{"name":{"type":"string"}}}'
```

## Response Format Options

The `--response-format` argument accepts three values:

- **`text`**: Plain text output (default)
- **`json`**: JSON-formatted output without schema validation
- **`json-schema`**: JSON output conforming to a provided schema (requires `--response-schema`)

## Schema Input Methods

### File-Based Schema

Create a JSON schema file and reference it:

```bash
rad step agent-id "prompt" --response-format json-schema --response-schema schema.json
```

The schema file should contain valid JSON Schema:

```json
{
  "type": "object",
  "properties": {
    "name": {"type": "string"},
    "age": {"type": "number"}
  },
  "required": ["name"]
}
```

### Inline Schema

Pass the schema directly as a string:

```bash
rad step agent-id "prompt" --response-format json-schema --response-schema '{"type":"object","properties":{"name":{"type":"string"}}}'
```

## Example Schemas

### User Profile Schema

**File**: `examples/schema-examples/user-profile.json`

```json
{
  "type": "object",
  "properties": {
    "name": {"type": "string"},
    "email": {"type": "string"},
    "age": {"type": "number", "minimum": 0, "maximum": 150},
    "address": {
      "type": "object",
      "properties": {
        "street": {"type": "string"},
        "city": {"type": "string"},
        "zip": {"type": "string"}
      },
      "required": ["city"]
    }
  },
  "required": ["name", "email"]
}
```

**Usage**:
```bash
rad step extract-user "Extract user information from: John Doe, john@example.com, 30 years old" \
  --response-format json-schema \
  --response-schema examples/schema-examples/user-profile.json
```

### API Response Schema

**File**: `examples/schema-examples/api-response.json`

```json
{
  "type": "object",
  "properties": {
    "status": {"type": "string", "enum": ["success", "error"]},
    "data": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "id": {"type": "string"},
          "title": {"type": "string"},
          "price": {"type": "number"}
        },
        "required": ["id", "title"]
      }
    },
    "message": {"type": "string"}
  },
  "required": ["status"]
}
```

### Data Extraction Schema

**File**: `examples/schema-examples/data-extraction.json`

```json
{
  "type": "object",
  "properties": {
    "entities": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "type": {"type": "string"},
          "value": {"type": "string"},
          "confidence": {"type": "number", "minimum": 0, "maximum": 1}
        },
        "required": ["type", "value"]
      }
    },
    "summary": {"type": "string"}
  },
  "required": ["entities"]
}
```

### Enum Validation Schema

**File**: `examples/schema-examples/enum-validation.json`

```json
{
  "type": "object",
  "properties": {
    "status": {
      "type": "string",
      "enum": ["pending", "active", "completed", "cancelled"]
    },
    "priority": {
      "type": "string",
      "enum": ["low", "medium", "high", "urgent"]
    }
  },
  "required": ["status", "priority"]
}
```

## Provider Differences

### Gemini

- **Full Schema Support**: Gemini supports complete JSON Schema validation
- **Strict Enforcement**: Schema violations are caught by the API
- **Format**: Uses `response_mime_type` and `response_schema` fields

**Example**:
```bash
rad step agent-id "prompt" --response-format json-schema --response-schema schema.json --engine gemini
```

### OpenAI

- **Structured Outputs**: OpenAI supports JSON schema with strict mode
- **Format**: Uses `response_format` with `json_schema` object containing `name`, `schema`, and `strict: true`
- **Default Name**: Radium uses "response_schema" as the default schema name

**Example**:
```bash
rad step agent-id "prompt" --response-format json-schema --response-schema schema.json --engine openai
```

### Key Differences

| Feature | Gemini | OpenAI |
|---------|--------|--------|
| Schema Support | ✅ Full | ✅ Full |
| Strict Mode | ✅ Always | ✅ Always (strict: true) |
| Schema Name | N/A | Required (default: "response_schema") |
| Error Handling | API-level | API-level |

## Common Errors and Solutions

### Error: "Invalid response format"

**Problem**: Invalid value for `--response-format`

**Solution**: Use one of: `text`, `json`, `json-schema`

```bash
# Wrong
rad step agent-id "prompt" --response-format xml

# Correct
rad step agent-id "prompt" --response-format json
```

### Error: "--response-schema is required when using --response-format json-schema"

**Problem**: Missing schema argument

**Solution**: Provide a schema file or inline JSON

```bash
# Wrong
rad step agent-id "prompt" --response-format json-schema

# Correct
rad step agent-id "prompt" --response-format json-schema --response-schema schema.json
```

### Error: "Failed to read schema file"

**Problem**: Schema file doesn't exist or is unreadable

**Solution**: Check file path and permissions

```bash
# Check file exists
ls -la schema.json

# Use absolute path if needed
rad step agent-id "prompt" --response-format json-schema --response-schema /full/path/to/schema.json
```

### Error: "Invalid JSON schema"

**Problem**: Schema is not valid JSON

**Solution**: Validate your JSON schema

```bash
# Validate JSON
cat schema.json | jq .

# Or use online validator
# https://jsonschema.dev/
```

### Error: "--response-schema cannot be used with --response-format json"

**Problem**: Schema argument provided with json format (not json-schema)

**Solution**: Use `json-schema` format instead

```bash
# Wrong
rad step agent-id "prompt" --response-format json --response-schema schema.json

# Correct
rad step agent-id "prompt" --response-format json-schema --response-schema schema.json
```

## Best Practices

### When to Use Schemas

- **Data Extraction**: When you need structured data from unstructured text
- **API Integration**: When responses need to match specific formats
- **Form Filling**: When populating forms or databases
- **Validation**: When you need guaranteed structure

### Schema Design Tips

1. **Start Simple**: Begin with basic object schemas, add complexity gradually
2. **Use Required Fields**: Mark essential fields as `required` for validation
3. **Leverage Enums**: Use `enum` for constrained string values
4. **Nested Structures**: Organize related data in nested objects
5. **Array Constraints**: Use `minItems` and `maxItems` for array validation

### Performance Considerations

- **Schema Size**: Large schemas may increase API latency slightly
- **Validation**: Schema validation happens server-side (no client overhead)
- **Caching**: Consider caching parsed schemas for repeated use

## Advanced Usage

### Complex Nested Schemas

For deeply nested structures:

```json
{
  "type": "object",
  "properties": {
    "user": {
      "type": "object",
      "properties": {
        "profile": {
          "type": "object",
          "properties": {
            "personal": {
              "type": "object",
              "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
              }
            }
          }
        }
      }
    }
  }
}
```

### Pattern Validation

Use regex patterns for string validation:

```json
{
  "type": "object",
  "properties": {
    "email": {
      "type": "string",
      "pattern": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
    }
  }
}
```

### Array Schemas

Define arrays with item constraints:

```json
{
  "type": "object",
  "properties": {
    "items": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "id": {"type": "string"},
          "value": {"type": "number"}
        }
      },
      "minItems": 1,
      "maxItems": 100
    }
  }
}
```

## Reference

- [JSON Schema Specification](https://json-schema.org/)
- [OpenAI Structured Outputs](https://platform.openai.com/docs/guides/structured-outputs)
- [Gemini JSON Schema](https://ai.google.dev/gemini-api/docs/json-schema)

## See Also

- [CLI Documentation](docs/cli/)
- [Model Parameters Guide](docs/features/)
- [Provider Comparison](docs/requirements/provider-agnostic-features.md)


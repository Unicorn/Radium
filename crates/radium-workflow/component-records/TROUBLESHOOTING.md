# Component Migration Troubleshooting Guide

This guide addresses common issues encountered during component schema migration and TypeScript code generation.

---

## Table of Contents

1. [Serde Serialization Issues](#serde-serialization-issues)
2. [TypeScript Generation Errors](#typescript-generation-errors)
3. [Validation Failures](#validation-failures)
4. [Handlebars Template Issues](#handlebars-template-issues)
5. [Test Failures](#test-failures)
6. [Temporal Integration Issues](#temporal-integration-issues)

---

## Serde Serialization Issues

### Field Not Appearing in JSON Output

**Symptom**: Optional field is missing from serialized JSON

**Cause**: Using `Option<T>` with `#[serde(skip_serializing_if = "Option::is_none")]`

**Solution**: This is usually intentional. If the field should always appear:
```rust
// Instead of:
#[serde(skip_serializing_if = "Option::is_none")]
pub field: Option<String>,

// Use:
#[serde(default)]
pub field: String,
```

---

### Enum Serializes as Object Instead of String

**Symptom**: `{"Type": "value"}` instead of `"value"`

**Cause**: Default serde enum serialization

**Solution**: Add `#[serde(rename_all = "lowercase")]`:
```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MyEnum {
    OptionA,
    OptionB,
}
```

**Variants**:
- `"lowercase"` → `optiona`
- `"camelCase"` → `optionA`
- `"UPPERCASE"` → `OPTIONA` (for HTTP methods)
- `"snake_case"` → `option_a`

---

### camelCase Not Working

**Symptom**: JSON field is `field_name` instead of `fieldName`

**Cause**: Missing rename directive or applied to wrong level

**Solution**: Apply to struct, not just field:
```rust
// Correct:
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyStruct {
    pub field_name: String,  // serializes as "fieldName"
}

// Incorrect (only renames this field):
pub struct MyStruct {
    #[serde(rename = "fieldName")]
    pub field_name: String,
}
```

---

### Untagged Enum Deserialization Fails

**Symptom**: `Error("data did not match any variant")`

**Cause**: Ambiguous variants or incorrect structure

**Solution**: Ensure variants are distinguishable:
```rust
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionGroup {
    // Put most specific variants first
    Expression { expression: String },
    Compound { operator: LogicalOperator, conditions: Vec<ConditionGroup> },
    Single(Condition),
}
```

**Tip**: Order variants from most specific to least specific.

---

## TypeScript Generation Errors

### TSC: "Module has no exports"

**Symptom**: `error TS2306: File 'activities.ts' is not a module`

**Cause**: Generated file has no exports when no activities exist

**Solution**: Template must include fallback export:
```handlebars
{{#if activities}}
{{#each activities}}
export async function {{function_name}}() { ... }
{{/each}}
{{else}}
// No activities defined for this workflow
export {};
{{/if}}
```

---

### TSC: "Conversion of undefined to type X"

**Symptom**: `error TS2352: Conversion of type 'undefined' to type 'WorkflowResult'`

**Cause**: Type assertion on undefined: `let result = undefined as ResultType;`

**Solution**: Use union type with non-null assertion:
```typescript
// Before (fails):
let result: WorkflowResult = undefined as WorkflowResult;
return result;

// After (works):
let result: WorkflowResult | undefined = undefined;
return result!;
```

---

### ESLint: "Unexpected any" (no-explicit-any)

**Symptom**: ESLint error on `any` type usage

**Cause**: Using `any` in generated code

**Solution**: Use `unknown` instead:
```rust
// In Rust:
pub body: Option<serde_json::Value>,

// Maps to TypeScript:
body?: unknown
// NOT: body?: any
```

---

### TSC: "Cannot find module '@temporalio/workflow'"

**Symptom**: Import resolution failure

**Cause**: Running tsc without node_modules

**Solution**: Test must install dependencies:
```rust
let npm_install = std::process::Command::new("npm")
    .args(["install", "--silent"])
    .current_dir(&temp_dir)
    .output()?;
```

Required `package.json`:
```json
{
  "dependencies": {
    "@temporalio/workflow": "^1.11.0"
  }
}
```

---

## Validation Failures

### Validation Not Running

**Symptom**: Invalid data passes without errors

**Cause**: Missing `Validate` derive or not calling validate()

**Solution**: Ensure both derive and call:
```rust
#[derive(Validate)]
pub struct Input {
    #[validate(length(min = 1))]
    pub name: String,
}

// Must call validate():
let input = Input { name: "".to_string() };
assert!(input.validate().is_err());  // Catches the error
```

---

### Custom Validation Not Triggered

**Symptom**: `#[validate]` attribute not doing anything

**Cause**: Using wrong validation approach

**Solution**: Use correct validator syntax:
```rust
// String length:
#[validate(length(min = 1, max = 100))]
pub name: String,

// URL validation:
#[validate(url)]
pub endpoint: String,

// Email validation:
#[validate(email)]
pub contact: String,

// Range validation:
#[validate(range(min = 1, max = 1000))]
pub count: u32,

// Custom validation (separate method):
impl Input {
    pub fn validate_business_rules(&self) -> Result<(), Vec<String>> {
        // Custom logic here
    }
}
```

---

## Handlebars Template Issues

### Template Variable Not Rendering

**Symptom**: Output shows `{{variable}}` literally or empty string

**Cause**: Variable not in context or wrong path

**Solution**: Check context structure:
```rust
// In code:
let context = json!({
    "workflow_name": "MyWorkflow",
    "activities": activities_vec,
});

// In template:
// Correct:
{{workflow_name}}
{{#each activities}}{{this.name}}{{/each}}

// Incorrect:
{{name}}  // Missing context path
```

---

### Triple Braces Needed for Raw HTML/Code

**Symptom**: Generated code has escaped characters: `&lt;` instead of `<`

**Cause**: Using double braces escapes HTML

**Solution**: Use triple braces for raw output:
```handlebars
// Escapes (wrong for code generation):
input: {{input_type}}   → input: Record&lt;string, string&gt;

// Raw (correct):
input: {{{input_type}}} → input: Record<string, string>
```

---

### Conditional Block Not Working

**Symptom**: Content not appearing despite condition being true

**Cause**: Checking wrong type or falsy value

**Solution**: Use explicit boolean checks:
```handlebars
// Checking if array has items:
{{#if activities}}  // Works for non-empty arrays

// Checking boolean:
{{#if is_long_running}}  // Works for true/false

// Checking optional:
{{#if has_retry}}  // Must be explicit boolean in context
```

---

## Test Failures

### Migration Record Quality Test Fails

**Symptom**: `Component 'X' is missing required section: Y`

**Cause**: Migration record YAML incomplete

**Solution**: Ensure all required sections:
```yaml
component:
  name: component_name
  # ...
migration:
  # ...
schemaDecisions:
  - field: ...
    rationale: ...   # Required!
inputSchema:
  fields:
    # ...
outputSchema:
  fields:
    # ...
testCases:
  - name: test_something
    # ...
lessonsLearned:
  whatWorkedWell:
    - Something that worked
```

---

### Test Passes Locally, Fails in CI

**Symptom**: Tests pass with `cargo test` locally but fail in CI

**Common Causes**:
1. Node.js not installed in CI
2. Different file paths
3. Missing test fixtures

**Solutions**:
```rust
// 1. Skip tests requiring external tools:
#[test]
#[ignore = "Requires Node.js installed"]
fn test_typescript_compilation() { ... }

// 2. Use CARGO_MANIFEST_DIR for paths:
let records_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("component-records");

// 3. Create fixtures in test:
std::fs::write(&fixture_path, fixture_content)?;
```

---

### Roundtrip Serialization Fails

**Symptom**: Deserialize(Serialize(x)) != x

**Cause**: Default values or optional field handling

**Solution**: Compare semantically, not structurally:
```rust
#[test]
fn test_roundtrip() {
    let original = Input::new("test");
    let json = serde_json::to_string(&original).unwrap();
    let restored: Input = serde_json::from_str(&json).unwrap();

    // Compare relevant fields, not entire struct
    assert_eq!(original.name, restored.name);
    assert_eq!(original.important_field, restored.important_field);
}
```

---

## Temporal Integration Issues

### Activity Timeout Too Short

**Symptom**: Activity fails with timeout before completing

**Cause**: Default timeout insufficient for operation

**Solution**: Configure appropriate timeout in template:
```handlebars
const {{var_name}} = proxyActivities<typeof activities>({
  startToCloseTimeout: '{{timeout}}',  // e.g., '5m' for 5 minutes
  {{#if has_retry}}
  retry: {
    maximumAttempts: {{retry_policy.max_attempts}},
  },
  {{/if}}
});
```

---

### Signal Handler Not Receiving Signals

**Symptom**: Signals sent but handler never called

**Cause**: Handler registered after signal sent

**Solution**: Register handlers at workflow start:
```typescript
export async function myWorkflow(input: Input): Promise<Output> {
  // Register handlers FIRST
  setHandler(mySignal, (data) => {
    // Handle signal
  });

  // Then do work that might receive signals
  await someActivity();
}
```

---

### continueAsNew Not Preserving State

**Symptom**: State lost after continue-as-new

**Cause**: Not passing state to new execution

**Solution**: Include state in input:
```typescript
interface WorkflowInput {
  originalInput: OriginalInput;
  iterationState?: IterationState;  // Preserved across continue-as-new
}

if (shouldContinueAsNew) {
  await continueAsNew<typeof workflow>({
    ...input,
    iterationState: currentState,
  });
}
```

---

## Quick Reference: Common Error Messages

| Error | Likely Cause | Quick Fix |
|-------|--------------|-----------|
| `not a module` | No exports in file | Add `export {}` fallback |
| `undefined to type X` | Type assertion on undefined | Use `X \| undefined` |
| `camelCase` missing | Serde rename not applied | Add `#[serde(rename_all = "camelCase")]` |
| `any` in output | Using serde_json::Value wrong | Map to `unknown` not `any` |
| `missing section` | YAML incomplete | Check required sections list |
| `validate()` not working | Not calling the method | Call `.validate()` explicitly |

---

## Getting Help

1. **Check migration records**: Review similar components' records for patterns
2. **Run quality tests**: `cargo test migration_record_quality`
3. **Check TypeScript output**: Generate code and run `tsc --noEmit --strict`
4. **Consult patterns doc**: See `MIGRATION_PATTERNS.md` for established patterns

---

## Reporting New Issues

When encountering a new issue:

1. Document the symptom clearly
2. Identify the root cause
3. Record the solution
4. Add to the relevant component's `lessonsLearned` section
5. Consider adding to this troubleshooting guide

# Component Migration Patterns

## Overview

This document captures the patterns, decisions, and lessons learned during the Phase 6 component migration from TypeScript to Rust schemas.

---

## Core Patterns

### 1. Input/Output Schema Pattern

Every component follows a consistent input/output pattern:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ComponentInput {
    // Required fields first
    #[validate(length(min = 1))]
    pub name: String,

    // Optional fields with defaults
    #[serde(default)]
    pub enabled: bool,

    // Optional fields without defaults
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<Config>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentOutput {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub duration_ms: u64,
}
```

**Key Decisions**:
- Use `#[serde(rename_all = "camelCase")]` for TypeScript compatibility
- Use `#[serde(default)]` for optional fields with sensible defaults
- Use `#[serde(skip_serializing_if = "Option::is_none")]` to omit null fields
- Implement `Validate` trait for input validation

---

### 2. Enum Serialization Pattern

Enums are serialized as lowercase strings for TypeScript:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ComponentType {
    #[default]
    TypeA,
    TypeB,
    TypeC,
}
```

**Exception**: HTTP methods use UPPERCASE:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
    Put,
}
```

---

### 3. Builder Pattern

Complex inputs use builder methods for ergonomic construction:

```rust
impl HttpRequestInput {
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Get,
            ..Default::default()
        }
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = auth;
        self
    }
}
```

**Benefits**:
- Fluent API for test construction
- Type-safe configuration
- Sensible defaults for unspecified fields

---

### 4. TypeScript Generation Pattern

Components implement `to_typescript()` for code generation:

```rust
impl Condition {
    pub fn to_typescript(&self) -> String {
        let left = format!("state.variables.{}", self.left);

        match self.operator {
            ComparisonOperator::Equals => {
                format!("{} === {}", left, self.right_to_ts())
            }
            // ... other operators
        }
    }
}
```

**Rules**:
- Generate valid TypeScript expressions
- Use strict equality (`===`, `!==`)
- Never generate `any` types (use `unknown`)
- Use optional chaining where appropriate (`?.`)

---

### 5. Validation Pattern

Use the `validator` crate for input validation:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Input {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,

    #[validate(url(message = "Invalid URL"))]
    pub url: String,

    #[validate(range(min = 1, max = 100))]
    pub count: u32,

    #[validate(email)]
    pub email: String,
}
```

**Custom Validation**:
```rust
impl ComponentInput {
    pub fn validate_config(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        match self.component_type {
            ComponentType::TypeA if self.config_a.is_none() => {
                errors.push("TypeA requires config_a".to_string());
            }
            // ... other validations
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

---

### 6. Default Value Pattern

Provide sensible defaults via functions:

```rust
fn default_timeout() -> u64 { 30000 }
fn default_max_attempts() -> u32 { 3 }
fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,

    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,

    #[serde(default = "default_true")]
    pub enabled: bool,
}
```

---

## Migration Lessons

### What Worked Well

1. **Parallel Module Structure**
   - Each component in its own file
   - Clean separation of concerns
   - Easy to test in isolation

2. **Builder Pattern**
   - Reduces boilerplate in tests
   - Provides type-safe construction
   - Self-documenting API

3. **Serde Integration**
   - Seamless JSON serialization
   - TypeScript-compatible output
   - Flexible customization

4. **Validator Crate**
   - Declarative validation rules
   - Clear error messages
   - Easy to extend

### Challenges Encountered

1. **UPPERCASE Serialization for HTTP Methods**
   - Solution: Use `#[serde(rename_all = "UPPERCASE")]`
   - Time spent: 15 minutes

2. **Complex Nested Enums (ModelConfig)**
   - Solution: Use `#[serde(untagged)]` for polymorphic types
   - Time spent: 2 hours

3. **Condition Expression Generation**
   - Solution: Custom `to_typescript()` methods per operator
   - Time spent: 1 hour

4. **Optional vs Default Fields**
   - Distinction: `Option<T>` for truly optional, `#[serde(default)]` for has-default
   - Clarity: Document which pattern to use when

### Recommendations for Future Components

1. **Start with the Input Schema**
   - Define required fields first
   - Add optional fields with clear defaults
   - Validate early and often

2. **Test Serialization Immediately**
   - Write roundtrip tests
   - Verify camelCase in JSON
   - Check enum value serialization

3. **Design for TypeScript First**
   - Consider how types will look in TS
   - Avoid patterns that don't translate well
   - Test generated code with tsc

4. **Document Schema Decisions**
   - Record rationale in migration records
   - Note alternatives considered
   - Capture lessons learned

---

## Type Mapping Reference

| Rust Type | TypeScript Type | Notes |
|-----------|-----------------|-------|
| `String` | `string` | |
| `u64` | `number` | JavaScript numbers |
| `i64` | `number` | Be careful with precision |
| `bool` | `boolean` | |
| `Option<T>` | `T \| undefined` | With skip_serializing_if |
| `Vec<T>` | `T[]` | |
| `HashMap<K, V>` | `Record<K, V>` | |
| `serde_json::Value` | `unknown` | NOT `any` |
| `DateTime<Utc>` | `string` (ISO 8601) | |

---

## Migration Record Template

Each component produces a migration record following this structure:

```yaml
component:
  name: component_name
  category: activities|agents|advanced
  version: 1.0.0
  description: Brief description
  temporalType: activity|workflow|signal|timer

migration:
  migratedBy: radium-workflow-compiler
  migrationDate: ISO timestamp
  durationHours: 2.0
  difficulty: low|medium|high
  breakingChanges: false
  filesCreated: []
  filesModified: []

schemaDecisions:
  - field: field_name
    decision: What was decided
    rationale: Why this decision
    alternativesConsidered: []

inputSchema:
  rustStruct: InputStruct
  typescriptInterface: InputInterface
  fields:
    - name: field
      rustType: String
      typescriptType: string
      required: true
      description: Field description

outputSchema:
  # Similar to inputSchema

testCases:
  - name: test_name
    category: unit|integration
    input: Test input
    expectedOutput: Expected result
    passed: true

lessonsLearned:
  whatWorkedWell: []
  challenges: []
  recommendations: []
```

---

## Testing Patterns

### Unit Test Pattern
```rust
#[test]
fn test_component_creation() {
    let input = ComponentInput::new("test");
    assert_eq!(input.name, "test");
    assert!(input.validate().is_ok());
}
```

### Serialization Test Pattern
```rust
#[test]
fn test_serialization_roundtrip() {
    let original = ComponentInput::new("test");
    let json = serde_json::to_string(&original).unwrap();
    let restored: ComponentInput = serde_json::from_str(&json).unwrap();
    assert_eq!(original.name, restored.name);
}
```

### TypeScript Generation Test Pattern
```rust
#[test]
fn test_typescript_generation() {
    let condition = Condition::equals("x", json!(10));
    let ts = condition.to_typescript();
    assert!(ts.contains("==="));
    assert!(!ts.contains(": any"));
}
```

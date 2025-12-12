# Phase 6: Component Migration

**Status**: Not Started
**Duration**: 4-6 weeks
**Prerequisites**: Phase 5 (Variables & State)
**Blocks**: Phase 7

## Overview

Migrate all existing TypeScript component patterns to Rust schemas with generated TypeScript. This is the core migration phase where every component type is systematically converted, validated, and documented. Each migration produces a detailed component record that will feed Phase 9's Component Builder Agent.

## Goals

1. Migrate all component types to Rust schemas
2. Generate type-safe TypeScript for each component
3. Create detailed migration records for each component
4. Verify 100% feature parity
5. Zero runtime behavior changes

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Component Migration Pipeline                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌────────────┐    ┌────────────┐    ┌────────────┐    ┌────────────┐  │
│  │ TypeScript │    │   Rust     │    │ Generated  │    │ Migration  │  │
│  │ Component  │───►│  Schema    │───►│ TypeScript │───►│  Record    │  │
│  │ Pattern    │    │ Definition │    │    Code    │    │   YAML     │  │
│  └────────────┘    └────────────┘    └────────────┘    └────────────┘  │
│        │                 │                  │                 │         │
│        └─────────────────┼──────────────────┼─────────────────┘         │
│                          │                  │                           │
│                          ▼                  ▼                           │
│                   ┌─────────────────────────────────┐                   │
│                   │      Verification Pipeline       │                   │
│                   │  - tsc --strict                  │                   │
│                   │  - ESLint                        │                   │
│                   │  - Integration tests             │                   │
│                   │  - Behavior comparison           │                   │
│                   └─────────────────────────────────┘                   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## Component Inventory

| Category | Component | Priority | Complexity | Dependencies |
|----------|-----------|----------|------------|--------------|
| **Control Flow** | trigger | 1 | Low | None |
| **Control Flow** | start | 1 | Low | None |
| **Control Flow** | stop | 1 | Low | None |
| **Control Flow** | conditional | 2 | Medium | Variables |
| **Control Flow** | loop | 2 | Medium | Variables, State |
| **Activities** | activity | 1 | Medium | Input/Output |
| **Activities** | kong-logging | 1 | Medium | Kong API |
| **Activities** | http-request | 2 | Medium | External |
| **Activities** | database-query | 2 | Medium | Supabase |
| **Agents** | agent | 3 | High | AI Models |
| **Advanced** | child-workflow | 3 | High | Workflow refs |
| **Advanced** | signal | 3 | High | Temporal |
| **Advanced** | timer | 2 | Low | Temporal |
| **Advanced** | parallel | 3 | High | State merge |

---

## Tasks

### 5.1 Migration Framework Setup

**Description**: Create the infrastructure for systematic component migration

**Subtasks**:
- [ ] 5.1.1 Create ComponentMigration trait/interface
- [ ] 5.1.2 Create migration CLI tool for single components
- [ ] 5.1.3 Create batch migration runner
- [ ] 5.1.4 Set up component record template validation
- [ ] 5.1.5 Create comparison test framework
- [ ] 5.1.6 Set up TypeScript output verification
- [ ] 5.1.7 Create migration progress tracking
- [ ] 5.1.8 Write framework documentation

**Files to Create**:
```rust
// src/migration/mod.rs
mod framework;
mod cli;
mod runner;
mod comparison;
mod record;

pub use framework::*;
pub use cli::*;
pub use runner::*;
pub use comparison::*;
pub use record::*;
```

```rust
// src/migration/framework.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Trait for component migration implementations
#[async_trait]
pub trait ComponentMigration {
    /// Component type identifier
    fn component_type(&self) -> &str;

    /// Original TypeScript file path
    fn typescript_source(&self) -> PathBuf;

    /// Analyze the existing TypeScript component
    async fn analyze(&self) -> Result<ComponentAnalysis, MigrationError>;

    /// Generate Rust schema from analysis
    fn generate_rust_schema(&self, analysis: &ComponentAnalysis) -> Result<String, MigrationError>;

    /// Generate TypeScript from Rust schema
    fn generate_typescript(&self) -> Result<String, MigrationError>;

    /// Verify generated code matches original behavior
    async fn verify(&self) -> Result<VerificationResult, MigrationError>;

    /// Create migration record
    fn create_record(&self) -> MigrationRecord;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentAnalysis {
    pub component_type: String,
    pub typescript_source: PathBuf,
    pub input_schema: SchemaAnalysis,
    pub output_schema: SchemaAnalysis,
    pub config_schema: SchemaAnalysis,
    pub dependencies: Vec<String>,
    pub external_calls: Vec<ExternalCall>,
    pub error_handling: Vec<ErrorPattern>,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaAnalysis {
    pub fields: Vec<FieldAnalysis>,
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
    pub default_values: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAnalysis {
    pub name: String,
    pub typescript_type: String,
    pub rust_type: String,
    pub is_optional: bool,
    pub default_value: Option<String>,
    pub validation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalCall {
    pub target: String,
    pub method: String,
    pub input_mapping: String,
    pub output_mapping: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub error_type: String,
    pub handling: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub field: String,
    pub rule: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub passed: bool,
    pub typescript_compiles: bool,
    pub eslint_passes: bool,
    pub behavior_matches: bool,
    pub test_results: Vec<TestResult>,
    pub differences: Vec<BehaviorDifference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub original_output: String,
    pub generated_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorDifference {
    pub scenario: String,
    pub original_behavior: String,
    pub generated_behavior: String,
    pub severity: DifferenceSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifferenceSeverity {
    Breaking,
    Warning,
    Cosmetic,
}

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Schema generation failed: {0}")]
    SchemaGenerationFailed(String),

    #[error("Code generation failed: {0}")]
    CodeGenerationFailed(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

```rust
// src/migration/record.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Migration record that feeds the Component Builder Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    // Component Identification
    pub component: ComponentInfo,

    // Migration Metadata
    pub migration: MigrationMetadata,

    // Discovery Phase
    pub discovery: DiscoveryInfo,

    // Schema Decisions
    pub schema_decisions: Vec<SchemaDecision>,

    // Input Schema
    pub input_schema: SchemaDefinition,

    // Output Schema
    pub output_schema: SchemaDefinition,

    // Validation Rules
    pub validation_rules: Vec<ValidationRuleRecord>,

    // Connection Rules
    pub connections: ConnectionRules,

    // Rust Schema
    pub rust_schema: RustSchemaRecord,

    // TypeScript Template
    pub typescript_template: TypeScriptTemplateRecord,

    // Test Cases
    pub test_cases: Vec<TestCaseRecord>,

    // Lessons Learned
    pub lessons_learned: LessonsLearned,

    // Related Components
    pub related_components: Vec<RelatedComponent>,

    // Future Improvements
    pub future_improvements: Vec<FutureImprovement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub name: String,
    pub category: String,
    pub version: String,
    pub description: String,
    pub temporal_type: String, // activity, workflow, signal, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationMetadata {
    pub migrated_by: String,
    pub migration_date: DateTime<Utc>,
    pub duration_hours: f32,
    pub difficulty: Difficulty,
    pub breaking_changes: bool,
    pub files_created: Vec<String>,
    pub files_modified: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Difficulty {
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryInfo {
    pub original_typescript_file: String,
    pub lines_of_code: usize,
    pub existing_tests: Vec<String>,
    pub usage_locations: Vec<String>,
    pub dependencies: Vec<DependencyInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub name: String,
    pub dependency_type: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDecision {
    pub field: String,
    pub decision: String,
    pub rationale: String,
    pub alternatives_considered: Vec<Alternative>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub approach: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub why_rejected: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDefinition {
    pub rust_struct: String,
    pub typescript_interface: String,
    pub fields: Vec<FieldDefinition>,
    pub validation: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    pub rust_type: String,
    pub typescript_type: String,
    pub required: bool,
    pub default: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRuleRecord {
    pub rule: String,
    pub implementation: String,
    pub error_message: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRules {
    pub allowed_sources: Vec<String>,
    pub allowed_targets: Vec<String>,
    pub connection_validation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustSchemaRecord {
    pub file_path: String,
    pub structs: Vec<String>,
    pub enums: Vec<String>,
    pub derives: Vec<String>,
    pub validation_implementation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeScriptTemplateRecord {
    pub template_path: String,
    pub generated_code_example: String,
    pub key_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseRecord {
    pub name: String,
    pub category: TestCategory,
    pub input: String,
    pub expected_output: String,
    pub actual_output: Option<String>,
    pub passed: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestCategory {
    Unit,
    Integration,
    Compilation,
    BehaviorComparison,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonsLearned {
    pub what_worked_well: Vec<String>,
    pub challenges: Vec<ChallengeRecord>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeRecord {
    pub challenge: String,
    pub solution: String,
    pub time_spent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedComponent {
    pub component: String,
    pub relationship: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FutureImprovement {
    pub improvement: String,
    pub priority: String,
    pub effort: String,
}

impl MigrationRecord {
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        let yaml = self.to_yaml().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?;
        std::fs::write(path, yaml)
    }
}
```

**Acceptance Criteria**:
- [ ] Migration framework compiles
- [ ] CLI tool works for single components
- [ ] Batch runner processes multiple components
- [ ] Records are generated correctly

---

### 5.2 Control Flow Components

**Description**: Migrate control flow components (trigger, start, stop, conditional, loop)

**Subtasks**:
- [ ] 5.2.1 Migrate `trigger` component
- [ ] 5.2.2 Migrate `start` component
- [ ] 5.2.3 Migrate `stop` component
- [ ] 5.2.4 Migrate `conditional` component
- [ ] 5.2.5 Migrate `loop` component
- [ ] 5.2.6 Create migration records for each
- [ ] 5.2.7 Verify all control flow tests pass
- [ ] 5.2.8 Document control flow patterns

**Migration Order**:

#### 5.2.1 Trigger Component

```rust
// src/schemas/components/trigger.rs
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TriggerType {
    Manual,
    Schedule,
    Webhook,
    Event,
    Signal,
}

impl Default for TriggerType {
    fn default() -> Self {
        Self::Manual
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScheduleConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval_seconds: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebhookConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    #[serde(default)]
    pub methods: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TriggerInput {
    #[serde(default)]
    pub trigger_type: TriggerType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<ScheduleConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<WebhookConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_name: Option<String>,

    /// Payload passed to workflow
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerOutput {
    pub triggered: bool,
    pub trigger_id: String,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub payload: serde_json::Value,
}

impl TriggerInput {
    pub fn validate_config(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        match self.trigger_type {
            TriggerType::Schedule if self.schedule.is_none() => {
                errors.push("Schedule trigger requires schedule configuration".to_string());
            }
            TriggerType::Webhook if self.webhook.is_none() => {
                errors.push("Webhook trigger requires webhook configuration".to_string());
            }
            TriggerType::Event if self.event_type.is_none() => {
                errors.push("Event trigger requires event_type".to_string());
            }
            TriggerType::Signal if self.signal_name.is_none() => {
                errors.push("Signal trigger requires signal_name".to_string());
            }
            _ => {}
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

#### 5.2.4 Conditional Component

```rust
// src/schemas/components/conditional.rs
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::expressions::Expression;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    Contains,
    StartsWith,
    EndsWith,
    Matches, // Regex
    IsNull,
    IsNotNull,
    IsEmpty,
    IsNotEmpty,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Condition {
    /// Left side of comparison (variable reference)
    #[validate(length(min = 1, message = "Left operand is required"))]
    pub left: String,

    /// Comparison operator
    pub operator: ComparisonOperator,

    /// Right side of comparison (value or variable reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionGroup {
    Single(Condition),
    Compound {
        operator: LogicalOperator,
        conditions: Vec<ConditionGroup>,
    },
    Expression(String), // Raw expression string
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ConditionalInput {
    /// The condition(s) to evaluate
    pub condition: ConditionGroup,

    /// Label for the 'true' branch
    #[serde(default = "default_true_label")]
    pub true_label: String,

    /// Label for the 'false' branch
    #[serde(default = "default_false_label")]
    pub false_label: String,
}

fn default_true_label() -> String {
    "Yes".to_string()
}

fn default_false_label() -> String {
    "No".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalOutput {
    pub result: bool,
    pub branch: String,
    pub evaluated_expression: String,
}

impl Condition {
    pub fn to_typescript(&self) -> String {
        let left = format!("state.variables.{}", self.left);

        match self.operator {
            ComparisonOperator::Equals => {
                format!("{} === {}", left, self.right_to_ts())
            }
            ComparisonOperator::NotEquals => {
                format!("{} !== {}", left, self.right_to_ts())
            }
            ComparisonOperator::GreaterThan => {
                format!("{} > {}", left, self.right_to_ts())
            }
            ComparisonOperator::LessThan => {
                format!("{} < {}", left, self.right_to_ts())
            }
            ComparisonOperator::GreaterOrEqual => {
                format!("{} >= {}", left, self.right_to_ts())
            }
            ComparisonOperator::LessOrEqual => {
                format!("{} <= {}", left, self.right_to_ts())
            }
            ComparisonOperator::Contains => {
                format!("{}.includes({})", left, self.right_to_ts())
            }
            ComparisonOperator::StartsWith => {
                format!("{}.startsWith({})", left, self.right_to_ts())
            }
            ComparisonOperator::EndsWith => {
                format!("{}.endsWith({})", left, self.right_to_ts())
            }
            ComparisonOperator::Matches => {
                format!("new RegExp({}).test({})", self.right_to_ts(), left)
            }
            ComparisonOperator::IsNull => {
                format!("{} === null", left)
            }
            ComparisonOperator::IsNotNull => {
                format!("{} !== null", left)
            }
            ComparisonOperator::IsEmpty => {
                format!("({} === '' || {}.length === 0)", left, left)
            }
            ComparisonOperator::IsNotEmpty => {
                format!("({} !== '' && {}.length > 0)", left, left)
            }
        }
    }

    fn right_to_ts(&self) -> String {
        match &self.right {
            Some(v) => match v {
                serde_json::Value::String(s) => format!("'{}'", s),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => "null".to_string(),
                _ => v.to_string(),
            },
            None => "undefined".to_string(),
        }
    }
}

impl ConditionGroup {
    pub fn to_typescript(&self) -> String {
        match self {
            ConditionGroup::Single(c) => c.to_typescript(),
            ConditionGroup::Compound { operator, conditions } => {
                let op = match operator {
                    LogicalOperator::And => " && ",
                    LogicalOperator::Or => " || ",
                };
                let parts: Vec<String> = conditions.iter()
                    .map(|c| format!("({})", c.to_typescript()))
                    .collect();
                parts.join(op)
            }
            ConditionGroup::Expression(expr) => expr.clone(),
        }
    }
}
```

#### 5.2.5 Loop Component

```rust
// src/schemas/components/loop.rs
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LoopType {
    ForEach,    // Iterate over array
    While,      // Condition-based
    DoWhile,    // At least once
    Count,      // Fixed iterations
    Batch,      // Process in batches
}

impl Default for LoopType {
    fn default() -> Self {
        Self::ForEach
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatchConfig {
    pub batch_size: usize,

    #[serde(default)]
    pub parallel: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_between_batches_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoopInput {
    pub loop_type: LoopType,

    /// Array to iterate (for ForEach/Batch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<String>, // Variable reference

    /// Condition expression (for While/DoWhile)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,

    /// Number of iterations (for Count)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,

    /// Current item variable name
    #[serde(default = "default_item_var")]
    pub item_variable: String,

    /// Current index variable name
    #[serde(default = "default_index_var")]
    pub index_variable: String,

    /// Batch configuration (for Batch type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_config: Option<BatchConfig>,

    /// Maximum iterations (safety limit)
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u64,

    /// Continue-as-new threshold
    #[serde(default = "default_continue_threshold")]
    pub continue_as_new_threshold: u64,
}

fn default_item_var() -> String {
    "item".to_string()
}

fn default_index_var() -> String {
    "index".to_string()
}

fn default_max_iterations() -> u64 {
    10_000
}

fn default_continue_threshold() -> u64 {
    1_000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopOutput {
    pub completed: bool,
    pub iterations_completed: u64,
    pub total_items: Option<u64>,
    pub results: Vec<serde_json::Value>,
    pub continued_as_new: bool,
}

impl LoopInput {
    pub fn validate_config(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        match self.loop_type {
            LoopType::ForEach | LoopType::Batch if self.items.is_none() => {
                errors.push("ForEach/Batch loop requires items array reference".to_string());
            }
            LoopType::While | LoopType::DoWhile if self.condition.is_none() => {
                errors.push("While/DoWhile loop requires condition expression".to_string());
            }
            LoopType::Count if self.count.is_none() => {
                errors.push("Count loop requires count value".to_string());
            }
            LoopType::Batch if self.batch_config.is_none() => {
                errors.push("Batch loop requires batch configuration".to_string());
            }
            _ => {}
        }

        if self.max_iterations == 0 {
            errors.push("max_iterations must be greater than 0".to_string());
        }

        if self.continue_as_new_threshold > self.max_iterations {
            errors.push("continue_as_new_threshold cannot exceed max_iterations".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

**Acceptance Criteria**:
- [ ] All 5 control flow components migrated
- [ ] Migration records created for each
- [ ] TypeScript compiles without errors
- [ ] Original tests still pass

---

### 5.3 Activity Components

**Description**: Migrate activity components (activity, kong-logging, http-request, database-query)

**Subtasks**:
- [ ] 5.3.1 Migrate generic `activity` component
- [ ] 5.3.2 Migrate `kong-logging` component (use as reference)
- [ ] 5.3.3 Migrate `http-request` component
- [ ] 5.3.4 Migrate `database-query` component
- [ ] 5.3.5 Create migration records for each
- [ ] 5.3.6 Verify activity invocation works
- [ ] 5.3.7 Test error handling patterns
- [ ] 5.3.8 Document activity patterns

**Activity Component Schema**:
```rust
// src/schemas/components/activity.rs
use serde::{Deserialize, Serialize};
use validator::Validate;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum RetryPolicy {
    NoRetry,
    Linear,
    Exponential,
    Custom,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::Exponential
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetryConfig {
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,

    #[serde(default = "default_initial_interval")]
    pub initial_interval_ms: u64,

    #[serde(default = "default_max_interval")]
    pub max_interval_ms: u64,

    #[serde(default = "default_backoff_coefficient")]
    pub backoff_coefficient: f64,

    #[serde(default)]
    pub non_retryable_errors: Vec<String>,
}

fn default_max_attempts() -> u32 { 3 }
fn default_initial_interval() -> u64 { 1000 }
fn default_max_interval() -> u64 { 60000 }
fn default_backoff_coefficient() -> f64 { 2.0 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimeoutConfig {
    /// Start to close timeout (how long the activity can run)
    #[serde(default = "default_start_to_close")]
    pub start_to_close_ms: u64,

    /// Schedule to start timeout (how long to wait for worker)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_to_start_ms: Option<u64>,

    /// Schedule to close timeout (total time including queue)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_to_close_ms: Option<u64>,

    /// Heartbeat timeout (for long-running activities)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_ms: Option<u64>,
}

fn default_start_to_close() -> u64 { 300000 } // 5 minutes

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ActivityInput {
    /// Activity name/identifier
    #[validate(length(min = 1, message = "Activity name is required"))]
    pub activity_name: String,

    /// Task queue for the activity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_queue: Option<String>,

    /// Input parameters
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,

    /// Retry configuration
    #[serde(default)]
    pub retry: RetryConfig,

    /// Timeout configuration
    #[serde(default)]
    pub timeouts: TimeoutConfig,

    /// Whether to wait for result
    #[serde(default = "default_true")]
    pub await_result: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityOutput {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<ActivityError>,
    pub duration_ms: u64,
    pub attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}
```

**HTTP Request Component**:
```rust
// src/schemas/components/http_request.rs
use serde::{Deserialize, Serialize};
use validator::Validate;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl Default for HttpMethod {
    fn default() -> Self {
        Self::Get
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BodyType {
    Json,
    FormData,
    FormUrlencoded,
    Text,
    Binary,
    None,
}

impl Default for BodyType {
    fn default() -> Self {
        Self::Json
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    None,
    Basic,
    Bearer,
    ApiKey,
    OAuth2,
}

impl Default for AuthType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    pub auth_type: AuthType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_header: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct HttpRequestInput {
    #[validate(url(message = "Invalid URL"))]
    pub url: String,

    #[serde(default)]
    pub method: HttpMethod,

    #[serde(default)]
    pub headers: HashMap<String, String>,

    #[serde(default)]
    pub query_params: HashMap<String, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,

    #[serde(default)]
    pub body_type: BodyType,

    #[serde(default)]
    pub auth: AuthConfig,

    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,

    #[serde(default)]
    pub follow_redirects: bool,

    #[serde(default)]
    pub validate_ssl: bool,

    /// Expected status codes (empty means any 2xx)
    #[serde(default)]
    pub expected_status: Vec<u16>,
}

fn default_timeout() -> u64 { 30000 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequestOutput {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
    pub duration_ms: u64,
    pub success: bool,
}
```

**Acceptance Criteria**:
- [ ] All 4 activity components migrated
- [ ] Retry and timeout configs work
- [ ] HTTP request handles all methods/auth types
- [ ] Database query integrates with Supabase

---

### 5.4 Agent Components

**Description**: Migrate AI agent components

**Subtasks**:
- [ ] 5.4.1 Analyze existing agent component
- [ ] 5.4.2 Create AgentInput schema with model config
- [ ] 5.4.3 Create AgentOutput schema with response types
- [ ] 5.4.4 Handle streaming vs non-streaming
- [ ] 5.4.5 Support multiple AI providers
- [ ] 5.4.6 Create migration record
- [ ] 5.4.7 Test with Claude/GPT models
- [ ] 5.4.8 Document agent patterns

**Agent Component Schema**:
```rust
// src/schemas/components/agent.rs
use serde::{Deserialize, Serialize};
use validator::Validate;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AIProvider {
    Anthropic,
    OpenAI,
    Google,
    Azure,
    Bedrock,
    Custom,
}

impl Default for AIProvider {
    fn default() -> Self {
        Self::Anthropic
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnthropicModel {
    #[serde(rename = "claude-3-5-sonnet-20241022")]
    Claude35Sonnet,
    #[serde(rename = "claude-3-opus-20240229")]
    Claude3Opus,
    #[serde(rename = "claude-3-haiku-20240307")]
    Claude3Haiku,
}

impl Default for AnthropicModel {
    fn default() -> Self {
        Self::Claude35Sonnet
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModelConfig {
    Anthropic {
        model: AnthropicModel,
        #[serde(default = "default_max_tokens")]
        max_tokens: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        temperature: Option<f32>,
    },
    OpenAI {
        model: String,
        #[serde(default = "default_max_tokens")]
        max_tokens: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        temperature: Option<f32>,
    },
    Custom {
        endpoint: String,
        model: String,
        config: HashMap<String, serde_json::Value>,
    },
}

fn default_max_tokens() -> u32 { 4096 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AgentInput {
    pub provider: AIProvider,

    pub model_config: ModelConfig,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,

    #[validate(length(min = 1, message = "At least one message required"))]
    pub messages: Vec<Message>,

    #[serde(default)]
    pub tools: Vec<Tool>,

    #[serde(default)]
    pub stream: bool,

    #[serde(default = "default_agent_timeout")]
    pub timeout_ms: u64,

    /// Variable to store response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_variable: Option<String>,
}

fn default_agent_timeout() -> u64 { 120000 } // 2 minutes

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub response: String,
    pub model: String,
    pub provider: String,
    pub usage: TokenUsage,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: FinishReason,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
    Error,
}
```

**Acceptance Criteria**:
- [ ] Agent component handles multiple providers
- [ ] Model configuration is type-safe
- [ ] Tool calling works
- [ ] Streaming option available

---

### 5.5 Advanced Components

**Description**: Migrate advanced components (child-workflow, signal, timer, parallel)

**Subtasks**:
- [ ] 5.5.1 Migrate `child-workflow` component
- [ ] 5.5.2 Migrate `signal` component
- [ ] 5.5.3 Migrate `timer` component
- [ ] 5.5.4 Migrate `parallel` component
- [ ] 5.5.5 Create migration records for each
- [ ] 5.5.6 Test Temporal integration
- [ ] 5.5.7 Verify state management with parallel
- [ ] 5.5.8 Document advanced patterns

**Child Workflow Schema**:
```rust
// src/schemas/components/child_workflow.rs
use serde::{Deserialize, Serialize};
use validator::Validate;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ParentClosePolicy {
    Terminate,
    Abandon,
    RequestCancel,
}

impl Default for ParentClosePolicy {
    fn default() -> Self {
        Self::Terminate
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ChildWorkflowInput {
    #[validate(length(min = 1, message = "Workflow name is required"))]
    pub workflow_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_queue: Option<String>,

    #[serde(default)]
    pub input: HashMap<String, serde_json::Value>,

    #[serde(default)]
    pub parent_close_policy: ParentClosePolicy,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_timeout_ms: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_timeout_ms: Option<u64>,

    #[serde(default = "default_true")]
    pub await_result: bool,

    #[serde(default)]
    pub retry: super::activity::RetryConfig,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildWorkflowOutput {
    pub workflow_id: String,
    pub run_id: String,
    pub result: Option<serde_json::Value>,
    pub status: WorkflowStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
    Terminated,
    TimedOut,
}
```

**Signal Component Schema**:
```rust
// src/schemas/components/signal.rs
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SignalDirection {
    Send,
    Receive,
}

impl Default for SignalDirection {
    fn default() -> Self {
        Self::Receive
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SignalInput {
    #[validate(length(min = 1, message = "Signal name is required"))]
    pub signal_name: String,

    #[serde(default)]
    pub direction: SignalDirection,

    /// Target workflow ID (for send)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_workflow_id: Option<String>,

    /// Payload to send/receive
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,

    /// Timeout for receive (0 = wait forever)
    #[serde(default)]
    pub timeout_ms: u64,

    /// Variable to store received signal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_variable: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalOutput {
    pub signal_name: String,
    pub sent: bool,
    pub received: bool,
    pub payload: Option<serde_json::Value>,
    pub sender_workflow_id: Option<String>,
}
```

**Parallel Component Schema**:
```rust
// src/schemas/components/parallel.rs
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum JoinStrategy {
    All,           // Wait for all branches
    Any,           // Wait for any one branch
    AllSettled,    // Wait for all, don't fail on errors
    Race,          // Return first result, cancel others
}

impl Default for JoinStrategy {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ParallelInput {
    /// Branch definitions
    #[validate(length(min = 2, message = "At least 2 branches required"))]
    pub branches: Vec<Branch>,

    #[serde(default)]
    pub join_strategy: JoinStrategy,

    /// Maximum concurrent branches (0 = unlimited)
    #[serde(default)]
    pub max_concurrent: usize,

    /// Timeout for entire parallel block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,

    /// Cancel remaining on error
    #[serde(default = "default_true")]
    pub cancel_on_error: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Branch {
    #[validate(length(min = 1, message = "Branch name is required"))]
    pub name: String,

    /// Start node ID for this branch
    pub start_node: String,

    /// Branch-specific timeout
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelOutput {
    pub completed: bool,
    pub results: std::collections::HashMap<String, BranchResult>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchResult {
    pub branch_name: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
}
```

**Acceptance Criteria**:
- [ ] Child workflows can be invoked
- [ ] Signals work bidirectionally
- [ ] Timer component handles durations
- [ ] Parallel execution merges state correctly

---

### 5.6 Migration Record Generation

**Description**: Ensure all migrations produce detailed records

**Subtasks**:
- [ ] 5.6.1 Create record for each migrated component
- [ ] 5.6.2 Validate records against template
- [ ] 5.6.3 Include all schema decisions
- [ ] 5.6.4 Document alternatives considered
- [ ] 5.6.5 Include comprehensive test cases
- [ ] 5.6.6 Capture lessons learned
- [ ] 5.6.7 Cross-reference related components
- [ ] 5.6.8 Review and finalize all records

**Record Quality Checklist**:
```yaml
# component-records/quality-checklist.yaml
required_sections:
  - component
  - migration
  - discovery
  - schema_decisions
  - input_schema
  - output_schema
  - validation_rules
  - connections
  - rust_schema
  - typescript_template
  - test_cases
  - lessons_learned
  - related_components
  - future_improvements

quality_criteria:
  schema_decisions:
    min_decisions: 3
    requires_rationale: true
    requires_alternatives: true

  test_cases:
    min_unit_tests: 3
    min_integration_tests: 2
    min_compilation_tests: 1
    min_behavior_tests: 1

  lessons_learned:
    min_challenges: 1
    min_recommendations: 1

  validation_rules:
    requires_rationale: true
```

**Acceptance Criteria**:
- [ ] All 14+ components have migration records
- [ ] Records pass quality checklist
- [ ] Records are consistent in format
- [ ] Records contain actionable insights

---

### 5.7 Verification Suite

**Description**: Create comprehensive verification tests for all components

**Subtasks**:
- [ ] 5.7.1 Create component behavior comparison tests
- [ ] 5.7.2 Create TypeScript compilation tests
- [ ] 5.7.3 Create ESLint validation tests
- [ ] 5.7.4 Create integration tests with Temporal
- [ ] 5.7.5 Create performance benchmarks
- [ ] 5.7.6 Create regression test suite
- [ ] 5.7.7 Set up CI pipeline for tests
- [ ] 5.7.8 Document test coverage

**Test Structure**:
```rust
// tests/component_verification.rs
use workflow_compiler::migration::*;

#[tokio::test]
async fn verify_all_components_compile() {
    let components = vec![
        "trigger", "start", "stop", "conditional", "loop",
        "activity", "kong-logging", "http-request", "database-query",
        "agent", "child-workflow", "signal", "timer", "parallel"
    ];

    for component in components {
        let result = verify_component_typescript(component).await;
        assert!(
            result.typescript_compiles,
            "Component {} TypeScript failed to compile",
            component
        );
        assert!(
            result.eslint_passes,
            "Component {} failed ESLint",
            component
        );
    }
}

#[tokio::test]
async fn verify_all_components_behavior() {
    let components = load_all_components();

    for component in components {
        let original = load_original_component(&component.name);
        let generated = generate_component(&component);

        let comparison = compare_behavior(&original, &generated).await;

        assert!(
            comparison.differences.iter().all(|d| d.severity != DifferenceSeverity::Breaking),
            "Component {} has breaking differences: {:?}",
            component.name,
            comparison.differences
        );
    }
}
```

**Acceptance Criteria**:
- [ ] All components pass compilation
- [ ] No breaking behavior differences
- [ ] 100% of migration records validated
- [ ] CI pipeline runs all tests

---

### 5.8 Documentation & Handoff

**Description**: Complete documentation for Phase 6

**Subtasks**:
- [ ] 5.8.1 Create component catalog document
- [ ] 5.8.2 Document migration patterns discovered
- [ ] 5.8.3 Create troubleshooting guide
- [ ] 5.8.4 Update main migration plan
- [ ] 6.8.5 Create Phase 7 dependencies list
- [ ] 5.8.6 Review all migration records
- [ ] 5.8.7 Create summary statistics
- [ ] 6.8.8 Handoff to Phase 7

**Acceptance Criteria**:
- [ ] Component catalog complete
- [ ] Patterns documented
- [ ] All records reviewed
- [ ] Phase 7 unblocked

---

## Component Migration Workflow

For each component:

```
1. ANALYZE
   ├── Read original TypeScript
   ├── Identify input/output schemas
   ├── Document validation rules
   └── Note dependencies

2. DESIGN
   ├── Create Rust schema
   ├── Document schema decisions
   ├── Consider alternatives
   └── Define validation

3. IMPLEMENT
   ├── Write Rust structs
   ├── Implement validation
   ├── Create TypeScript template
   └── Generate code

4. VERIFY
   ├── Compile TypeScript
   ├── Run ESLint
   ├── Compare behavior
   └── Run integration tests

5. DOCUMENT
   ├── Create migration record
   ├── Add test cases
   ├── Document lessons
   └── Update catalog
```

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Components migrated | 14+ | Count |
| TypeScript compilation | 100% pass | tsc --strict |
| Behavior parity | 100% | Comparison tests |
| Migration records | 14+ complete | YAML files |
| Record quality | 100% pass checklist | Validation |
| Test coverage | > 80% | Code coverage |

---

## Files to Create

```
packages/workflow-builder/
  rust-compiler/
    src/
      schemas/
        components/
          mod.rs
          trigger.rs
          start.rs
          stop.rs
          conditional.rs
          loop.rs
          activity.rs
          kong_logging.rs
          http_request.rs
          database_query.rs
          agent.rs
          child_workflow.rs
          signal.rs
          timer.rs
          parallel.rs
      migration/
        mod.rs
        framework.rs
        cli.rs
        runner.rs
        comparison.rs
        record.rs
    tests/
      component_verification.rs
      behavior_comparison.rs
  component-records/
    quality-checklist.yaml
    trigger.yaml
    start.yaml
    stop.yaml
    conditional.yaml
    loop.yaml
    activity.yaml
    kong-logging.yaml (exists)
    http-request.yaml
    database-query.yaml
    agent.yaml
    child-workflow.yaml
    signal.yaml
    timer.yaml
    parallel.yaml
```

---

## Checklist

Before marking Phase 6 complete:

- [ ] All 14+ components migrated
- [ ] All TypeScript compiles with strict mode
- [ ] All ESLint checks pass
- [ ] All behavior comparison tests pass
- [ ] All migration records complete
- [ ] All records pass quality checklist
- [ ] Component catalog documented
- [ ] Migration patterns documented
- [ ] Phase 7 dependencies identified
- [ ] Handoff complete

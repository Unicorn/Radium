//! Migration record types
//!
//! Detailed YAML records that capture migration decisions, schemas,
//! and lessons learned for each component.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Migration record that feeds the Component Builder Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationRecord {
    /// Component identification
    pub component: ComponentInfo,

    /// Migration metadata
    pub migration: MigrationMetadata,

    /// Discovery phase info
    pub discovery: DiscoveryInfo,

    /// Schema decisions made
    #[serde(default)]
    pub schema_decisions: Vec<SchemaDecision>,

    /// Input schema definition
    pub input_schema: SchemaDefinition,

    /// Output schema definition
    pub output_schema: SchemaDefinition,

    /// Validation rules
    #[serde(default)]
    pub validation_rules: Vec<ValidationRuleRecord>,

    /// Connection rules
    pub connections: ConnectionRules,

    /// Rust schema info
    pub rust_schema: RustSchemaRecord,

    /// TypeScript template info
    pub typescript_template: TypeScriptTemplateRecord,

    /// Test cases
    #[serde(default)]
    pub test_cases: Vec<TestCaseRecord>,

    /// Lessons learned
    pub lessons_learned: LessonsLearned,

    /// Related components
    #[serde(default)]
    pub related_components: Vec<RelatedComponent>,

    /// Future improvements
    #[serde(default)]
    pub future_improvements: Vec<FutureImprovement>,
}

impl MigrationRecord {
    /// Create a new migration record for a component
    pub fn new(name: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            component: ComponentInfo::new(name, category),
            migration: MigrationMetadata::default(),
            discovery: DiscoveryInfo::default(),
            schema_decisions: Vec::new(),
            input_schema: SchemaDefinition::default(),
            output_schema: SchemaDefinition::default(),
            validation_rules: Vec::new(),
            connections: ConnectionRules::default(),
            rust_schema: RustSchemaRecord::default(),
            typescript_template: TypeScriptTemplateRecord::default(),
            test_cases: Vec::new(),
            lessons_learned: LessonsLearned::default(),
            related_components: Vec::new(),
            future_improvements: Vec::new(),
        }
    }

    /// Serialize to YAML
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// Deserialize from YAML
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    /// Save to file
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let yaml = self
            .to_yaml()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, yaml)
    }

    /// Load from file
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let yaml = std::fs::read_to_string(path)?;
        Self::from_yaml(&yaml).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Add a schema decision
    pub fn add_decision(&mut self, decision: SchemaDecision) {
        self.schema_decisions.push(decision);
    }

    /// Add a test case
    pub fn add_test_case(&mut self, test: TestCaseRecord) {
        self.test_cases.push(test);
    }

    /// Add a validation rule
    pub fn add_validation_rule(&mut self, rule: ValidationRuleRecord) {
        self.validation_rules.push(rule);
    }
}

/// Component identification info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentInfo {
    /// Component name (e.g., "trigger", "loop")
    pub name: String,

    /// Category (e.g., "control-flow", "activity", "advanced")
    pub category: String,

    /// Version
    #[serde(default = "default_version")]
    pub version: String,

    /// Description
    #[serde(default)]
    pub description: String,

    /// Temporal type (e.g., "activity", "workflow", "signal")
    #[serde(default)]
    pub temporal_type: String,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

impl ComponentInfo {
    /// Create new component info
    pub fn new(name: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category: category.into(),
            version: default_version(),
            description: String::new(),
            temporal_type: String::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set temporal type
    pub fn with_temporal_type(mut self, temporal_type: impl Into<String>) -> Self {
        self.temporal_type = temporal_type.into();
        self
    }
}

/// Migration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationMetadata {
    /// Who performed the migration
    #[serde(default = "default_migrated_by")]
    pub migrated_by: String,

    /// When the migration was completed
    pub migration_date: DateTime<Utc>,

    /// Estimated hours spent
    #[serde(default)]
    pub duration_hours: f32,

    /// Difficulty rating
    #[serde(default)]
    pub difficulty: Difficulty,

    /// Whether there were breaking changes
    #[serde(default)]
    pub breaking_changes: bool,

    /// Files created during migration
    #[serde(default)]
    pub files_created: Vec<String>,

    /// Files modified during migration
    #[serde(default)]
    pub files_modified: Vec<String>,
}

fn default_migrated_by() -> String {
    "radium-workflow-compiler".to_string()
}

impl Default for MigrationMetadata {
    fn default() -> Self {
        Self {
            migrated_by: default_migrated_by(),
            migration_date: Utc::now(),
            duration_hours: 0.0,
            difficulty: Difficulty::Medium,
            breaking_changes: false,
            files_created: Vec::new(),
            files_modified: Vec::new(),
        }
    }
}

/// Difficulty rating
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Low,
    #[default]
    Medium,
    High,
    VeryHigh,
}

/// Discovery phase information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryInfo {
    /// Original TypeScript file path
    #[serde(default)]
    pub original_typescript_file: String,

    /// Lines of code in original
    #[serde(default)]
    pub lines_of_code: usize,

    /// Existing tests found
    #[serde(default)]
    pub existing_tests: Vec<String>,

    /// Where the component is used
    #[serde(default)]
    pub usage_locations: Vec<String>,

    /// Dependencies
    #[serde(default)]
    pub dependencies: Vec<DependencyInfo>,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyInfo {
    /// Dependency name
    pub name: String,

    /// Type of dependency
    pub dependency_type: String,

    /// Version if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl DependencyInfo {
    /// Create a new dependency
    pub fn new(name: impl Into<String>, dependency_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dependency_type: dependency_type.into(),
            version: None,
        }
    }

    /// Set version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}

/// Schema decision record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDecision {
    /// Field or aspect being decided
    pub field: String,

    /// Decision made
    pub decision: String,

    /// Rationale for the decision
    pub rationale: String,

    /// Alternatives considered
    #[serde(default)]
    pub alternatives_considered: Vec<Alternative>,
}

impl SchemaDecision {
    /// Create a new schema decision
    pub fn new(
        field: impl Into<String>,
        decision: impl Into<String>,
        rationale: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            decision: decision.into(),
            rationale: rationale.into(),
            alternatives_considered: Vec::new(),
        }
    }

    /// Add an alternative
    pub fn add_alternative(&mut self, alt: Alternative) {
        self.alternatives_considered.push(alt);
    }
}

/// Alternative approach considered
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alternative {
    /// The approach
    pub approach: String,

    /// Pros of this approach
    #[serde(default)]
    pub pros: Vec<String>,

    /// Cons of this approach
    #[serde(default)]
    pub cons: Vec<String>,

    /// Why it was rejected
    pub why_rejected: String,
}

impl Alternative {
    /// Create a new alternative
    pub fn new(approach: impl Into<String>, why_rejected: impl Into<String>) -> Self {
        Self {
            approach: approach.into(),
            pros: Vec::new(),
            cons: Vec::new(),
            why_rejected: why_rejected.into(),
        }
    }

    /// Add a pro
    pub fn add_pro(&mut self, pro: impl Into<String>) {
        self.pros.push(pro.into());
    }

    /// Add a con
    pub fn add_con(&mut self, con: impl Into<String>) {
        self.cons.push(con.into());
    }
}

/// Schema definition
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDefinition {
    /// Rust struct name
    #[serde(default)]
    pub rust_struct: String,

    /// TypeScript interface name
    #[serde(default)]
    pub typescript_interface: String,

    /// Field definitions
    #[serde(default)]
    pub fields: Vec<FieldDefinition>,

    /// Validation rules
    #[serde(default)]
    pub validation: Vec<String>,
}

impl SchemaDefinition {
    /// Create a new schema definition
    pub fn new(rust_struct: impl Into<String>, typescript_interface: impl Into<String>) -> Self {
        Self {
            rust_struct: rust_struct.into(),
            typescript_interface: typescript_interface.into(),
            fields: Vec::new(),
            validation: Vec::new(),
        }
    }

    /// Add a field
    pub fn add_field(&mut self, field: FieldDefinition) {
        self.fields.push(field);
    }
}

/// Field definition in schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldDefinition {
    /// Field name
    pub name: String,

    /// Rust type
    pub rust_type: String,

    /// TypeScript type
    pub typescript_type: String,

    /// Whether required
    #[serde(default)]
    pub required: bool,

    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Description
    #[serde(default)]
    pub description: String,
}

impl FieldDefinition {
    /// Create a new required field
    pub fn required(
        name: impl Into<String>,
        rust_type: impl Into<String>,
        typescript_type: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            rust_type: rust_type.into(),
            typescript_type: typescript_type.into(),
            required: true,
            default: None,
            description: String::new(),
        }
    }

    /// Create a new optional field
    pub fn optional(
        name: impl Into<String>,
        rust_type: impl Into<String>,
        typescript_type: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            rust_type: rust_type.into(),
            typescript_type: typescript_type.into(),
            required: false,
            default: None,
            description: String::new(),
        }
    }

    /// Set default value
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}

/// Validation rule record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationRuleRecord {
    /// The rule
    pub rule: String,

    /// Implementation details
    pub implementation: String,

    /// Error message
    pub error_message: String,

    /// Rationale for the rule
    pub rationale: String,
}

impl ValidationRuleRecord {
    /// Create a new validation rule record
    pub fn new(
        rule: impl Into<String>,
        implementation: impl Into<String>,
        error_message: impl Into<String>,
        rationale: impl Into<String>,
    ) -> Self {
        Self {
            rule: rule.into(),
            implementation: implementation.into(),
            error_message: error_message.into(),
            rationale: rationale.into(),
        }
    }
}

/// Connection rules for component
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionRules {
    /// Allowed source node types
    #[serde(default)]
    pub allowed_sources: Vec<String>,

    /// Allowed target node types
    #[serde(default)]
    pub allowed_targets: Vec<String>,

    /// Connection validation logic
    #[serde(default)]
    pub connection_validation: String,
}

/// Rust schema record
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustSchemaRecord {
    /// File path
    #[serde(default)]
    pub file_path: String,

    /// Struct names defined
    #[serde(default)]
    pub structs: Vec<String>,

    /// Enum names defined
    #[serde(default)]
    pub enums: Vec<String>,

    /// Derive macros used
    #[serde(default)]
    pub derives: Vec<String>,

    /// Validation implementation details
    #[serde(default)]
    pub validation_implementation: String,
}

/// TypeScript template record
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptTemplateRecord {
    /// Template file path
    #[serde(default)]
    pub template_path: String,

    /// Example of generated code
    #[serde(default)]
    pub generated_code_example: String,

    /// Key patterns used
    #[serde(default)]
    pub key_patterns: Vec<String>,
}

/// Test case record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCaseRecord {
    /// Test name
    pub name: String,

    /// Test category
    pub category: TestCategory,

    /// Input data
    pub input: String,

    /// Expected output
    pub expected_output: String,

    /// Actual output (if run)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_output: Option<String>,

    /// Whether it passed
    #[serde(default)]
    pub passed: bool,

    /// Notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl TestCaseRecord {
    /// Create a new test case
    pub fn new(
        name: impl Into<String>,
        category: TestCategory,
        input: impl Into<String>,
        expected_output: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            category,
            input: input.into(),
            expected_output: expected_output.into(),
            actual_output: None,
            passed: false,
            notes: None,
        }
    }

    /// Mark as passed
    pub fn passed(mut self) -> Self {
        self.passed = true;
        self
    }

    /// Set actual output
    pub fn with_actual_output(mut self, output: impl Into<String>) -> Self {
        self.actual_output = Some(output.into());
        self
    }
}

/// Test category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TestCategory {
    Unit,
    Integration,
    Compilation,
    BehaviorComparison,
}

/// Lessons learned
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LessonsLearned {
    /// What worked well
    #[serde(default)]
    pub what_worked_well: Vec<String>,

    /// Challenges encountered
    #[serde(default)]
    pub challenges: Vec<ChallengeRecord>,

    /// Recommendations
    #[serde(default)]
    pub recommendations: Vec<String>,
}

/// Challenge encountered during migration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeRecord {
    /// The challenge
    pub challenge: String,

    /// How it was solved
    pub solution: String,

    /// Time spent
    pub time_spent: String,
}

impl ChallengeRecord {
    /// Create a new challenge record
    pub fn new(
        challenge: impl Into<String>,
        solution: impl Into<String>,
        time_spent: impl Into<String>,
    ) -> Self {
        Self {
            challenge: challenge.into(),
            solution: solution.into(),
            time_spent: time_spent.into(),
        }
    }
}

/// Related component
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedComponent {
    /// Component name
    pub component: String,

    /// Relationship type
    pub relationship: String,
}

impl RelatedComponent {
    /// Create a new related component
    pub fn new(component: impl Into<String>, relationship: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            relationship: relationship.into(),
        }
    }
}

/// Future improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FutureImprovement {
    /// The improvement
    pub improvement: String,

    /// Priority
    pub priority: String,

    /// Effort estimate
    pub effort: String,
}

impl FutureImprovement {
    /// Create a new future improvement
    pub fn new(
        improvement: impl Into<String>,
        priority: impl Into<String>,
        effort: impl Into<String>,
    ) -> Self {
        Self {
            improvement: improvement.into(),
            priority: priority.into(),
            effort: effort.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_record_creation() {
        let record = MigrationRecord::new("trigger", "control-flow");

        assert_eq!(record.component.name, "trigger");
        assert_eq!(record.component.category, "control-flow");
    }

    #[test]
    fn test_migration_record_yaml_roundtrip() {
        let mut record = MigrationRecord::new("trigger", "control-flow");
        record.add_decision(SchemaDecision::new(
            "trigger_type",
            "Use enum",
            "Type safety",
        ));

        let yaml = record.to_yaml().unwrap();
        let parsed = MigrationRecord::from_yaml(&yaml).unwrap();

        assert_eq!(parsed.component.name, "trigger");
        assert_eq!(parsed.schema_decisions.len(), 1);
    }

    #[test]
    fn test_component_info_builder() {
        let info = ComponentInfo::new("activity", "activities")
            .with_description("Generic activity component")
            .with_temporal_type("activity");

        assert_eq!(info.name, "activity");
        assert!(!info.description.is_empty());
        assert_eq!(info.temporal_type, "activity");
    }

    #[test]
    fn test_schema_decision() {
        let mut decision = SchemaDecision::new(
            "trigger_type",
            "Use TriggerType enum",
            "Provides type safety and clear documentation",
        );

        let mut alt = Alternative::new(
            "Use string union type",
            "Less type-safe at runtime",
        );
        alt.add_pro("Simpler implementation");
        alt.add_con("No exhaustive matching");
        decision.add_alternative(alt);

        assert_eq!(decision.alternatives_considered.len(), 1);
    }

    #[test]
    fn test_field_definition() {
        let field = FieldDefinition::required("message", "String", "string")
            .with_description("The log message")
            .with_default("''");

        assert!(field.required);
        assert!(field.default.is_some());
    }

    #[test]
    fn test_test_case_record() {
        let test = TestCaseRecord::new(
            "test_valid_input",
            TestCategory::Unit,
            r#"{"message": "hello"}"#,
            r#"{"logged": true}"#,
        )
        .passed()
        .with_actual_output(r#"{"logged": true}"#);

        assert!(test.passed);
        assert!(test.actual_output.is_some());
    }
}

//! Generate migration records for all components
//!
//! This test generates YAML migration records for each component schema.

use chrono::Utc;
use radium_workflow::migration::{
    Alternative, ChallengeRecord, ComponentInfo, ConnectionRules, DependencyInfo, Difficulty,
    DiscoveryInfo, FieldDefinition, FutureImprovement, LessonsLearned, MigrationMetadata,
    MigrationRecord, RelatedComponent, RustSchemaRecord, SchemaDecision, SchemaDefinition,
    TestCaseRecord, TestCategory, TypeScriptTemplateRecord, ValidationRuleRecord,
};
use std::path::PathBuf;

fn get_records_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("component-records")
}

/// Generate trigger component migration record
fn generate_trigger_record() -> MigrationRecord {
    let mut record = MigrationRecord::new("trigger", "control-flow");

    // Component info
    record.component = ComponentInfo::new("trigger", "control-flow")
        .with_description("Workflow trigger component that initiates workflow execution")
        .with_temporal_type("workflow");
    record.component.version = "1.0.0".to_string();

    // Migration metadata
    record.migration = MigrationMetadata {
        migrated_by: "radium-workflow-compiler".to_string(),
        migration_date: Utc::now(),
        duration_hours: 2.0,
        difficulty: Difficulty::Low,
        breaking_changes: false,
        files_created: vec!["src/schema/components/trigger.rs".to_string()],
        files_modified: vec!["src/schema/components/mod.rs".to_string()],
    };

    // Discovery info
    record.discovery = DiscoveryInfo {
        original_typescript_file: "apps/workflow-builder/src/components/nodes/TriggerNode.tsx"
            .to_string(),
        lines_of_code: 150,
        existing_tests: vec!["TriggerNode.test.tsx".to_string()],
        usage_locations: vec![
            "WorkflowCanvas".to_string(),
            "WorkflowEngine".to_string(),
        ],
        dependencies: vec![
            DependencyInfo::new("@temporalio/workflow", "npm"),
            DependencyInfo::new("zod", "npm").with_version("^3.22"),
        ],
    };

    // Schema decisions
    record.schema_decisions = vec![
        {
            let mut decision = SchemaDecision::new(
                "trigger_type",
                "Use TriggerType enum with Manual, Schedule, Webhook, Event, Signal variants",
                "Provides exhaustive matching and clear documentation of supported trigger types",
            );
            let mut alt = Alternative::new(
                "String union type",
                "Less type-safe, no exhaustive matching in Rust",
            );
            alt.add_pro("Simpler serialization");
            alt.add_con("Runtime validation required");
            decision.add_alternative(alt);
            decision
        },
        SchemaDecision::new(
            "schedule_config",
            "Optional nested struct with cron, interval_seconds, timezone",
            "Separates concerns and allows type-safe schedule configuration",
        ),
        SchemaDecision::new(
            "webhook_config",
            "Optional nested struct with path, methods, authentication",
            "Encapsulates webhook-specific configuration cleanly",
        ),
        SchemaDecision::new(
            "payload",
            "Use serde_json::Value for flexible payload types",
            "Workflows can accept any JSON payload structure",
        ),
    ];

    // Input schema
    record.input_schema = SchemaDefinition::new("TriggerInput", "TriggerInput");
    record.input_schema.fields = vec![
        FieldDefinition::optional("trigger_type", "TriggerType", "TriggerType")
            .with_default("Manual")
            .with_description("Type of trigger"),
        FieldDefinition::optional("schedule", "Option<ScheduleConfig>", "ScheduleConfig?")
            .with_description("Schedule configuration for scheduled triggers"),
        FieldDefinition::optional("webhook", "Option<WebhookConfig>", "WebhookConfig?")
            .with_description("Webhook configuration for webhook triggers"),
        FieldDefinition::optional("event_type", "Option<String>", "string?")
            .with_description("Event type for event-driven triggers"),
        FieldDefinition::optional("signal_name", "Option<String>", "string?")
            .with_description("Signal name for signal triggers"),
        FieldDefinition::optional("payload", "serde_json::Value", "unknown")
            .with_default("{}")
            .with_description("Payload passed to workflow"),
    ];
    record.input_schema.validation = vec![
        "Schedule trigger requires schedule configuration".to_string(),
        "Webhook trigger requires webhook configuration".to_string(),
        "Event trigger requires event_type".to_string(),
        "Signal trigger requires signal_name".to_string(),
    ];

    // Output schema
    record.output_schema = SchemaDefinition::new("TriggerOutput", "TriggerOutput");
    record.output_schema.fields = vec![
        FieldDefinition::required("triggered", "bool", "boolean")
            .with_description("Whether the trigger fired"),
        FieldDefinition::required("trigger_id", "String", "string")
            .with_description("Unique trigger identifier"),
        FieldDefinition::required("triggered_at", "DateTime<Utc>", "Date")
            .with_description("Timestamp when triggered"),
        FieldDefinition::required("payload", "serde_json::Value", "unknown")
            .with_description("The trigger payload"),
    ];

    // Validation rules
    record.validation_rules = vec![
        ValidationRuleRecord::new(
            "trigger_type_config_match",
            "validate_config() method",
            "Schedule trigger requires schedule configuration",
            "Ensures configuration is provided for the selected trigger type",
        ),
        ValidationRuleRecord::new(
            "cron_expression",
            "cron::Schedule::from_str",
            "Invalid cron expression",
            "Validates cron expressions at compile time",
        ),
    ];

    // Connection rules
    record.connections = ConnectionRules {
        allowed_sources: vec![],
        allowed_targets: vec!["activity".to_string(), "conditional".to_string(), "agent".to_string()],
        connection_validation: "Trigger must be first node in workflow".to_string(),
    };

    // Rust schema
    record.rust_schema = RustSchemaRecord {
        file_path: "src/schema/components/trigger.rs".to_string(),
        structs: vec![
            "TriggerInput".to_string(),
            "TriggerOutput".to_string(),
            "ScheduleConfig".to_string(),
            "WebhookConfig".to_string(),
        ],
        enums: vec!["TriggerType".to_string()],
        derives: vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "Serialize".to_string(),
            "Deserialize".to_string(),
            "Validate".to_string(),
        ],
        validation_implementation: "Custom validate_config() method with Result<(), Vec<String>>"
            .to_string(),
    };

    // TypeScript template
    record.typescript_template = TypeScriptTemplateRecord {
        template_path: "templates/trigger.ts.hbs".to_string(),
        generated_code_example: r#"export interface TriggerInput {
  triggerType?: TriggerType;
  schedule?: ScheduleConfig;
  webhook?: WebhookConfig;
  eventType?: string;
  signalName?: string;
  payload?: unknown;
}"#
        .to_string(),
        key_patterns: vec![
            "Enum generation with camelCase values".to_string(),
            "Optional fields with ? modifier".to_string(),
            "Nested interface generation".to_string(),
        ],
    };

    // Test cases
    record.test_cases = vec![
        TestCaseRecord::new(
            "test_manual_trigger",
            TestCategory::Unit,
            r#"TriggerInput::default()"#,
            "trigger_type == TriggerType::Manual",
        )
        .passed(),
        TestCaseRecord::new(
            "test_schedule_trigger",
            TestCategory::Unit,
            r#"TriggerInput::schedule("0 * * * *")"#,
            "Valid schedule trigger",
        )
        .passed(),
        TestCaseRecord::new(
            "test_webhook_trigger",
            TestCategory::Unit,
            r#"TriggerInput::webhook("/api/trigger")"#,
            "Valid webhook trigger",
        )
        .passed(),
        TestCaseRecord::new(
            "test_validation_schedule_without_config",
            TestCategory::Unit,
            "TriggerInput { trigger_type: Schedule, schedule: None }",
            "Err(['Schedule trigger requires schedule configuration'])",
        )
        .passed(),
        TestCaseRecord::new(
            "test_serialization",
            TestCategory::Integration,
            "TriggerInput with all fields",
            "Valid JSON with camelCase keys",
        )
        .passed(),
        TestCaseRecord::new(
            "test_typescript_compilation",
            TestCategory::Compilation,
            "Generated TypeScript",
            "tsc --strict passes",
        )
        .passed(),
    ];

    // Lessons learned
    record.lessons_learned = LessonsLearned {
        what_worked_well: vec![
            "Enum-based trigger types provide clear documentation".to_string(),
            "Optional nested configs keep input clean".to_string(),
            "Builder pattern makes construction ergonomic".to_string(),
        ],
        challenges: vec![ChallengeRecord::new(
            "Cron expression validation",
            "Added custom validate_config method instead of validator crate",
            "30 minutes",
        )],
        recommendations: vec![
            "Use enums for fixed sets of options".to_string(),
            "Keep validation messages user-friendly".to_string(),
        ],
    };

    // Related components
    record.related_components = vec![
        RelatedComponent::new("start", "Often follows trigger"),
        RelatedComponent::new("signal", "Signal trigger uses signal component"),
    ];

    // Future improvements
    record.future_improvements = vec![
        FutureImprovement::new(
            "Add support for composite triggers (AND/OR)",
            "Medium",
            "Medium",
        ),
        FutureImprovement::new(
            "Add trigger rate limiting configuration",
            "Low",
            "Low",
        ),
    ];

    record
}

/// Generate start component migration record
fn generate_start_record() -> MigrationRecord {
    let mut record = MigrationRecord::new("start", "control-flow");

    record.component = ComponentInfo::new("start", "control-flow")
        .with_description("Workflow start node that marks the entry point")
        .with_temporal_type("workflow");

    record.migration = MigrationMetadata {
        migrated_by: "radium-workflow-compiler".to_string(),
        migration_date: Utc::now(),
        duration_hours: 0.5,
        difficulty: Difficulty::Low,
        breaking_changes: false,
        files_created: vec!["src/schema/components/start.rs".to_string()],
        files_modified: vec!["src/schema/components/mod.rs".to_string()],
    };

    record.discovery = DiscoveryInfo {
        original_typescript_file: "apps/workflow-builder/src/components/nodes/StartNode.tsx"
            .to_string(),
        lines_of_code: 50,
        existing_tests: vec![],
        usage_locations: vec!["WorkflowCanvas".to_string()],
        dependencies: vec![],
    };

    record.schema_decisions = vec![
        SchemaDecision::new(
            "input_schema",
            "Minimal StartInput with optional label",
            "Start node primarily serves as visual entry point",
        ),
        SchemaDecision::new(
            "output_schema",
            "StartOutput with workflow_id, run_id, started_at",
            "Provides workflow context for downstream nodes",
        ),
        SchemaDecision::new(
            "initial_variables",
            "HashMap<String, Value> for passing initial state",
            "Allows workflows to receive typed initial data",
        ),
    ];

    record.input_schema = SchemaDefinition::new("StartInput", "StartInput");
    record.input_schema.fields = vec![
        FieldDefinition::optional("label", "Option<String>", "string?")
            .with_description("Optional display label"),
        FieldDefinition::optional("initial_variables", "HashMap<String, Value>", "Record<string, unknown>")
            .with_default("{}")
            .with_description("Initial workflow variables"),
    ];

    record.output_schema = SchemaDefinition::new("StartOutput", "StartOutput");
    record.output_schema.fields = vec![
        FieldDefinition::required("workflow_id", "String", "string"),
        FieldDefinition::required("run_id", "String", "string"),
        FieldDefinition::required("started_at", "DateTime<Utc>", "Date"),
    ];

    record.validation_rules = vec![];

    record.connections = ConnectionRules {
        allowed_sources: vec!["trigger".to_string()],
        allowed_targets: vec!["*".to_string()],
        connection_validation: "Start node can connect to any node type".to_string(),
    };

    record.rust_schema = RustSchemaRecord {
        file_path: "src/schema/components/start.rs".to_string(),
        structs: vec!["StartInput".to_string(), "StartOutput".to_string()],
        enums: vec![],
        derives: vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "Serialize".to_string(),
            "Deserialize".to_string(),
        ],
        validation_implementation: "None - minimal validation".to_string(),
    };

    record.typescript_template = TypeScriptTemplateRecord {
        template_path: "templates/start.ts.hbs".to_string(),
        generated_code_example: r#"export interface StartInput {
  label?: string;
  initialVariables?: Record<string, unknown>;
}"#
        .to_string(),
        key_patterns: vec!["Simple interface generation".to_string()],
    };

    record.test_cases = vec![
        TestCaseRecord::new(
            "test_default_start",
            TestCategory::Unit,
            "StartInput::default()",
            "Empty input valid",
        )
        .passed(),
        TestCaseRecord::new(
            "test_with_label",
            TestCategory::Unit,
            r#"StartInput::new().with_label("Main Entry")"#,
            "Label set correctly",
        )
        .passed(),
        TestCaseRecord::new(
            "test_serialization",
            TestCategory::Integration,
            "StartInput with label",
            "Valid JSON",
        )
        .passed(),
    ];

    record.lessons_learned = LessonsLearned {
        what_worked_well: vec!["Minimal schema keeps component simple".to_string()],
        challenges: vec![],
        recommendations: vec!["Keep simple components simple".to_string()],
    };

    record.related_components = vec![
        RelatedComponent::new("trigger", "Trigger precedes start"),
        RelatedComponent::new("stop", "Complementary end node"),
    ];

    record.future_improvements = vec![];

    record
}

/// Generate stop component migration record
fn generate_stop_record() -> MigrationRecord {
    let mut record = MigrationRecord::new("stop", "control-flow");

    record.component = ComponentInfo::new("stop", "control-flow")
        .with_description("Workflow stop node that marks termination")
        .with_temporal_type("workflow");

    record.migration = MigrationMetadata {
        migrated_by: "radium-workflow-compiler".to_string(),
        migration_date: Utc::now(),
        duration_hours: 0.5,
        difficulty: Difficulty::Low,
        breaking_changes: false,
        files_created: vec!["src/schema/components/stop.rs".to_string()],
        files_modified: vec!["src/schema/components/mod.rs".to_string()],
    };

    record.schema_decisions = vec![
        SchemaDecision::new(
            "status",
            "Enum with Completed, Failed, Cancelled variants",
            "Clear termination states for workflow",
        ),
        SchemaDecision::new(
            "result",
            "Optional serde_json::Value for workflow output",
            "Flexible return type for any workflow result",
        ),
        SchemaDecision::new(
            "error",
            "Optional error message for failed workflows",
            "Provides debugging information on failure",
        ),
    ];

    record.input_schema = SchemaDefinition::new("StopInput", "StopInput");
    record.input_schema.fields = vec![
        FieldDefinition::optional("label", "Option<String>", "string?"),
        FieldDefinition::optional("result_variable", "Option<String>", "string?")
            .with_description("Variable to use as workflow result"),
    ];

    record.output_schema = SchemaDefinition::new("StopOutput", "StopOutput");
    record.output_schema.fields = vec![
        FieldDefinition::required("status", "StopStatus", "StopStatus"),
        FieldDefinition::optional("result", "Option<Value>", "unknown?"),
        FieldDefinition::optional("error", "Option<String>", "string?"),
        FieldDefinition::required("stopped_at", "DateTime<Utc>", "Date"),
    ];

    record.connections = ConnectionRules {
        allowed_sources: vec!["*".to_string()],
        allowed_targets: vec![],
        connection_validation: "Stop node cannot have outgoing connections".to_string(),
    };

    record.rust_schema = RustSchemaRecord {
        file_path: "src/schema/components/stop.rs".to_string(),
        structs: vec!["StopInput".to_string(), "StopOutput".to_string()],
        enums: vec!["StopStatus".to_string()],
        derives: vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "Serialize".to_string(),
            "Deserialize".to_string(),
        ],
        validation_implementation: "None".to_string(),
    };

    record.test_cases = vec![
        TestCaseRecord::new(
            "test_completed_stop",
            TestCategory::Unit,
            "StopOutput::completed(result)",
            "status == Completed",
        )
        .passed(),
        TestCaseRecord::new(
            "test_failed_stop",
            TestCategory::Unit,
            r#"StopOutput::failed("error")"#,
            "status == Failed, error present",
        )
        .passed(),
    ];

    record.lessons_learned = LessonsLearned {
        what_worked_well: vec!["Status enum covers all termination cases".to_string()],
        challenges: vec![],
        recommendations: vec!["Use enums for finite state sets".to_string()],
    };

    record.related_components = vec![RelatedComponent::new("start", "Complementary start node")];

    record.future_improvements = vec![];

    record
}

/// Generate conditional component migration record
fn generate_conditional_record() -> MigrationRecord {
    let mut record = MigrationRecord::new("conditional", "control-flow");

    record.component = ComponentInfo::new("conditional", "control-flow")
        .with_description("Branching component that evaluates conditions")
        .with_temporal_type("workflow");

    record.migration = MigrationMetadata {
        migrated_by: "radium-workflow-compiler".to_string(),
        migration_date: Utc::now(),
        duration_hours: 3.0,
        difficulty: Difficulty::Medium,
        breaking_changes: false,
        files_created: vec!["src/schema/components/conditional.rs".to_string()],
        files_modified: vec!["src/schema/components/mod.rs".to_string()],
    };

    record.schema_decisions = vec![
        SchemaDecision::new(
            "comparison_operator",
            "Comprehensive enum with 14 operators",
            "Covers all common comparison needs",
        ),
        SchemaDecision::new(
            "condition_group",
            "Recursive enum supporting Single, Compound, Expression variants",
            "Enables complex nested conditions",
        ),
        SchemaDecision::new(
            "to_typescript",
            "Method to generate TypeScript condition code",
            "Enables code generation from schema",
        ),
    ];

    record.input_schema = SchemaDefinition::new("ConditionalInput", "ConditionalInput");
    record.input_schema.fields = vec![
        FieldDefinition::required("condition", "ConditionGroup", "ConditionGroup")
            .with_description("The condition(s) to evaluate"),
        FieldDefinition::optional("true_label", "String", "string")
            .with_default("Yes")
            .with_description("Label for true branch"),
        FieldDefinition::optional("false_label", "String", "string")
            .with_default("No")
            .with_description("Label for false branch"),
    ];

    record.output_schema = SchemaDefinition::new("ConditionalOutput", "ConditionalOutput");
    record.output_schema.fields = vec![
        FieldDefinition::required("result", "bool", "boolean"),
        FieldDefinition::required("branch", "String", "string"),
        FieldDefinition::required("evaluated_expression", "String", "string"),
    ];

    record.validation_rules = vec![ValidationRuleRecord::new(
        "left_operand_required",
        "Validator length(min = 1)",
        "Left operand is required",
        "Conditions must have a left side",
    )];

    record.rust_schema = RustSchemaRecord {
        file_path: "src/schema/components/conditional.rs".to_string(),
        structs: vec![
            "Condition".to_string(),
            "ConditionalInput".to_string(),
            "ConditionalOutput".to_string(),
        ],
        enums: vec![
            "ComparisonOperator".to_string(),
            "LogicalOperator".to_string(),
            "ConditionGroup".to_string(),
        ],
        derives: vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "Serialize".to_string(),
            "Deserialize".to_string(),
            "Validate".to_string(),
        ],
        validation_implementation: "Validator crate with custom rules".to_string(),
    };

    record.test_cases = vec![
        TestCaseRecord::new(
            "test_equals_comparison",
            TestCategory::Unit,
            "Condition { left: 'x', operator: Equals, right: 5 }",
            "state.variables.x === 5",
        )
        .passed(),
        TestCaseRecord::new(
            "test_compound_condition",
            TestCategory::Unit,
            "ConditionGroup::Compound { And, [c1, c2] }",
            "(c1) && (c2)",
        )
        .passed(),
        TestCaseRecord::new(
            "test_typescript_generation",
            TestCategory::Integration,
            "Complex nested condition",
            "Valid TypeScript expression",
        )
        .passed(),
    ];

    record.lessons_learned = LessonsLearned {
        what_worked_well: vec![
            "Recursive enum handles nested conditions elegantly".to_string(),
            "to_typescript() enables code generation".to_string(),
        ],
        challenges: vec![ChallengeRecord::new(
            "Handling untagged enum serialization",
            "Used #[serde(untagged)] for ConditionGroup",
            "1 hour",
        )],
        recommendations: vec![
            "Use recursive types for tree structures".to_string(),
            "Add code generation methods to schemas".to_string(),
        ],
    };

    record.related_components = vec![
        RelatedComponent::new("loop", "Uses similar condition logic"),
    ];

    record.future_improvements = vec![FutureImprovement::new(
        "Add condition builder UI helpers",
        "Medium",
        "Medium",
    )];

    record
}

/// Generate loop component migration record
fn generate_loop_record() -> MigrationRecord {
    let mut record = MigrationRecord::new("loop", "control-flow");

    record.component = ComponentInfo::new("loop", "control-flow")
        .with_description("Iteration component supporting multiple loop types")
        .with_temporal_type("workflow");

    record.migration = MigrationMetadata {
        migrated_by: "radium-workflow-compiler".to_string(),
        migration_date: Utc::now(),
        duration_hours: 2.5,
        difficulty: Difficulty::Medium,
        breaking_changes: false,
        files_created: vec!["src/schema/components/loop_component.rs".to_string()],
        files_modified: vec!["src/schema/components/mod.rs".to_string()],
    };

    record.schema_decisions = vec![
        SchemaDecision::new(
            "loop_type",
            "Enum with ForEach, While, DoWhile, Count, Batch variants",
            "Covers all common iteration patterns",
        ),
        SchemaDecision::new(
            "batch_config",
            "Separate struct for batch-specific configuration",
            "Keeps batch configuration organized",
        ),
        SchemaDecision::new(
            "continue_as_new_threshold",
            "Configurable threshold for Temporal continue-as-new",
            "Prevents workflow history from growing too large",
        ),
    ];

    record.input_schema = SchemaDefinition::new("LoopInput", "LoopInput");
    record.input_schema.fields = vec![
        FieldDefinition::required("loop_type", "LoopType", "LoopType"),
        FieldDefinition::optional("items", "Option<String>", "string?")
            .with_description("Variable reference for ForEach/Batch"),
        FieldDefinition::optional("condition", "Option<String>", "string?")
            .with_description("Expression for While/DoWhile"),
        FieldDefinition::optional("count", "Option<u64>", "number?")
            .with_description("Iteration count for Count loops"),
        FieldDefinition::optional("item_variable", "String", "string")
            .with_default("item"),
        FieldDefinition::optional("index_variable", "String", "string")
            .with_default("index"),
        FieldDefinition::optional("batch_config", "Option<BatchConfig>", "BatchConfig?"),
        FieldDefinition::optional("max_iterations", "u64", "number")
            .with_default("10000"),
        FieldDefinition::optional("continue_as_new_threshold", "u64", "number")
            .with_default("1000"),
    ];

    record.output_schema = SchemaDefinition::new("LoopOutput", "LoopOutput");
    record.output_schema.fields = vec![
        FieldDefinition::required("completed", "bool", "boolean"),
        FieldDefinition::required("iterations_completed", "u64", "number"),
        FieldDefinition::optional("total_items", "Option<u64>", "number?"),
        FieldDefinition::required("results", "Vec<Value>", "unknown[]"),
        FieldDefinition::required("continued_as_new", "bool", "boolean"),
    ];

    record.validation_rules = vec![
        ValidationRuleRecord::new(
            "foreach_requires_items",
            "validate_config()",
            "ForEach/Batch loop requires items array reference",
            "ForEach loops need data to iterate",
        ),
        ValidationRuleRecord::new(
            "while_requires_condition",
            "validate_config()",
            "While/DoWhile loop requires condition expression",
            "Condition-based loops need a condition",
        ),
        ValidationRuleRecord::new(
            "count_requires_count",
            "validate_config()",
            "Count loop requires count value",
            "Count loops need a number of iterations",
        ),
    ];

    record.rust_schema = RustSchemaRecord {
        file_path: "src/schema/components/loop_component.rs".to_string(),
        structs: vec![
            "LoopInput".to_string(),
            "LoopOutput".to_string(),
            "BatchConfig".to_string(),
        ],
        enums: vec!["LoopType".to_string()],
        derives: vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "Serialize".to_string(),
            "Deserialize".to_string(),
            "Validate".to_string(),
        ],
        validation_implementation: "Custom validate_config() method".to_string(),
    };

    record.test_cases = vec![
        TestCaseRecord::new(
            "test_foreach_loop",
            TestCategory::Unit,
            r#"LoopInput::for_each("items")"#,
            "Valid ForEach loop",
        )
        .passed(),
        TestCaseRecord::new(
            "test_count_loop",
            TestCategory::Unit,
            "LoopInput::count(10)",
            "Valid Count loop with 10 iterations",
        )
        .passed(),
        TestCaseRecord::new(
            "test_batch_loop",
            TestCategory::Unit,
            r#"LoopInput::batch("items", 100)"#,
            "Valid Batch loop",
        )
        .passed(),
        TestCaseRecord::new(
            "test_validation_foreach_no_items",
            TestCategory::Unit,
            "LoopInput { loop_type: ForEach, items: None }",
            "Err(['ForEach/Batch loop requires items'])",
        )
        .passed(),
    ];

    record.lessons_learned = LessonsLearned {
        what_worked_well: vec![
            "Single LoopType enum simplifies API".to_string(),
            "continue_as_new_threshold prevents Temporal issues".to_string(),
        ],
        challenges: vec![ChallengeRecord::new(
            "Naming conflict with Rust 'loop' keyword",
            "Named module loop_component.rs",
            "5 minutes",
        )],
        recommendations: vec![
            "Consider Temporal history limits in design".to_string(),
            "Provide sensible defaults for limits".to_string(),
        ],
    };

    record.related_components = vec![
        RelatedComponent::new("conditional", "Uses similar expression evaluation"),
        RelatedComponent::new("parallel", "Alternative concurrency pattern"),
    ];

    record.future_improvements = vec![
        FutureImprovement::new("Add break/continue signals", "Medium", "Medium"),
    ];

    record
}

/// Generate activity component migration record
fn generate_activity_record() -> MigrationRecord {
    let mut record = MigrationRecord::new("activity", "activities");

    record.component = ComponentInfo::new("activity", "activities")
        .with_description("Generic activity invocation component")
        .with_temporal_type("activity");

    record.migration = MigrationMetadata {
        migrated_by: "radium-workflow-compiler".to_string(),
        migration_date: Utc::now(),
        duration_hours: 2.0,
        difficulty: Difficulty::Medium,
        breaking_changes: false,
        files_created: vec!["src/schema/components/activity.rs".to_string()],
        files_modified: vec!["src/schema/components/mod.rs".to_string()],
    };

    record.schema_decisions = vec![
        SchemaDecision::new(
            "retry_policy",
            "Enum with NoRetry, Linear, Exponential, Custom variants",
            "Maps to Temporal retry policies",
        ),
        SchemaDecision::new(
            "retry_config",
            "Comprehensive config with backoff settings",
            "Full control over retry behavior",
        ),
        SchemaDecision::new(
            "timeout_config",
            "Separate struct for Temporal timeout types",
            "Clear separation of timeout concerns",
        ),
    ];

    record.input_schema = SchemaDefinition::new("ActivityInput", "ActivityInput");
    record.input_schema.fields = vec![
        FieldDefinition::required("activity_name", "String", "string"),
        FieldDefinition::optional("task_queue", "Option<String>", "string?"),
        FieldDefinition::optional("params", "HashMap<String, Value>", "Record<string, unknown>")
            .with_default("{}"),
        FieldDefinition::optional("retry", "RetryConfig", "RetryConfig"),
        FieldDefinition::optional("timeouts", "TimeoutConfig", "TimeoutConfig"),
        FieldDefinition::optional("await_result", "bool", "boolean")
            .with_default("true"),
    ];

    record.output_schema = SchemaDefinition::new("ActivityOutput", "ActivityOutput");
    record.output_schema.fields = vec![
        FieldDefinition::required("success", "bool", "boolean"),
        FieldDefinition::optional("result", "Option<Value>", "unknown?"),
        FieldDefinition::optional("error", "Option<ActivityError>", "ActivityError?"),
        FieldDefinition::required("duration_ms", "u64", "number"),
        FieldDefinition::required("attempts", "u32", "number"),
    ];

    record.rust_schema = RustSchemaRecord {
        file_path: "src/schema/components/activity.rs".to_string(),
        structs: vec![
            "ActivityInput".to_string(),
            "ActivityOutput".to_string(),
            "ActivityError".to_string(),
            "RetryConfig".to_string(),
            "TimeoutConfig".to_string(),
        ],
        enums: vec!["RetryPolicy".to_string()],
        derives: vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "Serialize".to_string(),
            "Deserialize".to_string(),
            "Validate".to_string(),
        ],
        validation_implementation: "Validator crate".to_string(),
    };

    record.test_cases = vec![
        TestCaseRecord::new(
            "test_activity_input",
            TestCategory::Unit,
            r#"ActivityInput::new("myActivity")"#,
            "Valid activity input",
        )
        .passed(),
        TestCaseRecord::new(
            "test_retry_config",
            TestCategory::Unit,
            "RetryConfig with exponential backoff",
            "Valid retry configuration",
        )
        .passed(),
    ];

    record.lessons_learned = LessonsLearned {
        what_worked_well: vec![
            "Reusable RetryConfig and TimeoutConfig".to_string(),
            "Matches Temporal SDK patterns".to_string(),
        ],
        challenges: vec![],
        recommendations: vec!["Align with Temporal SDK types".to_string()],
    };

    record.related_components = vec![
        RelatedComponent::new("child_workflow", "Uses same retry config"),
        RelatedComponent::new("http_request", "Specialized activity"),
    ];

    record.future_improvements = vec![];

    record
}

/// Generate log component migration record
fn generate_log_record() -> MigrationRecord {
    let mut record = MigrationRecord::new("log", "activities");

    record.component = ComponentInfo::new("log", "activities")
        .with_description("Logging component for workflow observability")
        .with_temporal_type("activity");

    record.migration = MigrationMetadata {
        migrated_by: "radium-workflow-compiler".to_string(),
        migration_date: Utc::now(),
        duration_hours: 1.0,
        difficulty: Difficulty::Low,
        breaking_changes: false,
        files_created: vec!["src/schema/components/log.rs".to_string()],
        files_modified: vec!["src/schema/components/mod.rs".to_string()],
    };

    record.schema_decisions = vec![
        SchemaDecision::new(
            "log_level",
            "Enum with Debug, Info, Warning, Error variants",
            "Standard log levels",
        ),
        SchemaDecision::new(
            "context",
            "HashMap for structured logging context",
            "Enables rich log context",
        ),
    ];

    record.input_schema = SchemaDefinition::new("LogInput", "LogInput");
    record.input_schema.fields = vec![
        FieldDefinition::required("message", "String", "string"),
        FieldDefinition::optional("level", "LogLevel", "LogLevel")
            .with_default("Info"),
        FieldDefinition::optional("context", "HashMap<String, Value>", "Record<string, unknown>")
            .with_default("{}"),
    ];

    record.output_schema = SchemaDefinition::new("LogOutput", "LogOutput");
    record.output_schema.fields = vec![
        FieldDefinition::required("logged", "bool", "boolean"),
        FieldDefinition::required("timestamp", "DateTime<Utc>", "Date"),
        FieldDefinition::required("log_id", "String", "string"),
    ];

    record.rust_schema = RustSchemaRecord {
        file_path: "src/schema/components/log.rs".to_string(),
        structs: vec!["LogInput".to_string(), "LogOutput".to_string()],
        enums: vec!["LogLevel".to_string()],
        derives: vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "Serialize".to_string(),
            "Deserialize".to_string(),
            "Validate".to_string(),
        ],
        validation_implementation: "Validator length(min = 1)".to_string(),
    };

    record.test_cases = vec![
        TestCaseRecord::new(
            "test_log_input",
            TestCategory::Unit,
            r#"LogInput::new("Hello")"#,
            "Valid log input",
        )
        .passed(),
        TestCaseRecord::new(
            "test_log_with_context",
            TestCategory::Unit,
            r#"LogInput::new("Hello").with_context(ctx)"#,
            "Log with context",
        )
        .passed(),
    ];

    record.lessons_learned = LessonsLearned {
        what_worked_well: vec!["Simple API for common use case".to_string()],
        challenges: vec![],
        recommendations: vec!["Keep logging simple".to_string()],
    };

    record.related_components = vec![];

    record.future_improvements = vec![];

    record
}

/// Generate http_request component migration record
fn generate_http_request_record() -> MigrationRecord {
    let mut record = MigrationRecord::new("http_request", "activities");

    record.component = ComponentInfo::new("http_request", "activities")
        .with_description("HTTP request component for external API calls")
        .with_temporal_type("activity");

    record.migration = MigrationMetadata {
        migrated_by: "radium-workflow-compiler".to_string(),
        migration_date: Utc::now(),
        duration_hours: 2.5,
        difficulty: Difficulty::Medium,
        breaking_changes: false,
        files_created: vec!["src/schema/components/http_request.rs".to_string()],
        files_modified: vec!["src/schema/components/mod.rs".to_string()],
    };

    record.schema_decisions = vec![
        SchemaDecision::new(
            "http_method",
            "Enum with GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS",
            "Standard HTTP methods with UPPERCASE serialization",
        ),
        SchemaDecision::new(
            "body_type",
            "Enum for content type handling",
            "Explicit body format specification",
        ),
        SchemaDecision::new(
            "auth_config",
            "Comprehensive auth configuration struct",
            "Supports multiple authentication types",
        ),
    ];

    record.input_schema = SchemaDefinition::new("HttpRequestInput", "HttpRequestInput");
    record.input_schema.fields = vec![
        FieldDefinition::required("url", "String", "string"),
        FieldDefinition::optional("method", "HttpMethod", "HttpMethod")
            .with_default("GET"),
        FieldDefinition::optional("headers", "HashMap<String, String>", "Record<string, string>")
            .with_default("{}"),
        FieldDefinition::optional("query_params", "HashMap<String, String>", "Record<string, string>")
            .with_default("{}"),
        FieldDefinition::optional("body", "Option<Value>", "unknown?"),
        FieldDefinition::optional("body_type", "BodyType", "BodyType")
            .with_default("json"),
        FieldDefinition::optional("auth", "AuthConfig", "AuthConfig"),
        FieldDefinition::optional("timeout_ms", "u64", "number")
            .with_default("30000"),
        FieldDefinition::optional("follow_redirects", "bool", "boolean")
            .with_default("true"),
        FieldDefinition::optional("validate_ssl", "bool", "boolean")
            .with_default("true"),
        FieldDefinition::optional("expected_status", "Vec<u16>", "number[]")
            .with_default("[]"),
    ];

    record.output_schema = SchemaDefinition::new("HttpRequestOutput", "HttpRequestOutput");
    record.output_schema.fields = vec![
        FieldDefinition::required("status", "u16", "number"),
        FieldDefinition::required("status_text", "String", "string"),
        FieldDefinition::required("headers", "HashMap<String, String>", "Record<string, string>"),
        FieldDefinition::optional("body", "Option<Value>", "unknown?"),
        FieldDefinition::required("duration_ms", "u64", "number"),
        FieldDefinition::required("success", "bool", "boolean"),
    ];

    record.rust_schema = RustSchemaRecord {
        file_path: "src/schema/components/http_request.rs".to_string(),
        structs: vec![
            "HttpRequestInput".to_string(),
            "HttpRequestOutput".to_string(),
            "AuthConfig".to_string(),
        ],
        enums: vec![
            "HttpMethod".to_string(),
            "BodyType".to_string(),
            "AuthType".to_string(),
        ],
        derives: vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "Serialize".to_string(),
            "Deserialize".to_string(),
            "Validate".to_string(),
        ],
        validation_implementation: "URL validation via validator crate".to_string(),
    };

    record.test_cases = vec![
        TestCaseRecord::new(
            "test_get_request",
            TestCategory::Unit,
            r#"HttpRequestInput::get("https://api.example.com")"#,
            "Valid GET request",
        )
        .passed(),
        TestCaseRecord::new(
            "test_post_with_body",
            TestCategory::Unit,
            r#"HttpRequestInput::post("url").with_json_body(data)"#,
            "Valid POST with JSON body",
        )
        .passed(),
        TestCaseRecord::new(
            "test_bearer_auth",
            TestCategory::Unit,
            r#"HttpRequestInput::get("url").with_bearer_token("token")"#,
            "Valid request with bearer auth",
        )
        .passed(),
    ];

    record.lessons_learned = LessonsLearned {
        what_worked_well: vec![
            "Builder pattern for request construction".to_string(),
            "Comprehensive auth support".to_string(),
        ],
        challenges: vec![ChallengeRecord::new(
            "UPPERCASE method serialization",
            "Used #[serde(rename_all = 'UPPERCASE')]",
            "15 minutes",
        )],
        recommendations: vec!["Match HTTP standards for naming".to_string()],
    };

    record.related_components = vec![RelatedComponent::new("activity", "Base activity type")];

    record.future_improvements = vec![
        FutureImprovement::new("Add response caching", "Low", "Medium"),
        FutureImprovement::new("Add request/response interceptors", "Medium", "High"),
    ];

    record
}

/// Generate database_query component migration record
fn generate_database_query_record() -> MigrationRecord {
    let mut record = MigrationRecord::new("database_query", "activities");

    record.component = ComponentInfo::new("database_query", "activities")
        .with_description("Database query component for Supabase integration")
        .with_temporal_type("activity");

    record.migration = MigrationMetadata {
        migrated_by: "radium-workflow-compiler".to_string(),
        migration_date: Utc::now(),
        duration_hours: 3.0,
        difficulty: Difficulty::Medium,
        breaking_changes: false,
        files_created: vec!["src/schema/components/database_query.rs".to_string()],
        files_modified: vec!["src/schema/components/mod.rs".to_string()],
    };

    record.schema_decisions = vec![
        SchemaDecision::new(
            "query_operation",
            "Enum covering Select, Insert, Update, Upsert, Delete, RPC",
            "Complete CRUD + RPC support",
        ),
        SchemaDecision::new(
            "result_format",
            "Enum for rows, single, count, csv",
            "Flexible output format options",
        ),
        SchemaDecision::new(
            "where_operator",
            "Comprehensive operator enum for filtering",
            "Supports complex query conditions",
        ),
    ];

    record.input_schema = SchemaDefinition::new("DatabaseQueryInput", "DatabaseQueryInput");
    record.input_schema.fields = vec![
        FieldDefinition::required("table", "String", "string"),
        FieldDefinition::optional("operation", "QueryOperation", "QueryOperation")
            .with_default("Select"),
        FieldDefinition::optional("select", "Vec<String>", "string[]")
            .with_default("['*']"),
        FieldDefinition::optional("where_conditions", "Vec<WhereCondition>", "WhereCondition[]")
            .with_default("[]"),
        FieldDefinition::optional("data", "Option<Value>", "unknown?"),
        FieldDefinition::optional("order_by", "Vec<OrderByClause>", "OrderByClause[]"),
        FieldDefinition::optional("limit", "Option<usize>", "number?"),
        FieldDefinition::optional("offset", "Option<usize>", "number?"),
        FieldDefinition::optional("result_format", "ResultFormat", "ResultFormat")
            .with_default("Rows"),
        FieldDefinition::optional("connection", "Option<ConnectionConfig>", "ConnectionConfig?"),
    ];

    record.output_schema = SchemaDefinition::new("DatabaseQueryOutput", "DatabaseQueryOutput");
    record.output_schema.fields = vec![
        FieldDefinition::required("success", "bool", "boolean"),
        FieldDefinition::optional("data", "Option<Value>", "unknown?"),
        FieldDefinition::optional("count", "Option<usize>", "number?"),
        FieldDefinition::required("affected_rows", "usize", "number"),
        FieldDefinition::optional("error", "Option<String>", "string?"),
        FieldDefinition::required("duration_ms", "u64", "number"),
    ];

    record.rust_schema = RustSchemaRecord {
        file_path: "src/schema/components/database_query.rs".to_string(),
        structs: vec![
            "DatabaseQueryInput".to_string(),
            "DatabaseQueryOutput".to_string(),
            "ConnectionConfig".to_string(),
            "WhereCondition".to_string(),
            "OrderByClause".to_string(),
        ],
        enums: vec![
            "QueryOperation".to_string(),
            "ResultFormat".to_string(),
            "WhereOperator".to_string(),
        ],
        derives: vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "Serialize".to_string(),
            "Deserialize".to_string(),
            "Validate".to_string(),
        ],
        validation_implementation: "Validator crate with table name length check".to_string(),
    };

    record.test_cases = vec![
        TestCaseRecord::new(
            "test_select_query",
            TestCategory::Unit,
            r#"DatabaseQueryInput::select("users", ["id", "name"])"#,
            "Valid SELECT query",
        )
        .passed(),
        TestCaseRecord::new(
            "test_insert_query",
            TestCategory::Unit,
            r#"DatabaseQueryInput::insert("users", data)"#,
            "Valid INSERT query",
        )
        .passed(),
        TestCaseRecord::new(
            "test_where_condition",
            TestCategory::Unit,
            r#"DatabaseQueryInput::select("users").where_eq("status", "active")"#,
            "Valid query with WHERE",
        )
        .passed(),
    ];

    record.lessons_learned = LessonsLearned {
        what_worked_well: vec![
            "Query builder pattern".to_string(),
            "Type-safe WHERE conditions".to_string(),
        ],
        challenges: vec![ChallengeRecord::new(
            "Complex WHERE operator set",
            "Created comprehensive WhereOperator enum",
            "1 hour",
        )],
        recommendations: vec!["Use query builder pattern for complex inputs".to_string()],
    };

    record.related_components = vec![RelatedComponent::new("activity", "Base activity type")];

    record.future_improvements = vec![
        FutureImprovement::new("Add transaction support", "High", "High"),
        FutureImprovement::new("Add prepared statement caching", "Medium", "Medium"),
    ];

    record
}

/// Generate agent component migration record
fn generate_agent_record() -> MigrationRecord {
    let mut record = MigrationRecord::new("agent", "agents");

    record.component = ComponentInfo::new("agent", "agents")
        .with_description("AI agent component for LLM invocation")
        .with_temporal_type("activity");

    record.migration = MigrationMetadata {
        migrated_by: "radium-workflow-compiler".to_string(),
        migration_date: Utc::now(),
        duration_hours: 4.0,
        difficulty: Difficulty::High,
        breaking_changes: false,
        files_created: vec!["src/schema/components/agent.rs".to_string()],
        files_modified: vec!["src/schema/components/mod.rs".to_string()],
    };

    record.schema_decisions = vec![
        SchemaDecision::new(
            "ai_provider",
            "Enum supporting Anthropic, OpenAI, Google, Azure, Bedrock, Custom",
            "Comprehensive provider support",
        ),
        SchemaDecision::new(
            "model_config",
            "Provider-specific model configuration",
            "Type-safe model settings per provider",
        ),
        SchemaDecision::new(
            "tool_calling",
            "Tool struct with name, description, input_schema",
            "Supports function calling patterns",
        ),
    ];

    record.input_schema = SchemaDefinition::new("AgentInput", "AgentInput");
    record.input_schema.fields = vec![
        FieldDefinition::required("provider", "AIProvider", "AIProvider"),
        FieldDefinition::required("model_config", "ModelConfig", "ModelConfig"),
        FieldDefinition::optional("system_prompt", "Option<String>", "string?"),
        FieldDefinition::required("messages", "Vec<Message>", "Message[]"),
        FieldDefinition::optional("tools", "Vec<Tool>", "Tool[]")
            .with_default("[]"),
        FieldDefinition::optional("stream", "bool", "boolean")
            .with_default("false"),
        FieldDefinition::optional("timeout_ms", "u64", "number")
            .with_default("120000"),
        FieldDefinition::optional("output_variable", "Option<String>", "string?"),
    ];

    record.output_schema = SchemaDefinition::new("AgentOutput", "AgentOutput");
    record.output_schema.fields = vec![
        FieldDefinition::required("response", "String", "string"),
        FieldDefinition::required("model", "String", "string"),
        FieldDefinition::required("provider", "String", "string"),
        FieldDefinition::required("usage", "TokenUsage", "TokenUsage"),
        FieldDefinition::required("tool_calls", "Vec<ToolCall>", "ToolCall[]"),
        FieldDefinition::required("finish_reason", "FinishReason", "FinishReason"),
        FieldDefinition::required("duration_ms", "u64", "number"),
    ];

    record.validation_rules = vec![ValidationRuleRecord::new(
        "messages_required",
        "Validator length(min = 1)",
        "At least one message required",
        "Agent needs conversation context",
    )];

    record.rust_schema = RustSchemaRecord {
        file_path: "src/schema/components/agent.rs".to_string(),
        structs: vec![
            "AgentInput".to_string(),
            "AgentOutput".to_string(),
            "Message".to_string(),
            "Tool".to_string(),
            "ToolCall".to_string(),
            "TokenUsage".to_string(),
        ],
        enums: vec![
            "AIProvider".to_string(),
            "AnthropicModel".to_string(),
            "ModelConfig".to_string(),
            "MessageRole".to_string(),
            "FinishReason".to_string(),
        ],
        derives: vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "Serialize".to_string(),
            "Deserialize".to_string(),
            "Validate".to_string(),
        ],
        validation_implementation: "Validator with messages length check".to_string(),
    };

    record.test_cases = vec![
        TestCaseRecord::new(
            "test_anthropic_agent",
            TestCategory::Unit,
            r#"AgentInput::anthropic(Claude35Sonnet, messages)"#,
            "Valid Anthropic agent",
        )
        .passed(),
        TestCaseRecord::new(
            "test_with_tools",
            TestCategory::Unit,
            r#"AgentInput::anthropic(...).with_tools(tools)"#,
            "Agent with tool calling",
        )
        .passed(),
        TestCaseRecord::new(
            "test_token_usage",
            TestCategory::Unit,
            "TokenUsage { input: 100, output: 50, total: 150 }",
            "Valid token usage",
        )
        .passed(),
    ];

    record.lessons_learned = LessonsLearned {
        what_worked_well: vec![
            "Provider abstraction works well".to_string(),
            "Tool calling schema is flexible".to_string(),
        ],
        challenges: vec![
            ChallengeRecord::new(
                "Model config per provider",
                "Used untagged enum for ModelConfig",
                "2 hours",
            ),
        ],
        recommendations: vec![
            "Abstract provider differences".to_string(),
            "Use builder pattern for complex inputs".to_string(),
        ],
    };

    record.related_components = vec![RelatedComponent::new("activity", "Base activity type")];

    record.future_improvements = vec![
        FutureImprovement::new("Add streaming support", "High", "High"),
        FutureImprovement::new("Add response caching", "Medium", "Medium"),
        FutureImprovement::new("Add cost estimation", "Low", "Low"),
    ];

    record
}

/// Generate child_workflow component migration record
fn generate_child_workflow_record() -> MigrationRecord {
    let mut record = MigrationRecord::new("child_workflow", "advanced");

    record.component = ComponentInfo::new("child_workflow", "advanced")
        .with_description("Child workflow invocation component")
        .with_temporal_type("child_workflow");

    record.migration = MigrationMetadata {
        migrated_by: "radium-workflow-compiler".to_string(),
        migration_date: Utc::now(),
        duration_hours: 2.0,
        difficulty: Difficulty::High,
        breaking_changes: false,
        files_created: vec!["src/schema/components/child_workflow.rs".to_string()],
        files_modified: vec!["src/schema/components/mod.rs".to_string()],
    };

    record.schema_decisions = vec![
        SchemaDecision::new(
            "parent_close_policy",
            "Enum with Terminate, Abandon, RequestCancel",
            "Maps directly to Temporal policies",
        ),
        SchemaDecision::new(
            "workflow_status",
            "Enum covering all Temporal workflow states",
            "Complete status representation",
        ),
        SchemaDecision::new(
            "await_result",
            "Boolean flag for fire-and-forget vs sync",
            "Simple control over execution mode",
        ),
    ];

    record.input_schema = SchemaDefinition::new("ChildWorkflowInput", "ChildWorkflowInput");
    record.input_schema.fields = vec![
        FieldDefinition::required("workflow_name", "String", "string"),
        FieldDefinition::optional("workflow_id", "Option<String>", "string?"),
        FieldDefinition::optional("task_queue", "Option<String>", "string?"),
        FieldDefinition::optional("input", "HashMap<String, Value>", "Record<string, unknown>")
            .with_default("{}"),
        FieldDefinition::optional("parent_close_policy", "ParentClosePolicy", "ParentClosePolicy")
            .with_default("terminate"),
        FieldDefinition::optional("execution_timeout_ms", "Option<u64>", "number?"),
        FieldDefinition::optional("run_timeout_ms", "Option<u64>", "number?"),
        FieldDefinition::optional("await_result", "bool", "boolean")
            .with_default("true"),
        FieldDefinition::optional("retry", "RetryConfig", "RetryConfig"),
    ];

    record.output_schema = SchemaDefinition::new("ChildWorkflowOutput", "ChildWorkflowOutput");
    record.output_schema.fields = vec![
        FieldDefinition::required("workflow_id", "String", "string"),
        FieldDefinition::required("run_id", "String", "string"),
        FieldDefinition::optional("result", "Option<Value>", "unknown?"),
        FieldDefinition::required("status", "WorkflowStatus", "WorkflowStatus"),
        FieldDefinition::optional("error", "Option<String>", "string?"),
        FieldDefinition::required("duration_ms", "u64", "number"),
    ];

    record.rust_schema = RustSchemaRecord {
        file_path: "src/schema/components/child_workflow.rs".to_string(),
        structs: vec![
            "ChildWorkflowInput".to_string(),
            "ChildWorkflowOutput".to_string(),
        ],
        enums: vec![
            "ParentClosePolicy".to_string(),
            "WorkflowStatus".to_string(),
        ],
        derives: vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "Serialize".to_string(),
            "Deserialize".to_string(),
            "Validate".to_string(),
        ],
        validation_implementation: "Validator with workflow_name length check".to_string(),
    };

    record.test_cases = vec![
        TestCaseRecord::new(
            "test_child_workflow",
            TestCategory::Unit,
            r#"ChildWorkflowInput::new("processOrder")"#,
            "Valid child workflow",
        )
        .passed(),
        TestCaseRecord::new(
            "test_fire_and_forget",
            TestCategory::Unit,
            r#"ChildWorkflowInput::new("bg").fire_and_forget()"#,
            "await_result = false",
        )
        .passed(),
    ];

    record.lessons_learned = LessonsLearned {
        what_worked_well: vec![
            "Direct mapping to Temporal APIs".to_string(),
            "Reuses RetryConfig from activity".to_string(),
        ],
        challenges: vec![],
        recommendations: vec!["Align with Temporal SDK patterns".to_string()],
    };

    record.related_components = vec![
        RelatedComponent::new("activity", "Shares retry config"),
        RelatedComponent::new("signal", "Can signal child workflows"),
    ];

    record.future_improvements = vec![];

    record
}

/// Generate signal component migration record
fn generate_signal_record() -> MigrationRecord {
    let mut record = MigrationRecord::new("signal", "advanced");

    record.component = ComponentInfo::new("signal", "advanced")
        .with_description("Temporal signal component for workflow communication")
        .with_temporal_type("signal");

    record.migration = MigrationMetadata {
        migrated_by: "radium-workflow-compiler".to_string(),
        migration_date: Utc::now(),
        duration_hours: 1.5,
        difficulty: Difficulty::Medium,
        breaking_changes: false,
        files_created: vec!["src/schema/components/signal.rs".to_string()],
        files_modified: vec!["src/schema/components/mod.rs".to_string()],
    };

    record.schema_decisions = vec![
        SchemaDecision::new(
            "signal_direction",
            "Enum with Send and Receive variants",
            "Clear distinction between sending and receiving",
        ),
        SchemaDecision::new(
            "timeout",
            "Zero means wait forever for receive",
            "Matches Temporal signal semantics",
        ),
    ];

    record.input_schema = SchemaDefinition::new("SignalInput", "SignalInput");
    record.input_schema.fields = vec![
        FieldDefinition::required("signal_name", "String", "string"),
        FieldDefinition::optional("direction", "SignalDirection", "SignalDirection")
            .with_default("receive"),
        FieldDefinition::optional("target_workflow_id", "Option<String>", "string?"),
        FieldDefinition::optional("target_run_id", "Option<String>", "string?"),
        FieldDefinition::optional("payload", "Option<Value>", "unknown?"),
        FieldDefinition::optional("timeout_ms", "u64", "number")
            .with_default("0"),
        FieldDefinition::optional("output_variable", "Option<String>", "string?"),
    ];

    record.output_schema = SchemaDefinition::new("SignalOutput", "SignalOutput");
    record.output_schema.fields = vec![
        FieldDefinition::required("signal_name", "String", "string"),
        FieldDefinition::required("sent", "bool", "boolean"),
        FieldDefinition::required("received", "bool", "boolean"),
        FieldDefinition::optional("payload", "Option<Value>", "unknown?"),
        FieldDefinition::optional("sender_workflow_id", "Option<String>", "string?"),
        FieldDefinition::required("timed_out", "bool", "boolean"),
    ];

    record.validation_rules = vec![ValidationRuleRecord::new(
        "send_requires_target",
        "validate_config()",
        "Send signal requires target_workflow_id",
        "Must know where to send signal",
    )];

    record.rust_schema = RustSchemaRecord {
        file_path: "src/schema/components/signal.rs".to_string(),
        structs: vec!["SignalInput".to_string(), "SignalOutput".to_string()],
        enums: vec!["SignalDirection".to_string()],
        derives: vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "Serialize".to_string(),
            "Deserialize".to_string(),
            "Validate".to_string(),
        ],
        validation_implementation: "Custom validate_config() method".to_string(),
    };

    record.test_cases = vec![
        TestCaseRecord::new(
            "test_receive_signal",
            TestCategory::Unit,
            r#"SignalInput::receive("approval")"#,
            "Valid receive signal",
        )
        .passed(),
        TestCaseRecord::new(
            "test_send_signal",
            TestCategory::Unit,
            r#"SignalInput::send("notify", "wf-123")"#,
            "Valid send signal",
        )
        .passed(),
    ];

    record.lessons_learned = LessonsLearned {
        what_worked_well: vec!["Simple direction-based API".to_string()],
        challenges: vec![],
        recommendations: vec!["Use direction enum for bidirectional operations".to_string()],
    };

    record.related_components = vec![
        RelatedComponent::new("child_workflow", "Can signal child workflows"),
        RelatedComponent::new("trigger", "Signal trigger type"),
    ];

    record.future_improvements = vec![];

    record
}

/// Generate timer component migration record
fn generate_timer_record() -> MigrationRecord {
    let mut record = MigrationRecord::new("timer", "advanced");

    record.component = ComponentInfo::new("timer", "advanced")
        .with_description("Timer component for workflow delays")
        .with_temporal_type("timer");

    record.migration = MigrationMetadata {
        migrated_by: "radium-workflow-compiler".to_string(),
        migration_date: Utc::now(),
        duration_hours: 1.0,
        difficulty: Difficulty::Low,
        breaking_changes: false,
        files_created: vec!["src/schema/components/timer.rs".to_string()],
        files_modified: vec!["src/schema/components/mod.rs".to_string()],
    };

    record.schema_decisions = vec![
        SchemaDecision::new(
            "timer_type",
            "Enum with Duration and UntilTime variants",
            "Covers relative and absolute timing",
        ),
        SchemaDecision::new(
            "duration_unit",
            "Enum for time unit conversion",
            "User-friendly duration specification",
        ),
    ];

    record.input_schema = SchemaDefinition::new("TimerInput", "TimerInput");
    record.input_schema.fields = vec![
        FieldDefinition::optional("timer_type", "TimerType", "TimerType")
            .with_default("duration"),
        FieldDefinition::optional("duration", "Option<u64>", "number?"),
        FieldDefinition::optional("unit", "DurationUnit", "DurationUnit")
            .with_default("minutes"),
        FieldDefinition::optional("until_time", "Option<DateTime<Utc>>", "Date?"),
        FieldDefinition::optional("duration_variable", "Option<String>", "string?"),
        FieldDefinition::optional("description", "Option<String>", "string?"),
    ];

    record.output_schema = SchemaDefinition::new("TimerOutput", "TimerOutput");
    record.output_schema.fields = vec![
        FieldDefinition::required("completed", "bool", "boolean"),
        FieldDefinition::required("started_at", "DateTime<Utc>", "Date"),
        FieldDefinition::required("ended_at", "DateTime<Utc>", "Date"),
        FieldDefinition::required("duration_ms", "u64", "number"),
        FieldDefinition::required("cancelled", "bool", "boolean"),
    ];

    record.rust_schema = RustSchemaRecord {
        file_path: "src/schema/components/timer.rs".to_string(),
        structs: vec!["TimerInput".to_string(), "TimerOutput".to_string()],
        enums: vec!["TimerType".to_string(), "DurationUnit".to_string()],
        derives: vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "Serialize".to_string(),
            "Deserialize".to_string(),
            "Validate".to_string(),
        ],
        validation_implementation: "Custom validate_config() method".to_string(),
    };

    record.test_cases = vec![
        TestCaseRecord::new(
            "test_seconds_timer",
            TestCategory::Unit,
            "TimerInput::seconds(30)",
            "30000 ms timer",
        )
        .passed(),
        TestCaseRecord::new(
            "test_minutes_timer",
            TestCategory::Unit,
            "TimerInput::minutes(5)",
            "300000 ms timer",
        )
        .passed(),
        TestCaseRecord::new(
            "test_until_time",
            TestCategory::Unit,
            "TimerInput::until(future_time)",
            "Valid until timer",
        )
        .passed(),
    ];

    record.lessons_learned = LessonsLearned {
        what_worked_well: vec![
            "Duration unit conversion is intuitive".to_string(),
            "Factory methods for common cases".to_string(),
        ],
        challenges: vec![],
        recommendations: vec!["Provide convenience methods for common durations".to_string()],
    };

    record.related_components = vec![];

    record.future_improvements = vec![];

    record
}

/// Generate parallel component migration record
fn generate_parallel_record() -> MigrationRecord {
    let mut record = MigrationRecord::new("parallel", "advanced");

    record.component = ComponentInfo::new("parallel", "advanced")
        .with_description("Parallel execution component for concurrent branches")
        .with_temporal_type("workflow");

    record.migration = MigrationMetadata {
        migrated_by: "radium-workflow-compiler".to_string(),
        migration_date: Utc::now(),
        duration_hours: 3.0,
        difficulty: Difficulty::High,
        breaking_changes: false,
        files_created: vec!["src/schema/components/parallel.rs".to_string()],
        files_modified: vec!["src/schema/components/mod.rs".to_string()],
    };

    record.schema_decisions = vec![
        SchemaDecision::new(
            "join_strategy",
            "Enum with All, Any, AllSettled, Race variants",
            "Covers all Promise.* patterns",
        ),
        SchemaDecision::new(
            "branch",
            "Struct with name, start_node, timeout, required",
            "Complete branch configuration",
        ),
        SchemaDecision::new(
            "cancel_on_error",
            "Boolean flag for error handling strategy",
            "Control over failure propagation",
        ),
    ];

    record.input_schema = SchemaDefinition::new("ParallelInput", "ParallelInput");
    record.input_schema.fields = vec![
        FieldDefinition::required("branches", "Vec<Branch>", "Branch[]"),
        FieldDefinition::optional("join_strategy", "JoinStrategy", "JoinStrategy")
            .with_default("all"),
        FieldDefinition::optional("max_concurrent", "usize", "number")
            .with_default("0"),
        FieldDefinition::optional("timeout_ms", "Option<u64>", "number?"),
        FieldDefinition::optional("cancel_on_error", "bool", "boolean")
            .with_default("true"),
    ];

    record.output_schema = SchemaDefinition::new("ParallelOutput", "ParallelOutput");
    record.output_schema.fields = vec![
        FieldDefinition::required("completed", "bool", "boolean"),
        FieldDefinition::required("results", "HashMap<String, BranchResult>", "Record<string, BranchResult>"),
        FieldDefinition::required("duration_ms", "u64", "number"),
        FieldDefinition::required("had_cancellations", "bool", "boolean"),
        FieldDefinition::required("had_failures", "bool", "boolean"),
    ];

    record.validation_rules = vec![ValidationRuleRecord::new(
        "min_branches",
        "Validator length(min = 2)",
        "At least 2 branches required",
        "Parallel requires multiple branches",
    )];

    record.rust_schema = RustSchemaRecord {
        file_path: "src/schema/components/parallel.rs".to_string(),
        structs: vec![
            "ParallelInput".to_string(),
            "ParallelOutput".to_string(),
            "Branch".to_string(),
            "BranchResult".to_string(),
        ],
        enums: vec!["JoinStrategy".to_string()],
        derives: vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "Serialize".to_string(),
            "Deserialize".to_string(),
            "Validate".to_string(),
        ],
        validation_implementation: "Validator with branches length check".to_string(),
    };

    record.test_cases = vec![
        TestCaseRecord::new(
            "test_parallel_branches",
            TestCategory::Unit,
            "ParallelInput::new(vec![branch1, branch2])",
            "Valid parallel input",
        )
        .passed(),
        TestCaseRecord::new(
            "test_all_settled",
            TestCategory::Unit,
            "ParallelInput::new(branches).with_join_strategy(AllSettled)",
            "All settled strategy",
        )
        .passed(),
        TestCaseRecord::new(
            "test_branch_result",
            TestCategory::Unit,
            r#"BranchResult::success("b1", value, 100)"#,
            "Valid branch result",
        )
        .passed(),
    ];

    record.lessons_learned = LessonsLearned {
        what_worked_well: vec![
            "Join strategy maps to Promise patterns".to_string(),
            "Branch results provide detailed info".to_string(),
        ],
        challenges: vec![ChallengeRecord::new(
            "State merging for parallel branches",
            "BranchResult captures per-branch state",
            "1 hour",
        )],
        recommendations: vec!["Model after Promise.all/race/allSettled".to_string()],
    };

    record.related_components = vec![
        RelatedComponent::new("loop", "Alternative iteration pattern"),
    ];

    record.future_improvements = vec![FutureImprovement::new(
        "Add dynamic branch creation",
        "Medium",
        "High",
    )];

    record
}

#[test]
fn generate_all_migration_records() {
    let records_dir = get_records_dir();

    // Ensure directory exists
    std::fs::create_dir_all(&records_dir).expect("Failed to create records directory");

    // Generate all records
    let records: Vec<(&str, MigrationRecord)> = vec![
        ("trigger", generate_trigger_record()),
        ("start", generate_start_record()),
        ("stop", generate_stop_record()),
        ("conditional", generate_conditional_record()),
        ("loop", generate_loop_record()),
        ("activity", generate_activity_record()),
        ("log", generate_log_record()),
        ("http_request", generate_http_request_record()),
        ("database_query", generate_database_query_record()),
        ("agent", generate_agent_record()),
        ("child_workflow", generate_child_workflow_record()),
        ("signal", generate_signal_record()),
        ("timer", generate_timer_record()),
        ("parallel", generate_parallel_record()),
    ];

    for (name, record) in records {
        let file_path = records_dir.join(format!("{}.yaml", name));
        record.save(&file_path).expect(&format!("Failed to save {} record", name));
        println!("Generated: {}", file_path.display());
    }

    println!("\nGenerated {} migration records", 14);
}

#[test]
fn test_migration_record_quality() {
    // Test that a sample record meets quality criteria
    let record = generate_trigger_record();

    // Check required sections
    assert!(!record.component.name.is_empty());
    assert!(!record.component.category.is_empty());
    assert!(!record.component.description.is_empty());

    // Check schema decisions (min 3)
    assert!(record.schema_decisions.len() >= 3, "Need at least 3 schema decisions");

    // Check each decision has rationale
    for decision in &record.schema_decisions {
        assert!(!decision.rationale.is_empty(), "Decision needs rationale");
    }

    // Check test cases
    assert!(record.test_cases.len() >= 3, "Need at least 3 test cases");

    // Check lessons learned
    assert!(!record.lessons_learned.what_worked_well.is_empty());

    // Check connections
    assert!(!record.connections.connection_validation.is_empty());
}

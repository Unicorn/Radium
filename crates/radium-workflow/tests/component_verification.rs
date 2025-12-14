//! Component Verification Suite
//!
//! Comprehensive tests to verify all components meet migration requirements:
//! - Schema compilation
//! - Serialization/deserialization
//! - Validation rules
//! - TypeScript code generation
//! - Migration record quality

use radium_workflow::migration::{
    Difficulty, MigrationRecord,
};
use radium_workflow::schema::components::*;
use std::collections::HashMap;
use std::path::PathBuf;
use validator::Validate;

fn get_records_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("component-records")
}

// =============================================================================
// CONTROL FLOW COMPONENT TESTS
// =============================================================================

mod trigger_tests {
    use super::*;

    #[test]
    fn test_trigger_type_default() {
        let trigger = TriggerInput::default();
        assert_eq!(trigger.trigger_type, TriggerType::Manual);
    }

    #[test]
    fn test_trigger_schedule_cron() {
        let trigger = TriggerInput::scheduled_cron("0 * * * *");
        assert_eq!(trigger.trigger_type, TriggerType::Schedule);
        assert!(trigger.schedule.is_some());
        assert!(trigger.validate_config().is_ok());
    }

    #[test]
    fn test_trigger_schedule_interval() {
        let trigger = TriggerInput::scheduled_interval(3600);
        assert_eq!(trigger.trigger_type, TriggerType::Schedule);
        assert!(trigger.schedule.is_some());
        assert!(trigger.validate_config().is_ok());
    }

    #[test]
    fn test_trigger_webhook() {
        let trigger = TriggerInput::webhook();
        assert_eq!(trigger.trigger_type, TriggerType::Webhook);
        assert!(trigger.webhook.is_some());
        assert!(trigger.validate_config().is_ok());
    }

    #[test]
    fn test_trigger_event() {
        let trigger = TriggerInput::event("user.created");
        assert_eq!(trigger.trigger_type, TriggerType::Event);
        assert!(trigger.event_type.is_some());
        assert!(trigger.validate_config().is_ok());
    }

    #[test]
    fn test_trigger_signal() {
        let trigger = TriggerInput::signal("approvalReceived");
        assert_eq!(trigger.trigger_type, TriggerType::Signal);
        assert!(trigger.signal_name.is_some());
        assert!(trigger.validate_config().is_ok());
    }

    #[test]
    fn test_trigger_validation_schedule_without_config() {
        let trigger = TriggerInput {
            trigger_type: TriggerType::Schedule,
            schedule: None,
            ..Default::default()
        };
        let result = trigger.validate_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("schedule")));
    }

    #[test]
    fn test_trigger_validation_webhook_without_config() {
        let trigger = TriggerInput {
            trigger_type: TriggerType::Webhook,
            webhook: None,
            ..Default::default()
        };
        let result = trigger.validate_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_trigger_serialization() {
        let trigger = TriggerInput::scheduled_cron("0 0 * * *");
        let json = serde_json::to_string(&trigger).unwrap();
        assert!(json.contains("triggerType"));
        assert!(json.contains("schedule"));

        let deserialized: TriggerInput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.trigger_type, TriggerType::Schedule);
    }

    #[test]
    fn test_trigger_output() {
        let output = TriggerOutput::new("trig-123", serde_json::json!({"key": "value"}));
        assert!(output.triggered);
        assert_eq!(output.trigger_id, "trig-123");
    }
}

mod conditional_tests {
    use super::*;

    #[test]
    fn test_comparison_operators_typescript() {
        let condition = Condition::equals("status", serde_json::json!("active"));
        let ts = condition.to_typescript();
        assert!(ts.contains("==="));
        assert!(ts.contains("'active'"));
    }

    #[test]
    fn test_compound_condition() {
        let c1 = Condition::greater_than("age", serde_json::json!(18));
        let c2 = Condition::equals("verified", serde_json::json!(true));

        let group = ConditionGroup::Compound {
            operator: LogicalOperator::And,
            conditions: vec![ConditionGroup::Single(c1), ConditionGroup::Single(c2)],
        };

        let ts = group.to_typescript();
        assert!(ts.contains("&&"));
    }

    #[test]
    fn test_null_operators() {
        let condition = Condition::is_null("value");
        let ts = condition.to_typescript();
        assert!(ts.contains("=== null"));

        let condition = Condition::is_not_null("value");
        let ts = condition.to_typescript();
        assert!(ts.contains("!== null"));
    }

    #[test]
    fn test_string_operators() {
        let condition = Condition::new("name", ComparisonOperator::Contains, Some(serde_json::json!("test")));
        let ts = condition.to_typescript();
        assert!(ts.contains(".includes("));

        let condition = Condition::new("name", ComparisonOperator::StartsWith, Some(serde_json::json!("pre")));
        let ts = condition.to_typescript();
        assert!(ts.contains(".startsWith("));

        let condition = Condition::new("name", ComparisonOperator::EndsWith, Some(serde_json::json!("fix")));
        let ts = condition.to_typescript();
        assert!(ts.contains(".endsWith("));
    }

    #[test]
    fn test_conditional_serialization() {
        let condition = ConditionalInput {
            condition: ConditionGroup::Single(Condition::new(
                "x",
                ComparisonOperator::GreaterOrEqual,
                Some(serde_json::json!(10)),
            )),
            true_label: "Yes".to_string(),
            false_label: "No".to_string(),
        };

        let json = serde_json::to_string(&condition).unwrap();
        assert!(json.contains("condition"));
        assert!(json.contains("trueLabel"));
    }
}

mod loop_tests {
    use super::*;

    #[test]
    fn test_for_each_loop() {
        let loop_input = LoopInput::for_each("items");
        assert_eq!(loop_input.loop_type, LoopType::ForEach);
        assert!(loop_input.validate_config().is_ok());
    }

    #[test]
    fn test_count_loop() {
        let loop_input = LoopInput::count(10);
        assert_eq!(loop_input.loop_type, LoopType::Count);
        assert_eq!(loop_input.count, Some(10));
        assert!(loop_input.validate_config().is_ok());
    }

    #[test]
    fn test_while_loop() {
        let loop_input = LoopInput::while_loop("status !== 'done'");
        assert_eq!(loop_input.loop_type, LoopType::While);
        assert!(loop_input.validate_config().is_ok());
    }

    #[test]
    fn test_batch_loop() {
        let loop_input = LoopInput::batch("items", BatchConfig::new(100));
        assert_eq!(loop_input.loop_type, LoopType::Batch);
        assert!(loop_input.batch_config.is_some());
        assert!(loop_input.validate_config().is_ok());
    }

    #[test]
    fn test_loop_validation_foreach_no_items() {
        let loop_input = LoopInput {
            loop_type: LoopType::ForEach,
            items: None,
            ..Default::default()
        };
        let result = loop_input.validate_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_loop_validation_count_no_count() {
        let loop_input = LoopInput {
            loop_type: LoopType::Count,
            count: None,
            ..Default::default()
        };
        let result = loop_input.validate_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_loop_validation_threshold() {
        let loop_input = LoopInput {
            loop_type: LoopType::Count,
            count: Some(100),
            max_iterations: 500,
            continue_as_new_threshold: 1000, // Greater than max!
            ..Default::default()
        };
        let result = loop_input.validate_config();
        assert!(result.is_err());
    }
}

// =============================================================================
// ACTIVITY COMPONENT TESTS
// =============================================================================

mod activity_tests {
    use super::*;

    #[test]
    fn test_activity_input_basic() {
        let activity = ActivityInput::new("processOrder");
        assert_eq!(activity.activity_name, "processOrder");
        assert!(activity.await_result);
    }

    #[test]
    fn test_activity_with_params() {
        let activity = ActivityInput::new("sendEmail")
            .with_param("to", serde_json::json!("user@example.com"))
            .with_param("subject", serde_json::json!("Hello"));
        assert_eq!(activity.params.len(), 2);
    }

    #[test]
    fn test_retry_config() {
        let retry = RetryConfig::new()
            .with_max_attempts(5)
            .with_initial_interval(2000)
            .with_backoff_coefficient(1.5);

        assert_eq!(retry.max_attempts, 5);
        assert_eq!(retry.initial_interval_ms, 2000);
    }

    #[test]
    fn test_timeout_config() {
        let timeout = TimeoutConfig::new()
            .with_start_to_close(60000)
            .with_heartbeat(5000);

        assert_eq!(timeout.start_to_close_ms, 60000);
        assert_eq!(timeout.heartbeat_ms, Some(5000));
    }

    #[test]
    fn test_activity_error() {
        let error =
            ActivityError::new("TIMEOUT", "Activity timed out after 30s", true).with_details(
                serde_json::json!({
                    "timeout_ms": 30000
                }),
            );

        assert!(error.retryable);
        assert!(error.details.is_some());
    }

    #[test]
    fn test_activity_output_success() {
        let output = ActivityOutput::success(serde_json::json!({"result": "ok"}), 1500, 1);
        assert!(output.success);
        assert!(output.error.is_none());
    }

    #[test]
    fn test_activity_output_failure() {
        let error = ActivityError::new("ERROR", "Failed", false);
        let output = ActivityOutput::failure(error, 1500, 3);
        assert!(!output.success);
        assert!(output.error.is_some());
        assert_eq!(output.attempts, 3);
    }
}

mod http_request_tests {
    use super::*;

    #[test]
    fn test_http_get() {
        let request = HttpRequestInput::get("https://api.example.com/users");
        assert_eq!(request.method, HttpMethod::Get);
        assert_eq!(request.url, "https://api.example.com/users");
    }

    #[test]
    fn test_http_post_with_body() {
        let request =
            HttpRequestInput::post("https://api.example.com/users").with_json_body(serde_json::json!({
                "name": "John",
                "email": "john@example.com"
            }));

        assert_eq!(request.method, HttpMethod::Post);
        assert!(request.body.is_some());
        assert_eq!(request.body_type, BodyType::Json);
    }

    #[test]
    fn test_http_with_headers() {
        let request = HttpRequestInput::get("https://api.example.com")
            .with_header("Content-Type", "application/json")
            .with_header("X-API-Key", "secret");

        assert_eq!(request.headers.len(), 2);
    }

    #[test]
    fn test_http_bearer_auth() {
        let request =
            HttpRequestInput::get("https://api.example.com").with_auth(AuthConfig::bearer("my-token"));

        assert_eq!(request.auth.auth_type, AuthType::Bearer);
        assert_eq!(request.auth.token, Some("my-token".to_string()));
    }

    #[test]
    fn test_http_basic_auth() {
        let request =
            HttpRequestInput::get("https://api.example.com").with_auth(AuthConfig::basic("user", "pass"));

        assert_eq!(request.auth.auth_type, AuthType::Basic);
    }

    #[test]
    fn test_http_method_serialization() {
        let request = HttpRequestInput::delete("https://api.example.com/users/1");
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("DELETE"));
    }
}

mod database_query_tests {
    use super::*;

    #[test]
    fn test_select_query() {
        let query = DatabaseQueryInput::select("users").columns(vec!["id", "name", "email"]);
        assert_eq!(query.table, Some("users".to_string()));
        assert_eq!(query.operation, QueryOperation::Select);
        assert_eq!(query.columns.len(), 3);
    }

    #[test]
    fn test_insert_query() {
        let query = DatabaseQueryInput::insert(
            "users",
            serde_json::json!({
                "name": "John",
                "email": "john@example.com"
            }),
        );

        assert_eq!(query.operation, QueryOperation::Insert);
        assert!(query.data.is_some());
    }

    #[test]
    fn test_query_with_where() {
        let query = DatabaseQueryInput::select("users")
            .columns(vec!["*"])
            .where_eq("status", serde_json::json!("active"))
            .where_eq("active", serde_json::json!(true));

        assert_eq!(query.where_conditions.len(), 2);
    }

    #[test]
    fn test_query_with_order_and_limit() {
        let query = DatabaseQueryInput::select("products")
            .columns(vec!["*"])
            .order_by("created_at", false)
            .limit(10)
            .offset(20);

        assert_eq!(query.order_by.len(), 1);
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(20));
    }

    #[test]
    fn test_update_query() {
        let query = DatabaseQueryInput::update(
            "users",
            serde_json::json!({"name": "John"}),
        ).where_eq("id", serde_json::json!(1));

        assert_eq!(query.operation, QueryOperation::Update);
    }

    #[test]
    fn test_raw_sql() {
        let query = DatabaseQueryInput::raw("SELECT * FROM users WHERE id = $1")
            .with_param("id", serde_json::json!(123));

        assert_eq!(query.operation, QueryOperation::Raw);
    }
}

// =============================================================================
// AGENT COMPONENT TESTS
// =============================================================================

mod agent_tests {
    use super::*;

    #[test]
    fn test_agent_basic() {
        let agent = AgentInput::new("What is the weather?");
        assert_eq!(agent.messages.len(), 1);
        assert_eq!(agent.messages[0].role, MessageRole::User);
    }

    #[test]
    fn test_agent_with_system_prompt() {
        let agent = AgentInput::new("Hello")
            .with_system_prompt("You are a helpful assistant")
            .with_model(ModelConfig::claude_sonnet());

        assert!(agent.system_prompt.is_some());
    }

    #[test]
    fn test_agent_with_tools() {
        let tool = Tool::new(
            "get_weather",
            "Get weather for a location",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                }
            }),
        );

        let agent = AgentInput::new("What's the weather in NYC?").add_tool(tool);

        assert_eq!(agent.tools.len(), 1);
    }

    #[test]
    fn test_model_configs() {
        let sonnet = ModelConfig::claude_sonnet();
        assert_eq!(sonnet.provider(), AIProvider::Anthropic);

        let opus = ModelConfig::claude_opus();
        assert_eq!(opus.provider(), AIProvider::Anthropic);

        let gpt4 = ModelConfig::gpt4();
        assert_eq!(gpt4.provider(), AIProvider::OpenAI);
    }

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage::new(100, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_agent_output() {
        let output = AgentOutput::success(
            "The weather is sunny",
            "claude-3-5-sonnet",
            "anthropic",
            TokenUsage::new(50, 30),
            1500,
        );

        assert_eq!(output.finish_reason, FinishReason::EndTurn);
    }

    #[test]
    fn test_messages() {
        let system = Message::system("You are a bot");
        let user = Message::user("Hello");
        let assistant = Message::assistant("Hi there!");

        assert_eq!(system.role, MessageRole::System);
        assert_eq!(user.role, MessageRole::User);
        assert_eq!(assistant.role, MessageRole::Assistant);
    }
}

// =============================================================================
// ADVANCED COMPONENT TESTS
// =============================================================================

mod child_workflow_tests {
    use super::*;

    #[test]
    fn test_child_workflow_basic() {
        let child = ChildWorkflowInput::new("processOrder");
        assert_eq!(child.workflow_name, "processOrder");
        assert!(child.await_result);
    }

    #[test]
    fn test_child_workflow_fire_and_forget() {
        let child = ChildWorkflowInput::new("backgroundTask").fire_and_forget();
        assert!(!child.await_result);
    }

    #[test]
    fn test_child_workflow_with_input() {
        let child = ChildWorkflowInput::new("processOrder")
            .with_input("orderId", serde_json::json!("123"))
            .with_input("priority", serde_json::json!("high"));

        assert_eq!(child.input.len(), 2);
    }

    #[test]
    fn test_parent_close_policies() {
        let terminate = ChildWorkflowInput::new("task")
            .with_parent_close_policy(ParentClosePolicy::Terminate);
        assert_eq!(terminate.parent_close_policy, ParentClosePolicy::Terminate);

        let abandon = ChildWorkflowInput::new("task")
            .with_parent_close_policy(ParentClosePolicy::Abandon);
        assert_eq!(abandon.parent_close_policy, ParentClosePolicy::Abandon);
    }

    #[test]
    fn test_workflow_status() {
        assert!(!WorkflowStatus::Running.is_finished());
        assert!(WorkflowStatus::Completed.is_finished());
        assert!(WorkflowStatus::Completed.is_success());
        assert!(!WorkflowStatus::Failed.is_success());
    }
}

mod signal_tests {
    use super::*;

    #[test]
    fn test_receive_signal() {
        let signal = SignalInput::receive("approval");
        assert_eq!(signal.direction, SignalDirection::Receive);
        assert!(signal.validate_config().is_ok());
    }

    #[test]
    fn test_send_signal() {
        let signal = SignalInput::send("notify", "target-workflow-123");
        assert_eq!(signal.direction, SignalDirection::Send);
        assert!(signal.target_workflow_id.is_some());
        assert!(signal.validate_config().is_ok());
    }

    #[test]
    fn test_send_signal_validation() {
        let signal = SignalInput {
            direction: SignalDirection::Send,
            target_workflow_id: None,
            ..SignalInput::receive("test")
        };
        assert!(signal.validate_config().is_err());
    }

    #[test]
    fn test_signal_with_payload() {
        let signal = SignalInput::send("update", "wf-123")
            .with_payload(serde_json::json!({"status": "approved"}));
        assert!(signal.payload.is_some());
    }

    #[test]
    fn test_signal_output() {
        let sent = SignalOutput::sent("notify");
        assert!(sent.sent);
        assert!(!sent.received);

        let received = SignalOutput::received(
            "approval",
            serde_json::json!({"approved": true}),
            Some("sender-123".to_string()),
        );
        assert!(received.received);
        assert!(received.payload.is_some());

        let timeout = SignalOutput::timeout("approval");
        assert!(timeout.timed_out);
    }
}

mod timer_tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_timer_seconds() {
        let timer = TimerInput::seconds(30);
        assert_eq!(timer.duration_ms(), Some(30000));
    }

    #[test]
    fn test_timer_minutes() {
        let timer = TimerInput::minutes(5);
        assert_eq!(timer.duration_ms(), Some(300000));
    }

    #[test]
    fn test_timer_hours() {
        let timer = TimerInput::hours(2);
        assert_eq!(timer.duration_ms(), Some(7200000));
    }

    #[test]
    fn test_timer_until() {
        let target = Utc::now() + Duration::hours(1);
        let timer = TimerInput::until(target);
        assert_eq!(timer.timer_type, TimerType::UntilTime);
        assert!(timer.until_time.is_some());
    }

    #[test]
    fn test_timer_from_variable() {
        let timer = TimerInput::from_variable("delayMs");
        assert!(timer.duration_variable.is_some());
    }

    #[test]
    fn test_timer_validation() {
        let valid = TimerInput::seconds(10);
        assert!(valid.validate_config().is_ok());

        let invalid = TimerInput {
            timer_type: TimerType::Duration,
            duration: None,
            duration_variable: None,
            ..Default::default()
        };
        assert!(invalid.validate_config().is_err());
    }

    #[test]
    fn test_duration_unit_conversion() {
        assert_eq!(DurationUnit::Milliseconds.to_milliseconds(1000), 1000);
        assert_eq!(DurationUnit::Seconds.to_milliseconds(1), 1000);
        assert_eq!(DurationUnit::Minutes.to_milliseconds(1), 60000);
        assert_eq!(DurationUnit::Hours.to_milliseconds(1), 3600000);
        assert_eq!(DurationUnit::Days.to_milliseconds(1), 86400000);
    }
}

mod parallel_tests {
    use super::*;

    #[test]
    fn test_parallel_basic() {
        let parallel = ParallelInput::new(vec![
            Branch::new("fetch", "fetch-node"),
            Branch::new("process", "process-node"),
        ]);

        assert_eq!(parallel.branches.len(), 2);
        assert_eq!(parallel.join_strategy, JoinStrategy::All);
    }

    #[test]
    fn test_parallel_all_settled() {
        let parallel = ParallelInput::new(vec![
            Branch::new("a", "node-a"),
            Branch::new("b", "node-b"),
        ])
        .with_join_strategy(JoinStrategy::AllSettled);

        assert_eq!(parallel.join_strategy, JoinStrategy::AllSettled);
    }

    #[test]
    fn test_parallel_race() {
        let parallel = ParallelInput::new(vec![
            Branch::new("a", "node-a"),
            Branch::new("b", "node-b"),
        ])
        .with_join_strategy(JoinStrategy::Race);

        assert_eq!(parallel.join_strategy, JoinStrategy::Race);
    }

    #[test]
    fn test_branch_with_timeout() {
        let branch = Branch::new("slow-task", "slow-node").with_timeout(30000);
        assert_eq!(branch.timeout_ms, Some(30000));
    }

    #[test]
    fn test_branch_optional() {
        let branch = Branch::new("optional-task", "opt-node").optional();
        assert!(!branch.required);
    }

    #[test]
    fn test_parallel_validation() {
        let valid = ParallelInput::new(vec![
            Branch::new("a", "node-a"),
            Branch::new("b", "node-b"),
        ]);
        assert!(valid.validate().is_ok());

        let invalid = ParallelInput::new(vec![Branch::new("only", "node-only")]);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_branch_results() {
        let success = BranchResult::success("b1", serde_json::json!({"result": "ok"}), 1000);
        assert!(success.success);

        let failure = BranchResult::failure("b2", "Timeout", 5000);
        assert!(!failure.success);
        assert!(failure.error.is_some());

        let cancelled = BranchResult::cancelled("b3", 500);
        assert!(cancelled.cancelled);
    }

    #[test]
    fn test_parallel_output() {
        let mut results = HashMap::new();
        results.insert(
            "b1".to_string(),
            BranchResult::success("b1", serde_json::json!(1), 100),
        );
        results.insert(
            "b2".to_string(),
            BranchResult::success("b2", serde_json::json!(2), 200),
        );

        let output = ParallelOutput::new(results, 200);
        assert!(output.completed);
        assert!(!output.had_failures);
        assert!(!output.had_cancellations);
    }

    #[test]
    fn test_parallel_output_with_failure() {
        let mut results = HashMap::new();
        results.insert(
            "b1".to_string(),
            BranchResult::success("b1", serde_json::json!(1), 100),
        );
        results.insert(
            "b2".to_string(),
            BranchResult::failure("b2", "Error", 50),
        );

        let output = ParallelOutput::new(results, 100);
        assert!(!output.completed);
        assert!(output.had_failures);
        assert_eq!(output.failed_branches().len(), 1);
    }
}

// =============================================================================
// MIGRATION RECORD QUALITY TESTS
// =============================================================================

mod migration_record_quality_tests {
    use super::*;

    fn load_record(name: &str) -> MigrationRecord {
        let path = get_records_dir().join(format!("{}.yaml", name));
        MigrationRecord::load(&path).expect(&format!("Failed to load {} record", name))
    }

    #[test]
    fn test_all_records_exist() {
        let components = [
            "trigger",
            "start",
            "stop",
            "conditional",
            "loop",
            "activity",
            "log",
            "http_request",
            "database_query",
            "agent",
            "child_workflow",
            "signal",
            "timer",
            "parallel",
        ];

        for component in components {
            let path = get_records_dir().join(format!("{}.yaml", component));
            assert!(path.exists(), "Missing record for {}", component);
        }
    }

    #[test]
    fn test_trigger_record_quality() {
        let record = load_record("trigger");

        // Component info
        assert_eq!(record.component.name, "trigger");
        assert_eq!(record.component.category, "control-flow");
        assert!(!record.component.description.is_empty());

        // Schema decisions (min 3)
        assert!(
            record.schema_decisions.len() >= 3,
            "Need at least 3 schema decisions"
        );
        for decision in &record.schema_decisions {
            assert!(!decision.rationale.is_empty(), "Decision needs rationale");
        }

        // Test cases (min 3)
        assert!(
            record.test_cases.len() >= 3,
            "Need at least 3 test cases"
        );

        // Lessons learned
        assert!(!record.lessons_learned.what_worked_well.is_empty());

        // Input/Output schemas
        assert!(!record.input_schema.rust_struct.is_empty());
        assert!(!record.output_schema.rust_struct.is_empty());
    }

    #[test]
    fn test_agent_record_quality() {
        let record = load_record("agent");

        assert_eq!(record.component.name, "agent");
        assert_eq!(record.component.category, "agents");
        assert_eq!(record.migration.difficulty, Difficulty::High);
        assert!(record.schema_decisions.len() >= 3);
    }

    #[test]
    fn test_parallel_record_quality() {
        let record = load_record("parallel");

        assert_eq!(record.component.name, "parallel");
        assert_eq!(record.component.category, "advanced");
        assert!(record.validation_rules.len() >= 1);
    }

    #[test]
    fn test_records_have_rust_schema_info() {
        let components = ["trigger", "conditional", "activity", "agent", "parallel"];

        for component in components {
            let record = load_record(component);

            assert!(
                !record.rust_schema.file_path.is_empty(),
                "{} missing rust file_path",
                component
            );
            assert!(
                !record.rust_schema.structs.is_empty(),
                "{} missing structs",
                component
            );
            assert!(
                !record.rust_schema.derives.is_empty(),
                "{} missing derives",
                component
            );
        }
    }

    #[test]
    fn test_records_have_test_cases() {
        let components = ["trigger", "loop", "http_request", "agent", "signal"];

        for component in components {
            let record = load_record(component);

            assert!(
                !record.test_cases.is_empty(),
                "{} has no test cases",
                component
            );

            // Check test case completeness
            for test in &record.test_cases {
                assert!(!test.name.is_empty(), "Test needs a name");
                assert!(!test.input.is_empty(), "Test needs input");
                assert!(!test.expected_output.is_empty(), "Test needs expected output");
            }
        }
    }
}

// =============================================================================
// SERIALIZATION ROUNDTRIP TESTS
// =============================================================================

mod serialization_tests {
    use super::*;

    #[test]
    fn test_trigger_roundtrip() {
        let trigger = TriggerInput::scheduled_cron("0 * * * *");
        let json = serde_json::to_string(&trigger).unwrap();
        let parsed: TriggerInput = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.trigger_type, TriggerType::Schedule);
    }

    #[test]
    fn test_conditional_roundtrip() {
        let cond = ConditionalInput {
            condition: ConditionGroup::Single(Condition::equals("x", serde_json::json!(10))),
            true_label: "Yes".to_string(),
            false_label: "No".to_string(),
        };
        let json = serde_json::to_string(&cond).unwrap();
        let parsed: ConditionalInput = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.true_label, "Yes");
    }

    #[test]
    fn test_loop_roundtrip() {
        let loop_input = LoopInput::for_each("items");
        let json = serde_json::to_string(&loop_input).unwrap();
        let parsed: LoopInput = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.loop_type, LoopType::ForEach);
    }

    #[test]
    fn test_http_request_roundtrip() {
        let request = HttpRequestInput::post("https://api.example.com")
            .with_json_body(serde_json::json!({"key": "value"}));
        let json = serde_json::to_string(&request).unwrap();
        let parsed: HttpRequestInput = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.method, HttpMethod::Post);
    }

    #[test]
    fn test_parallel_roundtrip() {
        let parallel = ParallelInput::new(vec![
            Branch::new("a", "node-a"),
            Branch::new("b", "node-b"),
        ]);
        let json = serde_json::to_string(&parallel).unwrap();
        let parsed: ParallelInput = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.branches.len(), 2);
    }
}

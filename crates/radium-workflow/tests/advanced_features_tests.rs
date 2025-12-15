//! Comprehensive Tests for Phase 7 Advanced Features
//!
//! Tests all advanced features including:
//! - Child workflow orchestration
//! - Signal handlers
//! - Query handlers
//! - Cancellation scopes
//! - Search attributes
//! - Workflow versioning
//! - Workflow patterns (Saga, Pipeline, MapReduce, ScatterGather)

use radium_workflow::schema::{
    // Child workflow orchestration
    ChildWorkflowOrchestration, WorkflowIdStrategy, WorkflowIdReusePolicy,
    CancellationType, AdvancedSearchAttributeValue, ChildWorkflowHandle, WorkflowExecutionError,

    // Signals
    SignalDefinition, SignalHandler, SignalSchema, SignalSchemaField,
    SignalBuffering, SignalWithHandler, WorkflowSignals,
    SignalHandlerLogic, VariableSource, VariableUpdate,

    // Queries
    QueryDefinition, QuerySchema, QueryHandlerLogic, WorkflowQueries,

    // Cancellation
    CancellationScope, CleanupConfig, CleanupActivity, StateUpdate, WorkflowCancellationHandler,

    // Search attributes
    SearchAttributeDefinition, SearchAttributeType, TypedSearchAttributeValue,
    SearchAttributeUpdate, WorkflowSearchAttributes,

    // Versioning
    VersioningConfig, VersionInfo, VersionBranch, VersionChangePoint,

    // Patterns
    WorkflowPattern,
    SagaDefinition, SagaStep, SagaAction, CompensationBehavior,
    ScatterGatherDefinition, ScatterConfig, GatherConfig, GatherStrategy, ScatterWorker,
    PipelineDefinition, PipelineStage, StageProcessor,
    MapReduceDefinition, MapConfig, ReduceConfig, Mapper, Reducer,
};

// Import from components (not re-exported at schema level)
use radium_workflow::schema::components::{ParentClosePolicy, RetryConfig};

// Import standard functions from advanced module
use radium_workflow::schema::advanced::{standard_queries, standard_search_attributes};

// =============================================================================
// CHILD WORKFLOW ORCHESTRATION TESTS
// =============================================================================

mod child_workflow_tests {
    use super::*;

    #[test]
    fn test_child_workflow_basic_creation() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder");

        assert_eq!(config.workflow_type, "ProcessOrder");
        assert_eq!(config.id_strategy, WorkflowIdStrategy::Uuid);
        assert!(config.await_result);
    }

    #[test]
    fn test_child_workflow_with_explicit_id() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_workflow_id("order-123");

        assert_eq!(config.id_strategy, WorkflowIdStrategy::Explicit);
        assert_eq!(config.workflow_id, Some("order-123".to_string()));
    }

    #[test]
    fn test_child_workflow_with_id_pattern() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_id_pattern("order-{parent_id}-{index}");

        assert_eq!(config.id_strategy, WorkflowIdStrategy::Pattern);
        assert!(config.id_pattern.is_some());
    }

    #[test]
    fn test_child_workflow_parent_suffix_strategy() {
        let config = ChildWorkflowOrchestration::new("SubWorkflow")
            .with_parent_suffix();

        assert_eq!(config.id_strategy, WorkflowIdStrategy::ParentSuffix);

        let generated_id = config.generate_workflow_id("parent-1", 0);
        assert_eq!(generated_id, "parent-1-child-0");
    }

    #[test]
    fn test_child_workflow_with_task_queue() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_task_queue("high-priority");

        assert_eq!(config.task_queue, Some("high-priority".to_string()));
    }

    #[test]
    fn test_child_workflow_with_input() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_input("orderId", serde_json::json!("order-123"))
            .with_input("amount", serde_json::json!(100.0));

        assert_eq!(config.input.len(), 2);
        assert!(config.input.contains_key("orderId"));
    }

    #[test]
    fn test_child_workflow_with_parent_close_policy() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_parent_close_policy(ParentClosePolicy::Abandon);

        assert_eq!(config.parent_close_policy, ParentClosePolicy::Abandon);
    }

    #[test]
    fn test_child_workflow_with_id_reuse_policy() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_id_reuse_policy(WorkflowIdReusePolicy::RejectDuplicate);

        assert_eq!(config.id_reuse_policy, WorkflowIdReusePolicy::RejectDuplicate);
    }

    #[test]
    fn test_child_workflow_with_cancellation_type() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_cancellation_type(CancellationType::TryCancel);

        assert_eq!(config.cancellation_type, CancellationType::TryCancel);
    }

    #[test]
    fn test_child_workflow_fire_and_forget() {
        let config = ChildWorkflowOrchestration::new("BackgroundJob")
            .fire_and_forget();

        assert!(!config.await_result);
    }

    #[test]
    fn test_child_workflow_with_timeouts() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_execution_timeout(300000)
            .with_run_timeout(60000);

        assert_eq!(config.execution_timeout_ms, Some(300000));
        assert_eq!(config.run_timeout_ms, Some(60000));
    }

    #[test]
    fn test_child_workflow_with_retry_policy() {
        let retry = RetryConfig {
            max_attempts: 3,
            initial_interval_ms: 1000,
            max_interval_ms: 60000,
            backoff_coefficient: 2.0,
            non_retryable_errors: vec![],
        };

        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_retry_policy(retry);

        assert!(config.retry_policy.is_some());
        assert_eq!(config.retry_policy.as_ref().unwrap().max_attempts, 3);
    }

    #[test]
    fn test_child_workflow_with_search_attributes() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_search_attribute("CustomerId", AdvancedSearchAttributeValue::String("cust-123".into()));

        assert!(config.search_attributes.contains_key("CustomerId"));
    }

    #[test]
    fn test_child_workflow_with_memo() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_memo("reason", "customer request");

        assert!(config.memo.contains_key("reason"));
    }

    #[test]
    fn test_child_workflow_with_cron_schedule() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_cron_schedule("0 0 * * *");

        assert_eq!(config.cron_schedule, Some("0 0 * * *".to_string()));
    }

    #[test]
    fn test_child_workflow_validate_explicit_requires_id() {
        let config = ChildWorkflowOrchestration {
            id_strategy: WorkflowIdStrategy::Explicit,
            workflow_id: None,
            ..ChildWorkflowOrchestration::new("Test")
        };

        let result = config.validate_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_child_workflow_validate_pattern_requires_pattern() {
        let config = ChildWorkflowOrchestration {
            id_strategy: WorkflowIdStrategy::Pattern,
            id_pattern: None,
            ..ChildWorkflowOrchestration::new("Test")
        };

        let result = config.validate_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_child_workflow_validate_timeout_relationship() {
        let config = ChildWorkflowOrchestration {
            execution_timeout_ms: Some(60000),
            run_timeout_ms: Some(120000),
            ..ChildWorkflowOrchestration::new("Test")
        };

        let result = config.validate_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_child_workflow_to_typescript_basic() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_task_queue("orders");

        let ts = config.to_typescript();

        assert!(ts.contains("executeChild('ProcessOrder'"));
        assert!(ts.contains("taskQueue: 'orders'"));
    }

    #[test]
    fn test_child_workflow_to_typescript_fire_and_forget() {
        let config = ChildWorkflowOrchestration::new("BackgroundJob")
            .fire_and_forget();

        let ts = config.to_typescript();

        assert!(ts.contains("Fire and forget"));
        assert!(!ts.contains("await childHandle.result()"));
    }

    #[test]
    fn test_child_workflow_handle_creation() {
        let handle = ChildWorkflowHandle::new(
            "child-123",
            "run-456",
            "ProcessOrder",
            "parent-789",
            "parent-run-101",
        );

        assert_eq!(handle.workflow_id, "child-123");
        assert_eq!(handle.parent_workflow_id, "parent-789");
    }

    #[test]
    fn test_workflow_execution_error_creation() {
        let error = WorkflowExecutionError::new("NetworkError", "Connection refused")
            .retryable();

        assert!(error.retryable);
        assert_eq!(error.error_type, "NetworkError");
    }

    #[test]
    fn test_workflow_execution_error_with_cause() {
        let cause = WorkflowExecutionError::new("NetworkError", "Connection refused");
        let error = WorkflowExecutionError::new("ProcessingError", "Failed to process")
            .with_cause(cause);

        assert!(error.cause.is_some());
    }

    #[test]
    fn test_child_workflow_serialization_roundtrip() {
        let config = ChildWorkflowOrchestration::new("TestWorkflow")
            .with_task_queue("test-queue")
            .with_input("key", serde_json::json!("value"));

        let json = serde_json::to_string(&config).unwrap();
        let restored: ChildWorkflowOrchestration = serde_json::from_str(&json).unwrap();

        assert_eq!(config.workflow_type, restored.workflow_type);
        assert_eq!(config.task_queue, restored.task_queue);
    }
}

// =============================================================================
// SIGNAL HANDLER TESTS
// =============================================================================

mod signal_handler_tests {
    use super::*;

    #[test]
    fn test_signal_definition_basic() {
        let signal = SignalDefinition::new("approveOrder");

        assert_eq!(signal.name, "approveOrder");
        assert!(signal.external);
        assert_eq!(signal.buffering, SignalBuffering::Ordered);
    }

    #[test]
    fn test_signal_definition_with_description() {
        let signal = SignalDefinition::new("approveOrder")
            .with_description("Approve a pending order");

        assert!(signal.description.is_some());
    }

    #[test]
    fn test_signal_definition_internal_only() {
        let signal = SignalDefinition::new("internalSync")
            .internal_only();

        assert!(!signal.external);
    }

    #[test]
    fn test_signal_definition_with_buffering() {
        let signal = SignalDefinition::new("updateStatus")
            .with_buffering(SignalBuffering::Latest);

        assert_eq!(signal.buffering, SignalBuffering::Latest);
    }

    #[test]
    fn test_signal_definition_with_input_schema() {
        let schema = SignalSchema::with_fields(vec![
            SignalSchemaField {
                name: "approved".to_string(),
                typescript_type: "boolean".to_string(),
                required: true,
                description: Some("Whether approved".to_string()),
                default: None,
            },
        ]);

        let signal = SignalDefinition::new("approval")
            .with_input_schema(schema);

        assert!(!signal.input_schema.fields.is_empty());
    }

    #[test]
    fn test_signal_definition_typescript_type_name() {
        let signal = SignalDefinition::new("approveOrder");

        assert_eq!(signal.typescript_type_name(), "ApproveOrderSignal");
    }

    #[test]
    fn test_signal_definition_typescript_payload_type() {
        let signal = SignalDefinition::new("approveOrder");

        // Empty schema means void payload
        assert_eq!(signal.typescript_payload_type(), "void");
    }

    #[test]
    fn test_signal_definition_to_typescript_definition() {
        let signal = SignalDefinition::new("updateStatus")
            .with_description("Update workflow status");

        let ts = signal.to_typescript_definition();

        assert!(ts.contains("defineSignal<void>('updateStatus')"));
    }

    #[test]
    fn test_signal_handler_basic() {
        let handler = SignalHandler::new("approveOrder");

        assert_eq!(handler.signal_name, "approveOrder");
        assert!(handler.validate_input);
    }

    #[test]
    fn test_signal_handler_with_node() {
        let handler = SignalHandler::new("approveOrder")
            .with_node("process-approval");

        assert!(matches!(handler.handler, SignalHandlerLogic::NodeReference { .. }));
    }

    #[test]
    fn test_signal_handler_with_custom_code() {
        let handler = SignalHandler::new("approveOrder")
            .with_custom_code("console.log('Approved!');");

        assert!(matches!(handler.handler, SignalHandlerLogic::Custom { .. }));
    }

    #[test]
    fn test_signal_handler_with_update() {
        let handler = SignalHandler::new("updateCount")
            .with_update(VariableUpdate::new(
                "count",
                VariableSource::from_expression("state.variables.count + 1"),
            ));

        assert_eq!(handler.updates.len(), 1);
    }

    #[test]
    fn test_signal_handler_without_validation() {
        let handler = SignalHandler::new("approveOrder")
            .without_validation();

        assert!(!handler.validate_input);
    }

    #[test]
    fn test_signal_handler_to_typescript() {
        let signal = SignalDefinition::new("updateCount");
        let handler = SignalHandler::new("updateCount")
            .with_update(VariableUpdate::new(
                "count",
                VariableSource::from_expression("state.variables.count + 1"),
            ));

        let ts = handler.to_typescript(&signal);

        assert!(ts.contains("setHandler(updateCount"));
        assert!(ts.contains("state.variables.count"));
    }

    #[test]
    fn test_signal_with_handler() {
        let definition = SignalDefinition::new("notify")
            .with_description("Send notification");
        let handler = SignalHandler::new("notify")
            .with_custom_code("console.log('Notified!');");

        let signal = SignalWithHandler::new(definition, handler);
        let ts = signal.to_typescript();

        assert!(ts.contains("defineSignal"));
        assert!(ts.contains("setHandler"));
        assert!(ts.contains("console.log"));
    }

    #[test]
    fn test_workflow_signals_collection() {
        let mut signals = WorkflowSignals::new();

        signals.add(SignalWithHandler::new(
            SignalDefinition::new("start"),
            SignalHandler::new("start"),
        ));
        signals.add(SignalWithHandler::new(
            SignalDefinition::new("stop"),
            SignalHandler::new("stop"),
        ));

        assert_eq!(signals.signals.len(), 2);
        assert!(signals.get("start").is_some());
        assert!(signals.get("stop").is_some());
    }

    #[test]
    fn test_workflow_signals_to_typescript() {
        let mut signals = WorkflowSignals::new();
        signals.add(SignalWithHandler::new(
            SignalDefinition::new("test"),
            SignalHandler::new("test"),
        ));

        let ts = signals.to_typescript();

        assert!(ts.contains("defineSignal"));
        assert!(ts.contains("setHandler"));
    }

    #[test]
    fn test_variable_source_from_payload() {
        let source = VariableSource::from_payload("approved");
        assert_eq!(source.to_typescript(), "payload.approved");
    }

    #[test]
    fn test_variable_source_from_constant() {
        let source = VariableSource::from_constant(serde_json::json!("test"));
        assert_eq!(source.to_typescript(), "'test'");
    }

    #[test]
    fn test_variable_source_from_expression() {
        let source = VariableSource::from_expression("Date.now()");
        assert_eq!(source.to_typescript(), "Date.now()");
    }

    #[test]
    fn test_signal_buffering_typescript_comment() {
        assert!(SignalBuffering::Ordered.to_typescript_comment().contains("order"));
        assert!(SignalBuffering::Latest.to_typescript_comment().contains("recent"));
        assert!(SignalBuffering::Immediate.to_typescript_comment().contains("immediately"));
    }

    #[test]
    fn test_signal_schema_typescript_interface() {
        let schema = SignalSchema::with_fields(vec![
            SignalSchemaField {
                name: "approved".to_string(),
                typescript_type: "boolean".to_string(),
                required: true,
                description: Some("Whether approved".to_string()),
                default: None,
            },
        ]);

        let ts = schema.to_typescript_interface("ApprovalPayload");

        assert!(ts.contains("interface ApprovalPayload"));
        assert!(ts.contains("approved: boolean"));
    }

    #[test]
    fn test_signal_serialization_roundtrip() {
        let signal = SignalDefinition::new("testSignal")
            .with_description("Test signal")
            .with_buffering(SignalBuffering::Latest);

        let json = serde_json::to_string(&signal).unwrap();
        let restored: SignalDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(signal.name, restored.name);
        assert_eq!(signal.buffering, restored.buffering);
    }
}

// =============================================================================
// QUERY HANDLER TESTS
// =============================================================================

mod query_handler_tests {
    use super::*;

    #[test]
    fn test_query_definition_basic() {
        let query = QueryDefinition::new(
            "getCount",
            QuerySchema::object(vec![("count", "number")]),
            QueryHandlerLogic::computed("state.variables.count"),
        );

        assert_eq!(query.name, "getCount");
    }

    #[test]
    fn test_query_definition_with_description() {
        let query = QueryDefinition::new(
            "getCount",
            QuerySchema::any(),
            QueryHandlerLogic::project(vec!["count"]),
        )
        .with_description("Get the current count");

        assert!(query.description.is_some());
    }

    #[test]
    fn test_query_definition_with_input_schema() {
        let query = QueryDefinition::new(
            "getItem",
            QuerySchema::object(vec![("item", "unknown")]),
            QueryHandlerLogic::computed("state.variables.items[input.index]"),
        )
        .with_input_schema(QuerySchema::object(vec![("index", "number")]));

        assert!(!query.input_schema.is_void());
    }

    #[test]
    fn test_query_handler_logic_projection() {
        let logic = QueryHandlerLogic::project(vec!["name", "status"]);
        let ts = logic.to_typescript();

        assert!(ts.contains("name: state.variables.name"));
        assert!(ts.contains("status: state.variables.status"));
    }

    #[test]
    fn test_query_handler_logic_all_projection() {
        let logic = QueryHandlerLogic::project(vec!["*"]);
        let ts = logic.to_typescript();

        assert!(ts.contains("...state.variables"));
    }

    #[test]
    fn test_query_handler_logic_computed() {
        let logic = QueryHandlerLogic::computed("state.variables.items.length");
        let ts = logic.to_typescript();

        assert!(ts.contains("return state.variables.items.length"));
    }

    #[test]
    fn test_query_handler_logic_custom() {
        let logic = QueryHandlerLogic::custom("return calculateStats(state);");
        let ts = logic.to_typescript();

        assert!(ts.contains("return calculateStats(state);"));
    }

    #[test]
    fn test_query_definition_to_typescript() {
        let query = QueryDefinition::new(
            "getCount",
            QuerySchema::object(vec![("count", "number")]),
            QueryHandlerLogic::computed("state.variables.count"),
        );

        let ts = query.to_typescript();

        assert!(ts.contains("defineQuery"));
        assert!(ts.contains("setHandler"));
        assert!(ts.contains("getCountQuery"));
    }

    #[test]
    fn test_standard_queries() {
        let queries = standard_queries();

        assert!(!queries.is_empty());

        let names: Vec<_> = queries.iter().map(|q| q.name.as_str()).collect();
        assert!(names.contains(&"getState"));
        assert!(names.contains(&"getProgress"));
        assert!(names.contains(&"getStatus"));
    }

    #[test]
    fn test_workflow_queries_collection() {
        let mut queries = WorkflowQueries::new();
        queries.add(QueryDefinition::new(
            "customQuery",
            QuerySchema::any(),
            QueryHandlerLogic::project(vec!["*"]),
        ));

        assert_eq!(queries.queries.len(), 1);
        assert!(queries.get("customQuery").is_some());
    }

    #[test]
    fn test_workflow_queries_with_standard() {
        let queries = WorkflowQueries::with_standard_queries();

        assert!(queries.get("getState").is_some());
        assert!(queries.get("getProgress").is_some());
    }

    #[test]
    fn test_workflow_queries_to_typescript() {
        let queries = WorkflowQueries::with_standard_queries();
        let ts = queries.to_typescript();

        assert!(ts.contains("defineQuery"));
        assert!(ts.contains("setHandler"));
        assert!(ts.contains("@temporalio/workflow"));
    }

    #[test]
    fn test_query_schema_object() {
        let schema = QuerySchema::object(vec![
            ("name", "string"),
            ("count", "number"),
        ]);

        assert_eq!(schema.fields.len(), 2);
    }

    #[test]
    fn test_query_schema_typescript_interface() {
        let schema = QuerySchema::object(vec![
            ("name", "string"),
            ("count", "number"),
        ]);

        let ts = schema.to_typescript_interface("TestOutput");

        assert!(ts.contains("interface TestOutput"));
        assert!(ts.contains("name: string"));
        assert!(ts.contains("count: number"));
    }

    #[test]
    fn test_query_serialization_roundtrip() {
        let query = QueryDefinition::new(
            "test",
            QuerySchema::object(vec![("value", "string")]),
            QueryHandlerLogic::project(vec!["value"]),
        );

        let json = serde_json::to_string(&query).unwrap();
        let restored: QueryDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(query.name, restored.name);
    }
}

// =============================================================================
// CANCELLATION HANDLING TESTS
// =============================================================================

mod cancellation_tests {
    use super::*;

    #[test]
    fn test_cancellation_scope_basic() {
        let scope = CancellationScope::new("orderProcessing");

        assert_eq!(scope.name, "orderProcessing");
        assert!(!scope.shielded);
        assert!(scope.cleanup.is_none());
    }

    #[test]
    fn test_cancellation_scope_shielded() {
        let scope = CancellationScope::shielded("criticalOperation");

        assert!(scope.shielded);
    }

    #[test]
    fn test_cancellation_scope_with_cleanup() {
        let cleanup = CleanupConfig::new()
            .with_activity(CleanupActivity::new("releaseResources"));

        let scope = CancellationScope::new("resourceScope")
            .with_cleanup(cleanup);

        assert!(scope.cleanup.is_some());
    }

    #[test]
    fn test_cancellation_scope_with_cleanup_timeout() {
        let scope = CancellationScope::new("test")
            .with_cleanup_timeout(60000);

        assert_eq!(scope.cleanup_timeout_ms, 60000);
    }

    #[test]
    fn test_cancellation_scope_to_typescript() {
        let scope = CancellationScope::new("processOrder");
        let ts = scope.to_typescript("await processOrder();");

        assert!(ts.contains("CancellationScope.cancellable"));
        assert!(ts.contains("await processOrder()"));
    }

    #[test]
    fn test_shielded_scope_to_typescript() {
        let scope = CancellationScope::shielded("saveState");
        let ts = scope.to_typescript("await saveState();");

        assert!(ts.contains("CancellationScope.nonCancellable"));
    }

    #[test]
    fn test_cleanup_config_with_activity() {
        let cleanup = CleanupConfig::new()
            .with_activity(CleanupActivity::new("cleanup"));

        assert_eq!(cleanup.activities.len(), 1);
    }

    #[test]
    fn test_cleanup_config_with_state_update() {
        let cleanup = CleanupConfig::new()
            .with_state_update(StateUpdate::new("status", serde_json::json!("cancelled")));

        assert_eq!(cleanup.state_updates.len(), 1);
    }

    #[test]
    fn test_cleanup_config_with_custom_code() {
        let cleanup = CleanupConfig::new()
            .with_custom_code("console.log('Cleanup complete');");

        assert!(cleanup.custom_code.is_some());
    }

    #[test]
    fn test_cleanup_config_to_typescript() {
        let cleanup = CleanupConfig::new()
            .with_activity(CleanupActivity::new("cleanup"))
            .with_state_update(StateUpdate::new("status", serde_json::json!("cleaned")));

        let ts = cleanup.to_typescript();

        assert!(ts.contains("activities.cleanup"));
        assert!(ts.contains("state.variables.status"));
    }

    #[test]
    fn test_cleanup_activity_builder() {
        let activity = CleanupActivity::new("releaseResource")
            .with_input(serde_json::json!({"resourceId": "123"}))
            .with_max_attempts(3)
            .fail_on_error();

        assert_eq!(activity.activity_name, "releaseResource");
        assert_eq!(activity.max_attempts, 3);
        assert!(!activity.continue_on_failure);
    }

    #[test]
    fn test_workflow_cancellation_handler_basic() {
        let handler = WorkflowCancellationHandler::new();

        assert!(handler.enabled);
    }

    #[test]
    fn test_workflow_cancellation_handler_disabled() {
        let handler = WorkflowCancellationHandler::disabled();

        assert!(!handler.enabled);
    }

    #[test]
    fn test_workflow_cancellation_handler_with_cleanup() {
        let handler = WorkflowCancellationHandler::new()
            .with_cleanup(CleanupConfig::new());

        assert!(handler.cleanup.is_some());
    }

    #[test]
    fn test_workflow_cancellation_handler_with_scope() {
        let handler = WorkflowCancellationHandler::new()
            .with_scope(CancellationScope::new("scope1"));

        assert_eq!(handler.scopes.len(), 1);
        assert!(handler.get_scope("scope1").is_some());
    }

    #[test]
    fn test_workflow_cancellation_handler_typescript_imports() {
        let imports = WorkflowCancellationHandler::typescript_imports();

        assert!(imports.contains("CancellationScope"));
        assert!(imports.contains("isCancellation"));
    }

    #[test]
    fn test_cancellation_serialization_roundtrip() {
        let scope = CancellationScope::new("test")
            .with_cleanup_timeout(10000);

        let json = serde_json::to_string(&scope).unwrap();
        let restored: CancellationScope = serde_json::from_str(&json).unwrap();

        assert_eq!(scope.name, restored.name);
        assert_eq!(scope.cleanup_timeout_ms, restored.cleanup_timeout_ms);
    }
}

// =============================================================================
// SEARCH ATTRIBUTES TESTS
// =============================================================================

mod search_attributes_tests {
    use super::*;

    #[test]
    fn test_search_attribute_type_typescript() {
        assert_eq!(SearchAttributeType::Bool.typescript_type(), "boolean");
        assert_eq!(SearchAttributeType::Int.typescript_type(), "number");
        assert_eq!(SearchAttributeType::Double.typescript_type(), "number");
        assert_eq!(SearchAttributeType::Keyword.typescript_type(), "string");
        assert_eq!(SearchAttributeType::KeywordList.typescript_type(), "string[]");
        assert_eq!(SearchAttributeType::Text.typescript_type(), "string");
        assert_eq!(SearchAttributeType::Datetime.typescript_type(), "Date");
    }

    #[test]
    fn test_search_attribute_definition_basic() {
        let def = SearchAttributeDefinition::new("CustomStatus", SearchAttributeType::Keyword);

        assert_eq!(def.name, "CustomStatus");
        assert!(def.indexed);
    }

    #[test]
    fn test_search_attribute_definition_with_description() {
        let def = SearchAttributeDefinition::new("CustomStatus", SearchAttributeType::Keyword)
            .with_description("Workflow status");

        assert!(def.description.is_some());
    }

    #[test]
    fn test_search_attribute_definition_with_default() {
        let def = SearchAttributeDefinition::new("CustomStatus", SearchAttributeType::Keyword)
            .with_default(serde_json::json!("pending"));

        assert!(def.default.is_some());
    }

    #[test]
    fn test_search_attribute_definition_not_indexed() {
        let def = SearchAttributeDefinition::new("InternalField", SearchAttributeType::Text)
            .not_indexed();

        assert!(!def.indexed);
    }

    #[test]
    fn test_search_attribute_definition_typescript_field() {
        let def = SearchAttributeDefinition::new("Status", SearchAttributeType::Keyword);
        assert_eq!(def.to_typescript_field(), "Status: string;");

        let def_with_default = def.with_default(serde_json::json!("pending"));
        assert_eq!(def_with_default.to_typescript_field(), "Status?: string;");
    }

    #[test]
    fn test_typed_search_attribute_value_typescript() {
        assert_eq!(TypedSearchAttributeValue::Bool(true).to_typescript(), "true");
        assert_eq!(TypedSearchAttributeValue::Int(42).to_typescript(), "42");
        assert_eq!(TypedSearchAttributeValue::Double(3.14).to_typescript(), "3.14");
        assert_eq!(
            TypedSearchAttributeValue::Keyword("test".to_string()).to_typescript(),
            "'test'"
        );
        assert_eq!(
            TypedSearchAttributeValue::KeywordList(vec!["a".to_string(), "b".to_string()])
                .to_typescript(),
            "['a', 'b']"
        );
    }

    #[test]
    fn test_search_attribute_update() {
        let update = SearchAttributeUpdate::new(
            "CustomStatus",
            TypedSearchAttributeValue::Keyword("completed".to_string()),
        );

        let ts = update.to_typescript();

        assert!(ts.contains("upsertSearchAttributes"));
        assert!(ts.contains("CustomStatus"));
        assert!(ts.contains("'completed'"));
    }

    #[test]
    fn test_workflow_search_attributes_basic() {
        let mut attrs = WorkflowSearchAttributes::new();
        attrs.add_definition(
            SearchAttributeDefinition::new("UserId", SearchAttributeType::Keyword),
        );

        assert!(attrs.get_definition("UserId").is_some());
    }

    #[test]
    fn test_workflow_search_attributes_set_initial_value() {
        let mut attrs = WorkflowSearchAttributes::new();
        attrs.add_definition(
            SearchAttributeDefinition::new("UserId", SearchAttributeType::Keyword),
        );
        attrs.set_initial_value(
            "UserId",
            TypedSearchAttributeValue::Keyword("user-123".to_string()),
        );

        assert!(attrs.initial_values.contains_key("UserId"));
    }

    #[test]
    fn test_workflow_search_attributes_typescript_interface() {
        let mut attrs = WorkflowSearchAttributes::new();
        attrs.add_definition(
            SearchAttributeDefinition::new("Status", SearchAttributeType::Keyword),
        );
        attrs.add_definition(
            SearchAttributeDefinition::new("Count", SearchAttributeType::Int),
        );

        let ts = attrs.to_typescript_interface();

        assert!(ts.contains("interface WorkflowSearchAttributes"));
        assert!(ts.contains("string"));
        assert!(ts.contains("number"));
    }

    #[test]
    fn test_standard_search_attributes() {
        let attrs = standard_search_attributes();

        assert!(!attrs.is_empty());

        let names: Vec<_> = attrs.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"CustomStatus"));
    }

    #[test]
    fn test_search_attribute_serialization_roundtrip() {
        let def = SearchAttributeDefinition::new("Test", SearchAttributeType::Bool)
            .with_description("Test attribute");

        let json = serde_json::to_string(&def).unwrap();
        let restored: SearchAttributeDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(def.name, restored.name);
        assert_eq!(def.attribute_type, restored.attribute_type);
    }
}

// =============================================================================
// VERSIONING TESTS
// =============================================================================

mod versioning_tests {
    use super::*;

    #[test]
    fn test_version_info_basic() {
        let version = VersionInfo::new("1.0.0", "Initial release");

        assert_eq!(version.version, "1.0.0");
        assert!(!version.breaking_changes);
    }

    #[test]
    fn test_version_info_breaking() {
        let version = VersionInfo::new("2.0.0", "Major update")
            .breaking();

        assert!(version.breaking_changes);
    }

    #[test]
    fn test_version_info_with_changes() {
        let version = VersionInfo::new("1.0.0", "Initial release")
            .with_changes(vec!["change-1", "change-2"]);

        assert_eq!(version.changes.len(), 2);
    }

    #[test]
    fn test_version_branch_from_version() {
        let branch = VersionBranch::from_version("2.0.0", "await newFeature();");

        assert_eq!(branch.min_version, Some("2.0.0".to_string()));
        assert!(branch.max_version.is_none());
    }

    #[test]
    fn test_version_branch_before_version() {
        let branch = VersionBranch::before_version("2.0.0", "await legacyFeature();");

        assert!(branch.min_version.is_none());
        assert_eq!(branch.max_version, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_version_branch_between() {
        let branch = VersionBranch::between("1.0.0", "2.0.0", "await v1Feature();");

        assert_eq!(branch.min_version, Some("1.0.0".to_string()));
        assert_eq!(branch.max_version, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_version_branch_with_description() {
        let branch = VersionBranch::from_version("2.0.0", "newCode();")
            .with_description("New implementation");

        assert!(branch.description.is_some());
    }

    #[test]
    fn test_version_change_point_basic() {
        let change = VersionChangePoint::new(
            "new-validation",
            "2.0.0",
            "Added input validation",
        );

        assert_eq!(change.change_id, "new-validation");
        assert_eq!(change.introduced_version, "2.0.0");
    }

    #[test]
    fn test_version_change_point_with_branches() {
        let change = VersionChangePoint::new("test-change", "1.0.0", "Test change")
            .with_branch(VersionBranch::from_version("1.0.0", "newCode();"))
            .with_branch(VersionBranch::before_version("1.0.0", "oldCode();"));

        assert_eq!(change.branches.len(), 2);
    }

    #[test]
    fn test_version_change_point_to_typescript() {
        let change = VersionChangePoint::new("test-change", "1.0.0", "Test change")
            .with_branch(VersionBranch::from_version("1.0.0", "newCode();"))
            .with_branch(VersionBranch::before_version("1.0.0", "oldCode();"));

        let ts = change.to_typescript();

        assert!(ts.contains("patched('test-change')"));
        assert!(ts.contains("newCode()"));
        assert!(ts.contains("oldCode()"));
    }

    #[test]
    fn test_versioning_config_basic() {
        let config = VersioningConfig::new("1.0.0");

        assert_eq!(config.current_version, "1.0.0");
    }

    #[test]
    fn test_versioning_config_add_version() {
        let mut config = VersioningConfig::new("2.0.0");
        config.add_version(VersionInfo::new("1.0.0", "Initial release"));
        config.add_version(VersionInfo::new("2.0.0", "Major update").breaking());

        assert_eq!(config.version_history.len(), 2);
    }

    #[test]
    fn test_versioning_config_add_change_point() {
        let mut config = VersioningConfig::new("2.0.0");
        config.add_change_point(VersionChangePoint::new(
            "feature-x",
            "2.0.0",
            "New feature X",
        ));

        assert!(config.get_change_point("feature-x").is_some());
    }

    #[test]
    fn test_versioning_config_typescript_imports() {
        let imports = VersioningConfig::typescript_imports();

        assert!(imports.contains("patched"));
    }

    #[test]
    fn test_versioning_config_typescript_version_constant() {
        let config = VersioningConfig::new("1.5.0");
        let ts = config.to_typescript_version_constant();

        assert!(ts.contains("1.5.0"));
    }

    #[test]
    fn test_versioning_serialization_roundtrip() {
        let config = VersioningConfig::new("1.0.0");

        let json = serde_json::to_string(&config).unwrap();
        let restored: VersioningConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.current_version, restored.current_version);
    }
}

// =============================================================================
// SAGA PATTERN TESTS
// =============================================================================

mod saga_pattern_tests {
    use super::*;
    use radium_workflow::schema::patterns::WorkflowPattern;

    #[test]
    fn test_saga_definition_basic() {
        let saga = SagaDefinition::new("OrderSaga");

        assert_eq!(saga.name, "OrderSaga");
        assert!(saga.steps.is_empty());
    }

    #[test]
    fn test_saga_definition_with_description() {
        let saga = SagaDefinition::new("OrderSaga")
            .with_description("Process order with compensation");

        assert!(saga.description.is_some());
    }

    #[test]
    fn test_saga_definition_with_step() {
        let saga = SagaDefinition::new("OrderSaga")
            .with_step(SagaStep::new("reserveInventory", SagaAction::activity("reserveInventory")));

        assert_eq!(saga.steps.len(), 1);
    }

    #[test]
    fn test_saga_step_with_compensation() {
        let step = SagaStep::new("reserveInventory", SagaAction::activity("reserveInventory"))
            .with_compensation(SagaAction::activity("releaseInventory"));

        assert!(step.compensation.is_some());
    }

    #[test]
    fn test_saga_step_optional() {
        let step = SagaStep::new("notifyUser", SagaAction::activity("sendNotification"))
            .optional();

        assert!(step.optional);
    }

    #[test]
    fn test_saga_action_activity() {
        let action = SagaAction::activity("processPayment");
        let ts = action.to_typescript();

        assert!(ts.contains("activities.processPayment"));
    }

    #[test]
    fn test_saga_action_child_workflow() {
        let action = SagaAction::child_workflow("SubProcess");
        let ts = action.to_typescript();

        assert!(ts.contains("executeChild"));
    }

    #[test]
    fn test_saga_action_custom() {
        let action = SagaAction::custom("await customLogic();");
        let ts = action.to_typescript();

        assert!(ts.contains("customLogic"));
    }

    #[test]
    fn test_saga_with_compensation_behavior() {
        let saga = SagaDefinition::new("OrderSaga")
            .with_compensation_behavior(CompensationBehavior::CompensateFailed);

        assert_eq!(saga.compensation_behavior, CompensationBehavior::CompensateFailed);
    }

    #[test]
    fn test_saga_parallel_compensation() {
        let saga = SagaDefinition::new("OrderSaga")
            .parallel_compensation();

        assert!(saga.parallel_compensation);
    }

    #[test]
    fn test_saga_validate_empty() {
        let saga = SagaDefinition::new("EmptySaga");

        assert!(saga.validate_pattern().is_err());
    }

    #[test]
    fn test_saga_validate_with_steps() {
        let saga = SagaDefinition::new("ValidSaga")
            .with_step(SagaStep::new("step1", SagaAction::activity("doStep1")));

        assert!(saga.validate_pattern().is_ok());
    }

    #[test]
    fn test_saga_to_typescript() {
        let saga = SagaDefinition::new("TestSaga")
            .with_step(SagaStep::new("step1", SagaAction::activity("doStep1")));

        let ts = saga.to_typescript();

        assert!(ts.contains("TestSagaSaga"));
        assert!(ts.contains("activities.doStep1"));
        assert!(ts.contains("compensateStep"));
    }

    #[test]
    fn test_saga_to_typescript_parallel_compensation() {
        let saga = SagaDefinition::new("ParallelSaga")
            .parallel_compensation()
            .with_step(SagaStep::new("step1", SagaAction::activity("doStep1")));

        let ts = saga.to_typescript();

        assert!(ts.contains("Promise.allSettled"));
    }

    #[test]
    fn test_saga_serialization_roundtrip() {
        let saga = SagaDefinition::new("TestSaga")
            .with_step(SagaStep::new("step1", SagaAction::activity("doStep1")));

        let json = serde_json::to_string(&saga).unwrap();
        let restored: SagaDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(saga.name, restored.name);
        assert_eq!(saga.steps.len(), restored.steps.len());
    }
}

// =============================================================================
// SCATTER-GATHER PATTERN TESTS
// =============================================================================

mod scatter_gather_pattern_tests {
    use super::*;
    use radium_workflow::schema::patterns::WorkflowPattern;

    #[test]
    fn test_scatter_gather_basic() {
        let sg = ScatterGatherDefinition::new("ProcessBatch");

        assert_eq!(sg.name, "ProcessBatch");
    }

    #[test]
    fn test_scatter_gather_with_description() {
        let sg = ScatterGatherDefinition::new("ProcessBatch")
            .with_description("Process items in parallel");

        assert!(sg.description.is_some());
    }

    #[test]
    fn test_scatter_gather_with_scatter() {
        let sg = ScatterGatherDefinition::new("ProcessBatch")
            .with_scatter(ScatterConfig::with_workers(vec![
                ScatterWorker::activity("processItem"),
            ]));

        assert_eq!(sg.scatter.workers.len(), 1);
    }

    #[test]
    fn test_scatter_gather_with_gather() {
        let sg = ScatterGatherDefinition::new("ProcessBatch")
            .with_gather(GatherConfig::wait_all());

        assert_eq!(sg.gather.strategy, GatherStrategy::WaitAll);
    }

    #[test]
    fn test_scatter_gather_with_timeout() {
        let sg = ScatterGatherDefinition::new("ProcessBatch")
            .with_timeout(30000);

        assert_eq!(sg.timeout_ms, Some(30000));
    }

    #[test]
    fn test_scatter_worker_activity() {
        let worker = ScatterWorker::activity("processItem");
        let ts = worker.to_typescript();

        assert!(ts.contains("activities.processItem"));
    }

    #[test]
    fn test_scatter_worker_child_workflow() {
        let worker = ScatterWorker::child_workflow("SubProcess");
        let ts = worker.to_typescript();

        assert!(ts.contains("executeChild"));
    }

    #[test]
    fn test_gather_config_wait_all() {
        let config = GatherConfig::wait_all();
        assert_eq!(config.strategy, GatherStrategy::WaitAll);
    }

    #[test]
    fn test_gather_config_wait_first() {
        let config = GatherConfig::wait_first();
        assert_eq!(config.strategy, GatherStrategy::WaitFirst);
    }

    #[test]
    fn test_gather_config_wait_threshold() {
        let config = GatherConfig::wait_threshold(3);
        assert_eq!(config.strategy, GatherStrategy::WaitThreshold);
        assert_eq!(config.threshold, Some(3));
    }

    #[test]
    fn test_scatter_gather_validate_empty() {
        let sg = ScatterGatherDefinition::new("Empty");

        assert!(sg.validate_pattern().is_err());
    }

    #[test]
    fn test_scatter_gather_validate_invalid_threshold() {
        let sg = ScatterGatherDefinition::new("Invalid")
            .with_scatter(ScatterConfig::with_workers(vec![ScatterWorker::activity("work")]))
            .with_gather(GatherConfig::wait_threshold(5));

        assert!(sg.validate_pattern().is_err());
    }

    #[test]
    fn test_scatter_gather_to_typescript() {
        let sg = ScatterGatherDefinition::new("TestGather")
            .with_scatter(ScatterConfig::with_workers(vec![ScatterWorker::activity("doWork")]))
            .with_gather(GatherConfig::wait_all());

        let ts = sg.to_typescript();

        assert!(ts.contains("TestGatherScatterGather"));
        assert!(ts.contains("Promise.allSettled"));
    }

    #[test]
    fn test_scatter_gather_serialization_roundtrip() {
        let sg = ScatterGatherDefinition::new("Test")
            .with_scatter(ScatterConfig::with_workers(vec![ScatterWorker::activity("work")]));

        let json = serde_json::to_string(&sg).unwrap();
        let restored: ScatterGatherDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(sg.name, restored.name);
    }
}

// =============================================================================
// PIPELINE PATTERN TESTS
// =============================================================================

mod pipeline_pattern_tests {
    use super::*;
    use radium_workflow::schema::patterns::WorkflowPattern;

    #[test]
    fn test_pipeline_basic() {
        let pipeline = PipelineDefinition::new("DataProcessor");

        assert_eq!(pipeline.name, "DataProcessor");
        assert!(pipeline.stages.is_empty());
    }

    #[test]
    fn test_pipeline_with_description() {
        let pipeline = PipelineDefinition::new("DataProcessor")
            .with_description("Process data through stages");

        assert!(pipeline.description.is_some());
    }

    #[test]
    fn test_pipeline_with_stage() {
        let pipeline = PipelineDefinition::new("DataProcessor")
            .with_stage(PipelineStage::activity("validate"));

        assert_eq!(pipeline.stages.len(), 1);
    }

    #[test]
    fn test_pipeline_activity_shortcut() {
        let pipeline = PipelineDefinition::new("DataProcessor")
            .activity("validate")
            .activity("transform")
            .activity("enrich");

        assert_eq!(pipeline.stages.len(), 3);
    }

    #[test]
    fn test_pipeline_track_intermediate() {
        let pipeline = PipelineDefinition::new("DataProcessor")
            .track_intermediate();

        assert!(pipeline.track_intermediate_results);
    }

    #[test]
    fn test_pipeline_stage_with_retries() {
        let stage = PipelineStage::activity("fetchData")
            .with_retries(3);

        assert_eq!(stage.retry_count, 3);
    }

    #[test]
    fn test_pipeline_stage_with_timeout() {
        let stage = PipelineStage::activity("fetchData")
            .with_timeout(5000);

        assert_eq!(stage.timeout_ms, Some(5000));
    }

    #[test]
    fn test_pipeline_stage_optional() {
        let stage = PipelineStage::activity("enrichData")
            .optional();

        assert!(stage.optional);
    }

    #[test]
    fn test_stage_processor_activity() {
        let processor = StageProcessor::activity("processItem");
        let ts = processor.to_typescript();

        assert!(ts.contains("activities.processItem"));
    }

    #[test]
    fn test_stage_processor_transform() {
        let processor = StageProcessor::transform("currentData.map(x => x * 2)");
        let ts = processor.to_typescript();

        assert!(ts.contains("map"));
    }

    #[test]
    fn test_pipeline_validate_empty() {
        let pipeline = PipelineDefinition::new("Empty");

        assert!(pipeline.validate_pattern().is_err());
    }

    #[test]
    fn test_pipeline_validate_with_stages() {
        let pipeline = PipelineDefinition::new("Valid")
            .with_stage(PipelineStage::activity("step1"));

        assert!(pipeline.validate_pattern().is_ok());
    }

    #[test]
    fn test_pipeline_to_typescript() {
        let pipeline = PipelineDefinition::new("TestPipeline")
            .with_stage(PipelineStage::activity("step1"))
            .with_stage(PipelineStage::activity("step2"));

        let ts = pipeline.to_typescript();

        assert!(ts.contains("TestPipelinePipeline"));
        assert!(ts.contains("activities.step1"));
        assert!(ts.contains("activities.step2"));
        assert!(ts.contains("stagesCompleted"));
    }

    #[test]
    fn test_pipeline_with_retries_typescript() {
        let pipeline = PipelineDefinition::new("RetryPipeline")
            .with_stage(PipelineStage::activity("flaky").with_retries(3));

        let ts = pipeline.to_typescript();

        assert!(ts.contains("for (let attempt"));
        assert!(ts.contains("retrying"));
    }

    #[test]
    fn test_pipeline_serialization_roundtrip() {
        let pipeline = PipelineDefinition::new("Test")
            .with_stage(PipelineStage::activity("step1"));

        let json = serde_json::to_string(&pipeline).unwrap();
        let restored: PipelineDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(pipeline.name, restored.name);
        assert_eq!(pipeline.stages.len(), restored.stages.len());
    }
}

// =============================================================================
// MAP-REDUCE PATTERN TESTS
// =============================================================================

mod map_reduce_pattern_tests {
    use super::*;
    use radium_workflow::schema::patterns::WorkflowPattern;

    #[test]
    fn test_map_reduce_basic() {
        let mr = MapReduceDefinition::new("ProcessItems");

        assert_eq!(mr.name, "ProcessItems");
        assert_eq!(mr.max_concurrency, 10);
    }

    #[test]
    fn test_map_reduce_with_description() {
        let mr = MapReduceDefinition::new("ProcessItems")
            .with_description("Process items in parallel");

        assert!(mr.description.is_some());
    }

    #[test]
    fn test_map_reduce_with_map() {
        let mr = MapReduceDefinition::new("ProcessItems")
            .with_map(MapConfig::activity("processItem"));

        assert!(matches!(mr.map.mapper, Mapper::Activity { .. }));
    }

    #[test]
    fn test_map_reduce_with_reduce() {
        let mr = MapReduceDefinition::new("ProcessItems")
            .with_reduce(ReduceConfig::sum());

        assert!(matches!(mr.reduce.reducer, Reducer::Sum));
    }

    #[test]
    fn test_map_reduce_with_max_concurrency() {
        let mr = MapReduceDefinition::new("ProcessItems")
            .with_max_concurrency(5);

        assert_eq!(mr.max_concurrency, 5);
    }

    #[test]
    fn test_map_reduce_with_batch_size() {
        let mr = MapReduceDefinition::new("ProcessItems")
            .with_batch_size(100);

        assert_eq!(mr.batch_size, Some(100));
    }

    #[test]
    fn test_map_reduce_continue_on_failure() {
        let mr = MapReduceDefinition::new("ProcessItems")
            .continue_on_failure();

        assert!(mr.continue_on_failure);
    }

    #[test]
    fn test_map_config_activity() {
        let config = MapConfig::activity("transform");

        assert!(matches!(config.mapper, Mapper::Activity { .. }));
    }

    #[test]
    fn test_map_config_with_order_preserved() {
        let config = MapConfig::activity("transform")
            .with_order_preserved();

        assert!(config.preserve_order);
    }

    #[test]
    fn test_map_config_with_retries() {
        let config = MapConfig::activity("transform")
            .with_retries(3);

        assert_eq!(config.retry_count, 3);
    }

    #[test]
    fn test_mapper_activity_typescript() {
        let mapper = Mapper::activity("processItem");
        let ts = mapper.to_typescript();

        assert!(ts.contains("activities.processItem"));
    }

    #[test]
    fn test_mapper_transform_typescript() {
        let mapper = Mapper::transform("item * 2");
        assert_eq!(mapper.to_typescript(), "item * 2");
    }

    #[test]
    fn test_reduce_config_sum() {
        let config = ReduceConfig::sum();
        assert!(matches!(config.reducer, Reducer::Sum));
    }

    #[test]
    fn test_reduce_config_concat() {
        let config = ReduceConfig::concat();
        assert!(matches!(config.reducer, Reducer::Concat));
    }

    #[test]
    fn test_reduce_config_merge() {
        let config = ReduceConfig::merge();
        assert!(matches!(config.reducer, Reducer::Merge));
    }

    #[test]
    fn test_map_reduce_validate_invalid_concurrency() {
        let mr = MapReduceDefinition::new("Invalid")
            .with_max_concurrency(0);

        assert!(mr.validate_pattern().is_err());
    }

    #[test]
    fn test_map_reduce_validate_invalid_batch_size() {
        let mr = MapReduceDefinition {
            batch_size: Some(0),
            ..MapReduceDefinition::new("Invalid")
        };

        assert!(mr.validate_pattern().is_err());
    }

    #[test]
    fn test_map_reduce_validate_valid() {
        let mr = MapReduceDefinition::new("Valid");

        assert!(mr.validate_pattern().is_ok());
    }

    #[test]
    fn test_map_reduce_to_typescript() {
        let mr = MapReduceDefinition::new("TestMapReduce")
            .with_map(MapConfig::activity("process"))
            .with_reduce(ReduceConfig::sum());

        let ts = mr.to_typescript();

        assert!(ts.contains("TestMapReduceMapReduce"));
        assert!(ts.contains("activities.process"));
        assert!(ts.contains("reduce"));
    }

    #[test]
    fn test_map_reduce_with_batching_typescript() {
        let mr = MapReduceDefinition::new("BatchedMR")
            .with_batch_size(50)
            .with_map(MapConfig::activity("process"));

        let ts = mr.to_typescript();

        assert!(ts.contains("batchSize = 50"));
        assert!(ts.contains("mapBatch"));
    }

    #[test]
    fn test_map_reduce_serialization_roundtrip() {
        let mr = MapReduceDefinition::new("Test")
            .with_map(MapConfig::activity("process"))
            .with_reduce(ReduceConfig::sum());

        let json = serde_json::to_string(&mr).unwrap();
        let restored: MapReduceDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(mr.name, restored.name);
    }
}

// =============================================================================
// CROSS-FEATURE INTEGRATION TESTS
// =============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn test_child_workflow_with_search_attributes() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_search_attribute("CustomerId", AdvancedSearchAttributeValue::String("cust-123".into()))
            .with_search_attribute("Priority", AdvancedSearchAttributeValue::Int(5));

        assert_eq!(config.search_attributes.len(), 2);

        let ts = config.to_typescript();
        assert!(ts.contains("searchAttributes"));
    }

    #[test]
    fn test_signal_handler_with_query() {
        let mut signals = WorkflowSignals::new();
        signals.add(SignalWithHandler::new(
            SignalDefinition::new("updateProgress"),
            SignalHandler::new("updateProgress")
                .with_update(VariableUpdate::new(
                    "progress",
                    VariableSource::from_payload("percentage"),
                )),
        ));

        let mut queries = WorkflowQueries::new();
        queries.add(QueryDefinition::new(
            "getProgress",
            QuerySchema::object(vec![("progress", "number")]),
            QueryHandlerLogic::project(vec!["progress"]),
        ));

        let signals_ts = signals.to_typescript();
        let queries_ts = queries.to_typescript();

        assert!(signals_ts.contains("updateProgress"));
        assert!(queries_ts.contains("getProgress"));
    }

    #[test]
    fn test_versioned_signal_handler() {
        let signal = SignalDefinition::new("updateStatus")
            .with_description("Update workflow status");

        let change = VersionChangePoint::new(
            "enhanced-status-update",
            "2.0.0",
            "Enhanced status update with validation",
        )
        .with_branch(VersionBranch::from_version("2.0.0", "validateAndUpdateStatus(payload);"))
        .with_branch(VersionBranch::before_version("2.0.0", "updateStatus(payload);"));

        let signal_ts = signal.to_typescript_definition();
        let version_ts = change.to_typescript();

        assert!(signal_ts.contains("defineSignal"));
        assert!(version_ts.contains("patched"));
    }

    #[test]
    fn test_pipeline_with_search_attribute_updates() {
        let pipeline = PipelineDefinition::new("DataProcessor")
            .activity("validate")
            .activity("transform")
            .activity("enrich");

        let mut attrs = WorkflowSearchAttributes::new();
        attrs.add_definition(
            SearchAttributeDefinition::new("ProcessingStage", SearchAttributeType::Keyword),
        );

        let update = SearchAttributeUpdate::new(
            "ProcessingStage",
            TypedSearchAttributeValue::Keyword("validation".to_string()),
        );

        let pipeline_ts = pipeline.to_typescript();
        let update_ts = update.to_typescript();

        assert!(pipeline_ts.contains("validate"));
        assert!(update_ts.contains("ProcessingStage"));
    }
}

// =============================================================================
// TYPESCRIPT GENERATION QUALITY TESTS
// =============================================================================

mod typescript_generation_tests {
    use super::*;
    use radium_workflow::schema::patterns::WorkflowPattern;

    #[test]
    fn test_all_patterns_generate_valid_function_signature() {
        let saga = SagaDefinition::new("Test")
            .with_step(SagaStep::new("step", SagaAction::activity("act")));
        let ts = saga.to_typescript();
        assert!(ts.contains("export async function"));
        assert!(ts.contains("Promise<"));

        let pipeline = PipelineDefinition::new("Test")
            .with_stage(PipelineStage::activity("step"));
        let ts = pipeline.to_typescript();
        assert!(ts.contains("export async function"));
        assert!(ts.contains("Promise<"));

        let sg = ScatterGatherDefinition::new("Test")
            .with_scatter(ScatterConfig::with_workers(vec![ScatterWorker::activity("w")]));
        let ts = sg.to_typescript();
        assert!(ts.contains("export async function"));
        assert!(ts.contains("Promise<"));

        let mr = MapReduceDefinition::new("Test");
        let ts = mr.to_typescript();
        assert!(ts.contains("export async function"));
        assert!(ts.contains("Promise<"));
    }

    #[test]
    fn test_typescript_uses_temporal_imports() {
        let signals = WorkflowSignals::new();
        let ts = signals.to_typescript();
        assert!(ts.contains("@temporalio/workflow"));

        let queries = WorkflowQueries::with_standard_queries();
        let ts = queries.to_typescript();
        assert!(ts.contains("@temporalio/workflow"));

        let imports = WorkflowCancellationHandler::typescript_imports();
        assert!(imports.contains("@temporalio/workflow"));

        let imports = VersioningConfig::typescript_imports();
        assert!(imports.contains("@temporalio/workflow"));
    }

    #[test]
    fn test_typescript_uses_camel_case_variables() {
        let handler = SignalHandler::new("update_status");
        let signal = SignalDefinition::new("update_status");
        let ts = handler.to_typescript(&signal);
        // camelCase conversion
        assert!(ts.contains("updateStatus"));
    }

    #[test]
    fn test_workflow_id_strategies_generate_correct_typescript() {
        // UUID strategy
        let config = ChildWorkflowOrchestration::new("Test");
        let ts = config.to_typescript();
        assert!(ts.contains("uuid4()"));

        // Explicit strategy
        let config = ChildWorkflowOrchestration::new("Test")
            .with_workflow_id("explicit-id");
        let ts = config.to_typescript();
        assert!(ts.contains("workflowId: 'explicit-id'"));

        // Pattern strategy
        let config = ChildWorkflowOrchestration::new("Test")
            .with_id_pattern("child-{parent_id}-{index}");
        let ts = config.to_typescript();
        assert!(ts.contains("${workflowInfo().workflowId}"));
    }

    #[test]
    fn test_generated_interfaces_have_correct_types() {
        let schema = QuerySchema::object(vec![
            ("name", "string"),
            ("count", "number"),
            ("active", "boolean"),
            ("items", "string[]"),
        ]);
        let ts = schema.to_typescript_interface("TestInterface");

        assert!(ts.contains("name: string"));
        assert!(ts.contains("count: number"));
        assert!(ts.contains("active: boolean"));
        assert!(ts.contains("items: string[]"));
    }
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_signal_name_still_creates_definition() {
        // Empty names should be caught by validation, not creation
        let signal = SignalDefinition::new("");
        assert_eq!(signal.name, "");
    }

    #[test]
    fn test_special_characters_in_names() {
        let signal = SignalDefinition::new("approve-order_v2");
        let ts = signal.to_typescript_definition();
        // Should convert to camelCase
        assert!(ts.contains("approveOrderV2"));
    }

    #[test]
    fn test_very_long_timeout_values() {
        let config = ChildWorkflowOrchestration::new("Test")
            .with_execution_timeout(u64::MAX);

        assert_eq!(config.execution_timeout_ms, Some(u64::MAX));
    }

    #[test]
    fn test_empty_input_object() {
        let config = ChildWorkflowOrchestration::new("Test");

        assert!(config.input.is_empty());

        let ts = config.to_typescript();
        assert!(ts.contains("args: [{}]"));
    }

    #[test]
    fn test_null_json_values() {
        let config = ChildWorkflowOrchestration::new("Test")
            .with_input("nullValue", serde_json::Value::Null);

        let ts = config.to_typescript();
        assert!(ts.contains("null"));
    }

    #[test]
    fn test_nested_json_values() {
        let nested = serde_json::json!({
            "nested": {
                "deeply": {
                    "value": 42
                }
            }
        });

        let config = ChildWorkflowOrchestration::new("Test")
            .with_input("complex", nested);

        let ts = config.to_typescript();
        assert!(ts.contains("nested"));
    }

    #[test]
    fn test_multiple_compensations_in_saga() {
        use radium_workflow::schema::patterns::WorkflowPattern;

        let saga = SagaDefinition::new("MultiCompensation")
            .with_step(
                SagaStep::new("step1", SagaAction::activity("action1"))
                    .with_compensation(SagaAction::activity("compensate1")),
            )
            .with_step(
                SagaStep::new("step2", SagaAction::activity("action2"))
                    .with_compensation(SagaAction::activity("compensate2")),
            )
            .with_step(
                SagaStep::new("step3", SagaAction::activity("action3"))
                    .with_compensation(SagaAction::activity("compensate3")),
            );

        let ts = saga.to_typescript();
        assert!(ts.contains("compensate1"));
        assert!(ts.contains("compensate2"));
        assert!(ts.contains("compensate3"));
    }

    #[test]
    fn test_all_gather_strategies() {
        use radium_workflow::schema::patterns::WorkflowPattern;

        // WaitAll
        let sg = ScatterGatherDefinition::new("WaitAll")
            .with_scatter(ScatterConfig::with_workers(vec![ScatterWorker::activity("w")]))
            .with_gather(GatherConfig::wait_all());
        let ts = sg.to_typescript();
        assert!(ts.contains("Promise.allSettled"));

        // WaitFirst
        let sg = ScatterGatherDefinition::new("WaitFirst")
            .with_scatter(ScatterConfig::with_workers(vec![ScatterWorker::activity("w")]))
            .with_gather(GatherConfig::wait_first());
        let ts = sg.to_typescript();
        assert!(ts.contains("Promise.race"));

        // WaitThreshold
        let sg = ScatterGatherDefinition::new("WaitThreshold")
            .with_scatter(ScatterConfig::with_workers(vec![
                ScatterWorker::activity("w1"),
                ScatterWorker::activity("w2"),
                ScatterWorker::activity("w3"),
            ]))
            .with_gather(GatherConfig::wait_threshold(2));
        let ts = sg.to_typescript();
        assert!(ts.contains("threshold = 2"));
    }

    #[test]
    fn test_all_reducer_types() {
        use radium_workflow::schema::patterns::WorkflowPattern;

        // Sum
        let mr = MapReduceDefinition::new("Sum")
            .with_reduce(ReduceConfig::sum());
        let ts = mr.to_typescript();
        assert!(ts.contains("reduce"));

        // Concat
        let mr = MapReduceDefinition::new("Concat")
            .with_reduce(ReduceConfig::concat());
        let ts = mr.to_typescript();
        assert!(ts.contains("flat"));

        // Merge
        let mr = MapReduceDefinition::new("Merge")
            .with_reduce(ReduceConfig::merge());
        let ts = mr.to_typescript();
        assert!(ts.contains("...acc"));

        // Custom
        let mr = MapReduceDefinition::new("Custom")
            .with_reduce(ReduceConfig::custom("customReduce(mapResults)"));
        let ts = mr.to_typescript();
        assert!(ts.contains("customReduce"));

        // Activity
        let mr = MapReduceDefinition::new("Activity")
            .with_reduce(ReduceConfig::activity("reduceActivity"));
        let ts = mr.to_typescript();
        assert!(ts.contains("activities.reduceActivity"));
    }

    #[test]
    fn test_all_search_attribute_types() {
        let mut attrs = WorkflowSearchAttributes::new();

        attrs.add_definition(SearchAttributeDefinition::new("BoolAttr", SearchAttributeType::Bool));
        attrs.add_definition(SearchAttributeDefinition::new("DateAttr", SearchAttributeType::Datetime));
        attrs.add_definition(SearchAttributeDefinition::new("DoubleAttr", SearchAttributeType::Double));
        attrs.add_definition(SearchAttributeDefinition::new("IntAttr", SearchAttributeType::Int));
        attrs.add_definition(SearchAttributeDefinition::new("KeywordAttr", SearchAttributeType::Keyword));
        attrs.add_definition(SearchAttributeDefinition::new("KeywordListAttr", SearchAttributeType::KeywordList));
        attrs.add_definition(SearchAttributeDefinition::new("TextAttr", SearchAttributeType::Text));

        let ts = attrs.to_typescript_interface();

        assert!(ts.contains("boolean"));
        assert!(ts.contains("Date"));
        assert!(ts.contains("number"));
        assert!(ts.contains("string"));
        assert!(ts.contains("string[]"));
    }

    #[test]
    fn test_unicode_in_signal_names() {
        // Test that Unicode characters are handled gracefully
        let signal = SignalDefinition::new("approve_订单");
        let ts = signal.to_typescript_definition();
        // Should not panic and should generate some output
        assert!(!ts.is_empty());
    }

    #[test]
    fn test_very_long_names() {
        let long_name = "a".repeat(1000);
        let signal = SignalDefinition::new(&long_name);
        let ts = signal.to_typescript_definition();
        // Should handle without crashing
        assert!(!ts.is_empty());
    }

    #[test]
    fn test_zero_max_concurrency_validation() {
        let mr = MapReduceDefinition::new("Test")
            .with_max_concurrency(0);

        let result = mr.validate_pattern();
        assert!(result.is_err(), "Zero concurrency should fail validation");
    }

    #[test]
    fn test_batch_size_zero_validation() {
        let mr = MapReduceDefinition::new("Test")
            .with_batch_size(0);

        let result = mr.validate_pattern();
        assert!(result.is_err(), "Zero batch size should fail validation");
    }

    #[test]
    fn test_threshold_exceeds_workers_validation() {
        let sg = ScatterGatherDefinition::new("Test")
            .with_scatter(ScatterConfig::with_workers(vec![ScatterWorker::activity("w")]))
            .with_gather(GatherConfig::wait_threshold(5));

        let result = sg.validate_pattern();
        assert!(result.is_err(), "Threshold exceeding workers should fail validation");
    }

    #[test]
    fn test_empty_workers_validation() {
        let sg = ScatterGatherDefinition::new("Test");

        let result = sg.validate_pattern();
        assert!(result.is_err(), "Empty workers should fail validation");
    }

    #[test]
    fn test_empty_stages_validation() {
        let pipeline = PipelineDefinition::new("Test");

        let result = pipeline.validate_pattern();
        assert!(result.is_err(), "Empty stages should fail validation");
    }

    #[test]
    fn test_empty_saga_steps_validation() {
        let saga = SagaDefinition::new("Test");

        let result = saga.validate_pattern();
        assert!(result.is_err(), "Empty saga steps should fail validation");
    }

    #[test]
    fn test_multiple_signal_handlers() {
        let mut signals = WorkflowSignals::new();

        signals.add(SignalWithHandler::new(
            SignalDefinition::new("update"),
            SignalHandler::new("update"),
        ));

        signals.add(SignalWithHandler::new(
            SignalDefinition::new("cancel"),
            SignalHandler::new("cancel"),
        ));

        // The typescript output should contain both handlers
        let ts = signals.to_typescript();
        assert!(ts.contains("update"));
        assert!(ts.contains("cancel"));
    }

    #[test]
    fn test_json_array_values_in_input() {
        let config = ChildWorkflowOrchestration::new("Test")
            .with_input("items", serde_json::json!(["a", "b", "c"]));

        let ts = config.to_typescript();
        assert!(ts.contains("["));
        assert!(ts.contains("]"));
    }

    #[test]
    fn test_boolean_json_values() {
        let config = ChildWorkflowOrchestration::new("Test")
            .with_input("active", serde_json::json!(true))
            .with_input("disabled", serde_json::json!(false));

        let ts = config.to_typescript();
        assert!(ts.contains("true"));
        assert!(ts.contains("false"));
    }

    #[test]
    fn test_numeric_json_values() {
        let config = ChildWorkflowOrchestration::new("Test")
            .with_input("integer", serde_json::json!(42))
            .with_input("float", serde_json::json!(3.14))
            .with_input("negative", serde_json::json!(-100));

        let ts = config.to_typescript();
        assert!(ts.contains("42"));
        assert!(ts.contains("3.14"));
        assert!(ts.contains("-100"));
    }

    #[test]
    fn test_signal_schema_with_fields() {
        // Use SignalSchema::with_fields which accepts a Vec of SignalSchemaField
        let schema = SignalSchema::with_fields(vec![
            SignalSchemaField {
                name: "stringField".to_string(),
                typescript_type: "string".to_string(),
                required: true,
                description: None,
                default: None,
            },
            SignalSchemaField {
                name: "numberField".to_string(),
                typescript_type: "number".to_string(),
                required: true,
                description: None,
                default: None,
            },
        ]);

        let ts = schema.to_typescript_interface("TestPayload");

        assert!(ts.contains("stringField: string"));
        assert!(ts.contains("numberField: number"));
    }

    #[test]
    fn test_query_schema_with_fields() {
        let schema = QuerySchema::object(vec![
            ("required", "string"),
        ]);

        let ts = schema.to_typescript_interface("TestQuery");
        assert!(ts.contains("required: string"));
    }
}

// =============================================================================
// TYPESCRIPT SYNTAX VALIDATION TESTS
// =============================================================================

mod typescript_syntax_validation {
    use super::*;
    use radium_workflow::schema::patterns::WorkflowPattern;

    /// Validates that generated TypeScript has balanced braces
    fn validate_balanced_braces(code: &str) -> bool {
        let mut brace_count = 0;
        let mut paren_count = 0;
        let mut bracket_count = 0;

        for ch in code.chars() {
            match ch {
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                '(' => paren_count += 1,
                ')' => paren_count -= 1,
                '[' => bracket_count += 1,
                ']' => bracket_count -= 1,
                _ => {}
            }

            // Should never go negative
            if brace_count < 0 || paren_count < 0 || bracket_count < 0 {
                return false;
            }
        }

        brace_count == 0 && paren_count == 0 && bracket_count == 0
    }

    /// Validates that generated TypeScript has proper string handling
    fn validate_no_unclosed_strings(code: &str) -> bool {
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut in_template = false;
        let mut prev_char = ' ';

        for ch in code.chars() {
            if prev_char != '\\' {
                match ch {
                    '\'' if !in_double_quote && !in_template => in_single_quote = !in_single_quote,
                    '"' if !in_single_quote && !in_template => in_double_quote = !in_double_quote,
                    '`' if !in_single_quote && !in_double_quote => in_template = !in_template,
                    _ => {}
                }
            }
            prev_char = ch;
        }

        !in_single_quote && !in_double_quote && !in_template
    }

    #[test]
    fn test_saga_typescript_has_balanced_braces() {
        let saga = SagaDefinition::new("OrderSaga")
            .with_step(SagaStep::new("reserve", SagaAction::activity("reserveInventory"))
                .with_compensation(SagaAction::activity("releaseInventory")))
            .with_step(SagaStep::new("charge", SagaAction::activity("chargePayment"))
                .with_compensation(SagaAction::activity("refundPayment")));

        let ts = saga.to_typescript();
        assert!(validate_balanced_braces(&ts), "Saga TypeScript has unbalanced braces:\n{}", ts);
        assert!(validate_no_unclosed_strings(&ts), "Saga TypeScript has unclosed strings:\n{}", ts);
    }

    #[test]
    fn test_scatter_gather_typescript_has_balanced_braces() {
        let sg = ScatterGatherDefinition::new("ParallelProcess")
            .with_scatter(ScatterConfig::with_workers(vec![
                ScatterWorker::activity("worker1"),
                ScatterWorker::activity("worker2"),
                ScatterWorker::child_workflow("SubWorkflow"),
            ]))
            .with_gather(GatherConfig::wait_all())
            .with_timeout(30000);

        let ts = sg.to_typescript();
        assert!(validate_balanced_braces(&ts), "ScatterGather TypeScript has unbalanced braces:\n{}", ts);
        assert!(validate_no_unclosed_strings(&ts), "ScatterGather TypeScript has unclosed strings:\n{}", ts);
    }

    #[test]
    fn test_pipeline_typescript_has_balanced_braces() {
        let pipeline = PipelineDefinition::new("DataPipeline")
            .with_stage(PipelineStage::activity("validate"))
            .with_stage(PipelineStage::activity("transform").with_retries(3))
            .with_stage(PipelineStage::activity("enrich"))
            .track_intermediate();

        let ts = pipeline.to_typescript();
        assert!(validate_balanced_braces(&ts), "Pipeline TypeScript has unbalanced braces:\n{}", ts);
        assert!(validate_no_unclosed_strings(&ts), "Pipeline TypeScript has unclosed strings:\n{}", ts);
    }

    #[test]
    fn test_map_reduce_typescript_has_balanced_braces() {
        let mr = MapReduceDefinition::new("ProcessItems")
            .with_map(MapConfig::activity("processItem"))
            .with_reduce(ReduceConfig::sum())
            .with_max_concurrency(5)
            .with_batch_size(100);

        let ts = mr.to_typescript();
        assert!(validate_balanced_braces(&ts), "MapReduce TypeScript has unbalanced braces:\n{}", ts);
        assert!(validate_no_unclosed_strings(&ts), "MapReduce TypeScript has unclosed strings:\n{}", ts);
    }

    #[test]
    fn test_map_reduce_with_batching_has_balanced_braces() {
        let mr = MapReduceDefinition::new("BatchedProcess")
            .with_map(MapConfig::activity("processItem"))
            .with_reduce(ReduceConfig::concat())
            .with_batch_size(50);

        let ts = mr.to_typescript();
        assert!(validate_balanced_braces(&ts), "Batched MapReduce TypeScript has unbalanced braces:\n{}", ts);
    }

    #[test]
    fn test_map_reduce_continue_on_failure_has_balanced_braces() {
        let mr = MapReduceDefinition::new("ResilientProcess")
            .with_map(MapConfig::activity("processItem"))
            .continue_on_failure();

        let ts = mr.to_typescript();
        assert!(validate_balanced_braces(&ts), "Continue-on-failure MapReduce TypeScript has unbalanced braces:\n{}", ts);
    }

    #[test]
    fn test_child_workflow_typescript_has_balanced_braces() {
        let config = ChildWorkflowOrchestration::new("ProcessOrder")
            .with_workflow_id("order-123")
            .with_task_queue("high-priority")
            .with_input("orderId", serde_json::json!("order-123"))
            .with_input("nested", serde_json::json!({"a": {"b": 1}}))
            .with_execution_timeout(60000)
            .with_parent_close_policy(ParentClosePolicy::Terminate);

        let ts = config.to_typescript();
        assert!(validate_balanced_braces(&ts), "ChildWorkflow TypeScript has unbalanced braces:\n{}", ts);
        assert!(validate_no_unclosed_strings(&ts), "ChildWorkflow TypeScript has unclosed strings:\n{}", ts);
    }

    #[test]
    fn test_signal_handler_typescript_has_balanced_braces() {
        let handler = SignalHandler::new("updateOrder")
            .with_update(VariableUpdate::new(
                "orderStatus",
                VariableSource::from_payload("status"),
            ));

        let signal = SignalDefinition::new("updateOrder");

        let ts = handler.to_typescript(&signal);
        assert!(validate_balanced_braces(&ts), "SignalHandler TypeScript has unbalanced braces:\n{}", ts);
        assert!(validate_no_unclosed_strings(&ts), "SignalHandler TypeScript has unclosed strings:\n{}", ts);
    }

    #[test]
    fn test_query_handler_typescript_has_balanced_braces() {
        let query = QueryDefinition::new(
            "getOrderStatus",
            QuerySchema::object(vec![
                ("status", "string"),
                ("items", "number"),
            ]),
            QueryHandlerLogic::project(vec!["status", "itemCount"]),
        );

        let ts = query.to_typescript();
        assert!(validate_balanced_braces(&ts), "QueryHandler TypeScript has unbalanced braces:\n{}", ts);
        assert!(validate_no_unclosed_strings(&ts), "QueryHandler TypeScript has unclosed strings:\n{}", ts);
    }

    #[test]
    fn test_cancellation_scope_typescript_has_balanced_braces() {
        let scope = CancellationScope::new("cleanup")
            .with_cleanup(CleanupConfig::new()
                .with_activity(CleanupActivity::new("releaseResources"))
                .with_activity(CleanupActivity::new("sendNotification")));

        let ts = scope.to_typescript("await activities.doWork();");
        assert!(validate_balanced_braces(&ts), "CancellationScope TypeScript has unbalanced braces:\n{}", ts);
        assert!(validate_no_unclosed_strings(&ts), "CancellationScope TypeScript has unclosed strings:\n{}", ts);
    }

    #[test]
    fn test_versioning_typescript_has_balanced_braces() {
        let config = VersioningConfig::new("2.0.0");

        let ts = config.to_typescript_version_constant();
        assert!(validate_balanced_braces(&ts), "Versioning TypeScript has unbalanced braces:\n{}", ts);
        assert!(validate_no_unclosed_strings(&ts), "Versioning TypeScript has unclosed strings:\n{}", ts);

        // Also check change points
        let change_points_ts = config.to_typescript_change_points();
        assert!(validate_balanced_braces(&change_points_ts), "Change points TypeScript has unbalanced braces:\n{}", change_points_ts);
    }

    #[test]
    fn test_version_change_point_typescript_has_balanced_braces() {
        let change_point = VersionChangePoint::new("feature-v2", "2.0.0", "New feature implementation")
            .with_branch(
                VersionBranch::from_version("2.0.0", "await activities.newImplementation();")
                    .with_description("New implementation")
            )
            .with_branch(
                VersionBranch::before_version("2.0.0", "await activities.legacyImplementation();")
                    .with_description("Legacy implementation")
            );

        let ts = change_point.to_typescript();
        assert!(validate_balanced_braces(&ts), "VersionChangePoint TypeScript has unbalanced braces:\n{}", ts);
        assert!(validate_no_unclosed_strings(&ts), "VersionChangePoint TypeScript has unclosed strings:\n{}", ts);
    }

    #[test]
    fn test_workflow_signals_typescript_has_balanced_braces() {
        let mut signals = WorkflowSignals::new();
        signals.add(SignalWithHandler::new(
            SignalDefinition::new("update"),
            SignalHandler::new("update"),
        ));
        signals.add(SignalWithHandler::new(
            SignalDefinition::new("cancel"),
            SignalHandler::new("cancel"),
        ));

        let ts = signals.to_typescript();
        assert!(validate_balanced_braces(&ts), "WorkflowSignals TypeScript has unbalanced braces:\n{}", ts);
        assert!(validate_no_unclosed_strings(&ts), "WorkflowSignals TypeScript has unclosed strings:\n{}", ts);
    }

    #[test]
    fn test_workflow_queries_typescript_has_balanced_braces() {
        let mut queries = WorkflowQueries::new();
        queries.add(QueryDefinition::new(
            "getStatus",
            QuerySchema::object(vec![("status", "string")]),
            QueryHandlerLogic::project(vec!["status"]),
        ));

        let ts = queries.to_typescript();
        assert!(validate_balanced_braces(&ts), "WorkflowQueries TypeScript has unbalanced braces:\n{}", ts);
        assert!(validate_no_unclosed_strings(&ts), "WorkflowQueries TypeScript has unclosed strings:\n{}", ts);
    }

    #[test]
    fn test_search_attributes_typescript_has_balanced_braces() {
        let mut attrs = WorkflowSearchAttributes::new();
        attrs.add_definition(
            SearchAttributeDefinition::new("CustomerId", SearchAttributeType::Keyword)
        );
        attrs.add_definition(
            SearchAttributeDefinition::new("ProcessedCount", SearchAttributeType::Int)
        );

        let ts = attrs.to_typescript_interface();
        assert!(validate_balanced_braces(&ts), "SearchAttributes TypeScript has unbalanced braces:\n{}", ts);
        assert!(validate_no_unclosed_strings(&ts), "SearchAttributes TypeScript has unclosed strings:\n{}", ts);
    }

    #[test]
    fn test_all_scatter_gather_strategies_have_balanced_braces() {
        // WaitAll
        let sg = ScatterGatherDefinition::new("Test")
            .with_scatter(ScatterConfig::with_workers(vec![ScatterWorker::activity("w")]))
            .with_gather(GatherConfig::wait_all());
        assert!(validate_balanced_braces(&sg.to_typescript()));

        // WaitFirst
        let sg = ScatterGatherDefinition::new("Test")
            .with_scatter(ScatterConfig::with_workers(vec![ScatterWorker::activity("w")]))
            .with_gather(GatherConfig::wait_first());
        assert!(validate_balanced_braces(&sg.to_typescript()));

        // WaitThreshold
        let sg = ScatterGatherDefinition::new("Test")
            .with_scatter(ScatterConfig::with_workers(vec![
                ScatterWorker::activity("w1"),
                ScatterWorker::activity("w2"),
            ]))
            .with_gather(GatherConfig::wait_threshold(1));
        assert!(validate_balanced_braces(&sg.to_typescript()));
    }

    #[test]
    fn test_all_workflow_id_strategies_have_balanced_braces() {
        // UUID
        let config = ChildWorkflowOrchestration::new("Test");
        assert!(validate_balanced_braces(&config.to_typescript()));

        // Explicit
        let config = ChildWorkflowOrchestration::new("Test")
            .with_workflow_id("test-id");
        assert!(validate_balanced_braces(&config.to_typescript()));

        // Pattern
        let config = ChildWorkflowOrchestration::new("Test")
            .with_id_pattern("child-{parent_id}-{index}");
        assert!(validate_balanced_braces(&config.to_typescript()));

        // ParentSuffix
        let config = ChildWorkflowOrchestration::new("Test")
            .with_parent_suffix();
        assert!(validate_balanced_braces(&config.to_typescript()));
    }
}

// =============================================================================
// SERIALIZATION CONSISTENCY TESTS
// =============================================================================

mod serialization_consistency_tests {
    use super::*;
    

    /// Test that JSON serialization uses camelCase (TypeScript convention)
    fn assert_camel_case_keys(json: &str, keys: &[&str]) {
        for key in keys {
            assert!(
                json.contains(&format!("\"{}\"", key)),
                "Expected camelCase key '{}' not found in JSON: {}",
                key,
                json
            );
        }
    }

    #[test]
    fn test_child_workflow_serializes_to_camel_case() {
        let config = ChildWorkflowOrchestration::new("Test")
            .with_task_queue("queue")
            .with_execution_timeout(1000);

        let json = serde_json::to_string(&config).unwrap();
        assert_camel_case_keys(&json, &["workflowType", "taskQueue", "executionTimeoutMs", "idStrategy"]);
    }

    #[test]
    fn test_signal_definition_serializes_to_camel_case() {
        let signal = SignalDefinition::new("test")
            .with_description("Test signal")
            .with_buffering(SignalBuffering::Immediate);

        let json = serde_json::to_string(&signal).unwrap();
        // SignalDefinition uses 'external' field with camelCase 'inputSchema'
        assert_camel_case_keys(&json, &["inputSchema"]);
    }

    #[test]
    fn test_saga_serializes_to_camel_case() {
        let saga = SagaDefinition::new("Test")
            .with_step(SagaStep::new("step", SagaAction::activity("act")));

        let json = serde_json::to_string(&saga).unwrap();
        assert_camel_case_keys(&json, &["compensationBehavior", "parallelCompensation", "compensationTimeoutMs"]);
    }

    #[test]
    fn test_map_reduce_serializes_to_camel_case() {
        let mr = MapReduceDefinition::new("Test")
            .with_max_concurrency(5)
            .with_batch_size(10);

        let json = serde_json::to_string(&mr).unwrap();
        assert_camel_case_keys(&json, &["maxConcurrency", "batchSize", "continueOnFailure"]);
    }

    #[test]
    fn test_scatter_gather_serializes_to_camel_case() {
        let sg = ScatterGatherDefinition::new("Test")
            .with_scatter(ScatterConfig::with_workers(vec![ScatterWorker::activity("w")]))
            .with_timeout(1000);

        let json = serde_json::to_string(&sg).unwrap();
        assert_camel_case_keys(&json, &["timeoutMs", "partialResultsOnTimeout", "maxConcurrency"]);
    }

    #[test]
    fn test_pipeline_serializes_to_camel_case() {
        let pipeline = PipelineDefinition::new("Test")
            .with_stage(PipelineStage::activity("step").with_retries(3));

        let json = serde_json::to_string(&pipeline).unwrap();
        assert_camel_case_keys(&json, &["errorHandling", "trackIntermediateResults", "retryCount"]);
    }

    #[test]
    fn test_versioning_serializes_to_camel_case() {
        let config = VersioningConfig::new("1.0.0");

        let json = serde_json::to_string(&config).unwrap();
        assert_camel_case_keys(&json, &["currentVersion", "changePoints"]);
    }

    #[test]
    fn test_search_attributes_serializes_to_camel_case() {
        let attr = SearchAttributeDefinition::new("Test", SearchAttributeType::Keyword);

        let json = serde_json::to_string(&attr).unwrap();
        assert_camel_case_keys(&json, &["attributeType"]);
    }

    #[test]
    fn test_cancellation_scope_serializes_to_camel_case() {
        let scope = CancellationScope::new("test")
            .with_cleanup_timeout(1000);

        let json = serde_json::to_string(&scope).unwrap();
        assert_camel_case_keys(&json, &["cleanupTimeoutMs"]);
    }
}

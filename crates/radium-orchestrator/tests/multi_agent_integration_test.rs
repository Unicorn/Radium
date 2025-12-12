//! Integration tests for multi-agent parallel execution
//!
//! Tests concurrent execution, timeout handling, cancellation, and partial failures
//! to ensure the orchestration engine handles multi-agent scenarios correctly.

use radium_orchestrator::orchestration::{
    execution::execute_tool_calls,
    tool::{Tool, ToolArguments, ToolCall, ToolHandler, ToolParameters, ToolResult},
    FunctionExecutionStrategy, ToolExecutionConfig,
};
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;
use tokio::time::{sleep, Duration, timeout};

/// Test concurrent execution of multiple tools
#[tokio::test]
async fn test_concurrent_execution_performance() {
    struct DelayedToolHandler {
        delay_ms: u64,
    }

    #[async_trait]
    impl ToolHandler for DelayedToolHandler {
        async fn execute(&self, _args: &ToolArguments) -> radium_orchestrator::error::Result<ToolResult> {
            sleep(Duration::from_millis(self.delay_ms)).await;
            Ok(ToolResult::success("completed".to_string()))
        }
    }

    let tools: Vec<Tool> = (0..5)
        .map(|i| {
            Tool::new(
                &format!("tool_{}", i),
                &format!("tool_{}", i),
                "Test tool",
                ToolParameters::new(),
                Arc::new(DelayedToolHandler { delay_ms: 100 }),
            )
        })
        .collect();

    let calls: Vec<ToolCall> = (0..5)
        .map(|i| ToolCall {
            id: format!("call_{}", i),
            name: format!("tool_{}", i),
            arguments: serde_json::json!({}),
        })
        .collect();

    let config = ToolExecutionConfig {
        execution_strategy: FunctionExecutionStrategy::Concurrent,
        timeout_per_call: Duration::from_secs(5),
        max_parallel: None,
        ..Default::default()
    };

    let start = Instant::now();
    let results = execute_tool_calls(&calls, &tools, &config).await;
    let elapsed = start.elapsed();

    // All should succeed
    assert_eq!(results.len(), 5);
    for result in &results {
        assert!(result.is_ok());
    }

    // Concurrent execution should take ~100ms (not 500ms)
    // Allow some overhead (150ms max)
    assert!(
        elapsed.as_millis() < 150,
        "Concurrent execution took {}ms, expected ~100ms",
        elapsed.as_millis()
    );
}

/// Test timeout handling for individual tools
#[tokio::test]
async fn test_timeout_handling() {
    struct SlowToolHandler {
        delay_ms: u64,
    }

    #[async_trait]
    impl ToolHandler for SlowToolHandler {
        async fn execute(&self, _args: &ToolArguments) -> radium_orchestrator::error::Result<ToolResult> {
            sleep(Duration::from_millis(self.delay_ms)).await;
            Ok(ToolResult::success("completed".to_string()))
        }
    }

    let tools: Vec<Tool> = vec![
        Tool::new(
            "fast_tool",
            "fast_tool",
            "Fast tool",
            ToolParameters::new(),
            Arc::new(SlowToolHandler { delay_ms: 50 }),
        ),
        Tool::new(
            "slow_tool",
            "slow_tool",
            "Slow tool",
            ToolParameters::new(),
            Arc::new(SlowToolHandler { delay_ms: 200 }),
        ),
    ];

    let calls: Vec<ToolCall> = vec![
        ToolCall {
            id: "call1".to_string(),
            name: "fast_tool".to_string(),
            arguments: serde_json::json!({}),
        },
        ToolCall {
            id: "call2".to_string(),
            name: "slow_tool".to_string(),
            arguments: serde_json::json!({}),
        },
    ];

    // Set timeout to 100ms - fast tool should succeed, slow tool should timeout
    let config = ToolExecutionConfig {
        execution_strategy: FunctionExecutionStrategy::Concurrent,
        timeout_per_call: Duration::from_millis(100),
        max_parallel: None,
        ..Default::default()
    };

    let results = execute_tool_calls(&calls, &tools, &config).await;

    assert_eq!(results.len(), 2);
    
    // Fast tool should succeed
    assert!(results[0].is_ok());
    
    // Slow tool should timeout
    assert!(results[1].is_err());
    let error_msg = format!("{}", results[1].as_ref().unwrap_err());
    assert!(
        error_msg.contains("timeout") || error_msg.contains("timed out"),
        "Expected timeout error, got: {}",
        error_msg
    );
}

/// Test partial failure handling
#[tokio::test]
async fn test_partial_failure_handling() {
    struct FailingToolHandler {
        should_fail: bool,
    }

    #[async_trait]
    impl ToolHandler for FailingToolHandler {
        async fn execute(&self, _args: &ToolArguments) -> radium_orchestrator::error::Result<ToolResult> {
            if self.should_fail {
                Err(radium_orchestrator::error::OrchestrationError::Other(
                    "Tool execution failed".to_string(),
                ))
            } else {
                Ok(ToolResult::success("completed".to_string()))
            }
        }
    }

    let tools: Vec<Tool> = vec![
        Tool::new(
            "success_tool1",
            "success_tool1",
            "Success tool 1",
            ToolParameters::new(),
            Arc::new(FailingToolHandler { should_fail: false }),
        ),
        Tool::new(
            "failing_tool",
            "failing_tool",
            "Failing tool",
            ToolParameters::new(),
            Arc::new(FailingToolHandler { should_fail: true }),
        ),
        Tool::new(
            "success_tool2",
            "success_tool2",
            "Success tool 2",
            ToolParameters::new(),
            Arc::new(FailingToolHandler { should_fail: false }),
        ),
    ];

    let calls: Vec<ToolCall> = vec![
        ToolCall {
            id: "call1".to_string(),
            name: "success_tool1".to_string(),
            arguments: serde_json::json!({}),
        },
        ToolCall {
            id: "call2".to_string(),
            name: "failing_tool".to_string(),
            arguments: serde_json::json!({}),
        },
        ToolCall {
            id: "call3".to_string(),
            name: "success_tool2".to_string(),
            arguments: serde_json::json!({}),
        },
    ];

    let config = ToolExecutionConfig {
        execution_strategy: FunctionExecutionStrategy::Concurrent,
        timeout_per_call: Duration::from_secs(5),
        max_parallel: None,
        ..Default::default()
    };

    let results = execute_tool_calls(&calls, &tools, &config).await;

    assert_eq!(results.len(), 3);

    // First tool should succeed
    assert!(results[0].is_ok());
    assert_eq!(results[0].as_ref().unwrap().output, "completed");

    // Second tool should fail
    assert!(results[1].is_err());

    // Third tool should succeed (partial failure doesn't stop other tools)
    assert!(results[2].is_ok());
    assert_eq!(results[2].as_ref().unwrap().output, "completed");
}

/// Test that cancellation doesn't corrupt state
#[tokio::test]
async fn test_cancellation_no_state_corruption() {
    let cancellation_flag = Arc::new(AtomicBool::new(false));
    let execution_count = Arc::new(AtomicUsize::new(0));

    struct CancellableToolHandler {
        cancellation_flag: Arc<AtomicBool>,
        execution_count: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl ToolHandler for CancellableToolHandler {
        async fn execute(&self, _args: &ToolArguments) -> radium_orchestrator::error::Result<ToolResult> {
            self.execution_count.fetch_add(1, Ordering::SeqCst);

            // Simulate work with periodic cancellation checks
            for _ in 0..10 {
                if self.cancellation_flag.load(Ordering::SeqCst) {
                    return Err(radium_orchestrator::error::OrchestrationError::Other(
                        "Cancelled".to_string(),
                    ));
                }
                sleep(Duration::from_millis(10)).await;
            }

            Ok(ToolResult::success("completed".to_string()))
        }
    }

    let tools: Vec<Tool> = (0..3)
        .map(|i| {
            Tool::new(
                &format!("tool_{}", i),
                &format!("tool_{}", i),
                "Test tool",
                ToolParameters::new(),
                Arc::new(CancellableToolHandler {
                    cancellation_flag: Arc::clone(&cancellation_flag),
                    execution_count: Arc::clone(&execution_count),
                }),
            )
        })
        .collect();

    let calls: Vec<ToolCall> = (0..3)
        .map(|i| ToolCall {
            id: format!("call_{}", i),
            name: format!("tool_{}", i),
            arguments: serde_json::json!({}),
        })
        .collect();

    let config = ToolExecutionConfig {
        execution_strategy: FunctionExecutionStrategy::Concurrent,
        timeout_per_call: Duration::from_secs(5),
        max_parallel: None,
        ..Default::default()
    };

    // Start execution
    let execution_handle = tokio::spawn(async move {
        execute_tool_calls(&calls, &tools, &config).await
    });

    // Cancel after a short delay
    sleep(Duration::from_millis(50)).await;
    cancellation_flag.store(true, Ordering::SeqCst);

    // Wait for execution to complete (should handle cancellation gracefully)
    let results = timeout(Duration::from_secs(2), execution_handle)
        .await
        .expect("Execution should complete within timeout");

    // Verify execution count (should have started executing)
    let count = execution_count.load(Ordering::SeqCst);
    assert!(count > 0, "Tools should have started executing");

    // Results may be errors due to cancellation, but should not panic or corrupt state
    if let Ok(results) = results {
        assert_eq!(results.len(), 3, "Should return results for all tools");
        // Some may be errors due to cancellation, which is expected
    }
}

/// Test that concurrent execution maintains result ordering
#[tokio::test]
async fn test_concurrent_execution_result_ordering() {
    struct OrderedToolHandler {
        index: usize,
        delay_ms: u64,
    }

    #[async_trait]
    impl ToolHandler for OrderedToolHandler {
        async fn execute(&self, _args: &ToolArguments) -> radium_orchestrator::error::Result<ToolResult> {
            sleep(Duration::from_millis(self.delay_ms)).await;
            Ok(ToolResult::success(format!("result_{}", self.index)))
        }
    }

    // Create tools with varying delays (will complete out of order)
    let tools: Vec<Tool> = (0..5)
        .map(|i| {
            // Reverse delay order: tool 0 takes longest, tool 4 takes shortest
            let delay_ms = (5 - i) * 20;
            Tool::new(
                &format!("tool_{}", i),
                &format!("tool_{}", i),
                "Test tool",
                ToolParameters::new(),
                Arc::new(OrderedToolHandler {
                    index: i,
                    delay_ms,
                }),
            )
        })
        .collect();

    let calls: Vec<ToolCall> = (0..5)
        .map(|i| ToolCall {
            id: format!("call_{}", i),
            name: format!("tool_{}", i),
            arguments: serde_json::json!({}),
        })
        .collect();

    let config = ToolExecutionConfig {
        execution_strategy: FunctionExecutionStrategy::Concurrent,
        timeout_per_call: Duration::from_secs(5),
        max_parallel: None,
        ..Default::default()
    };

    let results = execute_tool_calls(&calls, &tools, &config).await;

    assert_eq!(results.len(), 5);

    // Results should be in call order, not completion order
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok());
        assert_eq!(
            result.as_ref().unwrap().output,
            format!("result_{}", i),
            "Result {} should match call order",
            i
        );
    }
}

/// Test max_parallel limit enforcement
#[tokio::test]
async fn test_max_parallel_limit() {
    let concurrent_count = Arc::new(AtomicUsize::new(0));
    let max_concurrent = Arc::new(AtomicUsize::new(0));

    struct LimitedToolHandler {
        concurrent_count: Arc<AtomicUsize>,
        max_concurrent: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl ToolHandler for LimitedToolHandler {
        async fn execute(&self, _args: &ToolArguments) -> radium_orchestrator::error::Result<ToolResult> {
            let current = self.concurrent_count.fetch_add(1, Ordering::SeqCst);
            let max = self.max_concurrent.load(Ordering::SeqCst);
            if current + 1 > max {
                self.max_concurrent.store(current + 1, Ordering::SeqCst);
            }

            sleep(Duration::from_millis(50)).await;

            self.concurrent_count.fetch_sub(1, Ordering::SeqCst);
            Ok(ToolResult::success("completed".to_string()))
        }
    }

    let tools: Vec<Tool> = (0..10)
        .map(|i| {
            Tool::new(
                &format!("tool_{}", i),
                &format!("tool_{}", i),
                "Test tool",
                ToolParameters::new(),
                Arc::new(LimitedToolHandler {
                    concurrent_count: Arc::clone(&concurrent_count),
                    max_concurrent: Arc::clone(&max_concurrent),
                }),
            )
        })
        .collect();

    let calls: Vec<ToolCall> = (0..10)
        .map(|i| ToolCall {
            id: format!("call_{}", i),
            name: format!("tool_{}", i),
            arguments: serde_json::json!({}),
        })
        .collect();

    // Set max_parallel to 3
    let config = ToolExecutionConfig {
        execution_strategy: FunctionExecutionStrategy::Concurrent,
        timeout_per_call: Duration::from_secs(5),
        max_parallel: Some(3),
        ..Default::default()
    };

    let results = execute_tool_calls(&calls, &tools, &config).await;

    assert_eq!(results.len(), 10);
    for result in &results {
        assert!(result.is_ok());
    }

    // Max concurrent should not exceed 3 (allow some tolerance for timing)
    let max = max_concurrent.load(Ordering::SeqCst);
    assert!(
        max <= 3,
        "Max concurrent executions should be <= 3, got {}",
        max
    );
}

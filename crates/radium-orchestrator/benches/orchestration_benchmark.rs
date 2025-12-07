//! Performance benchmarks for the orchestration system.
//!
//! Measures orchestration overhead to ensure <500ms overhead for orchestration layer
//! (excluding actual model API calls).
//!
//! ## Performance Targets
//!
//! The orchestration system must maintain low overhead to ensure responsive user experience:
//! - Engine creation: < 10µs
//! - Provider selection: < 1µs
//! - Tool registry build (100 tools): < 10ms
//! - Single tool call overhead: < 5ms
//! - Multi-tool iteration (5 iterations): < 50ms
//! - **Full orchestration flow overhead: < 500ms** (excluding API calls)
//!
//! These benchmarks use mock providers with 0ms response time to isolate orchestration
//! overhead from API latency. Real-world performance will include API call times which
//! are outside the scope of orchestration layer optimization.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use radium_orchestrator::orchestration::{
    FinishReason, OrchestrationProvider, OrchestrationResult,
    context::OrchestrationContext,
    engine::{EngineConfig, OrchestrationEngine},
    tool::{Tool, ToolArguments, ToolCall, ToolParameters, ToolResult, ToolHandler},
};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Mock provider that returns immediate results (no API calls)
struct MockBenchmarkProvider {
    response_time_ms: u64,
}

impl MockBenchmarkProvider {
    fn new(response_time_ms: u64) -> Self {
        Self { response_time_ms }
    }
}

#[async_trait]
impl OrchestrationProvider for MockBenchmarkProvider {
    async fn execute_with_tools(
        &self,
        _input: &str,
        _tools: &[Tool],
        _context: &OrchestrationContext,
    ) -> radium_orchestrator::error::Result<OrchestrationResult> {
        // Simulate minimal processing time
        if self.response_time_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.response_time_ms)).await;
        }
        Ok(OrchestrationResult::new(
            "Response".to_string(),
            vec![],
            FinishReason::Stop,
        ))
    }

    fn supports_function_calling(&self) -> bool {
        true
    }

    fn provider_name(&self) -> &'static str {
        "mock_benchmark"
    }
}

/// Mock tool handler that executes immediately
struct MockBenchmarkToolHandler;

#[async_trait]
impl ToolHandler for MockBenchmarkToolHandler {
    async fn execute(
        &self,
        _args: &ToolArguments,
    ) -> radium_orchestrator::error::Result<ToolResult> {
        // Immediate execution - no actual work
        Ok(ToolResult::success("Tool executed".to_string()))
    }
}

fn create_test_tools(count: usize) -> Vec<Tool> {
    (0..count)
        .map(|i| {
            Tool::new(
                format!("agent_{}", i),
                format!("tool_{}", i),
                format!("Test tool {}", i),
                ToolParameters::new().add_property("task", "string", "Task to perform", true),
                Arc::new(MockBenchmarkToolHandler),
            )
        })
        .collect()
}

fn benchmark_engine_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let provider: Arc<dyn OrchestrationProvider> = Arc::new(MockBenchmarkProvider::new(0));
    let tools = create_test_tools(10);

    c.bench_function("orchestration_engine_creation", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(OrchestrationEngine::with_defaults(
                    Arc::clone(&provider),
                    tools.clone(),
                ));
            });
        });
    });
}

fn benchmark_provider_selection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("orchestration_provider_selection", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Simulate provider creation overhead
                let provider = Arc::new(MockBenchmarkProvider::new(0));
                black_box(provider);
            });
        });
    });
}

fn benchmark_tool_registry_build(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    for count in [10, 50, 100] {
        let tools = create_test_tools(count);
        c.bench_function(&format!("orchestration_tool_registry_build_{}tools", count), |b| {
            b.iter(|| {
                rt.block_on(async {
                    // Simulate tool registry operations
                    let _tool_count = black_box(tools.len());
                    let _tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
                    black_box(_tool_names);
                });
            });
        });
    }
}

fn benchmark_single_tool_call(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("orchestration_single_tool_call", |b| {
        b.iter(|| {
            rt.block_on(async {
                let provider: Arc<dyn OrchestrationProvider> = Arc::new(MockBenchmarkProvider::new(0));
                let tool = create_test_tools(1);
                let engine = OrchestrationEngine::with_defaults(provider, tool);
                let mut ctx = OrchestrationContext::new("bench-session");
                let result = engine.execute("Test input", &mut ctx).await.unwrap();
                black_box(result);
            });
        });
    });
}

fn benchmark_multi_tool_iteration(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Create a provider that requests tools for multiple iterations
    struct MultiIterProvider {
        iterations: usize,
        current: Arc<std::sync::atomic::AtomicUsize>,
    }

    #[async_trait]
    impl OrchestrationProvider for MultiIterProvider {
        async fn execute_with_tools(
            &self,
            _input: &str,
            tools: &[Tool],
            _context: &OrchestrationContext,
        ) -> radium_orchestrator::error::Result<OrchestrationResult> {
            let iter = self.current.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            
            if iter < self.iterations && !tools.is_empty() {
                // Request a tool call
                Ok(OrchestrationResult::new(
                    "Calling tool".to_string(),
                    vec![ToolCall {
                        id: format!("call_{}", iter),
                        name: tools[0].name.clone(),
                        arguments: json!({"task": "test"}),
                    }],
                    FinishReason::Stop,
                ))
            } else {
                // Final response
                Ok(OrchestrationResult::new(
                    "Done".to_string(),
                    vec![],
                    FinishReason::Stop,
                ))
            }
        }

        fn supports_function_calling(&self) -> bool {
            true
        }

        fn provider_name(&self) -> &'static str {
            "multi_iter"
        }
    }

    for iteration_count in [2, 3, 5] {
        c.bench_function(&format!("orchestration_multi_tool_iteration_{}iters", iteration_count), |b| {
            b.iter(|| {
                rt.block_on(async {
                    // Reset provider state
                    let provider = Arc::new(MultiIterProvider {
                        iterations: iteration_count,
                        current: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
                    });
                    let tool = create_test_tools(1);
                    let engine = OrchestrationEngine::new(
                        provider,
                        tool,
                        EngineConfig { max_iterations: 10, timeout_seconds: 120 },
                    );
                    let mut ctx = OrchestrationContext::new("bench-session");
                    let result = engine.execute("Test input", &mut ctx).await.unwrap();
                    black_box(result);
                });
            });
        });
    }
}

fn benchmark_context_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("orchestration_context_creation", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(OrchestrationContext::new("bench-session"));
            });
        });
    });

    c.bench_function("orchestration_context_add_message", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut ctx = OrchestrationContext::new("bench-session");
                ctx.add_user_message("Test message");
                black_box(ctx);
            });
        });
    });

    c.bench_function("orchestration_context_conversation_history", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut ctx = OrchestrationContext::new("bench-session");
                for i in 0..10 {
                    ctx.add_user_message(&format!("Message {}", i));
                    ctx.add_assistant_message(&format!("Response {}", i));
                }
                let history_len = ctx.history_length();
                black_box(history_len);
            });
        });
    });
}

fn benchmark_tool_execution_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let handler = Arc::new(MockBenchmarkToolHandler);

    c.bench_function("orchestration_tool_execution_overhead", |b| {
        b.iter(|| {
            rt.block_on(async {
                let args = ToolArguments::new(json!({"task": "test"}));
                let result = handler.execute(&args).await.unwrap();
                black_box(result);
            });
        });
    });
}

/// Benchmark full orchestration flow overhead
/// 
/// This benchmark measures the total orchestration layer overhead (excluding API calls)
/// to validate it meets the <500ms requirement. The mock provider has 0ms response time
/// to isolate orchestration overhead from API latency.
fn benchmark_full_orchestration_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Create a provider with 0ms response time to measure pure orchestration overhead
    let provider: Arc<dyn OrchestrationProvider> = Arc::new(MockBenchmarkProvider::new(0));
    let tools = create_test_tools(10); // Realistic number of tools
    
    c.bench_function("orchestration_full_flow_overhead", |b| {
        b.iter(|| {
            rt.block_on(async {
                let engine = OrchestrationEngine::with_defaults(
                    Arc::clone(&provider),
                    tools.clone(),
                );
                let mut ctx = OrchestrationContext::new("bench-session");
                
                // Measure full orchestration execution (no API calls, just orchestration logic)
                let result = engine.execute("Test orchestration input", &mut ctx).await.unwrap();
                black_box(result);
            });
        });
    });
    
    // Add a benchmark with tool execution to measure overhead with tool calls
    let tool = create_test_tools(1);
    let provider_with_tools: Arc<dyn OrchestrationProvider> = Arc::new(MockBenchmarkProvider::new(0));
    
    c.bench_function("orchestration_full_flow_with_tool_call", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Provider that requests one tool call then finishes
                struct SingleToolProvider {
                    called: Arc<std::sync::atomic::AtomicBool>,
                }
                
                #[async_trait]
                impl OrchestrationProvider for SingleToolProvider {
                    async fn execute_with_tools(
                        &self,
                        _input: &str,
                        tools: &[Tool],
                        _context: &OrchestrationContext,
                    ) -> radium_orchestrator::error::Result<OrchestrationResult> {
                        let was_called = self.called.swap(true, std::sync::atomic::Ordering::SeqCst);
                        if !was_called && !tools.is_empty() {
                            Ok(OrchestrationResult::new(
                                "Calling tool".to_string(),
                                vec![ToolCall {
                                    id: "call_1".to_string(),
                                    name: tools[0].name.clone(),
                                    arguments: json!({"task": "test"}),
                                }],
                                FinishReason::Stop,
                            ))
                        } else {
                            Ok(OrchestrationResult::new(
                                "Done".to_string(),
                                vec![],
                                FinishReason::Stop,
                            ))
                        }
                    }
                    
                    fn supports_function_calling(&self) -> bool {
                        true
                    }
                    
                    fn provider_name(&self) -> &'static str {
                        "single_tool"
                    }
                }
                
                let provider = Arc::new(SingleToolProvider {
                    called: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                });
                let engine = OrchestrationEngine::with_defaults(provider, tool.clone());
                let mut ctx = OrchestrationContext::new("bench-session");
                
                let result = engine.execute("Test with tool", &mut ctx).await.unwrap();
                black_box(result);
            });
        });
    });
}

criterion_group!(
    benches,
    benchmark_engine_creation,
    benchmark_provider_selection,
    benchmark_tool_registry_build,
    benchmark_single_tool_call,
    benchmark_multi_tool_iteration,
    benchmark_context_operations,
    benchmark_tool_execution_overhead,
    benchmark_full_orchestration_overhead
);
criterion_main!(benches);


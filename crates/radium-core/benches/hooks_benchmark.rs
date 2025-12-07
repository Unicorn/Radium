//! Performance benchmarks for the hooks system.
//!
//! Measures hook execution overhead to ensure <5% performance impact.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use radium_core::hooks::registry::{Hook, HookRegistry, HookType};
use radium_core::hooks::types::{HookContext, HookPriority, HookResult as HookExecutionResult};
use std::sync::Arc;
use async_trait::async_trait;

/// Simple test hook for benchmarking.
struct TestHook {
    name: String,
    priority: HookPriority,
}

#[async_trait]
impl Hook for TestHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    fn hook_type(&self) -> HookType {
        HookType::BeforeModel
    }

    async fn execute(&self, _context: &HookContext) -> radium_core::hooks::error::Result<HookExecutionResult> {
        Ok(HookExecutionResult::success())
    }
}

fn benchmark_hook_registry_creation(c: &mut Criterion) {
    c.bench_function("hook_registry_creation", |b| {
        b.iter(|| {
            black_box(HookRegistry::new());
        });
    });
}

fn benchmark_hook_registration(c: &mut Criterion) {
    let registry = Arc::new(HookRegistry::new());
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("hook_registration", |b| {
        b.iter(|| {
            let hook = Arc::new(TestHook {
                name: format!("hook-{}", black_box(0)),
                priority: HookPriority::new(100),
            });
            rt.block_on(async {
                registry.register(hook).await.unwrap();
            });
        });
    });
}

fn benchmark_hook_context_creation(c: &mut Criterion) {
    c.bench_function("hook_context_creation", |b| {
        b.iter(|| {
            let data = serde_json::json!({
                "test": "data",
                "value": 42,
            });
            black_box(HookContext::new("test_hook", data));
        });
    });
}

fn benchmark_hook_execution_no_hooks(c: &mut Criterion) {
    let registry = Arc::new(HookRegistry::new());
    let rt = tokio::runtime::Runtime::new().unwrap();
    let context = HookContext::new("test", serde_json::json!({}));

    c.bench_function("hook_execution_no_hooks", |b| {
        b.iter(|| {
            rt.block_on(async {
                registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
            });
        });
    });
}

fn benchmark_hook_execution_with_hooks(c: &mut Criterion) {
    let registry = Arc::new(HookRegistry::new());
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Register 1, 5, and 10 hooks
    for count in [1, 5, 10] {
        let registry_clone = Arc::clone(&registry);
        rt.block_on(async {
            for i in 0..count {
                let hook = Arc::new(TestHook {
                    name: format!("hook-{}", i),
                    priority: HookPriority::new(100 - i as u32),
                });
                registry_clone.register(hook).await.unwrap();
            }
        });

        let context = HookContext::new("test", serde_json::json!({}));
        c.bench_function(&format!("hook_execution_{}_hooks", count), |b| {
            b.iter(|| {
                rt.block_on(async {
                    registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
                });
            });
        });
    }
}

fn benchmark_hook_result_creation(c: &mut Criterion) {
    c.bench_function("hook_result_success", |b| {
        b.iter(|| {
            black_box(HookExecutionResult::success());
        });
    });

    c.bench_function("hook_result_with_data", |b| {
        b.iter(|| {
            let data = serde_json::json!({"test": "data"});
            black_box(HookExecutionResult::with_data(data));
        });
    });
}

criterion_group!(
    benches,
    benchmark_hook_registry_creation,
    benchmark_hook_registration,
    benchmark_hook_context_creation,
    benchmark_hook_execution_no_hooks,
    benchmark_hook_execution_with_hooks,
    benchmark_hook_result_creation
);
criterion_main!(benches);


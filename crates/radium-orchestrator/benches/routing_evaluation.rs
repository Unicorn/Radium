//! Comprehensive routing evaluation benchmark suite.
//!
//! Compares three routing approaches:
//! 1. Complexity-based routing (baseline)
//! 2. Skill-based routing (Ai-Agent-Skills framework)
//! 3. ML-based routing (Arch-Router 1.5B)
//!
//! Benchmark categories:
//! 1. Isolated routing latency (pure decision time)
//! 2. End-to-end accuracy (correct agent selection)
//! 3. Real workload replay (production task patterns)
//! 4. Stress testing (concurrent load)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use radium_orchestrator::routing::{
    ComplexityEstimator, MLRouter, MLRouterConfig, SkillRouter,
};
use std::time::Duration;
use tokio::runtime::Runtime;

/// Test task with expected routing result.
#[derive(Debug, Clone)]
struct TestTask {
    description: String,
    expected_agent: String,
    task_type: TaskType,
}

/// Task type for categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskType {
    CodeGeneration,
    CodeReview,
    Documentation,
    Debugging,
    Refactoring,
    Testing,
    Architecture,
}

impl TaskType {
    fn as_str(&self) -> &str {
        match self {
            TaskType::CodeGeneration => "code-generation",
            TaskType::CodeReview => "code-review",
            TaskType::Documentation => "documentation",
            TaskType::Debugging => "debugging",
            TaskType::Refactoring => "refactoring",
            TaskType::Testing => "testing",
            TaskType::Architecture => "architecture",
        }
    }
}

/// Create a comprehensive test dataset.
fn create_test_dataset() -> Vec<TestTask> {
    vec![
        // Code generation tasks
        TestTask {
            description: "Write a Python function to calculate fibonacci numbers".to_string(),
            expected_agent: "code-agent".to_string(),
            task_type: TaskType::CodeGeneration,
        },
        TestTask {
            description: "Implement a REST API endpoint for user authentication".to_string(),
            expected_agent: "code-agent".to_string(),
            task_type: TaskType::CodeGeneration,
        },
        TestTask {
            description: "Create a React component for data visualization".to_string(),
            expected_agent: "code-agent".to_string(),
            task_type: TaskType::CodeGeneration,
        },
        TestTask {
            description: "Generate SQL queries for data aggregation".to_string(),
            expected_agent: "code-agent".to_string(),
            task_type: TaskType::CodeGeneration,
        },
        // Code review tasks
        TestTask {
            description: "Review this code for security vulnerabilities".to_string(),
            expected_agent: "review-agent".to_string(),
            task_type: TaskType::CodeReview,
        },
        TestTask {
            description: "Analyze code quality and suggest improvements".to_string(),
            expected_agent: "review-agent".to_string(),
            task_type: TaskType::CodeReview,
        },
        TestTask {
            description: "Check for performance bottlenecks in this function".to_string(),
            expected_agent: "review-agent".to_string(),
            task_type: TaskType::CodeReview,
        },
        // Documentation tasks
        TestTask {
            description: "Write API documentation for this endpoint".to_string(),
            expected_agent: "doc-agent".to_string(),
            task_type: TaskType::Documentation,
        },
        TestTask {
            description: "Generate README with setup instructions".to_string(),
            expected_agent: "doc-agent".to_string(),
            task_type: TaskType::Documentation,
        },
        TestTask {
            description: "Create technical documentation for architecture".to_string(),
            expected_agent: "doc-agent".to_string(),
            task_type: TaskType::Documentation,
        },
        // Debugging tasks
        TestTask {
            description: "Fix this TypeError: 'NoneType' object is not subscriptable".to_string(),
            expected_agent: "debug-agent".to_string(),
            task_type: TaskType::Debugging,
        },
        TestTask {
            description: "Investigate why the API returns 500 errors".to_string(),
            expected_agent: "debug-agent".to_string(),
            task_type: TaskType::Debugging,
        },
        // Refactoring tasks
        TestTask {
            description: "Refactor this code to use dependency injection".to_string(),
            expected_agent: "refactor-agent".to_string(),
            task_type: TaskType::Refactoring,
        },
        TestTask {
            description: "Extract common logic into reusable functions".to_string(),
            expected_agent: "refactor-agent".to_string(),
            task_type: TaskType::Refactoring,
        },
        // Testing tasks
        TestTask {
            description: "Write unit tests for this component".to_string(),
            expected_agent: "test-agent".to_string(),
            task_type: TaskType::Testing,
        },
        TestTask {
            description: "Create integration tests for the API".to_string(),
            expected_agent: "test-agent".to_string(),
            task_type: TaskType::Testing,
        },
        // Architecture tasks
        TestTask {
            description: "Design a microservices architecture for this system".to_string(),
            expected_agent: "architecture-agent".to_string(),
            task_type: TaskType::Architecture,
        },
        TestTask {
            description: "Evaluate trade-offs between SQL and NoSQL databases".to_string(),
            expected_agent: "architecture-agent".to_string(),
            task_type: TaskType::Architecture,
        },
    ]
}

/// Benchmark 1: Isolated routing latency.
fn bench_routing_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let dataset = create_test_dataset();

    let mut group = c.benchmark_group("routing_latency");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(1000);

    // Complexity-based routing (baseline)
    group.bench_function("complexity_based", |b| {
        let estimator = ComplexityEstimator::new();
        b.iter(|| {
            for task in &dataset {
                let _score = estimator.estimate(black_box(&task.description), None);
            }
        });
    });

    // Skill-based routing
    group.bench_function("skill_based", |b| {
        let router = rt.block_on(async {
            let router = SkillRouter::new();
            // Load skills from test data
            router.load_skills_from_directory("skills").await.ok();
            router
        });

        b.to_async(&rt).iter(|| async {
            for task in &dataset {
                let _result = router.route(black_box(&task.description)).await.ok();
            }
        });
    });

    // ML-based routing
    group.bench_function("ml_based", |b| {
        let router = rt.block_on(async {
            MLRouter::new().await.expect("Failed to create ML router")
        });

        b.to_async(&rt).iter(|| async {
            for task in &dataset {
                let _result = router.route(black_box(&task.description)).await.ok();
            }
        });
    });

    group.finish();
}

/// Benchmark 2: Routing accuracy (correct agent selection).
fn bench_routing_accuracy(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let dataset = create_test_dataset();

    let mut group = c.benchmark_group("routing_accuracy");
    group.measurement_time(Duration::from_secs(10));

    // Skill-based routing accuracy
    group.bench_function("skill_based_accuracy", |b| {
        let router = rt.block_on(async {
            let router = SkillRouter::new();
            router.load_skills_from_directory("skills").await.ok();
            router
        });

        b.iter(|| {
            rt.block_on(async {
                let mut correct = 0;
                let mut total = 0;

                for task in &dataset {
                    if let Ok(Some(result)) = router.route(&task.description).await {
                        total += 1;
                        if result.skill_name.contains(&task.expected_agent)
                            || task.expected_agent.contains(&result.skill_name)
                        {
                            correct += 1;
                        }
                    }
                }

                let accuracy = if total > 0 {
                    correct as f64 / total as f64
                } else {
                    0.0
                };

                black_box(accuracy)
            })
        });
    });

    // ML-based routing accuracy
    group.bench_function("ml_based_accuracy", |b| {
        let router = rt.block_on(async {
            let router = MLRouter::new().await.expect("Failed to create ML router");

            // Register test agents
            router
                .register_agent("code-agent".to_string(), "Writes production code".to_string())
                .await;
            router
                .register_agent(
                    "review-agent".to_string(),
                    "Reviews code for quality".to_string(),
                )
                .await;
            router
                .register_agent(
                    "doc-agent".to_string(),
                    "Writes documentation".to_string(),
                )
                .await;
            router
                .register_agent(
                    "debug-agent".to_string(),
                    "Debugs and fixes errors".to_string(),
                )
                .await;
            router
                .register_agent(
                    "refactor-agent".to_string(),
                    "Refactors code for better design".to_string(),
                )
                .await;
            router
                .register_agent("test-agent".to_string(), "Writes tests".to_string())
                .await;
            router
                .register_agent(
                    "architecture-agent".to_string(),
                    "Designs system architecture".to_string(),
                )
                .await;

            router
        });

        b.iter(|| {
            rt.block_on(async {
                let mut correct = 0;
                let mut total = 0;

                for task in &dataset {
                    if let Ok(result) = router.route(&task.description).await {
                        total += 1;
                        if result.agent_id.contains(&task.expected_agent)
                            || task.expected_agent.contains(&result.agent_id)
                        {
                            correct += 1;
                        }
                    }
                }

                let accuracy = if total > 0 {
                    correct as f64 / total as f64
                } else {
                    0.0
                };

                black_box(accuracy)
            })
        });
    });

    group.finish();
}

/// Benchmark 3: Throughput (requests per second).
fn bench_routing_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let dataset = create_test_dataset();

    let mut group = c.benchmark_group("routing_throughput");
    group.throughput(Throughput::Elements(dataset.len() as u64));
    group.measurement_time(Duration::from_secs(10));

    // Complexity-based throughput
    group.bench_function("complexity_based", |b| {
        let estimator = ComplexityEstimator::new();
        b.iter(|| {
            for task in &dataset {
                let _score = estimator.estimate(black_box(&task.description), None);
            }
        });
    });

    // Skill-based throughput
    group.bench_function("skill_based", |b| {
        let router = rt.block_on(async {
            let router = SkillRouter::new();
            router.load_skills_from_directory("skills").await.ok();
            router
        });

        b.to_async(&rt).iter(|| async {
            for task in &dataset {
                let _result = router.route(black_box(&task.description)).await;
            }
        });
    });

    // ML-based throughput
    group.bench_function("ml_based", |b| {
        let router = rt.block_on(async {
            let router = MLRouter::new().await.expect("Failed to create ML router");

            // Register agents
            router
                .register_agent("code-agent".to_string(), "Writes code".to_string())
                .await;
            router
                .register_agent("review-agent".to_string(), "Reviews code".to_string())
                .await;
            router
                .register_agent("doc-agent".to_string(), "Writes docs".to_string())
                .await;

            router
        });

        b.to_async(&rt).iter(|| async {
            for task in &dataset {
                let _result = router.route(black_box(&task.description)).await;
            }
        });
    });

    group.finish();
}

/// Benchmark 4: Concurrent routing stress test.
fn bench_concurrent_routing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let dataset = create_test_dataset();

    let mut group = c.benchmark_group("concurrent_routing");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(50);

    for concurrency in [1, 5, 10, 20] {
        // Skill-based concurrent routing
        group.bench_with_input(
            BenchmarkId::new("skill_based", concurrency),
            &concurrency,
            |b, &concurrency| {
                let router = rt.block_on(async {
                    let router = SkillRouter::new();
                    router.load_skills_from_directory("skills").await.ok();
                    std::sync::Arc::new(router)
                });

                b.to_async(&rt).iter(|| {
                    let router = router.clone();
                    let dataset = dataset.clone();
                    async move {
                        let mut handles = Vec::new();

                        for _ in 0..concurrency {
                            let router = router.clone();
                            let dataset = dataset.clone();
                            let handle = tokio::spawn(async move {
                                for task in &dataset {
                                    let _result = router.route(&task.description).await;
                                }
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            handle.await.ok();
                        }
                    }
                });
            },
        );

        // ML-based concurrent routing
        group.bench_with_input(
            BenchmarkId::new("ml_based", concurrency),
            &concurrency,
            |b, &concurrency| {
                let router = rt.block_on(async {
                    let router = MLRouter::new().await.expect("Failed to create ML router");
                    router
                        .register_agent("code-agent".to_string(), "Writes code".to_string())
                        .await;
                    router
                        .register_agent("review-agent".to_string(), "Reviews code".to_string())
                        .await;
                    router
                        .register_agent("doc-agent".to_string(), "Writes docs".to_string())
                        .await;
                    std::sync::Arc::new(router)
                });

                b.to_async(&rt).iter(|| {
                    let router = router.clone();
                    let dataset = dataset.clone();
                    async move {
                        let mut handles = Vec::new();

                        for _ in 0..concurrency {
                            let router = router.clone();
                            let dataset = dataset.clone();
                            let handle = tokio::spawn(async move {
                                for task in &dataset {
                                    let _result = router.route(&task.description).await;
                                }
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            handle.await.ok();
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark 5: Per-task-type routing performance.
fn bench_task_type_routing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let dataset = create_test_dataset();

    // Group tasks by type
    let mut tasks_by_type: std::collections::HashMap<&str, Vec<&TestTask>> =
        std::collections::HashMap::new();
    for task in &dataset {
        tasks_by_type
            .entry(task.task_type.as_str())
            .or_insert_with(Vec::new)
            .push(task);
    }

    let mut group = c.benchmark_group("task_type_routing");

    for (task_type, tasks) in tasks_by_type {
        if tasks.is_empty() {
            continue;
        }

        // ML-based routing for this task type
        group.bench_with_input(
            BenchmarkId::new("ml_based", task_type),
            &tasks,
            |b, tasks| {
                let router = rt.block_on(async {
                    let router = MLRouter::new().await.expect("Failed to create ML router");
                    router
                        .register_agent("code-agent".to_string(), "Writes code".to_string())
                        .await;
                    router
                        .register_agent("review-agent".to_string(), "Reviews code".to_string())
                        .await;
                    router
                        .register_agent("doc-agent".to_string(), "Writes docs".to_string())
                        .await;
                    std::sync::Arc::new(router)
                });

                b.to_async(&rt).iter(|| {
                    let router = router.clone();
                    let tasks_clone = tasks.clone();
                    async move {
                        for task in tasks_clone {
                            let _result = router.route(black_box(&task.description)).await;
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_routing_latency,
    bench_routing_accuracy,
    bench_routing_throughput,
    bench_concurrent_routing,
    bench_task_type_routing
);
criterion_main!(benches);

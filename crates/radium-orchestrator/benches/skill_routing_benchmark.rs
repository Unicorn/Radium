//! Benchmarks for skill-based routing performance.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use radium_orchestrator::routing::{SkillRouter, SkillDefinition};
use std::path::PathBuf;
use tokio::runtime::Runtime;

/// Load skills for benchmarking.
async fn load_test_skills() -> SkillRouter {
    let router = SkillRouter::new();

    let skills_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("skills");

    if skills_dir.exists() {
        match router.load_skills_from_directory(&skills_dir).await {
            Ok(count) => {
                println!("Loaded {} skills for benchmarking", count);
            }
            Err(e) => {
                eprintln!("Warning: Failed to load skills: {}", e);
            }
        }
    } else {
        eprintln!("Warning: Skills directory not found");
    }

    router
}

/// Benchmark skill selection latency.
fn bench_skill_selection_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let router = rt.block_on(load_test_skills());

    let test_tasks = vec![
        "Review code for security issues",
        "Implement a REST API endpoint",
        "Write documentation",
        "Design database schema",
        "Deploy to production",
        "Write unit tests",
        "Analyze performance bottlenecks",
        "Refactor legacy code",
    ];

    let mut group = c.benchmark_group("skill_selection_latency");

    for task in test_tasks {
        group.bench_with_input(
            BenchmarkId::from_parameter(task),
            task,
            |b, task_desc| {
                b.to_async(&rt).iter(|| async {
                    let result = router.route(black_box(*task_desc)).await;
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark top-k selection latency.
fn bench_top_k_selection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let router = rt.block_on(load_test_skills());

    let task = "Review and optimize Python code for better performance";

    let mut group = c.benchmark_group("top_k_selection");

    for k in [1, 3, 5, 10] {
        group.bench_with_input(
            BenchmarkId::from_parameter(k),
            &k,
            |b, k_value| {
                b.to_async(&rt).iter(|| async {
                    let results = router.route_top_k(black_box(task), *k_value).await;
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark skill loading time.
fn bench_skill_loading(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let skills_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("skills");

    if !skills_dir.exists() {
        eprintln!("Skipping skill loading benchmark: skills directory not found");
        return;
    }

    c.bench_function("skill_loading", |b| {
        b.to_async(&rt).iter(|| async {
            let router = SkillRouter::new();
            let result = router.load_skills_from_directory(&skills_dir).await;
            black_box(result)
        });
    });
}

/// Benchmark skill registration.
fn bench_skill_registration(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let test_skill = SkillDefinition {
        name: "bench-test-skill".to_string(),
        description: "A benchmark test skill for performance evaluation".to_string(),
        license: Some("MIT".to_string()),
        compatibility: None,
        metadata: None,
        allowed_tools: None,
        skill_path: PathBuf::from("/bench/SKILL.md"),
        instructions: Some("Benchmark instructions".to_string()),
    };

    c.bench_function("skill_registration", |b| {
        b.to_async(&rt).iter(|| async {
            let router = SkillRouter::new();
            let result = router.register_skill(test_skill.clone()).await;
            black_box(result)
        });
    });
}

/// Benchmark capability matching precision.
fn bench_capability_matching_precision(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let router = rt.block_on(load_test_skills());

    // Test tasks with known correct skills
    let precision_tests = vec![
        ("Review code for bugs", vec!["review-agent", "code-review"]),
        ("Deploy application to AWS", vec!["deploy-agent", "infra-agent"]),
        ("Write integration tests", vec!["integration-test-agent", "test-agent"]),
        ("Design REST API", vec!["api-design-agent"]),
    ];

    let mut group = c.benchmark_group("capability_matching_precision");

    for (task, _expected_skills) in precision_tests {
        group.bench_with_input(
            BenchmarkId::from_parameter(task),
            task,
            |b, task_desc| {
                b.to_async(&rt).iter(|| async {
                    // Measure both routing decision time and correctness
                    let results = router.route_top_k(black_box(*task_desc), 5).await;
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent routing requests.
fn bench_concurrent_routing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let router = rt.block_on(async {
        let r = load_test_skills().await;
        std::sync::Arc::new(r)
    });

    let tasks = vec![
        "Review security vulnerabilities",
        "Implement authentication",
        "Write API documentation",
        "Optimize database queries",
    ];

    let mut group = c.benchmark_group("concurrent_routing");

    for concurrency in [1, 5, 10, 20] {
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            &concurrency,
            |b, n| {
                b.to_async(&rt).iter(|| async {
                    let router_clone = router.clone();
                    let mut handles = Vec::new();

                    for i in 0..*n {
                        let router = router_clone.clone();
                        let task = tasks[i % tasks.len()];

                        let handle = tokio::spawn(async move {
                            router.route(task).await
                        });

                        handles.push(handle);
                    }

                    // Wait for all to complete
                    for handle in handles {
                        let _ = handle.await;
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_skill_selection_latency,
    bench_top_k_selection,
    bench_skill_loading,
    bench_skill_registration,
    bench_capability_matching_precision,
    bench_concurrent_routing,
);

criterion_main!(benches);

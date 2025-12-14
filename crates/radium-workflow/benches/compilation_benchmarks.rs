//! Compilation Benchmarks
//!
//! Benchmarks for measuring workflow compilation performance including:
//! - JSON parsing performance
//! - Cache hit/miss performance
//! - End-to-end compilation

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use radium_workflow::performance::{
    CachedCompilation, CompilationCache, CompilationProfiler, CompilationStage, ProfileBuilder,
    WorkflowHash,
};
use radium_workflow::schema::{NodeData, NodeType, Position, WorkflowDefinition, WorkflowEdge, WorkflowNode};
use std::time::Duration;

/// Generate a workflow with N nodes for benchmarking
fn generate_workflow(num_nodes: usize) -> WorkflowDefinition {
    let mut nodes = Vec::with_capacity(num_nodes + 1);
    let mut edges = Vec::with_capacity(num_nodes);

    // Add a trigger node
    nodes.push(WorkflowNode {
        id: "trigger".to_string(),
        node_type: NodeType::Trigger,
        data: NodeData {
            label: "Start".to_string(),
            ..Default::default()
        },
        position: Position { x: 100.0, y: 100.0 },
    });

    // Add activity nodes
    for i in 0..num_nodes.saturating_sub(1) {
        nodes.push(WorkflowNode {
            id: format!("activity_{}", i),
            node_type: NodeType::Activity,
            data: NodeData {
                label: format!("Activity {}", i),
                activity_name: Some(format!("doSomething{}", i)),
                ..Default::default()
            },
            position: Position {
                x: 100.0 + (i as f64 * 150.0),
                y: 250.0,
            },
        });
    }

    // Add end node
    nodes.push(WorkflowNode {
        id: "end".to_string(),
        node_type: NodeType::End,
        data: NodeData {
            label: "End".to_string(),
            ..Default::default()
        },
        position: Position {
            x: 100.0 + (num_nodes as f64 * 150.0),
            y: 250.0,
        },
    });

    // Create edges
    edges.push(WorkflowEdge::new("edge_0", "trigger", "activity_0"));
    for i in 1..num_nodes.saturating_sub(1) {
        edges.push(WorkflowEdge::new(
            format!("edge_{}", i),
            format!("activity_{}", i - 1),
            format!("activity_{}", i),
        ));
    }
    if num_nodes > 1 {
        edges.push(WorkflowEdge::new(
            format!("edge_final"),
            format!("activity_{}", num_nodes.saturating_sub(2)),
            "end",
        ));
    }

    WorkflowDefinition {
        id: "benchmark-workflow".to_string(),
        name: format!("Benchmark Workflow with {} nodes", num_nodes),
        nodes,
        edges,
        variables: vec![],
        settings: Default::default(),
    }
}

/// Benchmark JSON serialization
fn bench_json_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialization");

    for size in [5, 10, 25, 50, 100].iter() {
        let workflow = generate_workflow(*size);
        let json = serde_json::to_string(&workflow).unwrap();

        group.throughput(Throughput::Bytes(json.len() as u64));
        group.bench_with_input(BenchmarkId::new("serialize", size), &workflow, |b, wf| {
            b.iter(|| serde_json::to_string(black_box(wf)).unwrap())
        });

        group.bench_with_input(BenchmarkId::new("deserialize", size), &json, |b, json| {
            b.iter(|| serde_json::from_str::<WorkflowDefinition>(black_box(json)).unwrap())
        });
    }

    group.finish();
}

/// Benchmark cache operations
fn bench_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_operations");

    // Cache with 1000 capacity
    let cache = CompilationCache::new(1000);

    // Pre-populate cache
    for i in 0..500 {
        let hash = WorkflowHash::new(i, "1.0.0");
        let compilation = CachedCompilation::new(format!("// code for workflow {}", i), 10);
        cache.put(hash, compilation);
    }

    group.bench_function("cache_hit", |b| {
        let hash = WorkflowHash::new(250, "1.0.0");
        b.iter(|| cache.get(black_box(&hash)))
    });

    group.bench_function("cache_miss", |b| {
        let hash = WorkflowHash::new(9999, "1.0.0");
        b.iter(|| cache.get(black_box(&hash)))
    });

    group.bench_function("cache_put_new", |b| {
        let mut i = 1000u64;
        b.iter(|| {
            let hash = WorkflowHash::new(i, "1.0.0");
            cache.put(hash, CachedCompilation::new("code".to_string(), 10));
            i += 1;
        })
    });

    group.bench_function("hash_from_json", |b| {
        let json = r#"{"id":"test","name":"Test","version":"1.0.0"}"#;
        b.iter(|| WorkflowHash::from_json(black_box(json)))
    });

    group.finish();
}

/// Benchmark profiler operations
fn bench_profiler_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("profiler_operations");

    let profiler = CompilationProfiler::new(100);

    // Pre-populate with profiles
    for i in 0..50 {
        let mut builder = ProfileBuilder::new(format!("workflow-{}", i)).with_input_size(1000);
        builder.start_stage(CompilationStage::Parsing);
        builder.end_current_stage();
        builder.start_stage(CompilationStage::CodeGeneration);
        builder.end_current_stage();
        builder.set_output_size(2000);
        profiler.record(builder.build());
    }

    group.bench_function("record_profile", |b| {
        b.iter(|| {
            let mut builder = ProfileBuilder::new("bench").with_input_size(1000);
            builder.start_stage(CompilationStage::Parsing);
            builder.end_current_stage();
            builder.set_output_size(2000);
            profiler.record(builder.build())
        })
    });

    group.bench_function("get_stats", |b| {
        b.iter(|| profiler.stats())
    });

    group.bench_function("average_stage_timings", |b| {
        b.iter(|| profiler.average_stage_timings())
    });

    group.finish();
}

/// Benchmark end-to-end workflow processing
fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");
    group.measurement_time(Duration::from_secs(10));

    for size in [5, 10, 25].iter() {
        let workflow = generate_workflow(*size);
        let json = serde_json::to_string(&workflow).unwrap();

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("parse_workflow", size),
            &json,
            |b, json| {
                b.iter(|| {
                    let _wf: WorkflowDefinition = serde_json::from_str(black_box(json)).unwrap();
                })
            },
        );
    }

    group.finish();
}

/// Benchmark memory-intensive cache operations
fn bench_cache_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_scaling");
    group.sample_size(50);

    for capacity in [100, 500, 1000, 5000].iter() {
        let cache = CompilationCache::new(*capacity);

        // Fill to capacity
        for i in 0..*capacity {
            let hash = WorkflowHash::new(i as u64, "1.0.0");
            let code = format!("// Generated code for workflow {} with some padding to simulate real code size", i);
            cache.put(hash, CachedCompilation::new(code, 10));
        }

        group.bench_with_input(
            BenchmarkId::new("eviction", capacity),
            capacity,
            |b, &cap| {
                let mut i = (cap * 2) as u64;
                b.iter(|| {
                    let hash = WorkflowHash::new(i, "1.0.0");
                    cache.put(hash, CachedCompilation::new("new code".to_string(), 10));
                    i += 1;
                })
            },
        );
    }

    group.finish();
}

/// Benchmark workflow lookup operations
fn bench_workflow_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_operations");

    for size in [10, 50, 100].iter() {
        let workflow = generate_workflow(*size);

        group.bench_with_input(
            BenchmarkId::new("find_trigger", size),
            &workflow,
            |b, wf| {
                b.iter(|| wf.find_trigger())
            },
        );

        group.bench_with_input(
            BenchmarkId::new("find_end_nodes", size),
            &workflow,
            |b, wf| {
                b.iter(|| wf.find_end_nodes())
            },
        );

        group.bench_with_input(
            BenchmarkId::new("find_node", size),
            &workflow,
            |b, wf| {
                // Find a node in the middle
                let mid = format!("activity_{}", size / 2);
                b.iter(|| wf.find_node(black_box(&mid)))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("edges_from", size),
            &workflow,
            |b, wf| {
                b.iter(|| wf.edges_from(black_box("trigger")))
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_json_serialization,
    bench_cache_operations,
    bench_profiler_operations,
    bench_end_to_end,
    bench_cache_scaling,
    bench_workflow_operations,
);

criterion_main!(benches);

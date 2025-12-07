//! Benchmarks for CLI command execution.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_command_parsing(c: &mut Criterion) {
    c.bench_function("parse_status_command", |b| {
        b.iter(|| {
            // Simulate command parsing overhead
            let args = vec!["rad", "status"];
            black_box(args);
        });
    });
}

fn bench_json_serialization(c: &mut Criterion) {
    use serde_json::json;

    c.bench_function("json_status_output", |b| {
        b.iter(|| {
            let output = json!({
                "workspace": {
                    "path": "/path/to/workspace",
                    "valid": true,
                },
                "agents": {
                    "total": 5,
                },
            });
            let serialized = serde_json::to_string_pretty(&output).unwrap();
            black_box(serialized);
        });
    });
}

criterion_group!(command_benches, bench_command_parsing, bench_json_serialization);
criterion_main!(command_benches);


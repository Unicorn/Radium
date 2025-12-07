//! Benchmarks for workspace-related commands.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;
use tempfile::TempDir;

fn bench_workspace_discovery(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path().join(".radium");
    std::fs::create_dir_all(&workspace_path).unwrap();

    c.bench_function("workspace_discovery", |b| {
        b.iter(|| {
            // Simulate workspace discovery
            let mut current = black_box(temp_dir.path());
            for _ in 0..10 {
                if current.join(".radium").exists() {
                    break;
                }
                if let Some(parent) = current.parent() {
                    current = parent;
                } else {
                    break;
                }
            }
        });
    });
}

fn bench_workspace_init(c: &mut Criterion) {
    c.bench_function("workspace_init_structure", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let radium_dir = temp_dir.path().join(".radium");
            std::fs::create_dir_all(&radium_dir).unwrap();
            std::fs::create_dir_all(&radium_dir.join("backlog")).unwrap();
            std::fs::create_dir_all(&radium_dir.join("development")).unwrap();
            std::fs::create_dir_all(&radium_dir.join("_internals")).unwrap();
            black_box(radium_dir);
        });
    });
}

criterion_group!(workspace_benches, bench_workspace_discovery, bench_workspace_init);
criterion_main!(workspace_benches);


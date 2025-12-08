//! Performance benchmarks for privacy mode functionality.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use radium_core::security::{PatternLibrary, PrivacyFilter, RedactionStyle};
use std::sync::Arc;

fn generate_test_text(size_kb: usize) -> String {
    let base_text = "Connect to 192.168.1.100 and email user@example.com. \
                     Call 555-123-4567. AWS account: 123456789012. \
                     Card: 4532015112830366. SSN: 123-45-6789. \
                     API key: sk_live_PLACEHOLDER_API_KEY_FOR_TESTING_ONLY_NOT_A_REAL_SECRET.";
    
    let mut text = String::with_capacity(size_kb * 1024);
    let repetitions = (size_kb * 1024) / base_text.len() + 1;
    for _ in 0..repetitions {
        text.push_str(base_text);
    }
    text.truncate(size_kb * 1024);
    text
}

fn benchmark_pattern_matching(c: &mut Criterion) {
    let library = PatternLibrary::default();
    
    let mut group = c.benchmark_group("pattern_matching");
    
    for size in [1, 10, 100].iter() {
        let text = generate_test_text(*size);
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}KB", size)),
            &text,
            |b, text| {
                b.iter(|| {
                    black_box(library.find_matches(black_box(text)));
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_redaction_styles(c: &mut Criterion) {
    let library = PatternLibrary::default();
    let text = generate_test_text(10); // 10KB text
    
    let mut group = c.benchmark_group("redaction_styles");
    
    let styles = vec![
        ("full", RedactionStyle::Full),
        ("partial", RedactionStyle::Partial),
        ("hash", RedactionStyle::Hash),
    ];
    
    for (name, style) in styles {
        let filter = PrivacyFilter::new(style, library.clone());
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &text,
            |b, text| {
                b.iter(|| {
                    black_box(filter.redact(black_box(text)).unwrap());
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_concurrent_redaction(c: &mut Criterion) {
    use std::thread;
    
    let library = PatternLibrary::default();
    let filter = Arc::new(PrivacyFilter::new(RedactionStyle::Partial, library));
    let text = generate_test_text(1); // 1KB text
    
    c.bench_function("concurrent_redaction_10_threads", |b| {
        b.iter(|| {
            let mut handles = vec![];
            for _ in 0..10 {
                let filter_clone = Arc::clone(&filter);
                let text_clone = text.clone();
                let handle = thread::spawn(move || {
                    black_box(filter_clone.redact(&text_clone).unwrap());
                });
                handles.push(handle);
            }
            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
}

fn benchmark_context_building_with_privacy(c: &mut Criterion) {
    use radium_core::config::Config;
    use radium_core::context::ContextManager;
    use radium_core::workspace::Workspace;
    use tempfile::TempDir;
    use std::fs;
    
    let temp_dir = TempDir::new().unwrap();
    let workspace = Workspace::create(temp_dir.path()).unwrap();
    
    // Create config with privacy enabled
    let mut config = Config::default();
    config.security.privacy.enable = true;
    config.security.privacy.redaction_style = "partial".to_string();
    
    let mut manager = ContextManager::new_with_config(&workspace, Some(&config));
    
    // Create context file with sensitive data
    let context_file = temp_dir.path().join(".radium").join("GEMINI.md");
    fs::create_dir_all(context_file.parent().unwrap()).unwrap();
    let context_content = generate_test_text(10); // 10KB
    fs::write(&context_file, &context_content).unwrap();
    
    c.bench_function("context_building_with_privacy", |b| {
        b.iter(|| {
            black_box(manager.build_context("test", None).unwrap());
        });
    });
    
    // Benchmark without privacy for comparison
    let mut config_no_privacy = Config::default();
    config_no_privacy.security.privacy.enable = false;
    let mut manager_no_privacy = ContextManager::new_with_config(&workspace, Some(&config_no_privacy));
    
    c.bench_function("context_building_without_privacy", |b| {
        b.iter(|| {
            black_box(manager_no_privacy.build_context("test", None).unwrap());
        });
    });
}

criterion_group!(
    benches,
    benchmark_pattern_matching,
    benchmark_redaction_styles,
    benchmark_concurrent_redaction,
    benchmark_context_building_with_privacy
);
criterion_main!(benches);


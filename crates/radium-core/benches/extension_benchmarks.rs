//! Performance benchmarks for extension system operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use radium_core::extensions::manifest::{ExtensionComponents, ExtensionManifest};
use radium_core::extensions::structure::MANIFEST_FILE;
use radium_core::extensions::{ExtensionDiscovery, ExtensionManager};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

fn create_test_manifest(name: &str) -> ExtensionManifest {
    ExtensionManifest {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: "Test extension".to_string(),
        author: "Test Author".to_string(),
        components: ExtensionComponents::default(),
        dependencies: Vec::new(),
        metadata: HashMap::new(),
    }
}

fn create_test_extension_package(temp_dir: &TempDir, name: &str) -> std::path::PathBuf {
    let package_dir = temp_dir.path().join(format!("package-{}", name));
    fs::create_dir_all(&package_dir).unwrap();

    let manifest_path = package_dir.join(MANIFEST_FILE);
    let manifest = create_test_manifest(name);
    let manifest_json = serde_json::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    package_dir
}

fn benchmark_manifest_parsing(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join(MANIFEST_FILE);

    let manifest_json = r#"{
        "name": "test-extension",
        "version": "1.0.0",
        "description": "Test extension description",
        "author": "Test Author",
        "components": {
            "prompts": ["prompts/*.md"],
            "commands": ["commands/*.toml"]
        },
        "dependencies": []
    }"#;

    fs::write(&manifest_path, manifest_json).unwrap();

    c.bench_function("manifest_parsing", |b| {
        b.iter(|| {
            black_box(ExtensionManifest::load(black_box(&manifest_path))).unwrap();
        });
    });
}

fn benchmark_extension_discovery(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    // Create 100 extensions
    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let options = radium_core::extensions::InstallOptions::default();

    for i in 0..100 {
        let package_path = create_test_extension_package(&temp_dir, &format!("ext{}", i));
        manager.install(&package_path, options.clone()).unwrap();
    }

    c.bench_function("discover_100_extensions", |b| {
        let discovery = ExtensionDiscovery::with_options(
            radium_core::extensions::DiscoveryOptions {
                search_paths: vec![extensions_dir.clone()],
                validate_structure: false,
            },
        );
        b.iter(|| {
            black_box(discovery.discover_all()).unwrap();
        });
    });
}

fn benchmark_extension_installation(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    // Create extension with 50 components
    let package_path = temp_dir.path().join("package-large");
    fs::create_dir_all(&package_path).unwrap();

    let manifest_path = package_path.join(MANIFEST_FILE);
    let mut manifest = create_test_manifest("large-ext");

    // Create many component files
    let prompts_dir = package_path.join("prompts");
    fs::create_dir_all(&prompts_dir).unwrap();
    for i in 0..50 {
        fs::write(prompts_dir.join(format!("agent{}.md", i)), format!("# Agent {}", i)).unwrap();
        manifest.components.prompts.push(format!("prompts/agent{}.md", i));
    }

    let manifest_json = serde_json::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    c.bench_function("install_large_extension", |b| {
        b.iter(|| {
            let manager = ExtensionManager::with_directory(extensions_dir.clone());
            let options = radium_core::extensions::InstallOptions::default();
            black_box(manager.install(black_box(&package_path), options)).unwrap();
            // Clean up for next iteration
            let _ = manager.uninstall("large-ext");
        });
    });
}

fn benchmark_dependency_graph(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let extensions_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&extensions_dir).unwrap();

    let manager = ExtensionManager::with_directory(extensions_dir.clone());
    let options = radium_core::extensions::InstallOptions::default();

    // Create extensions with dependencies
    for i in 0..50 {
        let package_path = create_test_extension_package(&temp_dir, &format!("ext{}", i));
        manager.install(&package_path, options.clone()).unwrap();
    }

    c.bench_function("build_dependency_graph_50_extensions", |b| {
        let extensions = manager.list().unwrap();
        b.iter(|| {
            use radium_core::extensions::DependencyGraph;
            black_box(DependencyGraph::from_extensions(black_box(&extensions)));
        });
    });
}

fn benchmark_signature_verification(c: &mut Criterion) {
    use radium_core::extensions::signing::{ExtensionSigner, SignatureVerifier};
    use std::path::Path;

    let temp_dir = TempDir::new().unwrap();
    let extension_path = create_test_extension_package(&temp_dir, "test-ext");

    // Generate keypair and sign
    let (signer, public_key) = ExtensionSigner::generate();
    let _signature_path = signer.sign_extension(&extension_path).unwrap();
    let verifier = SignatureVerifier::from_public_key(&public_key).unwrap();

    c.bench_function("verify_extension_signature", |b| {
        b.iter(|| {
            black_box(verifier.verify_extension(black_box(&extension_path))).unwrap();
        });
    });
}

criterion_group!(
    benches,
    benchmark_manifest_parsing,
    benchmark_extension_discovery,
    benchmark_extension_installation,
    benchmark_dependency_graph,
    benchmark_signature_verification
);
criterion_main!(benches);


---
id: "integration-guide"
title: "Extension System Integration Guide"
sidebar_label: "Extension System Integration Guide"
---

# Extension System Integration Guide

This guide shows how to integrate the Radium extension system into your own Rust projects or extend its functionality.

## Table of Contents

- [Basic Integration](#basic-integration)
- [Custom Extension Discovery](#custom-extension-discovery)
- [Marketplace Integration](#marketplace-integration)
- [Signature Verification](#signature-verification)
- [Extending the System](#extending-the-system)
- [Error Handling](#error-handling)
- [Best Practices](#best-practices)

## Basic Integration

### Installing an Extension

The simplest integration is installing an extension programmatically:

```rust
use radium_core::extensions::{ExtensionManager, InstallOptions};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create extension manager
    let manager = ExtensionManager::new()?;
    
    // Configure installation options
    let options = InstallOptions {
        overwrite: false,
        install_dependencies: true,
        validate_after_install: true,
    };
    
    // Install extension
    let extension = manager.install(Path::new("./my-extension"), options)?;
    
    println!("Installed extension: {} v{}", extension.name, extension.version);
    Ok(())
}
```

### Listing Installed Extensions

Discover and list all installed extensions:

```rust
use radium_core::extensions::ExtensionManager;

fn list_extensions() -> Result<(), Box<dyn std::error::Error>> {
    let manager = ExtensionManager::new()?;
    let extensions = manager.list()?;
    
    println!("Installed Extensions:");
    for ext in extensions {
        println!("  - {} v{}", ext.name, ext.version);
        println!("    {}", ext.description);
    }
    
    Ok(())
}
```

### Loading Extension Components

Use integration helpers to load extension components:

```rust
use radium_core::extensions::get_extension_prompt_dirs;
use std::fs;

fn load_extension_prompts() -> Result<(), Box<dyn std::error::Error>> {
    let prompt_dirs = get_extension_prompt_dirs()?;
    
    for dir in prompt_dirs {
        println!("Loading prompts from: {}", dir.display());
        
        // Read all .md files in the directory
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension() == Some(std::ffi::OsStr::new("md")) {
                let content = fs::read_to_string(&path)?;
                println!("  Found prompt: {}", path.file_name().unwrap().to_string_lossy());
                // Process prompt content...
            }
        }
    }
    
    Ok(())
}
```

## Custom Extension Discovery

### Custom Search Paths

Configure custom search paths for extension discovery:

```rust
use radium_core::extensions::{ExtensionDiscovery, DiscoveryOptions};
use std::path::PathBuf;

fn custom_discovery() -> Result<(), Box<dyn std::error::Error>> {
    let options = DiscoveryOptions {
        search_paths: vec![
            PathBuf::from("/custom/extensions/path1"),
            PathBuf::from("/custom/extensions/path2"),
            PathBuf::from(".radium/extensions"), // Project-specific
        ],
        validate_structure: true,
    };
    
    let discovery = ExtensionDiscovery::with_options(options);
    let extensions = discovery.discover_all()?;
    
    println!("Found {} extensions in custom paths", extensions.len());
    Ok(())
}
```

### Filtering Extensions

Filter extensions by criteria:

```rust
use radium_core::extensions::ExtensionDiscovery;

fn filter_extensions() -> Result<(), Box<dyn std::error::Error>> {
    let discovery = ExtensionDiscovery::new();
    let all_extensions = discovery.discover_all()?;
    
    // Filter by author
    let my_extensions: Vec<_> = all_extensions
        .into_iter()
        .filter(|ext| ext.author == "My Name")
        .collect();
    
    // Filter by version
    let recent_extensions: Vec<_> = discovery
        .discover_all()?
        .into_iter()
        .filter(|ext| {
            // Simple version check (use VersionComparator for proper comparison)
            ext.version.starts_with("2.")
        })
        .collect();
    
    Ok(())
}
```

### Searching Extensions

Implement custom search logic:

```rust
use radium_core::extensions::ExtensionDiscovery;

fn search_extensions(query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let discovery = ExtensionDiscovery::new();
    
    // Use built-in search
    let results = discovery.search(query)?;
    
    // Or implement custom search
    let all_extensions = discovery.discover_all()?;
    let custom_results: Vec<_> = all_extensions
        .into_iter()
        .filter(|ext| {
            ext.name.contains(query)
                || ext.description.to_lowercase().contains(&query.to_lowercase())
        })
        .collect();
    
    println!("Found {} matching extensions", custom_results.len());
    Ok(())
}
```

## Marketplace Integration

### Searching the Marketplace

Search for extensions in the marketplace:

```rust
use radium_core::extensions::MarketplaceClient;

fn search_marketplace(query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = MarketplaceClient::new()?;
    let results = client.search(query)?;
    
    println!("Marketplace Results:");
    for ext in results {
        println!("  {} v{}", ext.name, ext.version);
        println!("    {}", ext.description);
        if let Some(rating) = ext.rating {
            println!("    Rating: ⭐ {:.1}", rating);
        }
        if let Some(downloads) = ext.download_count {
            println!("    Downloads: {}", downloads);
        }
    }
    
    Ok(())
}
```

### Installing from Marketplace

Download and install extensions from the marketplace:

```rust
use radium_core::extensions::{MarketplaceClient, ExtensionManager, InstallOptions};

fn install_from_marketplace(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let marketplace = MarketplaceClient::new()?;
    let manager = ExtensionManager::new()?;
    
    // Get extension info
    if let Some(ext_info) = marketplace.get_extension_info(name)? {
        println!("Found extension: {}", ext_info.name);
        println!("Downloading from: {}", ext_info.download_url);
        
        // Install from URL
        let options = InstallOptions {
            overwrite: false,
            install_dependencies: true,
            validate_after_install: true,
        };
        
        let extension = manager.install_from_source(&ext_info.download_url, options)?;
        println!("Installed: {} v{}", extension.name, extension.version);
    } else {
        println!("Extension not found in marketplace");
    }
    
    Ok(())
}
```

### Browsing Popular Extensions

Browse popular extensions:

```rust
use radium_core::extensions::MarketplaceClient;

fn browse_popular() -> Result<(), Box<dyn std::error::Error>> {
    let client = MarketplaceClient::new()?;
    let popular = client.browse()?;
    
    println!("Popular Extensions:");
    for (i, ext) in popular.iter().take(10).enumerate() {
        println!("{}. {} v{}", i + 1, ext.name, ext.version);
        if let Some(rating) = ext.rating {
            println!("   ⭐ {:.1} stars", rating);
        }
    }
    
    Ok(())
}
```

## Signature Verification

### Verifying Extension Signatures

Verify extension signatures before installation:

```rust
use radium_core::extensions::{SignatureVerifier, ExtensionManager, InstallOptions};
use std::fs;
use std::path::Path;

fn install_with_verification(extension_path: &Path, public_key_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Load public key
    let public_key_bytes = fs::read(public_key_path)?;
    let verifier = SignatureVerifier::from_public_key(&public_key_bytes)?;
    
    // Verify signature
    verifier.verify_extension(extension_path)?;
    println!("Signature verified!");
    
    // Now safe to install
    let manager = ExtensionManager::new()?;
    let options = InstallOptions::default();
    let extension = manager.install(extension_path, options)?;
    
    println!("Installed verified extension: {}", extension.name);
    Ok(())
}
```

### Using Trusted Keys

Manage trusted keys for automatic verification:

```rust
use radium_core::extensions::TrustedKeysManager;
use std::fs;

fn setup_trusted_keys() -> Result<(), Box<dyn std::error::Error>> {
    let manager = TrustedKeysManager::new()?;
    
    // Add trusted publisher keys
    let publisher1_key = fs::read("publisher1-public.key")?;
    manager.add_trusted_key("Publisher One", &publisher1_key)?;
    
    let publisher2_key = fs::read("publisher2-public.key")?;
    manager.add_trusted_key("Publisher Two", &publisher2_key)?;
    
    // List trusted keys
    let trusted = manager.list_trusted_keys()?;
    println!("Trusted keys:");
    for (name, _key) in trusted {
        println!("  - {}", name);
    }
    
    Ok(())
}
```

### Signing Extensions

Sign extensions before distribution:

```rust
use radium_core::extensions::ExtensionSigner;
use std::fs;
use std::path::Path;

fn sign_extension(extension_path: &Path, private_key_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Load private key
    let private_key_bytes = fs::read(private_key_path)?;
    let signer = ExtensionSigner::from_private_key(&private_key_bytes)?;
    
    // Sign extension
    let signature_path = signer.sign_extension(extension_path)?;
    println!("Extension signed: {}", signature_path.display());
    
    Ok(())
}

fn generate_keypair() -> Result<(), Box<dyn std::error::Error>> {
    let (signer, public_key) = ExtensionSigner::generate();
    
    // Save keys
    fs::write("private.key", signer.signing_key.to_bytes())?;
    fs::write("public.key", &public_key)?;
    
    println!("Keypair generated:");
    println!("  Private key: private.key (keep secure!)");
    println!("  Public key: public.key (share with users)");
    
    Ok(())
}
```

## Extending the System

### Custom Extension Validator

Create a custom validator for extension validation:

```rust
use radium_core::extensions::{ExtensionManifest, ExtensionValidationError};
use std::path::Path;

struct CustomValidator;

impl CustomValidator {
    fn validate_custom_rules(&self, manifest: &ExtensionManifest) -> Result<(), ExtensionValidationError> {
        // Custom validation logic
        if manifest.name.len() < 3 {
            return Err(ExtensionValidationError::InvalidFormat(
                "Extension name must be at least 3 characters".to_string()
            ));
        }
        
        // Check for required metadata
        if !manifest.metadata.contains_key("category") {
            return Err(ExtensionValidationError::InvalidFormat(
                "Extension must have 'category' in metadata".to_string()
            ));
        }
        
        Ok(())
    }
}
```

### Custom Conflict Resolver

Implement custom conflict resolution (requires `workflow` feature):

```rust
#[cfg(feature = "workflow")]
use radium_core::extensions::conflict::{ConflictDetector, ConflictResolution};

#[cfg(feature = "workflow")]
fn custom_conflict_resolution() -> Result<(), Box<dyn std::error::Error>> {
    let detector = ConflictDetector::new();
    
    // Detect conflicts
    let conflicts = detector.detect_conflicts(
        &existing_extensions,
        &new_extension
    )?;
    
    if !conflicts.is_empty() {
        // Custom resolution strategy
        for conflict in conflicts {
            match conflict.resolution {
                ConflictResolution::Skip => {
                    println!("Skipping conflicting file: {}", conflict.path.display());
                }
                ConflictResolution::Overwrite => {
                    println!("Overwriting: {}", conflict.path.display());
                }
                ConflictResolution::Rename => {
                    println!("Renaming: {}", conflict.path.display());
                }
            }
        }
    }
    
    Ok(())
}
```

### Custom Extension Loader

Create a custom extension loader with caching:

```rust
use radium_core::extensions::{ExtensionDiscovery, Extension};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

struct CachedExtensionLoader {
    discovery: ExtensionDiscovery,
    cache: Arc<RwLock<HashMap<String, Extension>>>,
}

impl CachedExtensionLoader {
    fn new() -> Self {
        Self {
            discovery: ExtensionDiscovery::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    fn get_extension(&self, name: &str) -> Result<Option<Extension>, Box<dyn std::error::Error>> {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(ext) = cache.get(name) {
                return Ok(Some(ext.clone()));
            }
        }
        
        // Load from discovery
        if let Some(ext) = self.discovery.get(name)? {
            let mut cache = self.cache.write().unwrap();
            cache.insert(name.to_string(), ext.clone());
            Ok(Some(ext))
        } else {
            Ok(None)
        }
    }
    
    fn invalidate_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }
}
```

## Error Handling

### Comprehensive Error Handling

Handle all error types appropriately:

```rust
use radium_core::extensions::{
    ExtensionManager, ExtensionInstallerError, ExtensionError,
    ExtensionDiscoveryError, MarketplaceError,
};
use std::path::Path;

fn robust_installation(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let manager = ExtensionManager::new()?;
    let options = InstallOptions::default();
    
    match manager.install(path, options) {
        Ok(ext) => {
            println!("Successfully installed: {}", ext.name);
            Ok(())
        }
        Err(ExtensionInstallerError::AlreadyInstalled(name)) => {
            eprintln!("Extension {} is already installed", name);
            eprintln!("Use --overwrite flag to reinstall");
            Ok(()) // Not a fatal error
        }
        Err(ExtensionInstallerError::Dependency(msg)) => {
            eprintln!("Dependency error: {}", msg);
            eprintln!("Try installing dependencies first");
            Err(msg.into())
        }
        Err(ExtensionInstallerError::Conflict(msg)) => {
            eprintln!("Conflict detected: {}", msg);
            eprintln!("Resolve conflicts before installing");
            Err(msg.into())
        }
        Err(e) => {
            eprintln!("Installation failed: {}", e);
            Err(e.into())
        }
    }
}
```

### Retry Logic

Implement retry logic for network operations:

```rust
use radium_core::extensions::{MarketplaceClient, MarketplaceError};
use std::time::Duration;
use std::thread;

fn search_with_retry(query: &str, max_retries: u32) -> Result<Vec<MarketplaceExtension>, MarketplaceError> {
    let client = MarketplaceClient::new()?;
    
    for attempt in 1..=max_retries {
        match client.search(query) {
            Ok(results) => return Ok(results),
            Err(MarketplaceError::Timeout) | Err(MarketplaceError::Http(_)) => {
                if attempt < max_retries {
                    let delay = Duration::from_secs(2_u64.pow(attempt));
                    eprintln!("Attempt {} failed, retrying in {:?}...", attempt, delay);
                    thread::sleep(delay);
                } else {
                    return Err(MarketplaceError::Timeout);
                }
            }
            Err(e) => return Err(e),
        }
    }
    
    Err(MarketplaceError::Timeout)
}
```

## Best Practices

### 1. Always Validate Extensions

Always validate extensions before using them:

```rust
use radium_core::extensions::{ExtensionManifest, ExtensionValidator};

fn safe_extension_usage(extension_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Load and validate manifest
    let manifest = ExtensionManifest::load(&extension_path.join("radium-extension.json"))?;
    manifest.validate()?;
    
    // Use validator for additional checks
    let validator = ExtensionValidator::new();
    validator.validate_extension(&extension_path)?;
    
    // Now safe to use
    Ok(())
}
```

### 2. Handle Dependencies Properly

Always handle extension dependencies:

```rust
fn install_with_dependencies(manager: &ExtensionManager, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let options = InstallOptions {
        overwrite: false,
        install_dependencies: true, // Enable dependency resolution
        validate_after_install: true,
    };
    
    manager.install(path, options)?;
    Ok(())
}
```

### 3. Use Appropriate Error Types

Use specific error types for better error handling:

```rust
use radium_core::extensions::{
    ExtensionError, ExtensionInstallerError, ExtensionDiscoveryError,
};

fn handle_extension_errors() -> Result<(), ExtensionError> {
    // Use ExtensionError for unified error handling
    let manager = ExtensionManager::new()?;
    let extensions = manager.list()?;
    
    Ok(())
}
```

### 4. Cache Marketplace Results

Cache marketplace results to reduce API calls:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct CachedMarketplace {
    client: MarketplaceClient,
    cache: HashMap<String, (Vec<MarketplaceExtension>, Instant)>,
    ttl: Duration,
}

impl CachedMarketplace {
    fn search_cached(&mut self, query: &str) -> Result<Vec<MarketplaceExtension>, MarketplaceError> {
        if let Some((results, timestamp)) = self.cache.get(query) {
            if timestamp.elapsed() < self.ttl {
                return Ok(results.clone());
            }
        }
        
        let results = self.client.search(query)?;
        self.cache.insert(query.to_string(), (results.clone(), Instant::now()));
        Ok(results)
    }
}
```

### 5. Log Extension Operations

Log extension operations for debugging:

```rust
use log::{info, warn, error};

fn install_with_logging(manager: &ExtensionManager, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting extension installation from: {}", path.display());
    
    match manager.install(path, InstallOptions::default()) {
        Ok(ext) => {
            info!("Successfully installed extension: {} v{}", ext.name, ext.version);
            Ok(())
        }
        Err(e) => {
            error!("Installation failed: {}", e);
            Err(e.into())
        }
    }
}
```

## Complete Example: Extension Manager Service

A complete example of an extension manager service:

```rust
use radium_core::extensions::{
    ExtensionManager, ExtensionDiscovery, MarketplaceClient,
    InstallOptions, DiscoveryOptions,
};
use std::path::Path;
use std::sync::Arc;

pub struct ExtensionService {
    manager: Arc<ExtensionManager>,
    discovery: Arc<ExtensionDiscovery>,
    marketplace: Arc<MarketplaceClient>,
}

impl ExtensionService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            manager: Arc::new(ExtensionManager::new()?),
            discovery: Arc::new(ExtensionDiscovery::new()),
            marketplace: Arc::new(MarketplaceClient::new()?),
        })
    }
    
    pub fn install_local(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let options = InstallOptions {
            overwrite: false,
            install_dependencies: true,
            validate_after_install: true,
        };
        
        self.manager.install(path, options)?;
        Ok(())
    }
    
    pub fn install_from_marketplace(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ext_info) = self.marketplace.get_extension_info(name)? {
            let options = InstallOptions {
                overwrite: false,
                install_dependencies: true,
                validate_after_install: true,
            };
            
            self.manager.install_from_source(&ext_info.download_url, options)?;
        }
        
        Ok(())
    }
    
    pub fn list_installed(&self) -> Result<Vec<Extension>, Box<dyn std::error::Error>> {
        Ok(self.manager.list()?)
    }
    
    pub fn search_local(&self, query: &str) -> Result<Vec<Extension>, Box<dyn std::error::Error>> {
        Ok(self.discovery.search(query)?)
    }
    
    pub fn search_marketplace(&self, query: &str) -> Result<Vec<MarketplaceExtension>, Box<dyn std::error::Error>> {
        Ok(self.marketplace.search(query)?)
    }
}
```

## Next Steps

- [API Reference](api-reference.md) - Complete API documentation
- [Architecture](architecture.md) - System architecture details
- [User Guide](user-guide.md) - User-facing documentation

